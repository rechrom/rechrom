//! PaintArtifact to independent Skia canvas conversion.
#![allow(non_snake_case)]
use crate::convert::{resources, ToSkia};
use paint::paint_engine::PaintArtifact;
use skia::cpu::canvas::Canvas;
pub use skia::cpu::canvas::GlyphRasterMode;

#[cfg(test)]
use layoutng_assembly::internal::layout_input_types::Color;
#[cfg(test)]
use paint::paint_engine::{DisplayItem, DisplayItemType, PaintRect};
#[cfg(test)]
fn rect_commands(
    rect: PaintRect,
) -> Option<Vec<layoutng_assembly::internal::layout_input::PaintPathCommand>> {
    use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb};
    // Test fixture commands; production path generation is owned by Skia.
    let mut path: Vec<_> = [
        (rect.x, rect.y),
        (rect.x + rect.width, rect.y),
        (rect.x + rect.width, rect.y + rect.height),
        (rect.x, rect.y + rect.height),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (x, y))| PaintPathCommand {
        verb: if i == 0 {
            PaintPathVerb::kMoveTo
        } else {
            PaintPathVerb::kLineTo
        },
        point: Offset { x, y },
        ..Default::default()
    })
    .collect();
    path.push(PaintPathCommand {
        verb: PaintPathVerb::kClose,
        ..Default::default()
    });
    Some(path)
}

pub fn RasterizeSourceDisplayItemList(list: &PaintArtifact, width: u32, height: u32) -> Vec<u8> {
    RasterizeSourceDisplayItemListWithGlyphRasterMode(
        list,
        width,
        height,
        GlyphRasterMode::Platform,
    )
}
pub fn RasterizeSourceDisplayItemListWithScale(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
) -> Vec<u8> {
    let resources = resources(list);
    let mut canvas = Canvas::new(&resources, width, height);
    canvas.set_scale(scale);
    for item in list.items.iter() {
        canvas.replay_item(&item.to_skia(), &resources);
    }
    canvas.finish()
}
pub fn RasterizeDisplayItemList(list: &PaintArtifact, width: u32, height: u32) -> Vec<u8> {
    let resources = resources(list);
    let mut canvas = Canvas::new(&resources, width, height);
    canvas.clear_transparent();
    for item in list.items.iter() {
        canvas.replay_item(&item.to_skia(), &resources);
    }
    canvas.finish_native()
}
#[derive(Clone, Copy)]
struct RasterTile {
    origin: u32,
    length: u32,
    leading_border: u32,
    trailing_border: u32,
}

fn tiles_for_axis(extent: u32) -> Vec<RasterTile> {
    let mut tiles = Vec::new();
    let mut origin = 0;
    while origin < extent {
        let leading_border = u32::from(origin != 0);
        let length = (256 - leading_border - 1).min(extent - origin);
        let trailing_border = u32::from(origin + length < extent);
        tiles.push(RasterTile {
            origin,
            length,
            leading_border,
            trailing_border,
        });
        origin += length;
    }
    tiles
}

// The PNG source renderer creates real 256-pixel canvases, translates their
// CTMs, and replays the complete scene into each one. Their actual canvas clips
// are part of Skia's geometry and blitter semantics. Window/scale/profile
// entry points above continue to replay one whole canvas.
pub fn RasterizeSourceDisplayItemListWithGlyphRasterMode(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    mode: GlyphRasterMode,
) -> Vec<u8> {
    let length = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .expect("raster dimensions overflow");
    let resources = resources(list);
    let mut output = vec![255; length];
    let x_tiles = tiles_for_axis(width);
    let y_tiles = tiles_for_axis(height);
    for y_tile in &y_tiles {
        for x_tile in &x_tiles {
            let tile_width = x_tile.leading_border + x_tile.length + x_tile.trailing_border;
            let tile_height = y_tile.leading_border + y_tile.length + y_tile.trailing_border;
            let mut canvas = Canvas::new(&resources, tile_width, tile_height);
            canvas.set_glyph_mode(mode);
            let mut translation = paint::paint_engine::DisplayItem {
                r#type: paint::paint_engine::DisplayItemType::kConcat,
                ..Default::default()
            };
            // SkCanvas::translate uses f32 after the integer tile-origin delta.
            translation.transform.values[12] =
                (i64::from(x_tile.leading_border) - i64::from(x_tile.origin)) as f32 as f64;
            translation.transform.values[13] =
                (i64::from(y_tile.leading_border) - i64::from(y_tile.origin)) as f32 as f64;
            canvas.replay_item(&translation.to_skia(), &resources);
            for item in list.items.iter() {
                canvas.replay_item(&item.to_skia(), &resources);
            }
            let tile_rgba = canvas.finish();
            for row in 0..y_tile.length as usize {
                let source = ((y_tile.leading_border as usize + row) * tile_width as usize
                    + x_tile.leading_border as usize)
                    * 4;
                let destination =
                    ((y_tile.origin as usize + row) * width as usize + x_tile.origin as usize) * 4;
                let count = x_tile.length as usize * 4;
                output[destination..destination + count]
                    .copy_from_slice(&tile_rgba[source..source + count]);
            }
        }
    }
    output
}
/// Explicit diagnostic entry point. The normal renderer has no per-item timers.
#[cfg(feature = "profiling")]
pub struct ReplayProfile {
    pub setup: std::time::Duration,
    pub readback: std::time::Duration,
    pub items: Vec<ReplayItemProfile>,
}
#[cfg(feature = "profiling")]
pub struct ReplayItemProfile {
    pub index: usize,
    pub elapsed: std::time::Duration,
    pub antialiased_clip: bool,
    pub partial_clip_pixels: usize,
}
#[cfg(feature = "profiling")]
pub fn ProfileSourceDisplayItemListWithScale(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
) -> (Vec<u8>, ReplayProfile) {
    use std::time::Instant;
    assert!(scale.is_finite() && scale > 0.0);
    let started = Instant::now();
    let resources = resources(list);
    let mut canvas = Canvas::new(&resources, width, height);
    canvas.set_scale(scale);
    let mut profile = ReplayProfile {
        setup: started.elapsed(),
        readback: Default::default(),
        items: Vec::new(),
    };
    // Clip inspection is deliberately outside the timed replay_item call.
    // Cache by Arc isn't possible here because Canvas owns a mutable Mask.
    for (index, item) in list.items.iter().enumerate() {
        let (antialiased_clip, partial_clip_pixels) = canvas.clip_profile();
        let started = Instant::now();
        canvas.replay_item(&item.to_skia(), &resources);
        profile.items.push(ReplayItemProfile {
            index,
            elapsed: started.elapsed(),
            antialiased_clip,
            partial_clip_pixels,
        });
    }
    let started = Instant::now();
    let pixels = canvas.finish();
    profile.readback = started.elapsed();
    (pixels, profile)
}

#[cfg(feature = "profiling")]
pub fn ProfileConstantMaskBlit(
    color: layoutng_assembly::internal::layout_input_types::Color,
    width: u32,
    height: u32,
) -> Vec<(&'static str, std::time::Duration, bool)> {
    skia::compat::canvas::ProfileConstantMaskBlit(color.to_skia(), width, height)
}
#[cfg(feature = "profiling")]
pub fn ProfileRoundedRectAttempt(
    rect: paint::paint_engine::PaintRect,
    radii: layoutng_assembly::internal::paint_input::PaintCornerRadii,
    width: u32,
    height: u32,
    scale: f64,
) -> (std::time::Duration, bool) {
    skia::compat::canvas::ProfileRoundedRectAttempt(
        rect.to_skia(),
        radii.to_skia(),
        width,
        height,
        scale,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use layoutng_assembly::internal::layout_input::{
        FontSmoothing, PaintPathCommand, PaintPathVerb, TextDecorationStyle, TransformMatrix,
    };
    use layoutng_assembly::internal::layout_input_types::Color;
    use layoutng_assembly::internal::paint_input::{
        PaintBlendMode, PaintCornerRadii, PaintShaderKind, PaintSpreadMethod, SvgStrokeLineCap,
        SvgStrokeLineJoin,
    };
    use paint::paint_engine::{DisplayItem, DisplayItemType, PaintRect};
    use std::sync::Arc;

    use layoutng_assembly::fragment_tree::PaintResources;
    use layoutng_assembly::internal::layout_input::{Offset, PaintImage};
    use layoutng_assembly::internal::paint_input::{
        PaintColorStop, PaintCornerRadius, PaintShader,
    };

    fn rgba_at(bytes: &[u8], width: usize, x: usize, y: usize) -> &[u8] {
        let offset = (y * width + x) * 4;
        &bytes[offset..offset + 4]
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn synthetic_italic_glyphs_match_native_skia() {
        use layoutng_assembly::fragment_tree::PaintGlyph;
        use layoutng_assembly::internal::layout_input::FontFace;
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../artifacts/cpp-reference/synthetic-italic-glyphs");
        let expected = std::fs::read(root.join("frames.rgba")).unwrap();
        let resources = Arc::new(PaintResources {
            fonts: vec![FontFace {
                family: "sans-serif".into(),
                bytes: std::fs::read(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
                ))
                .unwrap()
                .into(),
                ..Default::default()
            }],
            ..Default::default()
        });
        let mut actual = Vec::new();
        let mut comparisons = String::new();
        let mut differences = 0;
        for line in std::fs::read_to_string(root.join("input.tsv"))
            .unwrap()
            .lines()
        {
            let v: Vec<f64> = line
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect();
            let index = v[0] as usize;
            let mut matrix = DisplayItem {
                r#type: DisplayItemType::kConcat,
                ..Default::default()
            };
            for (i, value) in [0, 1, 4, 5].into_iter().zip(&v[2..6]) {
                matrix.transform.values[i] = *value;
            }
            matrix.transform.values[12] = 40.125;
            matrix.transform.values[13] = 32.375;
            let list = PaintArtifact {
                items: vec![
                    DisplayItem {
                        r#type: DisplayItemType::kDrawRect,
                        rect: PaintRect {
                            x: 0.0,
                            y: 0.0,
                            width: 80.0,
                            height: 64.0,
                        },
                        color: Color {
                            red: 0.9,
                            green: 0.8,
                            blue: 0.7,
                            alpha: 1.0,
                        },
                        antialias: false,
                        ..Default::default()
                    },
                    matrix,
                    DisplayItem {
                        r#type: DisplayItemType::kDrawGlyphRun,
                        font_size: v[1],
                        synthetic_italic: true,
                        color: Color {
                            red: v[7] as f32,
                            green: v[8] as f32,
                            blue: v[9] as f32,
                            alpha: v[10] as f32,
                        },
                        font_smoothing: if v[6] == 1.0 {
                            FontSmoothing::kNone
                        } else {
                            FontSmoothing::kAuto
                        },
                        glyphs: [(36, 0.0), (74, 12.25), (82, 23.875)]
                            .into_iter()
                            .map(|(id, x)| PaintGlyph {
                                id,
                                offset: Offset { x, y: 0.0 },
                                ..Default::default()
                            })
                            .collect(),
                        ..Default::default()
                    },
                ]
                .into(),
                resources: Some(resources.clone()),
                ..Default::default()
            };
            let frame = RasterizeDisplayItemList(&list, 80, 64);
            let native = &expected[index * 20480..(index + 1) * 20480];
            let diff = frame
                .chunks_exact(4)
                .zip(native.chunks_exact(4))
                .filter(|(a, b)| a != b)
                .count();
            comparisons.push_str(&format!("{index}\t{diff}\t5120\n"));
            actual.extend_from_slice(&frame);
            differences += diff;
        }
        std::fs::write(root.join("frames-rust.rgba"), &actual).unwrap();
        std::fs::write(root.join("comparison.tsv"), comparisons).unwrap();
        assert_eq!(actual.len(), expected.len());
        assert_eq!(
            differences, 0,
            "synthetic italic native glyph mask/color/position profiles"
        );
    }
    #[test]
    fn rectangular_strokes_match_native_skia() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../artifacts/cpp-reference/rect-strokes");
        let input = std::fs::read_to_string(root.join("input.tsv")).unwrap();
        let expected = std::fs::read(root.join("frames.rgba")).unwrap();
        let mut actual = Vec::new();
        let mut comparisons = String::new();
        let mut differences = 0;
        for line in input.lines() {
            let v: Vec<f64> = line
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect();
            let index = v[0] as usize;
            let mut list = PaintArtifact::default();
            if v[14] != 0.0 {
                std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
                    r#type: DisplayItemType::kClipRect,
                    rect: PaintRect {
                        x: 3.125,
                        y: 4.375,
                        width: 25.5,
                        height: 22.75,
                    },
                    antialias: true,
                    ..Default::default()
                });
            }
            if v[13] != 0.0 {
                std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
                    r#type: DisplayItemType::kSaveLayerAlpha,
                    opacity: 0.625,
                    ..Default::default()
                });
            }
            let mut transform = DisplayItem {
                r#type: DisplayItemType::kConcat,
                ..Default::default()
            };
            for (i, value) in [0, 1, 4, 5, 12, 13].into_iter().zip(&v[5..11]) {
                transform.transform.values[i] = *value;
            }
            std::sync::Arc::make_mut(&mut list.items).push(transform);
            std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
                r#type: DisplayItemType::kStrokeRect,
                rect: PaintRect {
                    x: v[1],
                    y: v[2],
                    width: v[3],
                    height: v[4],
                },
                stroke_width: v[11],
                antialias: v[12] != 0.0,
                color: Color {
                    red: v[15] as f32,
                    green: v[16] as f32,
                    blue: v[17] as f32,
                    alpha: v[18] as f32,
                },
                ..Default::default()
            });
            if v[13] != 0.0 {
                std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    ..Default::default()
                });
            }
            let frame = RasterizeDisplayItemList(&list, 40, 40);
            let native = &expected[index * 6400..(index + 1) * 6400];
            let diff = frame
                .chunks_exact(4)
                .zip(native.chunks_exact(4))
                .filter(|(a, b)| a != b)
                .count();
            comparisons.push_str(&format!("{index}\t{diff}\t1600\n"));
            differences += diff;
            actual.extend_from_slice(&frame);
        }
        std::fs::write(root.join("frames-rust.rgba"), &actual).unwrap();
        std::fs::write(root.join("comparison.tsv"), comparisons).unwrap();
        assert_eq!(actual.len(), expected.len());
        assert_eq!(
            differences, 0,
            "native rectangular strokes, coverage/AA clipping/N32/F16"
        );
    }

    #[test]
    fn save_and_clip_do_not_escape_state() {
        let red = Color {
            red: 1.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        let blue = Color {
            red: 0.0,
            green: 0.0,
            blue: 1.0,
            alpha: 1.0,
        };
        let list = PaintArtifact {
            items: vec![
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    ..DisplayItem::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kClipRect,
                    rect: PaintRect {
                        x: 1.0,
                        y: 1.0,
                        width: 2.0,
                        height: 2.0,
                    },
                    ..DisplayItem::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: 4.0,
                        height: 4.0,
                    },
                    color: red,
                    ..DisplayItem::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    ..DisplayItem::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: 1.0,
                        height: 1.0,
                    },
                    color: blue,
                    ..DisplayItem::default()
                },
            ]
            .into(),
            ..PaintArtifact::default()
        };
        let bytes = RasterizeSourceDisplayItemList(&list, 4, 4);
        assert_eq!(rgba_at(&bytes, 4, 0, 0), &[0, 0, 255, 255]);
        assert_eq!(rgba_at(&bytes, 4, 1, 1), &[255, 0, 0, 255]);
        assert_eq!(rgba_at(&bytes, 4, 3, 3), &[255, 255, 255, 255]);
    }

    #[test]
    fn rounded_clip_keeps_center_and_rejects_corner() {
        let radii = PaintCornerRadii {
            top_left: PaintCornerRadius { x: 4.0, y: 4.0 },
            top_right: PaintCornerRadius { x: 4.0, y: 4.0 },
            bottom_right: PaintCornerRadius { x: 4.0, y: 4.0 },
            bottom_left: PaintCornerRadius { x: 4.0, y: 4.0 },
        };
        let list = PaintArtifact {
            items: vec![
                DisplayItem {
                    r#type: DisplayItemType::kClipRoundedRect,
                    rect: PaintRect {
                        x: 1.0,
                        y: 1.0,
                        width: 8.0,
                        height: 8.0,
                    },
                    corner_radii: radii,
                    ..DisplayItem::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: 10.0,
                        height: 10.0,
                    },
                    color: Color {
                        red: 0.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 1.0,
                    },
                    ..DisplayItem::default()
                },
            ]
            .into(),
            ..PaintArtifact::default()
        };
        let bytes = RasterizeSourceDisplayItemList(&list, 10, 10);
        assert_eq!(rgba_at(&bytes, 10, 5, 5), &[0, 0, 0, 255]);
        assert_eq!(rgba_at(&bytes, 10, 1, 1), &[255, 255, 255, 255]);
    }

    #[test]
    fn google_layers_filters_and_masks_match_unchanged_cpp() {
        use layoutng_assembly::internal::paint_input::{PaintFilterOperation, PaintFilterType};
        let expected =
            include_bytes!("../../../artifacts/cpp-reference/google-layer-reference.rgba");
        let mut differences = Vec::new();
        for stage in 0..8 {
            let mut list = PaintArtifact::default();
            let item = |kind| DisplayItem {
                r#type: kind,
                ..Default::default()
            };
            if stage == 1 {
                std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
                    opacity: 0.63,
                    ..item(DisplayItemType::kSaveLayerAlpha)
                });
            }
            let mut save = item(if (2..=5).contains(&stage) {
                DisplayItemType::kSaveLayerFilter
            } else {
                DisplayItemType::kSaveLayer
            });
            if (2..=5).contains(&stage) {
                save.filters.push(PaintFilterOperation {
                    r#type: PaintFilterType::kBlur,
                    amount: [0.5, 1.0, 2.0, 5.0][stage - 2],
                    ..Default::default()
                });
            }
            if stage == 6 {
                save.rect = PaintRect {
                    x: 2.0,
                    y: 2.0,
                    width: 10.0,
                    height: 10.0,
                };
            }
            std::sync::Arc::make_mut(&mut list.items).push(save);
            if stage == 7 {
                std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
                    mask_layers: vec![paint::paint_engine::DisplayMaskLayer {
                        clip_rect: PaintRect {
                            x: 4.0,
                            y: 4.0,
                            width: 6.0,
                            height: 6.0,
                        },
                        ..Default::default()
                    }],
                    ..item(DisplayItemType::kBeginMask)
                });
            }
            std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
                rect: PaintRect {
                    x: 3.0,
                    y: 3.0,
                    width: 8.0,
                    height: 8.0,
                },
                color: Color {
                    red: 1.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 0.4,
                },
                ..item(DisplayItemType::kDrawRect)
            });
            std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
                rect: PaintRect {
                    x: 7.0,
                    y: 6.0,
                    width: 7.0,
                    height: 6.0,
                },
                color: Color {
                    red: 0.0,
                    green: 0.0,
                    blue: 1.0,
                    alpha: 0.6,
                },
                ..item(DisplayItemType::kDrawRect)
            });
            if stage == 7 {
                std::sync::Arc::make_mut(&mut list.items).push(item(DisplayItemType::kEndMask));
            }
            std::sync::Arc::make_mut(&mut list.items).push(item(DisplayItemType::kRestore));
            if stage == 1 {
                std::sync::Arc::make_mut(&mut list.items).push(item(DisplayItemType::kRestore));
            }
            let actual = RasterizeDisplayItemList(&list, 16, 16);
            let reference = &expected[stage * 1024..(stage + 1) * 1024];
            let count = actual.iter().zip(reference).filter(|(a, b)| a != b).count();
            if count != 0 {
                differences.push((stage, count));
            }
        }
        assert!(
            differences.is_empty(),
            "native layer differential failures: {differences:?}"
        );
    }

    #[test]
    fn inverse_path_respects_clip_and_normal_blend_layer() {
        let list = PaintArtifact {
            items: vec![
                DisplayItem {
                    r#type: DisplayItemType::kClipRect,
                    rect: PaintRect {
                        x: 2.0,
                        y: 2.0,
                        width: 12.0,
                        height: 12.0,
                    },
                    antialias: false,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerBlend,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kDrawPath,
                    path: rect_commands(PaintRect {
                        x: 5.0,
                        y: 5.0,
                        width: 6.0,
                        height: 6.0,
                    })
                    .unwrap(),
                    inverse_winding: true,
                    color: Color {
                        red: 1.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 0.5,
                    },
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    ..Default::default()
                },
            ]
            .into(),
            ..Default::default()
        };
        let bytes = RasterizeSourceDisplayItemList(&list, 16, 16);
        assert_eq!(rgba_at(&bytes, 16, 1, 8), [255, 255, 255, 255]);
        assert_eq!(rgba_at(&bytes, 16, 8, 8), [255, 255, 255, 255]);
        assert_eq!(rgba_at(&bytes, 16, 3, 8), [255, 127, 127, 255]);
    }

    #[test]
    fn inset_shadow_leaves_the_hole_and_page_backdrop_unpainted() {
        let list = PaintArtifact {
            items: vec![DisplayItem {
                r#type: DisplayItemType::kDrawBoxShadow,
                rect: PaintRect {
                    x: 3.0,
                    y: 3.0,
                    width: 10.0,
                    height: 10.0,
                },
                inset: true,
                spread: 2.0,
                color: Color {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                },
                ..Default::default()
            }]
            .into(),
            ..Default::default()
        };
        let bytes = RasterizeSourceDisplayItemList(&list, 16, 16);
        assert_eq!(rgba_at(&bytes, 16, 2, 8), [255, 255, 255, 255]);
        assert_eq!(rgba_at(&bytes, 16, 8, 8), [255, 255, 255, 255]);
        assert_eq!(rgba_at(&bytes, 16, 4, 8), [0, 0, 0, 255]);
        assert_eq!(rgba_at(&bytes, 16, 12, 8), [0, 0, 0, 255]);
    }

    #[test]
    fn alpha_layer_composites_on_restore() {
        let list = PaintArtifact {
            items: vec![
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerAlpha,
                    opacity: 0.5,
                    ..DisplayItem::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: 2.0,
                        height: 2.0,
                    },
                    color: Color {
                        red: 1.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 1.0,
                    },
                    ..DisplayItem::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    ..DisplayItem::default()
                },
            ]
            .into(),
            ..PaintArtifact::default()
        };
        let bytes = RasterizeSourceDisplayItemList(&list, 2, 2);
        let pixel = rgba_at(&bytes, 2, 0, 0);
        assert_eq!(pixel[0], 255);
        assert!((127..=129).contains(&pixel[1]));
        assert_eq!(pixel[1], pixel[2]);
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn linear_gradient_reaches_both_endpoint_colors() {
        let shader = PaintShader {
            start: Offset { x: 0.0, y: 0.0 },
            end: Offset { x: 10.0, y: 0.0 },
            stops: vec![
                PaintColorStop {
                    offset: 0.0,
                    color: Color {
                        red: 1.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 1.0,
                    },
                    ..PaintColorStop::default()
                },
                PaintColorStop {
                    offset: 1.0,
                    color: Color {
                        red: 0.0,
                        green: 0.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                    ..PaintColorStop::default()
                },
            ],
            ..PaintShader::default()
        };
        let rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 1.0,
        };
        let list = PaintArtifact {
            items: vec![DisplayItem {
                r#type: DisplayItemType::kDrawGradientRect,
                rect,
                tile_rect: rect,
                paint_shader: Some(shader),
                ..DisplayItem::default()
            }]
            .into(),
            ..PaintArtifact::default()
        };
        let bytes = RasterizeSourceDisplayItemList(&list, 10, 1);
        let first = rgba_at(&bytes, 10, 0, 0);
        let last = rgba_at(&bytes, 10, 9, 0);
        assert!(first[0] > first[2], "left endpoint must be red: {first:?}");
        assert!(last[2] > last[0], "right endpoint must be blue: {last:?}");
    }

    #[test]
    fn image_resource_draws_opaque_pixels_in_source_order() {
        let bounds = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 2.0,
            height: 1.0,
        };
        let list = PaintArtifact {
            items: vec![DisplayItem {
                r#type: DisplayItemType::kDrawImageRect,
                rect: bounds,
                source_rect: bounds,
                resource_id: 7,
                ..DisplayItem::default()
            }]
            .into(),
            resources: Some(Arc::new(PaintResources {
                images: vec![PaintImage {
                    id: 7,
                    width: 2,
                    height: 1,
                    rgba8: vec![255, 0, 0, 255, 0, 0, 255, 255].into(),
                    ..PaintImage::default()
                }],
                ..PaintResources::default()
            })),
            ..PaintArtifact::default()
        };
        let bytes = RasterizeSourceDisplayItemList(&list, 2, 1);
        assert_eq!(rgba_at(&bytes, 2, 0, 0), &[255, 0, 0, 255]);
        assert_eq!(rgba_at(&bytes, 2, 1, 0), &[0, 0, 255, 255]);
    }
}

#[cfg(all(test, feature = "source_replay"))]
#[path = "../../../artifacts/live-google-fidelity/raster-diff/path_regression.rs"]
mod google_path_regression;
