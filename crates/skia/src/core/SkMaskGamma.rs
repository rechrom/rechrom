// CPU implementation moved from renderer/glyphs/mask_gamma.rs.

// SkMaskGamma A8 coverage correction, translated to Rust.
use crate::compat::commands::Color;

fn linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}
fn srgb(value: f32) -> f32 {
    if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}
fn canonical(value: u8) -> u8 {
    let n = value >> 5;
    (n << 5) | (n << 2) | (n >> 1)
}

// SkMaskGamma.cpp: SkTMaskGamma_build_correcting_lut, with sRGB and zero
// contrast, the project/Chromium macOS software surface configuration.
fn correcting_lut(source: u8) -> [u8; 256] {
    let src = f32::from(source) / 255.0;
    let dst = 1.0 - src;
    let lin_src = linear(src);
    let lin_dst = linear(dst);
    std::array::from_fn(|i| {
        let alpha = i as f32 / 255.0;
        let result = if (src - dst).abs() < 1.0 / 256.0 {
            alpha
        } else {
            (srgb(lin_src * alpha + (1.0 - alpha) * lin_dst) - dst) / (src - dst)
        };
        (255.0 * result + 0.5).floor().clamp(0.0, 255.0) as u8
    })
}

// SkMaskGamma.h: SkTMaskGamma<3,3,3> constructs eight canonical tables
// once per gamma/contrast configuration, then preBlend selects a table by the
// high three luminance bits. SkScalerContextRec::CachedMaskGamma retains this
// configuration across scaler contexts. Our surface configuration is fixed to
// sRGB (gamma=0) and contrast=0, so this immutable 2KB table set is sufficient.
fn canonical_luts() -> &'static [[u8; 256]; 8] {
    static TABLES: std::sync::OnceLock<[[u8; 256]; 8]> = std::sync::OnceLock::new();
    TABLES.get_or_init(|| std::array::from_fn(|i| correcting_lut(canonical((i as u8) << 5))))
}

// SkTypeface_mac_ct.cpp: onFilterRec. Skia canonicalizes to three bits,
// halves the canonical foreground for gray CoreGraphics smoothing, then
// canonicalizes again. A8 conversion then uses canonical luminance.
pub(crate) fn mac_smoothing_lut(
    color: Color,
    behavior: crate::cpu::scaler_context_mac_ct::SmoothBehavior,
) -> [u8; 256] {
    let channel = |value: f32| {
        let value = (value.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8;
        let value = canonical(value);
        let value = match behavior {
            crate::cpu::scaler_context_mac_ct::SmoothBehavior::Gray => value / 2,
            crate::cpu::scaler_context_mac_ct::SmoothBehavior::Subpixel => {
                (u32::from(value) * 3 / 4) as u8
            }
            crate::cpu::scaler_context_mac_ct::SmoothBehavior::None => value,
        };
        u32::from(canonical(value))
    };
    // SkScalerContext.cpp: PreprocessRec converts the filtered canonical RGB
    // luminance color to A8 luminance and canonicalizes a third time.
    let luminance =
        ((channel(color.red) * 54 + channel(color.green) * 183 + channel(color.blue) * 19) >> 8)
            as u8;
    canonical_luts()[usize::from(luminance >> 5)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::scaler_context_mac_ct::SmoothBehavior;

    // Independent pre-cache scalar path, retaining its operation order. No
    // production canonicalization, color-conversion, or cache helper is used.
    fn previous_scalar_lut(color: Color, behavior: SmoothBehavior) -> [u8; 256] {
        let channel = |value: f32| {
            let canonical = |value: u8| {
                let n = value >> 5;
                (n << 5) | (n << 2) | (n >> 1)
            };
            let value = canonical((value.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8);
            let value = match behavior {
                SmoothBehavior::None => value,
                SmoothBehavior::Gray => value / 2,
                SmoothBehavior::Subpixel => (u32::from(value) * 3 / 4) as u8,
            };
            u32::from(canonical(value))
        };
        let luminance = ((channel(color.red) * 54
            + channel(color.green) * 183
            + channel(color.blue) * 19)
            >> 8) as u8;
        let n = luminance >> 5;
        previous_scalar_table((n << 5) | (n << 2) | (n >> 1))
    }

    fn previous_scalar_table(source: u8) -> [u8; 256] {
        let to_linear = |v: f32| {
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        let from_linear = |v: f32| {
            if v <= 0.0031308 {
                v * 12.92
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            }
        };
        let foreground = f32::from(source) / 255.0;
        let background = 1.0 - foreground;
        let fg = to_linear(foreground);
        let bg = to_linear(background);
        let mut result = [0; 256];
        for (i, entry) in result.iter_mut().enumerate() {
            let a = i as f32 / 255.0;
            let corrected = if (foreground - background).abs() < 1.0 / 256.0 {
                a
            } else {
                (from_linear(fg * a + (1.0 - a) * bg) - background) / (foreground - background)
            };
            *entry = (255.0 * corrected + 0.5).floor().clamp(0.0, 255.0) as u8;
        }
        result
    }

    #[test]
    fn mask_gamma_all_canonical_tables_are_exact_and_reused() {
        let tables = canonical_luts();
        for (i, &source) in [0, 36, 73, 109, 146, 182, 219, 255].iter().enumerate() {
            assert_eq!(
                tables[i],
                previous_scalar_table(source),
                "luminance={source}"
            );
        }
        assert!(core::ptr::eq(tables, canonical_luts()));
        // Native SkTMaskGamma<3,3,3>(contrast=0,gamma=0) oracle generated
        // by artifacts/baidu-16ms-20261002/mask-gamma/gamma-native-oracle.cc.
        let hash = tables
            .iter()
            .flatten()
            .fold(0xcbf29ce484222325_u64, |h, &b| {
                (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
            });
        assert_eq!(hash, 0x85528d34f620e3c5);
    }

    #[test]
    fn mask_gamma_all_behaviors_and_colors_match_previous_scalar() {
        let channels = [
            0.0,
            31.0 / 255.0,
            32.0 / 255.0,
            63.0 / 255.0,
            64.0 / 255.0,
            95.0 / 255.0,
            96.0 / 255.0,
            127.0 / 255.0,
            128.0 / 255.0,
            159.0 / 255.0,
            160.0 / 255.0,
            191.0 / 255.0,
            192.0 / 255.0,
            223.0 / 255.0,
            224.0 / 255.0,
            1.0,
        ];
        for behavior in [
            SmoothBehavior::None,
            SmoothBehavior::Gray,
            SmoothBehavior::Subpixel,
        ] {
            for red in channels {
                for green in channels {
                    for blue in channels {
                        let color = Color {
                            red,
                            green,
                            blue,
                            alpha: 0.37,
                        };
                        assert_eq!(
                            mac_smoothing_lut(color, behavior),
                            previous_scalar_lut(color, behavior)
                        );
                    }
                }
            }
            for i in 0..=255_u32 {
                for red in [(i as f32 - 0.5) / 255.0, (i as f32 + 0.5) / 255.0] {
                    let color = Color {
                        red,
                        green: ((i * 73) % 256) as f32 / 255.0,
                        blue: ((i * 37 + 17) % 256) as f32 / 255.0,
                        alpha: 1.0,
                    };
                    assert_eq!(
                        mac_smoothing_lut(color, behavior),
                        previous_scalar_lut(color, behavior)
                    );
                    assert_eq!(
                        mac_smoothing_lut(
                            Color {
                                alpha: 0.0,
                                ..color
                            },
                            behavior
                        ),
                        mac_smoothing_lut(color, behavior)
                    );
                }
            }
        }
    }

    #[test]
    fn mask_gamma_table_set_is_shared_by_concurrent_readers() {
        let readers: Vec<_> = (0..8)
            .map(|i| {
                std::thread::spawn(move || {
                    let tables = canonical_luts();
                    assert_eq!(
                        tables[i],
                        previous_scalar_table([0, 36, 73, 109, 146, 182, 219, 255][i])
                    );
                    tables.as_ptr() as usize
                })
            })
            .collect();
        let address = canonical_luts().as_ptr() as usize;
        for reader in readers {
            assert_eq!(reader.join().unwrap(), address);
        }
    }

    #[test]
    #[ignore = "manual Debug timing; no performance assertion"]
    fn mask_gamma_debug_cache_benchmark() {
        use std::time::Instant;
        let _ = canonical_luts();
        for behavior in [
            SmoothBehavior::None,
            SmoothBehavior::Gray,
            SmoothBehavior::Subpixel,
        ] {
            let mut before = Vec::new();
            let mut after = Vec::new();
            for _ in 0..3 {
                for old in [true, false] {
                    let start = Instant::now();
                    for i in 0..4096_u32 {
                        let color = Color {
                            red: (i % 256) as f32 / 255.0,
                            green: ((i * 73) % 256) as f32 / 255.0,
                            blue: ((i * 37 + 17) % 256) as f32 / 255.0,
                            alpha: 1.0,
                        };
                        let lut = if old {
                            previous_scalar_lut(std::hint::black_box(color), behavior)
                        } else {
                            mac_smoothing_lut(std::hint::black_box(color), behavior)
                        };
                        std::hint::black_box(lut);
                    }
                    let ms = start.elapsed().as_secs_f64() * 1000.0;
                    if old {
                        before.push(ms);
                    } else {
                        after.push(ms);
                    }
                }
            }
            before.sort_by(f64::total_cmp);
            after.sort_by(f64::total_cmp);
            eprintln!("MASK_GAMMA_BENCH behavior={} calls=4096 before_scalar_ms={:.3} after_cached_ms={:.3} speedup={:.2}",behavior as u8,before[1],after[1],before[1]/after[1]);
        }
    }
}
