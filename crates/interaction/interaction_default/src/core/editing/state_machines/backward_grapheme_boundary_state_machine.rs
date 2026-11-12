// Copyright 2016 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/state_machines/backward_grapheme_boundary_state_machine.h:17-72;
// backward_grapheme_boundary_state_machine.cc:18-42,56-237.
use super::{
    state_machine_util::*, text_segmentation_machine_state::TextSegmentationMachineState as M,
    TextSegmentationMachine,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Start,
    StartWaitLeadSurrogate,
    Search,
    SearchWaitLeadSurrogate,
    CountRIS,
    CountRISWaitLeadSurrogate,
    Finished,
}
pub struct BackwardGraphemeBoundaryStateMachine {
    trail_surrogate: u16,
    next_code_point: u32,
    boundary_offset: i32,
    preceding_ris_count: u32,
    state: State,
}
impl Default for BackwardGraphemeBoundaryStateMachine {
    fn default() -> Self {
        Self {
            trail_surrogate: 0,
            next_code_point: 0x110000,
            boundary_offset: 0,
            preceding_ris_count: 0,
            state: State::Start,
        }
    }
}
impl TextSegmentationMachine for BackwardGraphemeBoundaryStateMachine {
    fn FeedPrecedingCodeUnit(&mut self, unit: u16) -> M {
        use State as S;
        match self.state {
            S::Start => {
                debug_assert_eq!(self.trail_surrogate, 0);
                debug_assert_eq!(self.next_code_point, 0x110000);
                debug_assert_eq!(self.boundary_offset, 0);
                debug_assert_eq!(self.preceding_ris_count, 0);
                if IsTrail(unit) {
                    self.trail_surrogate = unit;
                    return self.MoveToNextState(S::StartWaitLeadSurrogate);
                }
                if IsLead(unit) {
                    self.boundary_offset = -1;
                    return self.Finish();
                }
                self.next_code_point = u32::from(unit);
                self.boundary_offset -= 1;
                self.MoveToNextState(S::Search)
            }
            S::StartWaitLeadSurrogate => {
                debug_assert_ne!(self.trail_surrogate, 0);
                debug_assert_eq!(self.next_code_point, 0x110000);
                debug_assert_eq!(self.boundary_offset, 0);
                debug_assert_eq!(self.preceding_ris_count, 0);
                if !IsLead(unit) {
                    self.boundary_offset = -1;
                    return self.Finish();
                }
                self.next_code_point = Supplementary(unit, self.trail_surrogate);
                self.boundary_offset = -2;
                self.trail_surrogate = 0;
                self.MoveToNextState(S::Search)
            }
            S::Search => {
                debug_assert_eq!(self.trail_surrogate, 0);
                debug_assert_ne!(self.next_code_point, 0x110000);
                debug_assert!(self.boundary_offset < 0);
                debug_assert_eq!(self.preceding_ris_count, 0);
                if IsTrail(unit) {
                    self.trail_surrogate = unit;
                    return self.MoveToNextState(S::SearchWaitLeadSurrogate);
                }
                if IsLead(unit) || IsGraphemeBreak(u32::from(unit), self.next_code_point) {
                    return self.Finish();
                }
                self.next_code_point = u32::from(unit);
                self.boundary_offset -= 1;
                self.StaySameState()
            }
            S::SearchWaitLeadSurrogate => {
                debug_assert_ne!(self.trail_surrogate, 0);
                debug_assert_ne!(self.next_code_point, 0x110000);
                debug_assert!(self.boundary_offset < 0);
                debug_assert_eq!(self.preceding_ris_count, 0);
                if !IsLead(unit) {
                    return self.Finish();
                }
                let point = Supplementary(unit, self.trail_surrogate);
                self.trail_surrogate = 0;
                if IsRegionalIndicator(self.next_code_point) && IsRegionalIndicator(point) {
                    self.preceding_ris_count = 1;
                    return self.MoveToNextState(S::CountRIS);
                }
                if IsGraphemeBreak(point, self.next_code_point) {
                    return self.Finish();
                }
                self.next_code_point = point;
                self.boundary_offset -= 2;
                self.MoveToNextState(S::Search)
            }
            S::CountRIS => {
                debug_assert_eq!(self.trail_surrogate, 0);
                debug_assert!(IsRegionalIndicator(self.next_code_point));
                debug_assert!(self.boundary_offset < 0);
                debug_assert!(self.preceding_ris_count > 0);
                if IsTrail(unit) {
                    self.trail_surrogate = unit;
                    return self.MoveToNextState(S::CountRISWaitLeadSurrogate);
                }
                if self.preceding_ris_count % 2 != 0 {
                    self.boundary_offset -= 2
                }
                self.Finish()
            }
            S::CountRISWaitLeadSurrogate => {
                debug_assert_ne!(self.trail_surrogate, 0);
                debug_assert!(IsRegionalIndicator(self.next_code_point));
                debug_assert!(self.boundary_offset < 0);
                debug_assert!(self.preceding_ris_count > 0);
                if IsLead(unit) {
                    let point = Supplementary(unit, self.trail_surrogate);
                    self.trail_surrogate = 0;
                    if IsRegionalIndicator(point) {
                        self.preceding_ris_count += 1;
                        return self.MoveToNextState(S::CountRIS);
                    }
                }
                if self.preceding_ris_count % 2 != 0 {
                    self.boundary_offset -= 2
                }
                self.Finish()
            }
            S::Finished => unreachable!("finished preceding grapheme machine"),
        }
    }
    fn TellEndOfPrecedingText(&mut self) -> M {
        use State as S;
        match self.state {
            S::Start => {
                debug_assert_eq!(self.boundary_offset, 0);
                self.Finish()
            }
            S::StartWaitLeadSurrogate => {
                debug_assert_eq!(self.boundary_offset, 0);
                self.boundary_offset = -1;
                self.Finish()
            }
            S::Search | S::SearchWaitLeadSurrogate => self.Finish(),
            S::CountRIS | S::CountRISWaitLeadSurrogate => {
                debug_assert!(self.preceding_ris_count > 0);
                if self.preceding_ris_count % 2 != 0 {
                    self.boundary_offset -= 2
                }
                self.Finish()
            }
            S::Finished => unreachable!("finished preceding grapheme machine"),
        }
    }
    fn FeedFollowingCodeUnit(&mut self, _: u16) -> M {
        unreachable!("backward grapheme machine does not request following input")
    }
    fn FinalizeAndGetBoundaryOffset(&mut self) -> i32 {
        if self.state != State::Finished {
            self.TellEndOfPrecedingText();
        }
        debug_assert!(self.boundary_offset <= 0);
        self.boundary_offset
    }
}
impl BackwardGraphemeBoundaryStateMachine {
    fn MoveToNextState(&mut self, next: State) -> M {
        debug_assert_ne!(next, State::Finished);
        debug_assert_ne!(next, State::Start);
        debug_assert_ne!(next, self.state);
        self.state = next;
        M::NeedMoreCodeUnit
    }
    fn StaySameState(&self) -> M {
        debug_assert_eq!(self.state, State::Search);
        M::NeedMoreCodeUnit
    }
    fn Finish(&mut self) -> M {
        debug_assert_ne!(self.state, State::Finished);
        self.state = State::Finished;
        M::Finished
    }
    pub fn Reset(&mut self) {
        *self = Self::default()
    }
}
