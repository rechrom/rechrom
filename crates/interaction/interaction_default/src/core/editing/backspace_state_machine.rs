// Copyright 2016 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source backspace_state_machine.h:21-93/.cc:18-328;
// state_machine_util.h:25-32; editing_utilities.cc:716-728,761-774.
pub use super::state_machines::text_segmentation_machine_state::TextSegmentationMachineState;
use foundation::{
    emoji_property_data::{kEmojiModifierBaseRanges, kEmojiRanges, HasEmojiProperty},
    grapheme_break_property_data::IsExtendedPictographic,
    unicode::{CombiningClass, IsVariationSelector},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Start,
    BeforeLF,
    BeforeKeycap,
    BeforeVSAndKeycap,
    BeforeEmojiModifier,
    BeforeVSAndEmojiModifier,
    BeforeVS,
    BeforeZWJEmoji,
    BeforeZWJ,
    BeforeVSAndZWJ,
    OddNumberedRIS,
    EvenNumberedRIS,
    BeforeTagTerm,
    InTagSequence,
    Finished,
}
pub struct BackspaceStateMachine {
    state: State,
    trail_surrogate: u16,
    code_units_to_be_deleted: i32,
    last_seen_vs_code_units: i32,
    processing_tag_sequence_base: bool,
}
impl Default for BackspaceStateMachine {
    fn default() -> Self {
        Self {
            state: State::Start,
            trail_surrogate: 0,
            code_units_to_be_deleted: 0,
            last_seen_vs_code_units: 0,
            processing_tag_sequence_base: false,
        }
    }
}
fn length(ch: u32) -> i32 {
    if ch >= 0x10000 {
        2
    } else {
        1
    }
}
fn regional(ch: u32) -> bool {
    (0x1f1e6..=0x1f1ff).contains(&ch)
}
fn modifier(ch: u32) -> bool {
    (0x1f3fb..=0x1f3ff).contains(&ch)
}
fn modifier_base(ch: u32) -> bool {
    HasEmojiProperty(ch, &kEmojiModifierBaseRanges)
}
fn keycap_base(ch: u32) -> bool {
    (0x30..=0x39).contains(&ch) || ch == 0x23 || ch == 0x2a
}
fn tag(ch: u32) -> bool {
    (0xe0030..=0xe0039).contains(&ch) || (0xe0061..=0xe007a).contains(&ch)
}
// Source retains Emoji as well as Extended_Pictographic until the state machine
// supports Emoji_Modifier as Extended. Both borrow the original shared tables.
fn pictographic_gb11(ch: u32) -> bool {
    IsExtendedPictographic(ch) || HasEmojiProperty(ch, &kEmojiRanges)
}
impl BackspaceStateMachine {
    pub fn FeedPrecedingCodeUnit(&mut self, unit: u16) -> TextSegmentationMachineState {
        use State as S;
        use TextSegmentationMachineState as M;
        assert_ne!(self.state, S::Finished);
        let ch = if (0xd800..=0xdbff).contains(&unit) {
            if self.trail_surrogate == 0 {
                self.code_units_to_be_deleted += 1;
                return M::Finished;
            }
            let ch = 0x10000 + ((u32::from(unit) - 0xd800) << 10) + u32::from(self.trail_surrogate)
                - 0xdc00;
            self.trail_surrogate = 0;
            ch
        } else if (0xdc00..=0xdfff).contains(&unit) {
            if self.trail_surrogate != 0 {
                return M::Finished;
            }
            self.trail_surrogate = unit;
            return M::NeedMoreCodeUnit;
        } else {
            if self.trail_surrogate != 0 {
                return M::Finished;
            }
            u32::from(unit)
        };
        match self.state {
            S::Start => {
                self.code_units_to_be_deleted = length(ch);
                let next = if ch == 0x0a {
                    S::BeforeLF
                } else if IsVariationSelector(ch) {
                    S::BeforeVS
                } else if regional(ch) {
                    S::OddNumberedRIS
                } else if modifier(ch) {
                    S::BeforeEmojiModifier
                } else if pictographic_gb11(ch) {
                    S::BeforeZWJEmoji
                } else if ch == 0x20e3 {
                    S::BeforeKeycap
                } else if ch == 0xe007f {
                    S::BeforeTagTerm
                } else {
                    return self.Finish();
                };
                self.MoveToNextState(next)
            }
            S::BeforeLF => {
                if ch == 0x0d {
                    self.code_units_to_be_deleted += 1;
                }
                self.Finish()
            }
            S::BeforeKeycap => {
                if IsVariationSelector(ch) {
                    assert_eq!(self.last_seen_vs_code_units, 0);
                    self.last_seen_vs_code_units = length(ch);
                    return self.MoveToNextState(S::BeforeVSAndKeycap);
                }
                if keycap_base(ch) {
                    self.code_units_to_be_deleted += length(ch);
                }
                self.Finish()
            }
            S::BeforeVSAndKeycap => {
                if keycap_base(ch) {
                    self.AssertVS();
                    self.code_units_to_be_deleted += self.last_seen_vs_code_units + length(ch);
                }
                self.Finish()
            }
            S::BeforeEmojiModifier => {
                if IsVariationSelector(ch) {
                    assert_eq!(self.last_seen_vs_code_units, 0);
                    self.last_seen_vs_code_units = length(ch);
                    return self.MoveToNextState(S::BeforeVSAndEmojiModifier);
                }
                if modifier_base(ch) {
                    self.code_units_to_be_deleted += length(ch);
                    if self.processing_tag_sequence_base {
                        self.processing_tag_sequence_base = false;
                        return self.Finish();
                    }
                    return self.MoveToNextState(S::BeforeZWJEmoji);
                }
                self.Finish()
            }
            S::BeforeVSAndEmojiModifier => {
                if modifier_base(ch) {
                    self.AssertVS();
                    self.code_units_to_be_deleted += self.last_seen_vs_code_units + length(ch);
                }
                self.Finish()
            }
            S::BeforeVS => {
                if pictographic_gb11(ch) {
                    self.code_units_to_be_deleted += length(ch);
                    if self.processing_tag_sequence_base {
                        self.processing_tag_sequence_base = false;
                        return self.Finish();
                    }
                    return self.MoveToNextState(S::BeforeZWJEmoji);
                }
                if !IsVariationSelector(ch) && CombiningClass(ch) == 0 {
                    self.code_units_to_be_deleted += length(ch);
                }
                self.Finish()
            }
            S::BeforeZWJEmoji => {
                if ch == 0x200d {
                    self.MoveToNextState(S::BeforeZWJ)
                } else {
                    self.Finish()
                }
            }
            S::BeforeZWJ => {
                if pictographic_gb11(ch) {
                    self.code_units_to_be_deleted += length(ch) + 1;
                    return self.MoveToNextState(if modifier(ch) {
                        S::BeforeEmojiModifier
                    } else {
                        S::BeforeZWJEmoji
                    });
                }
                if IsVariationSelector(ch) {
                    assert_eq!(self.last_seen_vs_code_units, 0);
                    self.last_seen_vs_code_units = length(ch);
                    return self.MoveToNextState(S::BeforeVSAndZWJ);
                }
                self.Finish()
            }
            S::BeforeVSAndZWJ => {
                if !pictographic_gb11(ch) {
                    return self.Finish();
                }
                self.AssertVS();
                self.code_units_to_be_deleted += length(ch) + 1 + self.last_seen_vs_code_units;
                self.last_seen_vs_code_units = 0;
                self.MoveToNextState(S::BeforeZWJEmoji)
            }
            S::OddNumberedRIS => {
                if !regional(ch) {
                    return self.Finish();
                }
                self.code_units_to_be_deleted += 2;
                self.MoveToNextState(S::EvenNumberedRIS)
            }
            S::EvenNumberedRIS => {
                if !regional(ch) {
                    return self.Finish();
                }
                self.code_units_to_be_deleted -= 2;
                self.MoveToNextState(S::OddNumberedRIS)
            }
            S::BeforeTagTerm => {
                if tag(ch) {
                    self.code_units_to_be_deleted += length(ch);
                    return self.MoveToNextState(S::InTagSequence);
                }
                self.Finish()
            }
            S::InTagSequence => {
                if tag(ch) {
                    self.code_units_to_be_deleted += length(ch);
                    return self.StayInSameState();
                }
                if modifier(ch) {
                    self.code_units_to_be_deleted += length(ch);
                    self.processing_tag_sequence_base = true;
                    return self.MoveToNextState(S::BeforeEmojiModifier);
                }
                if IsVariationSelector(ch) {
                    self.code_units_to_be_deleted += length(ch);
                    self.processing_tag_sequence_base = true;
                    return self.MoveToNextState(S::BeforeVS);
                }
                if pictographic_gb11(ch) {
                    self.code_units_to_be_deleted += length(ch);
                    return self.Finish();
                }
                self.Finish()
            }
            S::Finished => unreachable!("cannot feed a finished BackspaceStateMachine"),
        }
    }
    fn AssertVS(&self) {
        assert!((1..=2).contains(&self.last_seen_vs_code_units));
    }
    pub fn TellEndOfPrecedingText(&mut self) -> TextSegmentationMachineState {
        if self.trail_surrogate != 0 {
            self.code_units_to_be_deleted += 1;
            self.trail_surrogate = 0;
        }
        TextSegmentationMachineState::Finished
    }
    pub fn FeedFollowingCodeUnit(&mut self, _unit: u16) -> TextSegmentationMachineState {
        unreachable!("BackspaceStateMachine never requests following text")
    }
    pub fn FinalizeAndGetBoundaryOffset(&mut self) -> i32 {
        if self.trail_surrogate != 0 {
            self.code_units_to_be_deleted += 1;
            self.trail_surrogate = 0;
        }
        if self.state != State::Finished {
            self.last_seen_vs_code_units = 0;
            self.state = State::Finished;
        }
        -self.code_units_to_be_deleted
    }
    pub fn Reset(&mut self) {
        *self = Self::default();
    }
    fn MoveToNextState(&mut self, next: State) -> TextSegmentationMachineState {
        assert_ne!(next, State::Finished);
        assert_ne!(next, State::Start);
        assert_ne!(next, self.state);
        self.state = next;
        TextSegmentationMachineState::NeedMoreCodeUnit
    }
    fn StayInSameState(&self) -> TextSegmentationMachineState {
        assert_eq!(self.state, State::InTagSequence);
        TextSegmentationMachineState::NeedMoreCodeUnit
    }
    fn Finish(&mut self) -> TextSegmentationMachineState {
        assert_ne!(self.state, State::Finished);
        self.state = State::Finished;
        TextSegmentationMachineState::Finished
    }
}
pub fn PreviousBackwardDeletionOffset(text: &[u16], current: usize) -> usize {
    if current <= 1 {
        return 0;
    }
    assert!(current <= text.len());
    let mut machine = BackspaceStateMachine::default();
    let mut state = TextSegmentationMachineState::NeedMoreCodeUnit;
    for unit in text[..current].iter().rev() {
        state = machine.FeedPrecedingCodeUnit(*unit);
        if state != TextSegmentationMachineState::NeedMoreCodeUnit {
            break;
        }
    }
    if state == TextSegmentationMachineState::NeedMoreCodeUnit {
        machine.TellEndOfPrecedingText();
    }
    current
        .checked_add_signed(machine.FinalizeAndGetBoundaryOffset() as isize)
        .expect("source boundary is within preceding text")
}
