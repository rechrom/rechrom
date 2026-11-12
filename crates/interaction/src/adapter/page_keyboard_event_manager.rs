//! Live Page assembly for Blink's retained `KeyboardEventManager`.
//!
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: third_party/blink/renderer/core/input/keyboard_event_manager.h:28-86
//! Source: third_party/blink/renderer/core/input/keyboard_event_manager.cc:203-699
//!
//! Event creation, keydown/keypress ordering, editor/default dispatch, focus
//! navigation and scroll completion stay in the translated manager. These
//! projections retain canonical DOM/EventTarget owners and borrow live facts
//! for each operation. No DOM borrow survives dispatch, focus or host callbacks.

use std::{cell::RefCell, rc::Rc};

#[path = "page_keyboard_caps_lock.rs"]
mod caps_lock;
pub use caps_lock::UpdatePasswordCapsLockIndicator;
#[path = "page_editor_key_bindings.rs"]
mod editing_keyboard;
#[path = "page_keyboard_keycodes.rs"]
mod keycodes;

use dom::UserInteractionState;
use interaction_default::core::dom::focus_params::FocusParams;
use interaction_dispatch::core::{
    dom::events::{
        event::{EventRef, EventTargetRef},
        event_dispatch_result::DispatchEventResult,
        simulated_click_options::SimulatedClickCreationScope,
    },
    events::{
        keyboard_event::{KeyboardEventKeycodeConverter, KeyboardEventRef, KeyboardEventWindow},
        ui_event::{AbstractViewRef, InputDeviceCapabilitiesRef},
    },
};
use interaction_input::public::common::input::web_keyboard_event::WebKeyboardEvent;
use interaction_scroll::core::input::scroll_manager::ScrollEffectEmitter;
use interaction_scroll::core::input::scroll_manager::{ScrollManager, ScrollNodeRef};

use super::{
    dom_event::DomEventTargets,
    event::EventTargetResolver,
    input_event::KeyEvent,
    interaction_contract::{InteractionDocument, ReadNode},
    page_event_dispatcher::{DispatchEventResult as PageDispatchEventResult, EventDispatcher},
    page_scroll_manager::PageScrollManagerOwners,
};
use crate::core::input::keyboard_event_manager::{
    KeyboardAccessKeyElement, KeyboardDocument, KeyboardEventManager, KeyboardEventManagerState,
    KeyboardNode, KeyboardStringOperations,
};

/// Assembly ports retain the existing frame/editor/page controllers. A frame's
/// `GetDocument` must resolve its current document after every synchronous
/// callback; its editor and page ports invoke their source owners immediately.
/// In particular, the compatibility text editor is not a replacement for
/// `Editor::HandleKeyboardEvent` or SpatialNavigationController.
pub use crate::core::input::keyboard_event_manager::{
    KeyboardDisplayMode, KeyboardEditor, KeyboardFrame, KeyboardManagerKeycodeConverter,
    KeyboardPage, KeyboardSpatialNavigation,
};

/// Source KeycodeConverter with the current host event's exact string pair.
///
/// The host boundary already carries the standardized UI Events `key` and
/// `code` strings. `KeyEvent::ToWebEvent` converts those strings with
/// Chromium's dom_code_data/dom_key_data values; this owner retains the exact
/// inverse pair for the same WebKeyboardEvent. The source mapping also handles
/// keys outside that event, so a mouse/gesture binding needs no fabricated key.
// Source: ui/events/keycodes/dom/keycode_converter.cc:319-345,429-474.
#[derive(Default)]
pub struct PageKeyboardKeycodeConverter {
    dom_code: i32,
    code: String,
    dom_key: u32,
    key: String,
}

impl PageKeyboardKeycodeConverter {
    pub fn new(host: &crate::input_event::KeyEvent, web: &WebKeyboardEvent) -> Rc<Self> {
        Rc::new(Self {
            dom_code: web.dom_code,
            code: host.code.clone(),
            dom_key: web.dom_key,
            key: host.key.clone(),
        })
    }
}

impl KeyboardEventKeycodeConverter for PageKeyboardKeycodeConverter {
    fn DomCodeToCodeString(&self, dom_code: i32) -> String {
        if dom_code != 0 && dom_code == self.dom_code {
            self.code.clone()
        } else {
            keycodes::DomCodeToCodeString(dom_code)
        }
    }

    fn DomKeyToKeyString(&self, dom_key: u32) -> String {
        if dom_key != 0 && dom_key == self.dom_key {
            self.key.clone()
        } else {
            keycodes::DomKeyToKeyString(dom_key)
        }
    }
}

impl KeyboardManagerKeycodeConverter for PageKeyboardKeycodeConverter {
    fn IsDomKeyForModifier(&self, dom_key: u32) -> bool {
        const SYMBOLIC: u32 = 1 << 21;
        matches!(
            dom_key,
            value if value == SYMBOLIC | 0x0101 // Accel
                || value == SYMBOLIC | 0x0102 // Alt
                || value == SYMBOLIC | 0x0103 // AltGraph
                || value == SYMBOLIC | 0x0104 // CapsLock
                || value == SYMBOLIC | 0x0105 // Control
                || value == SYMBOLIC | 0x0106 // Fn
                || value == SYMBOLIC | 0x0107 // FnLock
                || value == SYMBOLIC | 0x0108 // Hyper
                || value == SYMBOLIC | 0x0109 // Meta
                || value == SYMBOLIC | 0x010a // NumLock
                || value == SYMBOLIC | 0x010c // ScrollLock
                || value == SYMBOLIC | 0x010d // Shift
                || value == SYMBOLIC | 0x010e // Super
                || value == SYMBOLIC | 0x010f // Symbol
                || value == SYMBOLIC | 0x0110 // SymbolLock
                || value == SYMBOLIC | 0x0111 // ShiftLevel5
        )
    }

    fn IsEscapeDomKey(&self, dom_key: u32) -> bool {
        const CODEPOINT: u32 = 1 << 22;
        dom_key == CODEPOINT | 0x001b
    }

    fn EventKeycodeConverter(&self) -> &dyn KeyboardEventKeycodeConverter {
        self
    }
}

/// The native scroll owner resolves the same canonical node used by wheel,
/// mouse and keyboard managers. A DOM node without a LayoutObject still needs
/// a node handle whose `Facts().has_layout_object` is false: returning the
/// viewport here would bypass ScrollManager's focused/mouse/default ordering.
pub trait PageKeyboardScrollNodes {
    fn ScrollNode(&self, target: &EventTargetRef) -> ScrollNodeRef;
}

/// Binds keyboard target identity to the same resident ScrollManager owner
/// used by wheel and logical scroll. It retains neither DOM nodes nor layout
/// cursors; each key transaction resolves the current target and LayoutObject.
pub struct PageKeyboardScrollNodesForPage {
    targets: Rc<DomEventTargets>,
    scroll: Rc<PageScrollManagerOwners>,
}

impl PageKeyboardScrollNodesForPage {
    pub fn new(targets: Rc<DomEventTargets>, scroll: Rc<PageScrollManagerOwners>) -> Rc<Self> {
        Rc::new(Self { targets, scroll })
    }
}

impl PageKeyboardScrollNodes for PageKeyboardScrollNodesForPage {
    fn ScrollNode(&self, target: &EventTargetRef) -> ScrollNodeRef {
        self.scroll.KeyboardScrollNode(
            self.targets
                .NodeId(target)
                .expect("keyboard target belongs to the current Page document"),
        )
    }
}

/// Source operations whose owners are outside this DOM projection.
///
/// Dispatch must pass the supplied canonical EventRef through the existing
/// synchronous dispatcher and default handlers. Focus/access-key actions must
/// likewise finish before returning. Implementations enter their current
/// call-scoped dispatch/editor/lifecycle binding; they must not retain borrowed
/// Layout geometry or defer these operations beyond listener reentry.
///
/// Access-key lookup belongs to TreeScope (tree_scope.cc:880-896), including
/// shadow-root traversal and last-match order. The adapter must not replace it
/// with a new element scan. PluginNode is the actual PluginDocument owner query,
/// returning None for documents without such an owner.
pub trait PageKeyboardDocumentServices {
    fn DispatchEvent(&self, target: &EventTargetRef, event: EventRef) -> DispatchEventResult;
    fn Focus(&self, element: &EventTargetRef, params: &FocusParams);
    fn AccessKeyAction(&self, element: &EventTargetRef, scope: SimulatedClickCreationScope);
    /// Node::UpdateHadKeyboardEvent effects after the Document bit changes:
    /// resident LayoutObject effective-appearance invalidation and an Element's
    /// FocusVisibleStateChanged, only when it has a LayoutObject.
    fn HadKeyboardEventChanged(&self, target: &EventTargetRef);
    fn FrameHasComposition(&self) -> Option<bool>;
    fn InputCapsLockStateMayHaveChanged(&self, element: &EventTargetRef);
}

/// One Document/Window binding. A replacement Document gets a new binding;
/// LocalFrame's current-document getter chooses it at each source call site.
/// The targets must be the exact registry used by the production dispatcher.
pub struct PageKeyboardDocument<'scope> {
    document: InteractionDocument,
    state: Rc<RefCell<UserInteractionState>>,
    targets: Rc<DomEventTargets>,
    services: Rc<dyn PageKeyboardDocumentServices + 'scope>,
    scroll: Rc<dyn PageKeyboardScrollNodes>,
}

impl<'scope> PageKeyboardDocument<'scope> {
    pub fn new(
        state: Rc<RefCell<UserInteractionState>>,
        targets: Rc<DomEventTargets>,
        services: Rc<dyn PageKeyboardDocumentServices + 'scope>,
        scroll: Rc<dyn PageKeyboardScrollNodes>,
    ) -> Rc<Self> {
        Rc::new(Self {
            document: targets.DocumentOwner(),
            state,
            targets,
            services,
            scroll,
        })
    }

    /// Project an existing target; never resolve an old target by numeric ID in
    /// a different Document. This is also the Node port for default handling.
    pub fn Node(&self, target: EventTargetRef) -> Option<Rc<dyn KeyboardNode + 'scope>> {
        let id = self.targets.NodeId(&target)?;
        let owner = self.targets.OwnerDocumentHandle(&target)?;
        let document = self.document.borrow();
        if id == 0 || document.GetDocument().RootHandle() != owner {
            return None;
        }
        document.GetDocument().FindNodeById(id)?;
        drop(document);
        Some(Rc::new(PageKeyboardNode {
            document: self.document.clone(),
            target,
            id,
            services: self.services.clone(),
            scroll: self.scroll.clone(),
        }))
    }
}

// Source: keyboard_event_manager.cc:321-323,335-336,405-406,603-606;
// keyboard_event.cc:72-79,107-114. WindowView and capabilities retain the
// dispatch registry's original LocalDOMWindow identity, including reuse.
impl KeyboardEventWindow for PageKeyboardDocument<'_> {
    fn View(&self) -> AbstractViewRef {
        self.targets.WindowView()
    }
    fn FrameHasComposition(&self) -> Option<bool> {
        self.services.FrameHasComposition()
    }
    fn FiresTouchEvents(&self, fires_touch_events: bool) -> InputDeviceCapabilitiesRef {
        self.targets.InputDeviceCapabilities(fires_touch_events)
    }
}

impl<'scope> KeyboardDocument<'scope> for PageKeyboardDocument<'scope> {
    // Source: keyboard_event_manager.cc:246,381; document.cc:8913-8926.
    fn EventTargetNodeForDocument(&self) -> Option<Rc<dyn KeyboardNode + 'scope>> {
        self.Node(
            self.targets
                .EventTargetNodeForDocument(self.state.borrow().focused_node_id)?,
        )
    }

    // Source: keyboard_event_manager.cc:214-222.
    fn GetElementByAccessKey(
        &self,
        lowered_key: &[u16],
    ) -> Option<Rc<dyn KeyboardAccessKeyElement + 'scope>> {
        let element = {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            let element = document.GetElementByAccessKey(document.Root(), lowered_key)?;
            self.targets
                .TargetFromBorrowedDocument(document, document.Node(element).Id())?
        };
        self.Node(element.clone())?;
        Some(Rc::new(PageKeyboardAccessKeyElement {
            element,
            services: self.services.clone(),
        }))
    }
    fn DomWindow(&self) -> &dyn KeyboardEventWindow {
        self
    }
    fn IsInWebAppScope(&self) -> bool {
        // Rechrom currently has no installed-web-app manifest/scope owner.
        false
    }
    fn InDesignMode(&self) -> bool {
        self.document.borrow().GetDocument().InDesignMode()
    }
    // Source: keyboard_event_manager.cc:419-424. The guard is HTMLInputElement,
    // including non-text inputs, rather than ToTextControlOrNull.
    fn FocusedInputCapsLockStateMayHaveChanged(&self) {
        let focused = self.state.borrow().focused_node_id;
        let Some(id) = focused else { return };
        if ReadNode(&self.document, id, |document, _, index| {
            document.InputType(index).is_some()
        }) != Some(true)
        {
            return;
        }
        if let Some(target) = self.targets.ResolveTarget(id) {
            self.services.InputCapsLockStateMayHaveChanged(&target);
        }
    }
    fn LoseInterestInAllElements(&self) {
        // Element interest invokers are not admitted by PersistentDocument,
        // so Document::ElementsWithInterest is structurally empty.
    }
}

struct PageKeyboardNode<'scope> {
    document: InteractionDocument,
    target: EventTargetRef,
    id: u64,
    services: Rc<dyn PageKeyboardDocumentServices + 'scope>,
    scroll: Rc<dyn PageKeyboardScrollNodes>,
}

impl KeyboardNode for PageKeyboardNode<'_> {
    fn EventTarget(&self) -> EventTargetRef {
        self.target.clone()
    }
    // Source: keyboard_event_manager.cc:326-327,361-363,410-411.
    fn DispatchEvent(&self, event: &KeyboardEventRef) -> DispatchEventResult {
        self.services
            .DispatchEvent(&self.target, EventRef::FromShared(event.clone()))
    }
    // Source: keyboard_event_manager.cc:354-357; node.cc:3472-3488.
    fn UpdateHadKeyboardEvent(&self, _event: &KeyboardEventRef) {
        let was_keyboard = self
            .document
            .borrow_mut()
            .GetDocumentMut()
            .SetHadKeyboardEvent(true);
        if !was_keyboard {
            self.services.HadKeyboardEventChanged(&self.target);
        }
    }
    // Source: keyboard_event_manager.cc:299-307. IsEditable is the DOM-owned
    // translated editing predicate; controls consume normalized InputType.
    fn IsEditable(&self) -> bool {
        ReadNode(&self.document, self.id, |document, _, index| {
            document.IsEditableForEditing(index)
        })
        .unwrap_or(false)
    }
    fn TextControlIsDisabledOrReadOnly(&self) -> Option<bool> {
        ReadNode(&self.document, self.id, |document, node, index| {
            (node.IsHTMLElement("textarea")
                || document
                    .InputType(index)
                    .is_some_and(|kind| kind.IsTextFieldInputType()))
            .then(|| {
                document.IsDisabledFormControl(index) || node.FindAttribute("readonly").is_some()
            })
        })
        .flatten()
    }
    fn RoleAttribute(&self) -> Option<String> {
        ReadNode(&self.document, self.id, |_, node, _| {
            node.FindAttribute("role")
                .map(|attribute| attribute.value.clone())
        })
        .flatten()
    }
    fn ScrollNode(&self) -> ScrollNodeRef {
        self.scroll.ScrollNode(&self.target)
    }
}

struct PageKeyboardAccessKeyElement<'scope> {
    element: EventTargetRef,
    services: Rc<dyn PageKeyboardDocumentServices + 'scope>,
}
impl KeyboardAccessKeyElement for PageKeyboardAccessKeyElement<'_> {
    fn Focus(&self, params: &FocusParams) {
        self.services.Focus(&self.element, params)
    }
    fn AccessKeyAction(&self, scope: SimulatedClickCreationScope) {
        self.services.AccessKeyAction(&self.element, scope)
    }
}

struct PageKeyboardStrings;
impl KeyboardStringOperations for PageKeyboardStrings {
    // Source: keyboard_event_manager.cc:214-216; foundation's ICU translation
    // preserves expanding/contextual mappings and unpaired UTF-16 surrogates.
    fn DeprecatedLower(&self, text: &[u16]) -> Vec<u16> {
        foundation::String::from_utf16(text)
            .DeprecatedLower()
            .Span16()
            .unwrap_or_default()
            .to_vec()
    }
}

/// Assemble once per EventHandler using its original retained state and the
/// exact ScrollManager Rc from PageScrollManagerOwners. All source services
/// are required: no missing editor, focus, activation, close-watcher, keycode
/// conversion or spatial-navigation owner silently becomes placeholder behavior.
/// The caller retains this Rc and exposes its Weak alias to scroll services,
/// avoiding a KeyboardEventManager -> ScrollManager -> frame -> manager cycle.
// Source: keyboard_event_manager.h:39-46,75-86; .cc:193-201,203-699.
pub fn CreatePageKeyboardEventManager<'scope>(
    frame: Rc<dyn KeyboardFrame<'scope> + 'scope>,
    scroll: Rc<ScrollManager>,
    keycodes: Rc<dyn KeyboardManagerKeycodeConverter>,
    state: Rc<KeyboardEventManagerState>,
) -> Rc<KeyboardEventManager<'scope>> {
    Rc::new(KeyboardEventManager::WithState(
        frame,
        scroll,
        keycodes,
        Rc::new(PageKeyboardStrings),
        state,
    ))
}

struct PageKeyboardDocumentServicesForDispatch<'listener, 'frame> {
    dispatcher: EventDispatcher<'listener, 'frame>,
    last_result: Rc<RefCell<PageDispatchEventResult>>,
    error: Rc<RefCell<Option<&'static str>>>,
}

impl PageKeyboardDocumentServices for PageKeyboardDocumentServicesForDispatch<'_, '_> {
    fn DispatchEvent(&self, target: &EventTargetRef, event: EventRef) -> DispatchEventResult {
        let (native, result) = self
            .dispatcher
            .DispatchCanonicalKeyboardEvent(target, event);
        *self.last_result.borrow_mut() = result;
        native
    }

    fn Focus(&self, element: &EventTargetRef, params: &FocusParams) {
        self.dispatcher.FocusKeyboardTarget(element, params)
    }

    fn AccessKeyAction(&self, element: &EventTargetRef, scope: SimulatedClickCreationScope) {
        // Keyboard access keys use kFromUserAgent; the Page clock supplies
        // the sole required construction input when no underlying event exists.
        let _ = self.dispatcher.DispatchAccessKeyAction(element, scope);
    }

    fn HadKeyboardEventChanged(&self, target: &EventTargetRef) {
        self.dispatcher.KeyboardHadEventChanged(target)
    }

    fn FrameHasComposition(&self) -> Option<bool> {
        self.dispatcher.KeyboardFrameHasComposition()
    }

    fn InputCapsLockStateMayHaveChanged(&self, element: &EventTargetRef) {
        if let Err(error) = caps_lock::CapsLockStateMayHaveChanged(&self.dispatcher, element) {
            self.error.borrow_mut().get_or_insert(error);
        }
    }
}

/// LocalFrame's call-scoped Editor edge. Native key-binding IDs and fallback
/// EditingBehavior commands use the existing canonical EditorCommand table.
/// Other commands continue through their existing default-action owners.
struct PageKeyboardEditor<'listener, 'frame> {
    dispatcher: EventDispatcher<'listener, 'frame>,
    edit_commands: RefCell<
        Vec<interaction_default::core::editing::commands::editor_command_table::EditingCommandType>,
    >,
    error: Rc<RefCell<Option<&'static str>>>,
}
impl<'listener: 'frame, 'frame> KeyboardEditor for PageKeyboardEditor<'listener, 'frame> {
    fn HandleKeyboardEvent(&self, event: &KeyboardEventRef) {
        self.HandleSourceKeyboardEvent(event)
    }
}

struct PageKeyboardPage {
    focus: Option<Rc<super::page_focus_navigation::PageFocusNavigation>>,
}
impl<'scope> KeyboardPage<'scope> for PageKeyboardPage {
    fn IsFocusedOrMainFrame(&self, _frame: &dyn KeyboardFrame<'scope>) -> bool {
        self.focus.as_ref().is_some_and(|focus| {
            focus
                .Controller()
                .FocusedOrMainFrame()
                .is_some_and(|current| Rc::ptr_eq(&current, &focus.Frame()))
        })
    }
    fn TabKeyCyclesThroughElements(&self) -> bool {
        true
    }
    fn AdvanceFocus(
        &self,
        _focus_type: interaction_default::public::mojom::input::focus_type::FocusType,
        _capabilities: InputDeviceCapabilitiesRef,
    ) -> bool {
        // Production keyboard dispatch always calls AdvanceFocusWithScope so
        // focus/blur events receive a fresh canonical Event projection.
        false
    }
    fn AdvanceFocusWithScope(
        &self,
        focus_type: interaction_default::public::mojom::input::focus_type::FocusType,
        capabilities: InputDeviceCapabilitiesRef,
        scope: &interaction_default::core::page::focus_controller::FocusDispatchScope<'_>,
    ) -> bool {
        self.focus
            .as_ref()
            .is_some_and(|focus| focus.AdvanceFocus(focus_type, capabilities, scope))
    }
    fn SpatialNavigationController(&self) -> &dyn KeyboardSpatialNavigation {
        // Source keyboard_event_manager.cc:534-541,623-625,638-640,651-653
        // requests this owner only behind IsSpatialNavigationEnabled(). The
        // embedding does not enable that setting until it supplies the owner.
        panic!("spatial navigation requires its source controller")
    }
}

struct PageKeyboardFrameForDispatch<'listener, 'frame> {
    document: Rc<PageKeyboardDocument<'frame>>,
    page: Rc<PageKeyboardPage>,
    editor: PageKeyboardEditor<'listener, 'frame>,
    effects: ScrollEffectEmitter,
}
impl<'listener: 'frame, 'frame> KeyboardFrame<'frame>
    for PageKeyboardFrameForDispatch<'listener, 'frame>
{
    fn GetDocument(&self) -> Rc<dyn KeyboardDocument<'frame> + 'frame> {
        self.document.clone()
    }
    fn GetPage(&self) -> Option<Rc<dyn KeyboardPage<'frame> + 'frame>> {
        Some(self.page.clone())
    }
    fn GetEditor(&self) -> &dyn KeyboardEditor {
        &self.editor
    }
    fn NotifyInteractionUserActivation(&self) {
        // Source keyboard_event_manager.cc:244-260. Pointer and keyboard input
        // activate the same LocalFrame user-activation owner before listeners.
        let dispatcher = &self.editor.dispatcher;
        dispatcher
            .input_frame
            .as_ref()
            .expect("keyboard input requires its LocalFrame activation owner")
            .activation
            .borrow_mut()
            .Activate(
                interaction_input::public::common::frame::user_activation_state::UserActivationNotificationType::kInteraction,
                dispatcher.EventClock().Now(),
            );
        super::page_event_dispatcher::pointer_dom_effects::SetHadUserInteraction(dispatcher, true);
    }
    fn DontSendKeyEventsToJavascript(&self) -> bool {
        false
    }
    fn DisplayMode(&self) -> KeyboardDisplayMode {
        KeyboardDisplayMode::Browser
    }
    fn IsSpatialNavigationEnabled(&self) -> bool {
        false
    }
    fn HandleFocusgroupKeyboardEvent(&self, event: &KeyboardEventRef) -> bool {
        self.page.focus.as_ref().is_some_and(|focus| {
            focus.HandleFocusgroupKeyboardEvent(&self.editor.dispatcher, event)
        })
    }
    fn CloseWatcherEscapeKeyHandler(&self, event: &KeyboardEventRef) {
        super::page_event_dispatcher::pointer_dom_effects::CloseWatcherEscapeKeyHandler(
            &self.editor.dispatcher,
            event,
        )
        .unwrap_or_else(|error| std::panic::panic_any(error));
    }
    fn MacDisableCtrlHomeEndEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::MacDisableCtrlHomeEndEnabled()
    }
    fn MiddleClickAutoscrollEnabled(&self) -> bool {
        false
    }
    fn ScrollEffects(&self) -> ScrollEffectEmitter {
        self.effects.clone()
    }
}

struct KeyboardDefaultBinding<'dispatcher, 'listener, 'frame> {
    dispatcher: &'dispatcher EventDispatcher<'listener, 'frame>,
    previous: Option<Rc<dyn Fn(&KeyboardEventRef, u64) + 'frame>>,
}

/// Call-scoped EventHandler edge over the original retained keyboard manager.
/// It keeps the canonical document projection needed for mouse-press fallback;
/// key dispatch and defaults stay in KeyboardEventManager.
pub struct PageEventHandlerKeyboard<'listener, 'frame> {
    dispatcher: EventDispatcher<'listener, 'frame>,
    manager: Rc<KeyboardEventManager<'frame>>,
    document: Rc<PageKeyboardDocument<'frame>>,
    nodes: Rc<super::page_event_handler_node::PageEventHandlerNodes>,
    mouse: Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
    result: Rc<RefCell<PageDispatchEventResult>>,
    error: Rc<RefCell<Option<&'static str>>>,
}

impl<'listener: 'frame, 'frame> PageEventHandlerKeyboard<'listener, 'frame> {
    pub fn new(
        dispatcher: &EventDispatcher<'listener, 'frame>,
        current_key: Option<(&KeyEvent, &WebKeyboardEvent)>,
        scroll: Rc<PageScrollManagerOwners>,
        owners: &super::source_input_managers::ResidentCursorOwners,
        nodes: Rc<super::page_event_handler_node::PageEventHandlerNodes>,
        focus: Option<Rc<super::page_focus_navigation::PageFocusNavigation>>,
    ) -> Rc<Self> {
        let targets = dispatcher.KeyboardTargets();
        assert!(Rc::ptr_eq(&targets, nodes.Targets()));
        let result = Rc::new(RefCell::new(PageDispatchEventResult::default()));
        let error = Rc::new(RefCell::new(None));
        let services = Rc::new(PageKeyboardDocumentServicesForDispatch {
            dispatcher: dispatcher.clone(),
            last_result: result.clone(),
            error: error.clone(),
        });
        let scroll_nodes = PageKeyboardScrollNodesForPage::new(targets.clone(), scroll.clone());
        let document =
            PageKeyboardDocument::new(dispatcher.KeyboardState(), targets, services, scroll_nodes);
        let frame = Rc::new(PageKeyboardFrameForDispatch {
            document: document.clone(),
            page: Rc::new(PageKeyboardPage { focus }),
            editor: PageKeyboardEditor {
                dispatcher: dispatcher.clone(),
                edit_commands: RefCell::new(
                    current_key
                        .map(|(host, _)| host.edit_commands.clone())
                        .unwrap_or_default(),
                ),
                error: error.clone(),
            },
            effects: scroll.ScrollEffects(),
        });
        let manager = CreatePageKeyboardEventManager(
            frame,
            scroll.ScrollManager(),
            current_key
                .map(|(host, web)| PageKeyboardKeycodeConverter::new(host, web))
                .unwrap_or_else(|| Rc::new(PageKeyboardKeycodeConverter::default())),
            owners.keyboard.clone(),
        );
        Rc::new(Self {
            dispatcher: dispatcher.clone(),
            manager,
            document,
            nodes,
            mouse: owners.mouse.clone(),
            result,
            error,
        })
    }

    pub fn TakeDispatchResult(&self) -> PageDispatchEventResult {
        std::mem::take(&mut *self.result.borrow_mut())
    }
    pub fn Finish(&self) -> Result<(), &'static str> {
        self.error.borrow_mut().take().map_or(Ok(()), Err)
    }
}

impl<'listener: 'frame, 'frame> crate::core::input::event_handler::EventHandlerKeyboard
    for PageEventHandlerKeyboard<'listener, 'frame>
{
    fn IsHandlingKeyEvent(&self) -> bool {
        self.manager.is_handling_key_event()
    }
    fn HandleAccessKey(&self, event: &WebKeyboardEvent) -> bool {
        self.manager.HandleAccessKey(event)
    }
    fn KeyEvent(
        &self,
        event: &WebKeyboardEvent,
    ) -> interaction_dispatch::public::platform::web_input_event_result::WebInputEventResult {
        let manager = self.manager.clone();
        let document = self.document.clone();
        let mouse = self.mouse.clone();
        let dispatcher = self.dispatcher.clone();
        let previous = self.dispatcher.KeyboardDefaultHandler();
        self.dispatcher
            .SetKeyboardDefaultHandler(Some(Rc::new(move |event, _| {
                let possible = mouse
                    .MousePressNode()
                    .and_then(|target| document.Node(target));
                manager.DefaultKeyboardEventHandlerWithFocusScope(
                    event,
                    possible.as_deref(),
                    &interaction_default::core::page::focus_controller::FocusDispatchScope {
                        focus_events: Some(&dispatcher),
                        ..Default::default()
                    },
                );
            })));
        let _binding = KeyboardDefaultBinding {
            dispatcher: &self.dispatcher,
            previous,
        };
        self.manager.KeyEvent(event)
    }
    fn DefaultKeyboardEventHandler(
        &self,
        event: &KeyboardEventRef,
        mouse_press_node: Option<crate::core::input::event_handler::NodeRef>,
    ) {
        let possible = mouse_press_node
            .and_then(|node| self.nodes.EventTarget(&node))
            .and_then(|target| self.document.Node(target));
        self.manager.DefaultKeyboardEventHandlerWithFocusScope(
            event,
            possible.as_deref(),
            &interaction_default::core::page::focus_controller::FocusDispatchScope {
                focus_events: Some(&self.dispatcher),
                ..Default::default()
            },
        );
    }
    fn DefaultTabEventHandler(&self, event: &KeyboardEventRef) -> bool {
        self.manager.DefaultTabEventHandlerWithFocusScope(
            event,
            &interaction_default::core::page::focus_controller::FocusDispatchScope {
                focus_events: Some(&self.dispatcher),
                ..Default::default()
            },
        )
    }
    fn CapsLockStateMayHaveChanged(&self) {
        self.manager.CapsLockStateMayHaveChanged();
    }
}
impl Drop for KeyboardDefaultBinding<'_, '_, '_> {
    fn drop(&mut self) {
        self.dispatcher
            .SetKeyboardDefaultHandler(self.previous.take())
    }
}

/// Runs one host key through Blink's KeyboardEventManager and the canonical
/// DOM dispatcher. The manager, document and Node default edge are call-scoped;
/// only KeyboardEventManagerState and ScrollManager survive the transaction.
// Source: event_handler.cc:2463-2471; keyboard_event_manager.cc:226-417.
pub(crate) fn DispatchPageKeyboardEvent<'listener: 'frame, 'frame>(
    dispatcher: &EventDispatcher<'listener, 'frame>,
    host: &KeyEvent,
    web: &WebKeyboardEvent,
    scroll: Rc<PageScrollManagerOwners>,
    state: Rc<KeyboardEventManagerState>,
    mouse: Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
    focus: Option<Rc<super::page_focus_navigation::PageFocusNavigation>>,
) -> PageDispatchEventResult {
    let targets = dispatcher.KeyboardTargets();
    let result = Rc::new(RefCell::new(PageDispatchEventResult::default()));
    let error = Rc::new(RefCell::new(None));
    let services = Rc::new(PageKeyboardDocumentServicesForDispatch {
        dispatcher: dispatcher.clone(),
        last_result: result.clone(),
        error: error.clone(),
    });
    let scroll_nodes = PageKeyboardScrollNodesForPage::new(targets.clone(), scroll.clone());
    let document =
        PageKeyboardDocument::new(dispatcher.KeyboardState(), targets, services, scroll_nodes);
    let frame = Rc::new(PageKeyboardFrameForDispatch {
        document: document.clone(),
        page: Rc::new(PageKeyboardPage { focus }),
        editor: PageKeyboardEditor {
            dispatcher: dispatcher.clone(),
            edit_commands: RefCell::new(host.edit_commands.clone()),
            error: error.clone(),
        },
        effects: scroll.ScrollEffects(),
    });
    let keycodes = PageKeyboardKeycodeConverter::new(host, web);
    let manager = CreatePageKeyboardEventManager(frame, scroll.ScrollManager(), keycodes, state);
    let default_manager = manager.clone();
    let default_document = document.clone();
    let focus_dispatcher = dispatcher.clone();
    let previous = dispatcher.KeyboardDefaultHandler();
    dispatcher.SetKeyboardDefaultHandler(Some(Rc::new(move |event, _raw_target| {
        let possible = mouse
            .MousePressNode()
            .and_then(|target| default_document.Node(target));
        default_manager.DefaultKeyboardEventHandlerWithFocusScope(
            event,
            possible.as_deref(),
            &interaction_default::core::page::focus_controller::FocusDispatchScope {
                focus_events: Some(&focus_dispatcher),
                ..Default::default()
            },
        );
    })));
    let _default_binding = KeyboardDefaultBinding {
        dispatcher,
        previous,
    };
    manager.KeyEvent(web);
    if let Some(error) = error.borrow_mut().take() {
        panic!("required keyboard port: {error}");
    }
    let dispatch_result = *result.borrow();
    dispatch_result
}
