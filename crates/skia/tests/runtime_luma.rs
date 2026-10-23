use skia::src::{
    core::SkColor::SkColor4f, effects::colorfilters::SkRuntimeColorFilter::MakeLumaColorFilter,
};

#[test]
fn runtime_luma_filter_handles_single_pixel_tail_concurrently() {
    let filter = MakeLumaColorFilter().expect("known luma runtime effect");
    let threads = (0..8)
        .map(|_| {
            let filter = filter.clone();
            std::thread::spawn(move || {
                for iteration in 0..10_000 {
                    let transparent = SkColor4f {
                        fR: 0.0,
                        fG: 0.0,
                        fB: 0.0,
                        fA: 0.0,
                    };
                    let input = match iteration % 4 {
                        0 => transparent,
                        1 => SkColor4f {
                            fR: 1.0,
                            fG: 0.0,
                            fB: 0.0,
                            fA: 1.0,
                        },
                        2 => SkColor4f {
                            fR: 0.0,
                            fG: 1.0,
                            fB: 0.0,
                            fA: 1.0,
                        },
                        _ => SkColor4f {
                            fR: 0.0,
                            fG: 0.0,
                            fB: 1.0,
                            fA: 0.5,
                        },
                    };
                    let expected_alpha =
                        (0.2126 * input.fR + 0.7152 * input.fG + 0.0722 * input.fB) * input.fA;
                    let output = filter
                        .filterColor4f(&input, None, None)
                        .expect("luma filter evaluation");
                    assert_eq!([output.fR, output.fG, output.fB], [0.0; 3]);
                    assert!(
                        (output.fA - expected_alpha).abs() < 0.0001,
                        "{input:?} -> {output:?}, expected alpha {expected_alpha}"
                    );
                }
            })
        })
        .collect::<Vec<_>>();

    for thread in threads {
        thread.join().expect("luma worker");
    }
}
