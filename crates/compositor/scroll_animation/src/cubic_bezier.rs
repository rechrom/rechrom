//! Geometry dependency adapter; excluded from cc translation coverage.
//! Algorithms follow ui/gfx/geometry/cubic_bezier.{h,cc}: 11-entry spline initial
//! guess, four Newton steps and bisection fallback, including endpoint gradients
//! and finite-value handling. The adapter owns no compositor lifecycle.

const SAMPLES: usize = 11;
const EPSILON: f64 = 1e-7;

#[derive(Clone, Debug)]
pub struct CubicBezier {
    ax_: f64,
    bx_: f64,
    cx_: f64,
    ay_: f64,
    by_: f64,
    cy_: f64,
    start_gradient_: f64,
    end_gradient_: f64,
    range_min_: f64,
    range_max_: f64,
    spline_samples_: [f64; SAMPLES],
    monotonically_increasing_: bool,
}
impl CubicBezier {
    fn ToFinite(value: f64) -> f64 {
        if value.is_infinite() {
            if value > 0.0 {
                return f64::MAX;
            }
            return f64::MIN;
        }
        value
    }
    pub fn new(p1x: f64, p1y: f64, p2x: f64, p2y: f64) -> Self {
        let mut out = Self {
            ax_: 0.0,
            bx_: 0.0,
            cx_: 0.0,
            ay_: 0.0,
            by_: 0.0,
            cy_: 0.0,
            start_gradient_: 0.0,
            end_gradient_: 0.0,
            range_min_: 0.0,
            range_max_: 1.0,
            spline_samples_: [0.0; SAMPLES],
            monotonically_increasing_: false,
        };
        out.InitCoefficients(p1x, p1y, p2x, p2y);
        out.InitGradients(p1x, p1y, p2x, p2y);
        out.InitRange(p1y, p2y);
        out.InitSpline();
        out
    }
    fn InitCoefficients(&mut self, p1x: f64, p1y: f64, p2x: f64, p2y: f64) {
        self.cx_ = 3.0 * p1x;
        self.bx_ = 3.0 * (p2x - p1x) - self.cx_;
        self.ax_ = 1.0 - self.cx_ - self.bx_;
        self.cy_ = Self::ToFinite(3.0 * p1y);
        self.by_ = Self::ToFinite(3.0 * (p2y - p1y) - self.cy_);
        self.ay_ = Self::ToFinite(1.0 - self.cy_ - self.by_);
        self.monotonically_increasing_ = p1x >= 0.0 && p1x <= 1.0 && p2x >= 0.0 && p2x <= 1.0;
    }
    fn InitGradients(&mut self, p1x: f64, p1y: f64, p2x: f64, p2y: f64) {
        self.start_gradient_ = if p1x > 0.0 {
            p1y / p1x
        } else if p1y == 0.0 && p2x > 0.0 {
            p2y / p2x
        } else if p1y == 0.0 && p2y == 0.0 {
            1.0
        } else {
            0.0
        };
        self.end_gradient_ = if p2x < 1.0 {
            (p2y - 1.0) / (p2x - 1.0)
        } else if p2y == 1.0 && p1x < 1.0 {
            (p1y - 1.0) / (p1x - 1.0)
        } else if p2y == 1.0 && p1y == 1.0 {
            1.0
        } else {
            0.0
        };
    }
    fn InitRange(&mut self, p1y: f64, p2y: f64) {
        self.range_min_ = 0.0;
        self.range_max_ = 1.0;
        if 0.0 <= p1y && p1y < 1.0 && 0.0 <= p2y && p2y <= 1.0 {
            return;
        }
        let epsilon = EPSILON;
        let a = 3.0 * self.ay_;
        let b = 2.0 * self.by_;
        let c = self.cy_;
        if a.abs() < epsilon && b.abs() < epsilon {
            return;
        }
        let t1;
        let mut t2 = 0.0;
        if a.abs() < epsilon {
            t1 = -c / b;
        } else {
            let discriminant = b * b - 4.0 * a * c;
            if discriminant < 0.0 {
                return;
            }
            let discriminant_sqrt = discriminant.sqrt();
            t1 = (-b + discriminant_sqrt) / (2.0 * a);
            t2 = (-b - discriminant_sqrt) / (2.0 * a);
        }
        let mut sol1 = 0.0;
        let mut sol2 = 0.0;
        if 0.0 < t1 && t1 < 1.0 {
            sol1 = self.SampleCurveY(t1);
        }
        if 0.0 < t2 && t2 < 1.0 {
            sol2 = self.SampleCurveY(t2);
        }
        for solution in [sol1, sol2] {
            if solution < self.range_min_ {
                self.range_min_ = solution;
            }
            if self.range_max_ < solution {
                self.range_max_ = solution;
            }
        }
    }
    fn InitSpline(&mut self) {
        let delta_t = 1.0 / (SAMPLES - 1) as f64;
        for i in 0..SAMPLES {
            self.spline_samples_[i] = self.SampleCurveX(i as f64 * delta_t);
        }
    }
    pub fn SampleCurveX(&self, t: f64) -> f64 {
        ((self.ax_ * t + self.bx_) * t + self.cx_) * t
    }
    pub fn SampleCurveY(&self, t: f64) -> f64 {
        Self::ToFinite(((self.ay_ * t + self.by_) * t + self.cy_) * t)
    }
    pub fn SampleCurveDerivativeX(&self, t: f64) -> f64 {
        (3.0 * self.ax_ * t + 2.0 * self.bx_) * t + self.cx_
    }
    pub fn SampleCurveDerivativeY(&self, t: f64) -> f64 {
        Self::ToFinite(
            Self::ToFinite(Self::ToFinite(3.0 * self.ay_) * t + Self::ToFinite(2.0 * self.by_)) * t
                + self.cy_,
        )
    }
    pub fn GetDefaultEpsilon() -> f64 {
        EPSILON
    }
    pub fn SolveCurveX(&self, x: f64, epsilon: f64) -> f64 {
        debug_assert!(x >= 0.0);
        debug_assert!(x <= 1.0);
        debug_assert!(self.monotonically_increasing_);
        // In the source t0/t1 are assigned by the guaranteed spline interval.
        // Option makes that definite-assignment guarantee explicit in Rust.
        let mut interval = None;
        let mut t2 = x;
        let delta_t = 1.0 / (SAMPLES - 1) as f64;
        for i in 1..SAMPLES {
            if x <= self.spline_samples_[i] {
                let t1 = delta_t * i as f64;
                let t0 = t1 - delta_t;
                t2 = t0
                    + (t1 - t0) * (x - self.spline_samples_[i - 1])
                        / (self.spline_samples_[i] - self.spline_samples_[i - 1]);
                interval = Some((t0, t1));
                break;
            }
        }
        let (mut t0, mut t1) = interval.expect("monotonic Bezier spline covers [0,1]");
        let newton_epsilon = if epsilon < EPSILON { epsilon } else { EPSILON };
        let mut x2 = 0.0;
        for _ in 0..4 {
            x2 = self.SampleCurveX(t2) - x;
            if x2.abs() < newton_epsilon {
                return t2;
            }
            let d2 = self.SampleCurveDerivativeX(t2);
            if d2.abs() < EPSILON {
                break;
            }
            t2 = t2 - x2 / d2;
        }
        if x2.abs() < epsilon {
            return t2;
        }
        while t0 < t1 {
            x2 = self.SampleCurveX(t2);
            if (x2 - x).abs() < epsilon {
                return t2;
            }
            if x > x2 {
                t0 = t2;
            } else {
                t1 = t2;
            }
            t2 = (t1 + t0) * 0.5;
        }
        t2
    }
    pub fn SolveWithEpsilon(&self, x: f64, epsilon: f64) -> f64 {
        if x < 0.0 {
            return Self::ToFinite(0.0 + self.start_gradient_ * x);
        }
        if x > 1.0 {
            return Self::ToFinite(1.0 + self.end_gradient_ * (x - 1.0));
        }
        self.SampleCurveY(self.SolveCurveX(x, epsilon))
    }
    pub fn Solve(&self, x: f64) -> f64 {
        self.SolveWithEpsilon(x, EPSILON)
    }
    pub fn SlopeWithEpsilon(&self, mut x: f64, epsilon: f64) -> f64 {
        if x < 0.0 {
            x = 0.0;
        } else if 1.0 < x {
            x = 1.0;
        }
        let t = self.SolveCurveX(x, epsilon);
        let dx = self.SampleCurveDerivativeX(t);
        let dy = self.SampleCurveDerivativeY(t);
        if dx == 0.0 && dy == 0.0 {
            return 0.0;
        }
        Self::ToFinite(dy / dx)
    }
    pub fn Slope(&self, x: f64) -> f64 {
        self.SlopeWithEpsilon(x, EPSILON)
    }
    pub fn GetX1(&self) -> f64 {
        self.cx_ / 3.0
    }
    pub fn GetY1(&self) -> f64 {
        self.cy_ / 3.0
    }
    pub fn GetX2(&self) -> f64 {
        (self.bx_ + self.cx_) / 3.0 + self.GetX1()
    }
    pub fn GetY2(&self) -> f64 {
        (self.by_ + self.cy_) / 3.0 + self.GetY1()
    }
    pub fn range_min(&self) -> f64 {
        self.range_min_
    }
    pub fn range_max(&self) -> f64 {
        self.range_max_
    }
}
