//! Retained FrameSelection over the existing Layout editing-state owner.
//! Chromium: core/editing/frame_selection.cc:253-465,1390-1442;
//! selection_editor.cc:140-170. No selection endpoints are retained here.

pub use super::page_event_dispatcher::gesture_selection::GestureSelectionError;
use crate::ownership::InteractionDocument;
use interaction_default::core::editing::{
    editor::EditorFrameClient,
    frame_selection::{
        FrameSelection, FrameSelectionFeatures, RevealExtentOption, SelectionAlignment,
        SelectionEffects, SelectionFocusEffects, SelectionOwner, SelectionRevealSource,
        SelectionValue,
    },
    set_selection_options::{SetSelectionBy, SetSelectionOptions},
};
use layoutng_assembly::{
    editing_state::{DOMSelection, SelectionState},
    EditingPosition, EditingTextAffinity,
};
use std::rc::{Rc, Weak};

pub use super::page_editing_visible_position::{
    CaretSelectionForContentsHit, CompositionMarkerOwner, CompositionMarkerUpdate,
    CompositionTextQuery, CompositionTextResult, QueryCompositionText, TypingDeletionGeometryQuery,
    TypingDeletionGeometryResult,
};
pub use super::page_editing_visible_position::{
    DragCaretForSelection, FrameCaretForSelection, FrameCaretPosition,
};

/// A call-scoped value from the canonical owner, never another stored range.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameSelectionValue(pub Option<DOMSelection>);
impl SelectionValue for FrameSelectionValue {
    fn IsNone(&self) -> bool {
        self.0.is_none()
    }
}

#[derive(Clone, Copy, Default)]
pub struct FrameSelectionSettings {
    pub features: FrameSelectionFeatures,
    pub always_use_directional_selection: bool,
    pub center_align_revealed_selection: bool,
    pub remove_visible_selection_in_dom_selection: bool,
    pub text_overflow_clip_with_selection: bool,
}

/// Opaque native LayoutObject identity used only during the synchronous
/// SelectionEditor transaction. Anonymous containing blocks have no DOM id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectionTextOverflowOwner(pub usize);

/// Synchronous calls to the corresponding native source owners. The embedder
/// routes these leaves; selection decisions and notification order stay here.
pub enum FrameSelectionEffect {
    InvalidateInputCapsLockIndicatorPaint(u64),
    SetPointerCursor,
    InvalidateCompositionMarkerPaint(u64),
    MarkSelectionEndpointPaintingLayerNeedsRepaint(u64),
    CloseTyping,
    ClearTypingStyle,
    ClearDocumentCachedRange,
    SetContainsSelectionFocusFlag {
        owner: SelectionTextOverflowOwner,
        contains_focus: bool,
    },
    NotifyDisplayLock {
        old: Option<DOMSelection>,
        new: Option<DOMSelection>,
    },
    ScheduleVisualUpdateForVisualOverflow,
    StopCaretBlinkTimer,
    UpdateAppearance,
    SelectFrameElementInParentIfFullySelected,
    NotifyTextControlOfSelectionChange(SetSelectionBy),
    NotifyAccessibilityForSelectionChange,
    NotifyCompositorForSelectionChange,
    NotifyEventHandlerForSelectionChange,
    ScheduleTextControlSelectionchange,
    ScheduleDocumentSelectionchange,
    EnqueueLegacyDocumentSelectionchange,
    RespondToChangedSelection,
    DidChangeSelectionAfterCommand {
        is_caret: bool,
    },
    HandleAXEditableTextContentChanged {
        anchor: Option<DOMSelection>,
    },
    SpellCheckerRespondToChangedContents,
    SpellCheckerRespondToChangedSelection,
    FrameClientDidChangeContents,
    MarkInitialScrollWasScrolledByUser,
}

/// Required live frame/layout/Editor leaves. Missing native owners must be
/// exposed by assembly; none of these operations has a successful default.
pub trait PageFrameSelectionHost {
    fn CurrentKeyboardModifiers(
        &self,
    ) -> Result<crate::core::input::keyboard_event_manager::NativeKeyboardModifiers, &'static str>;
    fn ClipboardAccessPolicy(
        &self,
    ) -> interaction_default::core::editing::commands::clipboard_commands::ClipboardAccessPolicy
    {
        Default::default()
    }
    fn FocusEffects(&self) -> Rc<dyn SelectionFocusEffects<FrameSelectionValue>>;
    fn InteractionTextSettings(
        &self,
    ) -> Option<super::page_pointer_hit_test::InteractionTextSettings>;
    fn SuppressScrollAnchoringForNode(
        &self,
        node: Option<u64>,
    ) -> layoutng_assembly::layout_engine::SuppressScrollAnchorScope;
    fn SelectedHtmlForClipboard(&self,selection:DOMSelection)->Result<Vec<u16>,interaction_default::core::editing::serializers::styled_markup_serializer::SerializationError>;
    fn SerializeParagraphForMove(&self, selection: DOMSelection, constraining_ancestor: Option<u64>) -> Result<Vec<u16>, interaction_default::core::editing::serializers::styled_markup_serializer::SerializationError>;
    fn MarkupSanitizationHost(
        &self,
    ) -> Rc<dyn super::page_markup_sanitization::MarkupSanitizationHost>;
    fn ElementIsStyledSpanOrHtmlEquivalent(
        &self,
        node: u64,
    ) -> Result<bool, cssom::editing_style::EditingStyleError>;
    fn ApplyEditingStyle(
        &self,
        operation: cssom::editing_style::ApplyStyleOperation,
    ) -> Result<cssom::editing_style::ApplyStyleResult, cssom::editing_style::EditingStyleError>;
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
    >;
    fn ReplacementEditingStyle(
        &self,
        operation: cssom::editing_style::ReplacementStyleOperation,
    ) -> Result<cssom::editing_style::ReplacementStyleResult, cssom::editing_style::EditingStyleError>;
    fn EditingStyle(
        &self,
        operation: cssom::editing_style::EditingStyleOperation,
    ) -> Result<cssom::CSSInlineStyle, cssom::editing_style::EditingStyleError>;
    fn CompositionText(&self, query: CompositionTextQuery) -> Option<CompositionTextResult>;
    fn ReadPrimarySelection(
        &self,
    ) -> Result<
        interaction_default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
        interaction_default::core::editing::commands::clipboard_commands::ClipboardHostError,
    >;
    fn ReadClipboard(
        &self,
    ) -> Result<
        interaction_default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
        interaction_default::core::editing::commands::clipboard_commands::ClipboardHostError,
    >;
    fn WriteClipboard(
        &self,
        snapshot: &interaction_default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
    ) -> Result<
        (),
        interaction_default::core::editing::commands::clipboard_commands::ClipboardHostError,
    >;
    fn HasFrame(&self) -> bool;
    /// FocusController's retained frame identity comparison, independent of
    /// native widget/screen coordinate availability.
    fn IsFocusedOrMainFrame(&self) -> bool;
    /// LocalFrameView::ViewportToFrame for the WebLocalFrame entry point.
    fn ViewportToContentsPoint(&self, point: (i32, i32)) -> Option<(i32, i32)>;
    fn Settings(&self) -> FrameSelectionSettings;
    fn ApplyEffect(&self, effect: FrameSelectionEffect);
    fn CurrentEventType(&self) -> Option<String>;
    fn EditorFrameClient(&self) -> Option<Rc<dyn EditorFrameClient>>;
    fn EnclosingTextControlIsOutsideShadowTree(&self, selection: Option<DOMSelection>) -> bool;
    fn CaretSelectionForContentsPoint(&self, point: (i32, i32)) -> Option<DOMSelection>;
    fn UpdateStyleAndLayoutForSelection(&self);
    /// The deprecated source overload additionally forces display locks and
    /// updates layout before reading the same FrameSelection endpoints.
    fn ComputeVisibleSelectionRootEditableElementDeprecated(
        &self,
        selection: Option<DOMSelection>,
    ) -> Option<u64>;
    fn CreateVisibleSelection(&self, selection: Option<DOMSelection>) -> Option<DOMSelection>;
    fn VisibleSelectionRootEditableElement(&self, selection: Option<DOMSelection>) -> Option<u64>;
    fn VisibleSelectionStartAnchor(&self, selection: Option<DOMSelection>) -> Option<u64>;
    fn SelectionAtFirstPositionInOrBeforeNode(&self, node_id: u64) -> Option<DOMSelection>;
    fn HasLayoutObject(&self, node_id: u64) -> bool;
    fn ContainingBlockForTextOverflow(&self, node_id: u64) -> Option<SelectionTextOverflowOwner>;
    fn UpdateStyleAndLayoutForNode(&self, node_id: u64);
    fn AbsoluteCaretBounds(&self) -> foundation::gfx::Rect;
    fn AbsoluteCaretBoundsOfVisibleFocus(
        &self,
        selection: Option<DOMSelection>,
    ) -> foundation::gfx::Rect;
    fn AbsoluteSelectionBounds(&self) -> foundation::gfx::Rect;
    /// Native ScrollRectToVisible of the supplied selection rectangle, with
    /// CreateScrollIntoViewParams(alignment, alignment) for both axes.
    fn ScrollSelectionRectToVisible(
        &self,
        node_id: u64,
        rect: foundation::gfx::Rect,
        alignment: SelectionAlignment,
    );
}

struct CanonicalSelectionOwner {
    document: InteractionDocument,
    selections: Rc<SelectionState>,
    host: Rc<dyn PageFrameSelectionHost>,
}

/// Read the shared canonical frame endpoints, projecting an active text
/// control's existing cached UTF-16 offsets through its real inner editor.
/// Native geometry assembly uses the same accessor as FrameSelection itself.
pub fn CurrentDOMSelectionForFocus(
    document: &dom::Document,
    selections: &SelectionState,
) -> Option<DOMSelection> {
    if let Some(selection) = selections.DOMSelection() {
        return Some(selection);
    }
    if let Some(control) = selections.ActiveControl() {
        if let Some(selection) = selections.Get(control) {
            if let Some(control) = document.FindNodeById(control) {
                if let Some((anchor, focus)) = document
                    .TextControlInnerEditorPosition(control, selection.anchor)
                    .zip(document.TextControlInnerEditorPosition(control, selection.focus))
                {
                    return Some(DOMSelection {
                        anchor: EditingPosition::new(anchor.node_id, anchor.offset),
                        focus: EditingPosition::new(focus.node_id, focus.offset),
                        affinity: EditingTextAffinity::Downstream,
                    });
                }
            }
        }
    }
    None
}
impl SelectionOwner<FrameSelectionValue> for CanonicalSelectionOwner {
    fn Current(&self) -> FrameSelectionValue {
        let owner = self.document.borrow();
        FrameSelectionValue(CurrentDOMSelectionForFocus(
            owner.GetDocument(),
            &self.selections,
        ))
    }
    fn SetSelectionAndEndTyping(&self, selection: &FrameSelectionValue) {
        if let Some(selection) = selection.0 {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            for position in [selection.anchor, selection.focus] {
                let node = position.node_id.and_then(|id| document.FindNodeById(id));
                assert!(
                    node.is_some_and(
                        |node| document.NodeDocumentHandle(node) == document.RootHandle()
                    ),
                    "selection positions belong to the live Document"
                );
            }
        }
        let old = self.Current().0;
        self.host
            .ApplyEffect(FrameSelectionEffect::ClearDocumentCachedRange);
        // This is SelectionEditor's canonical publication, called only from
        // translated FrameSelection after its typing/cache/strategy work.
        // LayoutSelection owns the later derived Paint range; no text-only
        // or document-wide selection projection is manufactured here.
        self.selections.SetDOMSelection(selection.0, None);
        if self.host.Settings().text_overflow_clip_with_selection {
            let old_style_owner = old
                .and_then(|value| value.focus.node_id)
                .and_then(|node| self.host.ContainingBlockForTextOverflow(node));
            let new_focus = selection.0.and_then(|value| value.focus.node_id);
            let new_focus_is_editable = new_focus.is_some_and(|node| {
                let owner = self.document.borrow();
                foundation::editable::IsEditable(
                    &dom::persistent_document::DOMEditableFacts(owner.GetDocument()),
                    node,
                    foundation::editable::EditableFeatures::default(),
                )
            });
            let new_style_owner = new_focus
                .filter(|_| new_focus_is_editable)
                .and_then(|node| self.host.ContainingBlockForTextOverflow(node));
            if old_style_owner != new_style_owner {
                if let Some(owner) = old_style_owner {
                    self.host
                        .ApplyEffect(FrameSelectionEffect::SetContainsSelectionFocusFlag {
                            owner,
                            contains_focus: false,
                        });
                }
                if let Some(owner) = new_style_owner {
                    self.host
                        .ApplyEffect(FrameSelectionEffect::SetContainsSelectionFocusFlag {
                            owner,
                            contains_focus: true,
                        });
                }
            }
        }
    }
    fn EmptySelection(&self) -> FrameSelectionValue {
        FrameSelectionValue(None)
    }
    fn HasEditableRoot(&self) -> bool {
        self.host
            .VisibleSelectionRootEditableElement(self.Current().0)
            .is_some()
    }
    fn CaretSelectionForContentsPoint(&self, point: (i32, i32)) -> FrameSelectionValue {
        FrameSelectionValue(self.host.CaretSelectionForContentsPoint(point))
    }
    fn DocumentIdentity(&self) -> u128 {
        self.document.borrow().GetDocument().RootHandle().Identity()
    }
    fn IsAvailable(&self) -> bool {
        self.host.HasFrame() && self.document.borrow().GetDocument().IsActive()
    }
}

struct NativeSelectionEffects {
    owner: Weak<PageFrameSelection>,
    host: Rc<dyn PageFrameSelectionHost>,
}
impl NativeSelectionEffects {
    fn Emit(&self, effect: FrameSelectionEffect) {
        self.host.ApplyEffect(effect);
    }
}
impl SelectionEffects<FrameSelectionValue> for NativeSelectionEffects {
    fn AlwaysUseDirectionalSelection(&self) -> bool {
        self.host.Settings().always_use_directional_selection
    }
    fn ClearGranularityStrategy(&self) {
        self.owner
            .upgrade()
            .expect("live FrameSelection")
            .frame
            .ClearGranularityStrategy();
    }
    fn CloseTyping(&self) {
        if let Some(owner) = self.owner.upgrade() {
            owner.editor.ime_typing.typing.CloseTyping();
        }
        self.Emit(FrameSelectionEffect::CloseTyping);
    }
    fn ClearTypingStyle(&self) {
        if let Some(owner) = self.owner.upgrade() {
            owner.editor.ime_typing.editor.ClearTypingStyle();
        }
        self.Emit(FrameSelectionEffect::ClearTypingStyle);
    }
    fn NotifyDisplayLock(&self, old: &FrameSelectionValue, new: &FrameSelectionValue) {
        self.Emit(FrameSelectionEffect::NotifyDisplayLock {
            old: old.0,
            new: new.0,
        });
    }
    fn ScheduleVisualUpdateForVisualOverflow(&self) {
        let owner = self.owner.upgrade().expect("live FrameSelection");
        owner
            .selections
            .SetSelectionHandlesVisible(owner.frame.is_handle_visible.get());
        self.Emit(FrameSelectionEffect::ScheduleVisualUpdateForVisualOverflow);
    }
    fn SetFocusedNodeIfNeeded(&self) {
        if let Some(owner) = self.owner.upgrade() {
            if let Some(actions) = owner.focus_actions.get().and_then(Weak::upgrade) {
                actions.SetFocusedNodeForSelection();
            }
        }
    }
    fn StopCaretBlinkTimer(&self) {
        self.Emit(FrameSelectionEffect::StopCaretBlinkTimer);
    }
    fn UpdateAppearance(&self) {
        self.Emit(FrameSelectionEffect::UpdateAppearance);
    }
    fn SelectFrameElementInParentIfFullySelected(&self) {
        self.Emit(FrameSelectionEffect::SelectFrameElementInParentIfFullySelected);
    }
    fn NotifyTextControlOfSelectionChange(&self, by: SetSelectionBy) {
        if let Some(owner) = self.owner.upgrade() {
            owner.CacheTextControlSelection();
        }
        self.Emit(FrameSelectionEffect::NotifyTextControlOfSelectionChange(by));
    }
    fn CenterAlignRevealedSelection(&self) -> bool {
        self.host.Settings().center_align_revealed_selection
    }
    fn RevealSelection(&self, alignment: SelectionAlignment, extent: RevealExtentOption) {
        self.owner
            .upgrade()
            .expect("live FrameSelection")
            .RevealSelection(alignment, extent);
    }
    fn NotifyAccessibilityForSelectionChange(&self) {
        self.Emit(FrameSelectionEffect::NotifyAccessibilityForSelectionChange);
    }
    fn NotifyCompositorForSelectionChange(&self) {
        self.Emit(FrameSelectionEffect::NotifyCompositorForSelectionChange);
    }
    fn NotifyEventHandlerForSelectionChange(&self) {
        self.Emit(FrameSelectionEffect::NotifyEventHandlerForSelectionChange);
    }
    fn EnclosingTextControlIsOutsideShadowTree(&self) -> bool {
        let owner = self.owner.upgrade().expect("live FrameSelection");
        self.host
            .EnclosingTextControlIsOutsideShadowTree(owner.Current())
    }
    fn ScheduleTextControlSelectionchange(&self) {
        self.Emit(FrameSelectionEffect::ScheduleTextControlSelectionchange);
    }
    fn ScheduleDocumentSelectionchange(&self) {
        self.Emit(FrameSelectionEffect::ScheduleDocumentSelectionchange);
    }
    fn EnqueueLegacyDocumentSelectionchange(&self) {
        self.Emit(FrameSelectionEffect::EnqueueLegacyDocumentSelectionchange);
    }
    fn RespondToChangedSelection(&self) {
        self.Emit(FrameSelectionEffect::RespondToChangedSelection);
        if let Some(owner) = self.owner.upgrade() {
            owner
                .editor
                .ime_typing
                .editor
                .SetStartNewKillRingSequence(true);
        }
    }
    fn DidChangeSelectionAfterCommand(&self, is_caret: bool) {
        self.Emit(FrameSelectionEffect::DidChangeSelectionAfterCommand { is_caret });
    }
    fn GetEditorFrameClient(&self) -> Option<Rc<dyn EditorFrameClient>> {
        self.host.EditorFrameClient()
    }
    fn HandleAXEditableTextContentChanged(&self, anchor: Option<&FrameSelectionValue>) {
        self.Emit(FrameSelectionEffect::HandleAXEditableTextContentChanged {
            anchor: anchor.and_then(|v| v.0),
        });
    }
    fn SpellCheckerRespondToChangedContents(&self) {
        self.Emit(FrameSelectionEffect::SpellCheckerRespondToChangedContents);
    }
    fn SpellCheckerRespondToChangedSelection(&self) {
        self.Emit(FrameSelectionEffect::SpellCheckerRespondToChangedSelection);
    }
    fn FrameClientDidChangeContents(&self) {
        self.Emit(FrameSelectionEffect::FrameClientDidChangeContents);
    }
    fn CurrentEventType(&self) -> Option<String> {
        self.host.CurrentEventType()
    }
}

/// One instance per Document's interaction assembly. Only source derived
/// FrameSelection flags/caches live here; endpoints remain in SelectionState.
pub struct PageFrameSelection {
    document: InteractionDocument,
    editor: Rc<super::page_editing_owners::PageEditingOwners>,
    frame: FrameSelection<FrameSelectionValue>,
    selections: Rc<SelectionState>,
    host: Rc<dyn PageFrameSelectionHost>,
    focus_actions:
        std::cell::OnceCell<Weak<super::page_focus_navigation::PageFocusNavigationActions>>,
}
impl PageFrameSelection {
    pub(crate) fn SelectAll<C: interaction_default::core::editing::frame_selection_select_all::FrameSelectionSelectAllContext<FrameSelectionValue>>(&self, context: &C, by: SetSelectionBy) -> Result<(), &'static str>{
        self.frame.SelectAll(
            context,
            by,
            self.RemoveVisibleSelectionInDOMSelectionEnabled(),
        )
    }
    pub(crate) fn ClearSelection(&self) {
        self.frame.Clear();
    }
    /// editing_commands_utilities.cc:796-829, shared by UndoStep replay.
    pub(crate) fn ChangeSelectionAfterCommand(
        &self,
        selection: DOMSelection,
        options: SetSelectionOptions,
    ) {
        if selection.anchor.IsNull() {
            return;
        }
        self.frame.ChangeSelectionAfterCommand(
            &FrameSelectionValue(Some(selection)),
            |s| s.0.is_none_or(|s| s.anchor == s.focus),
            options,
        );
    }
    pub(crate) fn RespondToChangedContents(&self) {
        self.editor
            .ime_typing
            .editor
            .RespondToChangedContents(self, self.Current().as_ref());
    }
    pub(crate) fn Modify<
        C: interaction_default::core::editing::frame_selection_modify::FrameSelectionCommandContext<
            FrameSelectionValue,
        >,
    >(
        &self,
        context: &C,
        alteration: interaction_default::core::editing::selection_modifier::SelectionModifyAlteration,
        direction: interaction_default::core::editing::commands::editor_command_input_type::SelectionModifyDirection,
        granularity: interaction_default::core::editing::text_granularity::TextGranularity,
    ) -> bool {
        self.frame.Modify(
            context,
            alteration,
            direction,
            granularity,
            SetSelectionBy::kUser,
        )
    }
    fn CacheTextControlSelection(&self) {
        let Some(selection) = self.Current() else {
            return;
        };
        let document = &self.document;
        let owner = document.borrow();
        let document = owner.GetDocument();
        let Some(control) = super::page_focus_controller::EnclosingTextControlForSelectionForFocus(
            document,
            Some(selection),
        ) else {
            return;
        };
        let Some(index) = document.FindNodeById(control) else {
            return;
        };
        let offset = |position: EditingPosition| {
            let point = NativeEditingBoundaryPoint(document, position)?;
            document.TextControlInnerEditorOffset(index, point)
        };
        if let Some((anchor, focus)) = offset(selection.anchor).zip(offset(selection.focus)) {
            self.selections.SetActiveControlSelection(
                control,
                layoutng_assembly::editing_state::Selection { anchor, focus },
            );
        }
    }
    /// Reconcile the package-side text-control discriminator after Chromium's
    /// synchronous mouse-press SelectionController transaction. The resident
    /// FrameSelection remains the source of endpoints; this only projects its
    /// inner-editor positions back onto the same control's cached UTF-16 owner.
    pub(crate) fn ActivateFocusedTextControlSelection(&self, focused: Option<u64>) {
        let Some(focused) = focused.filter(|&id| {
            super::page_editing_owners::IsTextControl(&self.document, id)
        }) else {
            return;
        };
        let projected = self.Current().and_then(|selection| {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            let control = super::page_focus_controller::EnclosingTextControlForSelectionForFocus(
                document,
                Some(selection),
            )?;
            if control != focused {
                return None;
            }
            let index = document.FindNodeById(control)?;
            let offset = |position: EditingPosition| {
                let point = NativeEditingBoundaryPoint(document, position)?;
                document.TextControlInnerEditorOffset(index, point)
            };
            let (anchor, focus) = offset(selection.anchor).zip(offset(selection.focus))?;
            Some(layoutng_assembly::editing_state::Selection { anchor, focus })
        });
        if let Some(selection) = projected {
            self.selections
                .SetActiveControlSelection(focused, selection);
        } else {
            // If the click did not move FrameSelection (for example, it
            // returned to the same cached caret), retain Chromium's existing
            // text-control selection instead of leaving a non-control DOM
            // selection as the active editor owner.
            self.editor.Focus(&self.document, focused);
        }
    }
    pub(crate) fn NotifyAccessibilityOfDeletionInTextField(&self, selection: DOMSelection) {
        self.host
            .ApplyEffect(FrameSelectionEffect::HandleAXEditableTextContentChanged {
                anchor: Some(selection),
            });
    }
    pub(crate) fn InteractionTextSettings(
        &self,
    ) -> Option<super::page_pointer_hit_test::InteractionTextSettings> {
        self.host.InteractionTextSettings()
    }
    pub(crate) fn SuppressScrollAnchoringForNode(
        &self,
        node: Option<u64>,
    ) -> layoutng_assembly::layout_engine::SuppressScrollAnchorScope {
        self.host.SuppressScrollAnchoringForNode(node)
    }
    pub(crate) fn SelectionIsDirectional(&self) -> bool {
        self.frame.is_directional.get()
    }
    pub fn SelectedHtmlForClipboard(&self)->Result<Vec<u16>,interaction_default::core::editing::serializers::styled_markup_serializer::SerializationError>{
        match self.Current() {
            Some(selection) => self.host.SelectedHtmlForClipboard(selection),
            None => Ok(Vec::new()),
        }
    }
    pub(crate) fn SerializeParagraphForMove(&self, selection: DOMSelection, constraining_ancestor: Option<u64>) -> Result<Vec<u16>, interaction_default::core::editing::serializers::styled_markup_serializer::SerializationError>{
        self.host
            .SerializeParagraphForMove(selection, constraining_ancestor)
    }
    pub(crate) fn MarkupSanitizationHost(
        &self,
    ) -> Rc<dyn super::page_markup_sanitization::MarkupSanitizationHost> {
        self.host.MarkupSanitizationHost()
    }
    pub(crate) fn ElementIsStyledSpanOrHtmlEquivalent(
        &self,
        node: u64,
    ) -> Result<bool, cssom::editing_style::EditingStyleError> {
        self.host.ElementIsStyledSpanOrHtmlEquivalent(node)
    }
    pub(crate) fn SetPointerCursor(&self) {
        self.host
            .ApplyEffect(FrameSelectionEffect::SetPointerCursor);
    }
    pub(crate) fn IsFocusedOrMainFrame(&self) -> bool {
        self.host.IsFocusedOrMainFrame()
    }
    pub(crate) fn FrameIsFocused(&self) -> bool {
        self.frame.FrameIsFocused()
    }
    pub(crate) fn FrameIsFocusedAndActive(&self) -> bool {
        self.frame.FrameIsFocusedAndActive()
    }
    pub(crate) fn NotifyCapsLockStateMayHaveChanged(&self) -> Result<(), &'static str> {
        let focused = {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            let focused = document
                .FocusedUserActionElements()
                .next()
                .map(|node| document.Node(node).Id());
            focused
        };
        super::page_keyboard_event_manager::UpdatePasswordCapsLockIndicator(
            &self.document,
            focused,
            self.FrameIsFocusedAndActive(),
            self.host.as_ref(),
        )
    }
    pub(crate) fn SelectionControllerFlags(
        &self,
    ) -> (
        interaction_default::core::editing::text_granularity::TextGranularity,
        bool,
    ) {
        (
            self.frame.granularity.get(),
            self.frame.should_shrink_next_tap.get(),
        )
    }
    pub(crate) fn SetSelectionAndEndTyping(&self, selection: Option<DOMSelection>) {
        self.frame
            .SetSelectionAndEndTyping(&FrameSelectionValue(selection));
    }
    pub(crate) fn NotifyTextControlOfSelectionChange(&self) {
        self.frame
            .effects
            .NotifyTextControlOfSelectionChange(SetSelectionBy::kUser);
    }
    pub(crate) fn SelectFrameElementInParentIfFullySelected(&self) {
        self.frame
            .effects
            .SelectFrameElementInParentIfFullySelected();
    }
    pub(crate) fn ContainsWithSource<
        C: interaction_default::core::editing::frame_selection::SelectionContainmentSource<
            FrameSelectionValue,
        >,
    >(
        &self,
        source: &C,
        point: &foundation::PhysicalOffset,
    ) -> bool {
        self.frame.Contains(source, point)
    }
    pub fn ContainsPoint(
        self: &Rc<Self>,
        document: InteractionDocument,
        geometry: Rc<dyn super::page_pointer_hit_test::PagePointerGeometry>,
        settings: super::page_pointer_hit_test::InteractionTextSettings,
        point: foundation::PhysicalOffset,
        has_layout_view: bool,
    ) -> Result<bool, super::page_event_dispatcher::gesture_selection::GestureSelectionError> {
        super::page_event_dispatcher::gesture_selection::ContainsPoint(
            self.clone(),
            document,
            geometry,
            settings,
            point,
            has_layout_view,
        )
    }
    pub(crate) fn GestureSelectionFlags(&self) -> (bool, bool) {
        (
            self.frame.is_handle_visible.get(),
            self.frame.is_directional.get(),
        )
    }
    pub(crate) fn IsAvailable(&self) -> bool {
        self.frame.owner.IsAvailable()
    }
    pub(crate) fn PrepareEditing(&self) {
        self.host.UpdateStyleAndLayoutForSelection();
    }
    pub(crate) fn ReplacementEditingStyle(
        &self,
        operation: cssom::editing_style::ReplacementStyleOperation,
    ) -> Result<cssom::editing_style::ReplacementStyleResult, cssom::editing_style::EditingStyleError>
    {
        self.host.ReplacementEditingStyle(operation)
    }
    pub(crate) fn EditingStyle(
        &self,
        operation: cssom::editing_style::EditingStyleOperation,
    ) -> Result<cssom::CSSInlineStyle, cssom::editing_style::EditingStyleError> {
        self.host.EditingStyle(operation)
    }
    pub(crate) fn ApplyEditingStyle(
        &self,
        operation: cssom::editing_style::ApplyStyleOperation,
    ) -> Result<cssom::editing_style::ApplyStyleResult, cssom::editing_style::EditingStyleError>
    {
        self.host.ApplyEditingStyle(operation)
    }
    pub(crate) fn EditingStyleApplication(
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
        self.host.EditingStyleApplication(declaration, originals)
    }
    pub(crate) fn CompositionText(
        &self,
        query: CompositionTextQuery,
    ) -> Option<CompositionTextResult> {
        self.host.CompositionText(query)
    }
    pub(crate) fn InvalidateCompositionMarkerPaint(&self, node: u64) {
        self.host
            .ApplyEffect(FrameSelectionEffect::InvalidateCompositionMarkerPaint(node));
    }
    /// selection_controller.cc:145-160, invalidating the endpoint's original
    /// painting layer rather than rebuilding selection or layout geometry.
    pub(crate) fn MarkSelectionEndpointsForRepaint(&self, selection: Option<DOMSelection>) {
        if let Some(selection) = selection {
            for endpoint in [selection.anchor, selection.focus] {
                if let Some(node) = endpoint.node_id {
                    self.host.ApplyEffect(
                        FrameSelectionEffect::MarkSelectionEndpointPaintingLayerNeedsRepaint(node),
                    );
                }
            }
        }
    }
    pub(crate) fn ReadPrimarySelection(
        &self,
    ) -> Result<
        interaction_default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
        interaction_default::core::editing::commands::clipboard_commands::ClipboardHostError,
    > {
        self.host.ReadPrimarySelection()
    }
    pub(crate) fn ReadClipboard(
        &self,
    ) -> Result<
        interaction_default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
        interaction_default::core::editing::commands::clipboard_commands::ClipboardHostError,
    > {
        self.host.ReadClipboard()
    }
    pub(crate) fn ClipboardAccessPolicy(
        &self,
    ) -> interaction_default::core::editing::commands::clipboard_commands::ClipboardAccessPolicy
    {
        self.host.ClipboardAccessPolicy()
    }
    pub(crate) fn WriteClipboard(
        &self,
        snapshot: &interaction_default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
    ) -> Result<
        (),
        interaction_default::core::editing::commands::clipboard_commands::ClipboardHostError,
    > {
        self.host.WriteClipboard(snapshot)
    }
    pub(crate) fn new(
        document: InteractionDocument,
        selections: Rc<SelectionState>,
        editor: Rc<super::page_editing_owners::PageEditingOwners>,
        host: Rc<dyn PageFrameSelectionHost>,
    ) -> Rc<Self> {
        let focus_effects = host.FocusEffects();
        let owner = Rc::new_cyclic(|weak| Self {
            document: document.clone(),
            editor,
            frame: FrameSelection::new(
                Rc::new(CanonicalSelectionOwner {
                    document,
                    selections: selections.clone(),
                    host: host.clone(),
                }),
                Rc::new(NativeSelectionEffects {
                    owner: weak.clone(),
                    host: host.clone(),
                }),
                host.Settings().features,
            ),
            selections,
            host,
            focus_actions: std::cell::OnceCell::new(),
        });
        owner.frame.BindFocusEffects(focus_effects);
        owner
    }
    pub(crate) fn BindFocusActions(
        &self,
        actions: &Rc<super::page_focus_navigation::PageFocusNavigationActions>,
    ) {
        assert!(
            self.focus_actions.set(Rc::downgrade(actions)).is_ok(),
            "one FrameSelection focus transaction owner"
        );
    }
    pub(crate) fn SetFrameIsFocused(&self, focused: bool) {
        self.frame.SetFrameIsFocused(focused);
    }
    pub(crate) fn PageActivationChanged(&self) {
        self.frame.PageActivationChanged();
    }
    /// LocalFrame::PositionForPoint borrows this same resident host query
    /// without changing FrameSelection during drag-target updates.
    pub(crate) fn CaretSelectionForViewportPoint(&self, point: (i32, i32)) -> Option<DOMSelection> {
        self.host.UpdateStyleAndLayoutForSelection();
        self.host
            .CaretSelectionForContentsPoint(self.host.ViewportToContentsPoint(point)?)
    }
    pub fn MoveCaretSelection(&self, point: (i32, i32)) {
        // WebLocalFrameImpl::MoveCaretSelection updates layout before mapping
        // widget viewport coordinates through the live VisualViewport owner.
        self.host.UpdateStyleAndLayoutForSelection();
        if let Some(point) = self.host.ViewportToContentsPoint(point) {
            self.frame.MoveCaretSelection(point);
        }
    }
    pub fn Current(&self) -> Option<DOMSelection> {
        self.frame.owner.Current().0
    }
    /// Parent LocalFrame selection after the source has focused its Frame and
    /// revalidated the original connected owner element following listeners.
    pub fn SelectFrameOwnerElement(&self, owner: dom::persistent_document::DOMOwnerHandle) {
        let live = {
            let document = self.document.borrow();
            let document = document.GetDocument();
            document.FindNodeById(owner.Id()).is_some_and(|node| {
                document.NodeHandle(node) == owner
                    && document.IsConnectedToDocument(node)
                    && document.NodeDocumentHandle(node) == document.RootHandle()
            })
        };
        if !live {
            return;
        }
        let position = |anchor| EditingPosition::WithAnchor(owner.Id(), 0, anchor);
        self.SetSelection(
            Some(DOMSelection {
                anchor: position(layoutng_assembly::EditingPositionAnchorType::BeforeAnchor),
                focus: position(layoutng_assembly::EditingPositionAnchorType::AfterAnchor),
                affinity: EditingTextAffinity::Downstream,
            }),
            SetSelectionOptions::default(),
        );
    }
    pub(crate) fn SetSelection(
        &self,
        selection: Option<DOMSelection>,
        options: SetSelectionOptions,
    ) {
        self.frame
            .SetSelection(&FrameSelectionValue(selection), options);
    }
    pub(crate) fn DidChangeFocus(&self) {
        self.frame.DidChangeFocus();
    }
    pub fn MarkCacheDirty(&self) {
        self.frame.MarkCacheDirty();
    }
    pub(crate) fn VisibleSelectionRootEditableElement(&self) -> Option<u64> {
        self.host
            .ComputeVisibleSelectionRootEditableElementDeprecated(self.Current())
    }
    pub(crate) fn SelectionAtFirstPositionInOrBeforeNode(
        &self,
        node_id: u64,
    ) -> Option<DOMSelection> {
        self.host.SelectionAtFirstPositionInOrBeforeNode(node_id)
    }
    pub(crate) fn RemoveVisibleSelectionInDOMSelectionEnabled(&self) -> bool {
        self.host
            .Settings()
            .remove_visible_selection_in_dom_selection
    }
    pub(crate) fn CreateVisibleSelection(
        &self,
        selection: Option<DOMSelection>,
    ) -> Option<DOMSelection> {
        self.host.CreateVisibleSelection(selection)
    }
    pub(crate) fn RevealSelection(
        &self,
        alignment: SelectionAlignment,
        extent: RevealExtentOption,
    ) {
        self.frame.RevealSelection(self, alignment, extent);
    }
}
impl interaction_default::core::editing::editor::EditorChangedContentsContext<DOMSelection>
    for PageFrameSelection
{
    fn HandleAXEditableTextContentChanged(&self, anchor: Option<&DOMSelection>) {
        self.host
            .ApplyEffect(FrameSelectionEffect::HandleAXEditableTextContentChanged {
                anchor: anchor.copied(),
            });
    }
    fn SpellCheckerRespondToChangedContents(&self) {
        self.host
            .ApplyEffect(FrameSelectionEffect::SpellCheckerRespondToChangedContents);
    }
    fn FrameClientDidChangeContents(&self) {
        self.host
            .ApplyEffect(FrameSelectionEffect::FrameClientDidChangeContents);
    }
}
impl SelectionRevealSource<FrameSelectionValue> for PageFrameSelection {
    type Node = u64;
    fn UpdateStyleAndLayoutForSelection(&self) {
        self.host.UpdateStyleAndLayoutForSelection();
    }
    fn ComputeVisibleSelectionInDOMTree(
        &self,
        selection: &FrameSelectionValue,
    ) -> FrameSelectionValue {
        FrameSelectionValue(self.host.CreateVisibleSelection(selection.0))
    }
    fn SelectionIsCaret(&self, selection: &FrameSelectionValue) -> bool {
        selection.0.is_some_and(|s| s.anchor == s.focus)
    }
    fn VisibleSelectionStartAnchor(&self, selection: &FrameSelectionValue) -> Option<u64> {
        self.host.VisibleSelectionStartAnchor(selection.0)
    }
    fn MarkInitialScrollWasScrolledByUser(&self) {
        self.host
            .ApplyEffect(FrameSelectionEffect::MarkInitialScrollWasScrolledByUser);
    }
    fn HasLayoutObject(&self, node: u64) -> bool {
        self.host.HasLayoutObject(node)
    }
    fn UpdateStyleAndLayoutForNode(&self, node: u64) {
        self.host.UpdateStyleAndLayoutForNode(node);
    }
    fn AbsoluteCaretBounds(&self) -> foundation::gfx::Rect {
        self.host.AbsoluteCaretBounds()
    }
    fn AbsoluteCaretBoundsOfVisibleFocus(
        &self,
        selection: &FrameSelectionValue,
    ) -> foundation::gfx::Rect {
        self.host.AbsoluteCaretBoundsOfVisibleFocus(selection.0)
    }
    fn SetLayoutSelectionHasPendingSelection(&self) {
        self.selections
            .layout_selection
            .borrow_mut()
            .SetHasPendingSelection();
    }
    fn AbsoluteSelectionBounds(&self) -> foundation::gfx::Rect {
        self.host.AbsoluteSelectionBounds()
    }
    fn ScrollRectToVisible(
        &self,
        node: u64,
        rect: foundation::gfx::Rect,
        alignment: SelectionAlignment,
    ) {
        self.host
            .ScrollSelectionRectToVisible(node, rect, alignment);
    }
}

/// Position::ToOffsetInAnchor for native package-boundary positions. This is a
/// value projection over the same live DOM identities, not a selection owner.
pub(crate) fn NativeEditingBoundaryPoint(
    document: &dom::Document,
    position: EditingPosition,
) -> Option<dom::DOMBoundaryPoint> {
    use layoutng_assembly::EditingPositionAnchorType as A;
    let node = position.node_id?;
    let index = document.FindNodeById(node)?;
    let (node_id, offset) = match position.anchor {
        A::OffsetInAnchor => (node, position.offset),
        A::AfterChildren => (node, document.Node(index).Children().len()),
        A::BeforeAnchor | A::AfterAnchor => {
            let parent = document.Node(index).Parent()?;
            let index = document
                .Node(parent)
                .Children()
                .iter()
                .position(|child| *child == index)?;
            (
                document.Node(parent).Id(),
                index + usize::from(position.anchor == A::AfterAnchor),
            )
        }
    };
    Some(dom::DOMBoundaryPoint { node_id, offset })
}
