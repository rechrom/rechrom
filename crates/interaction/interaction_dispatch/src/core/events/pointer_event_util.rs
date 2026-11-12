// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/pointer_event_util.h:1-38
// Source: third_party/blink/renderer/core/events/pointer_event_util.cc:1-185

use std::f64::consts::{FRAC_PI_2, PI, TAU};

// Source: pointer_event_util.h:14-34
// The source class has a deleted constructor and only static operations.
pub enum PointerEventUtil {}

impl PointerEventUtil {
    // Source: pointer_event_util.h:18; pointer_event_util.cc:12-51
    pub fn AzimuthFromTilt(tilt_x_degrees: f64, tilt_y_degrees: f64) -> f64 {
        debug_assert!((-90.0..=90.0).contains(&tilt_x_degrees));
        debug_assert!((-90.0..=90.0).contains(&tilt_y_degrees));
        if tilt_x_degrees == 0.0 {
            if tilt_y_degrees > 0.0 {
                return FRAC_PI_2;
            }
            if tilt_y_degrees < 0.0 {
                return 3.0 * FRAC_PI_2;
            }
            return 0.0;
        }
        if tilt_y_degrees == 0.0 {
            if tilt_x_degrees < 0.0 {
                return PI;
            }
            return 0.0;
        }
        if tilt_x_degrees.abs() == 90.0 || tilt_y_degrees.abs() == 90.0 {
            return 0.0;
        }
        debug_assert!(
            tilt_x_degrees != 0.0
                && tilt_y_degrees != 0.0
                && tilt_x_degrees.abs() != 90.0
                && tilt_y_degrees.abs() != 90.0
        );
        let tilt_x_radians = PI / 180.0 * tilt_x_degrees;
        let tilt_y_radians = PI / 180.0 * tilt_y_degrees;
        let tan_x = tilt_x_radians.tan();
        let tan_y = tilt_y_radians.tan();
        let azimuth_radians = tan_y.atan2(tan_x);
        let azimuth_radians = if azimuth_radians >= 0.0 {
            azimuth_radians
        } else {
            azimuth_radians + TAU
        };
        debug_assert!((0.0..=TAU).contains(&azimuth_radians));
        azimuth_radians
    }

    // Source: pointer_event_util.h:19; pointer_event_util.cc:53-74
    pub fn AltitudeFromTilt(tilt_x_degrees: f64, tilt_y_degrees: f64) -> f64 {
        debug_assert!((-90.0..=90.0).contains(&tilt_x_degrees));
        debug_assert!((-90.0..=90.0).contains(&tilt_y_degrees));
        let tilt_x_radians = PI / 180.0 * tilt_x_degrees;
        let tilt_y_radians = PI / 180.0 * tilt_y_degrees;
        if tilt_x_degrees.abs() == 90.0 || tilt_y_degrees.abs() == 90.0 {
            return 0.0;
        }
        if tilt_x_degrees == 0.0 {
            return FRAC_PI_2 - tilt_y_radians.abs();
        }
        if tilt_y_degrees == 0.0 {
            return FRAC_PI_2 - tilt_x_radians.abs();
        }
        (1.0 / (tilt_x_radians.tan().powf(2.0) + tilt_y_radians.tan().powf(2.0)).sqrt()).atan()
    }

    // Source: pointer_event_util.h:20-21; pointer_event_util.cc:76-99
    pub fn TiltXFromSpherical(azimuth_radians: f64, altitude_radians: f64) -> i32 {
        debug_assert!((0.0..=TAU).contains(&azimuth_radians));
        debug_assert!((0.0..=FRAC_PI_2).contains(&altitude_radians));
        if altitude_radians != 0.0 {
            // Javascript Math.round uses floor(x + 0.5), including negatives.
            return ((azimuth_radians.cos() / altitude_radians.tan()).atan() * (180.0 / PI) + 0.5)
                .floor() as i32;
        }
        if azimuth_radians == FRAC_PI_2 || azimuth_radians == 3.0 * FRAC_PI_2 {
            0
        } else if azimuth_radians < FRAC_PI_2 || azimuth_radians > 3.0 * FRAC_PI_2 {
            90
        } else {
            -90
        }
    }

    // Source: pointer_event_util.h:22-23; pointer_event_util.cc:101-122
    pub fn TiltYFromSpherical(azimuth_radians: f64, altitude_radians: f64) -> i32 {
        debug_assert!((0.0..=TAU).contains(&azimuth_radians));
        debug_assert!((0.0..=FRAC_PI_2).contains(&altitude_radians));
        if altitude_radians != 0.0 {
            return ((azimuth_radians.sin() / altitude_radians.tan()).atan() * (180.0 / PI) + 0.5)
                .floor() as i32;
        }
        if azimuth_radians == 0.0 || azimuth_radians == PI || azimuth_radians == TAU {
            0
        } else if azimuth_radians < PI {
            90
        } else {
            -90
        }
    }

    // Source: pointer_event_util.h:25-27; pointer_event_util.cc:124-142
    pub fn TransformToTiltInValidRange(mut tilt_degrees: i32) -> i32 {
        if (-90..=90).contains(&tilt_degrees) {
            return tilt_degrees;
        }
        while tilt_degrees > 90 {
            tilt_degrees -= 180;
        }
        while tilt_degrees < -90 {
            tilt_degrees += 180;
        }
        debug_assert!((-90..=90).contains(&tilt_degrees));
        tilt_degrees
    }

    // Source: pointer_event_util.h:28-30; pointer_event_util.cc:144-163
    pub fn TransformToAzimuthInValidRange(mut azimuth_radians: f64) -> f64 {
        if (0.0..=TAU).contains(&azimuth_radians) {
            return azimuth_radians;
        }
        while azimuth_radians > TAU {
            azimuth_radians -= TAU;
        }
        while azimuth_radians < 0.0 {
            azimuth_radians += TAU;
        }
        debug_assert!((0.0..=TAU).contains(&azimuth_radians));
        azimuth_radians
    }

    // Source: pointer_event_util.h:31-33; pointer_event_util.cc:165-184
    pub fn TransformToAltitudeInValidRange(mut altitude_radians: f64) -> f64 {
        if (0.0..=FRAC_PI_2).contains(&altitude_radians) {
            return altitude_radians;
        }
        while altitude_radians > FRAC_PI_2 {
            altitude_radians -= FRAC_PI_2;
        }
        while altitude_radians < 0.0 {
            altitude_radians += FRAC_PI_2;
        }
        debug_assert!((0.0..=FRAC_PI_2).contains(&altitude_radians));
        altitude_radians
    }
}
