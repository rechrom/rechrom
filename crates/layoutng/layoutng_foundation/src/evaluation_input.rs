// C++: src/foundation/blink_geometry/geometry/evaluation_input.h:17-36
// cpp: foundation/blink_geometry/geometry/evaluation_input.h:17-36

use std::collections::BTreeMap;

use crate::length::Length;
use crate::{ColorChannelKeyword, LayoutUnit};

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CalcSizeKeywordBehavior {
    #[default]
    kAsSpecified,
    kAsAuto,
}

// The evaluator is borrowed for the duration of one calculation. The source
// std::function is owned by EvaluationInput; borrowing it here avoids a cycle
// with the compact, process-wide Length handle map.
#[derive(Clone, Default)]
pub struct EvaluationInput<'a> {
    pub size_keyword_basis: Option<f32>,
    pub intrinsic_evaluator: Option<&'a dyn Fn(&Length) -> LayoutUnit>,
    pub calc_size_keyword_behavior: CalcSizeKeywordBehavior,
    // ColorChannelKeyword comes from the read-only foundation crate without
    // Ord, so the map uses its stable repr(i32) discriminants as keys.
    pub color_channel_keyword_values: BTreeMap<i32, f32>,
}

impl EvaluationInput<'_> {
    pub fn SetColorChannelKeywordValue(&mut self, keyword: ColorChannelKeyword, value: f32) {
        self.color_channel_keyword_values
            .insert(keyword as i32, value);
    }

    pub fn ColorChannelKeywordValue(&self, keyword: ColorChannelKeyword) -> f32 {
        *self
            .color_channel_keyword_values
            .get(&(keyword as i32))
            .expect("missing color channel value")
    }
}
