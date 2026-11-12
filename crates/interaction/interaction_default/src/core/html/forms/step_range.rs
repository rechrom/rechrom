/*
 * Copyright (C) 2006, 2007, 2008, 2009, 2010 Apple Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB. If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/forms/step_range.h:1-138
// Source: third_party/blink/renderer/core/html/forms/step_range.cc:1-205
// Decimal remains a value supplied by the translated WTF decimal owner. This
// port preserves decimal arithmetic, including NaN, rather than substituting
// binary floating point for Chromium's number/step algorithms.

use std::ops::{Add, Div, Mul, Sub};

// Source: platform/wtf/decimal.h:97-119,126,134-144,153-154
// Binary-float conversion requires the separately injected source leaves.
// Only the original Decimal value operations used by this translation.
pub trait Decimal:
    Copy
    + PartialEq
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
{
    fn FromInt(value: i32) -> Self;
    fn FromComponents(negative: bool, exponent: i32, coefficient: u64) -> Self;
    fn Nan() -> Self;
    fn Infinity(negative: bool) -> Self;
    fn IsFinite(self) -> bool;
    fn Abs(self) -> Self;
    fn Round(self) -> Self;
    fn Floor(self) -> Self;
}
// The concrete value operations are owned by WTF; conversion leaves are
// supplied separately by NumberTypeParser/RangeControl.
impl Decimal for crate::platform::wtf::decimal::Decimal {
    fn FromInt(value: i32) -> Self {
        Self::FromInt(value)
    }
    fn FromComponents(negative: bool, exponent: i32, coefficient: u64) -> Self {
        Self::FromComponents(negative, exponent, coefficient)
    }
    fn Nan() -> Self {
        Self::Nan()
    }
    fn Infinity(negative: bool) -> Self {
        Self::Infinity(if negative {
            crate::platform::wtf::decimal::Sign::kNegative
        } else {
            crate::platform::wtf::decimal::Sign::kPositive
        })
    }
    fn IsFinite(self) -> bool {
        self.IsFinite()
    }
    fn Abs(self) -> Self {
        self.Abs()
    }
    fn Round(self) -> Self {
        self.Round()
    }
    fn Floor(self) -> Self {
        self.Floor()
    }
}
// Source: core/html/parser/html_parser_idioms.h:47-62. The parser retains the
// HTML number grammar and serializer; StepRange never reparses through f64.
pub trait NumberTypeParser<D: Decimal> {
    fn ParseToDecimalForNumberType(&self, value: &str, default: D) -> D;
    fn ParseToDoubleForNumberType(&self, value: &str) -> f64;
    fn SerializeForNumberType(&self, value: D) -> String;
}

// Source: step_range.h:31,37-41.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnyStepHandling {
    kRejectAny,
    kAnyIsDefaultStep,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StepValueShouldBe {
    kStepValueShouldBeReal,
    kParsedStepValueShouldBeInteger,
    kScaledStepValueShouldBeInteger,
}
// Source: step_range.h:43-69.
#[derive(Clone, Copy)]
pub struct StepDescription {
    pub default_step: i32,
    pub default_step_base: i32,
    pub step_scale_factor: i32,
    pub step_value_should_be: StepValueShouldBe,
}
impl StepDescription {
    pub const fn new(
        default_step: i32,
        default_step_base: i32,
        step_scale_factor: i32,
        step_value_should_be: StepValueShouldBe,
    ) -> Self {
        Self {
            default_step,
            default_step_base,
            step_scale_factor,
            step_value_should_be,
        }
    }
    pub fn DefaultValue<D: Decimal>(&self) -> D {
        D::FromInt(self.default_step) * D::FromInt(self.step_scale_factor)
    }
}
impl Default for StepDescription {
    fn default() -> Self {
        Self::new(1, 0, 1, StepValueShouldBe::kStepValueShouldBeReal)
    }
}

// Source: step_range.h:71-80,125-133; step_range.cc:32-65.
#[derive(Clone)]
pub struct StepRange<D: Decimal> {
    maximum_: D,
    minimum_: D,
    step_: D,
    step_base_: D,
    step_description_: StepDescription,
    has_step_: bool,
    has_min_: bool,
    has_max_: bool,
    supports_reversed_range_: bool,
}
impl<D: Decimal> Default for StepRange<D> {
    fn default() -> Self {
        Self {
            maximum_: D::FromInt(100),
            minimum_: D::FromInt(0),
            step_: D::FromInt(1),
            step_base_: D::FromInt(0),
            step_description_: StepDescription::default(),
            has_step_: false,
            has_min_: false,
            has_max_: false,
            supports_reversed_range_: false,
        }
    }
}
impl<D: Decimal> StepRange<D> {
    pub fn new(
        step_base: D,
        minimum: D,
        maximum: D,
        has_min: bool,
        has_max: bool,
        supports_reversed_range: bool,
        step: D,
        step_description: StepDescription,
    ) -> Self {
        let result = Self {
            maximum_: maximum,
            minimum_: minimum,
            step_: if step.IsFinite() { step } else { D::FromInt(1) },
            step_base_: if step_base.IsFinite() {
                step_base
            } else {
                D::FromInt(1)
            },
            step_description_: step_description,
            has_step_: step.IsFinite(),
            has_min_: has_min,
            has_max_: has_max,
            supports_reversed_range_: supports_reversed_range,
        };
        debug_assert!(
            result.maximum_.IsFinite()
                && result.minimum_.IsFinite()
                && result.step_.IsFinite()
                && result.step_base_.IsFinite()
        );
        result
    }
    // Source: step_range.h:82-118.
    pub fn HasStep(&self) -> bool {
        self.has_step_
    }
    pub fn Maximum(&self) -> D {
        self.maximum_
    }
    pub fn Minimum(&self) -> D {
        self.minimum_
    }
    pub fn HasMin(&self) -> bool {
        self.has_min_
    }
    pub fn HasMax(&self) -> bool {
        self.has_max_
    }
    pub fn HasRangeLimitations(&self) -> bool {
        self.has_min_ || self.has_max_
    }
    pub fn Step(&self) -> D {
        self.step_
    }
    pub fn StepBase(&self) -> D {
        self.step_base_
    }
    pub fn DefaultValue(&self) -> D {
        self.ClampValue((self.minimum_ + self.maximum_) / D::FromInt(2))
    }
    pub fn ProportionFromValue(&self, value: D) -> D {
        if self.minimum_ == self.maximum_ {
            return D::FromInt(0);
        }
        (value - self.minimum_) / (self.maximum_ - self.minimum_)
    }
    pub fn ValueFromProportion(&self, proportion: D) -> D {
        self.minimum_ + proportion * (self.maximum_ - self.minimum_)
    }
    // Source: step_range.cc:67-75.
    fn AcceptableError(&self) -> D {
        if self.step_description_.step_value_should_be == StepValueShouldBe::kStepValueShouldBeReal
        {
            self.step_ / D::FromComponents(false, 0, 1 << 24)
        } else {
            D::FromInt(0)
        }
    }
    // Source: step_range.cc:77-86.
    pub fn AlignValueForStep(&self, current: D, new_value: D) -> D {
        if new_value >= D::FromComponents(false, 21, 1) {
            return new_value;
        }
        if self.StepMismatch(current) {
            new_value
        } else {
            self.RoundByStep(new_value, self.step_base_)
        }
    }
    // Source: step_range.cc:88-102. Comparisons preserve std::min/max's operand
    // ordering even for NaN decimal values.
    pub fn ClampValue(&self, value: D) -> D {
        let smaller = if self.maximum_ < value {
            self.maximum_
        } else {
            value
        };
        let in_range = if self.minimum_ < smaller {
            smaller
        } else {
            self.minimum_
        };
        if !self.has_step_ {
            return in_range;
        }
        let rounded = self.RoundByStep(in_range, self.step_base_);
        let clamped = if rounded > self.maximum_ {
            rounded - self.step_
        } else if rounded < self.minimum_ {
            rounded + self.step_
        } else {
            rounded
        };
        if clamped < self.minimum_ || clamped > self.maximum_ {
            in_range
        } else {
            clamped
        }
    }
    // Source: step_range.cc:104-146.
    pub fn ParseStep(
        handling: AnyStepHandling,
        description: StepDescription,
        value: &str,
        parser: &dyn NumberTypeParser<D>,
    ) -> D {
        if value.is_empty() {
            return description.DefaultValue();
        }
        if value.eq_ignore_ascii_case("any") {
            return match handling {
                AnyStepHandling::kRejectAny => D::Nan(),
                AnyStepHandling::kAnyIsDefaultStep => description.DefaultValue(),
            };
        }
        let mut step = parser.ParseToDecimalForNumberType(value, D::Nan());
        if !step.IsFinite() || step <= D::FromInt(0) {
            return description.DefaultValue();
        }
        match description.step_value_should_be {
            StepValueShouldBe::kStepValueShouldBeReal => {
                step = step * D::FromInt(description.step_scale_factor)
            }
            StepValueShouldBe::kParsedStepValueShouldBeInteger => {
                let rounded = step.Round();
                step = if rounded < D::FromInt(1) {
                    D::FromInt(1)
                } else {
                    rounded
                };
                step = step * D::FromInt(description.step_scale_factor);
            }
            StepValueShouldBe::kScaledStepValueShouldBeInteger => {
                step = (step * D::FromInt(description.step_scale_factor)).Round();
                if step < D::FromInt(1) {
                    step = D::FromInt(1);
                }
            }
        }
        debug_assert!(step > D::FromInt(0));
        step
    }
    // Source: step_range.cc:148-151.
    fn RoundByStep(&self, value: D, base: D) -> D {
        base + ((value - base) / self.step_).Round() * self.step_
    }
    // Source: step_range.cc:153-177.
    pub fn StepMismatch(&self, value: D) -> bool {
        if !self.has_step_ || !value.IsFinite() {
            return false;
        }
        let value = (value - self.step_base_).Abs();
        if !value.IsFinite() || value / D::FromComponents(false, 0, 1 << 53) > self.step_ {
            return false;
        }
        let remainder = (value - self.step_ * (value / self.step_).Round()).Abs();
        let error = self.AcceptableError();
        error < remainder && remainder < self.step_ - error
    }
    // Source: step_range.cc:179-198.
    pub fn StepSnappedMaximum(&self) -> D {
        let base = self.StepBase();
        let step = self.Step();
        if step < D::FromInt(0) || base - step == base || !(base / step).IsFinite() {
            return D::Nan();
        }
        let divided = (self.Maximum() - base) / step;
        let mut aligned = if divided == divided.Floor() {
            self.Maximum()
        } else {
            base + divided.Floor() * step
        };
        if aligned > self.Maximum() {
            aligned = aligned - step;
        }
        debug_assert!(aligned <= self.Maximum());
        if aligned < self.Minimum() {
            D::Nan()
        } else {
            aligned
        }
    }
    // Source: step_range.cc:200-203.
    pub fn HasReversedRange(&self) -> bool {
        self.supports_reversed_range_ && self.Maximum() < self.Minimum()
    }
}
