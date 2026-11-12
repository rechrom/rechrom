use skia::{
    include::{
        core::{
            SkClipOp::SkClipOp,
            SkPathTypes::SkPathDirection,
            SkSamplingOptions::{SkFilterMode, SkSamplingOptions},
            SkSurfaceProps::SkSurfaceProps,
            SkTileMode::SkTileMode,
        },
        effects::SkGradient,
    },
    raster_target::{N32BlitMode, PixelFormat, RasterTarget},
    src::core::{
        SkBlendMode::SkBlendMode,
        SkColor::SkColor4f,
        SkColorSpace::SkColorSpace,
        SkImageInfo::SkImageInfo,
        SkMatrix::SkMatrix,
        SkPaint::{SkPaint, Style},
        SkPathBuilder::{ArcSize, SkPathBuilder},
        SkPixmap::SkPixmap,
        SkPoint::SkPoint,
        SkRRect::SkRRect,
        SkRect::SkRect,
    },
    src::image::SkImage_RasterFactories::RasterFromPixmapCopy,
};

fn github_card_border_corner() -> ([u8; 4], [u8; 4], [u8; 4]) {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 256;
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    let props = SkSurfaceProps::default();
    let mut target = RasterTarget::new_with_color_space(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    target.translate(-508., -3066.);
    let boundary = SkRRect::MakeRectXY(&SkRect::MakeLTRB(211.5, 2886.5, 557.5, 3111.5), 7.5, 7.5);
    let mut fill = paint(
        SkColor4f {
            fR: 1.,
            fG: 1.,
            fB: 1.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    fill.setAntiAlias(true);
    unsafe { target.draw_rrect(&boundary, &fill).unwrap() };
    target.finish().unwrap();
    let at = |x: usize, y: usize| (y * WIDTH + x) * 4;
    let before = pixels[at(47, 43)..at(47, 43) + 4].try_into().unwrap();

    let mut target = RasterTarget::new_with_color_space(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    target.translate(-508., -3066.);

    let mut border = paint(
        SkColor4f {
            fR: 210. / 255.,
            fG: 217. / 255.,
            fB: 212. / 255.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    border.setAntiAlias(true);
    border.setStyle(Style::kStroke_Style);
    border.setStrokeWidth(1.);
    if std::env::var_os("RECHROM_DUMP_GITHUB_CARD_PATH").is_some() {
        let mut source_builder = SkPathBuilder::new();
        source_builder.addRRectDefault(&boundary, SkPathDirection::kCW);
        let source_path = source_builder.detach(None);
        let ctm = SkMatrix::Translate(-508., -3066.);
        let mut stroke_builder = SkPathBuilder::new();
        skia::src::core::SkPathUtils::FillPathWithPaint(
            &source_path,
            &border,
            &mut stroke_builder,
            None,
            &ctm,
        );
        stroke_builder.transform(&ctm);
        let stroke_path = stroke_builder.detach(None);
        eprintln!("rust-stroked-path:");
        stroke_path.dumpHex();
    }
    unsafe { target.draw_rrect(&boundary, &border).unwrap() };
    target.finish().unwrap();
    let after = pixels[at(47, 43)..at(47, 43) + 4].try_into().unwrap();
    pixels.fill(0);
    let mut target = RasterTarget::new_with_color_space(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    target.translate(-508., -3066.);
    unsafe { target.draw_rrect(&boundary, &border).unwrap() };
    target.finish().unwrap();
    let border_only = pixels[at(47, 43)..at(47, 43) + 4].try_into().unwrap();
    (before, after, border_only)
}

#[test]
fn github_card_border_corner_records_coverage_regression() {
    assert_eq!(
        github_card_border_corner(),
        ([32, 32, 32, 32], [93, 95, 93, 108], [72, 74, 72, 87])
    );
}

#[test]
fn github_card_top_border_corner_records_coverage_regression() {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 256;
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    let props = SkSurfaceProps::default();
    let boundary = SkRRect::MakeRectXY(&SkRect::MakeLTRB(211.5, 2886.5, 557.5, 3111.5), 7.5, 7.5);
    let mut fill = paint(
        SkColor4f {
            fR: 1.,
            fG: 1.,
            fB: 1.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    fill.setAntiAlias(true);
    let mut border = paint(
        SkColor4f {
            fR: 210. / 255.,
            fG: 217. / 255.,
            fB: 212. / 255.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    border.setAntiAlias(true);
    border.setStyle(Style::kStroke_Style);
    border.setStrokeWidth(1.);
    let at = |x: usize, y: usize| (y * WIDTH + x) * 4;

    let mut target = RasterTarget::new_with_color_space(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    target.translate(-508., -2812.);
    unsafe { target.draw_rrect(&boundary, &fill).unwrap() };
    target.finish().unwrap();
    let before: [u8; 4] = pixels[at(47, 76)..at(47, 76) + 4].try_into().unwrap();

    let mut target = RasterTarget::new_with_color_space(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    target.translate(-508., -2812.);
    unsafe { target.draw_rrect(&boundary, &border).unwrap() };
    target.finish().unwrap();
    let after: [u8; 4] = pixels[at(47, 76)..at(47, 76) + 4].try_into().unwrap();

    assert_eq!((before, after), ([35, 35, 35, 35], [99, 101, 98, 114]));
}

#[test]
fn github_recorded_bounds_tile_preserves_picture_aa_phase() {
    const WIDTH: usize = 448;
    const HEIGHT: usize = 320;
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    let props = SkSurfaceProps::default();
    let mut target = RasterTarget::new_with_color_space(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    target.set_raster_phase_origin(128, 2304);
    target.translate(-128., -2304.);
    let boundary = SkRRect::MakeRectXY(&SkRect::MakeLTRB(211.5, 2360.5, 557.5, 2585.5), 7.5, 7.5);
    let mut fill = paint(
        SkColor4f {
            fR: 1.,
            fG: 1.,
            fB: 1.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    fill.setAntiAlias(true);
    unsafe { target.draw_rrect(&boundary, &fill).unwrap() };
    target.finish().unwrap();
    let at = |x: usize, y: usize| (y * WIDTH + x) * 4;
    let before: [u8; 4] = pixels[at(427, 279)..at(427, 279) + 4].try_into().unwrap();

    let mut target = RasterTarget::new_with_color_space(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    target.set_raster_phase_origin(128, 2304);
    target.translate(-128., -2304.);
    let mut border = paint(
        SkColor4f {
            fR: 210. / 255.,
            fG: 217. / 255.,
            fB: 212. / 255.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    border.setAntiAlias(true);
    border.setStyle(Style::kStroke_Style);
    border.setStrokeWidth(1.);
    unsafe { target.draw_rrect(&boundary, &border).unwrap() };
    target.finish().unwrap();
    let after: [u8; 4] = pixels[at(427, 279)..at(427, 279) + 4].try_into().unwrap();
    assert_eq!((before, after), ([31, 31, 31, 31], [93, 94, 92, 107]));
}

#[test]
fn save_layer_preserves_parent_picture_aa_phase() {
    const WIDTH: usize = 448;
    const HEIGHT: usize = 320;
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    let props = SkSurfaceProps::default();
    let mut target = RasterTarget::new_with_color_space(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    target.set_raster_phase_origin(128, 2304);
    target.translate(-128., -2304.);
    let bounds = SkRect::MakeLTRB(211.5, 2360.5, 557.5, 2585.5);
    let boundary = SkRRect::MakeRectXY(&bounds, 7.5, 7.5);
    target.save_layer(Some(&bounds), None).unwrap();
    let mut fill = paint(
        SkColor4f {
            fR: 1.,
            fG: 1.,
            fB: 1.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    fill.setAntiAlias(true);
    unsafe { target.draw_rrect(&boundary, &fill).unwrap() };
    target.restore().unwrap();
    target.finish().unwrap();
    let start = (279 * WIDTH + 427) * 4;
    assert_eq!(&pixels[start..start + 4], &[31, 31, 31, 31]);
}

fn github_y2600_partial_tile() -> Vec<u8> {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 256;
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    let props = SkSurfaceProps::default();
    let mut target = RasterTarget::new_with_color_space(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    // The tile starts at y=2540, then its containing paint record contributes
    // another -526 translation. Device pixel (47,43) therefore represents
    // picture coordinate (555,3109), not layer coordinate (555,2583).
    target.set_raster_phase_origin(508, 2540);
    target.translate(-508., -3066.);
    let boundary = SkRRect::MakeRectXY(&SkRect::MakeLTRB(211.5, 2886.5, 557.5, 3111.5), 7.5, 7.5);
    let mut fill = paint(
        SkColor4f {
            fR: 1.,
            fG: 1.,
            fB: 1.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    fill.setAntiAlias(true);
    unsafe { target.draw_rrect(&boundary, &fill).unwrap() };
    let mut border = paint(
        SkColor4f {
            fR: 210. / 255.,
            fG: 217. / 255.,
            fB: 212. / 255.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    border.setAntiAlias(true);
    border.setStyle(Style::kStroke_Style);
    border.setStrokeWidth(1.);
    unsafe { target.draw_rrect(&boundary, &border).unwrap() };
    target.finish().unwrap();
    pixels
}

#[test]
fn picture_aa_phase_matches_chrome_clipped_tile_corner() {
    const WIDTH: usize = 256;
    let pixels = github_y2600_partial_tile();
    let start = (43 * WIDTH + 47) * 4;
    // Chrome's actual partial tile is straight RGBA (222,226,224,115), i.e.
    // premultiplied BGRA [101,102,100,115] at this picture pixel. Full-layer
    // replay uses a different unclipped scan path and is not the tile oracle.
    assert_eq!(&pixels[start..start + 4], &[101, 102, 100, 115]);
}

#[test]
fn github_y2600_partial_tile_matches_chrome_after_strict_subset_composite() {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 256;
    let source = github_y2600_partial_tile();
    let info = SkImageInfo::MakeN32Premul(WIDTH as i32, HEIGHT as i32);
    let pixmap = SkPixmap::new(&info, source.as_ptr(), WIDTH * 4);
    let image = unsafe { RasterFromPixmapCopy(&pixmap) }.expect("raster tile image");
    let props = SkSurfaceProps::default();
    let mut composited = vec![255; WIDTH * HEIGHT * 4];
    let mut target = RasterTarget::new(
        &mut composited,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(&props),
    )
    .unwrap();
    let mut image_paint = paint(
        SkColor4f {
            fR: 0.,
            fG: 0.,
            fB: 0.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    image_paint.setAntiAlias(false);
    unsafe {
        target
            .draw_image_rect(
                &image,
                Some(&SkRect::MakeLTRB(1., 0., 255., 255.)),
                &SkRect::MakeLTRB(0., 0., 254., 255.),
                &SkSamplingOptions::with_filter(SkFilterMode::kLinear),
                Some(&image_paint),
                skia::src::core::SkCanvas::SrcRectConstraint::kStrict_SrcRectConstraint,
            )
            .unwrap();
    }
    target.finish().unwrap();
    let offset = (43 * WIDTH + 46) * 4;
    assert_eq!(
        &composited[offset..offset + 4],
        &[241, 242, 240, 255],
        "the former y=2600 one-pixel delta must match Chrome after final composition"
    );
}

#[test]
fn github_y800_partial_tile_matches_chrome_bottom_right_corner() {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 256;
    const TILE_X: i32 = 1016;
    const TILE_Y: i32 = 762;
    const RECORD_Y: i32 = 526;
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    let props = SkSurfaceProps::default();
    let mut target = RasterTarget::new_with_color_space(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    target.set_raster_phase_origin(TILE_X, TILE_Y);
    target.translate(-(TILE_X as f32), -(TILE_Y as f32));
    target.clip_rect(
        &SkRect::MakeLTRB(
            TILE_X as f32,
            TILE_Y as f32,
            (TILE_X + WIDTH as i32) as f32,
            (TILE_Y + HEIGHT as i32) as f32,
        ),
        SkClipOp::kIntersect,
        false,
    );
    target.save();
    // Layer 13 begins its recorded picture with translate(0, -526). Keep this
    // separate from the tile translation so the backing-origin residual is
    // identical to Chrome's PaintOp playback stack.
    target.translate(0., -(RECORD_Y as f32));
    target.save();
    target.clip_rect(
        &SkRect::MakeLTRB(0., 530., 1280., 1427.),
        SkClipOp::kIntersect,
        true,
    );

    let mut green = paint(
        SkColor4f {
            fR: 8. / 255.,
            fG: 135. / 255.,
            fB: 43. / 255.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    green.setAntiAlias(true);
    unsafe {
        target
            .draw_rrect(
                &SkRRect::MakeRectXY(&SkRect::MakeLTRB(-4., 598., 1136., 1367.), 12., 12.),
                &green,
            )
            .unwrap()
    };

    let boundary = SkRRect::MakeRectXY(&SkRect::MakeLTRB(0.5, 594.5, 1139.5, 1362.5), 7.5, 7.5);
    let mut fill = paint(
        SkColor4f {
            fR: 1.,
            fG: 1.,
            fB: 1.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    fill.setAntiAlias(true);
    unsafe { target.draw_rrect(&boundary, &fill).unwrap() };
    let mut border = paint(
        SkColor4f {
            fR: 210. / 255.,
            fG: 217. / 255.,
            fB: 212. / 255.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    border.setAntiAlias(true);
    border.setStyle(Style::kStroke_Style);
    border.setStrokeWidth(1.);
    unsafe { target.draw_rrect(&boundary, &border).unwrap() };
    target.restore().unwrap();
    target.restore().unwrap();
    target.finish().unwrap();

    // Chrome rasterizes the layer into transparent premultiplied N32, then the
    // software compositor applies SrcOver onto the opaque white root surface.
    let mut composited = vec![255; WIDTH * HEIGHT * 4];
    let mut destination = RasterTarget::new_with_color_space(
        &mut composited,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    assert!(destination.blit_n32(
        &pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        0,
        0,
        0,
        0,
        WIDTH as i32,
        HEIGHT as i32,
        N32BlitMode::SrcOver,
        255,
        false,
    ));
    destination.finish().unwrap();

    let at = |layer_x: usize, layer_y: usize| {
        ((layer_y - TILE_Y as usize) * WIDTH + layer_x - TILE_X as usize) * 4
    };
    let actual: Vec<[u8; 4]> = [(1137, 834), (1133, 836), (1132, 837)]
        .into_iter()
        .map(|(x, y)| {
            let start = at(x, y);
            composited[start..start + 4].try_into().unwrap()
        })
        .collect();
    let chrome = vec![
        [241, 242, 240, 255],
        [207, 218, 203, 255],
        [148, 195, 131, 255],
    ];
    assert_eq!(actual, chrome, "composited partial tile must match Chrome");
}

#[test]
fn strict_integer_tile_subset_matches_chrome_source_over_rounding() {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 256;
    const SOURCE_X: usize = 47;
    const SOURCE_Y: usize = 43;
    let mut source = vec![0u8; WIDTH * HEIGHT * 4];
    let source_offset = (SOURCE_Y * WIDTH + SOURCE_X) * 4;
    // Chrome's captured premultiplied BGRA pixel at the rounded card edge.
    source[source_offset..source_offset + 4].copy_from_slice(&[101, 102, 100, 115]);
    // Production software resources and the root RasterTarget are both
    // untagged (color space None); keep this test on that exact path.
    let info = SkImageInfo::MakeN32Premul(WIDTH as i32, HEIGHT as i32);
    let pixmap = SkPixmap::new(&info, source.as_ptr(), WIDTH * 4);
    let image = unsafe { RasterFromPixmapCopy(&pixmap) }.expect("raster tile image");

    let mut destination = vec![255u8; WIDTH * HEIGHT * 4];
    let props = SkSurfaceProps::default();
    let mut target = RasterTarget::new(
        &mut destination,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(&props),
    )
    .unwrap();
    let mut image_paint = paint(
        SkColor4f {
            fR: 0.,
            fG: 0.,
            fB: 0.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    image_paint.setAntiAlias(false);
    unsafe {
        target
            .draw_image_rect(
                &image,
                Some(&SkRect::MakeLTRB(1., 0., 255., 255.)),
                &SkRect::MakeLTRB(0., 0., 254., 255.),
                &SkSamplingOptions::with_filter(SkFilterMode::kLinear),
                Some(&image_paint),
                skia::src::core::SkCanvas::SrcRectConstraint::kStrict_SrcRectConstraint,
            )
            .unwrap();
    }
    target.finish().unwrap();

    let destination_offset = (SOURCE_Y * WIDTH + SOURCE_X - 1) * 4;
    assert_eq!(
        &destination[destination_offset..destination_offset + 4],
        &[241, 242, 240, 255],
        "integer tile subsets must use Chrome's exact SrcOver rounding"
    );
}

#[test]
fn integer_rrect_keeps_tile_local_aa_coordinates() {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 256;
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    let props = SkSurfaceProps::default();
    let mut target = RasterTarget::new_with_color_space(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    target.translate(-1016., -3066.);
    let boundary = SkRRect::MakeRectXY(&SkRect::MakeLTRB(719., 2890., 1065., 3115.), 12., 12.);
    let mut fill = paint(
        SkColor4f {
            fR: 95. / 255.,
            fG: 185. / 255.,
            fB: 1.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    fill.setAntiAlias(true);
    unsafe { target.draw_rrect(&boundary, &fill).unwrap() };
    target.finish().unwrap();
    let start = (45 * WIDTH + 45) * 4;
    // Chrome's transparent layer is straight RGBA (94,185,255,128), i.e.
    // premultiplied BGRA [128,93,47,128] at this exact picture pixel.
    assert_eq!(&pixels[start..start + 4], &[128, 93, 47, 128]);
}

fn canvas_rrect_border_corner(
    color: SkColor4f,
    translate_x: i32,
    explicit_zero_phase: bool,
) -> [u8; 4] {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 64;
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    let props = SkSurfaceProps::default();
    let mut target = RasterTarget::new_with_color_space(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    if explicit_zero_phase {
        target.set_raster_phase_origin(0, 0);
    }
    // Match the direct Canvas2D roundRect probe exactly. The one-pixel
    // translation maps the inspected corner to device coordinate (3, 2).
    target.translate(translate_x as f32, 0.);
    if explicit_zero_phase {
        // Production raster playback clips the authored playback rect after
        // installing the compact-backing translation. This clip is part of
        // hairline edge decomposition and must be present in the phase oracle.
        target.clip_rect(
            &SkRect::MakeLTRB(0., 0., 201., 36.),
            SkClipOp::kIntersect,
            false,
        );
    }
    let boundary = SkRRect::MakeRectXY(&SkRect::MakeLTRB(0.5, 0.5, 199.5, 35.5), 5.5, 5.5);
    let mut border = paint(color, SkBlendMode::kSrcOver);
    border.setAntiAlias(true);
    border.setStyle(Style::kStroke_Style);
    border.setStrokeWidth(1.);
    unsafe { target.draw_rrect(&boundary, &border).unwrap() };
    target.finish().unwrap();
    let start = (2 * WIDTH + 2 + translate_x as usize) * 4;
    pixels[start..start + 4].try_into().unwrap()
}

#[test]
fn canvas_rrect_border_corner_matches_chrome_direct_raster() {
    // Chrome Canvas2D is straight RGBA (181,191,183,128) here, corresponding
    // exactly to premultiplied BGRA [92,96,91,128]. CSS border painting is a
    // separate geometry/recording path and must not be used as a Skia oracle.
    assert_eq!(
        canvas_rrect_border_corner(
            SkColor4f {
                fR: 182. / 255.,
                fG: 191. / 255.,
                fB: 184. / 255.,
                fA: 1.,
            },
            1,
            false,
        ),
        [92, 96, 91, 128]
    );
}

#[test]
fn untranslated_rrect_border_corner_matches_css_layer_raster() {
    assert_eq!(
        canvas_rrect_border_corner(
            SkColor4f {
                fR: 182. / 255.,
                fG: 191. / 255.,
                fB: 184. / 255.,
                fA: 1.,
            },
            0,
            false,
        ),
        [93, 97, 91, 129]
    );
}

#[test]
fn explicit_zero_picture_origin_preserves_css_layer_aa_phase() {
    // Chrome's layer-14 records the same RRect at x=0.5 and produces straight
    // RGBA (180,192,184,129), i.e. premultiplied BGRA [93,97,91,129]. During
    // compact backing replay Rechrom adds a 1px CTM translation; an explicitly
    // set (0,0) picture origin must still cancel that translation for AA phase.
    assert_eq!(
        canvas_rrect_border_corner(
            SkColor4f {
                fR: 182. / 255.,
                fG: 191. / 255.,
                fB: 184. / 255.,
                fA: 1.,
            },
            1,
            true,
        ),
        [93, 97, 91, 129]
    );
}

fn translated_svg_arrow_pixels(explicit_zero_origin: bool) -> Vec<u8> {
    const WIDTH: usize = 512;
    const HEIGHT: usize = 128;
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    let props = SkSurfaceProps::default();
    let mut target = RasterTarget::new(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(&props),
    )
    .unwrap();
    if explicit_zero_origin {
        target.set_raster_phase_origin(0, 0);
    }
    target.set_matrix(&SkMatrix::MakeAll(0., -1., 152., 1., 0., 26., 0., 0., 1.));

    let mut arrow = SkPathBuilder::new();
    arrow.moveToXY(6.427, 4.427);
    arrow.lineToXY(9.823, 7.823);
    arrow.arcToSvg(
        SkPoint::make(0.25, 0.25),
        0.,
        ArcSize::kSmall_ArcSize,
        SkPathDirection::kCW,
        SkPoint::make(9.823, 8.177),
    );
    arrow.lineToXY(6.427, 11.573);
    arrow.arcToSvg(
        SkPoint::make(0.25, 0.25),
        0.,
        ArcSize::kSmall_ArcSize,
        SkPathDirection::kCW,
        SkPoint::make(6., 11.396),
    );
    arrow.lineToXY(6., 4.604);
    arrow.arcToSvg(
        SkPoint::make(0.25, 0.25),
        0.,
        ArcSize::kSmall_ArcSize,
        SkPathDirection::kCW,
        SkPoint::make(6.427, 4.427),
    );
    arrow.close();
    let arrow = arrow.detach(None);
    let mut fill = paint(
        SkColor4f {
            fR: 88. / 255.,
            fG: 99. / 255.,
            fB: 91. / 255.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    fill.setAntiAlias(true);
    unsafe { target.draw_path(&arrow, &fill).unwrap() };
    target.finish().unwrap();
    pixels
}

#[test]
fn explicit_zero_picture_origin_does_not_cancel_authored_path_transform() {
    assert_eq!(
        translated_svg_arrow_pixels(true),
        translated_svg_arrow_pixels(false)
    );
}

fn css_layer_border_pixels(phase_replay: bool) -> Vec<u8> {
    const WIDTH: usize = 202;
    const HEIGHT: usize = 36;
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    let props = SkSurfaceProps::default();
    let mut target = RasterTarget::new_with_color_space(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    if phase_replay {
        target.set_raster_phase_origin(0, 0);
        target.translate(1., 0.);
        target.clip_rect(
            &SkRect::MakeLTRB(0., 0., 201., 36.),
            SkClipOp::kIntersect,
            false,
        );
    }
    let boundary = SkRRect::MakeRectXY(&SkRect::MakeLTRB(0.5, 0.5, 199.5, 35.5), 5.5, 5.5);
    let mut border = paint(
        SkColor4f {
            fR: 182. / 255.,
            fG: 191. / 255.,
            fB: 184. / 255.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    border.setAntiAlias(true);
    border.setStyle(Style::kStroke_Style);
    border.setStrokeWidth(1.);
    unsafe { target.draw_rrect(&boundary, &border).unwrap() };
    target.finish().unwrap();
    pixels
}

#[test]
fn compact_backing_phase_replay_matches_full_css_layer_border() {
    const WIDTH: usize = 202;
    const HEIGHT: usize = 36;
    let direct = css_layer_border_pixels(false);
    let phased = css_layer_border_pixels(true);
    for y in 0..HEIGHT {
        for x in 0..201 {
            let direct_at = (y * WIDTH + x) * 4;
            let phased_at = (y * WIDTH + x + 1) * 4;
            assert_eq!(
                &phased[phased_at..phased_at + 4],
                &direct[direct_at..direct_at + 4],
                "phase mismatch at CSS-layer pixel ({x}, {y})"
            );
        }
    }
}

#[test]
fn github_search_compact_tile_matches_all_observed_chrome_edge_pixels() {
    const WIDTH: usize = 202;
    const HEIGHT: usize = 36;
    let source = css_layer_border_pixels(true);
    let props = SkSurfaceProps::default();
    let mut composited = vec![255; WIDTH * HEIGHT * 4];
    let mut destination = RasterTarget::new(
        &mut composited,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(&props),
    )
    .unwrap();
    assert!(destination.blit_n32(
        &source,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        1,
        0,
        0,
        0,
        201,
        HEIGHT as i32,
        N32BlitMode::SrcOver,
        255,
        false,
    ));
    destination.finish().unwrap();

    // Chrome's 14 non-identical y=0 pixels were all on this 201x36 search
    // border. These are the exact final premultiplied BGRA values after its
    // compact backing (one transparent inset pixel) is composited onto white.
    let expected = [
        ((2, 2), [219, 223, 217, 255]),
        ((0, 3), [231, 234, 230, 255]),
        ((1, 3), [212, 216, 211, 255]),
        ((1, 4), [236, 237, 235, 255]),
        ((1, 5), [249, 249, 249, 255]),
        ((1, 30), [249, 249, 249, 255]),
        ((1, 31), [235, 237, 235, 255]),
        ((0, 32), [231, 234, 231, 255]),
        ((197, 33), [217, 222, 216, 255]),
        ((3, 34), [212, 216, 210, 255]),
        ((5, 34), [248, 249, 248, 255]),
        ((194, 34), [248, 249, 248, 255]),
        ((196, 34), [211, 215, 209, 255]),
        ((3, 35), [231, 233, 231, 255]),
    ];
    for ((x, y), pixel) in expected {
        let offset = (y * WIDTH + x) * 4;
        assert_eq!(
            &composited[offset..offset + 4],
            &pixel,
            "Chrome search-border mismatch at layer pixel ({x}, {y})"
        );
    }
}

fn draw_scroll_phase_card(target: &mut RasterTarget<'_>, origin_x: i32, origin_y: i32) {
    let boundary = SkRRect::MakeRectXY(
        &SkRect::MakeLTRB(
            origin_x as f32 + 8.5,
            origin_y as f32 + 8.5,
            origin_x as f32 + 87.5,
            origin_y as f32 + 55.5,
        ),
        7.5,
        7.5,
    );
    let mut fill = paint(
        SkColor4f {
            fR: 1.,
            fG: 1.,
            fB: 1.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    fill.setAntiAlias(true);
    let mut border = paint(
        SkColor4f {
            fR: 210. / 255.,
            fG: 217. / 255.,
            fB: 212. / 255.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    border.setAntiAlias(true);
    border.setStyle(Style::kStroke_Style);
    border.setStrokeWidth(1.);
    unsafe {
        target.draw_rrect(&boundary, &fill).unwrap();
        target.draw_rrect(&boundary, &border).unwrap();
    }
}

#[test]
fn compact_tiles_match_full_picture_across_scroll_phase_matrix() {
    const TILE_WIDTH: usize = 96;
    const TILE_HEIGHT: usize = 64;
    let props = SkSurfaceProps::default();
    // Include both sides of 256px tile boundaries and the two recorded GitHub
    // origins. The last two rows cover deep-page scroll positions without
    // assuming that the first viewport's AA phase repeats there.
    for (origin_x, origin_y) in [
        (0, 0),
        (1, 1),
        (127, 255),
        (128, 256),
        (508, 2540),
        (1016, 762),
        (128, 5200),
        (128, 6700),
    ] {
        let full_width = origin_x as usize + TILE_WIDTH;
        let full_height = origin_y as usize + TILE_HEIGHT;
        let mut full = vec![0; full_width * full_height * 4];
        let mut full_target = RasterTarget::new_with_color_space(
            &mut full,
            full_width as i32,
            full_height as i32,
            full_width * 4,
            PixelFormat::Bgra8888,
            Some(SkColorSpace::MakeSRGB()),
            Some(&props),
        )
        .unwrap();
        draw_scroll_phase_card(&mut full_target, origin_x, origin_y);
        full_target.finish().unwrap();

        let mut compact = vec![0; TILE_WIDTH * TILE_HEIGHT * 4];
        let mut compact_target = RasterTarget::new_with_color_space(
            &mut compact,
            TILE_WIDTH as i32,
            TILE_HEIGHT as i32,
            TILE_WIDTH * 4,
            PixelFormat::Bgra8888,
            Some(SkColorSpace::MakeSRGB()),
            Some(&props),
        )
        .unwrap();
        compact_target.set_raster_phase_origin(origin_x, origin_y);
        compact_target.translate(-(origin_x as f32), -(origin_y as f32));
        draw_scroll_phase_card(&mut compact_target, origin_x, origin_y);
        compact_target.finish().unwrap();

        for y in 0..TILE_HEIGHT {
            let full_start = ((origin_y as usize + y) * full_width + origin_x as usize) * 4;
            let compact_start = y * TILE_WIDTH * 4;
            assert_eq!(
                &compact[compact_start..compact_start + TILE_WIDTH * 4],
                &full[full_start..full_start + TILE_WIDTH * 4],
                "compact/full AA mismatch at picture origin ({origin_x}, {origin_y}), row {y}"
            );
        }
    }
}

#[test]
fn compact_tiles_match_full_picture_at_every_github_scroll_origin() {
    const ORIGIN_X: i32 = 128;
    const MAX_SCROLL_Y: i32 = 7047;
    const TILE_WIDTH: usize = 96;
    const TILE_HEIGHT: usize = 64;
    const FULL_WIDTH: usize = ORIGIN_X as usize + TILE_WIDTH;
    const FULL_HEIGHT: usize = MAX_SCROLL_Y as usize + TILE_HEIGHT;
    let props = SkSurfaceProps::default();
    let info = SkImageInfo::MakeN32Premul(TILE_WIDTH as i32, TILE_HEIGHT as i32);

    let mut full_pixels = vec![0; FULL_WIDTH * FULL_HEIGHT * 4];
    let mut full = RasterTarget::new_with_color_space(
        &mut full_pixels,
        FULL_WIDTH as i32,
        FULL_HEIGHT as i32,
        FULL_WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    let mut compact_pixels = vec![0; TILE_WIDTH * TILE_HEIGHT * 4];
    let mut compact = RasterTarget::new_with_color_space(
        &mut compact_pixels,
        TILE_WIDTH as i32,
        TILE_HEIGHT as i32,
        TILE_WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    let mut full_tile = vec![0; TILE_WIDTH * TILE_HEIGHT * 4];
    let mut compact_tile = vec![0; TILE_WIDTH * TILE_HEIGHT * 4];

    for origin_y in 0..=MAX_SCROLL_Y {
        full.set_matrix(SkMatrix::I());
        let save = full.save();
        full.clip_rect(
            &SkRect::MakeXYWH(
                ORIGIN_X as f32,
                origin_y as f32,
                TILE_WIDTH as f32,
                TILE_HEIGHT as f32,
            ),
            SkClipOp::kIntersect,
            false,
        );
        full.clear(0).unwrap();
        full.restore_to_count(save).unwrap();
        draw_scroll_phase_card(&mut full, ORIGIN_X, origin_y);
        assert!(full
            .read_pixels(&info, &mut full_tile, TILE_WIDTH * 4, ORIGIN_X, origin_y,)
            .unwrap());

        compact.set_matrix(SkMatrix::I());
        compact.set_raster_phase_origin(ORIGIN_X, origin_y);
        compact.clear(0).unwrap();
        compact.translate(-(ORIGIN_X as f32), -(origin_y as f32));
        draw_scroll_phase_card(&mut compact, ORIGIN_X, origin_y);
        assert!(compact
            .read_pixels(&info, &mut compact_tile, TILE_WIDTH * 4, 0, 0)
            .unwrap());

        assert_eq!(
            compact_tile, full_tile,
            "compact/full AA mismatch at GitHub scroll origin y={origin_y}"
        );
    }

    compact.finish().unwrap();
    full.finish().unwrap();
}

fn translated_rrect_border_pixels(use_layer: bool, explicit_zero_phase: bool) -> Vec<u8> {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 64;
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    let props = SkSurfaceProps::default();
    let mut target = RasterTarget::new_with_color_space(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        Some(SkColorSpace::MakeSRGB()),
        Some(&props),
    )
    .unwrap();
    if explicit_zero_phase {
        target.set_raster_phase_origin(0, 0);
    }
    target.translate(1., 0.);
    if use_layer {
        target.save_layer(None, None).unwrap();
    }
    let boundary = SkRRect::MakeRectXY(&SkRect::MakeLTRB(0.5, 0.5, 199.5, 35.5), 5.5, 5.5);
    let mut border = paint(
        SkColor4f {
            fR: 182. / 255.,
            fG: 191. / 255.,
            fB: 184. / 255.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    border.setAntiAlias(true);
    border.setStyle(Style::kStroke_Style);
    border.setStrokeWidth(1.);
    unsafe { target.draw_rrect(&boundary, &border).unwrap() };
    if use_layer {
        target.restore().unwrap();
    }
    target.finish().unwrap();
    pixels
}

#[test]
fn zero_origin_save_layer_preserves_translated_hairline_raster() {
    assert_eq!(
        translated_rrect_border_pixels(true, false),
        translated_rrect_border_pixels(false, false)
    );
}

#[test]
fn explicit_zero_origin_survives_save_layer_hairline_raster() {
    assert_eq!(
        translated_rrect_border_pixels(true, true),
        translated_rrect_border_pixels(false, true)
    );
}

fn paint(color: SkColor4f, blend: SkBlendMode) -> SkPaint {
    let mut paint = SkPaint::default();
    paint.setColor4f(&color, None);
    paint.setBlendMode(blend);
    paint
}

fn rounded_bleed_clip_corner(background: f32, foreground: f32) -> [u8; 4] {
    const WIDTH: usize = 160;
    const HEIGHT: usize = 140;
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    let mut target = RasterTarget::new(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        None,
    )
    .unwrap();
    let opaque = |value: f32| SkColor4f {
        fR: value,
        fG: value,
        fB: value,
        fA: 1.,
    };
    unsafe {
        target
            .draw_rect(
                &SkRect::MakeWH(WIDTH as f32, HEIGHT as f32),
                &paint(opaque(background), SkBlendMode::kSrc),
            )
            .unwrap();
    }

    // This is the BoxDecorationData::kBackgroundBleedClipLayer sequence used
    // by the GitHub footer button: save, AA rounded clip, saveLayer, paint,
    // restore the layer, then restore the clip.
    target.save();
    let rect = SkRect::MakeXYWH(32., 77.21875, 94.734375, 32.);
    target.clip_rrect(
        &SkRRect::MakeRectXY(&rect, 6., 6.),
        SkClipOp::kIntersect,
        true,
    );
    target.save_layer(Some(&rect), None).unwrap();
    unsafe {
        target
            .draw_rect(&rect, &paint(opaque(foreground), SkBlendMode::kSrcOver))
            .unwrap();
    }
    target.restore().unwrap();
    target.restore().unwrap();
    target.finish().unwrap();

    let start = (77 * WIDTH + 35) * 4;
    pixels[start..start + 4].try_into().unwrap()
}

fn rounded_bleed_clip_source_pixel() -> [u8; 4] {
    const WIDTH: usize = 160;
    const HEIGHT: usize = 140;
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    let mut target = RasterTarget::new(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        None,
    )
    .unwrap();
    let rect = SkRect::MakeXYWH(32., 77.21875, 94.734375, 32.);
    target.save();
    target.clip_rrect(
        &SkRRect::MakeRectXY(&rect, 6., 6.),
        SkClipOp::kIntersect,
        true,
    );
    target.save_layer(Some(&rect), None).unwrap();
    unsafe {
        target
            .draw_rect(
                &rect,
                &paint(
                    SkColor4f {
                        fR: 1.,
                        fG: 1.,
                        fB: 1.,
                        fA: 1.,
                    },
                    SkBlendMode::kSrcOver,
                ),
            )
            .unwrap();
    }
    target.restore().unwrap();
    target.restore().unwrap();
    target.finish().unwrap();
    let start = (77 * WIDTH + 35) * 4;
    pixels[start..start + 4].try_into().unwrap()
}

fn rounded_clip_corner_without_layer(background: f32, foreground: f32) -> [u8; 4] {
    const WIDTH: usize = 160;
    const HEIGHT: usize = 140;
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    let mut target = RasterTarget::new(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        None,
    )
    .unwrap();
    let opaque = |value: f32| SkColor4f {
        fR: value,
        fG: value,
        fB: value,
        fA: 1.,
    };
    unsafe {
        target
            .draw_rect(
                &SkRect::MakeWH(WIDTH as f32, HEIGHT as f32),
                &paint(opaque(background), SkBlendMode::kSrc),
            )
            .unwrap();
    }
    target.save();
    let rect = SkRect::MakeXYWH(32., 77.21875, 94.734375, 32.);
    target.clip_rrect(
        &SkRRect::MakeRectXY(&rect, 6., 6.),
        SkClipOp::kIntersect,
        true,
    );
    unsafe {
        target
            .draw_rect(&rect, &paint(opaque(foreground), SkBlendMode::kSrcOver))
            .unwrap();
    }
    target.restore().unwrap();
    target.finish().unwrap();
    let start = (77 * WIDTH + 35) * 4;
    pixels[start..start + 4].try_into().unwrap()
}

#[test]
fn rounded_bleed_clip_layer_uses_source_over_integer_rounding() {
    let black_over_white = rounded_bleed_clip_corner(1., 0.);
    let white_over_black = rounded_bleed_clip_corner(0., 1.);
    assert_eq!(
        (black_over_white, white_over_black),
        ([220, 220, 220, 255], [34, 34, 34, 255])
    );
}

fn github_rounded_button_pixels(use_layer: bool) -> Vec<u8> {
    const WIDTH: usize = 160;
    const HEIGHT: usize = 140;
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    let mut target = RasterTarget::new(
        &mut pixels,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH * 4,
        PixelFormat::Bgra8888,
        None,
    )
    .unwrap();
    unsafe {
        target
            .draw_rect(
                &SkRect::MakeWH(WIDTH as f32, HEIGHT as f32),
                &paint(
                    SkColor4f {
                        fR: 242. / 255.,
                        fG: 245. / 255.,
                        fB: 243. / 255.,
                        fA: 1.,
                    },
                    SkBlendMode::kSrc,
                ),
            )
            .unwrap();
    }

    target.save();
    let rect = SkRect::MakeXYWH(32., 77., 95., 32.);
    target.clip_rrect(
        &SkRRect::MakeRectXY(&rect, 6., 6.),
        SkClipOp::kIntersect,
        true,
    );
    if use_layer {
        target.save_layer(Some(&rect), None).unwrap();
    }
    unsafe {
        target
            .draw_rect(
                &rect,
                &paint(
                    SkColor4f {
                        fR: 8. / 255.,
                        fG: 135. / 255.,
                        fB: 43. / 255.,
                        fA: 1.,
                    },
                    SkBlendMode::kSrcOver,
                ),
            )
            .unwrap();
    }
    if use_layer {
        target.restore().unwrap();
    }
    target.restore().unwrap();
    target.finish().unwrap();
    pixels
}

#[test]
fn github_uniform_rounded_button_layer_uses_solid_color_rounding() {
    let layered = github_rounded_button_pixels(true);
    let at = |x: usize, y: usize| (y * 160 + x) * 4;
    assert_eq!(&layered[at(36, 77)..at(36, 77) + 4], &[107, 170, 83, 255]);
    assert_eq!(&layered[at(37, 77)..at(37, 77) + 4], &[63, 146, 32, 255]);
    assert_eq!(&layered[at(33, 78)..at(33, 78) + 4], &[218, 231, 213, 255]);
}

#[test]
fn rounded_bleed_clip_layer_keeps_premultiplied_white_channels_equal() {
    assert_eq!(rounded_bleed_clip_source_pixel(), [34, 34, 34, 34]);
}

#[test]
fn rounded_clip_without_layer_is_complementary_on_inverse_backgrounds() {
    assert_eq!(
        (
            rounded_clip_corner_without_layer(1., 0.),
            rounded_clip_corner_without_layer(0., 1.)
        ),
        ([221, 221, 221, 255], [34, 34, 34, 255])
    );
}

#[test]
fn save_layer_dst_in_preserves_destination_under_source_alpha() {
    let mut pixels = vec![0; 16 * 8 * 4];
    let mut target =
        RasterTarget::new(&mut pixels, 16, 8, 16 * 4, PixelFormat::Rgba8888, None).unwrap();

    unsafe {
        target
            .draw_rect(
                &SkRect::MakeWH(16., 8.),
                &paint(
                    SkColor4f {
                        fR: 1.,
                        fG: 0.,
                        fB: 0.,
                        fA: 1.,
                    },
                    SkBlendMode::kSrc,
                ),
            )
            .unwrap();
    }

    let restore = paint(
        SkColor4f {
            fR: 0.,
            fG: 0.,
            fB: 0.,
            fA: 1.,
        },
        SkBlendMode::kDstIn,
    );
    target.save_layer(None, Some(&restore)).unwrap();
    unsafe {
        target
            .draw_rect(
                &SkRect::MakeWH(8., 8.),
                &paint(
                    SkColor4f {
                        fR: 1.,
                        fG: 1.,
                        fB: 1.,
                        fA: 1.,
                    },
                    SkBlendMode::kSrc,
                ),
            )
            .unwrap();
    }
    target.restore().unwrap();
    target.finish().unwrap();

    assert_eq!(&pixels[0..4], &[255, 0, 0, 255]);
    assert_eq!(&pixels[(12 * 4)..(12 * 4 + 4)], &[0, 0, 0, 0]);
}

#[test]
fn nested_mask_layers_restore_into_bounded_dst_in_layer() {
    let mut pixels = vec![0; 32 * 16 * 4];
    let mut target =
        RasterTarget::new(&mut pixels, 32, 16, 32 * 4, PixelFormat::Rgba8888, None).unwrap();

    unsafe {
        target
            .draw_rect(
                &SkRect::MakeXYWH(8., 4., 16., 8.),
                &paint(
                    SkColor4f {
                        fR: 1.,
                        fG: 0.,
                        fB: 0.,
                        fA: 1.,
                    },
                    SkBlendMode::kSrc,
                ),
            )
            .unwrap();
    }

    let restore = paint(
        SkColor4f {
            fR: 0.,
            fG: 0.,
            fB: 0.,
            fA: 1.,
        },
        SkBlendMode::kDstIn,
    );
    target
        .save_layer(Some(&SkRect::MakeXYWH(8., 4., 16., 8.)), Some(&restore))
        .unwrap();
    target.save_layer(None, None).unwrap();
    target.save_layer(None, None).unwrap();
    target.clip_rect(
        &SkRect::MakeXYWH(8., 4., 8., 8.),
        skia::include::core::SkClipOp::SkClipOp::kIntersect,
        false,
    );
    unsafe {
        target
            .draw_rect(
                &SkRect::MakeXYWH(8., 4., 8., 8.),
                &paint(
                    SkColor4f {
                        fR: 1.,
                        fG: 1.,
                        fB: 1.,
                        fA: 1.,
                    },
                    SkBlendMode::kSrcOver,
                ),
            )
            .unwrap();
    }
    target.restore().unwrap();
    target.restore().unwrap();
    target.restore().unwrap();
    target.finish().unwrap();

    let at = |x: usize, y: usize| (y * 32 + x) * 4;
    assert_eq!(&pixels[at(10, 8)..at(10, 8) + 4], &[255, 0, 0, 255]);
    assert_eq!(&pixels[at(20, 8)..at(20, 8) + 4], &[0, 0, 0, 0]);
}

#[test]
fn nested_mask_layers_honor_global_bounds_after_tile_translation() {
    let mut pixels = vec![0; 32 * 16 * 4];
    let mut target =
        RasterTarget::new(&mut pixels, 32, 16, 32 * 4, PixelFormat::Rgba8888, None).unwrap();

    // RasterBackend maps content-space coordinates into a tile allocation by
    // translating the canvas by -content_rect.origin before replay.
    target.translate(-256., 0.);
    unsafe {
        target
            .draw_rect(
                &SkRect::MakeXYWH(264., 4., 16., 8.),
                &paint(
                    SkColor4f {
                        fR: 1.,
                        fG: 0.,
                        fB: 0.,
                        fA: 1.,
                    },
                    SkBlendMode::kSrc,
                ),
            )
            .unwrap();
    }

    let restore = paint(
        SkColor4f {
            fR: 0.,
            fG: 0.,
            fB: 0.,
            fA: 1.,
        },
        SkBlendMode::kDstIn,
    );
    target
        .save_layer(Some(&SkRect::MakeXYWH(264., 4., 16., 8.)), Some(&restore))
        .unwrap();
    target.save_layer(None, None).unwrap();
    target.save_layer(None, None).unwrap();
    target.clip_rect(
        &SkRect::MakeXYWH(264., 4., 8., 8.),
        skia::include::core::SkClipOp::SkClipOp::kIntersect,
        false,
    );
    unsafe {
        target
            .draw_rect(
                &SkRect::MakeXYWH(264., 4., 8., 8.),
                &paint(
                    SkColor4f {
                        fR: 1.,
                        fG: 1.,
                        fB: 1.,
                        fA: 1.,
                    },
                    SkBlendMode::kSrcOver,
                ),
            )
            .unwrap();
    }
    target.restore().unwrap();
    target.restore().unwrap();
    target.restore().unwrap();
    target.finish().unwrap();

    let at = |x: usize, y: usize| (y * 32 + x) * 4;
    assert_eq!(&pixels[at(10, 8)..at(10, 8) + 4], &[255, 0, 0, 255]);
    assert_eq!(&pixels[at(20, 8)..at(20, 8) + 4], &[0, 0, 0, 0]);
}

#[test]
fn gradient_mask_layers_honor_global_bounds_after_tile_translation() {
    let mut pixels = vec![0; 32 * 16 * 4];
    let mut target =
        RasterTarget::new(&mut pixels, 32, 16, 32 * 4, PixelFormat::Rgba8888, None).unwrap();

    target.translate(-256., 0.);
    unsafe {
        target
            .draw_rect(
                &SkRect::MakeXYWH(264., 4., 16., 8.),
                &paint(
                    SkColor4f {
                        fR: 1.,
                        fG: 0.,
                        fB: 0.,
                        fA: 1.,
                    },
                    SkBlendMode::kSrc,
                ),
            )
            .unwrap();
    }

    let restore = paint(
        SkColor4f {
            fR: 0.,
            fG: 0.,
            fB: 0.,
            fA: 1.,
        },
        SkBlendMode::kDstIn,
    );
    target
        .save_layer(Some(&SkRect::MakeXYWH(264., 4., 16., 8.)), Some(&restore))
        .unwrap();
    target.save_layer(None, None).unwrap();
    target.save_layer(None, None).unwrap();

    let colors = [
        SkColor4f {
            fR: 1.,
            fG: 1.,
            fB: 1.,
            fA: 1.,
        },
        SkColor4f {
            fR: 1.,
            fG: 1.,
            fB: 1.,
            fA: 0.,
        },
    ];
    let positions = [0., 1.];
    let gradient = SkGradient::SkGradient::new(
        SkGradient::Colors::new(&colors, &positions, SkTileMode::kClamp, None),
        SkGradient::Interpolation::default(),
    );
    let shader = SkGradient::LinearGradient(
        Some(&[SkPoint { fX: 264., fY: 4. }, SkPoint { fX: 280., fY: 4. }]),
        &gradient,
        None,
    )
    .expect("linear mask shader");
    let mut mask = paint(
        SkColor4f {
            fR: 1.,
            fG: 1.,
            fB: 1.,
            fA: 1.,
        },
        SkBlendMode::kSrcOver,
    );
    mask.setShader(Some(shader));
    unsafe {
        target
            .draw_rect(&SkRect::MakeXYWH(264., 4., 16., 8.), &mask)
            .unwrap();
    }
    target.restore().unwrap();
    target.restore().unwrap();
    target.restore().unwrap();
    target.finish().unwrap();

    let at = |x: usize, y: usize| (y * 32 + x) * 4;
    let left = &pixels[at(10, 8)..at(10, 8) + 4];
    let right = &pixels[at(20, 8)..at(20, 8) + 4];
    assert_eq!(&left[1..3], &[0, 0]);
    assert_eq!(&right[1..3], &[0, 0]);
    assert_eq!(
        left[0], left[3],
        "left red must remain premultiplied: {left:?}"
    );
    assert_eq!(
        right[0], right[3],
        "right red must remain premultiplied: {right:?}"
    );
    assert!(
        left[3] > right[3],
        "gradient alpha must decrease: {left:?} {right:?}"
    );
}
