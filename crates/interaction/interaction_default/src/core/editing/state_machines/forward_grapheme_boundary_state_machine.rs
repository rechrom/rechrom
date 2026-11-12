// Copyright 2016 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/state_machines/forward_grapheme_boundary_state_machine.h:17-75;
// forward_grapheme_boundary_state_machine.cc:17-40,54-248.
use super::{
    state_machine_util::*, text_segmentation_machine_state::TextSegmentationMachineState as M,
    TextSegmentationMachine,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    CountRIS,
    CountRISWaitLeadSurrogate,
    StartForward,
    StartForwardWaitTrailSurrogate,
    Search,
    SearchWaitTrailSurrogate,
    Finished,
}
pub struct ForwardGraphemeBoundaryStateMachine {
    pending_code_unit: u16,
    prev_code_point: u32,
    boundary_offset: i32,
    preceding_ris_count: u32,
    state: State,
}
impl Default for ForwardGraphemeBoundaryStateMachine {
    fn default() -> Self {
        Self {
            pending_code_unit: 0,
            prev_code_point: 0x110000,
            boundary_offset: 0,
            preceding_ris_count: 0,
            state: State::CountRIS,
        }
    }
}
impl TextSegmentationMachine for ForwardGraphemeBoundaryStateMachine {
    fn FeedPrecedingCodeUnit(&mut self, unit: u16) -> M {
        debug_assert_eq!(self.prev_code_point, 0x110000);
        debug_assert_eq!(self.boundary_offset, 0);
        use State as S;
        match self.state {
            S::CountRIS => {
                debug_assert_eq!(self.pending_code_unit, 0);
                if IsTrail(unit) {
                    self.pending_code_unit = unit;
                    return self.MoveToNextState(S::CountRISWaitLeadSurrogate);
                }
                self.MoveToNextState(S::StartForward)
            }
            S::CountRISWaitLeadSurrogate => {
                debug_assert_ne!(self.pending_code_unit, 0);
                if IsLead(unit) {
                    let point = Supplementary(unit, self.pending_code_unit);
                    self.pending_code_unit = 0;
                    if IsRegionalIndicator(point) {
                        self.preceding_ris_count += 1;
                        return self.MoveToNextState(S::CountRIS);
                    }
                }
                self.pending_code_unit = 0;
                self.MoveToNextState(S::StartForward)
            }
            _ => unreachable!("preceding input after forward grapheme search began"),
        }
    }
    fn FeedFollowingCodeUnit(&mut self, unit: u16) -> M {
        use State as S;
        match self.state {
            S::CountRIS | S::CountRISWaitLeadSurrogate => {
                unreachable!("following input before preceding scan ended")
            }
            S::StartForward => {
                debug_assert_eq!(self.prev_code_point, 0x110000);
                debug_assert_eq!(self.boundary_offset, 0);
                debug_assert_eq!(self.pending_code_unit, 0);
                if IsTrail(unit) {
                    self.boundary_offset = 1;
                    return self.Finish();
                }
                if IsLead(unit) {
                    self.pending_code_unit = unit;
                    return self.MoveToNextState(S::StartForwardWaitTrailSurrogate);
                }
                self.prev_code_point = u32::from(unit);
                self.boundary_offset = 1;
                self.MoveToNextState(S::Search)
            }
            S::StartForwardWaitTrailSurrogate => {
                debug_assert_eq!(self.prev_code_point, 0x110000);
                debug_assert_eq!(self.boundary_offset, 0);
                debug_assert_ne!(self.pending_code_unit, 0);
                if IsTrail(unit) {
                    self.prev_code_point = Supplementary(self.pending_code_unit, unit);
                    self.boundary_offset = 2;
                    self.pending_code_unit = 0;
                    return self.MoveToNextState(S::Search);
                }
                self.boundary_offset = 1;
                self.Finish()
            }
            S::Search => {
                debug_assert_ne!(self.prev_code_point, 0x110000);
                debug_assert_ne!(self.boundary_offset, 0);
                debug_assert_eq!(self.pending_code_unit, 0);
                if IsLead(unit) {
                    self.pending_code_unit = unit;
                    return self.MoveToNextState(S::SearchWaitTrailSurrogate);
                }
                if IsTrail(unit) || IsGraphemeBreak(self.prev_code_point, u32::from(unit)) {
                    return self.Finish();
                }
                self.prev_code_point = u32::from(unit);
                self.boundary_offset += 1;
                self.StaySameState()
            }
            S::SearchWaitTrailSurrogate => {
                debug_assert_ne!(self.prev_code_point, 0x110000);
                debug_assert_ne!(self.boundary_offset, 0);
                debug_assert_ne!(self.pending_code_unit, 0);
                if !IsTrail(unit) {
                    return self.Finish();
                }
                let point = Supplementary(self.pending_code_unit, unit);
                self.pending_code_unit = 0;
                if IsRegionalIndicator(self.prev_code_point) && IsRegionalIndicator(point) {
                    if self.preceding_ris_count % 2 == 0 {
                        self.boundary_offset += 2
                    }
                    return self.Finish();
                }
                if IsGraphemeBreak(self.prev_code_point, point) {
                    return self.Finish();
                }
                self.prev_code_point = point;
                self.boundary_offset += 2;
                self.MoveToNextState(S::Search)
            }
            S::Finished => unreachable!("finished forward grapheme machine"),
        }
    }
    fn TellEndOfPrecedingText(&mut self) -> M {
        debug_assert!(matches!(
            self.state,
            State::CountRIS | State::CountRISWaitLeadSurrogate
        ));
        self.pending_code_unit = 0;
        self.MoveToNextState(State::StartForward)
    }
    fn FinalizeAndGetBoundaryOffset(&mut self) -> i32 {
        if self.state != State::Finished {
            self.FinishWithEndOfText()
        }
        debug_assert!(self.boundary_offset >= 0);
        self.boundary_offset
    }
}
impl ForwardGraphemeBoundaryStateMachine {
    pub fn Reset(&mut self) {
        *self = Self::default()
    }
    fn Finish(&mut self) -> M {
        debug_assert_ne!(self.state, State::Finished);
        self.state = State::Finished;
        M::Finished
    }
    fn MoveToNextState(&mut self, next: State) -> M {
        debug_assert_ne!(next, State::Finished);
        debug_assert_ne!(next, self.state);
        self.state = next;
        if next == State::StartForward {
            M::NeedFollowingCodeUnit
        } else {
            M::NeedMoreCodeUnit
        }
    }
    fn StaySameState(&self) -> M {
        debug_assert_eq!(self.state, State::Search);
        M::NeedMoreCodeUnit
    }
    fn FinishWithEndOfText(&mut self) {
        use State as S;
        match self.state {
            S::CountRIS
            | S::CountRISWaitLeadSurrogate
            | S::StartForward
            | S::Search
            | S::SearchWaitTrailSurrogate => {}
            S::StartForwardWaitTrailSurrogate => self.boundary_offset = 1,
            S::Finished => unreachable!("finished forward grapheme machine"),
        }
    }
}
