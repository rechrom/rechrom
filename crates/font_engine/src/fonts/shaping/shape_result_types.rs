#![allow(non_camel_case_types)]

// The rest of shape_result.h and its method bodies remain blocked by shaping
// dependencies; this value enum is fully defined by the source header.
// cpp: font_engine/fonts/shaping/shape_result.h:79-84
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdjustMidCluster {
    kToEnd,
    kToStart,
}

// cpp: font_engine/fonts/shaping/shape_result.h:115-118
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct BreakGlyphsOption(pub bool);
