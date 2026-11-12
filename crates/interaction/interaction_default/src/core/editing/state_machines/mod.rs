pub mod backward_grapheme_boundary_state_machine;
pub mod forward_grapheme_boundary_state_machine;
mod state_machine_util;
pub mod text_segmentation_machine_state;
use backward_grapheme_boundary_state_machine::BackwardGraphemeBoundaryStateMachine;
use forward_grapheme_boundary_state_machine::ForwardGraphemeBoundaryStateMachine;
use text_segmentation_machine_state::TextSegmentationMachineState as M;
pub trait TextSegmentationMachine: Default {
    fn FeedPrecedingCodeUnit(&mut self, unit: u16) -> M;
    fn TellEndOfPrecedingText(&mut self) -> M;
    fn FeedFollowingCodeUnit(&mut self, unit: u16) -> M;
    fn FinalizeAndGetBoundaryOffset(&mut self) -> i32;
}
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/editing_utilities.cc:715-737,746-786.
fn FindNextBoundaryOffset<S: TextSegmentationMachine>(text: &[u16], current: usize) -> usize {
    let mut machine = S::default();
    let mut state = M::Invalid;
    for &unit in text[..current].iter().rev() {
        state = machine.FeedPrecedingCodeUnit(unit);
        if state != M::NeedMoreCodeUnit {
            break;
        }
    }
    if current == 0 || state == M::NeedMoreCodeUnit {
        state = machine.TellEndOfPrecedingText()
    }
    if state == M::Finished {
        return current
            .checked_add_signed(machine.FinalizeAndGetBoundaryOffset() as isize)
            .expect("source boundary within text");
    }
    debug_assert_eq!(state, M::NeedFollowingCodeUnit);
    for &unit in &text[current..] {
        state = machine.FeedFollowingCodeUnit(unit);
        if state != M::NeedMoreCodeUnit {
            break;
        }
    }
    current
        .checked_add_signed(machine.FinalizeAndGetBoundaryOffset() as isize)
        .expect("source boundary within text")
}
pub fn PreviousGraphemeBoundary(text: &[u16], current: usize) -> usize {
    if current <= 1 || current > text.len() {
        return current
            .checked_sub(1)
            .expect("previous boundary requires nonzero offset");
    }
    FindNextBoundaryOffset::<BackwardGraphemeBoundaryStateMachine>(text, current)
}
pub fn NextGraphemeBoundary(text: &[u16], current: usize) -> usize {
    debug_assert!(current <= text.len());
    if current >= text.len().wrapping_sub(1) {
        return current + 1;
    }
    FindNextBoundaryOffset::<ForwardGraphemeBoundaryStateMachine>(text, current)
}
