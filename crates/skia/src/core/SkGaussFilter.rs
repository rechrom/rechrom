// Copyright 2017 Google LLC. BSD-style license; see upstream LICENSE.

// Source: src/core/SkGaussFilter.h:15-15
// Source: src/core/SkGaussFilter.h:17-17
// Source: src/core/SkGaussFilter.h:19-19
// Source: src/core/SkGaussFilter.h:21-23
// Source: src/core/SkGaussFilter.h:26-27
// Source: src/core/SkGaussFilter.h:30-31
// Rust arrays retain the fixed storage; an active slice replaces begin/end pointers.
pub struct SkGaussFilter {
    fBasis: [f64; 6],
    fN: usize,
}
impl SkGaussFilter {
    pub const kGaussArrayMax: usize = 6;
    pub fn size(&self) -> usize {
        self.fN
    }
    pub fn radius(&self) -> i32 {
        self.fN as i32 - 1
    }
    pub fn width(&self) -> i32 {
        2 * self.radius() + 1
    }
    pub fn as_slice(&self) -> &[f64] {
        &self.fBasis[..self.fN]
    }
}

// Source: src/core/SkGaussFilter.cpp:16-16
const GOOD_ENOUGH: f64 = 1.0 / 100.0;

// Source: src/core/SkGaussFilter.cpp:20-20
// Source: src/core/SkGaussFilter.cpp:22-24
// Source: src/core/SkGaussFilter.cpp:26-26
// Source: src/core/SkGaussFilter.cpp:29-30
// Source: src/core/SkGaussFilter.cpp:35-37
// Source: src/core/SkGaussFilter.cpp:40-40
fn normalize(n: usize, gauss: &mut [f64; 6]) {
    let mut sum = 0.0;
    for i in (1..n).rev() {
        sum += 2.0 * gauss[i];
    }
    sum += gauss[0];
    for i in 0..n {
        gauss[i] /= sum;
    }
    sum = 0.0;
    for i in (1..n).rev() {
        sum += 2.0 * gauss[i];
    }
    gauss[0] = 1.0 - sum;
}

// Source: src/core/SkGaussFilter.cpp:43-44
// Source: src/core/SkGaussFilter.cpp:51-55
// Source: src/core/SkGaussFilter.cpp:58-61
// Source: src/core/SkGaussFilter.cpp:63-63
// Source: src/core/SkGaussFilter.cpp:66-74
// Source: src/core/SkGaussFilter.cpp:76-76
// Source: src/core/SkGaussFilter.cpp:82-85
// Source: src/core/SkGaussFilter.cpp:91-91
// Source: src/core/SkGaussFilter.cpp:94-97
// Source: src/core/SkGaussFilter.cpp:100-100
// Source: src/core/SkGaussFilter.cpp:102-102
fn calculate_bessel_factors(sigma: f64, gauss: &mut [f64; 6]) -> usize {
    let var = sigma * sigma;
    let bessel_i_0 = |t: f64| {
        let t_squared_over_4 = t * t / 4.0;
        let mut sum = 1.0;
        let mut factor = 1.0;
        let mut k = 1;
        while factor > 1.0 / 1000000.0 {
            factor *= t_squared_over_4 / ((k * k) as f64);
            sum += factor;
            k += 1;
        }
        sum
    };
    let bessel_i_1 = |t: f64| {
        let t_squared_over_4 = t * t / 4.0;
        let mut sum = t / 2.0;
        let mut factor = sum;
        let mut k = 1;
        while factor > 1.0 / 1000000.0 {
            factor *= t_squared_over_4 / ((k * (k + 1)) as f64);
            sum += factor;
            k += 1;
        }
        sum
    };
    let d = var.exp();
    let mut b = [0.0; SkGaussFilter::kGaussArrayMax];
    b[0] = bessel_i_0(var);
    b[1] = bessel_i_1(var);
    gauss[0] = b[0] / d;
    gauss[1] = b[1] / d;
    let mut n = 1;
    while gauss[n] > GOOD_ENOUGH {
        b[n + 1] = -(2.0 * n as f64 / var) * b[n] + b[n - 1];
        gauss[n + 1] = b[n + 1] / d;
        n += 1;
    }
    normalize(n, gauss);
    n
}

// Source: src/core/SkGaussFilter.cpp:105-106
// Source: src/core/SkGaussFilter.cpp:108-108
impl SkGaussFilter {
    pub fn new(sigma: f64) -> Self {
        crate::SkASSERT!(@source b"src/core/SkGaussFilter.cpp\0".as_ptr(),106; @condition b"0 <= sigma && sigma < 2\0".as_ptr(); 0.0 <= sigma && sigma < 2.0);
        let mut fBasis = [0.0; Self::kGaussArrayMax];
        let fN = calculate_bessel_factors(sigma, &mut fBasis);
        Self { fBasis, fN }
    }
}
