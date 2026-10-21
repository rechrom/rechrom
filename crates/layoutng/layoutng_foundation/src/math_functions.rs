// C++: src/foundation/blink_geometry/geometry/math_functions.h/.cc
// These are the float instantiations used by CalculationExpressionOperationNode.

use crate::calculation_operator::CalculationOperator;

// cpp: foundation/blink_geometry/geometry/math_functions.h:24-52
fn nearest_multiples(mut a: f32, mut b: f32) -> (f32, f32) {
    let is_negative = a < 0.0;
    a = a.abs();
    b = b.abs();
    let mut c = -(a % b);
    let mut lower = a + c;
    if a.abs() > c.abs() {
        std::mem::swap(&mut a, &mut c);
    }
    if a.abs() > b.abs() {
        std::mem::swap(&mut a, &mut b);
    }
    if b.abs() > c.abs() {
        std::mem::swap(&mut b, &mut c);
    }
    let mut upper = a + b + c;
    if is_negative {
        std::mem::swap(&mut lower, &mut upper);
        lower = -lower;
        upper = -upper;
    }
    (lower, upper)
}

// cpp: foundation/blink_geometry/geometry/math_functions.h:279-283
pub fn EvaluateSignFunction(value: f32) -> f32 {
    if value == 0.0 || value.is_nan() {
        value
    } else if value > 0.0 {
        1.0
    } else {
        -1.0
    }
}

// cpp: foundation/blink_geometry/geometry/math_functions.h:151-166
pub fn EvaluateRoundDownFunction(a: f32, b: f32) -> f32 {
    let (lower, _) = nearest_multiples(a, b);
    if a.is_finite() && b.is_infinite() {
        if a == 0.0 {
            a
        } else if a.is_sign_negative() {
            f32::NEG_INFINITY
        } else {
            0.0
        }
    } else {
        lower
    }
}

// cpp: foundation/blink_geometry/geometry/math_functions.h:168-275
pub fn EvaluateSteppedValueFunction(op: CalculationOperator, a: f32, b: f32) -> f32 {
    use CalculationOperator::*;
    assert!(matches!(
        op,
        kRoundNearest | kRoundUp | kRoundDown | kRoundToZero | kMod | kRem
    ));
    if b == 0.0 || (a.is_infinite() && b.is_infinite()) {
        return f32::NAN;
    }
    if matches!(op, kRoundNearest | kRoundUp | kRoundDown | kRoundToZero)
        && (a % b == 0.0 || (a.is_infinite() && !b.is_infinite()))
    {
        return a;
    }
    if matches!(op, kMod | kRem) && a.is_infinite() {
        return f32::NAN;
    }
    let (mut lower, mut upper) = nearest_multiples(a, b);
    match op {
        kRoundNearest => {
            if a.is_finite() && b.is_infinite() {
                return 0.0f32.copysign(a);
            }
            let a_is_negative = a < 0.0;
            if a_is_negative {
                std::mem::swap(&mut lower, &mut upper);
            }
            let distance = (a % b).abs();
            let half_b = b.abs() / 2.0;
            if distance < half_b || (a_is_negative && distance == half_b) {
                lower
            } else {
                upper
            }
        }
        kRoundUp => {
            if a.is_finite() && b.is_infinite() {
                if a == 0.0 {
                    a
                } else if a.is_sign_negative() {
                    -0.0
                } else {
                    f32::INFINITY
                }
            } else {
                upper
            }
        }
        kRoundDown => EvaluateRoundDownFunction(a, b),
        kRoundToZero => {
            if a.is_finite() && b.is_infinite() {
                0.0f32.copysign(a)
            } else if upper.abs() < lower.abs() {
                upper
            } else {
                lower
            }
        }
        kMod => {
            if b.is_infinite() && a.is_sign_negative() != b.is_sign_negative() {
                return f32::NAN;
            }
            let result = a % b;
            if result == 0.0 {
                0.0f32.copysign(b)
            } else if result.is_sign_negative() != b.is_sign_negative() {
                result + b
            } else {
                result
            }
        }
        kRem => a % b,
        _ => unreachable!(),
    }
}

// C++: src/foundation/gfx_geometry/sin_cos_degrees.h:25-102.
// cpp: foundation/gfx_geometry/sin_cos_degrees.h:25-102
fn sin_cos_degrees(mut degrees: f64) -> (f64, f64) {
    if degrees > -90_000_000.0 && degrees < 90_000_000.0 {
        let n45 = degrees / 45.0;
        let mut octant = n45 as i32;
        if octant as f64 == n45 {
            let half_sqrt2 = std::f64::consts::SQRT_2 / 2.0;
            return [
                (0.0, 1.0),
                (half_sqrt2, half_sqrt2),
                (1.0, 0.0),
                (half_sqrt2, -half_sqrt2),
                (0.0, -1.0),
                (-half_sqrt2, -half_sqrt2),
                (-1.0, 0.0),
                (-half_sqrt2, half_sqrt2),
            ][(octant & 7) as usize];
        }
        if degrees < 0.0 {
            octant -= 1;
        }
        degrees -= octant as f64 * 45.0;
        if octant & 1 != 0 {
            degrees = 45.0 - degrees;
        }
        let radians = degrees.to_radians();
        let mut sine = radians.sin();
        let mut cosine = radians.cos();
        if (octant + 1) & 2 != 0 {
            std::mem::swap(&mut sine, &mut cosine);
        }
        if octant & 4 != 0 {
            sine = -sine;
        }
        if (octant + 2) & 4 != 0 {
            cosine = -cosine;
        }
        return (sine, cosine);
    }
    let radians = (degrees % 360.0).to_radians();
    (radians.sin(), radians.cos())
}

// cpp: foundation/blink_geometry/geometry/math_functions.h:109-149
pub fn EvaluateTrigonometricFunction(op: CalculationOperator, a: f32, b: Option<f32>) -> f32 {
    use CalculationOperator::*;
    match op {
        kSin => sin_cos_degrees(a as f64).0 as f32,
        kCos => sin_cos_degrees(a as f64).1 as f32,
        kTan => {
            if a > -90_000_000.0 && a < 90_000_000.0 {
                let n45 = a / 45.0;
                let octant = n45 as i32;
                if octant as f32 == n45 {
                    return [
                        0.0,
                        1.0,
                        f32::INFINITY,
                        -1.0,
                        0.0,
                        1.0,
                        f32::NEG_INFINITY,
                        -1.0,
                    ][(octant & 7) as usize];
                }
            }
            a.to_radians().tan()
        }
        kAsin => a.asin().to_degrees(),
        kAcos => a.acos().to_degrees(),
        kAtan => a.atan().to_degrees(),
        kAtan2 => a
            .atan2(b.expect("atan2 requires a second operand"))
            .to_degrees(),
        _ => panic!("operator is not trigonometric: {op:?}"),
    }
}

// cpp: foundation/blink_geometry/geometry/math_functions.cc:10-49
pub fn ComputeCSSRandomValue(
    random_base_value: f64,
    min: f64,
    mut max: f64,
    step: Option<f64>,
) -> f64 {
    if max < min {
        max = min;
    }
    if min.is_infinite() {
        return min;
    }
    if (max - min).is_infinite() || min.is_nan() || max.is_nan() {
        return f64::NAN;
    }
    let Some(step) = step else {
        return min + random_base_value * (max - min);
    };
    if step.is_infinite() {
        return min;
    }
    if step.is_nan() {
        return f64::NAN;
    }
    if step <= 0.0 {
        return min + random_base_value * (max - min);
    }
    let n = ((max - min) / step) as i32;
    let index = (random_base_value * (n as f64 + 1.0)).floor() as i32;
    min + index as f64 * step
}

#[cfg(test)]
mod tests {
    use super::*;
    use CalculationOperator::*;

    #[test]
    fn stepped_value_edge_cases() {
        assert_eq!(EvaluateSteppedValueFunction(kRoundNearest, 5.0, 2.0), 6.0);
        assert_eq!(EvaluateSteppedValueFunction(kRoundNearest, -5.0, 2.0), -4.0);
        assert_eq!(EvaluateSteppedValueFunction(kMod, -5.0, 3.0), 1.0);
        assert_eq!(EvaluateSteppedValueFunction(kRem, -5.0, 3.0), -2.0);
        assert_eq!(EvaluateSteppedValueFunction(kRoundDown, -5.0, 2.0), -6.0);
        assert!(EvaluateSteppedValueFunction(kMod, 2.0, 0.0).is_nan());
        assert!(EvaluateSteppedValueFunction(kRoundUp, -0.0, f32::INFINITY).is_sign_negative());
    }

    #[test]
    fn trigonometric_and_random_values() {
        assert_eq!(EvaluateTrigonometricFunction(kSin, 180.0, None), 0.0);
        assert_eq!(EvaluateTrigonometricFunction(kCos, 90.0, None), 0.0);
        assert_eq!(EvaluateTrigonometricFunction(kSin, -90.0, None), -1.0);
        assert_eq!(
            EvaluateTrigonometricFunction(kTan, 90.0, None),
            f32::INFINITY
        );
        assert_eq!(EvaluateTrigonometricFunction(kTan, 135.0, None), -1.0);
        assert_eq!(ComputeCSSRandomValue(0.5, 0.0, 10.0, Some(3.0)), 6.0);
    }
}
