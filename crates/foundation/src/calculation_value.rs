// C++: src/foundation/blink_geometry/geometry/calculation_value.h/.cc
// Arc keeps the source's GC-owned immutable expression graph alive while a
// compact Length handle roots this value in the global handle map.

use std::sync::Arc;

use crate::calculation_expression_node::CalculationExpressionNode as Node;
use crate::calculation_operator::CalculationOperator as Op;
use crate::evaluation_input::EvaluationInput;
use crate::length::{LengthValueRange, PixelsAndPercent};

// cpp: foundation/blink_geometry/geometry/calculation_value.h:44-128
#[derive(Clone, Debug)]
pub struct CalculationValue {
    value_: PixelsAndPercent,
    expression_: Option<Arc<Node>>,
    is_non_negative_: bool,
}

impl CalculationValue {
    // C++: calculation_value.h:47-49
    // cpp: foundation/blink_geometry/geometry/calculation_value.h:47-49
    pub fn new(value: PixelsAndPercent, range: LengthValueRange) -> Arc<Self> {
        Arc::new(Self {
            value_: value,
            expression_: None,
            is_non_negative_: range == LengthValueRange::kNonNegative,
        })
    }

    // C++: calculation_value.cc:19-37
    // cpp: foundation/blink_geometry/geometry/calculation_value.cc:15-25
    pub fn CreateSimplified(expression: Arc<Node>, range: LengthValueRange) -> Arc<Self> {
        if let Node::PixelsAndPercent(value) = expression.as_ref() {
            return Self::new(*value, range);
        }
        Arc::new(Self {
            value_: PixelsAndPercent::default(),
            expression_: Some(expression),
            is_non_negative_: range == LengthValueRange::kNonNegative,
        })
    }

    // C++: calculation_value.cc:45-58
    // cpp: foundation/blink_geometry/geometry/calculation_value.cc:41-53
    pub fn Evaluate(&self, max_value: f32, input: &EvaluationInput<'_>) -> f32 {
        let mut value = if let Some(expression) = &self.expression_ {
            expression.Evaluate(max_value, input)
        } else {
            self.Pixels() + self.Percent() / 100.0 * max_value
        };
        if value.is_nan() {
            value = 0.0;
        }
        if self.IsNonNegative() && value < 0.0 {
            value = 0.0;
        }
        if value > f32::MAX {
            f32::MAX
        } else if value < f32::MIN {
            f32::MIN
        } else {
            value
        }
    }

    // C++: calculation_value.cc:60-65
    // cpp: foundation/blink_geometry/geometry/calculation_value.h:66-104
    pub fn IsExpression(&self) -> bool {
        self.expression_.is_some()
    }
    pub fn IsNonNegative(&self) -> bool {
        self.is_non_negative_
    }
    pub fn GetValueRange(&self) -> LengthValueRange {
        if self.is_non_negative_ {
            LengthValueRange::kNonNegative
        } else {
            LengthValueRange::kAll
        }
    }
    pub fn Pixels(&self) -> f32 {
        assert!(!self.IsExpression());
        self.value_.pixels
    }
    pub fn Percent(&self) -> f32 {
        assert!(!self.IsExpression());
        self.value_.percent
    }
    pub fn GetPixelsAndPercent(&self) -> PixelsAndPercent {
        assert!(!self.IsExpression());
        self.value_
    }
    pub fn HasExplicitPixels(&self) -> bool {
        assert!(!self.IsExpression());
        self.value_.has_explicit_pixels
    }
    pub fn HasExplicitPercent(&self) -> bool {
        assert!(!self.IsExpression());
        self.value_.has_explicit_percent
    }

    // C++: calculation_value.cc:67-73
    // cpp: foundation/blink_geometry/geometry/calculation_value.cc:62-69
    pub fn GetOrCreateExpression(&self) -> Arc<Node> {
        self.expression_
            .clone()
            .unwrap_or_else(|| Arc::new(Node::PixelsAndPercent(self.GetPixelsAndPercent())))
    }

    // C++: calculation_value.cc:75-108
    // cpp: foundation/blink_geometry/geometry/calculation_value.cc:71-108
    pub fn Blend(&self, from: &Self, progress: f64, range: LengthValueRange) -> Arc<Self> {
        if !self.IsExpression() && !from.IsExpression() {
            let a = from.GetPixelsAndPercent();
            let b = self.GetPixelsAndPercent();
            let blend = |a: f32, b: f32| (a as f64 + (b - a) as f64 * progress) as f32;
            return Self::new(
                PixelsAndPercent::new(
                    blend(a.pixels, b.pixels),
                    blend(a.percent, b.percent),
                    a.has_explicit_pixels | b.has_explicit_pixels,
                    a.has_explicit_percent | b.has_explicit_percent,
                ),
                range,
            );
        }
        let blended_from = Node::CreateSimplified(
            vec![
                from.GetOrCreateExpression(),
                Arc::new(Node::Number((1.0 - progress) as f32)),
            ],
            Op::kMultiply,
        );
        let blended_to = Node::CreateSimplified(
            vec![
                self.GetOrCreateExpression(),
                Arc::new(Node::Number(progress as f32)),
            ],
            Op::kMultiply,
        );
        Self::CreateSimplified(
            Node::CreateSimplified(vec![blended_from, blended_to], Op::kAdd),
            range,
        )
    }

    // C++: calculation_value.cc:110-133
    // cpp: foundation/blink_geometry/geometry/calculation_value.cc:110-127
    pub fn SubtractFromOneHundredPercent(&self) -> Arc<Self> {
        if !self.IsExpression() {
            return Self::new(
                PixelsAndPercent::new(
                    -self.Pixels(),
                    100.0 - self.Percent(),
                    self.HasExplicitPixels(),
                    true,
                ),
                LengthValueRange::kAll,
            );
        }
        let hundred_percent = Arc::new(Node::PixelsAndPercent(PixelsAndPercent::new(
            0.0, 100.0, false, true,
        )));
        Self::CreateSimplified(
            Node::CreateSimplified(
                vec![hundred_percent, self.GetOrCreateExpression()],
                Op::kSubtract,
            ),
            LengthValueRange::kAll,
        )
    }

    // C++: calculation_value.cc:135-149
    // cpp: foundation/blink_geometry/geometry/calculation_value.cc:129-136
    pub fn Add(&self, other: &Self) -> Arc<Self> {
        Self::CreateSimplified(
            Node::CreateSimplified(
                vec![self.GetOrCreateExpression(), other.GetOrCreateExpression()],
                Op::kAdd,
            ),
            LengthValueRange::kAll,
        )
    }

    // C++: calculation_value.cc:151-161
    // cpp: foundation/blink_geometry/geometry/calculation_value.cc:138-145
    pub fn Zoom(&self, factor: f64) -> Arc<Self> {
        if let Some(expression) = &self.expression_ {
            return Self::CreateSimplified(expression.Zoom(factor), self.GetValueRange());
        }
        Self::new(
            PixelsAndPercent::new(
                (self.Pixels() as f64 * factor) as f32,
                self.Percent(),
                self.HasExplicitPixels(),
                self.HasExplicitPercent(),
            ),
            self.GetValueRange(),
        )
    }

    // C++: calculation_value.cc:163-203
    // cpp: foundation/blink_geometry/geometry/calculation_value.cc:147-202
    pub fn HasAuto(&self) -> bool {
        self.expression_
            .as_ref()
            .is_some_and(|value| value.HasAuto())
    }
    pub fn HasContentOrIntrinsicSize(&self) -> bool {
        self.expression_
            .as_ref()
            .is_some_and(|value| value.HasContentOrIntrinsicSize())
    }
    pub fn HasAutoOrContentOrIntrinsicSize(&self) -> bool {
        self.expression_
            .as_ref()
            .is_some_and(|value| value.HasAutoOrContentOrIntrinsicSize())
    }
    pub fn HasPercent(&self) -> bool {
        if let Some(value) = &self.expression_ {
            value.HasPercent()
        } else {
            self.HasExplicitPercent()
        }
    }
    pub fn HasPercentOrStretch(&self) -> bool {
        if let Some(value) = &self.expression_ {
            value.HasPercentOrStretch()
        } else {
            self.HasExplicitPercent()
        }
    }
    pub fn HasStretch(&self) -> bool {
        self.expression_
            .as_ref()
            .is_some_and(|value| value.HasStretch())
    }
    pub fn HasMinContent(&self) -> bool {
        self.expression_
            .as_ref()
            .is_some_and(|value| value.HasContentOrIntrinsicSize() && value.HasMinContent())
    }
    pub fn HasMaxContent(&self) -> bool {
        self.expression_
            .as_ref()
            .is_some_and(|value| value.HasContentOrIntrinsicSize() && value.HasMaxContent())
    }
    pub fn HasFitContent(&self) -> bool {
        self.expression_
            .as_ref()
            .is_some_and(|value| value.HasContentOrIntrinsicSize() && value.HasFitContent())
    }
    pub fn HasOnlyFixedAndPercent(&self) -> bool {
        self.expression_
            .as_ref()
            .is_none_or(|value| !value.HasAutoOrContentOrIntrinsicSize() && !value.HasStretch())
    }
}

// C++: calculation_value.cc:60-65; the explicit presence bits are intentionally
// cpp: foundation/blink_geometry/geometry/calculation_value.cc:55-60
// omitted from equality, matching the two component comparisons in source.
impl PartialEq for CalculationValue {
    fn eq(&self, other: &Self) -> bool {
        self.value_.pixels == other.value_.pixels
            && self.value_.percent == other.value_.percent
            && self.expression_ == other.expression_
            && self.is_non_negative_ == other.is_non_negative_
    }
}
