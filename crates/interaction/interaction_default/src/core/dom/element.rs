// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/element.h:231-235
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionBehaviorOnFocus {
    kReset,
    kRestore,
    kNone,
}

/// Element's live FrameSelection and LayoutObject leaves. Selection values
/// retain canonical DOM positions; the Element algorithm owns neither a range
/// cache nor a second selection owner. The subclass override for text controls
/// is separate from this ordinary Element implementation.
pub trait ElementSelectionOnFocus {
    type Selection;
    fn IsRootEditableElement(&self) -> bool;
    fn HasFrame(&self) -> bool;
    fn VisibleSelectionRootEditableElementIsThis(&self) -> bool;
    fn SelectionAtFirstPositionInOrBeforeNode(&self) -> Self::Selection;
    fn RemoveVisibleSelectionInDOMSelectionEnabled(&self) -> bool;
    fn CreateVisibleSelection(&self, selection: Self::Selection) -> Self::Selection;
    fn SetSelection(
        &self,
        selection: Self::Selection,
        options: crate::core::editing::set_selection_options::SetSelectionOptions,
    );
    fn RevealSelection(&self);
    fn HasLayoutObject(&self) -> bool;
    fn IsLayoutEmbeddedContent(&self) -> bool;
    /// ScrollRectToVisible(BoundingBoxForScrollIntoView), with the default
    /// CenterIfNeeded alignment and align_x.rect_partial = kNoScroll.
    fn ScrollBoundingBoxForFocusIntoView(&self);
}

// Source: third_party/blink/renderer/core/dom/element.cc:8696-8760.
pub fn UpdateSelectionOnFocus<E: ElementSelectionOnFocus>(
    element: &E,
    selection_behavior: SelectionBehaviorOnFocus,
    options: &crate::core::html::focus_options::FocusOptions,
) {
    use crate::core::editing::set_selection_options::SetSelectionOptions;
    if selection_behavior == SelectionBehaviorOnFocus::kNone {
        return;
    }
    if element.IsRootEditableElement() {
        if !element.HasFrame() {
            return;
        }
        if element.VisibleSelectionRootEditableElementIsThis() {
            if !options.prevent_scroll {
                element.RevealSelection();
            }
            return;
        }
        let selection = element.SelectionAtFirstPositionInOrBeforeNode();
        let selection = if element.RemoveVisibleSelectionInDOMSelectionEnabled() {
            element.CreateVisibleSelection(selection)
        } else {
            selection
        };
        element.SetSelection(
            selection,
            SetSelectionOptions {
                should_close_typing: true,
                should_clear_typing_style: true,
                do_not_set_focus: true,
                ..SetSelectionOptions::default()
            },
        );
        if !options.prevent_scroll {
            element.RevealSelection();
        }
    } else if element.HasLayoutObject() && !element.IsLayoutEmbeddedContent() {
        if !options.prevent_scroll {
            element.ScrollBoundingBoxForFocusIntoView();
        }
    }
}

/// Scalar leaves of Element's own native arrow-key virtual. ComputedStyle
/// remains retained by Element; no scroll tree or style projection is copied.
pub trait ElementNativeArrowKeyState {
    fn IsFocusable(&self) -> bool;
    fn ScrollsOverflowX(&self) -> bool;
    fn ScrollsOverflowY(&self) -> bool;
}
// Source: element.cc:9026-9042.
pub fn NativeArrowKeyAxes(
    state: &dyn ElementNativeArrowKeyState,
) -> foundation::focusgroup_flags::FocusgroupFlags {
    use foundation::focusgroup_flags::FocusgroupFlags as F;
    let mut axes = F::kNone;
    if state.IsFocusable() {
        if state.ScrollsOverflowX() {
            axes |= F::kInline;
        }
        if state.ScrollsOverflowY() {
            axes |= F::kBlock;
        }
    }
    axes
}

// Source: third_party/blink/renderer/core/dom/element.h:237-241.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusableState {
    kNotFocusable,
    kFocusable,
    kKeyboardFocusableScroller,
}

/// Live Element inputs consumed by IsFocusableState/IsMouseFocusable.  The
/// concrete subclass remains responsible for SupportsFocus and DefaultTabIndex;
/// this translation must not infer those virtual results from a copied tree.
// Source: third_party/blink/renderer/core/dom/element.cc:8954-8975,8994-9017.
pub trait ElementMouseFocusability {
    fn IsConnected(&self) -> bool;
    fn IsFocusableStyle(&self) -> bool;
    fn SupportsFocus(&self) -> FocusableState;
    fn HasExplicitTabIndex(&self) -> bool;
    fn DefaultTabIndex(&self) -> i32;
}

// Source: third_party/blink/renderer/core/dom/element.cc:8994-8999.
pub fn IsFocusableState(element: &dyn ElementMouseFocusability) -> FocusableState {
    if !element.IsConnected() || !element.IsFocusableStyle() {
        FocusableState::kNotFocusable
    } else {
        element.SupportsFocus()
    }
}

// Source: third_party/blink/renderer/core/dom/element.cc:8954-8975.
pub fn IsMouseFocusable(element: &dyn ElementMouseFocusability) -> bool {
    let state = IsFocusableState(element);
    if state == FocusableState::kNotFocusable {
        return false;
    }
    if element.HasExplicitTabIndex() {
        return true;
    }
    if element.DefaultTabIndex() >= 0 {
        return true;
    }
    state != FocusableState::kKeyboardFocusableScroller
}

// Source: third_party/blink/renderer/core/dom/element.h:251-297.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandEventType {
    kNone,
    kCustom,
    kTogglePopover,
    kHidePopover,
    kShowPopover,
    kShowModal,
    kClose,
    kRequestClose,
    kToggle,
    kOpen,
    kShowPicker,
    kStepUp,
    kStepDown,
    kToggleFullscreen,
    kRequestFullscreen,
    kExitFullscreen,
    kPlayPause,
    kPause,
    kPlay,
    kToggleMuted,
    kPageUp,
    kPageDown,
    kPageLeft,
    kPageRight,
    kPageBlockStart,
    kPageBlockEnd,
    kPageInlineStart,
    kPageInlineEnd,
    kToggleOverscroll,
    kShowOverscroll,
    kHideOverscroll,
}

// Source: third_party/blink/renderer/core/dom/element.h:1278-1290.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateBehavior {
    kStyleAndLayout,
    kNoneForAccessibility,
    kNoneForFocusManagement,
    kAssertNoLayoutUpdates,
}

// Source: third_party/blink/renderer/core/dom/element.h:2148.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaseAppearanceValue {
    kBaseSelect,
    kBase,
}

// Source: third_party/blink/renderer/core/dom/element.h:825-831.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttributeModificationReason {
    kDirectly,
    kByParser,
    kByCloning,
    kByMoveToNewDocument,
    kBySynchronizationOfLazyAttribute,
}

// Source: element.cc:9184-9203. Node identity and traversal remain with DOM;
// HasActivationBehavior is the retained source subclass's virtual operation.
pub trait ClickableControlNode {
    fn Target(&self) -> interaction_dispatch::core::dom::events::event::EventTargetRef;
    fn IsElement(&self) -> bool;
    fn IsFormControlElement(&self) -> bool;
    fn OwnerShadowHost(&self) -> Option<std::rc::Rc<dyn ClickableControlNode>>;
    fn ParentOrShadowHost(&self) -> Option<std::rc::Rc<dyn ClickableControlNode>>;
    fn HasActivationBehavior(&self) -> bool;
}
pub fn IsClickableControl(
    owner: &interaction_dispatch::core::dom::events::event::EventTargetRef,
    node: Option<std::rc::Rc<dyn ClickableControlNode>>,
) -> bool {
    IsClickableControlWithOperations(owner, node, &SourceClickableControlOperations)
}

// Source: third_party/blink/renderer/core/dom/element.cc:9184-9205.
// The live DOM supplies identity/ancestry and source subclass facts only.
pub trait ClickableControlOperations<N, L = N> {
    fn IsElement(&self, node: &N) -> bool;
    fn IsFormControlElement(&self, node: &N) -> bool;
    fn OwnerShadowHost(&self, node: &N) -> Option<N>;
    fn ParentOrShadowHostNode(&self, node: &N) -> Option<N>;
    fn HasActivationBehavior(&self, node: &N) -> bool;
    fn SameNode(&self, left: &L, right: &N) -> bool;
}
pub fn IsClickableControlWithOperations<N, L>(
    element: &L,
    node: Option<N>,
    operations: &dyn ClickableControlOperations<N, L>,
) -> bool {
    let Some(mut node) = node.filter(|node| operations.IsElement(node)) else {
        return false;
    };
    if operations.IsFormControlElement(&node) {
        return true;
    }
    if operations
        .OwnerShadowHost(&node)
        .is_some_and(|host| operations.IsFormControlElement(&host))
    {
        return true;
    }
    while !operations.SameNode(element, &node) {
        if operations.HasActivationBehavior(&node) {
            return true;
        }
        let Some(parent) = operations.ParentOrShadowHostNode(&node) else {
            break;
        };
        node = parent;
    }
    false
}

struct SourceClickableControlOperations;
impl
    ClickableControlOperations<
        std::rc::Rc<dyn ClickableControlNode>,
        interaction_dispatch::core::dom::events::event::EventTargetRef,
    > for SourceClickableControlOperations
{
    fn IsElement(&self, node: &std::rc::Rc<dyn ClickableControlNode>) -> bool {
        node.IsElement()
    }
    fn IsFormControlElement(&self, node: &std::rc::Rc<dyn ClickableControlNode>) -> bool {
        node.IsFormControlElement()
    }
    fn OwnerShadowHost(
        &self,
        node: &std::rc::Rc<dyn ClickableControlNode>,
    ) -> Option<std::rc::Rc<dyn ClickableControlNode>> {
        node.OwnerShadowHost()
    }
    fn ParentOrShadowHostNode(
        &self,
        node: &std::rc::Rc<dyn ClickableControlNode>,
    ) -> Option<std::rc::Rc<dyn ClickableControlNode>> {
        node.ParentOrShadowHost()
    }
    fn HasActivationBehavior(&self, node: &std::rc::Rc<dyn ClickableControlNode>) -> bool {
        node.HasActivationBehavior()
    }
    fn SameNode(
        &self,
        left: &interaction_dispatch::core::dom::events::event::EventTargetRef,
        right: &std::rc::Rc<dyn ClickableControlNode>,
    ) -> bool {
        std::rc::Rc::ptr_eq(left, &right.Target())
    }
}
