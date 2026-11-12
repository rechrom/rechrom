// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source frame_selection.cc:583-715; blink_ax_event_intent.cc:214-344.
use super::{
    commands::editor_command_input_type::SelectionModifyDirection,
    frame_selection::{FrameSelection, SelectionValue},
    position::TextAffinity,
    selection_modifier::{SelectionModifier, SelectionModifierContext, SelectionModifyAlteration},
    set_selection_options::{SetSelectionBy, SetSelectionOptions},
    text_granularity::TextGranularity,
    visible_selection::VisibleSelection,
};
use foundation::TextDirection;
use interaction_dispatch::core::dom::{
    events::event_dispatch_result::DispatchEventResult,
    events::event_dispatcher::{
        AXSelectionCommand, AXSelectionIntent, AXSelectionMoveDirection, AXSelectionTextBoundary,
    },
};
use std::cell::Cell;

/// Values retained by FrameSelection while the call-scoped geometry borrow is
/// released for selectstart. No document or Layout owner crosses dispatch.
#[derive(Clone, Copy)]
pub struct SelectionModificationState {
    pub x_pos: foundation::LayoutUnit,
    pub caret_bidi_level: Option<u8>,
    pub entered_bidi_run: bool,
    pub directional: bool,
    pub bidi_visual_order: bool,
}
pub struct PreparedSelectionModification<S> {
    pub selection: S,
    pub modified: bool,
    pub spatial_navigation: bool,
    pub intent: AXSelectionIntent,
    pub state: SelectionModificationState,
}
pub trait FrameSelectionCommandContext<S: SelectionValue> {
    type AXIntentScope;
    fn PrepareModification(
        &self,
        selection: &S,
        state: SelectionModificationState,
        alter: SelectionModifyAlteration,
        direction: SelectionModifyDirection,
        granularity: TextGranularity,
    ) -> Option<PreparedSelectionModification<S>>;
    fn IsRange(&self, selection: &S) -> bool;
    fn CurrentVisibleSelectionIsCaret(&self) -> bool;
    fn DispatchSelectStart(&self) -> DispatchEventResult;
    fn CanonicalizeSelection(&self, selection: &S) -> S;
    fn BeginAXSelectionIntent(&self, intent: AXSelectionIntent) -> Option<Self::AXIntentScope>;
}

/// The same modifier path as Modify below, with only immutable results leaving
/// the resident geometry call. Publication still belongs to FrameSelection.
pub fn PrepareSelectionModification<C: SelectionModifierContext, S>(
    geometry: &C,
    selection: VisibleSelection<C::Node>,
    mut state: SelectionModificationState,
    alter: SelectionModifyAlteration,
    direction: SelectionModifyDirection,
    granularity: TextGranularity,
    build: impl FnOnce(&VisibleSelection<C::Node>) -> S,
) -> PreparedSelectionModification<S> {
    let mut modifier = SelectionModifier::new(selection, state.x_pos);
    if state.bidi_visual_order {
        modifier.SetCaretBidiLevel(state.caret_bidi_level);
        modifier.SetEnteredBidiRun(state.entered_bidi_run);
    }
    modifier.SetSelectionIsDirectional(state.directional);
    let modified = modifier.Modify(geometry, alter, direction, granularity);
    let intent = selection_intent(
        alter,
        direction,
        granularity,
        modifier.DirectionOfSelection(geometry),
        geometry.Behavior().ShouldSkipSpaceWhenMovingRight(),
    );
    let mut result = modifier.Selection(geometry);
    if state.bidi_visual_order {
        let previous = state.caret_bidi_level;
        state.caret_bidi_level = modifier.CaretBidiLevel();
        state.entered_bidi_run = modifier.EnteredBidiRun();
        if let Some(level) = state.caret_bidi_level {
            result.affinity = if previous == Some(level) && level & 1 == 0 {
                TextAffinity::Upstream
            } else {
                TextAffinity::Downstream
            };
        }
    }
    state.x_pos = modifier.XPosForVerticalArrowNavigation();
    PreparedSelectionModification {
        selection: build(&result),
        modified,
        spatial_navigation: geometry.IsSpatialNavigationEnabled(),
        intent,
        state,
    }
}
fn selection_intent(
    alter: SelectionModifyAlteration,
    direction: SelectionModifyDirection,
    granularity: TextGranularity,
    text_direction: TextDirection,
    skip_spaces: bool,
) -> AXSelectionIntent {
    use AXSelectionMoveDirection as D;
    use AXSelectionTextBoundary as B;
    let direction = match direction {
        SelectionModifyDirection::Backward => D::Backward,
        SelectionModifyDirection::Forward => D::Forward,
        SelectionModifyDirection::Left => {
            if text_direction == TextDirection::kLtr {
                D::Backward
            } else {
                D::Forward
            }
        }
        SelectionModifyDirection::Right => {
            if text_direction == TextDirection::kLtr {
                D::Forward
            } else {
                D::Backward
            }
        }
    };
    let backward = direction == D::Backward;
    let boundary = match granularity {
        TextGranularity::kCharacter => B::Character,
        TextGranularity::kWord => {
            if backward || skip_spaces {
                B::WordStart
            } else {
                B::WordEnd
            }
        }
        TextGranularity::kSentence => B::SentenceStart,
        TextGranularity::kLine => B::LineStart,
        TextGranularity::kParagraph => B::ParagraphStart,
        TextGranularity::kSentenceBoundary => {
            if backward {
                B::SentenceStart
            } else {
                B::SentenceEnd
            }
        }
        TextGranularity::kLineBoundary => {
            if backward {
                B::LineStart
            } else {
                B::LineEnd
            }
        }
        TextGranularity::kParagraphBoundary => {
            if backward {
                B::ParagraphStart
            } else {
                B::ParagraphEnd
            }
        }
        TextGranularity::kDocumentBoundary => B::WebPage,
    };
    AXSelectionIntent {
        command: if alter == SelectionModifyAlteration::Extend {
            AXSelectionCommand::ExtendSelection
        } else {
            AXSelectionCommand::MoveSelection
        },
        direction,
        boundary,
    }
}
struct ResetModified<'a>(&'a Cell<bool>, bool);
impl Drop for ResetModified<'_> {
    fn drop(&mut self) {
        self.0.set(self.1)
    }
}
impl<S: SelectionValue> FrameSelection<S> {
    pub fn Modify<C: FrameSelectionCommandContext<S>>(
        &self,
        ctx: &C,
        alter: SelectionModifyAlteration,
        direction: SelectionModifyDirection,
        granularity: TextGranularity,
        by: SetSelectionBy,
    ) -> bool {
        let document = self.owner.DocumentIdentity();
        let Some(result) = ctx.PrepareModification(
            &self.owner.Current(),
            SelectionModificationState {
                x_pos: self.x_pos_for_vertical_arrow_navigation.get(),
                caret_bidi_level: self.caret_bidi_level.get(),
                entered_bidi_run: self.entered_bidi_run.get(),
                directional: self.is_directional.get(),
                bidi_visual_order: self.features.bidi_visual_order_caret_movement,
            },
            alter,
            direction,
            granularity,
        ) else {
            return false;
        };
        if by == SetSelectionBy::kUser && ctx.IsRange(&result.selection) {
            if ctx.CurrentVisibleSelectionIsCaret()
                && ctx.DispatchSelectStart() != DispatchEventResult::kNotCanceled
            {
                return false;
            }
        }
        if !self.owner.IsAvailable() || document != self.owner.DocumentIdentity() {
            return false;
        }
        if !result.modified {
            return by != SetSelectionBy::kSystem && !result.spatial_navigation;
        }
        let _modified_scope = ResetModified(
            &self.is_being_modified,
            self.is_being_modified.replace(true),
        );
        let _intent = ctx.BeginAXSelectionIntent(result.intent);
        let directional = alter == SelectionModifyAlteration::Extend
            || self.effects.AlwaysUseDirectionalSelection();
        // Source recomputes the DOM visible conversion after selectstart reentry.
        if self.features.bidi_visual_order_caret_movement {
            self.caret_bidi_level.set(result.state.caret_bidi_level);
            self.entered_bidi_run.set(result.state.entered_bidi_run);
        }
        self.SetSelection(
            &ctx.CanonicalizeSelection(&result.selection),
            SetSelectionOptions {
                should_close_typing: true,
                should_clear_typing_style: true,
                set_selection_by: by,
                is_directional: directional,
                ..Default::default()
            },
        );
        if matches!(
            granularity,
            TextGranularity::kLine | TextGranularity::kParagraph
        ) {
            self.x_pos_for_vertical_arrow_navigation
                .set(result.state.x_pos);
        }
        if by == SetSelectionBy::kUser {
            self.granularity.set(TextGranularity::kCharacter);
        }
        self.effects.ScheduleVisualUpdateForVisualOverflow();
        true
    }
}
