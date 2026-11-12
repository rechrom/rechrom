// Copyright (C) 2005, 2006 Apple Inc. Chromium BSD; see Chromium LICENSE.
// Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/css/font_size_functions.cc:44-124,224-265,299-321;
// font_size_functions.h:57-65. Pure value model shared by native style/editing.
#[derive(Clone, Copy)]
pub struct FontSizeSettings {
    pub default_font_size: i32,
    pub default_fixed_font_size: i32,
    pub minimum_logical_font_size: i32,
    pub accessibility_scale_factor: f64,
}
#[derive(Clone, Copy)]
pub struct FontSizeDocumentInputs {
    pub settings: Option<FontSizeSettings>,
    pub quirks_mode: bool,
    pub text_scale_meta_tag_present: bool,
}
const QUIRKS: [[i32; 8]; 8] = [
    [9, 9, 9, 9, 11, 14, 18, 28],
    [9, 9, 9, 10, 12, 15, 20, 31],
    [9, 9, 9, 11, 13, 17, 22, 34],
    [9, 9, 10, 12, 14, 18, 24, 37],
    [9, 9, 10, 13, 16, 20, 26, 40],
    [9, 9, 11, 14, 17, 21, 28, 42],
    [9, 10, 12, 15, 17, 23, 30, 45],
    [9, 10, 13, 16, 18, 24, 32, 48],
];
const STRICT: [[i32; 8]; 8] = [
    [9, 9, 9, 9, 11, 14, 18, 27],
    [9, 9, 9, 10, 12, 15, 20, 30],
    [9, 9, 10, 11, 13, 17, 22, 33],
    [9, 9, 10, 12, 14, 18, 24, 36],
    [9, 10, 12, 13, 16, 20, 26, 39],
    [9, 10, 12, 14, 17, 21, 28, 42],
    [9, 10, 13, 15, 18, 23, 30, 45],
    [9, 10, 13, 16, 18, 24, 32, 48],
];
const FACTORS: [f32; 8] = [0.60, 0.75, 0.89, 1.0, 1.2, 1.5, 2.0, 3.0];
pub fn SnapToClosestFontScaleBucket(raw: f64) -> f64 {
    const BUCKETS: [f64; 10] = [0.85, 1.0, 1.15, 1.3, 1.5, 1.8, 2.0, 2.5, 3.0, 3.5];
    if raw <= BUCKETS[0] {
        return BUCKETS[0];
    }
    if raw >= BUCKETS[9] {
        return BUCKETS[9];
    }
    let upper = BUCKETS.partition_point(|candidate| *candidate < raw);
    let high = BUCKETS[upper];
    let low = BUCKETS[upper - 1];
    if (raw - low).abs() <= (raw - high).abs() {
        low
    } else {
        high
    }
}
fn Medium(inputs: FontSizeDocumentInputs, monospace: bool) -> i32 {
    let mut medium = inputs.settings.map_or(12, |s| {
        if monospace {
            s.default_fixed_font_size
        } else {
            s.default_font_size
        }
    });
    if let Some(settings) = inputs.settings {
        if inputs.text_scale_meta_tag_present {
            medium = (medium as f64
                * SnapToClosestFontScaleBucket(settings.accessibility_scale_factor))
            .round() as i32;
        }
    }
    medium
}
pub fn FontSizeForKeyword(inputs: FontSizeDocumentInputs, keyword: usize, monospace: bool) -> f32 {
    assert!((1..=8).contains(&keyword));
    let medium = Medium(inputs, monospace);
    if (9..=16).contains(&medium) {
        return (if inputs.quirks_mode { QUIRKS } else { STRICT })[(medium - 9) as usize]
            [keyword - 1] as f32;
    }
    let minimum = inputs
        .settings
        .map_or(1, |s| s.minimum_logical_font_size.max(1)) as f32;
    (FACTORS[keyword - 1] * medium as f32).max(minimum)
}
pub fn LegacyFontSize(inputs: FontSizeDocumentInputs, pixels: i32, monospace: bool) -> i32 {
    if inputs.settings.is_none() {
        return 1;
    }
    let medium = Medium(inputs, monospace);
    if (9..=16).contains(&medium) {
        let table = (if inputs.quirks_mode { QUIRKS } else { STRICT })[(medium - 9) as usize];
        for index in 1..7 {
            if pixels as i64 * 2 < (table[index] + table[index + 1]) as i64 {
                return index as i32;
            }
        }
    } else {
        for index in 1..7 {
            if pixels as f32 * 2.0 < (FACTORS[index] + FACTORS[index + 1]) * medium as f32 {
                return index as i32;
            }
        }
    }
    7
}
