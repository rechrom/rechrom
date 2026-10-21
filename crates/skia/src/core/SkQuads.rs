// Source: src/core/SkQuads.h:15-15
pub struct SkQuads;

// Source: src/core/SkQuads.h:32-36
#[derive(Clone, Copy, Debug)]
pub struct RootResult {
    pub discriminant: f64,
    pub root0: f64,
    pub root1: f64,
}

// Source: src/core/SkQuads.cpp:18-31
fn solve_linear(m: f64, b: f64, solution: &mut [f64; 2]) -> usize {
    if crate::src::core::SkFloatingPoint::sk_double_nearly_zero(m) {
        solution[0] = 0.0;
        if crate::src::core::SkFloatingPoint::sk_double_nearly_zero(b) {
            return 1;
        }
        return 0;
    }
    solution[0] = -b / m;
    if !solution[0].is_finite() {
        return 0;
    }
    1
}

// Source: src/core/SkQuads.cpp:37-45
fn close_to_linear(a: f64, b: f64) -> bool {
    if a != 0.0 {
        return (b / a).abs() >= 1.0e16;
    }
    true
}

// Source: src/core/SkQuads.cpp:47-96
impl SkQuads {
    // Source: src/core/SkQuads.h:30-30
    pub fn Discriminant(a: f64, b: f64, c: f64) -> f64 {
        let b2 = b * b;
        let ac = a * c;
        let rough_discriminant = b2 - ac;
        if 3.0 * rough_discriminant.abs() >= b2 + ac {
            return rough_discriminant;
        }
        let b2_rounding_error = b.mul_add(b, -b2);
        let ac_rounding_error = a.mul_add(c, -ac);
        let discriminant = (b2 - ac) + (b2_rounding_error - ac_rounding_error);
        discriminant
    }
}

// Source: src/core/SkQuads.cpp:98-129
impl SkQuads {
    // Source: src/core/SkQuads.h:49-59
    pub fn Roots(a: f64, b: f64, c: f64) -> RootResult {
        let discriminant = Self::Discriminant(a, b, c);
        if a == 0.0 {
            let root = if b == 0.0 {
                if c == 0.0 {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            } else {
                c / (2.0 * b)
            };
            return RootResult {
                discriminant,
                root0: root,
                root1: root,
            };
        }
        debug_assert!(a != 0.0);
        if discriminant == 0.0 {
            return RootResult {
                discriminant,
                root0: b / a,
                root1: b / a,
            };
        }
        if discriminant > 0.0 {
            let d = discriminant.sqrt();
            let r = if b > 0.0 { b + d } else { b - d };
            return RootResult {
                discriminant,
                root0: r / a,
                root1: c / r,
            };
        }
        RootResult {
            discriminant,
            root0: f64::NAN,
            root1: f64::NAN,
        }
    }
}

// Source: src/core/SkQuads.cpp:131-133
fn zero_if_tiny(x: f64) -> f64 {
    if crate::src::core::SkFloatingPoint::sk_double_nearly_zero(x) {
        0.0
    } else {
        x
    }
}

// Source: src/core/SkQuads.cpp:135-162
impl SkQuads {
    // Source: src/core/SkQuads.h:66-66
    pub fn RootsReal(a: f64, b: f64, c: f64, solution: &mut [f64; 2]) -> usize {
        if close_to_linear(a, b) {
            return solve_linear(b, c, solution);
        }
        debug_assert!(a != 0.0);
        let RootResult {
            discriminant,
            root0,
            root1,
        } = Self::Roots(a, -0.5 * b, c);
        if !discriminant.is_finite() || discriminant < 0.0 {
            return 0;
        }
        let mut roots = 0;
        let r0 = zero_if_tiny(root0);
        if r0.is_finite() {
            solution[roots] = r0;
            roots += 1;
        }
        let r1 = zero_if_tiny(root1);
        if r1.is_finite() {
            solution[roots] = r1;
            roots += 1;
        }
        if roots == 2
            && crate::src::core::SkFloatingPoint::sk_doubles_nearly_equal_ulps(
                solution[0],
                solution[1],
                16,
            )
        {
            roots = 1;
        }
        roots
    }
}

// Source: src/core/SkQuads.cpp:164-167
impl SkQuads {
    // Source: src/core/SkQuads.h:72-72
    pub fn EvalAt(a: f64, b: f64, c: f64, t: f64) -> f64 {
        a.mul_add(t, b).mul_add(t, c)
    }
}
