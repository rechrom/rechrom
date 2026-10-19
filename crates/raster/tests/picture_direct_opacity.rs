//! PictureDrawQuad must filter each primitive's paint on the active pass canvas.
use compositor_core::{
    adapter::geometry::Rect,
    runtime::{
        paint::{CpuNativePaintDiagnostics, CpuPaintOpBuffer, PaintRecord},
        paint_services::{CpuPaintFeatures, CpuPaintServices},
        skia::SkRegion,
    },
};
use layoutng_assembly::{
    fragment_tree::PaintResources,
    internal::{
        layout_input::{Offset, PaintImage},
        layout_input_types::Color,
        paint_input::{InterpolationQuality, PaintColorStop, PaintShader, PaintShaderKind},
    },
};
use paint::paint_engine::{DisplayItem, DisplayItemType, PaintArtifact, PaintRect};
use raster::native::CpuRasterProvider;
use skia::{compat::commands::ResourceContext, cpu::canvas::Canvas, raster::PixelStorage};
use std::{collections::BTreeMap, sync::Arc};

#[test]
fn picture_opacity_filters_primitives_once_including_overlap() {
    let services = CpuPaintServices::<SkRegion>::new_native(
        CpuPaintFeatures {
            cc_slimming: false,
            singular_transform_fix: true,
            merge_fixed_layers: false,
            merge_sticky_layers: false,
            raster_inducing_scroll: true,
            scrolling_contents_cull_rect: true,
            remove_scroll_node_workaround: false,
            selection_edge_visibility_uses_full_edge: true,
            canvas_draw_element: false,
            expensive_dchecks: false,
            paint_under_invalidation_checking: false,
            layer_merge_distance_limit: 0,
        },
        CpuNativePaintDiagnostics {
            trace: false,
            verbose_level: 0,
            histogram: false,
        },
    )
    .unwrap();
    let red = Color {
        red: 1.,
        green: 0.,
        blue: 0.,
        alpha: 1.,
    };
    let blue = Color {
        red: 0.,
        green: 0.,
        blue: 1.,
        alpha: 1.,
    };
    let provider = CpuRasterProvider::new(183, 1024 * 1024);
    // Each variant travels through the production PaintRecord lowering,
    // DisplayItemList RTree selection, playback folding, and Canvas receiver.
    for variant in 0..5 {
        let mut items = Vec::new();
        for (index, color) in [red, blue].into_iter().enumerate() {
            let bounds = PaintRect {
                x: index as f64,
                y: 0.,
                width: 2.,
                height: 1.,
            };
            let mut draw = DisplayItem {
                r#type: DisplayItemType::kDrawRect,
                rect: bounds,
                color,
                antialias: false,
                ..Default::default()
            };
            if variant == 1 || variant == 2 {
                draw.r#type = DisplayItemType::kDrawGradientRect;
                draw.paint_shader = Some(PaintShader {
                    kind: if variant == 1 {
                        PaintShaderKind::kLinearGradient
                    } else {
                        PaintShaderKind::kRadialGradient
                    },
                    start: Offset { x: 0., y: 0. },
                    end: Offset { x: 3., y: 0. },
                    center: Offset { x: 0., y: 0. },
                    focal: Offset { x: 0., y: 0. },
                    radius: 3.,
                    stops: vec![
                        PaintColorStop {
                            offset: 0.,
                            color,
                            ..Default::default()
                        },
                        PaintColorStop {
                            offset: 1.,
                            color,
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                });
            }
            if variant >= 3 {
                draw.r#type = if variant == 3 {
                    DisplayItemType::kDrawImageRect
                } else {
                    DisplayItemType::kDrawTiledImage
                };
                draw.resource_id = index as u64 + 1;
                draw.source_rect = PaintRect {
                    x: 0.,
                    y: 0.,
                    width: 1.,
                    height: 1.,
                };
                draw.tile_rect = bounds;
                draw.image_interpolation_quality = InterpolationQuality::kInterpolationNone;
            }
            items.push(draw);
        }
        let artifact = Arc::new(PaintArtifact {
            items: Arc::new(items),
            resources: Some(Arc::new(PaintResources {
                images: vec![
                    PaintImage {
                        id: 1,
                        paint_image_id: image_resource::PaintImageId(1),
                        compositor_metadata: Ok(static_bitmap_metadata()),
                        width: 1,
                        height: 1,
                        content: image_resource::PaintImageContent::Bitmap(
                            vec![255, 0, 0, 255].into(),
                        ),
                        ..Default::default()
                    },
                    PaintImage {
                        id: 2,
                        paint_image_id: image_resource::PaintImageId(2),
                        compositor_metadata: Ok(static_bitmap_metadata()),
                        width: 1,
                        height: 1,
                        content: image_resource::PaintImageContent::Bitmap(
                            vec![0, 0, 255, 255].into(),
                        ),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })),
            ..Default::default()
        });
        let values = services.CreatePaintOpBuffer().values().clone();
        let buffer =
            CpuPaintOpBuffer::<SkRegion>::from_native(PaintRecord::from_artifact(artifact), values);
        let mut list = services.CreateDisplayItemList();
        // Every authored item here lowers to one op. Insertion through the
        // canonical recording interface retains correct offsets/visual bounds.
        for index in 0..2 {
            list.StartPaint();
            list.push(buffer.RetainOpAt(index));
            list.EndPaintOfUnpaired(Rect::new(index as i32, 0, 2, 1));
        }
        list.Finalize();
        let target = Canvas::make_raster_direct_with_format_preserving(
            &ResourceContext::default(),
            3,
            1,
            12,
            PixelStorage::owned(vec![0; 12]),
            skia::PixelFormat::Rgba8888,
            false,
        )
        .unwrap();
        let target = provider
            .RasterizePictureInto(
                &list,
                Rect::new(0, 0, 3, 1),
                1.,
                true,
                &BTreeMap::new(),
                &BTreeMap::new(),
                0.5,
                target,
            )
            .unwrap();
        let pixels = target.finish_direct().into_vec();
        // Source per-primitive SrcOver: red .5, then blue .5 => alpha .75.
        // Flatten-then-opacity would produce blue alpha .5 in the overlap.
        let overlap = &pixels[4..8];
        for (actual, expected) in overlap.iter().zip([64, 0, 128, 192]) {
            assert!(
                (*actual as i32 - expected).abs() <= 1,
                "variant {variant}: {overlap:?}"
            );
        }
        assert!(
            (pixels[3] as i32 - 128).abs() <= 1,
            "variant {variant}: {pixels:?}"
        );
    }
}

fn static_bitmap_metadata() -> image_resource::PaintImageMetadata {
    image_resource::PaintImageMetadata {
        animation: image_resource::PaintImageAnimationMetadata::Known(
            image_resource::PaintImageAnimationInfo::confirmed_static(),
        ),
        completion_state: image_resource::PaintImageCompletionState::Done,
        is_multipart: false,
    }
}
