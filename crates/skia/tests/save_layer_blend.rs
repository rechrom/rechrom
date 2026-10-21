use skia::{
    include::{core::SkTileMode::SkTileMode, effects::SkGradient},
    raster_target::{PixelFormat, RasterTarget},
    src::core::{
        SkBlendMode::SkBlendMode, SkColor::SkColor4f, SkPaint::SkPaint, SkPoint::SkPoint,
        SkRect::SkRect,
    },
};

fn paint(color: SkColor4f, blend: SkBlendMode) -> SkPaint {
    let mut paint = SkPaint::default();
    paint.setColor4f(&color, None);
    paint.setBlendMode(blend);
    paint
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
