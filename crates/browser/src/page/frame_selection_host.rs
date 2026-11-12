//! Shared FrameSelection host leaves over the original Document/Layout owners.
//! Source: frame_selection.cc and selection_editor.cc. Selection algorithms stay
//! in Interaction; this assembly only routes native engine and platform effects.
use super::local_frame::{LocalFrame, LocalFrameRuntime};
use super::*;

#[derive(Clone)]
pub(super) struct ResidentFrameSelectionHost {
    page: Weak<PageState>,
    child: Option<(Weak<LocalFrame>, Weak<LocalFrameRuntime>)>,
}
struct FrameSelectionOwners {
    page: Rc<PageState>,
    child: Option<(Rc<LocalFrame>, Rc<LocalFrameRuntime>)>,
}
impl ResidentFrameSelectionHost {
    fn SerializeEditingRange(
        &self,
        selection: layoutng_assembly::editing_state::DOMSelection,
        options: interaction::default::core::editing::serializers::styled_markup_accumulator::CreateMarkupOptions,
        constraining_ancestor: Option<u64>,
    ) -> Result<Vec<u16>, interaction::default::core::editing::serializers::styled_markup_serializer::SerializationError>{
        use interaction::default::core::editing::serializers::styled_markup_serializer::SerializationError;
        let state = self.Owners().ok_or(SerializationError::MissingLayout)?;
        state.EnsureLayoutOutput();
        let layout = state.Layout().borrow();
        let output = layout.Output().ok_or(SerializationError::MissingLayout)?;
        let owner = state.Document().borrow();
        struct Styles<'a> {
            engine: &'a RefCell<style::StyleEngine>,
            document: &'a dom::Document,
        }
        impl interaction::adapter::page_selection_markup::MarkupStyleQueries for Styles<'_> {
            fn Query(
                &self,
                operation: cssom::editing_style::SerializationStyleOperation,
            ) -> Result<cssom::CSSInlineStyle, cssom::editing_style::EditingStyleError>
            {
                style::editing_style::QuerySerializationEditingStyle(
                    &mut self.engine.borrow_mut(),
                    self.document,
                    operation,
                    None,
                )
            }
        }
        interaction::adapter::page_selection_markup::SerializeMarkupRangeWithAncestor(
            owner.GetDocument(),
            output,
            &Styles {
                engine: state.Style(),
                document: owner.GetDocument(),
            },
            selection,
            options,
            constraining_ancestor,
        )
    }
    pub(super) fn Root(page: Weak<PageState>) -> Rc<Self> {
        Rc::new(Self { page, child: None })
    }
    pub(super) fn Child(frame: &Rc<LocalFrame>, runtime: &Rc<LocalFrameRuntime>) -> Rc<Self> {
        Rc::new(Self {
            page: frame
                .domain
                .upgrade()
                .map_or_else(Weak::new, |domain| domain.page.Downgrade()),
            child: Some((Rc::downgrade(frame), Rc::downgrade(runtime))),
        })
    }
    fn Owners(&self) -> Option<FrameSelectionOwners> {
        let page = self.page.upgrade()?;
        let child = if let Some((frame, runtime)) = &self.child {
            let frame = frame.upgrade()?;
            let runtime = runtime.upgrade()?;
            if !frame.IsCurrentRuntime(&runtime) {
                return None;
            }
            Some((frame, runtime))
        } else {
            None
        };
        Some(FrameSelectionOwners { page, child })
    }
    fn RootApplyEffect(
        &self,
        effect: interaction::adapter::page_frame_selection::FrameSelectionEffect,
    ) {
        use interaction::adapter::page_frame_selection::FrameSelectionEffect as E;
        let Some(state) = self.page.upgrade() else {
            return;
        };
        match effect {
            E::InvalidateInputCapsLockIndicatorPaint(node) => {
                state
                    .layout_engine
                    .borrow_mut()
                    .InvalidateInputCapsLockIndicatorPaint(node);
                state.dirty.set(true);
            }
            E::MarkSelectionEndpointPaintingLayerNeedsRepaint(node) => {
                state
                    .layout_engine
                    .borrow_mut()
                    .MarkSelectionEndpointPaintingLayerNeedsRepaint(node);
                state.dirty.set(true);
            }
            E::SetPointerCursor => state.client.borrow_mut().DidChangeCursor(Cursor::default()),
            E::InvalidateCompositionMarkerPaint(node) => {
                state
                    .layout_engine
                    .borrow_mut()
                    .InvalidateDocumentMarkerPaint(node);
                state.dirty.set(true);
            }
            E::NotifyDisplayLock { old, new } => {
                let mut owner = state.document.borrow_mut();
                interaction::adapter::page_focus_controller::NotifyDisplayLockForSelectionChangeForFocus(
                    owner.GetDocumentMut(),
                    old,
                    new,
                );
                state.dirty.set(true);
            }
            E::SetContainsSelectionFocusFlag {
                owner,
                contains_focus,
            } => {
                if let Some(node_id) = state
                    .layout_engine
                    .borrow_mut()
                    .SetContainsSelectionFocusByIdentity(owner.0, contains_focus)
                {
                    let document_identity = state
                        .document
                        .borrow()
                        .GetDocument()
                        .RootHandle()
                        .Identity();
                    state
                        .pending_text_overflow_layout_invalidations
                        .borrow_mut()
                        .push(PendingTextOverflowLayoutInvalidation {
                            document_identity,
                            node_id,
                        });
                }
                state.dirty.set(true);
            }
            E::StopCaretBlinkTimer => {
                if state
                    .layout_editing
                    .caret
                    .borrow_mut()
                    .StopCaretBlinkTimer()
                {
                    state.dirty.set(true);
                }
            }
            E::MarkInitialScrollWasScrolledByUser => {
                if let Some(owners) = state.document_loader_owners.borrow().as_ref() {
                    owners.initial_scroll.was_scrolled_by_user.set(true);
                }
            }
            E::ScheduleTextControlSelectionchange => {
                if state.bindings.borrow().upgrade().is_none() {
                    return;
                }
                let selection = PageMouseFrameGeometry::CurrentSelection(&state);
                let mut owner = state.document.borrow_mut();
                let document = owner.GetDocumentMut();
                if let Some(target) = interaction::adapter::page_focus_controller::EnclosingTextControlForSelectionForFocus(
                    document,
                    selection,
                ) {
                    if document.MarkSelectionchangeEventScheduled(target) {
                        state.selectionchange_event_targets.borrow_mut().push(target);
                    }
                }
            }
            E::ScheduleDocumentSelectionchange | E::EnqueueLegacyDocumentSelectionchange => {
                if state.bindings.borrow().upgrade().is_none() {
                    return;
                }
                let mut owner = state.document.borrow_mut();
                let document = owner.GetDocumentMut();
                let target = document.Node(document.Root()).Id();
                if document.MarkSelectionchangeEventScheduled(target) {
                    state
                        .selectionchange_event_targets
                        .borrow_mut()
                        .push(target);
                }
            }
            E::ScheduleVisualUpdateForVisualOverflow
            | E::UpdateAppearance
            | E::NotifyCompositorForSelectionChange
            | E::NotifyEventHandlerForSelectionChange
            | E::RespondToChangedSelection
            | E::DidChangeSelectionAfterCommand { .. }
            | E::HandleAXEditableTextContentChanged { .. }
            | E::NotifyAccessibilityForSelectionChange
            | E::NotifyTextControlOfSelectionChange(_)
            | E::SpellCheckerRespondToChangedContents
            | E::SpellCheckerRespondToChangedSelection
            | E::FrameClientDidChangeContents => {
                // These source notifications converge on the Page's existing
                // selection/caret paint lifecycle. Disabled AX/spell/embedder
                // owners have no independent state in this embedding.
                state.dirty.set(true);
            }
            E::CloseTyping
            | E::ClearTypingStyle
            | E::ClearDocumentCachedRange
            | E::SelectFrameElementInParentIfFullySelected => {
                // Interaction synchronously closes the original typing command
                // and clears its Editor style owner before routing these facts.
                // Page owns no second editor cache or parent frame. The
                // retained Interaction focus transaction handles focus itself.
            }
        }
    }
}
impl FrameSelectionOwners {
    fn MarkDirty(&self) {
        if let Some((frame, _)) = &self.child {
            frame.MarkNavigationDirty();
        } else {
            self.page.dirty.set(true);
        }
    }
    fn Token(&self) -> u64 {
        self.child
            .as_ref()
            .map_or(self.page.local_frames.get().unwrap().root, |(frame, _)| {
                frame.state.Identity()
            })
    }
    fn Document(&self) -> &Rc<DocumentEngine> {
        self.child
            .as_ref()
            .map_or(&self.page.document, |(_, runtime)| &runtime.document)
    }
    fn Layout(&self) -> &RefCell<layoutng_assembly::layout_engine::LayoutEngine> {
        self.child
            .as_ref()
            .map_or(&self.page.layout_engine, |(_, runtime)| &runtime.layout)
    }
    fn Style(&self) -> &RefCell<style::StyleEngine> {
        self.child
            .as_ref()
            .map_or(&self.page.style_engine, |(_, runtime)| &runtime.style)
    }
    fn Editing(&self) -> &Rc<layoutng_assembly::editing_state::LayoutEditingState> {
        self.child
            .as_ref()
            .map_or(&self.page.layout_editing, |(_, runtime)| &runtime.editing)
    }
    fn EnsureLayoutOutput(&self) {
        if let Some((frame, runtime)) = &self.child {
            frame.EnsureFocusLayout(runtime);
        } else {
            self.page.EnsureLayoutOutput();
        }
    }
    fn EnsureLayoutOutputForNode(&self, node: u64) {
        if self.child.is_some() {
            self.EnsureLayoutOutput();
        } else {
            self.page.EnsureLayoutOutputForNode(node);
        }
    }
    fn CurrentSelection(&self) -> Option<layoutng_assembly::editing_state::DOMSelection> {
        let owner = self.Document().borrow();
        interaction::adapter::page_frame_selection::CurrentDOMSelectionForFocus(
            owner.GetDocument(),
            &self.Editing().selections,
        )
    }
    fn CaretBounds(
        &self,
        selection: Option<layoutng_assembly::editing_state::DOMSelection>,
        extent: interaction::default::core::editing::frame_selection::RevealExtentOption,
    ) -> foundation::gfx::Rect {
        self.EnsureLayoutOutput();
        let layout = self.Layout().borrow();
        let Some(output) = layout.Output() else {
            return Default::default();
        };
        let owner = self.Document().borrow();
        interaction::adapter::page_focus_controller::CaretBoundsForSelectionForFocus(
            owner.GetDocument(),
            output,
            selection,
            extent,
            foundation::LayoutUnit::from_f64(layout.GetConstraints().device_pixel_ratio.max(1.0))
                .ToDouble(),
        )
    }
    fn ApplyNativeScrollResult(
        &self,
        result: layoutng_assembly::native_scroll_source::NativeScrollResult,
    ) {
        if let Some((frame, runtime)) = &self.child {
            frame.ApplyNativeScrollResult(runtime, result);
        } else {
            self.page.ApplyNativeScrollResult(result);
        }
    }
    fn ApplyChildEffect(
        &self,
        effect: interaction::adapter::page_frame_selection::FrameSelectionEffect,
    ) {
        use interaction::adapter::page_frame_selection::FrameSelectionEffect as E;
        let (frame, runtime) = self.child.as_ref().expect("child effects owner");
        match effect {
            E::InvalidateInputCapsLockIndicatorPaint(node) => {
                runtime.layout.borrow_mut().InvalidateInputCapsLockIndicatorPaint(node);
            }
            E::MarkSelectionEndpointPaintingLayerNeedsRepaint(node) => runtime.layout.borrow_mut().MarkSelectionEndpointPaintingLayerNeedsRepaint(node),
            E::InvalidateCompositionMarkerPaint(node) => {
                runtime
                    .layout
                    .borrow_mut()
                    .InvalidateDocumentMarkerPaint(node);
            }
            E::SetPointerCursor => self.page.client.borrow_mut().DidChangeCursor(Cursor::default()),
            E::NotifyDisplayLock { old, new } => interaction::adapter::page_focus_controller::NotifyDisplayLockForSelectionChangeForFocus(runtime.document.borrow_mut().GetDocumentMut(), old, new),
            E::SetContainsSelectionFocusFlag { owner, contains_focus } => {
                if let Some(node) = runtime.layout.borrow_mut().SetContainsSelectionFocusByIdentity(owner.0, contains_focus) {
                    runtime.pending_text_overflow_layout_invalidations.borrow_mut().push(node);
                }
            }
            E::StopCaretBlinkTimer => { runtime.editing.caret.borrow_mut().StopCaretBlinkTimer(); }
            E::MarkInitialScrollWasScrolledByUser => { if let Some(loader) = &runtime.loader { loader.borrow().InitialScrollState().was_scrolled_by_user.set(true); } }
            E::ScheduleTextControlSelectionchange => frame.QueueSelectionchange(runtime, true),
            E::ScheduleDocumentSelectionchange | E::EnqueueLegacyDocumentSelectionchange => frame.QueueSelectionchange(runtime, false),
            E::SelectFrameElementInParentIfFullySelected => frame.SelectOwnerInParentIfFullySelected(runtime),
            E::CloseTyping | E::ClearTypingStyle | E::ClearDocumentCachedRange => {},
            _ => {},
        }
        frame.MarkNavigationDirty();
    }
}
impl
    interaction::default::core::editing::frame_selection::SelectionFocusEffects<
        interaction::adapter::page_frame_selection::FrameSelectionValue,
    > for ResidentFrameSelectionHost
{
    fn PageIsActive(&self) -> bool {
        self.Owners().is_some_and(|owners| {
            owners.page.local_frames.get().is_some_and(|domain| {
                domain
                    .focus_controller
                    .borrow()
                    .as_ref()
                    .is_some_and(|controller| controller.IsActive())
            })
        })
    }
    fn PublishFocusState(&self, focused_and_active: bool, page_active: bool) {
        let Some(owners) = self.Owners() else {
            return;
        };
        let document = owners.Document().borrow();
        owners
            .Style()
            .borrow_mut()
            .SetPageFocusState(&document, focused_and_active, page_active)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    fn FocusedElementFocusStateChanged(&self) {
        let Some(owners) = self.Owners() else {
            return;
        };
        let mut owner = owners.Document().borrow_mut();
        let document = owner.GetDocumentMut();
        // UserActionElementSet is the original focused-element owner; this
        // bounded set does not reconstruct a document or layout tree.
        let focused: Vec<_> = document.FocusedUserActionElements().collect();
        for node in focused {
            let has_layout = owners
                .Layout()
                .borrow()
                .EventRegistryLayout(Some(document.Node(node).Id()))
                .is_some();
            document.InvalidateFocusStateStyle(node, has_layout);
        }
        owners.MarkDirty();
    }
    fn HasLayoutView(&self) -> bool {
        self.Owners()
            .is_some_and(|owners| owners.Layout().borrow().Output().is_some())
    }
    fn InvalidateStyleAndPaintForSelection(&self) {
        let Some(owners) = self.Owners() else {
            return;
        };
        let effects = owners
            .Layout()
            .borrow_mut()
            .InvalidateStyleAndPaintForSelection();
        let mut owner = owners.Document().borrow_mut();
        let document = owner.GetDocumentMut();
        for node in effects.selection_pseudo_elements {
            if let Some(node) = document.FindNodeById(node) {
                document.InvalidateSelectionPseudo(node);
            }
        }
        owners.MarkDirty();
    }
    fn UpdateStyleAndLayoutTree(&self) {
        if let Some(owners) = self.Owners() {
            if owners.child.is_some() {
                owners.EnsureLayoutOutput();
            } else {
                owners.page.ResolveStyles();
            }
        }
    }
    fn UpdateStyleAndLayoutForSelection(&self) {
        if let Some(owners) = self.Owners() {
            owners.EnsureLayoutOutput();
        }
    }
    fn DocumentIsEditable(&self) -> bool {
        self.Owners().is_some_and(|owners| {
            let owner = owners.Document().borrow();
            let document = owner.GetDocument();
            document.IsEditableForEditing(document.Root())
        })
    }
    fn SelectionAtBodyStart(
        &self,
    ) -> Option<interaction::adapter::page_frame_selection::FrameSelectionValue> {
        let owners = self.Owners()?;
        let owner = owners.Document().borrow();
        let document = owner.GetDocument();
        let body = document.BodyElement()?;
        if !document.Node(body).IsHTMLElement("body") {
            return None;
        }
        interaction::adapter::page_focus_controller::SelectionAtFirstPositionInOrBeforeNodeForFocus(
            document,
            document.Node(body).Id(),
        )
        .map(|selection| {
            interaction::adapter::page_frame_selection::FrameSelectionValue(Some(selection))
        })
    }
    fn SetCaretEnabled(&self, enabled: bool) {
        let Some(owners) = self.Owners() else {
            return;
        };
        if owners.Editing().caret.borrow_mut().SetCaretEnabled(enabled) {
            owners.MarkDirty();
        }
    }
    fn CapsLockStateMayHaveChanged(&self, frame_focused_and_active: bool) {
        let Some(owners) = self.Owners() else {
            return;
        };
        let focused = owners.child.as_ref().map_or_else(
            || owners.page.interaction.borrow().focused_node_id,
            |(_, runtime)| runtime.interaction_state.borrow().focused_node_id,
        );
        interaction::adapter::page_keyboard_event_manager::UpdatePasswordCapsLockIndicator(
            &owners.Document().Handle(),
            focused,
            frame_focused_and_active,
            self,
        )
        .unwrap_or_else(|port| panic!("required keyboard port: {port}"));
    }
}
impl interaction::adapter::page_frame_selection::PageFrameSelectionHost
    for ResidentFrameSelectionHost
{
    fn CurrentKeyboardModifiers(
        &self,
    ) -> Result<
        interaction::core::input::keyboard_event_manager::NativeKeyboardModifiers,
        &'static str,
    > {
        let owners = self
            .Owners()
            .ok_or("keyboard modifiers on detached frame")?;
        let result = owners.page.client.borrow().CurrentKeyboardModifiers();
        result
    }
    fn FocusEffects(
        &self,
    ) -> Rc<
        dyn interaction::default::core::editing::frame_selection::SelectionFocusEffects<
            interaction::adapter::page_frame_selection::FrameSelectionValue,
        >,
    > {
        Rc::new(self.clone())
    }
    fn InteractionTextSettings(
        &self,
    ) -> Option<interaction::adapter::page_pointer_hit_test::InteractionTextSettings> {
        let owners = self.Owners()?;
        let settings = owners.page.interaction_text_settings.borrow().clone();
        settings
    }
    fn SuppressScrollAnchoringForNode(
        &self,
        node: Option<u64>,
    ) -> layoutng_assembly::layout_engine::SuppressScrollAnchorScope {
        let owners = self.Owners().expect("live editing frame");
        let scope = owners
            .Layout()
            .borrow()
            .SuppressScrollAnchoringForNode(node);
        scope
    }

    fn MarkupSanitizationHost(
        &self,
    ) -> Rc<dyn interaction::adapter::page_markup_sanitization::MarkupSanitizationHost> {
        Rc::new(crate::markup_sanitization::BrowserMarkupSanitizationHost)
    }
    fn SelectedHtmlForClipboard(&self,selection:layoutng_assembly::editing_state::DOMSelection)->Result<Vec<u16>,interaction::default::core::editing::serializers::styled_markup_serializer::SerializationError>{
        use interaction::default::core::editing::serializers::{
            markup_formatter::ResolveUrls, styled_markup_accumulator::CreateMarkupOptions,
        };
        self.SerializeEditingRange(
            selection,
            CreateMarkupOptions {
                annotate_for_interchange: true,
                resolve_urls: ResolveUrls::NonLocal,
                ignore_css_text_transforms: true,
                skip_unselectable_content: true,
                ..Default::default()
            },
            None,
        )
    }
    fn SerializeParagraphForMove(&self, selection: layoutng_assembly::editing_state::DOMSelection, constraining_ancestor: Option<u64>) -> Result<Vec<u16>, interaction::default::core::editing::serializers::styled_markup_serializer::SerializationError>{
        self.SerializeEditingRange(selection, interaction::default::core::editing::serializers::styled_markup_accumulator::CreateMarkupOptions { convert_blocks_to_inlines: true, ..Default::default() }, constraining_ancestor)
    }
    fn ReadPrimarySelection(
        &self,
    ) -> Result<
        interaction::default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
        interaction::default::core::editing::commands::clipboard_commands::ClipboardHostError,
    > {
        use interaction::default::core::editing::commands::clipboard_commands::ClipboardHostError;
        let state = self.Owners().ok_or(ClipboardHostError::Unavailable)?;
        let result = state.page.client.borrow().ReadPrimarySelection();
        result
    }

    fn ClipboardAccessPolicy(
        &self,
    ) -> interaction::default::core::editing::commands::clipboard_commands::ClipboardAccessPolicy
    {
        let Some(state) = self.Owners() else {
            return Default::default();
        };
        let policy = state.page.client.borrow().ClipboardAccessPolicy();
        policy
    }
    fn ReadClipboard(
        &self,
    ) -> Result<
        interaction::default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
        interaction::default::core::editing::commands::clipboard_commands::ClipboardHostError,
    > {
        use interaction::default::core::editing::commands::clipboard_commands::ClipboardHostError;
        let state = self.Owners().ok_or(ClipboardHostError::Unavailable)?;
        let result = state.page.client.borrow().ReadClipboard();
        result
    }
    fn WriteClipboard(
        &self,
        snapshot: &interaction::default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
    ) -> Result<
        (),
        interaction::default::core::editing::commands::clipboard_commands::ClipboardHostError,
    > {
        use interaction::default::core::editing::commands::clipboard_commands::ClipboardHostError;
        let state = self.Owners().ok_or(ClipboardHostError::Unavailable)?;
        let result = state.page.client.borrow().WriteClipboard(snapshot);
        result
    }
    fn ElementIsStyledSpanOrHtmlEquivalent(
        &self,
        node: u64,
    ) -> Result<bool, cssom::editing_style::EditingStyleError> {
        let state = self
            .Owners()
            .ok_or(cssom::editing_style::EditingStyleError::MissingStyle)?;
        let owner = state.Document().borrow();
        style::editing_style::ElementIsStyledSpanOrHtmlEquivalent(owner.GetDocument(), node)
    }
    fn ApplyEditingStyle(
        &self,
        operation: cssom::editing_style::ApplyStyleOperation,
    ) -> Result<cssom::editing_style::ApplyStyleResult, cssom::editing_style::EditingStyleError>
    {
        let state = self
            .Owners()
            .ok_or(cssom::editing_style::EditingStyleError::MissingStyle)?;
        state.EnsureLayoutOutput();
        let owner = state.Document().borrow();
        style::editing_style::QueryApplyEditingStyle(owner.GetDocument(), operation)
    }
    fn EditingStyleApplication(
        &self,
        declaration: cssom::CSSInlineStyle,
        originals: &[(
            foundation::CSSPropertyID,
            foundation::CSSValueID,
            foundation::QualifiedName,
        )],
    ) -> Result<
        cssom::editing_style::EditingStyleApplication,
        cssom::editing_style::EditingStyleError,
    > {
        let state = self
            .Owners()
            .ok_or(cssom::editing_style::EditingStyleError::MissingStyle)?;
        state.EnsureLayoutOutput();
        let owner = state.Document().borrow();
        style::editing_style::QueryEditingStyleApplication(
            owner.GetDocument(),
            declaration,
            originals,
        )
    }
    fn ReplacementEditingStyle(
        &self,
        operation: cssom::editing_style::ReplacementStyleOperation,
    ) -> Result<cssom::editing_style::ReplacementStyleResult, cssom::editing_style::EditingStyleError>
    {
        let state = self
            .Owners()
            .ok_or(cssom::editing_style::EditingStyleError::MissingStyle)?;
        state.EnsureLayoutOutput();
        let owner = state.Document().borrow();
        let style = state.Style();
        let mut style = style.borrow_mut();
        let result = style::editing_style::QueryReplacementEditingStyle(
            &mut style,
            owner.GetDocument(),
            operation,
            None,
        );
        result
    }
    fn EditingStyle(
        &self,
        operation: cssom::editing_style::EditingStyleOperation,
    ) -> Result<cssom::CSSInlineStyle, cssom::editing_style::EditingStyleError> {
        let state = self
            .Owners()
            .ok_or(cssom::editing_style::EditingStyleError::MissingStyle)?;
        state.EnsureLayoutOutput();
        let owner = state.Document().borrow();
        style::editing_style::QueryEditingStyle(owner.GetDocument(), operation)
    }
    fn HasFrame(&self) -> bool {
        self.Owners()
            .is_some_and(|state| state.Document().borrow().GetDocument().IsActive())
    }

    fn Settings(&self) -> interaction::adapter::page_frame_selection::FrameSelectionSettings {
        interaction::adapter::page_frame_selection::FrameSelectionSettings {
            features:
                interaction::default::core::editing::frame_selection::FrameSelectionFeatures {
                    bidi_visual_order_caret_movement:
                        foundation::RuntimeEnabledFeatures::BidiVisualOrderCaretMovementEnabled(),
                    notify_unchanged: true,
                    per_element_selectionchange:
                        foundation::RuntimeEnabledFeatures::DispatchSelectionchangeEventPerElementEnabled(),
                },
            always_use_directional_selection: !cfg!(target_os = "macos"),
            center_align_revealed_selection: cfg!(target_os = "macos"),
            remove_visible_selection_in_dom_selection:
                foundation::RuntimeEnabledFeatures::RemoveVisibleSelectionInDOMSelectionEnabled(),
            text_overflow_clip_with_selection:
                foundation::RuntimeEnabledFeatures::TextOverflowClipWithSelectionEnabled(),
        }
    }

    fn ApplyEffect(
        &self,
        effect: interaction::adapter::page_frame_selection::FrameSelectionEffect,
    ) {
        if self.child.is_none() {
            self.RootApplyEffect(effect);
            return;
        }
        let Some(owner) = self.Owners() else {
            return;
        };
        owner.ApplyChildEffect(effect);
    }

    fn CurrentEventType(&self) -> Option<String> {
        None
    }

    fn EditorFrameClient(
        &self,
    ) -> Option<Rc<dyn interaction::default::core::editing::editor::EditorFrameClient>> {
        None
    }

    fn EnclosingTextControlIsOutsideShadowTree(
        &self,
        selection: Option<layoutng_assembly::editing_state::DOMSelection>,
    ) -> bool {
        let Some(state) = self.Owners() else {
            return false;
        };
        let owner = state.Document().borrow();
        let document = owner.GetDocument();
        interaction::adapter::page_focus_controller::EnclosingTextControlForSelectionForFocus(
            document, selection,
        )
        .and_then(|id| document.FindNodeById(id))
        .is_some_and(|node| !document.IsInShadowTree(node))
    }

    fn IsFocusedOrMainFrame(&self) -> bool {
        let Some(owner) = self.Owners() else {
            return false;
        };
        let domain = owner.page.local_frames.get().expect("native FrameTree");
        let Some(controller) = domain.focus_controller.borrow().clone() else {
            return false;
        };
        let Some(own) = domain.FocusFrame(owner.Token()) else {
            return false;
        };
        controller
            .FocusedOrMainFrame()
            .is_some_and(|focused| Rc::ptr_eq(&focused, &own))
    }
    fn ViewportToContentsPoint(&self, point: (i32, i32)) -> Option<(i32, i32)> {
        let owner = self.Owners()?;
        let root = owner
            .page
            .web_view_zoom
            .borrow()
            .ViewportToRootFrame(foundation::gfx::PointF::new(point.0 as f32, point.1 as f32));
        let point = foundation::gfx::Point::new(root.x().round() as i32, root.y().round() as i32);
        let domain = owner.page.local_frames.get()?;
        let point = domain.RootPointToFrame(owner.Token(), point)?;
        Some((point.x(), point.y()))
    }
    fn CaretSelectionForContentsPoint(
        &self,
        point: (i32, i32),
    ) -> Option<layoutng_assembly::editing_state::DOMSelection> {
        use interaction::hit_test::core::layout::{
            hit_test_location::HitTestLocation, hit_test_request::RequestType,
        };
        let state = self.Owners()?;
        if let Some((frame, runtime)) = state.child.as_ref() {
            return frame.CaretSelectionForContentsPoint(runtime, point);
        }
        let settings = state.page.interaction_text_settings.borrow().clone()?;
        let geometry = pointer_geometry::PagePointerGeometryProvider(self.page.clone());
        // visible_units.cc:490-505 calls LayoutView::HitTest with these flags.
        let hit = geometry
            .Query(|query| {
                query.HitTestInLocalFrame(
                    HitTestLocation::FromPhysicalOffset(foundation::PhysicalOffset::new(
                        foundation::LayoutUnit::FromDoubleRound(f64::from(point.0)),
                        foundation::LayoutUnit::FromDoubleRound(f64::from(point.1)),
                    )),
                    RequestType::kMove as u32
                        | RequestType::kReadOnly as u32
                        | RequestType::kActive as u32
                        | RequestType::kIgnoreClipping as u32,
                )
            })
            .ok()?
            .ok()?;
        let target = hit.result.InnerNode()?.DOMNodeId()?;
        // The source HitTestResult owns pseudo/default caret branches and
        // obtains PositionForPoint through a short resident Layout query.
        let position = hit.result.GetPosition().map(|position| position.Value());
        let selection = state.CurrentSelection()?;
        let layout = state.Layout().borrow();
        let owner = state.Document().borrow();
        interaction::adapter::page_frame_selection::CaretSelectionForContentsHit(
            owner.GetDocument(),
            layout.Output()?,
            selection,
            target,
            position,
            *hit.result.LocalPoint(),
            layoutng_assembly::EditingPositionOptions {
                move_caret_to_horizontal_boundary_when_past_top_or_bottom: settings
                    .editing_behavior
                    .ShouldMoveCaretToHorizontalBoundaryWhenPastTopOrBottom(),
                ..Default::default()
            },
            &settings.ui_locale,
        )
    }

    fn UpdateStyleAndLayoutForSelection(&self) {
        if let Some(state) = self.Owners() {
            state.EnsureLayoutOutput();
        }
    }

    fn ComputeVisibleSelectionRootEditableElementDeprecated(
        &self,
        selection: Option<layoutng_assembly::editing_state::DOMSelection>,
    ) -> Option<u64> {
        let state = self.Owners()?;
        let forced = {
            let mut owner = state.Document().borrow_mut();
            interaction::adapter::page_focus_controller::BeginDisplayLockForcedSelectionForFocus(
                owner.GetDocumentMut(),
                selection,
                false,
            )
        };
        state.EnsureLayoutOutput();
        let current_selection = state.CurrentSelection();
        let result = {
            let layout = state.Layout().borrow();
            let owner = state.Document().borrow();
            layout.Output().and_then(|output| {
                interaction::adapter::page_focus_controller::VisibleDOMSelectionRootEditableElementForFocus(
                    owner.GetDocument(),
                    output,
                    current_selection,
                )
            })
        };
        let mut owner = state.Document().borrow_mut();
        interaction::adapter::page_focus_controller::EndDisplayLockForcedSelectionForFocus(
            owner.GetDocumentMut(),
            forced,
        );
        result
    }

    fn CreateVisibleSelection(
        &self,
        selection: Option<layoutng_assembly::editing_state::DOMSelection>,
    ) -> Option<layoutng_assembly::editing_state::DOMSelection> {
        let state = self.Owners()?;
        state.EnsureLayoutOutput();
        let layout = state.Layout().borrow();
        let output = layout.Output()?;
        let owner = state.Document().borrow();
        interaction::adapter::page_focus_controller::CreateVisibleDOMSelectionForFocus(
            owner.GetDocument(),
            output,
            selection,
        )
    }

    fn VisibleSelectionRootEditableElement(
        &self,
        selection: Option<layoutng_assembly::editing_state::DOMSelection>,
    ) -> Option<u64> {
        let state = self.Owners()?;
        state.EnsureLayoutOutput();
        let layout = state.Layout().borrow();
        let output = layout.Output()?;
        let owner = state.Document().borrow();
        interaction::adapter::page_focus_controller::VisibleDOMSelectionRootEditableElementForFocus(
            owner.GetDocument(),
            output,
            selection,
        )
    }

    fn VisibleSelectionStartAnchor(
        &self,
        selection: Option<layoutng_assembly::editing_state::DOMSelection>,
    ) -> Option<u64> {
        let state = self.Owners()?;
        let owner = state.Document().borrow();
        interaction::adapter::page_focus_controller::VisibleDOMSelectionStartAnchorForFocus(
            owner.GetDocument(),
            selection,
        )
    }

    fn SelectionAtFirstPositionInOrBeforeNode(
        &self,
        node_id: u64,
    ) -> Option<layoutng_assembly::editing_state::DOMSelection> {
        let state = self.Owners()?;
        let owner = state.Document().borrow();
        interaction::adapter::page_focus_controller::SelectionAtFirstPositionInOrBeforeNodeForFocus(
            owner.GetDocument(),
            node_id,
        )
    }

    fn HasLayoutObject(&self, node_id: u64) -> bool {
        self.Owners().is_some_and(|owner| {
            owner
                .Layout()
                .borrow()
                .EventRegistryLayout(Some(node_id))
                .is_some()
        })
    }

    fn ContainingBlockForTextOverflow(
        &self,
        node_id: u64,
    ) -> Option<interaction::adapter::page_frame_selection::SelectionTextOverflowOwner> {
        let state = self.Owners()?;
        state.EnsureLayoutOutput();
        let owner = state
            .Layout()
            .borrow()
            .SelectionTextOverflowOwnerIdentity(node_id)
            .map(interaction::adapter::page_frame_selection::SelectionTextOverflowOwner);
        owner
    }

    fn UpdateStyleAndLayoutForNode(&self, node_id: u64) {
        if let Some(state) = self.Owners() {
            state.EnsureLayoutOutputForNode(node_id);
        }
    }

    fn AbsoluteCaretBounds(&self) -> foundation::gfx::Rect {
        let Some(state) = self.Owners() else {
            return Default::default();
        };
        state.CaretBounds(
            state.CurrentSelection(),
            interaction::default::core::editing::frame_selection::RevealExtentOption::kDoNotRevealExtent,
        )
    }

    fn CompositionText(
        &self,
        query: interaction::adapter::page_frame_selection::CompositionTextQuery,
    ) -> Option<interaction::adapter::page_frame_selection::CompositionTextResult> {
        let state = self.Owners()?;
        state.EnsureLayoutOutput();
        let layout = state.Layout().borrow();
        let output = layout.Output()?;
        let owner = state.Document().borrow();
        interaction::adapter::page_frame_selection::QueryCompositionText(
            owner.GetDocument(),
            output,
            query,
        )
    }

    fn AbsoluteCaretBoundsOfVisibleFocus(
        &self,
        selection: Option<layoutng_assembly::editing_state::DOMSelection>,
    ) -> foundation::gfx::Rect {
        let Some(state) = self.Owners() else {
            return Default::default();
        };
        state.CaretBounds(
            selection,
            interaction::default::core::editing::frame_selection::RevealExtentOption::kRevealExtent,
        )
    }

    fn AbsoluteSelectionBounds(&self) -> foundation::gfx::Rect {
        let Some(state) = self.Owners() else {
            return Default::default();
        };
        state.EnsureLayoutOutput();
        let selection = state.CurrentSelection();
        let owner = state.Document().borrow();
        let mut layout = state.Layout().borrow_mut();
        interaction::adapter::page_focus_controller::CommitLayoutSelectionForFocus(
            owner.GetDocument(),
            &mut layout,
            selection,
            false,
        );
        layout.AbsoluteLayoutSelectionBounds()
    }

    fn ScrollSelectionRectToVisible(
        &self,
        node_id: u64,
        rect: foundation::gfx::Rect,
        alignment: interaction::default::core::editing::frame_selection::SelectionAlignment,
    ) {
        use interaction::default::core::editing::frame_selection::SelectionAlignment as A;
        use layoutng_assembly::native_scroll_source::{
            ScrollAlignment, ScrollAlignmentBehavior, ScrollIntoViewParams,
        };
        let Some(state) = self.Owners() else {
            return;
        };
        let native_alignment = match alignment {
            A::CenterAlways => ScrollAlignment::CenterAlways(),
            A::CenterIfNeeded => ScrollAlignment::CenterIfNeeded(),
            A::TopAlways => ScrollAlignment {
                rect_visible: ScrollAlignmentBehavior::Top,
                rect_hidden: ScrollAlignmentBehavior::Top,
                rect_partial: ScrollAlignmentBehavior::Top,
            },
            A::EdgeIfNeeded => ScrollAlignment::ToEdgeIfNeeded(),
        };
        let result = {
            let mut layout = state.Layout().borrow_mut();
            layout.NativeScrollSource(node_id).map(|mut source| {
                source.ScrollRectToVisible(
                    layoutng_assembly::layout_engine::LayoutRect {
                        x: rect.x() as f64,
                        y: rect.y() as f64,
                        width: rect.width() as f64,
                        height: rect.height() as f64,
                    },
                    ScrollIntoViewParams {
                        align_x: native_alignment,
                        align_y: native_alignment,
                        ..Default::default()
                    },
                )
            })
        };
        if let Some(result) = result {
            state.ApplyNativeScrollResult(result);
        }
    }
}
