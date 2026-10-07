// C++: src/foundation/blink_geometry/geometry/length_functions.h/.cc

use crate::evaluation_input::EvaluationInput;
use crate::gfx::{PointF, SizeF};
use crate::{LayoutUnit, Length, LengthPoint, LengthSize, LengthType};

// C++: length_functions.cc:37-65
// cpp: foundation/blink_geometry/geometry/length_functions.cc:34-58
pub fn FloatValueForLength(length: &Length, maximum_value: f32) -> f32 {
    FloatValueForLengthWithInput(length, maximum_value, &EvaluationInput::default())
}

pub fn FloatValueForLengthWithInput(
    length: &Length,
    maximum_value: f32,
    input: &EvaluationInput<'_>,
) -> f32 {
    match length.GetType() {
        LengthType::kFixed => length.Pixels(),
        LengthType::kPercent => {
            let result = maximum_value * length.PercentValue() / 100.0;
            result.clamp(f32::MIN, f32::MAX)
        }
        LengthType::kStretch | LengthType::kAuto => maximum_value,
        LengthType::kCalculated => length.NonNanCalculatedValue(maximum_value, input),
        _ => panic!(
            "unsupported Length type for FloatValueForLength: {:?}",
            length.GetType()
        ),
    }
}

// C++: length_functions.h:53-63, length_functions.cc:67-91
// cpp: foundation/blink_geometry/geometry/length_functions.h:53-63
// cpp: foundation/blink_geometry/geometry/length_functions.cc:60-86
pub fn MinimumValueForLength(length: &Length, maximum_value: LayoutUnit) -> LayoutUnit {
    MinimumValueForLengthWithInput(length, maximum_value, &EvaluationInput::default())
}

pub fn MinimumValueForLengthWithInput(
    length: &Length,
    maximum_value: LayoutUnit,
    input: &EvaluationInput<'_>,
) -> LayoutUnit {
    if length.IsFixed() {
        return LayoutUnit::from_f32(length.Pixels());
    }
    match length.GetType() {
        LengthType::kPercent => LayoutUnit::from_f32(maximum_value * length.PercentValue() / 100.0),
        LengthType::kCalculated => {
            LayoutUnit::from_f32(length.NonNanCalculatedValue(maximum_value.ToFloat(), input))
        }
        LengthType::kStretch | LengthType::kAuto => LayoutUnit::new(),
        _ => panic!(
            "unsupported Length type for MinimumValueForLength: {:?}",
            length.GetType()
        ),
    }
}

// C++: length_functions.cc:93-115
// cpp: foundation/blink_geometry/geometry/length_functions.cc:88-110
pub fn ValueForLength(length: &Length, maximum_value: LayoutUnit) -> LayoutUnit {
    ValueForLengthWithInput(length, maximum_value, &EvaluationInput::default())
}

pub fn ValueForLengthWithInput(
    length: &Length,
    maximum_value: LayoutUnit,
    input: &EvaluationInput<'_>,
) -> LayoutUnit {
    match length.GetType() {
        LengthType::kFixed | LengthType::kPercent | LengthType::kCalculated => {
            MinimumValueForLengthWithInput(length, maximum_value, input)
        }
        LengthType::kStretch | LengthType::kAuto => maximum_value,
        _ => panic!(
            "unsupported Length type for ValueForLength: {:?}",
            length.GetType()
        ),
    }
}

// C++: length_functions.cc:117-130
// cpp: foundation/blink_geometry/geometry/length_functions.cc:112-117
pub fn SizeForLengthSize(length_size: &LengthSize, box_size: &SizeF) -> SizeF {
    SizeF::new(
        FloatValueForLength(length_size.Width(), box_size.width()),
        FloatValueForLength(length_size.Height(), box_size.height()),
    )
}

// C++: length_functions.cc:132-139
// cpp: foundation/blink_geometry/geometry/length_functions.cc:119-123
pub fn PointForLengthPoint(length_point: &LengthPoint, box_size: &SizeF) -> PointF {
    PointF::new(
        FloatValueForLength(length_point.X(), box_size.width()),
        FloatValueForLength(length_point.Y(), box_size.height()),
    )
}

// Rust assembly boundary declared by layoutng/internal/length_utils.rs.
// It carries the C++ EvaluationInput::intrinsic_evaluator callback without
// creating a Cargo dependency from foundation back to layoutng.
#[no_mangle]
pub extern "Rust" fn FoundationMinimumValueForLengthWithIntrinsicEvaluator(
    length: &Length,
    percentage_resolution_size: LayoutUnit,
    evaluator: &mut dyn FnMut(&Length) -> LayoutUnit,
    calc_size_keyword_behavior: crate::CalcSizeKeywordBehavior,
) -> LayoutUnit {
    let evaluator = std::cell::RefCell::new(evaluator);
    let callback = |length: &Length| (evaluator.borrow_mut())(length);
    let input = EvaluationInput {
        intrinsic_evaluator: Some(&callback),
        calc_size_keyword_behavior,
        ..EvaluationInput::default()
    };
    MinimumValueForLengthWithInput(length, percentage_resolution_size, &input)
}
