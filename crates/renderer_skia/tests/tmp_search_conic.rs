#![cfg(all(feature = "pure_replay", feature = "source_replay"))]
use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb as V};
use layoutng_assembly::internal::layout_input_types::Color;
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact, PaintRect};

fn command(verb: V, x: f64, y: f64, cx: f64, cy: f64, w: f64) -> PaintPathCommand {
    PaintPathCommand {
        verb,
        point: Offset { x, y },
        control1: Offset { x: cx, y: cy },
        conic_weight: w,
        ..Default::default()
    }
}

#[test]
fn github_search_conic_fill_matches_native_skia() {
    let path = vec![
        command(V::kMoveTo, 11.5, 7.0, 0.0, 0.0, 1.0),
        command(
            V::kConicTo,
            7.001499653,
            2.433924675,
            11.568082809,
            2.433924675,
            0.701815844,
        ),
        command(
            V::kConicTo,
            2.503000021,
            7.0,
            2.434916258,
            2.433925390,
            0.701815844,
        ),
        command(
            V::kConicTo,
            7.001499653,
            11.431924820,
            2.569082022,
            11.431924820,
            0.712358356,
        ),
        command(
            V::kConicTo,
            11.5,
            7.0,
            11.433917046,
            11.431924820,
            0.712358356,
        ),
        command(V::kClose, 0.0, 0.0, 0.0, 0.0, 1.0),
    ];
    let artifact = PaintArtifact {
        items: vec![DisplayItem {
            r#type: T::kDrawPath,
            path,
            rect: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 16.0,
                height: 16.0,
            },
            color: Color {
                red: 0.1,
                green: 0.15,
                blue: 0.2,
                alpha: 1.0,
            },
            antialias: true,
            ..Default::default()
        }]
        .into(),
        ..Default::default()
    };
    let native =
        raster::source_replay::RasterizeSourceDisplayItemListWithScale(&artifact, 32, 32, 2.0);
    let pure = raster::pure_replay::RasterizeSourceDisplayItemListWithScale(&artifact, 32, 32, 2.0);
    let diff = native
        .chunks_exact(4)
        .zip(pure.chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(diff, 0, "{diff} pixels differ");
}
