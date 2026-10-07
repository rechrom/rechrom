//! Precision implementations of upstream SkRasterPipeline_opts.h stages.
//! Migrated tiny-skia context/calling convention remain Rust compatibility adaptations.
/// Mirrors the upstream build-selected `SK_OPTS_NS` namespace.
pub mod SK_OPTS_NS {
    #[cfg(all(not(feature = "std"), feature = "no-std-float"))]
    use crate::path::NoStdFloat;
    // Skia cites http://www.machinedlearnings.com/2011/06/fast-approximate-logarithm-exponential.html
    pub(crate) fn approx_powf(x: f32, y: f32) -> f32 {
        if x == 0.0 || x == 1.0 {
            return x;
        }

        let e = x.to_bits() as f32 * (1.0f32 / ((1 << 23) as f32));
        let m = f32::from_bits((x.to_bits() & 0x007fffff) | 0x3f000000);

        let log2_x =
            e - 124.225514990f32 - 1.498030302f32 * m - 1.725879990f32 / (0.3520887068f32 + m);

        let x = log2_x * y;

        let f = x - x.floor();

        let mut a = x + 121.274057500f32;
        a -= f * 1.490129070f32;
        a += 27.728023300f32 / (4.84252568f32 - f);
        a *= (1 << 23) as f32;

        if a < f32::INFINITY.to_bits() as f32 {
            if a > 0.0 {
                f32::from_bits(a.round() as u32)
            } else {
                0.0
            }
        } else {
            f32::INFINITY
        }
    }

    // Copyright 2018 Google Inc.
    // Copyright 2020 Yevhenii Reizner
    //
    // Use of this source code is governed by a BSD-style license that can be
    // found in the LICENSE file.

    /*
    A high precision raster pipeline implementation.

    Unlike lowp, this one implements all stages.

    Just like Skia, this pipeline is implemented using f32x8.

    For some reason, we are almost 2x slower. Maybe because Skia uses clang's vector extensions
    and we're using a manual implementation.
    */

    use crate::raster::{PixmapRef, PremultipliedColorU8, SpreadMode};
    use crate::PixelFormat;

    use crate::raster::geom::ScreenIntRect;
    use crate::raster::pixmap::SubPixmapMut;
    use crate::raster::wide::{f32x8, i32x8, u32x8};

    pub const STAGE_WIDTH: usize = 8;

    pub type StageFn = fn(p: &mut Pipeline);

    pub struct Pipeline<'a, 'b: 'a> {
        index: usize,
        functions: &'a [StageFn],
        pixmap_src: PixmapRef<'a>,
        pixmap_dst: &'a mut SubPixmapMut<'b>,
        ctx: &'a mut crate::raster::pipeline::Context, // TODO: remove mut
        mask_ctx: crate::raster::pipeline::MaskCtx<'a>,
        aa_mask_ctx: crate::raster::pipeline::AAMaskCtx,
        r: f32x8,
        g: f32x8,
        b: f32x8,
        a: f32x8,
        dr: f32x8,
        dg: f32x8,
        db: f32x8,
        da: f32x8,
        tail: usize,
        dx: usize,
        dy: usize,
    }

    impl Pipeline<'_, '_> {
        #[inline(always)]
        fn next_stage(&mut self) {
            let next: fn(&mut Self) = self.functions[self.index];
            self.index += 1;
            next(self);
        }
    }

    // Must be in the same order as raster_pipeline::Stage
    pub const STAGES: &[StageFn; crate::raster::pipeline::STAGES_COUNT] = &[
        move_src_dst,
        move_dst_src,
        clamp_0,
        clamp_a,
        premul,
        uniform_color,
        seed_shader,
        load_dst,
        store,
        load_dst_u8,
        store_u8,
        gather,
        load_mask_u8,
        mask_u8,
        scale_u8,
        lerp_u8,
        scale_1_float,
        lerp_1_float,
        dstatop,
        dstin,
        dstout,
        dstover,
        srcatop,
        srcin,
        srcout,
        srcover,
        clear,
        modulate,
        multiply,
        plus,
        screen,
        xor,
        colorburn,
        colordodge,
        darken,
        difference,
        exclusion,
        hardlight,
        lighten,
        overlay,
        softlight,
        hue,
        saturation,
        color,
        luminosity,
        srcover_rgba_8888,
        matrix_2x3,
        reflect,
        repeat,
        bilinear,
        bicubic,
        clamp_x_1,
        mirror_x_1,
        repeat_x_1,
        gradient,
        evenly_spaced_2_stop_gradient,
        xy_to_unit_angle,
        xy_to_radius,
        xy_to_2pt_conical_focal_on_circle,
        xy_to_2pt_conical_well_behaved,
        xy_to_2pt_conical_smaller,
        xy_to_2pt_conical_greater,
        xy_to_2pt_conical_strip,
        mask_2pt_conical_nan,
        mask_2pt_conical_degenerates,
        apply_vector_mask,
        alter_2pt_conical_compensate_focal,
        alter_2pt_conical_unswap,
        negate_x,
        apply_concentric_scale_bias,
        gamma_expand_2,
        gamma_expand_dst_2,
        gamma_compress_2,
        gamma_expand_22,
        gamma_expand_dst_22,
        gamma_compress_22,
        gamma_expand_srgb,
        gamma_expand_dst_srgb,
        gamma_compress_srgb,
    ];

    pub fn fn_ptr(f: StageFn) -> *const () {
        f as *const ()
    }

    #[inline(never)]
    pub fn start(
        functions: &[StageFn],
        functions_tail: &[StageFn],
        rect: &ScreenIntRect,
        aa_mask_ctx: crate::raster::pipeline::AAMaskCtx,
        mask_ctx: crate::raster::pipeline::MaskCtx,
        ctx: &mut crate::raster::pipeline::Context,
        pixmap_src: PixmapRef,
        pixmap_dst: &mut SubPixmapMut,
    ) {
        let mut p = Pipeline {
            index: 0,
            functions: &[],
            pixmap_src,
            pixmap_dst,
            mask_ctx,
            aa_mask_ctx,
            ctx,
            r: f32x8::default(),
            g: f32x8::default(),
            b: f32x8::default(),
            a: f32x8::default(),
            dr: f32x8::default(),
            dg: f32x8::default(),
            db: f32x8::default(),
            da: f32x8::default(),
            tail: 0,
            dx: 0,
            dy: 0,
        };

        for y in rect.y()..rect.bottom() {
            let mut x = rect.x() as usize;
            let end = rect.right() as usize;

            p.functions = functions;
            while x + STAGE_WIDTH <= end {
                p.index = 0;
                p.dx = x;
                p.dy = y as usize;
                p.tail = STAGE_WIDTH;
                p.next_stage();
                x += STAGE_WIDTH;
            }

            if x != end {
                p.index = 0;
                p.functions = functions_tail;
                p.dx = x;
                p.dy = y as usize;
                p.tail = end - x;
                p.next_stage();
            }
        }
    }

    fn move_src_dst(p: &mut Pipeline) {
        p.dr = p.r;
        p.dg = p.g;
        p.db = p.b;
        p.da = p.a;

        p.next_stage();
    }

    fn premul(p: &mut Pipeline) {
        p.r *= p.a;
        p.g *= p.a;
        p.b *= p.a;

        p.next_stage();
    }

    fn move_dst_src(p: &mut Pipeline) {
        p.r = p.dr;
        p.g = p.dg;
        p.b = p.db;
        p.a = p.da;

        p.next_stage();
    }

    fn clamp_0(p: &mut Pipeline) {
        p.r = p.r.max(f32x8::default());
        p.g = p.g.max(f32x8::default());
        p.b = p.b.max(f32x8::default());
        p.a = p.a.max(f32x8::default());

        p.next_stage();
    }

    fn clamp_a(p: &mut Pipeline) {
        p.r = p.r.min(f32x8::splat(1.0));
        p.g = p.g.min(f32x8::splat(1.0));
        p.b = p.b.min(f32x8::splat(1.0));
        p.a = p.a.min(f32x8::splat(1.0));

        p.next_stage();
    }

    fn uniform_color(p: &mut Pipeline) {
        let ctx = &p.ctx.uniform_color;
        p.r = f32x8::splat(ctx.r);
        p.g = f32x8::splat(ctx.g);
        p.b = f32x8::splat(ctx.b);
        p.a = f32x8::splat(ctx.a);

        p.next_stage();
    }

    fn seed_shader(p: &mut Pipeline) {
        let iota = f32x8::from([0.5, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5]);

        p.r = f32x8::splat(p.dx as f32) + iota;
        p.g = f32x8::splat(p.dy as f32 + 0.5);
        p.b = f32x8::splat(1.0);
        p.a = f32x8::default();

        p.dr = f32x8::default();
        p.dg = f32x8::default();
        p.db = f32x8::default();
        p.da = f32x8::default();

        p.next_stage();
    }

    pub fn load_dst(p: &mut Pipeline) {
        let format = p.pixmap_dst.format;
        load_8888(
            format,
            p.pixmap_dst.slice4_at_xy(p.dx, p.dy),
            &mut p.dr,
            &mut p.dg,
            &mut p.db,
            &mut p.da,
        );
        p.next_stage();
    }

    pub fn load_dst_tail(p: &mut Pipeline) {
        let format = p.pixmap_dst.format;
        load_8888_tail(
            format,
            p.tail,
            p.pixmap_dst.slice_at_xy(p.dx, p.dy),
            &mut p.dr,
            &mut p.dg,
            &mut p.db,
            &mut p.da,
        );
        p.next_stage();
    }

    pub fn store(p: &mut Pipeline) {
        let format = p.pixmap_dst.format;
        store_8888(
            format,
            &p.r,
            &p.g,
            &p.b,
            &p.a,
            p.pixmap_dst.slice4_at_xy(p.dx, p.dy),
        );
        p.next_stage();
    }

    pub fn store_tail(p: &mut Pipeline) {
        let format = p.pixmap_dst.format;
        store_8888_tail(
            format,
            &p.r,
            &p.g,
            &p.b,
            &p.a,
            p.tail,
            p.pixmap_dst.slice_at_xy(p.dx, p.dy),
        );
        p.next_stage();
    }

    // Currently, all mask/A8 pixmaps are handled by lowp.
    pub fn load_dst_u8(_: &mut Pipeline) {
        // unreachable
    }

    pub fn load_dst_u8_tail(_: &mut Pipeline) {
        // unreachable
    }

    pub fn store_u8(_: &mut Pipeline) {
        // unreachable
    }

    pub fn store_u8_tail(_: &mut Pipeline) {
        // unreachable
    }

    pub fn gather(p: &mut Pipeline) {
        let ix = gather_ix(p.pixmap_src, p.r, p.g);
        load_8888(
            p.pixmap_src.format,
            &p.pixmap_src.gather(ix),
            &mut p.r,
            &mut p.g,
            &mut p.b,
            &mut p.a,
        );

        p.next_stage();
    }

    #[inline(always)]
    fn gather_ix(pixmap: PixmapRef, mut x: f32x8, mut y: f32x8) -> u32x8 {
        // Exclusive -> inclusive.
        let w = ulp_sub(pixmap.width() as f32);
        let h = ulp_sub(pixmap.height() as f32);
        x = x.max(f32x8::default()).min(f32x8::splat(w));
        y = y.max(f32x8::default()).min(f32x8::splat(h));

        (y.trunc_int() * i32x8::splat(pixmap.width() as i32) + x.trunc_int()).to_u32x8_bitcast()
    }

    #[inline(always)]
    fn ulp_sub(v: f32) -> f32 {
        // Somewhat similar to v - f32::EPSILON
        bytemuck::cast::<u32, f32>(bytemuck::cast::<f32, u32>(v) - 1)
    }

    fn load_mask_u8(_: &mut Pipeline) {
        // unreachable
    }

    fn mask_u8(p: &mut Pipeline) {
        let offset = p.mask_ctx.offset(p.dx, p.dy);
        let mut c = [0.0; 8];
        for i in 0..p.tail {
            c[i] = p.mask_ctx.data[offset + i] as f32;
        }
        let c = f32x8::from(c) / f32x8::splat(255.0);

        if c == f32x8::default() {
            return;
        }

        p.r *= c;
        p.g *= c;
        p.b *= c;
        p.a *= c;

        p.next_stage();
    }

    fn scale_u8(p: &mut Pipeline) {
        // Load u8xTail and cast it to f32x8.
        let data = p.aa_mask_ctx.copy_at_xy(p.dx, p.dy, p.tail);
        let c = f32x8::from([data[0] as f32, data[1] as f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let c = c / f32x8::splat(255.0);

        p.r *= c;
        p.g *= c;
        p.b *= c;
        p.a *= c;

        p.next_stage();
    }

    fn lerp_u8(p: &mut Pipeline) {
        // Load u8xTail and cast it to f32x8.
        let data = p.aa_mask_ctx.copy_at_xy(p.dx, p.dy, p.tail);
        let c = f32x8::from([data[0] as f32, data[1] as f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let c = c / f32x8::splat(255.0);

        p.r = lerp(p.dr, p.r, c);
        p.g = lerp(p.dg, p.g, c);
        p.b = lerp(p.db, p.b, c);
        p.a = lerp(p.da, p.a, c);

        p.next_stage();
    }

    fn scale_1_float(p: &mut Pipeline) {
        let c = f32x8::splat(p.ctx.current_coverage);
        p.r *= c;
        p.g *= c;
        p.b *= c;
        p.a *= c;

        p.next_stage();
    }

    fn lerp_1_float(p: &mut Pipeline) {
        let c = f32x8::splat(p.ctx.current_coverage);
        p.r = lerp(p.dr, p.r, c);
        p.g = lerp(p.dg, p.g, c);
        p.b = lerp(p.db, p.b, c);
        p.a = lerp(p.da, p.a, c);

        p.next_stage();
    }

    macro_rules! blend_fn {
        ($name:ident, $f:expr) => {
            fn $name(p: &mut Pipeline) {
                p.r = $f(p.r, p.dr, p.a, p.da);
                p.g = $f(p.g, p.dg, p.a, p.da);
                p.b = $f(p.b, p.db, p.a, p.da);
                p.a = $f(p.a, p.da, p.a, p.da);

                p.next_stage();
            }
        };
    }

    blend_fn!(clear, |_, _, _, _| f32x8::default());
    blend_fn!(srcatop, |s, d, sa, da| s * da + d * inv(sa));
    blend_fn!(dstatop, |s, d, sa, da| d * sa + s * inv(da));
    blend_fn!(srcin, |s, _, _, da| s * da);
    blend_fn!(dstin, |_, d, sa, _| d * sa);
    blend_fn!(srcout, |s, _, _, da| s * inv(da));
    blend_fn!(dstout, |_, d, sa, _| d * inv(sa));
    blend_fn!(srcover, |s, d, sa, _| mad(d, inv(sa), s));
    blend_fn!(dstover, |s, d, _, da| mad(s, inv(da), d));
    blend_fn!(modulate, |s, d, _, _| s * d);
    blend_fn!(multiply, |s, d, sa, da| s * inv(da) + d * inv(sa) + s * d);
    blend_fn!(screen, |s, d, _, _| s + d - s * d);
    blend_fn!(xor, |s, d, sa, da| s * inv(da) + d * inv(sa));

    // Wants a type for some reason.
    blend_fn!(plus, |s: f32x8, d: f32x8, _, _| (s + d)
        .min(f32x8::splat(1.0)));

    macro_rules! blend_fn2 {
        ($name:ident, $f:expr) => {
            fn $name(p: &mut Pipeline) {
                // The same logic applied to color, and srcover for alpha.
                p.r = $f(p.r, p.dr, p.a, p.da);
                p.g = $f(p.g, p.dg, p.a, p.da);
                p.b = $f(p.b, p.db, p.a, p.da);
                p.a = mad(p.da, inv(p.a), p.a);

                p.next_stage();
            }
        };
    }

    blend_fn2!(darken, |s: f32x8, d, sa, da: f32x8| s + d
        - (s * da).max(d * sa));
    blend_fn2!(lighten, |s: f32x8, d, sa, da: f32x8| s + d
        - (s * da).min(d * sa));
    blend_fn2!(difference, |s: f32x8, d, sa, da: f32x8| s + d
        - two((s * da).min(d * sa)));
    blend_fn2!(exclusion, |s: f32x8, d, _, _| s + d - two(s * d));

    blend_fn2!(colorburn, |s: f32x8, d: f32x8, sa: f32x8, da: f32x8| d
        .cmp_eq(da)
        .blend(
            d + s * inv(da),
            s.cmp_eq(f32x8::default()).blend(
                d * inv(sa),
                sa * (da - da.min((da - d) * sa * s.recip_fast())) + s * inv(da) + d * inv(sa)
            )
        ));

    blend_fn2!(colordodge, |s: f32x8, d: f32x8, sa: f32x8, da: f32x8| d
        .cmp_eq(f32x8::default())
        .blend(
            s * inv(da),
            s.cmp_eq(sa).blend(
                s + d * inv(sa),
                sa * da.min((d * sa) * (sa - s).recip_fast()) + s * inv(da) + d * inv(sa)
            )
        ));

    blend_fn2!(hardlight, |s: f32x8, d: f32x8, sa, da| s * inv(da)
        + d * inv(sa)
        + two(s)
            .cmp_le(sa)
            .blend(two(s * d), sa * da - two((da - d) * (sa - s))));

    blend_fn2!(overlay, |s: f32x8, d: f32x8, sa, da| s * inv(da)
        + d * inv(sa)
        + two(d)
            .cmp_le(da)
            .blend(two(s * d), sa * da - two((da - d) * (sa - s))));

    blend_fn2!(softlight, |s: f32x8, d: f32x8, sa: f32x8, da: f32x8| {
        let m = da.cmp_gt(f32x8::default()).blend(d / da, f32x8::default());
        let s2 = two(s);
        let m4 = two(two(m));

        // The logic forks three ways:
        //    1. dark src?
        //    2. light src, dark dst?
        //    3. light src, light dst?
        let dark_src = d * (sa + (s2 - sa) * (f32x8::splat(1.0) - m));
        let dark_dst = (m4 * m4 + m4) * (m - f32x8::splat(1.0)) + f32x8::splat(7.0) * m;
        let lite_dst = m.sqrt() - m;
        let lite_src = d * sa + da * (s2 - sa) * two(two(d)).cmp_le(da).blend(dark_dst, lite_dst); // 2 or 3?

        s * inv(da) + d * inv(sa) + s2.cmp_le(sa).blend(dark_src, lite_src) // 1 or (2 or 3)?
    });

    // We're basing our implementation of non-separable blend modes on
    //   https://www.w3.org/TR/compositing-1/#blendingnonseparable.
    // and
    //   https://www.khronos.org/registry/OpenGL/specs/es/3.2/es_spec_3.2.pdf
    // They're equivalent, but ES' math has been better simplified.
    //
    // Anything extra we add beyond that is to make the math work with premul inputs.

    macro_rules! blend_fn3 {
        ($name:ident, $f:expr) => {
            fn $name(p: &mut Pipeline) {
                let (tr, tg, tb, ta) = $f(p.r, p.g, p.b, p.a, p.dr, p.dg, p.db, p.da);
                p.r = tr;
                p.g = tg;
                p.b = tb;
                p.a = ta;

                p.next_stage();
            }
        };
    }

    blend_fn3!(hue, hue_k);

    #[inline(always)]
    fn hue_k(
        r: f32x8,
        g: f32x8,
        b: f32x8,
        a: f32x8,
        dr: f32x8,
        dg: f32x8,
        db: f32x8,
        da: f32x8,
    ) -> (f32x8, f32x8, f32x8, f32x8) {
        let rr = &mut (r * a);
        let gg = &mut (g * a);
        let bb = &mut (b * a);

        set_sat(rr, gg, bb, sat(dr, dg, db) * a);
        set_lum(rr, gg, bb, lum(dr, dg, db) * a);
        clip_color(rr, gg, bb, a * da);

        let r = r * inv(da) + dr * inv(a) + *rr;
        let g = g * inv(da) + dg * inv(a) + *gg;
        let b = b * inv(da) + db * inv(a) + *bb;
        let a = a + da - a * da;

        (r, g, b, a)
    }

    blend_fn3!(saturation, saturation_k);

    #[inline(always)]
    fn saturation_k(
        r: f32x8,
        g: f32x8,
        b: f32x8,
        a: f32x8,
        dr: f32x8,
        dg: f32x8,
        db: f32x8,
        da: f32x8,
    ) -> (f32x8, f32x8, f32x8, f32x8) {
        let rr = &mut (dr * a);
        let gg = &mut (dg * a);
        let bb = &mut (db * a);

        set_sat(rr, gg, bb, sat(r, g, b) * da);
        set_lum(rr, gg, bb, lum(dr, dg, db) * a); // (This is not redundant.)
        clip_color(rr, gg, bb, a * da);

        let r = r * inv(da) + dr * inv(a) + *rr;
        let g = g * inv(da) + dg * inv(a) + *gg;
        let b = b * inv(da) + db * inv(a) + *bb;
        let a = a + da - a * da;

        (r, g, b, a)
    }

    blend_fn3!(color, color_k);

    #[inline(always)]
    fn color_k(
        r: f32x8,
        g: f32x8,
        b: f32x8,
        a: f32x8,
        dr: f32x8,
        dg: f32x8,
        db: f32x8,
        da: f32x8,
    ) -> (f32x8, f32x8, f32x8, f32x8) {
        let rr = &mut (r * da);
        let gg = &mut (g * da);
        let bb = &mut (b * da);

        set_lum(rr, gg, bb, lum(dr, dg, db) * a);
        clip_color(rr, gg, bb, a * da);

        let r = r * inv(da) + dr * inv(a) + *rr;
        let g = g * inv(da) + dg * inv(a) + *gg;
        let b = b * inv(da) + db * inv(a) + *bb;
        let a = a + da - a * da;

        (r, g, b, a)
    }

    blend_fn3!(luminosity, luminosity_k);

    #[inline(always)]
    fn luminosity_k(
        r: f32x8,
        g: f32x8,
        b: f32x8,
        a: f32x8,
        dr: f32x8,
        dg: f32x8,
        db: f32x8,
        da: f32x8,
    ) -> (f32x8, f32x8, f32x8, f32x8) {
        let rr = &mut (dr * a);
        let gg = &mut (dg * a);
        let bb = &mut (db * a);

        set_lum(rr, gg, bb, lum(r, g, b) * da);
        clip_color(rr, gg, bb, a * da);

        let r = r * inv(da) + dr * inv(a) + *rr;
        let g = g * inv(da) + dg * inv(a) + *gg;
        let b = b * inv(da) + db * inv(a) + *bb;
        let a = a + da - a * da;

        (r, g, b, a)
    }

    #[inline(always)]
    fn sat(r: f32x8, g: f32x8, b: f32x8) -> f32x8 {
        r.max(g.max(b)) - r.min(g.min(b))
    }

    #[inline(always)]
    fn lum(r: f32x8, g: f32x8, b: f32x8) -> f32x8 {
        r * f32x8::splat(0.30) + g * f32x8::splat(0.59) + b * f32x8::splat(0.11)
    }

    #[inline(always)]
    fn set_sat(r: &mut f32x8, g: &mut f32x8, b: &mut f32x8, s: f32x8) {
        let mn = r.min(g.min(*b));
        let mx = r.max(g.max(*b));
        let sat = mx - mn;

        // Map min channel to 0, max channel to s, and scale the middle proportionally.
        let scale = |c| {
            sat.cmp_eq(f32x8::default())
                .blend(f32x8::default(), (c - mn) * s / sat)
        };

        *r = scale(*r);
        *g = scale(*g);
        *b = scale(*b);
    }

    #[inline(always)]
    fn set_lum(r: &mut f32x8, g: &mut f32x8, b: &mut f32x8, l: f32x8) {
        let diff = l - lum(*r, *g, *b);
        *r += diff;
        *g += diff;
        *b += diff;
    }

    #[inline(always)]
    fn clip_color(r: &mut f32x8, g: &mut f32x8, b: &mut f32x8, a: f32x8) {
        let mn = r.min(g.min(*b));
        let mx = r.max(g.max(*b));
        let l = lum(*r, *g, *b);

        let clip = |mut c| {
            c = mx
                .cmp_ge(f32x8::default())
                .blend(c, l + (c - l) * l / (l - mn));
            c = mx.cmp_gt(a).blend(l + (c - l) * (a - l) / (mx - l), c);
            c = c.max(f32x8::default()); // Sometimes without this we may dip just a little negative.
            c
        };

        *r = clip(*r);
        *g = clip(*g);
        *b = clip(*b);
    }

    pub fn srcover_rgba_8888(p: &mut Pipeline) {
        let format = p.pixmap_dst.format;
        let pixels = p.pixmap_dst.slice4_at_xy(p.dx, p.dy);
        load_8888(format, pixels, &mut p.dr, &mut p.dg, &mut p.db, &mut p.da);
        p.r = mad(p.dr, inv(p.a), p.r);
        p.g = mad(p.dg, inv(p.a), p.g);
        p.b = mad(p.db, inv(p.a), p.b);
        p.a = mad(p.da, inv(p.a), p.a);
        store_8888(format, &p.r, &p.g, &p.b, &p.a, pixels);

        p.next_stage();
    }

    pub fn srcover_rgba_8888_tail(p: &mut Pipeline) {
        let format = p.pixmap_dst.format;
        let pixels = p.pixmap_dst.slice_at_xy(p.dx, p.dy);
        load_8888_tail(
            format, p.tail, pixels, &mut p.dr, &mut p.dg, &mut p.db, &mut p.da,
        );
        p.r = mad(p.dr, inv(p.a), p.r);
        p.g = mad(p.dg, inv(p.a), p.g);
        p.b = mad(p.db, inv(p.a), p.b);
        p.a = mad(p.da, inv(p.a), p.a);
        store_8888_tail(format, &p.r, &p.g, &p.b, &p.a, p.tail, pixels);

        p.next_stage();
    }

    fn matrix_2x3(p: &mut Pipeline) {
        let ts = &p.ctx.transform;

        let tr = mad(
            p.r,
            f32x8::splat(ts.sx),
            mad(p.g, f32x8::splat(ts.kx), f32x8::splat(ts.tx)),
        );
        let tg = mad(
            p.r,
            f32x8::splat(ts.ky),
            mad(p.g, f32x8::splat(ts.sy), f32x8::splat(ts.ty)),
        );
        p.r = tr;
        p.g = tg;

        p.next_stage();
    }

    // Tile x or y to [0,limit) == [0,limit - 1 ulp] (think, sampling from images).
    // The gather stages will hard clamp the output of these stages to [0,limit)...
    // we just need to do the basic repeat or mirroring.

    fn reflect(p: &mut Pipeline) {
        let ctx = &p.ctx.limit_x;
        p.r = exclusive_reflect(p.r, ctx.scale, ctx.inv_scale);

        let ctx = &p.ctx.limit_y;
        p.g = exclusive_reflect(p.g, ctx.scale, ctx.inv_scale);

        p.next_stage();
    }

    #[inline(always)]
    fn exclusive_reflect(v: f32x8, limit: f32, inv_limit: f32) -> f32x8 {
        let limit = f32x8::splat(limit);
        let inv_limit = f32x8::splat(inv_limit);
        ((v - limit)
            - (limit + limit) * ((v - limit) * (inv_limit * f32x8::splat(0.5))).floor()
            - limit)
            .abs()
    }

    fn repeat(p: &mut Pipeline) {
        let ctx = &p.ctx.limit_x;
        p.r = exclusive_repeat(p.r, ctx.scale, ctx.inv_scale);

        let ctx = &p.ctx.limit_y;
        p.g = exclusive_repeat(p.g, ctx.scale, ctx.inv_scale);

        p.next_stage();
    }

    #[inline(always)]
    fn exclusive_repeat(v: f32x8, limit: f32, inv_limit: f32) -> f32x8 {
        v - (v * f32x8::splat(inv_limit)).floor() * f32x8::splat(limit)
    }

    fn bilinear(p: &mut Pipeline) {
        let x = p.r;
        let fx = (x + f32x8::splat(0.5)).fract();
        let y = p.g;
        let fy = (y + f32x8::splat(0.5)).fract();
        let one = f32x8::splat(1.0);
        let wx = [one - fx, fx];
        let wy = [one - fy, fy];

        sampler_2x2(
            p.pixmap_src,
            &p.ctx.sampler,
            x,
            y,
            &wx,
            &wy,
            &mut p.r,
            &mut p.g,
            &mut p.b,
            &mut p.a,
        );

        p.next_stage();
    }

    fn bicubic(p: &mut Pipeline) {
        let x = p.r;
        let fx = (x + f32x8::splat(0.5)).fract();
        let y = p.g;
        let fy = (y + f32x8::splat(0.5)).fract();
        let one = f32x8::splat(1.0);
        let wx = [
            bicubic_far(one - fx),
            bicubic_near(one - fx),
            bicubic_near(fx),
            bicubic_far(fx),
        ];
        let wy = [
            bicubic_far(one - fy),
            bicubic_near(one - fy),
            bicubic_near(fy),
            bicubic_far(fy),
        ];

        sampler_4x4(
            p.pixmap_src,
            &p.ctx.sampler,
            x,
            y,
            &wx,
            &wy,
            &mut p.r,
            &mut p.g,
            &mut p.b,
            &mut p.a,
        );

        p.next_stage();
    }

    // In bicubic interpolation, the 16 pixels and +/- 0.5 and +/- 1.5 offsets from the sample
    // pixel center are combined with a non-uniform cubic filter, with higher values near the center.
    //
    // We break this function into two parts, one for near 0.5 offsets and one for far 1.5 offsets.

    #[inline(always)]
    fn bicubic_near(t: f32x8) -> f32x8 {
        // 1/18 + 9/18t + 27/18t^2 - 21/18t^3 == t ( t ( -21/18t + 27/18) + 9/18) + 1/18
        mad(
            t,
            mad(
                t,
                mad(f32x8::splat(-21.0 / 18.0), t, f32x8::splat(27.0 / 18.0)),
                f32x8::splat(9.0 / 18.0),
            ),
            f32x8::splat(1.0 / 18.0),
        )
    }

    #[inline(always)]
    fn bicubic_far(t: f32x8) -> f32x8 {
        // 0/18 + 0/18*t - 6/18t^2 + 7/18t^3 == t^2 (7/18t - 6/18)
        (t * t) * mad(f32x8::splat(7.0 / 18.0), t, f32x8::splat(-6.0 / 18.0))
    }

    #[inline(always)]
    fn sampler_2x2(
        pixmap: PixmapRef,
        ctx: &crate::raster::pipeline::SamplerCtx,
        cx: f32x8,
        cy: f32x8,
        wx: &[f32x8; 2],
        wy: &[f32x8; 2],
        r: &mut f32x8,
        g: &mut f32x8,
        b: &mut f32x8,
        a: &mut f32x8,
    ) {
        *r = f32x8::default();
        *g = f32x8::default();
        *b = f32x8::default();
        *a = f32x8::default();

        let one = f32x8::splat(1.0);
        let start = -0.5;
        let mut y = cy + f32x8::splat(start);
        for j in 0..2 {
            let mut x = cx + f32x8::splat(start);
            for i in 0..2 {
                let mut rr = f32x8::default();
                let mut gg = f32x8::default();
                let mut bb = f32x8::default();
                let mut aa = f32x8::default();
                sample(pixmap, ctx, x, y, &mut rr, &mut gg, &mut bb, &mut aa);

                let w = wx[i] * wy[j];
                *r = mad(w, rr, *r);
                *g = mad(w, gg, *g);
                *b = mad(w, bb, *b);
                *a = mad(w, aa, *a);

                x += one;
            }

            y += one;
        }
    }

    #[inline(always)]
    fn sampler_4x4(
        pixmap: PixmapRef,
        ctx: &crate::raster::pipeline::SamplerCtx,
        cx: f32x8,
        cy: f32x8,
        wx: &[f32x8; 4],
        wy: &[f32x8; 4],
        r: &mut f32x8,
        g: &mut f32x8,
        b: &mut f32x8,
        a: &mut f32x8,
    ) {
        *r = f32x8::default();
        *g = f32x8::default();
        *b = f32x8::default();
        *a = f32x8::default();

        let one = f32x8::splat(1.0);
        let start = -1.5;
        let mut y = cy + f32x8::splat(start);
        for j in 0..4 {
            let mut x = cx + f32x8::splat(start);
            for i in 0..4 {
                let mut rr = f32x8::default();
                let mut gg = f32x8::default();
                let mut bb = f32x8::default();
                let mut aa = f32x8::default();
                sample(pixmap, ctx, x, y, &mut rr, &mut gg, &mut bb, &mut aa);

                let w = wx[i] * wy[j];
                *r = mad(w, rr, *r);
                *g = mad(w, gg, *g);
                *b = mad(w, bb, *b);
                *a = mad(w, aa, *a);

                x += one;
            }

            y += one;
        }
    }

    #[inline(always)]
    fn sample(
        pixmap: PixmapRef,
        ctx: &crate::raster::pipeline::SamplerCtx,
        mut x: f32x8,
        mut y: f32x8,
        r: &mut f32x8,
        g: &mut f32x8,
        b: &mut f32x8,
        a: &mut f32x8,
    ) {
        x = tile(x, ctx.spread_mode, pixmap.width() as f32, ctx.inv_width);
        y = tile(y, ctx.spread_mode, pixmap.height() as f32, ctx.inv_height);

        let ix = gather_ix(pixmap, x, y);
        load_8888(pixmap.format, &pixmap.gather(ix), r, g, b, a);
    }

    #[inline(always)]
    fn tile(v: f32x8, mode: SpreadMode, limit: f32, inv_limit: f32) -> f32x8 {
        match mode {
            SpreadMode::Pad => v,
            SpreadMode::Repeat => exclusive_repeat(v, limit, inv_limit),
            SpreadMode::Reflect => exclusive_reflect(v, limit, inv_limit),
        }
    }

    fn clamp_x_1(p: &mut Pipeline) {
        p.r = p.r.normalize();

        p.next_stage();
    }

    fn mirror_x_1(p: &mut Pipeline) {
        p.r = ((p.r - f32x8::splat(1.0))
            - two(((p.r - f32x8::splat(1.0)) * f32x8::splat(0.5)).floor())
            - f32x8::splat(1.0))
        .abs()
        .normalize();

        p.next_stage();
    }

    fn repeat_x_1(p: &mut Pipeline) {
        p.r = (p.r - p.r.floor()).normalize();

        p.next_stage();
    }

    fn gradient(p: &mut Pipeline) {
        let ctx = &p.ctx.gradient;

        // N.B. The loop starts at 1 because idx 0 is the color to use before the first stop.
        let t: [f32; 8] = p.r.into();
        let mut idx = u32x8::default();
        for i in 1..ctx.len {
            let tt = ctx.t_values[i].get();
            let n: u32x8 = bytemuck::cast([
                (t[0] >= tt) as u32,
                (t[1] >= tt) as u32,
                (t[2] >= tt) as u32,
                (t[3] >= tt) as u32,
                (t[4] >= tt) as u32,
                (t[5] >= tt) as u32,
                (t[6] >= tt) as u32,
                (t[7] >= tt) as u32,
            ]);
            idx = idx + n;
        }
        gradient_lookup(ctx, &idx, p.r, &mut p.r, &mut p.g, &mut p.b, &mut p.a);

        p.next_stage();
    }

    fn gradient_lookup(
        ctx: &crate::raster::pipeline::GradientCtx,
        idx: &u32x8,
        t: f32x8,
        r: &mut f32x8,
        g: &mut f32x8,
        b: &mut f32x8,
        a: &mut f32x8,
    ) {
        let idx: [u32; 8] = bytemuck::cast(*idx);

        macro_rules! gather {
            ($d:expr, $c:ident) => {
                // Surprisingly, but bound checking doesn't affect the performance.
                // And since `idx` can contain any number, we should leave it in place.
                f32x8::from([
                    $d[idx[0] as usize].$c,
                    $d[idx[1] as usize].$c,
                    $d[idx[2] as usize].$c,
                    $d[idx[3] as usize].$c,
                    $d[idx[4] as usize].$c,
                    $d[idx[5] as usize].$c,
                    $d[idx[6] as usize].$c,
                    $d[idx[7] as usize].$c,
                ])
            };
        }

        let fr = gather!(&ctx.factors, r);
        let fg = gather!(&ctx.factors, g);
        let fb = gather!(&ctx.factors, b);
        let fa = gather!(&ctx.factors, a);

        let br = gather!(&ctx.biases, r);
        let bg = gather!(&ctx.biases, g);
        let bb = gather!(&ctx.biases, b);
        let ba = gather!(&ctx.biases, a);

        *r = mad(t, fr, br);
        *g = mad(t, fg, bg);
        *b = mad(t, fb, bb);
        *a = mad(t, fa, ba);
    }

    fn evenly_spaced_2_stop_gradient(p: &mut Pipeline) {
        let ctx = &p.ctx.evenly_spaced_2_stop_gradient;

        let t = p.r;
        p.r = mad(t, f32x8::splat(ctx.factor.r), f32x8::splat(ctx.bias.r));
        p.g = mad(t, f32x8::splat(ctx.factor.g), f32x8::splat(ctx.bias.g));
        p.b = mad(t, f32x8::splat(ctx.factor.b), f32x8::splat(ctx.bias.b));
        p.a = mad(t, f32x8::splat(ctx.factor.a), f32x8::splat(ctx.bias.a));

        p.next_stage();
    }

    fn xy_to_unit_angle(p: &mut Pipeline) {
        let x = p.r;
        let y = p.g;
        let x_abs = x.abs();
        let y_abs = y.abs();
        let slope = x_abs.min(y_abs) / x_abs.max(y_abs);
        let s = slope * slope;
        // Use a 7th degree polynomial to approximate atan.
        // This was generated using sollya.gforge.inria.fr.
        // A float optimized polynomial was generated using the following command.
        // P1 = fpminimax((1/(2*Pi))*atan(x),[|1,3,5,7|],[|24...|],[2^(-40),1],relative);
        let phi = slope
            * (f32x8::splat(0.15912117063999176025390625)
                + s * (f32x8::splat(-5.185396969318389892578125e-2)
                    + s * (f32x8::splat(2.476101927459239959716796875e-2)
                        + s * (f32x8::splat(-7.0547382347285747528076171875e-3)))));
        let phi = x_abs.cmp_lt(y_abs).blend(f32x8::splat(0.25) - phi, phi);
        let phi = x
            .cmp_lt(f32x8::splat(0.0))
            .blend(f32x8::splat(0.5) - phi, phi);
        let phi = y
            .cmp_lt(f32x8::splat(0.0))
            .blend(f32x8::splat(1.0) - phi, phi);
        let phi = phi.cmp_ne(phi).blend(f32x8::splat(0.0), phi);
        p.r = phi;
        p.next_stage();
    }

    fn xy_to_radius(p: &mut Pipeline) {
        let x2 = p.r * p.r;
        let y2 = p.g * p.g;
        p.r = (x2 + y2).sqrt();

        p.next_stage();
    }

    fn xy_to_2pt_conical_focal_on_circle(p: &mut Pipeline) {
        let x = p.r;
        let y = p.g;
        p.r = x + y * y / x;

        p.next_stage();
    }

    fn xy_to_2pt_conical_well_behaved(p: &mut Pipeline) {
        let ctx = &p.ctx.two_point_conical_gradient;

        let x = p.r;
        let y = p.g;
        p.r = (x * x + y * y).sqrt() - x * f32x8::splat(ctx.p0);

        p.next_stage();
    }

    fn xy_to_2pt_conical_greater(p: &mut Pipeline) {
        let ctx = &p.ctx.two_point_conical_gradient;

        let x = p.r;
        let y = p.g;
        p.r = (x * x - y * y).sqrt() - x * f32x8::splat(ctx.p0);

        p.next_stage();
    }

    fn xy_to_2pt_conical_smaller(p: &mut Pipeline) {
        let ctx = &p.ctx.two_point_conical_gradient;

        let x = p.r;
        let y = p.g;
        p.r = -(x * x - y * y).sqrt() - x * f32x8::splat(ctx.p0);

        p.next_stage();
    }

    fn xy_to_2pt_conical_strip(p: &mut Pipeline) {
        let ctx = &p.ctx.two_point_conical_gradient;

        let x = p.r;
        let y = p.g;
        p.r = x + (f32x8::splat(ctx.p0) - y * y).sqrt();

        p.next_stage();
    }

    fn mask_2pt_conical_nan(p: &mut Pipeline) {
        let ctx = &mut p.ctx.two_point_conical_gradient;

        let t = p.r;
        let is_degenerate = t.cmp_ne(t);
        p.r = is_degenerate.blend(f32x8::default(), t);
        ctx.mask = cond_to_mask(!is_degenerate.to_u32x8_bitcast());

        p.next_stage();
    }

    fn mask_2pt_conical_degenerates(p: &mut Pipeline) {
        let ctx = &mut p.ctx.two_point_conical_gradient;

        let t = p.r;
        let is_degenerate = t.cmp_le(f32x8::default()) | t.cmp_ne(t);
        p.r = is_degenerate.blend(f32x8::default(), t);
        ctx.mask = cond_to_mask(!is_degenerate.to_u32x8_bitcast());

        p.next_stage();
    }

    fn apply_vector_mask(p: &mut Pipeline) {
        let ctx = &p.ctx.two_point_conical_gradient;

        p.r = (p.r.to_u32x8_bitcast() & ctx.mask).to_f32x8_bitcast();
        p.g = (p.g.to_u32x8_bitcast() & ctx.mask).to_f32x8_bitcast();
        p.b = (p.b.to_u32x8_bitcast() & ctx.mask).to_f32x8_bitcast();
        p.a = (p.a.to_u32x8_bitcast() & ctx.mask).to_f32x8_bitcast();

        p.next_stage();
    }

    fn alter_2pt_conical_compensate_focal(p: &mut Pipeline) {
        let ctx = &p.ctx.two_point_conical_gradient;

        p.r = p.r + f32x8::splat(ctx.p1);

        p.next_stage();
    }

    fn alter_2pt_conical_unswap(p: &mut Pipeline) {
        p.r = f32x8::splat(1.0) - p.r;

        p.next_stage();
    }

    fn negate_x(p: &mut Pipeline) {
        p.r = -p.r;

        p.next_stage();
    }

    fn apply_concentric_scale_bias(p: &mut Pipeline) {
        let ctx = &p.ctx.two_point_conical_gradient;

        // Apply t = t * scale + bias for concentric gradients
        let x = p.r;
        p.r = x * f32x8::splat(ctx.p0) + f32x8::splat(ctx.p1);

        p.next_stage();
    }

    fn gamma_expand_2(p: &mut Pipeline) {
        p.r = p.r * p.r;
        p.g = p.g * p.g;
        p.b = p.b * p.b;

        p.next_stage();
    }

    fn gamma_expand_dst_2(p: &mut Pipeline) {
        p.dr = p.dr * p.dr;
        p.dg = p.dg * p.dg;
        p.db = p.db * p.db;

        p.next_stage();
    }

    fn gamma_compress_2(p: &mut Pipeline) {
        p.r = p.r.sqrt();
        p.g = p.g.sqrt();
        p.b = p.b.sqrt();

        p.next_stage();
    }

    fn gamma_expand_22(p: &mut Pipeline) {
        p.r = p.r.powf(2.2);
        p.g = p.g.powf(2.2);
        p.b = p.b.powf(2.2);

        p.next_stage();
    }

    fn gamma_expand_dst_22(p: &mut Pipeline) {
        p.dr = p.dr.powf(2.2);
        p.dg = p.dg.powf(2.2);
        p.db = p.db.powf(2.2);

        p.next_stage();
    }

    fn gamma_compress_22(p: &mut Pipeline) {
        p.r = p.r.powf(0.45454545);
        p.g = p.g.powf(0.45454545);
        p.b = p.b.powf(0.45454545);

        p.next_stage();
    }

    fn srgb_expand(x: f32x8) -> f32x8 {
        let small = x.cmp_le(f32x8::splat(0.04045));
        let linear = x / f32x8::splat(12.92);
        let exp = ((x + f32x8::splat(0.055)) / f32x8::splat(1.055)).powf(2.4);
        small.blend(linear, exp)
    }

    fn srgb_compress(x: f32x8) -> f32x8 {
        let small = x.cmp_le(f32x8::splat(0.0031308));
        let linear = x * f32x8::splat(12.92);
        let exp = x.powf(0.416666666) * f32x8::splat(1.055) - f32x8::splat(0.055);
        small.blend(linear, exp)
    }

    fn gamma_expand_srgb(p: &mut Pipeline) {
        p.r = srgb_expand(p.r);
        p.g = srgb_expand(p.g);
        p.b = srgb_expand(p.b);

        p.next_stage();
    }

    fn gamma_expand_dst_srgb(p: &mut Pipeline) {
        p.dr = srgb_expand(p.dr);
        p.dg = srgb_expand(p.dg);
        p.db = srgb_expand(p.db);

        p.next_stage();
    }

    fn gamma_compress_srgb(p: &mut Pipeline) {
        p.r = srgb_compress(p.r);
        p.g = srgb_compress(p.g);
        p.b = srgb_compress(p.b);

        p.next_stage();
    }

    pub fn just_return(_: &mut Pipeline) {
        // Ends the loop.
    }

    #[inline(always)]
    fn cond_to_mask(cond: u32x8) -> u32x8 {
        let cond: [u32; 8] = bytemuck::cast(cond);
        bytemuck::cast([
            if cond[0] != 0 { !0 } else { 0 },
            if cond[1] != 0 { !0 } else { 0 },
            if cond[2] != 0 { !0 } else { 0 },
            if cond[3] != 0 { !0 } else { 0 },
            if cond[4] != 0 { !0 } else { 0 },
            if cond[5] != 0 { !0 } else { 0 },
            if cond[6] != 0 { !0 } else { 0 },
            if cond[7] != 0 { !0 } else { 0 },
        ])
    }

    #[inline(always)]
    fn load_8888(
        format: PixelFormat,
        data: &[PremultipliedColorU8; STAGE_WIDTH],
        r: &mut f32x8,
        g: &mut f32x8,
        b: &mut f32x8,
        a: &mut f32x8,
    ) {
        // Surprisingly, `f32 * FACTOR` is way faster than `f32x8 * f32x8::splat(FACTOR)`.

        const FACTOR: f32 = 1.0 / 255.0;

        *r = f32x8::from([
            data[0].red() as f32 * FACTOR,
            data[1].red() as f32 * FACTOR,
            data[2].red() as f32 * FACTOR,
            data[3].red() as f32 * FACTOR,
            data[4].red() as f32 * FACTOR,
            data[5].red() as f32 * FACTOR,
            data[6].red() as f32 * FACTOR,
            data[7].red() as f32 * FACTOR,
        ]);

        *g = f32x8::from([
            data[0].green() as f32 * FACTOR,
            data[1].green() as f32 * FACTOR,
            data[2].green() as f32 * FACTOR,
            data[3].green() as f32 * FACTOR,
            data[4].green() as f32 * FACTOR,
            data[5].green() as f32 * FACTOR,
            data[6].green() as f32 * FACTOR,
            data[7].green() as f32 * FACTOR,
        ]);

        *b = f32x8::from([
            data[0].blue() as f32 * FACTOR,
            data[1].blue() as f32 * FACTOR,
            data[2].blue() as f32 * FACTOR,
            data[3].blue() as f32 * FACTOR,
            data[4].blue() as f32 * FACTOR,
            data[5].blue() as f32 * FACTOR,
            data[6].blue() as f32 * FACTOR,
            data[7].blue() as f32 * FACTOR,
        ]);

        *a = f32x8::from([
            data[0].alpha() as f32 * FACTOR,
            data[1].alpha() as f32 * FACTOR,
            data[2].alpha() as f32 * FACTOR,
            data[3].alpha() as f32 * FACTOR,
            data[4].alpha() as f32 * FACTOR,
            data[5].alpha() as f32 * FACTOR,
            data[6].alpha() as f32 * FACTOR,
            data[7].alpha() as f32 * FACTOR,
        ]);
        // Upstream load_8888[_dst] followed by swap_rb[_dst] and force_opaque[_dst].
        // The metadata selects these storage boundary stages; shader channels stay canonical.
        if format != PixelFormat::Rgba8888 {
            core::mem::swap(r, b);
        }
        if format == PixelFormat::Bgrx8888 {
            *a = f32x8::splat(1.0);
        }
    }

    #[inline(always)]
    fn load_8888_tail(
        format: PixelFormat,
        tail: usize,
        data: &[PremultipliedColorU8],
        r: &mut f32x8,
        g: &mut f32x8,
        b: &mut f32x8,
        a: &mut f32x8,
    ) {
        // Fill a dummy array with `tail` values. `tail` is always in a 1..STAGE_WIDTH-1 range.
        // This way we can reuse the `load_8888_` method and remove any branches.
        let mut tmp = [PremultipliedColorU8::TRANSPARENT; STAGE_WIDTH];
        tmp[0..tail].copy_from_slice(&data[0..tail]);
        load_8888(format, &tmp, r, g, b, a);
    }

    #[inline(always)]
    fn store_8888(
        format: PixelFormat,
        r: &f32x8,
        g: &f32x8,
        b: &f32x8,
        a: &f32x8,
        data: &mut [PremultipliedColorU8; STAGE_WIDTH],
    ) {
        // Upstream swap_rb + store_8888 at the storage boundary. BGRX's zero unused
        // byte is a local opaque-window storage adaptation, not a Skia pixel color type.
        let (r, b) = if format == PixelFormat::Rgba8888 {
            (r, b)
        } else {
            (b, r)
        };
        let unused = f32x8::default();
        let a = if format == PixelFormat::Bgrx8888 {
            &unused
        } else {
            a
        };

        let r: [i32; 8] = unnorm(r).into();
        let g: [i32; 8] = unnorm(g).into();
        let b: [i32; 8] = unnorm(b).into();
        let a: [i32; 8] = unnorm(a).into();

        let conv = |rr, gg, bb, aa| {
            PremultipliedColorU8::from_rgba_unchecked(rr as u8, gg as u8, bb as u8, aa as u8)
        };

        data[0] = conv(r[0], g[0], b[0], a[0]);
        data[1] = conv(r[1], g[1], b[1], a[1]);
        data[2] = conv(r[2], g[2], b[2], a[2]);
        data[3] = conv(r[3], g[3], b[3], a[3]);
        data[4] = conv(r[4], g[4], b[4], a[4]);
        data[5] = conv(r[5], g[5], b[5], a[5]);
        data[6] = conv(r[6], g[6], b[6], a[6]);
        data[7] = conv(r[7], g[7], b[7], a[7]);
    }

    #[inline(always)]
    fn store_8888_tail(
        format: PixelFormat,
        r: &f32x8,
        g: &f32x8,
        b: &f32x8,
        a: &f32x8,
        tail: usize,
        data: &mut [PremultipliedColorU8],
    ) {
        // Upstream swap_rb + store_8888 at the storage boundary. BGRX's zero unused
        // byte is a local opaque-window storage adaptation, not a Skia pixel color type.
        let (r, b) = if format == PixelFormat::Rgba8888 {
            (r, b)
        } else {
            (b, r)
        };
        let unused = f32x8::default();
        let a = if format == PixelFormat::Bgrx8888 {
            &unused
        } else {
            a
        };

        let r: [i32; 8] = unnorm(r).into();
        let g: [i32; 8] = unnorm(g).into();
        let b: [i32; 8] = unnorm(b).into();
        let a: [i32; 8] = unnorm(a).into();

        // This is better than `for i in 0..tail`, because this way the compiler
        // knows that we have only 4 steps and slices access is guarantee to be valid.
        // This removes bounds checking and a possible panic call.
        for i in 0..STAGE_WIDTH {
            data[i] = PremultipliedColorU8::from_rgba_unchecked(
                r[i] as u8, g[i] as u8, b[i] as u8, a[i] as u8,
            );

            if i + 1 == tail {
                break;
            }
        }
    }

    #[inline(always)]
    fn unnorm(v: &f32x8) -> i32x8 {
        (v.max(f32x8::default()).min(f32x8::splat(1.0)) * f32x8::splat(255.0)).round_int()
    }

    #[inline(always)]
    fn inv(v: f32x8) -> f32x8 {
        f32x8::splat(1.0) - v
    }

    #[inline(always)]
    fn two(v: f32x8) -> f32x8 {
        v + v
    }

    #[inline(always)]
    fn mad(f: f32x8, m: f32x8, a: f32x8) -> f32x8 {
        f * m + a
    }

    #[inline(always)]
    fn lerp(from: f32x8, to: f32x8, t: f32x8) -> f32x8 {
        mad(to - from, t, from)
    }

    // Migrated unchanged in behavior from tiny-skia-0.12.0/src/pipeline/lowp.rs.
    pub mod lowp {
        // Copyright 2018 Google Inc.
        // Copyright 2020 Yevhenii Reizner
        //
        // Use of this source code is governed by a BSD-style license that can be
        // found in the LICENSE file.

        /*
        A low precision raster pipeline implementation.

        A lowp pipeline uses u16 instead of f32 for math.
        Because of that, it doesn't implement stages that require high precision.
        The pipeline compiler will automatically decide which one to use.

        Skia uses u16x8 (128bit) types for a generic CPU and u16x16 (256bit) for modern x86 CPUs.
        But instead of explicit SIMD instructions, it mainly relies on clang's vector extensions.
        And since they are unavailable in Rust, we have to do everything manually.

        According to our benchmarks, a SIMD-accelerated u16x8 in Rust is almost 2x slower than in Skia.
        Not sure why. For example, there are no div instruction for u16x8, so we have to use
        a basic scalar version. Which means unnecessary load/store. No idea what clang does in this case.
        Surprisingly, a SIMD-accelerated u16x8 is even slower than a scalar one. Again, not sure why.

        Therefore we are using scalar u16x16 by default and relying on rustc/llvm auto vectorization instead.
        When targeting a generic CPU, we're just 5-10% slower than Skia. While u16x8 is 30-40% slower.
        And while `-C target-cpu=haswell` boosts our performance by around 25%,
        we are still 40-60% behind Skia built for Haswell.

        On ARM AArch64 the story is different and explicit SIMD make our code up to 2-3x faster.
        */

        use crate::raster::PremultipliedColorU8;
        use crate::PixelFormat;

        use crate::raster::geom::ScreenIntRect;
        use crate::raster::pixmap::SubPixmapMut;
        use crate::raster::wide::{f32x16, f32x8, u16x16};

        pub const STAGE_WIDTH: usize = 16;

        pub type StageFn = fn(p: &mut Pipeline);

        pub struct Pipeline<'a, 'b: 'a> {
            index: usize,
            functions: &'a [StageFn],
            pixmap: &'a mut SubPixmapMut<'b>,
            mask_ctx: crate::raster::pipeline::MaskCtx<'a>,
            aa_mask_ctx: crate::raster::pipeline::AAMaskCtx,
            ctx: &'a mut crate::raster::pipeline::Context,
            r: u16x16,
            g: u16x16,
            b: u16x16,
            a: u16x16,
            dr: u16x16,
            dg: u16x16,
            db: u16x16,
            da: u16x16,
            tail: usize,
            dx: usize,
            dy: usize,
        }

        impl Pipeline<'_, '_> {
            #[inline(always)]
            fn next_stage(&mut self) {
                let next: fn(&mut Self) = self.functions[self.index];
                self.index += 1;
                next(self);
            }
        }

        // Must be in the same order as raster_pipeline::Stage
        pub const STAGES: &[StageFn; crate::raster::pipeline::STAGES_COUNT] = &[
            move_src_dst,
            move_dst_src,
            null_fn, // Clamp0
            null_fn, // ClampA
            premul,
            uniform_color,
            seed_shader,
            load_dst,
            store,
            load_dst_u8,
            store_u8,
            null_fn, // Gather
            load_mask_u8,
            mask_u8,
            scale_u8,
            lerp_u8,
            scale_1_float,
            lerp_1_float,
            dstatop,
            dstin,
            dstout,
            dstover,
            srcatop,
            srcin,
            srcout,
            srcover,
            clear,
            modulate,
            multiply,
            plus,
            screen,
            xor,
            null_fn, // ColorBurn
            null_fn, // ColorDodge
            darken,
            difference,
            exclusion,
            hardlight,
            lighten,
            overlay,
            null_fn, // SoftLight
            null_fn, // Hue
            null_fn, // Saturation
            null_fn, // Color
            null_fn, // Luminosity
            srcover_rgba_8888,
            matrix_2x3,
            null_fn, // Reflect
            null_fn, // Repeat
            null_fn, // Bilinear
            null_fn, // Bicubic
            clamp_x_1,
            mirror_x_1,
            repeat_x_1,
            gradient,
            evenly_spaced_2_stop_gradient,
            // TODO: Can be implemented for lowp as well. The implementation is very similar to its highp
            // variant.
            null_fn, // XYToUnitAngle
            xy_to_radius,
            null_fn, // XYTo2PtConicalFocalOnCircle
            null_fn, // XYTo2PtConicalWellBehaved
            null_fn, // XYTo2PtConicalSmaller
            null_fn, // XYTo2PtConicalGreater
            null_fn, // XYTo2PtConicalStrip
            null_fn, // Mask2PtConicalNan
            null_fn, // Mask2PtConicalDegenerates
            null_fn, // ApplyVectorMask
            null_fn, // Alter2PtConicalCompensateFocal
            null_fn, // Alter2PtConicalUnswap
            null_fn, // NegateX
            null_fn, // ApplyConcentricScaleBias
            null_fn, // GammaExpand2
            null_fn, // GammaExpandDestination2
            null_fn, // GammaCompress2
            null_fn, // GammaExpand22
            null_fn, // GammaExpandDestination22
            null_fn, // GammaCompress22
            null_fn, // GammaExpandSrgb
            null_fn, // GammaExpandDestinationSrgb
            null_fn, // GammaCompressSrgb
        ];

        pub fn fn_ptr(f: StageFn) -> *const () {
            f as *const ()
        }

        pub fn fn_ptr_eq(f1: StageFn, f2: StageFn) -> bool {
            core::ptr::eq(f1 as *const (), f2 as *const ())
        }

        #[inline(never)]
        pub fn start(
            functions: &[StageFn],
            functions_tail: &[StageFn],
            rect: &ScreenIntRect,
            aa_mask_ctx: crate::raster::pipeline::AAMaskCtx,
            mask_ctx: crate::raster::pipeline::MaskCtx,
            ctx: &mut crate::raster::pipeline::Context,
            pixmap: &mut SubPixmapMut,
        ) {
            let mut p = Pipeline {
                index: 0,
                functions: &[],
                pixmap,
                mask_ctx,
                aa_mask_ctx,
                ctx,
                r: u16x16::default(),
                g: u16x16::default(),
                b: u16x16::default(),
                a: u16x16::default(),
                dr: u16x16::default(),
                dg: u16x16::default(),
                db: u16x16::default(),
                da: u16x16::default(),
                tail: 0,
                dx: 0,
                dy: 0,
            };

            for y in rect.y()..rect.bottom() {
                let mut x = rect.x() as usize;
                let end = rect.right() as usize;

                p.functions = functions;
                while x + STAGE_WIDTH <= end {
                    p.index = 0;
                    p.dx = x;
                    p.dy = y as usize;
                    p.tail = STAGE_WIDTH;
                    p.next_stage();
                    x += STAGE_WIDTH;
                }

                if x != end {
                    p.index = 0;
                    p.functions = functions_tail;
                    p.dx = x;
                    p.dy = y as usize;
                    p.tail = end - x;
                    p.next_stage();
                }
            }
        }

        fn move_src_dst(p: &mut Pipeline) {
            p.dr = p.r;
            p.dg = p.g;
            p.db = p.b;
            p.da = p.a;

            p.next_stage();
        }

        fn move_dst_src(p: &mut Pipeline) {
            p.r = p.dr;
            p.g = p.dg;
            p.b = p.db;
            p.a = p.da;

            p.next_stage();
        }

        fn premul(p: &mut Pipeline) {
            p.r = div255(p.r * p.a);
            p.g = div255(p.g * p.a);
            p.b = div255(p.b * p.a);

            p.next_stage();
        }

        fn uniform_color(p: &mut Pipeline) {
            let ctx = p.ctx.uniform_color;
            p.r = u16x16::splat(ctx.rgba[0]);
            p.g = u16x16::splat(ctx.rgba[1]);
            p.b = u16x16::splat(ctx.rgba[2]);
            p.a = u16x16::splat(ctx.rgba[3]);

            p.next_stage();
        }

        fn seed_shader(p: &mut Pipeline) {
            let iota = f32x16(
                f32x8::from([0.5, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5]),
                f32x8::from([8.5, 9.5, 10.5, 11.5, 12.5, 13.5, 14.5, 15.5]),
            );

            let x = f32x16::splat(p.dx as f32) + iota;
            let y = f32x16::splat(p.dy as f32 + 0.5);
            split(&x, &mut p.r, &mut p.g);
            split(&y, &mut p.b, &mut p.a);

            p.next_stage();
        }

        pub fn load_dst(p: &mut Pipeline) {
            let format = p.pixmap.format;
            load_8888(
                format,
                p.pixmap.slice16_at_xy(p.dx, p.dy),
                &mut p.dr,
                &mut p.dg,
                &mut p.db,
                &mut p.da,
            );
            p.next_stage();
        }

        pub fn load_dst_tail(p: &mut Pipeline) {
            let format = p.pixmap.format;
            load_8888_tail(
                format,
                p.tail,
                p.pixmap.slice_at_xy(p.dx, p.dy),
                &mut p.dr,
                &mut p.dg,
                &mut p.db,
                &mut p.da,
            );
            p.next_stage();
        }

        pub fn store(p: &mut Pipeline) {
            let format = p.pixmap.format;
            store_8888(
                format,
                &p.r,
                &p.g,
                &p.b,
                &p.a,
                p.pixmap.slice16_at_xy(p.dx, p.dy),
            );
            p.next_stage();
        }

        pub fn store_tail(p: &mut Pipeline) {
            let format = p.pixmap.format;
            store_8888_tail(
                format,
                &p.r,
                &p.g,
                &p.b,
                &p.a,
                p.tail,
                p.pixmap.slice_at_xy(p.dx, p.dy),
            );
            p.next_stage();
        }

        pub fn load_dst_u8(p: &mut Pipeline) {
            load_8(p.pixmap.slice16_mask_at_xy(p.dx, p.dy), &mut p.da);
            p.next_stage();
        }

        pub fn load_dst_u8_tail(p: &mut Pipeline) {
            // Fill a dummy array with `tail` values. `tail` is always in a 1..STAGE_WIDTH-1 range.
            // This way we can reuse the `load_8888__` method and remove any branches.
            let data = p.pixmap.slice_mask_at_xy(p.dx, p.dy);
            let mut tmp = [0u8; STAGE_WIDTH];
            tmp[0..p.tail].copy_from_slice(&data[0..p.tail]);
            load_8(&tmp, &mut p.da);

            p.next_stage();
        }

        pub fn store_u8(p: &mut Pipeline) {
            let data = p.pixmap.slice16_mask_at_xy(p.dx, p.dy);
            let a = p.a.as_slice();

            data[0] = a[0] as u8;
            data[1] = a[1] as u8;
            data[2] = a[2] as u8;
            data[3] = a[3] as u8;
            data[4] = a[4] as u8;
            data[5] = a[5] as u8;
            data[6] = a[6] as u8;
            data[7] = a[7] as u8;
            data[8] = a[8] as u8;
            data[9] = a[9] as u8;
            data[10] = a[10] as u8;
            data[11] = a[11] as u8;
            data[12] = a[12] as u8;
            data[13] = a[13] as u8;
            data[14] = a[14] as u8;
            data[15] = a[15] as u8;

            p.next_stage();
        }

        pub fn store_u8_tail(p: &mut Pipeline) {
            let data = p.pixmap.slice_mask_at_xy(p.dx, p.dy);
            let a = p.a.as_slice();

            // This is better than `for i in 0..tail`, because this way the compiler
            // knows that we have only 16 steps and slices access is guarantee to be valid.
            // This removes bounds checking and a possible panic call.
            for i in 0..STAGE_WIDTH {
                data[i] = a[i] as u8;

                if i + 1 == p.tail {
                    break;
                }
            }

            p.next_stage();
        }

        // Similar to mask_u8, but only loads the mask values without actually masking the pipeline.
        fn load_mask_u8(p: &mut Pipeline) {
            let offset = p.mask_ctx.offset(p.dx, p.dy);

            let mut c = u16x16::default();
            for i in 0..p.tail {
                c.0[i] = u16::from(p.mask_ctx.data[offset + i]);
            }

            p.r = u16x16::splat(0);
            p.g = u16x16::splat(0);
            p.b = u16x16::splat(0);
            p.a = c;

            p.next_stage();
        }

        fn mask_u8(p: &mut Pipeline) {
            let offset = p.mask_ctx.offset(p.dx, p.dy);

            let mut c = u16x16::default();
            for i in 0..p.tail {
                c.0[i] = u16::from(p.mask_ctx.data[offset + i]);
            }

            if c == u16x16::default() {
                return;
            }

            p.r = div255(p.r * c);
            p.g = div255(p.g * c);
            p.b = div255(p.b * c);
            p.a = div255(p.a * c);

            p.next_stage();
        }

        fn scale_u8(p: &mut Pipeline) {
            // Load u8xTail and cast it to u16x16.
            let data = p.aa_mask_ctx.copy_at_xy(p.dx, p.dy, p.tail);
            let c = u16x16([
                u16::from(data[0]),
                u16::from(data[1]),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]);

            p.r = div255(p.r * c);
            p.g = div255(p.g * c);
            p.b = div255(p.b * c);
            p.a = div255(p.a * c);

            p.next_stage();
        }

        fn lerp_u8(p: &mut Pipeline) {
            // Load u8xTail and cast it to u16x16.
            let data = p.aa_mask_ctx.copy_at_xy(p.dx, p.dy, p.tail);
            let c = u16x16([
                u16::from(data[0]),
                u16::from(data[1]),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]);

            p.r = lerp(p.dr, p.r, c);
            p.g = lerp(p.dg, p.g, c);
            p.b = lerp(p.db, p.b, c);
            p.a = lerp(p.da, p.a, c);

            p.next_stage();
        }

        fn scale_1_float(p: &mut Pipeline) {
            let c = from_float(p.ctx.current_coverage);
            p.r = div255(p.r * c);
            p.g = div255(p.g * c);
            p.b = div255(p.b * c);
            p.a = div255(p.a * c);

            p.next_stage();
        }

        fn lerp_1_float(p: &mut Pipeline) {
            let c = from_float(p.ctx.current_coverage);
            p.r = lerp(p.dr, p.r, c);
            p.g = lerp(p.dg, p.g, c);
            p.b = lerp(p.db, p.b, c);
            p.a = lerp(p.da, p.a, c);

            p.next_stage();
        }

        macro_rules! blend_fn {
            ($name:ident, $f:expr) => {
                fn $name(p: &mut Pipeline) {
                    p.r = $f(p.r, p.dr, p.a, p.da);
                    p.g = $f(p.g, p.dg, p.a, p.da);
                    p.b = $f(p.b, p.db, p.a, p.da);
                    p.a = $f(p.a, p.da, p.a, p.da);

                    p.next_stage();
                }
            };
        }

        blend_fn!(clear, |_, _, _, _| u16x16::splat(0));
        blend_fn!(srcatop, |s, d, sa, da| div255(s * da + d * inv(sa)));
        blend_fn!(dstatop, |s, d, sa, da| div255(d * sa + s * inv(da)));
        blend_fn!(srcin, |s, _, _, da| div255(s * da));
        blend_fn!(dstin, |_, d, sa, _| div255(d * sa));
        blend_fn!(srcout, |s, _, _, da| div255(s * inv(da)));
        blend_fn!(dstout, |_, d, sa, _| div255(d * inv(sa)));
        blend_fn!(srcover, |s, d, sa, _| s + div255(d * inv(sa)));
        blend_fn!(dstover, |s, d, _, da| d + div255(s * inv(da)));
        blend_fn!(modulate, |s, d, _, _| div255(s * d));
        blend_fn!(multiply, |s, d, sa, da| div255(
            s * inv(da) + d * inv(sa) + s * d
        ));
        blend_fn!(screen, |s, d, _, _| s + d - div255(s * d));
        blend_fn!(xor, |s, d, sa, da| div255(s * inv(da) + d * inv(sa)));

        // Wants a type for some reason.
        blend_fn!(plus, |s: u16x16, d, _, _| (s + d).min(&u16x16::splat(255)));

        macro_rules! blend_fn2 {
            ($name:ident, $f:expr) => {
                fn $name(p: &mut Pipeline) {
                    // The same logic applied to color, and srcover for alpha.
                    p.r = $f(p.r, p.dr, p.a, p.da);
                    p.g = $f(p.g, p.dg, p.a, p.da);
                    p.b = $f(p.b, p.db, p.a, p.da);
                    p.a = p.a + div255(p.da * inv(p.a));

                    p.next_stage();
                }
            };
        }

        blend_fn2!(darken, |s: u16x16, d, sa, da| s + d
            - div255((s * da).max(&(d * sa))));
        blend_fn2!(lighten, |s: u16x16, d, sa, da| s + d
            - div255((s * da).min(&(d * sa))));
        blend_fn2!(exclusion, |s: u16x16, d, _, _| s + d
            - u16x16::splat(2) * div255(s * d));

        blend_fn2!(difference, |s: u16x16, d, sa, da| s + d
            - u16x16::splat(2) * div255((s * da).min(&(d * sa))));

        blend_fn2!(hardlight, |s: u16x16, d: u16x16, sa, da| {
            div255(
                s * inv(da)
                    + d * inv(sa)
                    + (s + s).cmp_le(&sa).blend(
                        u16x16::splat(2) * s * d,
                        sa * da - u16x16::splat(2) * (sa - s) * (da - d),
                    ),
            )
        });

        blend_fn2!(overlay, |s: u16x16, d: u16x16, sa, da| {
            div255(
                s * inv(da)
                    + d * inv(sa)
                    + (d + d).cmp_le(&da).blend(
                        u16x16::splat(2) * s * d,
                        sa * da - u16x16::splat(2) * (sa - s) * (da - d),
                    ),
            )
        });

        pub fn srcover_rgba_8888(p: &mut Pipeline) {
            let format = p.pixmap.format;
            let pixels = p.pixmap.slice16_at_xy(p.dx, p.dy);
            load_8888(format, pixels, &mut p.dr, &mut p.dg, &mut p.db, &mut p.da);
            p.r = p.r + div255(p.dr * inv(p.a));
            p.g = p.g + div255(p.dg * inv(p.a));
            p.b = p.b + div255(p.db * inv(p.a));
            p.a = p.a + div255(p.da * inv(p.a));
            store_8888(format, &p.r, &p.g, &p.b, &p.a, pixels);

            p.next_stage();
        }

        pub fn srcover_rgba_8888_tail(p: &mut Pipeline) {
            let format = p.pixmap.format;
            let pixels = p.pixmap.slice_at_xy(p.dx, p.dy);
            load_8888_tail(
                format, p.tail, pixels, &mut p.dr, &mut p.dg, &mut p.db, &mut p.da,
            );
            p.r = p.r + div255(p.dr * inv(p.a));
            p.g = p.g + div255(p.dg * inv(p.a));
            p.b = p.b + div255(p.db * inv(p.a));
            p.a = p.a + div255(p.da * inv(p.a));
            store_8888_tail(format, &p.r, &p.g, &p.b, &p.a, p.tail, pixels);

            p.next_stage();
        }

        fn matrix_2x3(p: &mut Pipeline) {
            let ts = &p.ctx.transform;

            let x = join(&p.r, &p.g);
            let y = join(&p.b, &p.a);

            let nx = mad(
                x,
                f32x16::splat(ts.sx),
                mad(y, f32x16::splat(ts.kx), f32x16::splat(ts.tx)),
            );
            let ny = mad(
                x,
                f32x16::splat(ts.ky),
                mad(y, f32x16::splat(ts.sy), f32x16::splat(ts.ty)),
            );

            split(&nx, &mut p.r, &mut p.g);
            split(&ny, &mut p.b, &mut p.a);

            p.next_stage();
        }

        fn clamp_x_1(p: &mut Pipeline) {
            let x = join(&p.r, &p.g);
            let x = x.normalize();
            split(&x, &mut p.r, &mut p.g);

            p.next_stage();
        }

        fn mirror_x_1(p: &mut Pipeline) {
            let x = join(&p.r, &p.g);
            let two = |x| x + x;
            let x = ((x - f32x16::splat(1.0))
                - two(((x - f32x16::splat(1.0)) * f32x16::splat(0.5)).floor())
                - f32x16::splat(1.0))
            .abs()
            .normalize();
            split(&x, &mut p.r, &mut p.g);

            p.next_stage();
        }

        fn repeat_x_1(p: &mut Pipeline) {
            let x = join(&p.r, &p.g);
            let x = (x - x.floor()).normalize();
            split(&x, &mut p.r, &mut p.g);

            p.next_stage();
        }

        fn gradient(p: &mut Pipeline) {
            let ctx = &p.ctx.gradient;

            // N.B. The loop starts at 1 because idx 0 is the color to use before the first stop.
            let t = join(&p.r, &p.g);
            let mut idx = u16x16::splat(0);
            for i in 1..ctx.len {
                let tt = ctx.t_values[i].get();
                let t0: [f32; 8] = t.0.into();
                let t1: [f32; 8] = t.1.into();
                idx.0[0] += (t0[0] >= tt) as u16;
                idx.0[1] += (t0[1] >= tt) as u16;
                idx.0[2] += (t0[2] >= tt) as u16;
                idx.0[3] += (t0[3] >= tt) as u16;
                idx.0[4] += (t0[4] >= tt) as u16;
                idx.0[5] += (t0[5] >= tt) as u16;
                idx.0[6] += (t0[6] >= tt) as u16;
                idx.0[7] += (t0[7] >= tt) as u16;
                idx.0[8] += (t1[0] >= tt) as u16;
                idx.0[9] += (t1[1] >= tt) as u16;
                idx.0[10] += (t1[2] >= tt) as u16;
                idx.0[11] += (t1[3] >= tt) as u16;
                idx.0[12] += (t1[4] >= tt) as u16;
                idx.0[13] += (t1[5] >= tt) as u16;
                idx.0[14] += (t1[6] >= tt) as u16;
                idx.0[15] += (t1[7] >= tt) as u16;
            }
            gradient_lookup(ctx, &idx, t, &mut p.r, &mut p.g, &mut p.b, &mut p.a);

            p.next_stage();
        }

        fn evenly_spaced_2_stop_gradient(p: &mut Pipeline) {
            let ctx = &p.ctx.evenly_spaced_2_stop_gradient;

            let t = join(&p.r, &p.g);
            round_f32_to_u16(
                mad(t, f32x16::splat(ctx.factor.r), f32x16::splat(ctx.bias.r)),
                mad(t, f32x16::splat(ctx.factor.g), f32x16::splat(ctx.bias.g)),
                mad(t, f32x16::splat(ctx.factor.b), f32x16::splat(ctx.bias.b)),
                mad(t, f32x16::splat(ctx.factor.a), f32x16::splat(ctx.bias.a)),
                &mut p.r,
                &mut p.g,
                &mut p.b,
                &mut p.a,
            );

            p.next_stage();
        }

        fn xy_to_radius(p: &mut Pipeline) {
            let x = join(&p.r, &p.g);
            let y = join(&p.b, &p.a);
            let x = (x * x + y * y).sqrt();
            split(&x, &mut p.r, &mut p.g);
            split(&y, &mut p.b, &mut p.a);

            p.next_stage();
        }

        // We are using u16 for index, not u32 as Skia, to simplify the code a bit.
        // The gradient creation code will not allow that many stops anyway.
        fn gradient_lookup(
            ctx: &crate::raster::pipeline::GradientCtx,
            idx: &u16x16,
            t: f32x16,
            r: &mut u16x16,
            g: &mut u16x16,
            b: &mut u16x16,
            a: &mut u16x16,
        ) {
            macro_rules! gather {
                ($d:expr, $c:ident) => {
                    // Surprisingly, but bound checking doesn't affect the performance.
                    // And since `idx` can contain any number, we should leave it in place.
                    f32x16(
                        f32x8::from([
                            $d[idx.0[0] as usize].$c,
                            $d[idx.0[1] as usize].$c,
                            $d[idx.0[2] as usize].$c,
                            $d[idx.0[3] as usize].$c,
                            $d[idx.0[4] as usize].$c,
                            $d[idx.0[5] as usize].$c,
                            $d[idx.0[6] as usize].$c,
                            $d[idx.0[7] as usize].$c,
                        ]),
                        f32x8::from([
                            $d[idx.0[8] as usize].$c,
                            $d[idx.0[9] as usize].$c,
                            $d[idx.0[10] as usize].$c,
                            $d[idx.0[11] as usize].$c,
                            $d[idx.0[12] as usize].$c,
                            $d[idx.0[13] as usize].$c,
                            $d[idx.0[14] as usize].$c,
                            $d[idx.0[15] as usize].$c,
                        ]),
                    )
                };
            }

            let fr = gather!(&ctx.factors, r);
            let fg = gather!(&ctx.factors, g);
            let fb = gather!(&ctx.factors, b);
            let fa = gather!(&ctx.factors, a);

            let br = gather!(&ctx.biases, r);
            let bg = gather!(&ctx.biases, g);
            let bb = gather!(&ctx.biases, b);
            let ba = gather!(&ctx.biases, a);

            round_f32_to_u16(
                mad(t, fr, br),
                mad(t, fg, bg),
                mad(t, fb, bb),
                mad(t, fa, ba),
                r,
                g,
                b,
                a,
            );
        }

        #[inline(always)]
        fn round_f32_to_u16(
            rf: f32x16,
            gf: f32x16,
            bf: f32x16,
            af: f32x16,
            r: &mut u16x16,
            g: &mut u16x16,
            b: &mut u16x16,
            a: &mut u16x16,
        ) {
            // TODO: may produce a slightly different result to Skia
            //       affects the two_stops_linear_mirror test

            let rf = rf.normalize() * f32x16::splat(255.0) + f32x16::splat(0.5);
            let gf = gf.normalize() * f32x16::splat(255.0) + f32x16::splat(0.5);
            let bf = bf.normalize() * f32x16::splat(255.0) + f32x16::splat(0.5);
            let af = af * f32x16::splat(255.0) + f32x16::splat(0.5);

            rf.save_to_u16x16(r);
            gf.save_to_u16x16(g);
            bf.save_to_u16x16(b);
            af.save_to_u16x16(a);
        }

        pub fn just_return(_: &mut Pipeline) {
            // Ends the loop.
        }

        pub fn null_fn(_: &mut Pipeline) {
            // Just for unsupported functions in STAGES.
        }

        #[inline(always)]
        fn load_8888(
            format: PixelFormat,
            data: &[PremultipliedColorU8; STAGE_WIDTH],
            r: &mut u16x16,
            g: &mut u16x16,
            b: &mut u16x16,
            a: &mut u16x16,
        ) {
            *r = u16x16([
                data[0].red() as u16,
                data[1].red() as u16,
                data[2].red() as u16,
                data[3].red() as u16,
                data[4].red() as u16,
                data[5].red() as u16,
                data[6].red() as u16,
                data[7].red() as u16,
                data[8].red() as u16,
                data[9].red() as u16,
                data[10].red() as u16,
                data[11].red() as u16,
                data[12].red() as u16,
                data[13].red() as u16,
                data[14].red() as u16,
                data[15].red() as u16,
            ]);

            *g = u16x16([
                data[0].green() as u16,
                data[1].green() as u16,
                data[2].green() as u16,
                data[3].green() as u16,
                data[4].green() as u16,
                data[5].green() as u16,
                data[6].green() as u16,
                data[7].green() as u16,
                data[8].green() as u16,
                data[9].green() as u16,
                data[10].green() as u16,
                data[11].green() as u16,
                data[12].green() as u16,
                data[13].green() as u16,
                data[14].green() as u16,
                data[15].green() as u16,
            ]);

            *b = u16x16([
                data[0].blue() as u16,
                data[1].blue() as u16,
                data[2].blue() as u16,
                data[3].blue() as u16,
                data[4].blue() as u16,
                data[5].blue() as u16,
                data[6].blue() as u16,
                data[7].blue() as u16,
                data[8].blue() as u16,
                data[9].blue() as u16,
                data[10].blue() as u16,
                data[11].blue() as u16,
                data[12].blue() as u16,
                data[13].blue() as u16,
                data[14].blue() as u16,
                data[15].blue() as u16,
            ]);

            *a = u16x16([
                data[0].alpha() as u16,
                data[1].alpha() as u16,
                data[2].alpha() as u16,
                data[3].alpha() as u16,
                data[4].alpha() as u16,
                data[5].alpha() as u16,
                data[6].alpha() as u16,
                data[7].alpha() as u16,
                data[8].alpha() as u16,
                data[9].alpha() as u16,
                data[10].alpha() as u16,
                data[11].alpha() as u16,
                data[12].alpha() as u16,
                data[13].alpha() as u16,
                data[14].alpha() as u16,
                data[15].alpha() as u16,
            ]);
            // Upstream load_8888[_dst] followed by swap_rb[_dst] and force_opaque[_dst].
            // The metadata selects these storage boundary stages; shader channels stay canonical.
            if format != PixelFormat::Rgba8888 {
                core::mem::swap(r, b);
            }
            if format == PixelFormat::Bgrx8888 {
                *a = u16x16::splat(255);
            }
        }

        #[inline(always)]
        fn load_8888_tail(
            format: PixelFormat,
            tail: usize,
            data: &[PremultipliedColorU8],
            r: &mut u16x16,
            g: &mut u16x16,
            b: &mut u16x16,
            a: &mut u16x16,
        ) {
            // Fill a dummy array with `tail` values. `tail` is always in a 1..STAGE_WIDTH-1 range.
            // This way we can reuse the `load_8888__` method and remove any branches.
            let mut tmp = [PremultipliedColorU8::TRANSPARENT; STAGE_WIDTH];
            tmp[0..tail].copy_from_slice(&data[0..tail]);
            load_8888(format, &tmp, r, g, b, a);
        }

        #[inline(always)]
        fn store_8888(
            format: PixelFormat,
            r: &u16x16,
            g: &u16x16,
            b: &u16x16,
            a: &u16x16,
            data: &mut [PremultipliedColorU8; STAGE_WIDTH],
        ) {
            // Upstream swap_rb + store_8888 at the storage boundary. BGRX's zero unused
            // byte is a local opaque-window storage adaptation, not a Skia pixel color type.
            let (r, b) = if format == PixelFormat::Rgba8888 {
                (r, b)
            } else {
                (b, r)
            };
            let unused = u16x16::splat(0);
            let a = if format == PixelFormat::Bgrx8888 {
                &unused
            } else {
                a
            };

            let r = r.as_slice();
            let g = g.as_slice();
            let b = b.as_slice();
            let a = a.as_slice();

            data[0] = PremultipliedColorU8::from_rgba_unchecked(
                r[0] as u8, g[0] as u8, b[0] as u8, a[0] as u8,
            );
            data[1] = PremultipliedColorU8::from_rgba_unchecked(
                r[1] as u8, g[1] as u8, b[1] as u8, a[1] as u8,
            );
            data[2] = PremultipliedColorU8::from_rgba_unchecked(
                r[2] as u8, g[2] as u8, b[2] as u8, a[2] as u8,
            );
            data[3] = PremultipliedColorU8::from_rgba_unchecked(
                r[3] as u8, g[3] as u8, b[3] as u8, a[3] as u8,
            );
            data[4] = PremultipliedColorU8::from_rgba_unchecked(
                r[4] as u8, g[4] as u8, b[4] as u8, a[4] as u8,
            );
            data[5] = PremultipliedColorU8::from_rgba_unchecked(
                r[5] as u8, g[5] as u8, b[5] as u8, a[5] as u8,
            );
            data[6] = PremultipliedColorU8::from_rgba_unchecked(
                r[6] as u8, g[6] as u8, b[6] as u8, a[6] as u8,
            );
            data[7] = PremultipliedColorU8::from_rgba_unchecked(
                r[7] as u8, g[7] as u8, b[7] as u8, a[7] as u8,
            );
            data[8] = PremultipliedColorU8::from_rgba_unchecked(
                r[8] as u8, g[8] as u8, b[8] as u8, a[8] as u8,
            );
            data[9] = PremultipliedColorU8::from_rgba_unchecked(
                r[9] as u8, g[9] as u8, b[9] as u8, a[9] as u8,
            );
            data[10] = PremultipliedColorU8::from_rgba_unchecked(
                r[10] as u8,
                g[10] as u8,
                b[10] as u8,
                a[10] as u8,
            );
            data[11] = PremultipliedColorU8::from_rgba_unchecked(
                r[11] as u8,
                g[11] as u8,
                b[11] as u8,
                a[11] as u8,
            );
            data[12] = PremultipliedColorU8::from_rgba_unchecked(
                r[12] as u8,
                g[12] as u8,
                b[12] as u8,
                a[12] as u8,
            );
            data[13] = PremultipliedColorU8::from_rgba_unchecked(
                r[13] as u8,
                g[13] as u8,
                b[13] as u8,
                a[13] as u8,
            );
            data[14] = PremultipliedColorU8::from_rgba_unchecked(
                r[14] as u8,
                g[14] as u8,
                b[14] as u8,
                a[14] as u8,
            );
            data[15] = PremultipliedColorU8::from_rgba_unchecked(
                r[15] as u8,
                g[15] as u8,
                b[15] as u8,
                a[15] as u8,
            );
        }

        #[inline(always)]
        fn store_8888_tail(
            format: PixelFormat,
            r: &u16x16,
            g: &u16x16,
            b: &u16x16,
            a: &u16x16,
            tail: usize,
            data: &mut [PremultipliedColorU8],
        ) {
            // Upstream swap_rb + store_8888 at the storage boundary. BGRX's zero unused
            // byte is a local opaque-window storage adaptation, not a Skia pixel color type.
            let (r, b) = if format == PixelFormat::Rgba8888 {
                (r, b)
            } else {
                (b, r)
            };
            let unused = u16x16::splat(0);
            let a = if format == PixelFormat::Bgrx8888 {
                &unused
            } else {
                a
            };

            let r = r.as_slice();
            let g = g.as_slice();
            let b = b.as_slice();
            let a = a.as_slice();

            // This is better than `for i in 0..tail`, because this way the compiler
            // knows that we have only 16 steps and slices access is guarantee to be valid.
            // This removes bounds checking and a possible panic call.
            for i in 0..STAGE_WIDTH {
                data[i] = PremultipliedColorU8::from_rgba_unchecked(
                    r[i] as u8, g[i] as u8, b[i] as u8, a[i] as u8,
                );

                if i + 1 == tail {
                    break;
                }
            }
        }

        #[inline(always)]
        fn load_8(data: &[u8; STAGE_WIDTH], a: &mut u16x16) {
            *a = u16x16([
                data[0] as u16,
                data[1] as u16,
                data[2] as u16,
                data[3] as u16,
                data[4] as u16,
                data[5] as u16,
                data[6] as u16,
                data[7] as u16,
                data[8] as u16,
                data[9] as u16,
                data[10] as u16,
                data[11] as u16,
                data[12] as u16,
                data[13] as u16,
                data[14] as u16,
                data[15] as u16,
            ]);
        }

        #[inline(always)]
        fn div255(v: u16x16) -> u16x16 {
            // Skia uses `vrshrq_n_u16(vrsraq_n_u16(v, v, 8), 8)` here when NEON is available,
            // but it doesn't affect performance much and breaks reproducible result. Ignore it.
            // NOTE: the compiler does not replace the division with a shift.
            (v + u16x16::splat(255)) >> u16x16::splat(8) // / u16x16::splat(256)
        }

        #[inline(always)]
        fn inv(v: u16x16) -> u16x16 {
            u16x16::splat(255) - v
        }

        #[inline(always)]
        fn from_float(f: f32) -> u16x16 {
            u16x16::splat((f * 255.0 + 0.5) as u16)
        }

        #[inline(always)]
        fn lerp(from: u16x16, to: u16x16, t: u16x16) -> u16x16 {
            div255(from * inv(t) + to * t)
        }

        #[inline(always)]
        fn split(v: &f32x16, lo: &mut u16x16, hi: &mut u16x16) {
            // We're splitting f32x16 (512bit) into two u16x16 (256 bit).
            let data: [u8; 64] = bytemuck::cast(*v);
            let d0: &mut [u8; 32] = bytemuck::cast_mut(&mut lo.0);
            let d1: &mut [u8; 32] = bytemuck::cast_mut(&mut hi.0);

            d0.copy_from_slice(&data[0..32]);
            d1.copy_from_slice(&data[32..64]);
        }

        #[inline(always)]
        fn join(lo: &u16x16, hi: &u16x16) -> f32x16 {
            // We're joining two u16x16 (256 bit) into f32x16 (512bit).

            let d0: [u8; 32] = bytemuck::cast(lo.0);
            let d1: [u8; 32] = bytemuck::cast(hi.0);

            let mut v = f32x16::default();
            let data: &mut [u8; 64] = bytemuck::cast_mut(&mut v);

            data[0..32].copy_from_slice(&d0);
            data[32..64].copy_from_slice(&d1);

            v
        }

        #[inline(always)]
        fn mad(f: f32x16, m: f32x16, a: f32x16) -> f32x16 {
            // NEON vmlaq_f32 doesn't seem to affect performance in any way. Ignore it.
            f * m + a
        }
    }
}

#[cfg(test)]
mod pixel_format_tests {
    use crate::raster::geom::ScreenIntRect;
    use crate::raster::pipeline::{AAMaskCtx, MaskCtx, RasterPipelineBuilder, SkRasterPipelineOp};
    use crate::raster::{Color, PixmapMut, PixmapRef};
    use crate::PixelFormat;

    // Independent byte swizzle for the test oracle; shader inputs remain canonical RGBA.
    fn storage(rgba: &[u8], format: PixelFormat) -> Vec<u8> {
        let mut out = rgba.to_vec();
        if format != PixelFormat::Rgba8888 {
            for p in out.chunks_exact_mut(4) {
                p.swap(0, 2);
                if format == PixelFormat::Bgrx8888 {
                    p[3] = 0;
                }
            }
        }
        out
    }

    fn run(
        width: u32,
        initial: &[u8],
        dst_format: PixelFormat,
        src: &[u8],
        src_format: PixelFormat,
        highp: bool,
        mode: u8,
    ) -> Vec<u8> {
        let mut bytes = storage(initial, dst_format);
        let active = bytes.len();
        bytes.extend_from_slice(&[0x5a; 32]);
        let mut target = PixmapMut::from_bytes(&mut bytes, width, 2).unwrap();
        target.format = dst_format;
        let source_bytes = storage(src, src_format);
        let mut source = PixmapRef::from_bytes(&source_bytes, width, 2).unwrap();
        source.format = src_format;
        let mut p = RasterPipelineBuilder::new();
        p.set_force_hq_pipeline(highp);
        if mode <= 2 {
            p.push_uniform_color(Color::from_rgba8(220, 140, 75, 180).premultiply());
        } else {
            p.push(SkRasterPipelineOp::SeedShader);
            p.push(match mode {
                3 => SkRasterPipelineOp::Gather,
                4 => SkRasterPipelineOp::Bilinear,
                _ => SkRasterPipelineOp::Bicubic,
            });
        }
        match mode {
            0 => p.push(SkRasterPipelineOp::Store),
            2 => p.push(SkRasterPipelineOp::SourceOverRgba),
            _ => {
                p.push(SkRasterPipelineOp::LoadDestination);
                p.push(SkRasterPipelineOp::SourceOver);
                p.push(SkRasterPipelineOp::Store);
            }
        }
        p.compile().run(
            &ScreenIntRect::from_xywh(0, 0, width, 2).unwrap(),
            AAMaskCtx::default(),
            MaskCtx::default(),
            source,
            &mut target.as_subpixmap(),
        );
        assert_eq!(&bytes[active..], &[0x5a; 32], "tail wrote past the target");
        bytes.truncate(active);
        bytes
    }

    fn scene(width: u32, opaque: bool) -> Vec<u8> {
        (0..width * 2)
            .flat_map(|i| {
                let a = if opaque {
                    255
                } else {
                    40 + (i * 11 % 216) as u8
                };
                [
                    ((i * 31 + 3) % (a as u32 + 1)) as u8,
                    ((i * 17 + 21) % (a as u32 + 1)) as u8,
                    ((i * 7 + 5) % (a as u32 + 1)) as u8,
                    a,
                ]
            })
            .collect()
    }

    #[test]
    fn destination_formats_cover_highp_lowp_full_tail_and_fused_srcover() {
        for width in 1..=37 {
            for highp in [false, true] {
                for mode in 0..=2 {
                    for format in [PixelFormat::Bgra8888, PixelFormat::Bgrx8888] {
                        let initial = scene(width, format == PixelFormat::Bgrx8888);
                        let source = scene(width, false);
                        let rgba = run(
                            width,
                            &initial,
                            PixelFormat::Rgba8888,
                            &source,
                            PixelFormat::Rgba8888,
                            highp,
                            mode,
                        );
                        assert_eq!(
                            run(
                                width,
                                &initial,
                                format,
                                &source,
                                PixelFormat::Rgba8888,
                                highp,
                                mode
                            ),
                            storage(&rgba, format),
                            "width={width} highp={highp} mode={mode} format={format:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn source_gather_and_filtered_sampling_decode_format_before_shader_math() {
        for width in 1..=19 {
            for mode in 3..=5 {
                for src_format in [PixelFormat::Bgra8888, PixelFormat::Bgrx8888] {
                    let initial = scene(width, false);
                    let source = scene(width, src_format == PixelFormat::Bgrx8888);
                    let rgba = run(
                        width,
                        &initial,
                        PixelFormat::Rgba8888,
                        &source,
                        PixelFormat::Rgba8888,
                        true,
                        mode,
                    );
                    let actual = run(
                        width,
                        &initial,
                        PixelFormat::Bgra8888,
                        &source,
                        src_format,
                        true,
                        mode,
                    );
                    assert_eq!(
                        actual,
                        storage(&rgba, PixelFormat::Bgra8888),
                        "width={width} mode={mode} source={src_format:?}"
                    );
                }
            }
        }
    }
}
