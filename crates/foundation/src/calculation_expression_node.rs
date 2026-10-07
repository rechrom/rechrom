// C++: src/foundation/blink_geometry/geometry/calculation_expression_node.h/.cc
// The six GC node subclasses form an immutable Arc-owned expression DAG here.

use std::sync::Arc;

use crate::calculation_operator::CalculationOperator as Op;
use crate::evaluation_input::{CalcSizeKeywordBehavior, EvaluationInput};
use crate::length::{Length, LengthType, PixelsAndPercent};
use crate::math_functions::{
    ComputeCSSRandomValue, EvaluateSignFunction, EvaluateSteppedValueFunction,
    EvaluateTrigonometricFunction,
};
use crate::{kIndefiniteSize, ColorChannelKeyword};

// cpp: foundation/blink_geometry/geometry/calculation_expression_node.h:17-53
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalculationSizingKeyword {
    kSize,
    kAny,
    kAuto,
    kContent,
    kMinContent,
    kWebkitMinContent,
    kMaxContent,
    kWebkitMaxContent,
    kFitContent,
    kWebkitFitContent,
    kStretch,
    kWebkitFillAvailable,
}

// cpp: foundation/blink_geometry/geometry/calculation_expression_node.h:56-349
#[derive(Clone, Debug)]
pub enum CalculationExpressionNode {
    Number(f32),
    Identifier(Vec<u16>),
    SizingKeyword(CalculationSizingKeyword),
    ColorChannelKeyword(ColorChannelKeyword),
    PixelsAndPercent(PixelsAndPercent),
    Operation {
        children: Vec<Arc<Self>>,
        operator: Op,
    },
}

fn cpp_min(a: f32, b: f32) -> f32 {
    if b < a {
        b
    } else {
        a
    }
}

fn cpp_max(a: f32, b: f32) -> f32 {
    if a < b {
        b
    } else {
        a
    }
}

impl CalculationExpressionNode {
    // C++: calculation_expression_node.cc:178-449
    // cpp: foundation/blink_geometry/geometry/calculation_expression_node.cc:178-463
    pub fn CreateSimplified(children: Vec<Arc<Self>>, op: Op) -> Arc<Self> {
        use Op::*;
        let px = |index: usize| match children[index].as_ref() {
            Self::PixelsAndPercent(value) => Some(*value),
            _ => None,
        };
        let pure_pixels = |index: usize| px(index).filter(|value| value.percent == 0.0);
        let pixels_node = |pixels: f32| {
            Arc::new(Self::PixelsAndPercent(PixelsAndPercent::from_pixels(
                pixels,
            )))
        };

        match op {
            kAdd | kSubtract => {
                assert_eq!(children.len(), 2);
                if let (Some(mut left), Some(right)) = (px(0), px(1)) {
                    if op == kAdd {
                        left += right;
                    } else {
                        left -= right;
                    }
                    return Arc::new(Self::PixelsAndPercent(left));
                }
            }
            kMultiply => {
                assert_eq!(children.len(), 2);
                let operands = match (children[0].as_ref(), children[1].as_ref()) {
                    (Self::Number(number), Self::PixelsAndPercent(value))
                    | (Self::PixelsAndPercent(value), Self::Number(number)) => {
                        Some((*number, *value))
                    }
                    _ => None,
                };
                if let Some((number, mut value)) = operands {
                    value *= number;
                    return Arc::new(Self::PixelsAndPercent(value));
                }
            }
            kInvert => {
                assert_eq!(children.len(), 1);
                if let Self::Number(number) = children[0].as_ref() {
                    return Arc::new(Self::Number(1.0 / number));
                }
            }
            kMin | kMax => {
                assert!(!children.is_empty());
                if let Some(values) = (0..children.len())
                    .map(pure_pixels)
                    .collect::<Option<Vec<_>>>()
                {
                    let mut result = values[0].pixels;
                    for value in &values[1..] {
                        result = if op == kMin {
                            cpp_min(result, value.pixels)
                        } else {
                            cpp_max(result, value.pixels)
                        };
                    }
                    return pixels_node(result);
                }
            }
            kClamp => {
                assert_eq!(children.len(), 3);
                if let (Some(min), Some(value), Some(max)) =
                    (pure_pixels(0), pure_pixels(1), pure_pixels(2))
                {
                    return pixels_node(cpp_max(min.pixels, cpp_min(value.pixels, max.pixels)));
                }
            }
            kRoundNearest | kRoundUp | kRoundDown | kRoundToZero | kMod | kRem => {
                assert_eq!(children.len(), 2);
                if let (Some(a), Some(b)) = (pure_pixels(0), pure_pixels(1)) {
                    return pixels_node(EvaluateSteppedValueFunction(op, a.pixels, b.pixels));
                }
            }
            kLog => {
                assert!((1..=2).contains(&children.len()));
                if let Some(values) = (0..children.len())
                    .map(pure_pixels)
                    .collect::<Option<Vec<_>>>()
                {
                    return pixels_node(if values.len() == 1 {
                        values[0].pixels.ln()
                    } else {
                        values[0].pixels.log2() / values[1].pixels.log2()
                    });
                }
            }
            kHypot => {
                assert!(!children.is_empty());
                if let Some(values) = (0..children.len())
                    .map(pure_pixels)
                    .collect::<Option<Vec<_>>>()
                {
                    return pixels_node(
                        values
                            .iter()
                            .fold(0.0f32, |acc, value| acc.hypot(value.pixels)),
                    );
                }
            }
            kSin | kCos | kTan | kAsin | kAcos | kAtan | kAbs | kExp | kSqrt | kSign => {
                assert_eq!(children.len(), 1);
                if let Some(value) = pure_pixels(0) {
                    let value = value.pixels;
                    return match op {
                        kAbs => pixels_node(value.abs()),
                        kSign => Arc::new(Self::Number(EvaluateSignFunction(value))),
                        kExp => Arc::new(Self::Number(value.exp())),
                        kSqrt => Arc::new(Self::Number(value.sqrt())),
                        _ => Arc::new(Self::Number(EvaluateTrigonometricFunction(op, value, None))),
                    };
                }
            }
            kProgress | kMediaProgress | kContainerProgress => {
                assert_eq!(children.len(), 3);
                if let (Some(progress), Some(from), Some(to)) =
                    (pure_pixels(0), pure_pixels(1), pure_pixels(2))
                {
                    let result = (progress.pixels - from.pixels) / (to.pixels - from.pixels);
                    return Arc::new(Self::Number(if result.is_nan() {
                        f32::NAN
                    } else {
                        result.clamp(0.0, 1.0)
                    }));
                }
            }
            kCalcSize => {
                assert_eq!(children.len(), 2);
            }
            kPow => {
                assert_eq!(children.len(), 2);
                if let (Self::Number(a), Self::Number(b)) =
                    (children[0].as_ref(), children[1].as_ref())
                {
                    return Arc::new(Self::Number(a.powf(*b)));
                }
            }
            kAtan2 => {
                assert_eq!(children.len(), 2);
                if let (Some(a), Some(b)) = (pure_pixels(0), pure_pixels(1)) {
                    return pixels_node(EvaluateTrigonometricFunction(
                        op,
                        a.pixels,
                        Some(b.pixels),
                    ));
                }
            }
            kRandom => {
                assert!((3..=4).contains(&children.len()));
            }
        }
        // C++: calculation_expression_node.cc:443-449
        // cpp: foundation/blink_geometry/geometry/calculation_expression_node.cc:443-449
        if op == kCalcSize {
            assert!(!children[1].HasAutoOrContentOrIntrinsicSize() && !children[1].HasStretch());
        } else {
            for child in &children {
                assert!(!child.HasAutoOrContentOrIntrinsicSize() && !child.HasStretch());
            }
        }
        Arc::new(Self::Operation {
            children,
            operator: op,
        })
    }

    // C++: calculation_expression_node.cc:19-174, 468-650
    // cpp: foundation/blink_geometry/geometry/calculation_expression_node.cc:21-164
    // cpp: foundation/blink_geometry/geometry/calculation_expression_node.cc:502-663
    pub fn Evaluate(&self, mut max_value: f32, input: &EvaluationInput<'_>) -> f32 {
        use Op::*;
        match self {
            Self::Number(value) => *value,
            Self::Identifier(_) => 0.0,
            Self::SizingKeyword(keyword) => {
                use CalculationSizingKeyword::*;
                if *keyword == kSize {
                    return input
                        .size_keyword_basis
                        .expect("calc-size basis is required");
                }
                if *keyword == kAny {
                    return 0.0;
                }
                let behavior = input.calc_size_keyword_behavior;
                let intrinsic_type = match keyword {
                    kAuto => LengthType::kAuto,
                    kContent => {
                        if behavior == CalcSizeKeywordBehavior::kAsAuto {
                            LengthType::kAuto
                        } else {
                            LengthType::kContent
                        }
                    }
                    kMinContent | kWebkitMinContent => {
                        assert_eq!(behavior, CalcSizeKeywordBehavior::kAsSpecified);
                        LengthType::kMinContent
                    }
                    kMaxContent | kWebkitMaxContent => {
                        assert_eq!(behavior, CalcSizeKeywordBehavior::kAsSpecified);
                        LengthType::kMaxContent
                    }
                    kFitContent | kWebkitFitContent => {
                        if behavior == CalcSizeKeywordBehavior::kAsAuto {
                            LengthType::kAuto
                        } else {
                            LengthType::kFitContent
                        }
                    }
                    kStretch | kWebkitFillAvailable => {
                        if behavior == CalcSizeKeywordBehavior::kAsAuto {
                            LengthType::kAuto
                        } else {
                            LengthType::kStretch
                        }
                    }
                    kSize | kAny => unreachable!(),
                };
                if let Some(evaluator) = input.intrinsic_evaluator {
                    evaluator(&Length::from_type(intrinsic_type)).ToFloat()
                } else {
                    assert!(
                        max_value == 1.0
                            || max_value == -1.0
                            || max_value == 0.0
                            || max_value == 100.0
                    );
                    0.0
                }
            }
            Self::ColorChannelKeyword(channel) => {
                if input.color_channel_keyword_values.is_empty() {
                    0.0
                } else {
                    input.ColorChannelKeywordValue(*channel)
                }
            }
            Self::PixelsAndPercent(value) => value.pixels + value.percent / 100.0 * max_value,
            Self::Operation { children, operator } => {
                let e = |index: usize| children[index].Evaluate(max_value, input);
                match operator {
                    kAdd => e(0) + e(1),
                    kSubtract => e(0) - e(1),
                    kMultiply => e(0) * e(1),
                    kInvert => 1.0 / e(0),
                    kMin => {
                        let mut value = e(0);
                        for child in children {
                            value = cpp_min(value, child.Evaluate(max_value, input));
                        }
                        value
                    }
                    kMax => {
                        let mut value = e(0);
                        for child in children {
                            value = cpp_max(value, child.Evaluate(max_value, input));
                        }
                        value
                    }
                    kClamp => cpp_max(e(0), cpp_min(e(1), e(2))),
                    kRoundNearest | kRoundUp | kRoundDown | kRoundToZero | kMod | kRem => {
                        EvaluateSteppedValueFunction(*operator, e(0), e(1))
                    }
                    kLog => {
                        if children.len() == 1 {
                            e(0).ln()
                        } else {
                            e(0).log2() / e(1).log2()
                        }
                    }
                    kHypot => children.iter().fold(0.0f32, |value, child| {
                        value.hypot(child.Evaluate(max_value, input))
                    }),
                    kAbs => e(0).abs(),
                    kExp => e(0).exp(),
                    kSqrt => e(0).sqrt(),
                    kSign => EvaluateSignFunction(e(0)),
                    kCalcSize => {
                        let mut calculation_input = input.clone();
                        calculation_input.size_keyword_basis = Some(e(0));
                        if max_value == kIndefiniteSize.ToFloat() {
                            max_value = 0.0;
                        }
                        children[1].Evaluate(max_value, &calculation_input)
                    }
                    kProgress | kMediaProgress | kContainerProgress => {
                        let value = (e(0) - e(1)) / (e(2) - e(1));
                        if value.is_nan() {
                            f32::NAN
                        } else {
                            value.clamp(0.0, 1.0)
                        }
                    }
                    kPow => e(0).powf(e(1)),
                    kSin | kCos | kTan | kAsin | kAcos | kAtan | kAtan2 => {
                        let mut a = e(0);
                        if matches!(operator, kSin | kCos | kTan) && children[0].EvaluatesToNumber()
                        {
                            a = a.to_degrees();
                        }
                        let b = if *operator == kAtan2 {
                            Some(e(1))
                        } else {
                            None
                        };
                        EvaluateTrigonometricFunction(*operator, a, b)
                    }
                    kRandom => {
                        let step = if children.len() == 4 {
                            Some(e(3) as f64)
                        } else {
                            None
                        };
                        ComputeCSSRandomValue(e(0) as f64, e(1) as f64, e(2) as f64, step) as f32
                    }
                }
            }
        }
    }

    // C++: calculation_expression_node.h:61-104, .cc:452-466
    // cpp: foundation/blink_geometry/geometry/calculation_expression_node.h:61-104
    // cpp: foundation/blink_geometry/geometry/calculation_expression_node.cc:465-500
    pub fn HasAuto(&self) -> bool {
        match self {
            Self::SizingKeyword(CalculationSizingKeyword::kAuto) => true,
            Self::Operation {
                children,
                operator: Op::kCalcSize,
            } => children[0].HasAuto(),
            _ => false,
        }
    }
    pub fn HasContentOrIntrinsicSize(&self) -> bool {
        match self {
            Self::SizingKeyword(keyword) => !matches!(
                keyword,
                CalculationSizingKeyword::kSize
                    | CalculationSizingKeyword::kAny
                    | CalculationSizingKeyword::kAuto
                    | CalculationSizingKeyword::kStretch
                    | CalculationSizingKeyword::kWebkitFillAvailable
            ),
            Self::Operation {
                children,
                operator: Op::kCalcSize,
            } => children[0].HasContentOrIntrinsicSize(),
            _ => false,
        }
    }
    pub fn HasAutoOrContentOrIntrinsicSize(&self) -> bool {
        self.HasAuto() || self.HasContentOrIntrinsicSize()
    }
    pub fn HasStretch(&self) -> bool {
        match self {
            Self::SizingKeyword(
                CalculationSizingKeyword::kStretch | CalculationSizingKeyword::kWebkitFillAvailable,
            ) => true,
            Self::Operation {
                children,
                operator: Op::kCalcSize,
            } => children[0].HasStretch(),
            _ => false,
        }
    }
    pub fn HasPercent(&self) -> bool {
        match self {
            Self::PixelsAndPercent(value) => value.has_explicit_percent,
            Self::Operation {
                children,
                operator: Op::kCalcSize,
            } => children[0].HasPercent(),
            Self::Operation { children, .. } => children.iter().any(|child| child.HasPercent()),
            _ => false,
        }
    }
    pub fn HasPercentOrStretch(&self) -> bool {
        self.HasPercent() || self.HasStretch()
    }
    pub fn HasColorChannelKeyword(&self) -> bool {
        match self {
            Self::ColorChannelKeyword(_) => true,
            Self::Operation { children, operator } if *operator != Op::kCalcSize => {
                children.iter().any(|child| child.HasColorChannelKeyword())
            }
            _ => false,
        }
    }
    pub fn HasMinContent(&self) -> bool {
        match self {
            Self::SizingKeyword(
                CalculationSizingKeyword::kMinContent | CalculationSizingKeyword::kWebkitMinContent,
            ) => true,
            Self::Operation {
                children,
                operator: Op::kCalcSize,
            } => children[0].HasMinContent(),
            _ => false,
        }
    }
    pub fn HasMaxContent(&self) -> bool {
        match self {
            Self::SizingKeyword(
                CalculationSizingKeyword::kMaxContent | CalculationSizingKeyword::kWebkitMaxContent,
            ) => true,
            Self::Operation {
                children,
                operator: Op::kCalcSize,
            } => children[0].HasMaxContent(),
            _ => false,
        }
    }
    pub fn HasFitContent(&self) -> bool {
        match self {
            Self::SizingKeyword(
                CalculationSizingKeyword::kFitContent | CalculationSizingKeyword::kWebkitFitContent,
            ) => true,
            Self::Operation {
                children,
                operator: Op::kCalcSize,
            } => children[0].HasFitContent(),
            _ => false,
        }
    }

    // C++: calculation_expression_node.cc:651-719
    // cpp: foundation/blink_geometry/geometry/calculation_expression_node.cc:665-712
    pub fn EvaluatesToNumber(&self) -> bool {
        use Op::*;
        match self {
            Self::Number(_) | Self::ColorChannelKeyword(_) => true,
            Self::Identifier(_) | Self::SizingKeyword(_) | Self::PixelsAndPercent(_) => false,
            Self::Operation { children, operator } => match operator {
                kLog | kExp | kSqrt | kSign | kPow | kSin | kCos | kTan | kProgress
                | kMediaProgress | kContainerProgress => true,
                kCalcSize | kAsin | kAcos | kAtan | kAtan2 => false,
                kMultiply => children[0].EvaluatesToNumber() && children[1].EvaluatesToNumber(),
                kRandom => children[1].EvaluatesToNumber(),
                _ => children[0].EvaluatesToNumber(),
            },
        }
    }

    // C++: calculation_expression_node.cc:738-749
    // cpp: foundation/blink_geometry/geometry/calculation_expression_node.cc:729-736
    pub fn Zoom(&self, factor: f64) -> Arc<Self> {
        match self {
            Self::PixelsAndPercent(value) => {
                Arc::new(Self::PixelsAndPercent(PixelsAndPercent::new(
                    (value.pixels as f64 * factor) as f32,
                    value.percent,
                    value.has_explicit_pixels,
                    value.has_explicit_percent,
                )))
            }
            Self::Operation { children, operator } => Self::CreateSimplified(
                children.iter().map(|child| child.Zoom(factor)).collect(),
                *operator,
            ),
            _ => Arc::new(self.clone()),
        }
    }
}

// C++ virtual Equals methods: leaf identity rules, operation order and arity.
// cpp: foundation/blink_geometry/geometry/calculation_expression_node.cc:714-727
impl PartialEq for CalculationExpressionNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::Identifier(a), Self::Identifier(b)) => a == b,
            (Self::SizingKeyword(a), Self::SizingKeyword(b)) => a == b,
            (Self::ColorChannelKeyword(a), Self::ColorChannelKeyword(b)) => a == b,
            (Self::PixelsAndPercent(a), Self::PixelsAndPercent(b)) => {
                a.pixels == b.pixels && a.percent == b.percent
            }
            (
                Self::Operation {
                    children: ac,
                    operator: ao,
                },
                Self::Operation {
                    children: bc,
                    operator: bo,
                },
            ) => ao == bo && ac.len() == bc.len() && ac.iter().zip(bc).all(|(a, b)| a == b),
            _ => false,
        }
    }
}
