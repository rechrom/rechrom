use compositor::{
    AffineTransform, CompositorFrame, CompositorFrameMetadata, CompositorRenderPass, DeviceRect,
    DrawQuad, LayerComposition, RenderPassDrawQuad, SharedQuadState, SolidColorDrawQuad,
};
use paint::{paint_property_tree::PropertyTreeState, PaintRect};
use raster::RasterResourceProvider;
use renderer::{RasterResourceRegistry, RenderTarget, RenderUpdate, Renderer};
use std::collections::HashMap;
use std::sync::Arc;
use viz::{AggregatedFrame, RenderPassId, SurfaceId};

#[derive(Default)]
struct Resources {
    tiles: HashMap<layer_tile::TileId, raster::RasterResource>,
}
impl RasterResourceProvider for Resources {
    fn stats(&self) -> raster::RasterStats {
        Default::default()
    }
    fn prepared_frame_id(&self) -> Option<u64> {
        Some(1)
    }
    fn prepared_damage_pixels(&self) -> usize {
        0
    }
    fn resource(&self, id: layer_tile::TileId) -> Option<&raster::RasterResource> {
        self.tiles.get(&id)
    }
    fn resources(&self) -> &HashMap<layer_tile::TileId, raster::RasterResource> {
        &self.tiles
    }
    fn resident_bytes(&self) -> usize {
        0
    }
}
impl RasterResourceRegistry for Resources {
    fn ResourcesFor(&self, _: SurfaceId) -> Option<&dyn RasterResourceProvider> {
        Some(self)
    }
}
const W: u32 = 16;
const H: u32 = 12;
fn rect() -> DeviceRect {
    DeviceRect::new(0, 0, W, H)
}
fn logical() -> PaintRect {
    PaintRect {
        x: 0.,
        y: 0.,
        width: W as f64,
        height: H as f64,
    }
}
fn state() -> SharedQuadState {
    SharedQuadState {
        layer_id: layer_tile::LayerId(1),
        properties: PropertyTreeState::default(),
        composition: LayerComposition {
            transform: AffineTransform::IDENTITY,
            projective: None,
            rendering_context_id: 0,
            clip: None,
            content_bounds: None,
            opaque_bounds: None,
            white_backing: false,
            grid_min: None,
        },
        visible_polygon: None,
        retained_clips: Arc::from([]),
        is_first_layer: true,
        requires_transparent_backing: false,
        raster_content_bounds: None,
        rect_known_to_be_opaque: PaintRect::default(),
        raster_origin: (0., 0.),
        root_translation: (0., 0.),
        root_clip: None,
        compositor_scroll: None,
    }
}
fn frame(ids: &[u64], color: [u8; 4], damaged: bool) -> CompositorFrame {
    let damage = if damaged {
        rect()
    } else {
        DeviceRect::new(0, 0, 0, 0)
    };
    let mut passes = Vec::new();
    for (i, &id) in ids.iter().enumerate() {
        let root = i + 1 == ids.len();
        let mut pass = CompositorRenderPass {
            id: RenderPassId(id),
            owner_effect: (!root).then(|| PropertyTreeState::default().effect),
            output_rect: rect(),
            damage_rect: damage,
            shared_quad_state_list: Vec::new(),
            quad_list: Vec::new(),
        };
        if i == 0 {
            pass.quad_list
                .push(DrawQuad::SolidColor(SolidColorDrawQuad {
                    rect: logical(),
                    visible_rect: logical(),
                    color,
                    corner_radius: 0.,
                }));
        } else {
            pass.shared_quad_state_list.push(state());
            pass.quad_list
                .push(DrawQuad::RenderPass(RenderPassDrawQuad {
                    shared_quad_state_index: 0,
                    render_pass_id: RenderPassId(ids[i - 1]),
                }));
        }
        passes.push(pass);
    }
    CompositorFrame {
        metadata: CompositorFrameMetadata {
            frame_token: 1,
            source_frame_id: 1,
            device_scale_factor: 1.,
            raster_scale_factor: 1.,
            logical_viewport: logical(),
            viewport: rect(),
            root_damage_rect: damage,
            raster_task_count: 0,
            begin_frame_ack: None,
        },
        resource_list: Vec::new(),
        render_pass_list: passes,
    }
}
// Each accepted fixture submission has a producer sequence independent of
// raster source_frame_id (which deliberately remains 1 for impl-only frames).
fn submit(
    viz: &mut viz::VizEngine,
    surface: SurfaceId,
    mut frame: CompositorFrame,
) -> Result<(), viz::VizError> {
    let previous = viz
        .Aggregate(
            rect(),
            &[viz::SurfacePlacement {
                surface_id: surface,
                destination: rect(),
            }],
        )
        .ok()
        .and_then(|frame| frame.RenderPasses().last().map(|pass| pass.FrameToken()));
    frame.metadata.frame_token = previous.unwrap_or(0).wrapping_add(1);
    viz.SubmitFrame(surface, frame)
}
fn aggregate(viz: &viz::VizEngine, ids: &[u64]) -> AggregatedFrame {
    viz.Aggregate(
        DeviceRect::new(0, 0, W, H * ids.len() as u32),
        &ids.iter()
            .enumerate()
            .map(|(i, &id)| viz::SurfacePlacement {
                surface_id: SurfaceId(id),
                destination: DeviceRect::new(0, H as i32 * i as i32, W, H),
            })
            .collect::<Vec<_>>(),
    )
    .unwrap()
}
fn render(
    renderer: &mut Renderer,
    frame: &AggregatedFrame,
    format: skia::PixelFormat,
) -> (Vec<u32>, RenderUpdate) {
    let height = frame.OutputRect().height;
    let mut pixels = vec![0xdeadbeef; W as usize * height as usize];
    let update = renderer
        .Render(
            frame,
            &Resources::default(),
            RenderTarget {
                width: W,
                height,
                pixels: &mut pixels,
                format,
                row_stride: W as usize,
            },
        )
        .unwrap();
    (pixels, update)
}
fn check(pixels: &[u32], color: [u8; 4]) {
    assert!(
        pixels.iter().all(|p| p.to_ne_bytes() == color),
        "first pixel {:?}, expected {color:?}",
        pixels[0].to_ne_bytes()
    );
}
#[test]
fn complete_pass_graph_reconciles_churn_and_retains_undamaged_backings() {
    let mut renderer = Renderer::default();
    let mut viz = viz::VizEngine::New();
    let red = [255, 0, 0, 255];
    for child in 10..90 {
        submit(&mut viz, SurfaceId(1), frame(&[child, 1], red, true)).unwrap();
        let current = aggregate(&viz, &[1]);
        let (pixels, update) = render(&mut renderer, &current, skia::PixelFormat::Rgba8888);
        check(&pixels, red);
        assert_eq!(update.copied_bytes, W as usize * H as usize * 4);
        assert_eq!(renderer.BackingStats().passes, 2);
        assert_eq!(
            renderer.BackingStats().composed_capacity_bytes,
            2 * W as usize * H as usize * 4
        );
        let before = renderer.BackingStats();
        submit(&mut viz, SurfaceId(1), frame(&[child, 1], red, false)).unwrap();
        let (retained, update) = render(
            &mut renderer,
            &aggregate(&viz, &[1]),
            skia::PixelFormat::Rgba8888,
        );
        assert_eq!(pixels, retained);
        assert_eq!(update.damage_pixels, 0);
        assert_eq!(update.copied_bytes, W as usize * H as usize * 4);
        assert_eq!(before, renderer.BackingStats());
    }
    let held = render(
        &mut renderer,
        &aggregate(&viz, &[1]),
        skia::PixelFormat::Rgba8888,
    )
    .0;
    submit(&mut viz, SurfaceId(1), frame(&[1], [0, 0, 255, 255], true)).unwrap();
    check(
        &render(
            &mut renderer,
            &aggregate(&viz, &[1]),
            skia::PixelFormat::Rgba8888,
        )
        .0,
        [0, 0, 255, 255],
    );
    assert_eq!(renderer.BackingStats().passes, 1);
    check(&held, red);
    // A removed child can return with the same ID and no compositor damage;
    // its missing backing must be drawn, while the changed parent is damaged.
    let mut recreated = frame(&[89, 1], [0, 255, 0, 255], true);
    recreated.render_pass_list[0].damage_rect = DeviceRect::new(0, 0, 0, 0);
    submit(&mut viz, SurfaceId(1), recreated).unwrap();
    check(
        &render(
            &mut renderer,
            &aggregate(&viz, &[1]),
            skia::PixelFormat::Rgba8888,
        )
        .0,
        [0, 255, 0, 255],
    );
}
#[test]
fn nested_graph_surface_namespaces_and_explicit_invalidation_are_independent() {
    let mut renderer = Renderer::default();
    let mut viz = viz::VizEngine::New();
    for surface in [1, 2] {
        submit(
            &mut viz,
            SurfaceId(surface),
            frame(&[3, 2, 1], [255, 0, 0, 255], true),
        )
        .unwrap();
    }
    check(
        &render(
            &mut renderer,
            &aggregate(&viz, &[1, 2]),
            skia::PixelFormat::Rgba8888,
        )
        .0,
        [255, 0, 0, 255],
    );
    assert_eq!(renderer.BackingStats().passes, 6);
    submit(&mut viz, SurfaceId(1), frame(&[1], [0, 0, 255, 255], true)).unwrap();
    render(
        &mut renderer,
        &aggregate(&viz, &[1]),
        skia::PixelFormat::Rgba8888,
    );
    assert_eq!(renderer.BackingStats().passes, 4);
    submit(
        &mut viz,
        SurfaceId(2),
        frame(&[3, 2, 1], [255, 0, 0, 255], false),
    )
    .unwrap();
    let (pixels, update) = render(
        &mut renderer,
        &aggregate(&viz, &[2]),
        skia::PixelFormat::Rgba8888,
    );
    check(&pixels, [255, 0, 0, 255]);
    assert_eq!(update.damage_pixels, 0);
    assert_eq!(update.copied_bytes, W as usize * H as usize * 4);
    renderer.InvalidateSurface(SurfaceId(2));
    assert_eq!(renderer.BackingStats().passes, 1);
    let (pixels, update) = render(
        &mut renderer,
        &aggregate(&viz, &[2]),
        skia::PixelFormat::Rgba8888,
    );
    check(&pixels, [255, 0, 0, 255]);
    assert_eq!(update.damage_pixels, 3 * W as usize * H as usize);
}

#[test]
fn draw_errors_restore_child_and_root_owners_without_staging_copies() {
    let mut renderer = Renderer::default();
    let mut viz = viz::VizEngine::New();
    let red = [255, 0, 0, 255];
    submit(&mut viz, SurfaceId(1), frame(&[2, 1], red, true)).unwrap();
    let (original, _) = render(
        &mut renderer,
        &aggregate(&viz, &[1]),
        skia::PixelFormat::Rgba8888,
    );
    for broken in [0, 1] {
        let backing = renderer.BackingStats();
        // Viz accepts these graphs, but composition rejects the child damage
        // or duplicate parent quad after removing its owner for borrowing.
        let mut invalid = frame(&[2, 1], red, false);
        let expected_error = if broken == 0 {
            invalid.render_pass_list[0].damage_rect = DeviceRect::new(W as i32, 0, 1, 1);
            "damage is outside viewport"
        } else {
            let repeated = invalid.render_pass_list[1].quad_list[0].clone();
            invalid.render_pass_list[1].quad_list.push(repeated);
            "shared state mixes tile and surface quads"
        };
        submit(&mut viz, SurfaceId(1), invalid).unwrap();
        let mut output = vec![0; (W * H) as usize];
        let error = renderer
            .Render(
                &aggregate(&viz, &[1]),
                &Resources::default(),
                RenderTarget {
                    width: W,
                    height: H,
                    pixels: &mut output,
                    format: skia::PixelFormat::Rgba8888,
                    row_stride: W as usize,
                },
            )
            .unwrap_err();
        assert!(error.to_string().contains(expected_error));
        assert_eq!(renderer.BackingStats(), backing);
        // The failed candidate advanced producer history without successful
        // root consumption. Even empty damage must now redraw the whole graph;
        // resident allocations are retained, but their old pixels are stale.
        submit(
            &mut viz,
            SurfaceId(1),
            frame(&[2, 1], [0, 0, 255, 255], false),
        )
        .unwrap();
        let (retained, update) = render(
            &mut renderer,
            &aggregate(&viz, &[1]),
            skia::PixelFormat::Rgba8888,
        );
        check(&retained, [0, 0, 255, 255]);
        assert_ne!(retained, original);
        assert_eq!(update.damage_pixels, 2 * W as usize * H as usize);
        assert_eq!(update.copied_bytes, W as usize * H as usize * 4);
    }
}
#[test]
fn invalid_candidate_preserves_backings_and_new_format_cannot_reuse_old_pixels() {
    let mut renderer = Renderer::default();
    let mut viz = viz::VizEngine::New();
    submit(
        &mut viz,
        SurfaceId(1),
        frame(&[2, 1], [12, 34, 56, 255], true),
    )
    .unwrap();
    render(
        &mut renderer,
        &aggregate(&viz, &[1]),
        skia::PixelFormat::Rgba8888,
    );
    submit(
        &mut viz,
        SurfaceId(1),
        frame(&[1], [12, 34, 56, 255], false),
    )
    .unwrap();
    let candidate = aggregate(&viz, &[1]);
    assert!(renderer
        .Render(
            &candidate,
            &Resources::default(),
            RenderTarget {
                width: W,
                height: H,
                pixels: &mut [],
                format: skia::PixelFormat::Rgba8888,
                row_stride: W as usize,
            }
        )
        .is_err());
    assert_eq!(renderer.BackingStats().passes, 2);
    let mut malformed = frame(&[2, 1], [0, 0, 0, 255], true);
    malformed.render_pass_list.remove(0);
    assert!(submit(&mut viz, SurfaceId(1), malformed).is_err());
    let (pixels, update) = render(&mut renderer, &candidate, skia::PixelFormat::Bgra8888);
    check(&pixels, [56, 34, 12, 255]);
    assert_eq!(update.damage_pixels, W as usize * H as usize);
}

#[test]
fn translucent_child_does_not_reblend_outside_parent_damage() {
    use layoutng_assembly::internal::paint_input::{PaintFilterOperation, PaintFilterType};
    for blur in [false, true] {
        let fixture = |damaged| {
            let mut current = frame(&[2, 1], [255, 0, 0, 128], damaged);
            if blur {
                // The dirty output lies entirely outside this source rect.
                // Blur must still sample the full child to regenerate it.
                let source = PaintRect {
                    x: 2.,
                    y: 2.,
                    width: 4.,
                    height: 4.,
                };
                let DrawQuad::SolidColor(quad) = &mut current.render_pass_list[0].quad_list[0]
                else {
                    unreachable!()
                };
                quad.rect = source;
                quad.visible_rect = source;
                let mut effect = (*current.render_pass_list[0]
                    .owner_effect
                    .as_ref()
                    .unwrap()
                    .as_ref())
                .clone();
                effect.filters.push(PaintFilterOperation {
                    r#type: PaintFilterType::kBlur,
                    amount: 2.,
                    ..Default::default()
                });
                let effect = Arc::new(effect);
                current.render_pass_list[0].owner_effect = Some(effect.clone());
                current.render_pass_list[1].shared_quad_state_list[0]
                    .properties
                    .effect = effect;
            } else {
                // Child output has a nonzero origin even though retained
                // allocation uses the surface stride. Pixels outside it are clear.
                current.render_pass_list[0].output_rect = DeviceRect::new(2, 2, 4, 4);
                if damaged {
                    current.render_pass_list[0].damage_rect = DeviceRect::new(2, 2, 4, 4);
                }
                let composition = &mut current.render_pass_list[1].shared_quad_state_list[0].composition;
                composition.content_bounds = Some(DeviceRect::new(2, 2, 4, 4));
                composition.clip = Some(DeviceRect::new(3, 1, 2, 5));
            }
            current
        };
        let mut renderer = Renderer::default();
        let mut viz = viz::VizEngine::New();
        submit(&mut viz, SurfaceId(1), fixture(true)).unwrap();
        let (before, first_update) = render(
            &mut renderer,
            &aggregate(&viz, &[1]),
            skia::PixelFormat::Rgba8888,
        );
        if !blur {
            assert_eq!(first_update.damage_pixels, (W * H + 16) as usize);
        }
        let mut partial = fixture(false);
        if !blur {
            assert_eq!(before[0], u32::MAX, "backing extent is not visible quad coverage");
            assert_eq!(before[3 * W as usize + 2], u32::MAX, "quad clip restricts content bounds");
            assert_ne!(before[3 * W as usize + 3], u32::MAX);
        }
        let damage = if blur {
            DeviceRect::new(6, 4, 2, 2)
        } else {
            DeviceRect::new(0, 0, 2, 2)
        };
        if blur {
            assert_ne!(
                before[4 * W as usize + 6],
                u32::MAX,
                "blur has a contribution outside source bounds"
            );
        }
        partial.metadata.root_damage_rect = damage;
        partial.render_pass_list[1].damage_rect = damage;
        submit(&mut viz, SurfaceId(1), partial).unwrap();
        let (after, update) = render(
            &mut renderer,
            &aggregate(&viz, &[1]),
            skia::PixelFormat::Rgba8888,
        );
        assert_eq!(update.damage_pixels, 4);
        let changed = before.iter().zip(&after).filter(|(a, b)| a != b).count();
        assert_eq!(
            changed,
            0,
            "blur={blur}, outside pixel before {:?}, after {:?}",
            before[W as usize * 2 + 2].to_ne_bytes(),
            after[W as usize * 2 + 2].to_ne_bytes()
        );
        if !blur {
            // Shrinking/moving output must not sample old pixels left in a
            // reused backing; expanding must repaint even with empty damage.
            for output in [DeviceRect::new(4, 3, 1, 2), DeviceRect::new(2, 2, 4, 4)] {
                let mut moved = fixture(false);
                moved.render_pass_list[0].output_rect = output;
                moved.metadata.root_damage_rect = rect();
                moved.render_pass_list[1].damage_rect = rect();
                submit(&mut viz, SurfaceId(1), moved).unwrap();
                let (pixels, _) = render(&mut renderer, &aggregate(&viz, &[1]), skia::PixelFormat::Rgba8888);
                if output.width == 1 {
                    assert_eq!(pixels[3 * W as usize + 3], u32::MAX);
                    assert_ne!(pixels[3 * W as usize + 4], u32::MAX);
                } else {
                    assert_eq!(pixels, before);
                }
            }
        }
    }
}

#[test]
fn submitted_damage_requires_successful_root_consumption_continuity() {
    struct MissingResources;
    impl RasterResourceRegistry for MissingResources {
        fn ResourcesFor(&self, _: SurfaceId) -> Option<&dyn RasterResourceProvider> {
            None
        }
    }
    let candidate = |token, color, damaged| {
        let mut value = frame(&[3, 2, 1], color, damaged);
        value.metadata.frame_token = token;
        value
    };
    let draw_error = |renderer: &mut Renderer,
                      frame: &AggregatedFrame,
                      resources: &dyn RasterResourceRegistry| {
        let mut pixels = vec![0; (W * H) as usize];
        renderer
            .Render(
                frame,
                resources,
                RenderTarget {
                    width: W,
                    height: H,
                    pixels: &mut pixels,
                    format: skia::PixelFormat::Rgba8888,
                    row_stride: W as usize,
                },
            )
            .unwrap_err()
    };
    let mut renderer = Renderer::default();
    let mut viz = viz::VizEngine::New();
    let surface = SurfaceId(1);
    viz.SubmitFrame(surface, candidate(10, [255, 0, 0, 255], true))
        .unwrap();
    render(
        &mut renderer,
        &aggregate(&viz, &[1]),
        skia::PixelFormat::Rgba8888,
    );
    // Build 11 was dropped before composition; 12 describes the same new blue
    // content with no further producer damage and the same raster source ID.
    viz.SubmitFrame(surface, candidate(12, [0, 0, 255, 255], false))
        .unwrap();
    let (pixels, update) = render(
        &mut renderer,
        &aggregate(&viz, &[1]),
        skia::PixelFormat::Rgba8888,
    );
    check(&pixels, [0, 0, 255, 255]);
    assert_eq!(update.damage_pixels, 3 * (W * H) as usize);
    let mut broken = candidate(13, [0, 255, 0, 255], true);
    let root = broken.render_pass_list.last_mut().unwrap();
    root.quad_list.push(root.quad_list[0].clone());
    viz.SubmitFrame(surface, broken).unwrap();
    assert!(
        draw_error(&mut renderer, &aggregate(&viz, &[1]), &Resources::default())
            .to_string()
            .contains("shared state mixes")
    );
    // Child passes completed green before root failed. They must not survive
    // into token 14 merely because its contributing damage is empty.
    viz.SubmitFrame(surface, candidate(14, [0, 0, 255, 255], false))
        .unwrap();
    check(
        &render(
            &mut renderer,
            &aggregate(&viz, &[1]),
            skia::PixelFormat::Rgba8888,
        )
        .0,
        [0, 0, 255, 255],
    );
    viz.SubmitFrame(surface, candidate(15, [0, 0, 255, 255], false))
        .unwrap();
    assert_eq!(
        render(
            &mut renderer,
            &aggregate(&viz, &[1]),
            skia::PixelFormat::Rgba8888
        )
        .1
        .damage_pixels,
        0
    );
    viz.SubmitFrame(surface, candidate(16, [255, 255, 0, 255], true))
        .unwrap();
    assert!(
        draw_error(&mut renderer, &aggregate(&viz, &[1]), &MissingResources)
            .to_string()
            .contains("no raster resource provider")
    );
    viz.SubmitFrame(surface, candidate(17, [255, 255, 0, 255], false))
        .unwrap();
    check(
        &render(
            &mut renderer,
            &aggregate(&viz, &[1]),
            skia::PixelFormat::Rgba8888,
        )
        .0,
        [255, 255, 0, 255],
    );
    // Duplicates are conservative rather than silently accepting a reused
    // sequence for changed payload; wrapped sequences are conservative too.
    for token in [17, u64::MAX, 0] {
        viz.SubmitFrame(surface, candidate(token, [9, 8, 7, 255], false))
            .unwrap();
        let (pixels, update) = render(
            &mut renderer,
            &aggregate(&viz, &[1]),
            skia::PixelFormat::Rgba8888,
        );
        check(&pixels, [9, 8, 7, 255]);
        assert_eq!(update.damage_pixels, 3 * (W * H) as usize);
    }
    renderer.InvalidateSurface(surface);
    viz.DestroySurface(surface);
    viz.SubmitFrame(surface, candidate(1, [1, 2, 3, 255], false))
        .unwrap();
    check(
        &render(
            &mut renderer,
            &aggregate(&viz, &[1]),
            skia::PixelFormat::Rgba8888,
        )
        .0,
        [1, 2, 3, 255],
    );
}
