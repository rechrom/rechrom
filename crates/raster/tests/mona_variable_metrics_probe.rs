use font_engine::{FontBackendFactory, FontVariation};
use raster::SkiaFontBackendFactory;

const OPSZ: u32 = u32::from_be_bytes(*b"opsz");
const WGHT: u32 = u32::from_be_bytes(*b"wght");

fn variations(values: &[(u32, f32)]) -> Vec<FontVariation> {
    values
        .iter()
        .map(|(tag, value)| FontVariation {
            tag: *tag,
            value: *value,
        })
        .collect()
}

#[test]
#[ignore = "diagnostic matrix for Chromium/CoreText variable-font metric parity"]
fn mona_sans_variable_axes_change_dpr2_metrics() {
    let bytes = include_bytes!("../../../work/MonaSansVF-wdth-wght-opsz-7a868ff47c93bd97.woff2");
    let factory = SkiaFontBackendFactory::default();
    let bytes = font_engine::SharedFontBytes::from(bytes.as_slice());
    let cases = [
        ("defaults", variations(&[])),
        ("wght440", variations(&[(WGHT, 440.0)])),
        ("opsz56", variations(&[(OPSZ, 56.0)])),
        ("wght440_opsz56", variations(&[(WGHT, 440.0), (OPSZ, 56.0)])),
        ("opsz56_wght440", variations(&[(OPSZ, 56.0), (WGHT, 440.0)])),
        ("wght440_opsz14", variations(&[(WGHT, 440.0), (OPSZ, 14.0)])),
    ];

    for (name, axes) in cases {
        let backend = factory.Create(
            &bytes,
            0,
            &axes,
            "Mona Sans VF",
            "Mona Sans VF",
            440.0,
            false,
        );
        for size in [56.0, 112.0] {
            let metrics = backend.Metrics(size, false, false);
            eprintln!(
                "{name:>18} size={size:>5.1} ascent={:.6} descent={:.6} leading={:.6} cap={:.6} x={:.6}",
                metrics.ascent,
                metrics.descent,
                metrics.leading,
                metrics.cap_height,
                metrics.x_height,
            );
        }
    }
}
