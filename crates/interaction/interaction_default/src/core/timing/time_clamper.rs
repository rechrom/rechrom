// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium 6c1d401f: time_clamper.cc:19-107; time_clamper.h:22-53.
/// Immutable time-resolution owner. The embedding supplies the original random
/// secret once; the algorithm owns no clock or random-number service.
pub struct TimeClamper {
    secret: u64,
}
impl TimeClamper {
    pub const fn new(secret: u64) -> Self {
        Self { secret }
    }
    pub fn ClampTimeResolution(
        &self,
        mut microseconds: i64,
        cross_origin_isolated_capability: bool,
    ) -> i64 {
        let was_negative = microseconds < 0;
        if was_negative {
            microseconds = -microseconds.max(-i64::MAX);
        }
        let lower = microseconds % 10_000_000_000;
        let upper = microseconds - lower;
        let resolution = if cross_origin_isolated_capability {
            5
        } else {
            100
        };
        let mut clamped = lower - lower % resolution;
        if lower as f64 >= self.ThresholdFor(clamped, resolution) {
            clamped += resolution;
        }
        clamped += upper.min(i64::MAX - clamped);
        if was_negative {
            -clamped
        } else {
            clamped
        }
    }
    fn ThresholdFor(&self, clamped: i64, resolution: i64) -> f64 {
        clamped as f64
            + resolution as f64 * Self::ToDouble(Self::MurmurHash3(clamped as u64 ^ self.secret))
    }
    fn ToDouble(value: u64) -> f64 {
        f64::from_bits((value & 0x000f_ffff_ffff_ffff) | 0x3ff0_0000_0000_0000) - 1.0
    }
    fn MurmurHash3(mut value: u64) -> u64 {
        value ^= value >> 33;
        value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
        value ^= value >> 33;
        value = value.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
        value ^ (value >> 33)
    }
}
