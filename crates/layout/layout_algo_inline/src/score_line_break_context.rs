// C++: layoutng_inline/score_line_break_context.h.
#![allow(non_snake_case, non_upper_case_globals)]

use foundation::WtfSizeT;

use crate::line_break_point::LineBreakPoint;
use crate::line_info_list::{LineInfoList, LineInfoListOf};

// cpp: layoutng_inline/score_line_break_context.h:16-20
pub const kMaxLinesForBalance: WtfSizeT = 6;
pub const kMaxLinesForOptimal: WtfSizeT = 4;
pub type LineBreakPoints = Vec<LineBreakPoint>;

// cpp: layoutng_inline/score_line_break_context.h:53-57
pub struct ScoreLineBreakContextState {
    line_break_points_: LineBreakPoints,
    line_break_points_index_: WtfSizeT,
    is_suspended_: bool,
}

impl Default for ScoreLineBreakContextState {
    fn default() -> Self {
        Self {
            line_break_points_: Vec::with_capacity(kMaxLinesForBalance as usize),
            line_break_points_index_: 0,
            is_suspended_: false,
        }
    }
}

// cpp: layoutng_inline/score_line_break_context.h:22-32,49-58
// The C++ derived object passes its own LineInfoList member to a base-class
// reference. An object-safe trait lets a Rust owner expose that member without
// storing a self-referential reference.
pub trait ScoreLineBreakContext {
    fn State(&self) -> &ScoreLineBreakContextState;
    fn StateMut(&mut self) -> &mut ScoreLineBreakContextState;
    fn GetLineInfoList(&mut self) -> &mut dyn LineInfoList;
    // The score breaker needs mutable access to these two distinct members
    // during its final optimization pass.
    fn GetLineInfoListAndBreakPoints(&mut self) -> (&mut dyn LineInfoList, &mut LineBreakPoints);

    // cpp: layoutng_inline/score_line_break_context.h:34-35
    fn GetLineBreakPoints(&mut self) -> &mut LineBreakPoints {
        &mut self.StateMut().line_break_points_
    }
    fn LineBreakPointsIndex(&self) -> WtfSizeT {
        self.State().line_break_points_index_
    }

    // cpp: layoutng_inline/score_line_break_context.h:40-44
    fn IsActive(&self) -> bool {
        self.State().line_break_points_.is_empty() && !self.State().is_suspended_
    }
    fn SuspendUntilEndParagraph(&mut self) {
        self.StateMut().is_suspended_ = true;
    }

    // cpp: layoutng_inline/score_line_break_context.h:36-38,72-79
    fn CurrentLineBreakPoint(&self) -> *const LineBreakPoint {
        let state = self.State();
        if state.line_break_points_.is_empty() {
            return std::ptr::null();
        }
        debug_assert!((state.line_break_points_index_ as usize) < state.line_break_points_.len());
        &state.line_break_points_[state.line_break_points_index_ as usize]
    }

    // cpp: layoutng_inline/score_line_break_context.h:46-47,81-95
    fn DidCreateLine(&mut self, is_end_paragraph: bool) {
        let state = self.StateMut();
        if state.is_suspended_ && is_end_paragraph {
            state.is_suspended_ = false;
        }
        if !state.line_break_points_.is_empty() {
            debug_assert!(
                (state.line_break_points_index_ as usize) < state.line_break_points_.len()
            );
            state.line_break_points_index_ += 1;
            if (state.line_break_points_index_ as usize) >= state.line_break_points_.len() {
                state.line_break_points_.clear();
                state.line_break_points_index_ = 0;
            }
        }
    }
}

// cpp: layoutng_inline/score_line_break_context.h:60-70
pub struct ScoreLineBreakContextOf<const MAX_LINES: usize> {
    state_: ScoreLineBreakContextState,
    line_info_list_instance_: LineInfoListOf<MAX_LINES>,
}

impl<const MAX_LINES: usize> ScoreLineBreakContextOf<MAX_LINES> {
    // cpp: layoutng_inline/score_line_break_context.h:50-51,66-69
    pub fn new() -> Self {
        Self {
            state_: ScoreLineBreakContextState::default(),
            line_info_list_instance_: LineInfoListOf::new(),
        }
    }
}

impl<const MAX_LINES: usize> Default for ScoreLineBreakContextOf<MAX_LINES> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX_LINES: usize> ScoreLineBreakContext for ScoreLineBreakContextOf<MAX_LINES> {
    fn State(&self) -> &ScoreLineBreakContextState {
        &self.state_
    }
    fn StateMut(&mut self) -> &mut ScoreLineBreakContextState {
        &mut self.state_
    }
    fn GetLineInfoList(&mut self) -> &mut dyn LineInfoList {
        &mut self.line_info_list_instance_
    }
    fn GetLineInfoListAndBreakPoints(&mut self) -> (&mut dyn LineInfoList, &mut LineBreakPoints) {
        (
            &mut self.line_info_list_instance_,
            &mut self.state_.line_break_points_,
        )
    }
}
