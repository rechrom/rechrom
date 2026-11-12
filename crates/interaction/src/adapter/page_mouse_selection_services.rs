//! Native services for SelectionController. All DOM/layout loans end before
//! listener dispatch. Selection, markers and paint invalidation keep their
//! original retained owners; EventHandler owns context-menu placement.
use super::mouse_selection::{MouseSelectionSettings, PageMouseSelectionServices};
use super::*;
use crate::adapter::{
    page_editing_visible_position::{
        CompositionMarkerOwner, CompositionTextQuery as Q, CompositionTextResult as V,
        MouseSelectionGeometryQuery as MQ, MouseSelectionGeometryResult as MV,
        NonLocatedGeometryQuery as NQ, NonLocatedGeometryResult as NV,
    },
    page_nonlocated_context_menu_coordinates::NativeContextMenuFrameCoordinates,
    page_pointer_hit_test::PagePointerGeometry,
    page_pointer_scroll_controls::PagePointerScrollControlOwners,
    source_input_managers::ResidentCursorOwners,
};
use crate::core::input::event_handler::nonlocated_context_menu::{
    self as menu, NonLocatedCaret, NonLocatedContextMenuSource, NonLocatedEventHandler,
    NonLocatedSelectionFlags,
};
use foundation::gfx;
use interaction_default::core::editing::selection_controller::SelectionContextMenu;
use interaction_dispatch::{
    core::clipboard::data_transfer::DataTransfer,
    public::platform::web_input_event_result::WebInputEventResult,
};
use interaction_input::{
    base::time::time::TimeTicks, public::common::input::web_mouse_event::WebMouseEvent,
    ui::base::mojom::menu_source_type::MenuSourceType,
};
use layoutng_assembly::{editing_state::DOMSelection, EditingPositionWithAffinity};

pub(crate) struct PageMouseSelectionServicesOwner<'a, 'listener, 'frame> {
    dispatcher: &'a EventDispatcher<'listener, 'frame>,
    owners: &'a ResidentCursorOwners,
    geometry: Rc<dyn PagePointerGeometry>,
    controls: Rc<PagePointerScrollControlOwners>,
    error: RefCell<Option<&'static str>>,
    document_identity: u128,
    menu_selection: Cell<
        Option<(
            NonLocatedSelectionFlags,
            Option<NonLocatedCaret>,
            Option<NonLocatedCaret>,
            Option<u64>,
        )>,
    >,
}
impl<'a, 'listener, 'frame> PageMouseSelectionServicesOwner<'a, 'listener, 'frame> {
    pub(crate) fn new(
        dispatcher: &'a EventDispatcher<'listener, 'frame>,
        owners: &'a ResidentCursorOwners,
        geometry: Rc<dyn PagePointerGeometry>,
        controls: Rc<PagePointerScrollControlOwners>,
    ) -> Self {
        let document_identity = dispatcher
            .document
            .borrow()
            .GetDocument()
            .RootHandle()
            .Identity();
        Self {
            dispatcher,
            owners,
            geometry,
            controls,
            error: RefCell::new(None),
            document_identity,
            menu_selection: Cell::new(None),
        }
    }
    pub(crate) fn Controls(&self) -> Rc<PagePointerScrollControlOwners> {
        self.controls.clone()
    }
    fn Frame(&self) -> &crate::adapter::page_frame_selection::PageFrameSelection {
        &self.dispatcher.focus_services.selection
    }
    fn Fault(&self, leaf: &'static str) {
        self.error.borrow_mut().get_or_insert(leaf);
    }
    pub(crate) fn Finish(&self) -> Result<(), &'static str> {
        self.error.borrow_mut().take().map_or(Ok(()), Err)
    }
    fn Native(&self, query: NQ) -> Option<NV> {
        match self.Frame().CompositionText(Q::NonLocatedGeometry(query)) {
            Some(V::NonLocatedGeometry(value)) => Some(value),
            _ => {
                self.Fault("EventHandler native context-menu geometry");
                None
            }
        }
    }
    fn Coordinates(&self) -> Option<NativeContextMenuFrameCoordinates> {
        let value = self
            .dispatcher
            .mouse_frame_geometry
            .borrow()
            .as_ref()
            .and_then(|f| f.NonLocatedContextMenuCoordinates());
        if value.is_none() {
            self.Fault("ChromeClient context-menu frame coordinates");
        }
        value
    }
    fn MenuSelection(
        &self,
    ) -> (
        NonLocatedSelectionFlags,
        Option<NonLocatedCaret>,
        Option<NonLocatedCaret>,
        Option<u64>,
    ) {
        if let Some(value) = self.menu_selection.get() {
            return value;
        }
        let value = match self.Native(NQ::Selection(self.Frame().Current())) {
            Some(NV::Selection {
                flags,
                start,
                end,
                anchor,
            }) => (flags, start, end, anchor),
            _ => (
                NonLocatedSelectionFlags {
                    is_range: false,
                    has_editable_root: false,
                    is_caret: false,
                },
                None,
                None,
                None,
            ),
        };
        self.menu_selection.set(Some(value));
        value
    }
    fn SelectionFocus(&self) -> bool {
        let Some(settings) = self.geometry.InteractionTextSettings() else {
            self.Fault("Selection Settings");
            return false;
        };
        self.Frame().PrepareEditing();
        matches!(
            self.Frame().CompositionText(Q::MouseSelectionGeometry {
                query: MQ::SelectionFocus {
                    selection: self.Frame().Current(),
                    focused: self.dispatcher.state.borrow().focused_node_id
                },
                ui_locale: settings.ui_locale
            }),
            Some(V::MouseSelectionGeometry(MV::Boolean(true)))
        )
    }
}
impl PageMouseSelectionServices for PageMouseSelectionServicesOwner<'_, '_, '_> {
    fn Settings(&self) -> MouseSelectionSettings {
        // Settings.json5:431,979,1260: these are the frame's current platform
        // defaults; the embedding has no overriding Settings mutation today.
        MouseSelectionSettings {
            select_trailing_whitespace: cfg!(target_os = "windows"),
            spellcheck_enabled_by_default: true,
            middle_click_paste_allowed: true,
        }
    }
    fn MarkerOwner(&self) -> CompositionMarkerOwner {
        self.dispatcher
            .editor
            .ime_typing
            .MarkerOwner(self.document_identity)
    }
    fn ShowContextMenu(&self, kind: SelectionContextMenu) -> Result<(), &'static str> {
        self.menu_selection.set(None);
        menu::ShowNonLocatedContextMenu(
            self,
            None,
            match kind {
                SelectionContextMenu::Touch => MenuSourceType::kTouch,
                SelectionContextMenu::AdjustSelectionReset => MenuSourceType::kAdjustSelectionReset,
            },
            self,
        );
        self.Finish()
    }
    fn MarkSelectionEndpointsForRepaint(
        &self,
        selection: Option<DOMSelection>,
    ) -> Result<(), &'static str> {
        self.Frame().MarkSelectionEndpointsForRepaint(selection);
        Ok(())
    }
    fn HandlePotentialSuggestionTap(
        &self,
        position: EditingPositionWithAffinity,
    ) -> Result<(), &'static str> {
        let owner = self.MarkerOwner();
        if !self.Frame().IsAvailable()
            || self
                .dispatcher
                .editor
                .ime_typing
                .text_suggestion
                .is_suggestion_menu_open
                .get()
        {
            return Ok(());
        }
        let Some(V::PotentialSuggestionTap(found)) = self
            .Frame()
            .CompositionText(Q::PotentialSuggestionTap { owner, position })
        else {
            return Err("TextSuggestionController marker geometry");
        };
        if found {
            // Text suggestion menus are an Android embedder capability. The
            // retained host is optional on other platforms, as in Blink.
            if let Some(host) = self
                .dispatcher
                .editor
                .ime_typing
                .text_suggestion_host
                .borrow()
                .as_ref()
            {
                host.StartSuggestionMenuTimer()?;
            }
        }
        Ok(())
    }
    fn ExecutePasteGlobalSelection(&self) -> Result<bool, &'static str> {
        super::clipboard_commands::Execute(
            self.dispatcher,
            interaction_default::core::editing::commands::editor_command_table::ExecuteFunction::ClipboardCommands_ExecutePasteGlobalSelection,
        )
        .expect("clipboard command descriptor")
    }
    fn IsFocusedOrMainFrame(&self) -> bool {
        self.Frame().IsFocusedOrMainFrame()
    }
}
impl NonLocatedEventHandler<EventTargetRef> for PageMouseSelectionServicesOwner<'_, '_, '_> {
    fn HasView(&self) -> bool {
        self.dispatcher
            .mouse_frame_geometry
            .borrow()
            .as_ref()
            .is_some_and(|f| f.HasFrameView())
    }
    fn IsOutermostMainFrame(&self) -> bool {
        self.Coordinates().is_some_and(|c| c.outermost_main_frame)
    }
    fn UpdateStyleAndLayoutForContextMenu(&self) {
        self.menu_selection.set(None);
        if self.geometry.PrepareLifecycleExceptPaint().is_err() {
            self.Fault("context-menu lifecycle");
        }
    }
    fn UpdateHoverActiveState(&self, focused: Option<EventTargetRef>) {
        let id = focused
            .as_ref()
            .and_then(|n| self.dispatcher.targets.NodeId(n));
        let update = self
            .dispatcher
            .document
            .borrow_mut()
            .GetDocumentMut()
            .UpdateHoverActiveState(true, true, id);
        if update.notify_element_activated {
            self.owners.gesture_target.NotifyElementActivated();
        }
        let mut state = self.dispatcher.state.borrow_mut();
        state.hovered_node_id = update.hovered_node_id;
        state.pressed_node_id = update.active_node_id;
    }
    fn SendContextMenuEvent(
        &self,
        event: &WebMouseEvent,
        focused: Option<EventTargetRef>,
    ) -> WebInputEventResult {
        if self.error.borrow().is_some() {
            return WebInputEventResult::kNotHandled;
        }
        match self.dispatcher.SendGestureContextMenuEvent(
            self.owners,
            self.geometry.clone(),
            self.controls.clone(),
            event,
            focused,
        ) {
            Ok(r) => r,
            Err(_) => {
                self.Fault("EventHandler context-menu dispatch");
                WebInputEventResult::kNotHandled
            }
        }
    }
}
impl NonLocatedContextMenuSource<EventTargetRef> for PageMouseSelectionServicesOwner<'_, '_, '_> {
    fn HasDocument(&self) -> bool {
        self.Frame().IsAvailable()
    }
    fn FocusedElement(&self) -> Option<EventTargetRef> {
        self.dispatcher
            .targets
            .ResolveTarget(self.dispatcher.state.borrow().focused_node_id?)
    }
    fn SelectionFlags(&self) -> NonLocatedSelectionFlags {
        self.MenuSelection().0
    }
    fn SelectionHasFocus(&self) -> bool {
        self.SelectionFocus()
    }
    fn SelectionStartCaret(&self) -> Option<NonLocatedCaret> {
        self.MenuSelection().1
    }
    fn SelectionEndCaret(&self) -> Option<NonLocatedCaret> {
        self.MenuSelection().2
    }
    fn EnclosingAnchorAtSelectionStart(&self) -> Option<EventTargetRef> {
        self.dispatcher
            .targets
            .ResolveTarget(self.MenuSelection().3?)
    }
    fn ElementVisibleBoundsInLocalRoot(&self, element: &EventTargetRef) -> gfx::Rect {
        let Some(id) = self.dispatcher.targets.NodeId(element) else {
            return Default::default();
        };
        match self.Native(NQ::ElementVisibleBounds(id)) {
            Some(NV::Rect(rect)) => rect,
            _ => Default::default(),
        }
    }
    fn ElementOutlineRectsInWidget(&self, element: &EventTargetRef) -> Vec<gfx::Rect> {
        let Some(id) = self.dispatcher.targets.NodeId(element) else {
            return vec![];
        };
        let Some(NV::Rects(rects)) = self.Native(NQ::ElementOutlineRectsInRootFrame(id)) else {
            return vec![];
        };
        let Some(coords) = self.Coordinates() else {
            return vec![];
        };
        let frame = self.dispatcher.mouse_frame_geometry.borrow();
        rects
            .into_iter()
            .filter_map(
                |r| match frame.as_ref().and_then(|f| f.NonLocatedRectToRootFrame(r)) {
                    Some(r) => Some(coords.FrameRectToWidget(r)),
                    None => {
                        self.Fault("FrameView outline ConvertToRootFrame");
                        None
                    }
                },
            )
            .collect()
    }
    fn ConvertRectFromRootFrame(&self, rect: gfx::Rect) -> gfx::Rect {
        self.dispatcher
            .mouse_frame_geometry
            .borrow()
            .as_ref()
            .and_then(|f| f.NonLocatedRectFromRootFrame(rect))
            .unwrap_or_else(|| {
                self.Fault("FrameView ConvertFromRootFrame");
                Default::default()
            })
    }
    fn ConvertPointToRootFrame(&self, point: gfx::Point) -> gfx::Point {
        self.dispatcher
            .mouse_frame_geometry
            .borrow()
            .as_ref()
            .and_then(|f| f.NonLocatedPointToRootFrame(point))
            .unwrap_or_else(|| {
                self.Fault("FrameView ConvertToRootFrame");
                Default::default()
            })
    }
    fn RootFrameToViewport(&self, rect: gfx::Rect) -> gfx::Rect {
        self.Coordinates()
            .map_or_else(Default::default, |c| c.RootFrameToViewport(rect))
    }
    fn ViewportToRootFrame(&self, rect: gfx::Rect) -> gfx::Rect {
        self.Coordinates()
            .map_or_else(Default::default, |c| c.ViewportToRootFrame(rect))
    }
    fn VisualViewportSize(&self) -> gfx::Size {
        self.Coordinates()
            .map_or_else(Default::default, |c| c.VisualViewportSize())
    }
    fn VisualViewportScrollOffset(&self) -> gfx::PointF {
        self.Coordinates()
            .map_or_else(Default::default, |c| c.VisualViewportScrollOffset())
    }
    fn SetPointerCursor(&self) {
        self.Frame().SetPointerCursor();
    }
    fn RootFramePointToScreenDIPs(&self, point: gfx::Point) -> gfx::Point {
        self.Coordinates()
            .map_or_else(Default::default, |c| c.RootFramePointToScreenDIPs(point))
    }
    fn ShowContextMenuOnMouseUp(&self) -> bool {
        self.owners
            .gesture_target
            .settings
            .get()
            .show_context_menu_on_mouse_up
    }
    fn Now(&self) -> TimeTicks {
        self.dispatcher.clock.Now()
    }
}
impl EventDispatcher<'_, '_> {
    pub(crate) fn CreatePasteTextFragment(
        &self,
        text: &[u16],
    ) -> Result<crate::adapter::page_selection_markup::RetainedFragment, &'static str> {
        let frame = &self.focus_services.selection;
        frame.PrepareEditing();
        let Some(selection) = frame.Current() else {
            return Err("ClipboardCommands selected range");
        };
        let Some(V::FragmentFromTextContext(mut context)) =
            frame.CompositionText(Q::FragmentFromTextContext(selection))
        else {
            return Err("CreateFragmentFromText context");
        };
        context.default_paragraph_separator =
            self.editor.ime_typing.editor.DefaultParagraphSeparator();
        crate::adapter::page_selection_markup::CreateFragmentFromText(
            &self.document,
            &self.emit,
            &context,
            text,
        )
        .map_err(|_| "CreateFragmentFromText")
    }
    pub(crate) fn ApplyPasteReplacement(
        &self,
        fragment: &crate::adapter::page_selection_markup::RetainedFragment,
        smart: bool,
        match_style: bool,
        transfer: Option<Rc<DataTransfer>>,
    ) -> Result<(), &'static str> {
        crate::adapter::page_input_method_controller::ApplyPasteReplacement(
            self,
            self.document.clone(),
            self.editor.clone(),
            self.selections.clone(),
            self.state.clone(),
            self.emit.clone(),
            self.focus_services.selection.clone(),
            fragment,
            smart,
            match_style,
            transfer,
        )
    }
}
