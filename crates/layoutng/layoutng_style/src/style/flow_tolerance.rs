// cpp: layoutng_style/style/flow_tolerance.h:8-9
// Pending foundation style_values_api and blink_geometry_api connections.
use foundation::{CSSValueID, Length};

// cpp: layoutng_style/style/flow_tolerance.h:46-50
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
enum FlowToleranceType {
    kNormal,
    kInfinite,
    kLength,
}

// cpp: layoutng_style/style/flow_tolerance.h:13-17
/// Computed CSS flow-tolerance value.
#[derive(Clone, Debug, PartialEq)]
pub struct FlowTolerance {
    flow_tolerance_type_: FlowToleranceType,
    length_: Length,
}

#[allow(non_snake_case)]
impl FlowTolerance {
    // cpp: layoutng_style/style/flow_tolerance.h:20-22
    pub fn from_length(length: &Length) -> Self {
        Self {
            flow_tolerance_type_: FlowToleranceType::kLength,
            length_: length.clone(),
        }
    }

    // cpp: layoutng_style/style/flow_tolerance.h:23-31
    pub fn from_keyword(keyword: CSSValueID) -> Self {
        if keyword == CSSValueID::kNormal {
            Self {
                flow_tolerance_type_: FlowToleranceType::kNormal,
                length_: Length::default(),
            }
        } else if keyword == CSSValueID::kInfinite {
            Self {
                flow_tolerance_type_: FlowToleranceType::kInfinite,
                length_: Length::default(),
            }
        } else {
            unreachable!("C++ NOTREACHED: invalid flow-tolerance keyword")
        }
    }

    // cpp: layoutng_style/style/flow_tolerance.h:33-38
    pub fn IsNormal(&self) -> bool {
        self.flow_tolerance_type_ == FlowToleranceType::kNormal
    }
    pub fn IsInfinite(&self) -> bool {
        self.flow_tolerance_type_ == FlowToleranceType::kInfinite
    }

    // cpp: layoutng_style/style/flow_tolerance.h:39-42
    pub fn GetLength(&self) -> &Length {
        debug_assert!(!self.IsNormal() && !self.IsInfinite());
        &self.length_
    }
}

// cpp: layoutng_style/style/flow_tolerance.h:20
impl Default for FlowTolerance {
    fn default() -> Self {
        Self {
            flow_tolerance_type_: FlowToleranceType::kNormal,
            length_: Length::default(),
        }
    }
}

// cpp: layoutng_style/style/flow_tolerance.h:44
// Fieldwise equality is derived above.
