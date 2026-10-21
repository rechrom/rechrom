#![cfg(target_vendor = "apple")]
use crate::src::core::{
    SkGlyph::SkGlyph,
    SkMask::Format,
    SkMatrix::SkMatrix,
    SkPathBuilder::SkPathBuilder,
    SkRect::SkRect,
    SkScalerContext::{GeneratedPath, GlyphMetrics, SkScalerContext},
};
use crate::src::ports::SkTypeface_mac_ct::*;
use crate::src::utils::mac::SkUniqueCFRef::SkUniqueCFRef;
use core::ffi::c_void;
// CoreGraphics SDK 64-bit CGFloat/geometry ABI; no platform geometry algorithms
// are replaced. Transform/bounds operations call the actual framework functions.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CGPoint {
    pub x: f64,
    pub y: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CGSize {
    pub width: f64,
    pub height: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CGRect {
    pub origin: CGPoint,
    pub size: CGSize,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CGAffineTransform {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub tx: f64,
    pub ty: f64,
}
#[repr(C)]
struct CGPathElement {
    kind: i32,
    points: *const CGPoint,
}
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGAffineTransformMake(a: f64, b: f64, c: f64, d: f64, tx: f64, ty: f64)
        -> CGAffineTransform;
    fn CGAffineTransformMakeScale(x: f64, y: f64) -> CGAffineTransform;
    fn CGAffineTransformConcat(a: CGAffineTransform, b: CGAffineTransform) -> CGAffineTransform;
    fn CGAffineTransformInvert(t: CGAffineTransform) -> CGAffineTransform;
    fn CGPointApplyAffineTransform(p: CGPoint, t: CGAffineTransform) -> CGPoint;
    fn CGSizeApplyAffineTransform(p: CGSize, t: CGAffineTransform) -> CGSize;
    fn CGRectApplyAffineTransform(p: CGRect, t: CGAffineTransform) -> CGRect;
    fn CGRectIsEmpty(rect: CGRect) -> bool;
    fn CGPathIsEmpty(path: *mut CGPath) -> bool;
    fn CGPathApply(
        path: *mut CGPath,
        ctx: *mut c_void,
        apply: unsafe extern "C" fn(*mut c_void, *const CGPathElement),
    );
    fn CGColorCreate(space: *mut CGColorSpace, components: *const f64) -> *mut CGColor;
    fn CGColorSpaceCreateDeviceRGB() -> *mut CGColorSpace;
    fn CGBitmapContextCreate(
        data: *mut c_void,
        width: usize,
        height: usize,
        bits: usize,
        row_bytes: usize,
        space: *mut CGColorSpace,
        bitmap_info: u32,
    ) -> *mut CGContext;
    fn CGContextSetAllowsFontSubpixelQuantization(ctx: *mut CGContext, enabled: bool);
    fn CGContextSetShouldSubpixelQuantizeFonts(ctx: *mut CGContext, enabled: bool);
    fn CGContextSetAllowsFontSubpixelPositioning(ctx: *mut CGContext, enabled: bool);
    fn CGContextSetShouldSubpixelPositionFonts(ctx: *mut CGContext, enabled: bool);
    fn CGContextSetTextDrawingMode(ctx: *mut CGContext, mode: i32);
    fn CGContextSetGrayFillColor(ctx: *mut CGContext, gray: f64, alpha: f64);
    fn CGContextSetFillColorWithColor(ctx: *mut CGContext, color: *mut CGColor);
    fn CGContextSetTextMatrix(ctx: *mut CGContext, matrix: CGAffineTransform);
    fn CGContextSetShouldAntialias(ctx: *mut CGContext, enabled: bool);
    fn CGContextSetShouldSmoothFonts(ctx: *mut CGContext, enabled: bool);
}
#[link(name = "CoreText", kind = "framework")]
unsafe extern "C" {
    fn CTFontCopyGraphicsFont(font: CTFontRef, attributes: *mut CTFontDescriptorRef)
        -> *mut CGFont;
    fn CTFontGetBoundingBox(font: CTFontRef) -> CGRect;
    fn CTFontGetAscent(font: CTFontRef) -> f64;
    fn CTFontGetDescent(font: CTFontRef) -> f64;
    fn CTFontGetLeading(font: CTFontRef) -> f64;
    fn CTFontGetXHeight(font: CTFontRef) -> f64;
    fn CTFontGetCapHeight(font: CTFontRef) -> f64;
    fn CTFontGetUnderlineThickness(font: CTFontRef) -> f64;
    fn CTFontGetUnderlinePosition(font: CTFontRef) -> f64;
    fn CTFontGetSize(font: CTFontRef) -> f64;
    fn CTFontGetUnitsPerEm(font: CTFontRef) -> u32;
    fn CTFontGetAdvancesForGlyphs(
        font: CTFontRef,
        orientation: u32,
        glyphs: *const u16,
        advances: *mut CGSize,
        count: isize,
    ) -> f64;
    fn CTFontGetBoundingRectsForGlyphs(
        font: CTFontRef,
        orientation: u32,
        glyphs: *const u16,
        bounds: *mut CGRect,
        count: isize,
    ) -> CGRect;
    fn CTFontCreatePathForGlyph(
        font: CTFontRef,
        glyph: u16,
        transform: *const CGAffineTransform,
    ) -> *mut CGPath;
    fn CTFontDrawGlyphs(
        font: CTFontRef,
        glyphs: *const u16,
        positions: *const CGPoint,
        count: usize,
        ctx: *mut CGContext,
    );
}
// Source: src/ports/SkScalerContext_mac_ct.h:40-40
pub type CGRGBPixel = u32;
// Source: src/ports/SkScalerContext_mac_ct.h:75-104
// Owning fields are reversed to retain C++ member destruction order. The actual
// base owns its native non-owning Typeface lease. Actual font sizing calls the
// shared SkCTFontCreateExactCopy helper; no synthetic font is supplied.
pub struct SkScalerContext_Mac {
    fCGFont: SkUniqueCFRef<CGFont>,
    fCTFont: SkUniqueCFRef<CTFont>,
    fInvTransform: CGAffineTransform,
    fTransform: CGAffineTransform,
    fDoSubPosition: bool,
    fOffscreen: Offscreen,
    fBase: SkScalerContext,
    // Native derived-type projection retained as a non-owning pointer because Rust
    // stores the canonical Typeface base and Mac virtual implementation separately.
    fMacTypeface: core::ptr::NonNull<SkTypeface_Mac>,
}
// Source: src/ports/SkScalerContext_mac_ct.h:53-74
// Source: src/ports/SkScalerContext_mac_ct.h:53-55,57-58
struct Offscreen {
    fCGForegroundColor: SkUniqueCFRef<CGColor>,
    fCG: SkUniqueCFRef<CGContext>,
    fRGBSpace: SkUniqueCFRef<CGColorSpace>,
    fImageStorage: crate::src::core::SkAutoMalloc::SkAutoSMalloc<{ Offscreen::kSize }>,
    fSKForegroundColor: u32,
    fSize: crate::include::core::SkSize::SkISize,
    fDoAA: bool,
    fDoLCD: bool,
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:59-59
const kSkShowTextBlitCoverage: bool = false;
// Source: src/ports/SkScalerContext_mac_ct.cpp:62-104
unsafe fn sk_memset_rect32(
    mut pixels: *mut u32,
    value: u32,
    width: i32,
    mut height: i32,
    mut row_bytes: usize,
) {
    debug_assert!(width != 0);
    debug_assert!(width as usize * 4 <= row_bytes);
    if width >= 32 {
        while height != 0 {
            unsafe {
                crate::src::opts::SkMemset_opts::memsetT_raw(pixels, value, width as usize);
                pixels = pixels.cast::<u8>().add(row_bytes).cast();
            }
            height -= 1;
        }
        return;
    }
    row_bytes -= width as usize * 4;
    if width >= 8 {
        while height != 0 {
            let mut w = width;
            loop {
                for _ in 0..8 {
                    unsafe {
                        pixels.write(value);
                        pixels = pixels.add(1);
                    }
                }
                w -= 8;
                if w < 8 {
                    break;
                }
            }
            loop {
                w -= 1;
                if w < 0 {
                    break;
                }
                unsafe {
                    pixels.write(value);
                    pixels = pixels.add(1);
                }
            }
            unsafe {
                pixels = pixels.cast::<u8>().add(row_bytes).cast();
            }
            height -= 1;
        }
    } else {
        while height != 0 {
            let mut w = width;
            loop {
                unsafe {
                    pixels.write(value);
                    pixels = pixels.add(1);
                }
                w -= 1;
                if w <= 0 {
                    break;
                }
            }
            unsafe {
                pixels = pixels.cast::<u8>().add(row_bytes).cast();
            }
            height -= 1;
        }
    }
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:106-108
fn CGRGBPixel_getAlpha(pixel: CGRGBPixel) -> u32 {
    pixel & 255
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:110-117
fn MatrixToCGAffineTransform(matrix: &SkMatrix) -> CGAffineTransform {
    unsafe {
        CGAffineTransformMake(
            matrix[0] as f64,
            -(matrix[3] as f64),
            -(matrix[1] as f64),
            matrix[4] as f64,
            matrix[2] as f64,
            matrix[5] as f64,
        )
    }
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:119-150
impl SkScalerContext_Mac {
    /// Safety: canonical typeface points to the live Mac Typeface containing the
    /// supplied actual Mac virtual implementation. Effect handles match effects and
    /// all Typeface/CF/backing leases outlive this scaler, as in native inheritance.
    pub unsafe fn new(
        typeface: core::ptr::NonNull<crate::src::core::SkTypeface::SkTypeface>,
        effects: &crate::src::core::SkScalerContext::SkScalerContextEffects<'_>,
        desc: &crate::src::core::SkDescriptor::SkDescriptor,
        path_owner: &crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkPathEffect::SkPathEffect,
        >,
        mask_owner: Option<&std::sync::Arc<dyn crate::src::core::SkMaskFilter::SkMaskFilter>>,
        mac_typeface: &SkTypeface_Mac,
    ) -> Self {
        let base = unsafe { SkScalerContext::new(typeface, effects, desc, path_owner, mask_owner) };
        let offscreen = Offscreen::new(base.getRec().fForegroundColor);
        let subposition = base.getRec().fFlags
            & crate::src::core::SkScalerContext::flags::SUBPIXEL_POSITIONING
            != 0;
        let font = unsafe { typeface.as_ref().internal_private_getCTFontRef() }.cast::<CTFont>();
        let mut scale = crate::src::core::SkPoint::SkPoint::make(0.0, 0.0);
        let mut transform = SkMatrix::default();
        let invertible = base.getRec().compute_matrices(
            crate::src::core::SkScalerContext::PreMatrixScale::Vertical,
            &mut scale,
            &mut transform,
            None,
            None,
            None,
        );
        let transform = MatrixToCGAffineTransform(&transform);
        let inverse = if invertible {
            unsafe { CGAffineTransformInvert(transform) }
        } else {
            transform
        };
        let font = unsafe {
            crate::src::utils::mac::SkCTFontCreateExactCopy::SkCTFontCreateExactCopy(
                font,
                scale.fY as f64,
                mac_typeface.fOpszVariation,
            )
        };
        let cgfont = unsafe {
            SkUniqueCFRef::from_raw(CTFontCopyGraphicsFont(font.get(), core::ptr::null_mut()))
        };
        Self {
            fCGFont: cgfont,
            fCTFont: font,
            fInvTransform: inverse,
            fTransform: transform,
            fDoSubPosition: subposition,
            fOffscreen: offscreen,
            fBase: base,
            fMacTypeface: core::ptr::NonNull::from(mac_typeface),
        }
    }
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:152-154
fn RoundSize(dimension: i32) -> i32 {
    crate::src::core::SkMathPriv::SkNextPow2(dimension)
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:156-166
unsafe fn CGColorForSkColor(space: *mut CGColorSpace, color: u32) -> *mut CGColor {
    use crate::src::core::SkColor::{SkColorGetB, SkColorGetG, SkColorGetR};
    let scale = (1.0f32 / 255.0f32) as f64;
    let components = [
        SkColorGetR(color) as f64 * scale,
        SkColorGetG(color) as f64 * scale,
        SkColorGetB(color) as f64 * scale,
        1.0,
    ];
    unsafe { CGColorCreate(space, components.as_ptr()) }
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:168-175
impl Offscreen {
    fn new(foreground: u32) -> Self {
        Self {
            fCGForegroundColor: SkUniqueCFRef::default(),
            fCG: SkUniqueCFRef::default(),
            fRGBSpace: SkUniqueCFRef::default(),
            fImageStorage: crate::src::core::SkAutoMalloc::SkAutoSMalloc::new(),
            fSKForegroundColor: foreground,
            fSize: crate::include::core::SkSize::SkISize::Make(0, 0),
            fDoAA: false,
            fDoLCD: false,
        }
    }
    // Source: src/ports/SkScalerContext_mac_ct.cpp:177-295
    // The native context borrow is projected into its actual immutable fields so
    // Rust does not alias this mutable Offscreen with a borrow of its containing object.
    unsafe fn getCG(
        &mut self,
        transform: CGAffineTransform,
        inverse: CGAffineTransform,
        font: CTFontRef,
        subposition: bool,
        glyph: &SkGlyph,
        glyph_id: u16,
        row_bytes: &mut usize,
        generate_a8_from_lcd: bool,
    ) -> *mut CGRGBPixel {
        unsafe {
            if self.fRGBSpace.is_null() {
                self.fRGBSpace.reset(CGColorSpaceCreateDeviceRGB());
                self.fCGForegroundColor.reset(CGColorForSkColor(
                    self.fRGBSpace.get(),
                    self.fSKForegroundColor,
                ));
            }
            let (mut aa, mut lcd) = (false, false);
            if glyph.maskFormat() != Format::kBW_Format {
                lcd = true;
                aa = true;
            }
            if !generate_a8_from_lcd && glyph.maskFormat() == Format::kA8_Format {
                lcd = false;
                aa = true;
            }
            if glyph.maskFormat() == Format::kARGB32_Format {
                lcd = false;
            }
            let mut stride = self.fSize.fWidth as usize * 4;
            if self.fCG.is_null()
                || self.fSize.fWidth < glyph.width()
                || self.fSize.fHeight < glyph.height()
            {
                if self.fSize.fWidth < glyph.width() {
                    self.fSize.fWidth = RoundSize(glyph.width());
                }
                if self.fSize.fHeight < glyph.height() {
                    self.fSize.fHeight = RoundSize(glyph.height());
                }
                stride = self.fSize.fWidth as usize * 4;
                let image = self.fImageStorage.reset(
                    stride * self.fSize.fHeight as usize,
                    crate::src::core::SkAutoMalloc::OnShrink::kReuse_OnShrink,
                    None,
                );
                // SDK kCGImageAlphaPremultipliedFirst=2, kCGImageAlphaNoneSkipFirst=6,
                // kCGBitmapByteOrder32Host on selected little-endian Apple64 =2<<12.
                let alpha = if glyph.isColor() { 2 } else { 6 };
                self.fCG.reset(CGBitmapContextCreate(
                    image.cast(),
                    self.fSize.fWidth as usize,
                    self.fSize.fHeight as usize,
                    8,
                    stride,
                    self.fRGBSpace.get(),
                    (2 << 12) | alpha,
                ));
                CGContextSetAllowsFontSubpixelQuantization(self.fCG.get(), false);
                CGContextSetShouldSubpixelQuantizeFonts(self.fCG.get(), false);
                CGContextSetAllowsFontSubpixelPositioning(self.fCG.get(), true);
                CGContextSetShouldSubpixelPositionFonts(self.fCG.get(), true);
                CGContextSetTextDrawingMode(self.fCG.get(), 0);
                if glyph.maskFormat() != Format::kARGB32_Format {
                    CGContextSetGrayFillColor(self.fCG.get(), 0.0, 1.0);
                } else {
                    CGContextSetFillColorWithColor(self.fCG.get(), self.fCGForegroundColor.get());
                }
                self.fDoAA = !aa;
                self.fDoLCD = !lcd;
                CGContextSetTextMatrix(self.fCG.get(), transform);
            }
            if self.fDoAA != aa {
                CGContextSetShouldAntialias(self.fCG.get(), aa);
                self.fDoAA = aa;
            }
            if self.fDoLCD != lcd {
                CGContextSetShouldSmoothFonts(self.fCG.get(), lcd);
                self.fDoLCD = lcd;
            }
            let image = self
                .fImageStorage
                .get()
                .cast::<u32>()
                .add(((self.fSize.fHeight - glyph.height()) * self.fSize.fWidth) as usize);
            sk_memset_rect32(
                image,
                if !glyph.isColor() { 0xffffffff } else { 0 },
                glyph.width(),
                glyph.height(),
                stride,
            );
            let (x, y) = if subposition {
                (
                    glyph.getSubXFixed() as f32 / 65536.0,
                    glyph.getSubYFixed() as f32 / 65536.0,
                )
            } else {
                (0.0, 0.0)
            };
            let point = CGPoint {
                x: (-glyph.left() as f32 + x) as f64,
                y: (glyph.top() as f32 + glyph.height() as f32 - y) as f64,
            };
            let point = CGPointApplyAffineTransform(point, inverse);
            CTFontDrawGlyphs(font, &glyph_id, &point, 1, self.fCG.get());
            *row_bytes = stride;
            image
        }
    }
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:297-364
impl SkScalerContext_Mac {
    pub unsafe fn generateMetrics(&mut self, glyph: &SkGlyph) -> GlyphMetrics {
        let mut metrics = GlyphMetrics::new(glyph.maskFormat());
        metrics.neverRequestPath = unsafe { self.fMacTypeface.as_ref() }.fHasColorGlyphs;
        let id = glyph.getGlyphID();
        let mut advance = core::mem::MaybeUninit::<CGSize>::uninit();
        unsafe {
            CTFontGetAdvancesForGlyphs(self.fCTFont.get(), 1, &id, advance.as_mut_ptr(), 1);
        }
        let advance = unsafe { CGSizeApplyAffineTransform(advance.assume_init(), self.fTransform) };
        metrics.advance.fX = advance.width as f32;
        metrics.advance.fY = -advance.height as f32;
        let mut bounds = core::mem::MaybeUninit::<CGRect>::uninit();
        unsafe {
            CTFontGetBoundingRectsForGlyphs(self.fCTFont.get(), 1, &id, bounds.as_mut_ptr(), 1);
        }
        let bounds = unsafe { CGRectApplyAffineTransform(bounds.assume_init(), self.fTransform) };
        if advance.width == 0.0 && advance.height == 0.0 {
            let path = unsafe {
                SkUniqueCFRef::from_raw(CTFontCreatePathForGlyph(
                    self.fCTFont.get(),
                    id,
                    core::ptr::null(),
                ))
            };
            if path.is_null() || unsafe { CGPathIsEmpty(path.get()) } {
                return metrics;
            }
        }
        if crate::src::utils::mac::SkCGGeometry::SkCGRectIsEmpty(&bounds) {
            return metrics;
        }
        let mut bounds = SkRect::MakeXYWH(
            bounds.origin.x as f32,
            (-bounds.origin.y - bounds.size.height) as f32,
            bounds.size.width as f32,
            bounds.size.height as f32,
        );
        if self.fDoSubPosition {
            bounds.fRight += glyph.getSubXFixed() as f32 / 65536.0;
            bounds.fBottom += glyph.getSubYFixed() as f32 / 65536.0;
        }
        bounds.roundOutRectTo(&mut metrics.bounds);
        metrics.bounds.outset(1.0, 1.0);
        metrics
    }
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:366-377
const fn sk_pow2_table(i: usize) -> u8 {
    ((i * i + 128) / 255) as u8
}
const gLinearCoverageFromCGLCDValue: [u8; 256] = {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = sk_pow2_table(i);
        i += 1;
    }
    table
};
// Source: src/ports/SkScalerContext_mac_ct.cpp:379-390
unsafe fn cgpixels_to_bits(mut dst: *mut u8, mut src: *const CGRGBPixel, mut count: i32) {
    while count > 0 {
        let mut mask = 0u8;
        for i in (0..=7).rev() {
            unsafe {
                mask |= (((CGRGBPixel_getAlpha(src.read()) >> 7) ^ 1) << i) as u8;
                src = src.add(1);
            }
            count -= 1;
            if count == 0 {
                break;
            }
        }
        unsafe {
            dst.write(mask);
            dst = dst.add(1);
        }
    }
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:392-402
unsafe fn rgb_to_a8<const APPLY_PREBLEND: bool>(rgb: CGRGBPixel, table: *const u8) -> u8 {
    let r = 255 - ((rgb >> 16) & 255);
    let g = 255 - ((rgb >> 8) & 255);
    let b = 255 - (rgb & 255);
    let mut lum = crate::src::core::SkColorData::SkComputeLuminance(r, g, b);
    if APPLY_PREBLEND {
        lum = unsafe { table.add(lum as usize).read() } as u32;
    }
    if kSkShowTextBlitCoverage {
        lum = lum.max(0x30);
    }
    lum as u8
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:404-419
unsafe fn RGBToA8<const APPLY_PREBLEND: bool>(
    mut pixels: *const CGRGBPixel,
    row_bytes: usize,
    glyph: &SkGlyph,
    image: *mut c_void,
    table: *const u8,
) {
    let mut dst = image.cast::<u8>();
    for _ in 0..glyph.height() {
        for x in 0..glyph.width() as usize {
            unsafe {
                dst.add(x)
                    .write(rgb_to_a8::<APPLY_PREBLEND>(pixels.add(x).read(), table));
            }
        }
        unsafe {
            pixels = pixels.cast::<u8>().add(row_bytes).cast();
            dst = dst.add(glyph.rowBytes());
        }
    }
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:421-433
unsafe fn rgb_to_lcd16<const APPLY_PREBLEND: bool>(
    rgb: CGRGBPixel,
    rtable: *const u8,
    gtable: *const u8,
    btable: *const u8,
) -> u16 {
    let (mut r, mut g, mut b) = (
        255 - ((rgb >> 16) & 255),
        255 - ((rgb >> 8) & 255),
        255 - (rgb & 255),
    );
    if APPLY_PREBLEND {
        unsafe {
            r = rtable.add(r as usize).read() as u32;
            g = gtable.add(g as usize).read() as u32;
            b = btable.add(b as usize).read() as u32;
        }
    }
    if kSkShowTextBlitCoverage {
        r = r.max(0x30);
        g = g.max(0x30);
        b = b.max(0x30);
    }
    crate::src::core::SkColorData::SkPack888ToRGB16(r, g, b) as u16
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:435-451
unsafe fn RGBToLcd16<const APPLY_PREBLEND: bool>(
    mut pixels: *const CGRGBPixel,
    row_bytes: usize,
    glyph: &SkGlyph,
    image: *mut c_void,
    r: *const u8,
    g: *const u8,
    b: *const u8,
) {
    let mut dst = image.cast::<u16>();
    for _ in 0..glyph.height() {
        for x in 0..glyph.width() as usize {
            unsafe {
                dst.add(x).write(rgb_to_lcd16::<APPLY_PREBLEND>(
                    pixels.add(x).read(),
                    r,
                    g,
                    b,
                ));
            }
        }
        unsafe {
            pixels = pixels.cast::<u8>().add(row_bytes).cast();
            dst = dst.cast::<u8>().add(glyph.rowBytes()).cast();
        }
    }
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:453-462
fn cgpixels_to_pmcolor(rgb: CGRGBPixel) -> u32 {
    let mut a = (rgb >> 24) & 255;
    let r = (rgb >> 16) & 255;
    let g = (rgb >> 8) & 255;
    let b = rgb & 255;
    if kSkShowTextBlitCoverage {
        a = a.max(0x30);
    }
    crate::src::core::SkColorPriv::SkPackARGB32(a, r, g, b)
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:464-546
impl SkScalerContext_Mac {
    /// Safety: glyph pixels and native CTFont/CGContext/Typeface leases are live;
    /// image_buffer is initialized/exclusively writable for glyph.imageSize().
    pub unsafe fn generateImage(&mut self, glyph: &SkGlyph, image_buffer: *mut c_void) {
        let id = glyph.getGlyphID();
        let smooth = self.fBase.getRec().get_hinting()
            != crate::include::core::SkFontTypes::SkFontHinting::kNone;
        let mut row_bytes = 0;
        let pixels = unsafe {
            self.fOffscreen.getCG(
                self.fTransform,
                self.fInvTransform,
                self.fCTFont.get(),
                self.fDoSubPosition,
                glyph,
                id,
                &mut row_bytes,
                smooth,
            )
        };
        if pixels.is_null() {
            return;
        }
        // Exact native dependency, not a host-version guess or constant substitute.
        if glyph.maskFormat() == Format::kLCD16_Format
            || (glyph.maskFormat() == Format::kA8_Format
                && smooth
                && crate::src::utils::mac::SkCTFont::SkCTFontGetSmoothBehavior()
                    != crate::src::utils::mac::SkCTFont::SkCTFontSmoothBehavior::none)
        {
            let mut addr = pixels;
            for _ in 0..glyph.height() {
                for x in 0..glyph.width() as usize {
                    unsafe {
                        let value = addr.add(x).read();
                        let r =
                            gLinearCoverageFromCGLCDValue[((value >> 16) & 255) as usize] as u32;
                        let g = gLinearCoverageFromCGLCDValue[((value >> 8) & 255) as usize] as u32;
                        let b = gLinearCoverageFromCGLCDValue[(value & 255) as usize] as u32;
                        addr.add(x).write((r << 16) | (g << 8) | b);
                    }
                }
                unsafe {
                    addr = addr.cast::<u8>().add(row_bytes).cast();
                }
            }
        }
        let preblend = &self.fBase.fPreBlend;
        let r = preblend
            .channel(0)
            .map_or(core::ptr::null(), |v| v.as_ptr());
        let g = preblend
            .channel(1)
            .map_or(core::ptr::null(), |v| v.as_ptr());
        let b = preblend
            .channel(2)
            .map_or(core::ptr::null(), |v| v.as_ptr());
        unsafe {
            match glyph.maskFormat() {
                Format::kLCD16_Format => {
                    if preblend.is_applicable() {
                        RGBToLcd16::<true>(pixels, row_bytes, glyph, image_buffer, r, g, b);
                    } else {
                        RGBToLcd16::<false>(pixels, row_bytes, glyph, image_buffer, r, g, b);
                    }
                }
                Format::kA8_Format => {
                    if preblend.is_applicable() {
                        RGBToA8::<true>(pixels, row_bytes, glyph, image_buffer, g);
                    } else {
                        RGBToA8::<false>(pixels, row_bytes, glyph, image_buffer, g);
                    }
                }
                Format::kBW_Format => {
                    let mut src = pixels;
                    let mut dst = image_buffer.cast::<u8>();
                    for _ in 0..glyph.height() {
                        cgpixels_to_bits(dst, src, glyph.width());
                        src = src.cast::<u8>().add(row_bytes).cast();
                        dst = dst.add(glyph.rowBytes());
                    }
                }
                Format::kARGB32_Format => {
                    let mut src = pixels;
                    let mut dst = image_buffer.cast::<u32>();
                    for _ in 0..glyph.height() {
                        for x in 0..glyph.width() as usize {
                            dst.add(x).write(cgpixels_to_pmcolor(src.add(x).read()));
                        }
                        src = src.cast::<u8>().add(row_bytes).cast();
                        dst = dst.cast::<u8>().add(glyph.rowBytes()).cast();
                    }
                }
                _ => debug_assert!(false, "unexpected mask format"),
            }
        }
    }
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:549-618
struct SkCTPathGeometrySink {
    fStarted: bool,
    fCurrent: CGPoint,
    fBuilder: SkPathBuilder,
}
impl SkCTPathGeometrySink {
    fn new() -> Self {
        Self {
            fStarted: false,
            fCurrent: CGPoint { x: 0.0, y: 0.0 },
            fBuilder: SkPathBuilder::new(),
        }
    }
    fn goingTo(&mut self, point: CGPoint) {
        if !self.fStarted {
            self.fStarted = true;
            self.fBuilder
                .moveToXY(self.fCurrent.x as f32, -self.fCurrent.y as f32);
        }
        self.fCurrent = point;
    }
    fn currentIsNot(&self, point: CGPoint) -> bool {
        self.fCurrent.x != point.x || self.fCurrent.y != point.y
    }
    unsafe extern "C" fn ApplyElement(ctx: *mut c_void, element: *const CGPathElement) {
        let this = unsafe { &mut *ctx.cast::<Self>() };
        let element = unsafe { &*element };
        let p = element.points;
        unsafe {
            match element.kind {
                0 => {
                    this.fStarted = false;
                    this.fCurrent = p.read();
                }
                1 => {
                    let a = p.read();
                    if this.currentIsNot(a) {
                        this.goingTo(a);
                        this.fBuilder.lineToXY(a.x as f32, -a.y as f32);
                    }
                }
                2 => {
                    let a = p.read();
                    let b = p.add(1).read();
                    if this.currentIsNot(a) || this.currentIsNot(b) {
                        this.goingTo(b);
                        this.fBuilder
                            .quadToXY(a.x as f32, -a.y as f32, b.x as f32, -b.y as f32);
                    }
                }
                3 => {
                    let a = p.read();
                    let b = p.add(1).read();
                    let c = p.add(2).read();
                    if this.currentIsNot(a) || this.currentIsNot(b) || this.currentIsNot(c) {
                        this.goingTo(c);
                        this.fBuilder.cubicToXY(
                            a.x as f32,
                            -a.y as f32,
                            b.x as f32,
                            -b.y as f32,
                            c.x as f32,
                            -c.y as f32,
                        );
                    }
                }
                4 => {
                    if this.fStarted {
                        this.fBuilder.close();
                    }
                }
                _ => debug_assert!(false, "Unknown path element"),
            }
        }
    }
}
// Source: src/ports/SkScalerContext_mac_ct.cpp:626-626
const kScaleForSubPixelPositionHinting: f32 = 4.0;
// Source: src/ports/SkScalerContext_mac_ct.cpp:628-678
impl SkScalerContext_Mac {
    pub unsafe fn generatePath(&mut self, glyph: &SkGlyph) -> Option<GeneratedPath> {
        let (mut scale_x, mut scale_y) = (1.0f32, 1.0f32);
        let mut transform = self.fTransform;
        if self.fDoSubPosition {
            scale_x = kScaleForSubPixelPositionHinting;
            scale_y = kScaleForSubPixelPositionHinting;
            match self.fBase.getRec().compute_axis_alignment_for_h_text() {
                crate::src::core::SkGlyph::SkAxisAlignment::X => scale_y = 1.0,
                crate::src::core::SkGlyph::SkAxisAlignment::Y => scale_x = 1.0,
                _ => {}
            }
            transform = unsafe {
                CGAffineTransformConcat(
                    self.fTransform,
                    CGAffineTransformMakeScale(scale_x as f64, scale_y as f64),
                )
            };
        }
        let path = unsafe {
            SkUniqueCFRef::from_raw(CTFontCreatePathForGlyph(
                self.fCTFont.get(),
                glyph.getGlyphID(),
                &transform,
            ))
        };
        if path.is_null() {
            return None;
        }
        let mut sink = SkCTPathGeometrySink::new();
        unsafe {
            CGPathApply(
                path.get(),
                (&mut sink as *mut SkCTPathGeometrySink).cast(),
                SkCTPathGeometrySink::ApplyElement,
            );
        }
        if self.fDoSubPosition {
            let mut matrix = SkMatrix::default();
            matrix.setScale(1.0 / scale_x, 1.0 / scale_y);
            sink.fBuilder.transform(&matrix);
        }
        Some(GeneratedPath {
            path: sink.fBuilder.detach(None),
            modified: false,
        })
    }
}

// Source: src/ports/SkScalerContext_mac_ct.cpp:680-755
impl SkScalerContext_Mac {
    /// Safety: native font and containing canonical Typeface are live; output is
    /// exclusively writable. Packed OS/2 values are loaded without aligned borrows.
    pub unsafe fn generateFontMetrics(
        &mut self,
        metrics: &mut crate::include::core::SkFontMetrics::SkFontMetrics,
    ) {
        use crate::include::core::SkFontMetrics::SkFontMetrics;
        use crate::src::sfnt::{
            SkOTTable_OS_2_V0::SkOTTableOS2_V0, SkOTTable_OS_2_V2::SkOTTableOS2_V2,
        };
        use crate::src::utils::mac::SkCGGeometry::*;
        let font = self.fCTFont.get();
        let bounds = unsafe { CTFontGetBoundingBox(font) };
        metrics.fTop = -SkCGRectGetMaxY(&bounds) as f32;
        metrics.fAscent = -unsafe { CTFontGetAscent(font) } as f32;
        metrics.fDescent = unsafe { CTFontGetDescent(font) } as f32;
        metrics.fBottom = -SkCGRectGetMinY(&bounds) as f32;
        metrics.fLeading = unsafe { CTFontGetLeading(font) } as f32;
        metrics.fAvgCharWidth = SkCGRectGetWidth(&bounds) as f32;
        metrics.fXMin = SkCGRectGetMinX(&bounds) as f32;
        metrics.fXMax = SkCGRectGetMaxX(&bounds) as f32;
        metrics.fMaxCharWidth = metrics.fXMax - metrics.fXMin;
        metrics.fXHeight = unsafe { CTFontGetXHeight(font) } as f32;
        metrics.fCapHeight = unsafe { CTFontGetCapHeight(font) } as f32;
        metrics.fUnderlineThickness = unsafe { CTFontGetUnderlineThickness(font) } as f32;
        metrics.fUnderlinePosition = -(unsafe { CTFontGetUnderlinePosition(font) } as f32);
        metrics.fStrikeoutThickness = 0.0;
        metrics.fStrikeoutPosition = 0.0;
        metrics.fFlags = SkFontMetrics::kUnderlineThicknessIsValid_Flag
            | SkFontMetrics::kUnderlinePositionIsValid_Flag;
        let mac = unsafe { self.fMacTypeface.as_ref() };
        let axes = mac.getVariationAxes();
        if (!axes.is_null() && unsafe { CFArrayGetCount(axes) } > 0) || mac.fHasColorGlyphs {
            metrics.fFlags |= SkFontMetrics::kBoundsInvalid_Flag;
        }
        let tag = crate::src::sfnt::SkOTTable_OS_2::SkOTTableOS2::TAG.to_be();
        let data = unsafe { (&*self.fBase.getTypeface()).copyTableData(tag) };
        if let Some(data) = data {
            let size = unsafe { CTFontGetSize(font) };
            let upem = unsafe { CTFontGetUnitsPerEm(font) };
            let max_height = upem * 2;
            if core::mem::size_of::<SkOTTableOS2_V2>() <= data.size() {
                let table = data.data().cast::<SkOTTableOS2_V2>();
                let x = u16::from_be(unsafe {
                    core::ptr::addr_of!((*table).sxHeight).read_unaligned()
                });
                if x != 0 && (x as u32) < max_height {
                    metrics.fXHeight = (x as f64 * size / upem as f64) as f32;
                }
                let cap = u16::from_be(unsafe {
                    core::ptr::addr_of!((*table).sCapHeight).read_unaligned()
                });
                if cap != 0 && (cap as u32) < max_height {
                    metrics.fCapHeight = (cap as f64 * size / upem as f64) as f32;
                }
            }
            if core::mem::size_of::<SkOTTableOS2_V0>() <= data.size() {
                let table = data.data().cast::<SkOTTableOS2_V0>();
                let thick = u16::from_be(unsafe {
                    core::ptr::addr_of!((*table).yStrikeoutSize).read_unaligned()
                });
                if thick != 0 && (thick as u32) < max_height {
                    metrics.fStrikeoutThickness = (thick as f64 * size / upem as f64) as f32;
                    metrics.fFlags |= SkFontMetrics::kStrikeoutThicknessIsValid_Flag;
                }
                let position = u16::from_be(unsafe {
                    core::ptr::addr_of!((*table).yStrikeoutPosition).read_unaligned()
                });
                if position != 0 && (position as u32) < max_height {
                    metrics.fStrikeoutPosition = -((position as f64 * size / upem as f64) as f32);
                    metrics.fFlags |= SkFontMetrics::kStrikeoutPositionIsValid_Flag;
                }
            }
        }
    }
}
// Source: src/ports/SkScalerContext_mac_ct.h:42-50
impl crate::src::core::SkScalerContext::SkScalerContextVirtual for SkScalerContext_Mac {
    fn base(&self) -> &SkScalerContext {
        &self.fBase
    }
    fn base_mut(&mut self) -> &mut SkScalerContext {
        &mut self.fBase
    }
    unsafe fn generateMetrics(
        &mut self,
        glyph: &SkGlyph,
        _alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> GlyphMetrics {
        unsafe { Self::generateMetrics(self, glyph) }
    }
    unsafe fn generateImage(&mut self, glyph: &SkGlyph, buffer: *mut c_void) {
        unsafe { Self::generateImage(self, glyph, buffer) }
    }
    unsafe fn generatePath(&mut self, glyph: &SkGlyph) -> Option<GeneratedPath> {
        unsafe { Self::generatePath(self, glyph) }
    }
    unsafe fn generateFontMetrics(
        &mut self,
        metrics: &mut crate::include::core::SkFontMetrics::SkFontMetrics,
    ) {
        unsafe { Self::generateFontMetrics(self, metrics) }
    }
}

impl Offscreen {
    // Source: src/ports/SkScalerContext_mac_ct.h:61-63
    const kSize: usize = 32 * 32 * core::mem::size_of::<CGRGBPixel>();
}
