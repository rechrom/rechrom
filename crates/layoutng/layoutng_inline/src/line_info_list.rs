// C++: layoutng_inline/line_info_list.h.
#![allow(non_snake_case)]

use std::ops::{Index, IndexMut};

use foundation::WtfSizeT;
use layoutng_fragment_tree::inline_break_token::InlineBreakToken;

use crate::line_info::LineInfo;

// cpp: layoutng_inline/line_info_list.h:87-90
#[derive(Default)]
pub struct LineInfoListState {
    size_: WtfSizeT,
    start_index_: WtfSizeT,
}

// cpp: layoutng_inline/line_info_list.h:15-21,75-90
// The C++ base uses a zero-length trailing array and is backed by a derived
// fixed-size class. A trait keeps that base interface while each implementor
// owns its slots directly; no self-referential pointer is stored.
pub trait LineInfoList {
    fn State(&self) -> &LineInfoListState;
    fn StateMut(&mut self) -> &mut LineInfoListState;
    fn Slots(&self) -> &[LineInfo];
    fn SlotsMut(&mut self) -> &mut [LineInfo];

    // cpp: layoutng_inline/line_info_list.h:24-26
    fn Size(&self) -> WtfSizeT {
        self.State().size_
    }
    fn IsEmpty(&self) -> bool {
        self.Size() == 0
    }
    fn MaxLines(&self) -> WtfSizeT {
        self.Slots().len() as WtfSizeT
    }

    // cpp: layoutng_inline/line_info_list.h:28-41
    fn At(&self, index: WtfSizeT) -> &LineInfo {
        debug_assert!(index < self.Size());
        let physical = (self.State().start_index_ + index) % self.MaxLines();
        &self.Slots()[physical as usize]
    }
    fn AtMut(&mut self, index: WtfSizeT) -> &mut LineInfo {
        debug_assert!(index < self.Size());
        let physical = (self.State().start_index_ + index) % self.MaxLines();
        &mut self.SlotsMut()[physical as usize]
    }

    // cpp: layoutng_inline/line_info_list.h:42-49
    fn Front(&self) -> &LineInfo {
        self.At(0)
    }
    fn FrontMut(&mut self) -> &mut LineInfo {
        self.AtMut(0)
    }
    fn Back(&self) -> &LineInfo {
        self.At(self.Size() - 1)
    }
    fn BackMut(&mut self) -> &mut LineInfo {
        self.AtMut(self.Size() - 1)
    }

    // cpp: layoutng_inline/line_info_list.h:51-55
    fn Shrink(&mut self, size: WtfSizeT) {
        debug_assert!(size < self.Size());
        self.StateMut().size_ = size;
    }
    fn Clear(&mut self) {
        self.StateMut().size_ = 0;
        self.StateMut().start_index_ = 0;
    }

    // cpp: layoutng_inline/line_info_list.h:57-67
    fn Append(&mut self) -> &mut LineInfo {
        debug_assert!(self.Size() < self.MaxLines());
        self.StateMut().size_ += 1;
        self.BackMut()
    }
    fn RemoveFront(&mut self) {
        debug_assert!(self.Size() > 0);
        let next_start = (self.State().start_index_ + 1) % self.MaxLines();
        let state = self.StateMut();
        state.size_ -= 1;
        state.start_index_ = next_start;
    }

    // cpp: layoutng_inline/line_info_list.h:81-85
    fn UnusedInstance(&mut self) -> &mut LineInfo {
        debug_assert!(self.IsEmpty());
        &mut self.SlotsMut()[0]
    }

    // cpp: layoutng_inline/line_info_list.h:69-73,93-110
    fn Get(
        &mut self,
        break_token: *const InlineBreakToken,
        is_cached_out: &mut bool,
    ) -> &mut LineInfo {
        debug_assert!(!*is_cached_out);
        if self.IsEmpty() {
            return self.UnusedInstance();
        }
        let matches = if break_token.is_null() {
            self.Front().Start().IsZero()
        } else {
            self.Front().Start() == unsafe { &*break_token }.Start()
        };
        if matches {
            let physical = self.State().start_index_;
            self.RemoveFront();
            *is_cached_out = true;
            return &mut self.SlotsMut()[physical as usize];
        }
        self.Clear();
        self.UnusedInstance()
    }
}

// cpp: layoutng_inline/line_info_list.h:112-122
// The const generic is usize because Rust array lengths use usize; MaxLines
// exposes the source's wtf_size_t (u32) to callers.
pub struct LineInfoListOf<const MAX_LINES: usize> {
    state_: LineInfoListState,
    line_infos_instance_: [LineInfo; MAX_LINES],
}

impl<const MAX_LINES: usize> LineInfoListOf<MAX_LINES> {
    // cpp: layoutng_inline/line_info_list.h:76-79,118-121
    pub fn new() -> Self {
        assert!(MAX_LINES > 0 && MAX_LINES <= WtfSizeT::MAX as usize);
        Self {
            state_: LineInfoListState::default(),
            line_infos_instance_: std::array::from_fn(|_| LineInfo::default()),
        }
    }
}

impl<const MAX_LINES: usize> Default for LineInfoListOf<MAX_LINES> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX_LINES: usize> LineInfoList for LineInfoListOf<MAX_LINES> {
    fn State(&self) -> &LineInfoListState {
        &self.state_
    }
    fn StateMut(&mut self) -> &mut LineInfoListState {
        &mut self.state_
    }
    fn Slots(&self) -> &[LineInfo] {
        &self.line_infos_instance_
    }
    fn SlotsMut(&mut self) -> &mut [LineInfo] {
        &mut self.line_infos_instance_
    }
}

// cpp: layoutng_inline/line_info_list.h:28-41
impl Index<usize> for dyn LineInfoList + '_ {
    type Output = LineInfo;
    fn index(&self, index: usize) -> &Self::Output {
        self.At(index as WtfSizeT)
    }
}
impl IndexMut<usize> for dyn LineInfoList + '_ {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.AtMut(index as WtfSizeT)
    }
}

impl<const MAX_LINES: usize> Index<usize> for LineInfoListOf<MAX_LINES> {
    type Output = LineInfo;
    fn index(&self, index: usize) -> &Self::Output {
        self.At(index as WtfSizeT)
    }
}
impl<const MAX_LINES: usize> IndexMut<usize> for LineInfoListOf<MAX_LINES> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.AtMut(index as WtfSizeT)
    }
}
