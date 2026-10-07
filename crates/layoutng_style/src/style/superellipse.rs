// cpp: layoutng_style/style/superellipse.h:15-68
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Superellipse {
    param_: f64,
}

#[allow(non_snake_case, non_upper_case_globals)]
impl Superellipse {
    // cpp: layoutng_style/style/superellipse.h:19
    pub const kHighCurvatureThreshold: f32 = 16.0;

    // cpp: layoutng_style/style/superellipse.h:21-41
    pub const fn Bevel() -> Self {
        Self::new(0.0)
    }
    pub const fn Notch() -> Self {
        Self::new(f64::NEG_INFINITY)
    }
    pub const fn Round() -> Self {
        Self::new(1.0)
    }
    pub const fn Scoop() -> Self {
        Self::new(-1.0)
    }
    pub const fn Squircle() -> Self {
        Self::new(2.0)
    }
    pub const fn Square() -> Self {
        Self::new(f64::INFINITY)
    }

    // cpp: layoutng_style/style/superellipse.h:43-53
    pub const fn IsDegenerate(&self) -> bool {
        self.param_ >= Self::kHighCurvatureThreshold as f64
    }
    pub const fn IsFullyConcave(&self) -> bool {
        self.param_ <= -(Self::kHighCurvatureThreshold as f64)
    }
    pub const fn IsConvex(&self) -> bool {
        self.param_ >= 0.0
    }

    // cpp: layoutng_style/style/superellipse.h:54
    pub const fn new(param: f64) -> Self {
        Self { param_: param }
    }

    // cpp: layoutng_style/style/superellipse.h:56-57
    pub const fn Parameter(&self) -> f64 {
        self.param_
    }

    // cpp: layoutng_style/style/superellipse.h:59-62
    pub fn Exponent(&self) -> f64 {
        // Matches ClampTo<float>(double, -16.f, 16.f): compare as double,
        // then convert the bounded value to float before passing to pow.
        debug_assert!(!self.param_.is_nan());
        let exponent: f32 = if self.param_ >= Self::kHighCurvatureThreshold as f64 {
            Self::kHighCurvatureThreshold
        } else if self.param_ <= -(Self::kHighCurvatureThreshold as f64) {
            -Self::kHighCurvatureThreshold
        } else {
            self.param_ as f32
        };
        2.0_f64.powf(exponent as f64)
    }
}
