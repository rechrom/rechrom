//! Opt-in work counters. Absent from ordinary builds; no rendering decisions.
use std::cell::RefCell;

#[derive(Clone, Copy, Default, Debug)]
struct Work {
    quads: usize,
    quad_pixels: usize,
    spans: usize,
    filtered_pixels: usize,
    simd_pixels: usize,
    scalar_pixels: usize,
    short_spans: usize,
    blend_pixels: usize,
}
#[derive(Default)]
struct Profile {
    active: bool,
    class: usize,
    work: [Work; 6],
}
thread_local! { static PROFILE: RefCell<Profile> = RefCell::new(Profile::default()); }

pub(super) fn begin() {
    PROFILE.with(|p| {
        *p.borrow_mut() = Profile {
            active: std::env::var_os("LAYOUTNG_COMPOSE_WORK_PROFILE").is_some(),
            ..Profile::default()
        }
    });
}
pub(super) fn class(class: usize) {
    PROFILE.with(|p| p.borrow_mut().class = class);
}
pub(super) fn quad(pixels: usize) {
    PROFILE.with(|p| {
        let mut p = p.borrow_mut();
        if p.active {
            let class = p.class;
            let w = &mut p.work[class];
            w.quads += 1;
            w.quad_pixels += pixels;
        }
    });
}
pub(super) fn span(pixels: usize, simd: usize) {
    PROFILE.with(|p| {
        let mut p = p.borrow_mut();
        if p.active {
            let class = p.class;
            let w = &mut p.work[class];
            w.spans += 1;
            w.filtered_pixels += pixels;
            w.simd_pixels += simd;
            w.scalar_pixels += pixels - simd;
            w.short_spans += usize::from(pixels < 4);
        }
    });
}
pub(super) fn blend(pixels: usize) {
    PROFILE.with(|p| {
        let mut p = p.borrow_mut();
        if p.active {
            let class = p.class;
            p.work[class].blend_pixels += pixels;
        }
    });
}
pub(super) fn end() {
    PROFILE.with(|p| {
        let mut p = p.borrow_mut();
        if p.active {
            eprintln!("layer-tile-compose-work {:?}", p.work);
            p.active = false;
        }
    });
}
