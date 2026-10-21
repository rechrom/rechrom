// cpp: layoutng_style/style/style_interest_delay.h:13-19
/// CSS interest delay in seconds; the default -1 means `normal`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StyleInterestDelay {
    seconds_: f64,
}

#[allow(non_snake_case)]
impl StyleInterestDelay {
    pub fn new(seconds: f64) -> Self {
        Self {
            seconds_: if seconds >= 0.0 { seconds } else { 0.0 },
        }
    }

    // cpp: layoutng_style/style/style_interest_delay.h:21-23
    pub fn DelaySeconds(&self) -> f64 {
        self.seconds_
    }
    pub fn IsNormal(&self) -> bool {
        self.seconds_ < 0.0
    }
}

// cpp: layoutng_style/style/style_interest_delay.h:25-34
// Derived f64 equality preserves NaN behavior.
impl Default for StyleInterestDelay {
    fn default() -> Self {
        Self { seconds_: -1.0 }
    }
}
