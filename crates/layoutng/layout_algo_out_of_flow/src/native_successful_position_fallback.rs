#![allow(non_snake_case)]

use cssom::css_property_value_set::CSSPropertyValueSet;
use foundation::{Member, Visitor};
use layoutng_style::style::position_try_fallbacks::{
    kNoTryTactics, PositionTryFallbacks, TryTacticList,
};

// cpp: layoutng/internal/css/successful_position_fallback.h:17-43
pub struct SuccessfulPositionFallback {
    pub(crate) position_try_fallbacks_: Member<PositionTryFallbacks>,
    pub(crate) try_set_: Member<CSSPropertyValueSet>,
    pub(crate) try_tactics_: TryTacticList,
    pub(crate) index_: Option<usize>,
}
impl Default for SuccessfulPositionFallback {
    fn default() -> Self {
        Self {
            position_try_fallbacks_: Member::default(),
            try_set_: Member::default(),
            try_tactics_: kNoTryTactics,
            index_: None,
        }
    }
}

impl SuccessfulPositionFallback {
    // cpp: layoutng/internal/css/successful_position_fallback.h:27-27
    pub fn IsEmpty(&self) -> bool {
        self.position_try_fallbacks_.Get().is_null()
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:61-66
    pub fn Clear(&mut self) {
        self.position_try_fallbacks_ = Member::default();
        self.try_set_ = Member::default();
        self.try_tactics_ = kNoTryTactics;
        self.index_ = None;
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:68-71
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.position_try_fallbacks_);
        visitor.Trace(&self.try_set_);
    }
}

// cpp: layoutng/internal/layout_node_metadata.cc:53-59
impl PartialEq for SuccessfulPositionFallback {
    fn eq(&self, other: &Self) -> bool {
        foundation::ValuesEquivalent(
            &self.position_try_fallbacks_,
            &other.position_try_fallbacks_,
        ) && self.try_set_.Get() == other.try_set_.Get()
            && self.try_tactics_ == other.try_tactics_
            && self.index_ == other.index_
    }
}
