use foundation::{AtomicString, HashSet, HeapVector, Member, ScopedCSSName, Visitor};

use super::computed_style_constants::TryTactic;
use super::position_area::PositionArea;

impl foundation::Traceable for PositionTryFallback {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        PositionTryFallback::Trace(self, visitor);
    }
}

impl foundation::Traceable for PositionTryFallbacks {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        PositionTryFallbacks::Trace(self, visitor);
    }
}

// cpp: layoutng_style/style/position_try_fallbacks.h:17-21
pub type TryTacticList = [TryTactic; 3];

// cpp: layoutng_style/style/position_try_fallbacks.h:23-24
#[allow(non_upper_case_globals)]
pub const kNoTryTactics: TryTacticList = [TryTactic::kNone; 3];

// cpp: layoutng_style/style/position_try_fallbacks.h:26-58
#[derive(Clone)]
pub struct PositionTryFallback {
    position_try_name_: Member<ScopedCSSName>,
    tactic_list_: TryTacticList,
    position_area_: PositionArea,
}

#[allow(non_snake_case)]
impl PositionTryFallback {
    // cpp: layoutng_style/style/position_try_fallbacks.h:31-32
    pub fn from_name(name: Member<ScopedCSSName>, tactic_list: TryTacticList) -> Self {
        Self {
            position_try_name_: name,
            tactic_list_: tactic_list,
            position_area_: PositionArea::default(),
        }
    }

    // cpp: layoutng_style/style/position_try_fallbacks.h:33-34
    pub fn from_position_area(position_area: PositionArea) -> Self {
        Self {
            position_try_name_: Member::default(),
            tactic_list_: kNoTryTactics,
            position_area_: position_area,
        }
    }

    // cpp: layoutng_style/style/position_try_fallbacks.h:36-38
    pub fn GetTryTactic(&self) -> &TryTacticList {
        &self.tactic_list_
    }
    pub fn GetPositionTryName(&self) -> *const ScopedCSSName {
        self.position_try_name_.Get()
    }
    pub fn GetPositionArea(&self) -> &PositionArea {
        &self.position_area_
    }

    // cpp: layoutng_style/style/position_try_fallbacks.h:42-46
    // No definition exists in the supplied C++ tree.
    pub fn Matches(&self, other: &PositionTryFallback) -> bool {
        unsafe { PositionTryFallbackMatches(self, other) }
    }

    // cpp: layoutng_style/style/position_try_fallbacks.h:48-51
    pub fn IsNone(&self) -> bool {
        self.position_try_name_.Get().is_null()
            && self.tactic_list_[0] == TryTactic::kNone
            && self.position_area_.IsNone()
    }

    // cpp: layoutng_style/style/position_try_fallbacks.h:52
    // cpp: layoutng_style/style/position_try_fallbacks.cc:14-16
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.position_try_name_);
    }
}

// cpp: layoutng_style/style/position_try_fallbacks.h:30
impl Default for PositionTryFallback {
    fn default() -> Self {
        Self {
            position_try_name_: Member::default(),
            tactic_list_: kNoTryTactics,
            position_area_: PositionArea::default(),
        }
    }
}

// cpp: layoutng_style/style/position_try_fallbacks.h:40
// cpp: layoutng_style/style/position_try_fallbacks.cc:8-12
impl PartialEq for PositionTryFallback {
    fn eq(&self, other: &Self) -> bool {
        self.tactic_list_ == other.tactic_list_
            && foundation::ValuesEquivalent(&self.position_try_name_, &other.position_try_name_)
            && self.position_area_ == other.position_area_
    }
}

// cpp: layoutng_style/style/position_try_fallbacks.h:60-77
pub struct PositionTryFallbacks {
    fallbacks_: HeapVector<PositionTryFallback>,
}

#[allow(non_snake_case)]
impl PositionTryFallbacks {
    // cpp: layoutng_style/style/position_try_fallbacks.h:63-64
    pub fn new(fallbacks: HeapVector<PositionTryFallback>) -> Self {
        Self {
            fallbacks_: fallbacks,
        }
    }

    // cpp: layoutng_style/style/position_try_fallbacks.h:66-68
    pub fn GetFallbacks(&self) -> &HeapVector<PositionTryFallback> {
        &self.fallbacks_
    }

    // cpp: layoutng_style/style/position_try_fallbacks.h:70-72
    // No definition exists in the supplied C++ tree.
    pub fn HasPositionTryName(&self, names: &HashSet<AtomicString>) -> bool {
        unsafe { PositionTryFallbacksHasPositionTryName(self, names) }
    }

    // cpp: layoutng_style/style/position_try_fallbacks.h:73
    // cpp: layoutng_style/style/position_try_fallbacks.cc:23-25
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.fallbacks_);
    }
}

// cpp: layoutng_style/style/position_try_fallbacks.h:69
// cpp: layoutng_style/style/position_try_fallbacks.cc:18-21
impl PartialEq for PositionTryFallbacks {
    fn eq(&self, other: &Self) -> bool {
        self.fallbacks_ == other.fallbacks_
    }
}

// cpp: layoutng_style/style/position_try_fallbacks.h:79-86
pub struct PositionTryFallbackVectorTraits;

#[allow(non_upper_case_globals)]
impl PositionTryFallbackVectorTraits {
    pub const kCanClearUnusedSlotsWithMemset: bool = true;
    pub const kCanInitializeWithMemset: bool = true;
    pub const kCanMoveWithMemcpy: bool = true;
    pub const kCanTraceConcurrently: bool = true;
}

unsafe extern "Rust" {
    fn PositionTryFallbackMatches(value: &PositionTryFallback, other: &PositionTryFallback)
        -> bool;
    fn PositionTryFallbacksHasPositionTryName(
        value: &PositionTryFallbacks,
        names: &HashSet<AtomicString>,
    ) -> bool;
}
