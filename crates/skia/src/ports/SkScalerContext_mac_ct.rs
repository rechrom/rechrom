// CPU implementation moved from renderer/glyphs/scaler_context_mac_ct.rs.

//! macOS glyph masks through the same system rasterizer as Skia's Mac port.
//! Font selection, mask conversion, positioning and compositing remain Rust.
use crate::src::core::SkStrike::SkStrike;
use crate::src::core::SkStrikeCache::SkStrikeCache;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr::{self, NonNull};
use std::sync::OnceLock;

use crate::compat::commands::Color;
use crate::compat::commands::{FontFace, FontSmoothing, FontVariation};
use crate::raster::{Mask, Pixmap, Transform};
use crate::src::core::SkScalerContext::{PreMatrixScale, SkScalerContextRec};

type Ref = *const c_void;
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Point {
    x: f64,
    y: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Size {
    width: f64,
    height: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rect {
    origin: Point,
    size: Size,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Affine {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    tx: f64,
    ty: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SmoothBehavior {
    None,
    Gray,
    Subpixel,
}

// SkCTFont.cpp: SkCTFontGetSmoothBehavior. Use its probe font and comparison
// rather than assuming the operating system's current font-smoothing mode.
fn smooth_behavior() -> SmoothBehavior {
    static BEHAVIOR: OnceLock<SmoothBehavior> = OnceLock::new();
    *BEHAVIOR.get_or_init(|| unsafe {
        // Copyright 2020 Google LLC; font from SkCTFont.cpp, see SKIA_LICENSE.
        let bytes = include_bytes!("spider_symbol.ttf");
        let data = Owned::new(CFDataCreate(
            ptr::null(),
            bytes.as_ptr(),
            bytes.len() as isize,
        ));
        let descriptors = Owned::new(CTFontManagerCreateFontDescriptorsFromData(data.get()));
        let font = Owned::new(CTFontCreateWithFontDescriptor(
            CFArrayGetValueAtIndex(descriptors.get(), 0),
            16.0,
            ptr::null(),
        ));
        let space = Owned::new(CGColorSpaceCreateDeviceRGB());
        let mut images = [[0u8; 16 * 16 * 4]; 2];
        for (i, image) in images.iter_mut().enumerate() {
            let context = Owned::new(CGBitmapContextCreate(
                image.as_mut_ptr().cast(),
                16,
                16,
                8,
                64,
                space.get(),
                (2 << 12) | 6,
            ));
            CGContextSetShouldSmoothFonts(context.get(), i == 1);
            CGContextSetShouldAntialias(context.get(), true);
            CGContextSetTextDrawingMode(context.get(), 0);
            CGContextSetGrayFillColor(context.get(), 1.0, 1.0);
            CTFontDrawGlyphs(font.get(), &3, &Point { x: 0.0, y: 3.0 }, 1, context.get());
        }
        let mut result = SmoothBehavior::None;
        for (plain, smooth) in images[0].chunks_exact(4).zip(images[1].chunks_exact(4)) {
            if smooth[0] != smooth[1] || smooth[1] != smooth[2] {
                return SmoothBehavior::Subpixel;
            }
            if plain != smooth {
                result = SmoothBehavior::Gray;
            }
        }
        result
    })
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(value: Ref);
    fn CFGetTypeID(value: Ref) -> usize;
    fn CFNumberGetTypeID() -> usize;
    fn CFNumberGetValue(number: Ref, kind: isize, value: *mut c_void) -> bool;
    fn CFStringCreateWithBytes(
        allocator: Ref,
        bytes: *const u8,
        length: isize,
        encoding: u32,
        external: bool,
    ) -> Ref;
    fn CFDataCreate(allocator: Ref, bytes: *const u8, length: isize) -> Ref;
    fn CFArrayGetCount(array: Ref) -> isize;
    fn CFArrayGetValueAtIndex(array: Ref, index: isize) -> Ref;
    fn CFSetCreate(allocator: Ref, values: *const Ref, count: isize, callbacks: Ref) -> Ref;
    fn CFNumberCreate(allocator: Ref, kind: isize, value: Ref) -> Ref;
    fn CFDictionaryCreate(
        allocator: Ref,
        keys: *const Ref,
        values: *const Ref,
        count: isize,
        key_callbacks: Ref,
        value_callbacks: Ref,
    ) -> Ref;
    static kCFTypeDictionaryKeyCallBacks: u8;
    static kCFTypeDictionaryValueCallBacks: u8;
    static kCFTypeSetCallBacks: u8;
}
#[link(name = "CoreText", kind = "framework")]
extern "C" {
    fn CTFontManagerCreateFontDescriptorsFromData(data: Ref) -> Ref;
    fn CTFontManagerCreateFontDescriptorFromData(data: Ref) -> Ref;
    fn CTFontGetBoundingBox(font: Ref) -> Rect;
    fn CTFontGetSymbolicTraits(font: Ref) -> u32;
    fn CTFontCopyVariationAxes(font: Ref) -> Ref;
    fn CTFontCopyAttribute(font: Ref, attribute: Ref) -> Ref;
    fn CTFontGetSize(font: Ref) -> f64;
    fn CTFontCreateWithFontDescriptor(descriptor: Ref, size: f64, matrix: *const Affine) -> Ref;
    fn CTFontDescriptorCreateWithAttributes(attributes: Ref) -> Ref;
    fn CTFontDescriptorCreateMatchingFontDescriptor(descriptor: Ref, mandatory: Ref) -> Ref;
    fn CTFontCreateCopyWithAttributes(
        font: Ref,
        size: f64,
        matrix: *const Affine,
        attributes: Ref,
    ) -> Ref;
    fn CTFontGetBoundingRectsForGlyphs(
        font: Ref,
        orientation: u32,
        glyphs: *const u16,
        rects: *mut Rect,
        count: isize,
    ) -> Rect;
    fn CTFontDrawGlyphs(
        font: Ref,
        glyphs: *const u16,
        points: *const Point,
        count: usize,
        context: Ref,
    );
    static kCTFontVariationAttribute: Ref;
    static kCTFontFamilyNameAttribute: Ref;
    static kCTFontTraitsAttribute: Ref;
    static kCTFontWeightTrait: Ref;
    static kCTFontWidthTrait: Ref;
    static kCTFontSlantTrait: Ref;
}
extern "C" {
    fn dlsym(handle: *mut c_void, name: *const std::ffi::c_char) -> *mut c_void;
}
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGColorSpaceCreateDeviceRGB() -> Ref;
    fn CGBitmapContextCreate(
        data: *mut c_void,
        width: usize,
        height: usize,
        bits: usize,
        row_bytes: usize,
        space: Ref,
        info: u32,
    ) -> Ref;
    fn CGContextSetAllowsFontSubpixelQuantization(context: Ref, enabled: bool);
    fn CGContextSetShouldSubpixelQuantizeFonts(context: Ref, enabled: bool);
    fn CGContextSetAllowsFontSubpixelPositioning(context: Ref, enabled: bool);
    fn CGContextSetShouldSubpixelPositionFonts(context: Ref, enabled: bool);
    fn CGContextSetShouldAntialias(context: Ref, enabled: bool);
    fn CGContextSetShouldSmoothFonts(context: Ref, enabled: bool);
    fn CGContextSetTextDrawingMode(context: Ref, mode: u32);
    fn CGContextSetGrayFillColor(context: Ref, gray: f64, alpha: f64);
    fn CGContextSetRGBFillColor(context: Ref, red: f64, green: f64, blue: f64, alpha: f64);
    fn CGContextSetTextMatrix(context: Ref, matrix: Affine);
}

struct Owned(NonNull<c_void>);
impl Owned {
    fn new(value: Ref) -> Self {
        Self(NonNull::new(value.cast_mut()).expect("CoreText/CoreGraphics allocation failed"))
    }
    fn get(&self) -> Ref {
        self.0.as_ptr()
    }
}
impl Drop for Owned {
    fn drop(&mut self) {
        unsafe { CFRelease(self.get()) }
    }
}

unsafe fn dictionary(keys: &[Ref], values: &[Ref]) -> Owned {
    assert_eq!(keys.len(), values.len());
    Owned::new(CFDictionaryCreate(
        ptr::null(),
        keys.as_ptr(),
        values.as_ptr(),
        keys.len() as isize,
        ptr::addr_of!(kCFTypeDictionaryKeyCallBacks).cast(),
        ptr::addr_of!(kCFTypeDictionaryValueCallBacks).cast(),
    ))
}

// SkCTFont.cpp: SkCTFontGetNSFontWeightMapping and
// SkTypeface_mac_ct.cpp: SkCTFontCTWeightForCSSWeight.
fn native_weight(weight: f64) -> f64 {
    static MAPPING: OnceLock<[f64; 11]> = OnceLock::new();
    let weights = MAPPING.get_or_init(|| {
        let fallback = [-1.0, -0.8, -0.6, -0.4, 0.0, 0.23, 0.3, 0.4, 0.56, 0.62, 1.0];
        let mut result = fallback;
        for (index, name) in [
            c"NSFontWeightUltraLight",
            c"NSFontWeightThin",
            c"NSFontWeightLight",
            c"NSFontWeightRegular",
            c"NSFontWeightMedium",
            c"NSFontWeightSemibold",
            c"NSFontWeightBold",
            c"NSFontWeightHeavy",
            c"NSFontWeightBlack",
        ]
        .iter()
        .enumerate()
        {
            // Darwin RTLD_DEFAULT, as in Skia's runtime AppKit lookup.
            let address = unsafe { dlsym((-2isize) as *mut c_void, name.as_ptr()) };
            if address.is_null() {
                return fallback;
            }
            result[index + 1] = unsafe { *address.cast::<f64>() };
        }
        result
    });
    let weight = (weight as i32).clamp(0, 1000) as f64 / 100.0;
    let lower = (weight.floor() as usize).min(9);
    weights[lower] + (weights[lower + 1] - weights[lower]) * (weight - lower as f64)
}

// SkFontMgr_mac_ct.cpp: create_descriptor and onMatchFamilyStyle.
unsafe fn match_native_family(name: Ref, weight: f64, italic: bool, size: f32) -> Owned {
    let traits = [native_weight(weight), 0.0, if italic { 0.07 } else { 0.0 }];
    let numbers = traits.map(|value| {
        Owned::new(CFNumberCreate(
            ptr::null(),
            13,
            (&value as *const f64).cast(),
        ))
    });
    let trait_dictionary = dictionary(
        &[kCTFontWeightTrait, kCTFontWidthTrait, kCTFontSlantTrait],
        &numbers.each_ref().map(Owned::get),
    );
    let attributes = dictionary(
        &[kCTFontFamilyNameAttribute, kCTFontTraitsAttribute],
        &[name, trait_dictionary.get()],
    );
    let request = Owned::new(CTFontDescriptorCreateWithAttributes(attributes.get()));
    let mandatory = Owned::new(CFSetCreate(
        ptr::null(),
        &kCTFontFamilyNameAttribute,
        1,
        ptr::addr_of!(kCFTypeSetCallBacks).cast(),
    ));
    let descriptor = Owned::new(CTFontDescriptorCreateMatchingFontDescriptor(
        request.get(),
        mandatory.get(),
    ));
    Owned::new(CTFontCreateWithFontDescriptor(
        descriptor.get(),
        size.into(),
        ptr::null(),
    ))
}

// Restricted translation of SkCTFontCreateExactCopy.cpp. Pinning optical size
// preserves variable-font outlines and glyph IDs when the requested CTFont size
// changes solely to absorb a device scale. Unit-size fonts bypass this helper.
unsafe fn exact_copy_preserving_optical_size(
    base: Ref,
    device_size: f32,
    opsz: Option<f64>,
) -> Owned {
    let optical_key_bytes = b"NSCTFontOpticalSizeAttribute";
    let optical_key = Owned::new(CFStringCreateWithBytes(
        ptr::null(),
        optical_key_bytes.as_ptr(),
        optical_key_bytes.len() as isize,
        0x08000100,
        false,
    ));
    let optical_value = if let Some(value) = opsz {
        value
    } else {
        let copied = CTFontCopyAttribute(base, optical_key.get());
        let mut value = 0.0f64;
        let valid = if copied.is_null() {
            false
        } else {
            let copied = Owned::new(copied);
            CFGetTypeID(copied.get()) == CFNumberGetTypeID()
                && CFNumberGetValue(copied.get(), 13, ptr::from_mut(&mut value).cast())
                && value > 0.0
        };
        if valid {
            value
        } else {
            CTFontGetSize(base)
        }
    };
    let number = Owned::new(CFNumberCreate(
        ptr::null(),
        13,
        ptr::from_ref(&optical_value).cast(),
    ));
    let tracking_key_bytes = b"NSCTFontUnscaledTrackingAttribute";
    let tracking_key = Owned::new(CFStringCreateWithBytes(
        ptr::null(),
        tracking_key_bytes.as_ptr(),
        tracking_key_bytes.len() as isize,
        0x08000100,
        false,
    ));
    let zero = 0i32;
    let tracking = Owned::new(CFNumberCreate(ptr::null(), 3, ptr::from_ref(&zero).cast()));
    let attributes = dictionary(
        &[optical_key.get(), tracking_key.get()],
        &[number.get(), tracking.get()],
    );
    let descriptor = Owned::new(CTFontDescriptorCreateWithAttributes(attributes.get()));
    Owned::new(CTFontCreateCopyWithAttributes(
        base,
        device_size.into(),
        ptr::null(),
        descriptor.get(),
    ))
}

// Borrowing adapter for glyph-run setup; keeps the native font alive while
// prepared images are looked up in the separate thread-local strike cache.
pub(crate) struct PreparedGlyphRun<'a> {
    font: Ref,
    remaining: Transform,
    // Device-linear whole-font bounds, before glyph translation and mask padding.
    // None preserves the original per-glyph path when native metrics are invalid.
    coarse_bounds: Option<[f64; 4]>,
    color_glyphs: bool,
    owner: core::marker::PhantomData<&'a SkScalerContext_Mac>,
}

// SkTypeface::onComputeBounds obtains the Mac font metrics, whose whole-font
// extent comes from CTFontGetBoundingBox in generateFontMetrics. The latter
// explicitly marks variable and color fonts as kBoundsInvalid: do the same.
// This native-font-owner cache is a local adapter, not a new glyph image cache.
// SkTypeface_Mac::getVariationAxes keeps its CFArray for the typeface lifetime.
// Keep the array with the actual CTFont cache owner here. Releasing a copied
// variation array immediately after traits/axes queries can make CoreText's
// default-instance glyph metrics empty (independent CoreText18case proof).
struct NativeFontMetadata {
    bounds: Option<Rect>,
    color_glyphs: bool,
    _axes: Option<Owned>,
}
unsafe fn native_font_bounds(font: Ref) -> NativeFontMetadata {
    let color_glyphs = CTFontGetSymbolicTraits(font) & (1 << 13) != 0;
    if color_glyphs {
        return NativeFontMetadata {
            bounds: None,
            color_glyphs,
            _axes: None,
        };
    }
    let native_axes = CTFontCopyVariationAxes(font);
    let axes = (!native_axes.is_null()).then(|| Owned::new(native_axes));
    let variable = axes
        .as_ref()
        .is_some_and(|axes| CFArrayGetCount(axes.get()) > 0);
    let bounds = if variable {
        None
    } else {
        let bounds = CTFontGetBoundingBox(font);
        (bounds.size.width > 0.0
            && bounds.size.height > 0.0
            && [
                bounds.origin.x,
                bounds.origin.y,
                bounds.origin.x + bounds.size.width,
                bounds.origin.y + bounds.size.height,
            ]
            .into_iter()
            .all(f64::is_finite))
        .then_some(bounds)
    };
    NativeFontMetadata {
        bounds,
        color_glyphs,
        _axes: axes,
    }
}

// SkFontPriv::GetFontBounds maps the typeface bounds by the font matrix. Our
// CTFont already absorbs the device text size, so only remaining is applied.
// Conversion to device Y-down follows the Mac generateMetrics text matrix.
fn device_font_bounds(bounds: Rect, remaining: Transform) -> Option<[f64; 4]> {
    let mut result = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for bx in [bounds.origin.x, bounds.origin.x + bounds.size.width] {
        for by in [bounds.origin.y, bounds.origin.y + bounds.size.height] {
            let x = f64::from(remaining.sx) * bx - f64::from(remaining.kx) * by;
            let y = f64::from(remaining.ky) * bx - f64::from(remaining.sy) * by;
            result[0] = result[0].min(x);
            result[1] = result[1].min(y);
            result[2] = result[2].max(x);
            result[3] = result[3].max(y);
        }
    }
    result
        .iter()
        .all(|value| value.is_finite())
        .then_some(result)
}

impl PreparedGlyphRun<'_> {
    pub(crate) fn is_color(&self) -> bool {
        self.color_glyphs
    }
    pub(crate) fn apply_color_italic(&mut self) {
        assert!(self.color_glyphs);
        self.remaining.kx = self.remaining.sx * -0.25 + self.remaining.kx;
        self.remaining.sy = self.remaining.ky * -0.25 + self.remaining.sy;
    }
    /// Exact allocation extent of this port's native gray mask, without
    /// generating its image. Unlike whole-font bounds this uses the resolved
    /// CTFont instance, so a variable outline's selected axes are included.
    /// Empty glyphs return a zero rectangle; unknown metrics return None.
    /// LOCAL_ADAPTER: mirrors generateMetrics; it is not a layout ink box.
    pub(crate) fn native_allocation_bounds_at(
        &self,
        glyph: u16,
        position: Transform,
    ) -> Option<[f64; 4]> {
        if ![position.tx, position.ty].into_iter().all(|v| {
            v.is_finite()
                && f64::from(v) > f64::from(i32::MIN) + 16.0
                && f64::from(v) < f64::from(i32::MAX) - 16.0
        }) {
            return None;
        }
        unsafe {
            // Color images may exceed outline metrics; do not admit them.
            if CTFontGetSymbolicTraits(self.font) & (1 << 13) != 0 {
                return None;
            }
            let cg_bounds =
                CTFontGetBoundingRectsForGlyphs(self.font, 0, &glyph, ptr::null_mut(), 1);
            if ![
                cg_bounds.origin.x,
                cg_bounds.origin.y,
                cg_bounds.size.width,
                cg_bounds.size.height,
            ]
            .into_iter()
            .all(f64::is_finite)
            {
                return None;
            }
            if cg_bounds.size.width <= 0.0 || cg_bounds.size.height <= 0.0 {
                return Some([0.0; 4]);
            }
            // Keep the four-corner arithmetic and order identical to draw's
            // existing generateMetrics; do not substitute another bbox map.
            let matrix = Affine {
                a: f64::from(self.remaining.sx),
                b: -f64::from(self.remaining.ky),
                c: -f64::from(self.remaining.kx),
                d: f64::from(self.remaining.sy),
                tx: 0.0,
                ty: 0.0,
            };
            let mut min_x = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_y = f64::NEG_INFINITY;
            for bx in [
                cg_bounds.origin.x,
                cg_bounds.origin.x + cg_bounds.size.width,
            ] {
                for by in [
                    cg_bounds.origin.y,
                    cg_bounds.origin.y + cg_bounds.size.height,
                ] {
                    let px = matrix.a * bx + matrix.c * by;
                    let py = matrix.b * bx + matrix.d * by;
                    min_x = min_x.min(px);
                    max_x = max_x.max(px);
                    min_y = min_y.min(py);
                    max_y = max_y.max(py);
                }
            }
            let x = position.tx.floor() as i32;
            let y = position.ty.floor() as i32;
            let sub_x = position.tx - x as f32;
            let sub_y = position.ty - y as f32;
            let values = [
                min_x.floor(),
                (-max_y).floor(),
                (max_x + f64::from(sub_x)).ceil(),
                (-min_y + f64::from(sub_y)).ceil(),
            ];
            if !values.into_iter().all(|v| {
                v.is_finite() && v > f64::from(i32::MIN) + 16.0 && v < f64::from(i32::MAX) - 16.0
            }) {
                return None;
            }
            let left = values[0] as i32 - 1;
            let top = values[1] as i32 - 1;
            let right = values[2] as i32 + 1;
            let bottom = values[3] as i32 + 1;
            Some([
                f64::from(x.checked_add(left)?),
                f64::from(y.checked_add(top)?),
                f64::from(x.checked_add(right)?),
                f64::from(y.checked_add(bottom)?),
            ])
        }
    }

    pub(crate) fn conservative_bounds_at(&self, position: Transform) -> Option<[f64; 4]> {
        let bounds = self.coarse_bounds?;
        if ![position.tx, position.ty].into_iter().all(|v| {
            v.is_finite()
                && f64::from(v) > f64::from(i32::MIN) + 16.0
                && f64::from(v) < f64::from(i32::MAX) - 16.0
        }) {
            return None;
        }
        let x = position.tx.floor() as i32;
        let y = position.ty.floor() as i32;
        let sub_x = position.tx - x as f32;
        let sub_y = position.ty - y as f32;
        // Identical to the existing quick_reject font extent, including two
        // pixels of numerical/CG padding and the mask's subpixel phase.
        Some([
            f64::from(x) + bounds[0].floor() - 2.0,
            f64::from(y) + bounds[1].floor() - 2.0,
            f64::from(x) + (bounds[2] + f64::from(sub_x)).ceil() + 2.0,
            f64::from(y) + (bounds[3] + f64::from(sub_y)).ceil() + 2.0,
        ])
    }

    fn quick_reject(
        &self,
        pixmap: &Pixmap,
        clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
        x: i32,
        y: i32,
        sub_x: f32,
        sub_y: f32,
    ) -> bool {
        let Some(bounds) = self.coarse_bounds else {
            return false;
        };
        if !sub_x.is_finite() || !sub_y.is_finite() {
            return false;
        }
        // generateMetrics keeps left/top fixed, adds phase to right/bottom,
        // rounds out, and outsets by one pixel for CG antialiasing. This coarse
        // adapter adds one further pixel to cover outward numerical/hinting
        // boundary differences; the actual per-glyph mask metrics stay intact.
        let left = f64::from(x) + bounds[0].floor() - 2.0;
        let top = f64::from(y) + bounds[1].floor() - 2.0;
        let right = f64::from(x) + (bounds[2] + f64::from(sub_x)).ceil() + 2.0;
        let bottom = f64::from(y) + (bounds[3] + f64::from(sub_y)).ceil() + 2.0;
        let (mut cl, mut ct, mut cr, mut cb) = (
            0.0f64,
            0.0f64,
            f64::from(pixmap.width()),
            f64::from(pixmap.height()),
        );
        if let Some(clip) = clip_summary.and_then(|summary| summary.getBounds()) {
            cl = cl.max(f64::from(clip.left()));
            ct = ct.max(f64::from(clip.top()));
            cr = cr.min(f64::from(clip.right()));
            cb = cb.min(f64::from(clip.bottom()));
        }
        left >= cr || top >= cb || right <= cl || bottom <= ct
    }
}

// Paint-dependent preparation belongs to the glyph run, as fPreBlend belongs
// to the official SkScalerContext. This adapter stores no glyph/frame images.
pub(crate) struct PreparedGlyphPaint {
    color: Color,
    blitter: crate::src::opts::SkBlitMask_opts::PreparedA8Blitter,
    smoothing: FontSmoothing,
    smooth: bool,
    table: Option<[u8; 256]>,
}

impl PreparedGlyphPaint {
    pub(crate) fn new(color: Color, smoothing: FontSmoothing) -> Self {
        let behavior = smooth_behavior();
        let smooth = behavior != SmoothBehavior::None
            && smoothing != FontSmoothing::kNone
            && smoothing != FontSmoothing::kAntialiased;
        Self {
            color,
            blitter: crate::src::opts::SkBlitMask_opts::PreparedA8Blitter::new(
                crate::cpu::mask_blitter::premultiply(color),
            ),
            smoothing,
            smooth,
            table: smooth.then(|| crate::cpu::mask_gamma::mac_smoothing_lut(color, behavior)),
        }
    }
}

// Rust descriptors use exact values rather than font-list indices or hashes.
// Integer device position and raster clip are applied after image lookup.
#[derive(Clone, PartialEq, Eq, Hash)]
struct ImageDescriptor {
    font: usize,
    matrix: [u32; 4],
    antialias: bool,
    smooth: bool,
    preblend: Option<[u8; 256]>,
    // SkScalerContextRec::fForegroundColor: color fonts can include COLR
    // foreground layers and ordinary outlines. Includes alpha as upstream does.
    foreground: Option<[u8; 4]>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct ImageKey {
    glyph: u16,
    sub_x: u32,
    sub_y: u32,
}
pub(crate) struct GlyphImage {
    left: i32,
    top: i32,
    width: usize,
    height: usize,
    pixels: GlyphPixels,
}
enum GlyphPixels {
    A8(Vec<u8>),
    // Canonical RGBA, premultiplied. CoreText BGRA is swizzled once on miss.
    Argb32(Vec<u8>),
}
impl GlyphImage {
    fn storage_bytes(&self) -> usize {
        match &self.pixels {
            GlyphPixels::A8(data) | GlyphPixels::Argb32(data) => data.capacity(),
        }
    }
}
fn paint_rgba8(color: Color) -> [u8; 4] {
    [color.red, color.green, color.blue, color.alpha]
        .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
}
struct TypefaceIdentity {
    id: u64,
    native_family: String,
    weight: u64,
    italic: bool,
    face_index: u32,
    bytes: Vec<u8>,
}
impl TypefaceIdentity {
    fn matches(&self, face: &FontFace<'_>) -> bool {
        self.native_family == face.native_family
            && self.weight == face.weight.to_bits()
            && self.italic == face.italic
            && self.face_index == face.face_index
            && self.bytes
                == if face.native_family.is_empty() {
                    face.bytes
                } else {
                    &[]
                }
    }
}
// SkTypeface_Mac::getVariationAxes initializes immutable axis metadata once
// per typeface. This adapter receives borrowed font data separately from its
// native-family CTFont identity, so cache by actual axis/validation table bytes.
// In particular, a native-family identity deliberately does not compare bytes.
struct OpticalAxisMetadata {
    tables: [Vec<u8>; 4],
    has_opsz: bool,
}
impl OpticalAxisMetadata {
    fn matches(&self, tables: &[&[u8]; 4]) -> bool {
        self.tables
            .iter()
            .zip(tables)
            .all(|(owned, current)| owned.as_slice() == *current)
    }
    fn bytes(&self) -> usize {
        self.tables.iter().map(Vec::len).sum()
    }
}

thread_local! {
    // SkStrikeCache.cpp also supports a thread-local cache. CG font owners
    // stay on their creating thread; no unsafe Send/Sync is required.
    static SCALER: RefCell<SkScalerContext_Mac> = RefCell::new(SkScalerContext_Mac::default());
    static GLYPH_IMAGES: RefCell<SkStrikeCache<ImageDescriptor, ImageKey, GlyphImage>> = RefCell::new(SkStrikeCache::default());
}
pub(crate) fn with_scaler<R>(f: impl FnOnce(&mut SkScalerContext_Mac) -> R) -> R {
    SCALER.with(|scaler| f(&mut scaler.borrow_mut()))
}
pub(crate) fn with_glyph_images<R>(
    prepared: &PreparedGlyphRun<'_>,
    paint: &PreparedGlyphPaint,
    f: impl FnOnce(&mut SkStrike<ImageKey, GlyphImage>) -> R,
) -> R {
    let remaining = prepared.remaining;
    let descriptor = ImageDescriptor {
        font: prepared.font as usize,
        matrix: [
            remaining.sx.to_bits(),
            remaining.ky.to_bits(),
            remaining.kx.to_bits(),
            remaining.sy.to_bits(),
        ],
        antialias: prepared.color_glyphs || paint.smoothing != FontSmoothing::kNone,
        smooth: !prepared.color_glyphs && paint.smooth,
        preblend: if prepared.color_glyphs {
            None
        } else {
            paint.table
        },
        foreground: prepared.color_glyphs.then(|| paint_rgba8(paint.color)),
    };
    GLYPH_IMAGES.with(|cache| cache.borrow_mut().with_strike(descriptor, f))
}

#[cfg(feature = "profiling")]
pub fn glyph_cache_stats() -> (u64, u64, u64, usize, usize, usize, usize) {
    let (hits, misses, evictions, bytes, strikes) = GLYPH_IMAGES.with(|cache| {
        let cache = cache.borrow();
        (
            cache.hits(),
            cache.misses(),
            cache.evictions(),
            cache.memory_used(),
            cache.count(),
        )
    });
    let (fonts, identity_bytes) = SCALER.with(|scaler| {
        let scaler = scaler.borrow();
        (
            scaler.fonts.len(),
            scaler.typefaces.iter().map(|face| face.bytes.len()).sum(),
        )
    });
    (
        hits,
        misses,
        evictions,
        bytes,
        strikes,
        fonts,
        identity_bytes,
    )
}

fn purge_images() {
    GLYPH_IMAGES.with(|cache| cache.borrow_mut().purge_all());
}

#[derive(Default)]
pub(crate) struct SkScalerContext_Mac {
    fonts: HashMap<(u64, u32, u32, Vec<(u32, u32)>), Owned>,
    font_bounds: HashMap<usize, NativeFontMetadata>,
    typefaces: Vec<TypefaceIdentity>,
    next_typeface: u64,
    optical_axes: Vec<OpticalAxisMetadata>,
}
impl Drop for SkScalerContext_Mac {
    fn drop(&mut self) {
        // Native addresses can be reused after release, including in tests.
        let _ = GLYPH_IMAGES.try_with(|cache| cache.borrow_mut().purge_all());
    }
}

impl SkScalerContext_Mac {
    // Existing unit-axis rotations/reflections retain the original font size.
    pub(crate) fn supports_unit_axis(t: Transform) -> bool {
        let axis = |a: f32, b: f32| (a.abs() == 1.0 && b == 0.0) || (b.abs() == 1.0 && a == 0.0);
        axis(t.sx, t.ky)
            && axis(t.kx, t.sy)
            && (t.sx * t.sy - t.kx * t.ky).abs() == 1.0
            && t.tx.is_finite()
            && t.ty.is_finite()
    }

    pub(crate) fn supports_transform(t: Transform) -> bool {
        Self::supports_unit_axis(t) || SkScalerContextRec::supports_positive_uniform_scale(t)
    }

    fn has_optical_axis(&mut self, face: &FontFace<'_>) -> bool {
        let Ok(raw) = ttf_parser::RawFace::parse(face.bytes, face.face_index & 0xffff) else {
            // Face::parse starts with exactly this RawFace parse, so an invalid
            // directory/index also failed the old complete-face query.
            return false;
        };
        let mut tables: [Option<&[u8]>; 4] = [None; 4];
        // Face::collect_tables accepts unsorted directories and lets the last
        // duplicate win. Preserve that behavior instead of RawFace::table's
        // sorted-directory binary search assumption.
        let font_offset = if face.bytes.starts_with(b"ttcf") {
            let at = 12 + ((face.face_index & 0xffff) as usize) * 4;
            u32::from_be_bytes(face.bytes[at..at + 4].try_into().unwrap()) as usize
        } else {
            0
        };
        let directory_offset = font_offset + 12;
        // RawFace already validated the collection index, face header, and this
        // complete directory. Borrow its original bytes: no checksum or unrelated
        // table fields need decoding on each repeated glyph-run query.
        let directory = &face.bytes
            [directory_offset..directory_offset + usize::from(raw.table_records.len()) * 16];
        for record in directory.chunks_exact(16) {
            let slot = match &record[..4] {
                b"fvar" => 0,
                b"head" => 1,
                b"hhea" => 2,
                b"maxp" => 3,
                _ => continue,
            };
            let offset = u32::from_be_bytes(record[8..12].try_into().unwrap()) as usize;
            let length = u32::from_be_bytes(record[12..16].try_into().unwrap()) as usize;
            if let Some(end) = offset.checked_add(length) {
                tables[slot] = face.bytes.get(offset..end);
            }
        }
        let Some(fvar) = tables[0] else {
            // An absent/unreadable fvar cannot expose opsz, regardless of
            // whether the complete Face would accept the required tables.
            return false;
        };
        // These are exactly the bytes consumed by ttf-parser's three required
        // table parsers. Their inclusion prevents a malformed font from reusing
        // a true result from a valid font with identical fvar contents.
        fn prefix(table: Option<&[u8]>, count: usize) -> &[u8] {
            let data = table.unwrap_or_default();
            &data[..data.len().min(count)]
        }
        let key = [
            fvar,
            prefix(tables[1], 54),
            prefix(tables[2], 36),
            prefix(tables[3], 6),
        ];
        if let Some(entry) = self.optical_axes.iter().find(|entry| entry.matches(&key)) {
            return entry.has_opsz;
        }
        // Cache misses keep the original parser's optional-table and failure
        // behavior. Only immutable optical-axis metadata survives this query.
        let has_opsz = ttf_parser::Face::parse(face.bytes, face.face_index & 0xffff)
            .ok()
            .is_some_and(|font| {
                font.variation_axes()
                    .into_iter()
                    .any(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"opsz"))
            });
        let size: usize = key.iter().map(|table| table.len()).sum();
        if size <= 64 * 1024 {
            if self.optical_axes.len() >= 32
                || self
                    .optical_axes
                    .iter()
                    .map(OpticalAxisMetadata::bytes)
                    .sum::<usize>()
                    .saturating_add(size)
                    > 64 * 1024
            {
                self.optical_axes.clear();
            }
            self.optical_axes.push(OpticalAxisMetadata {
                tables: key.map(<[u8]>::to_vec),
                has_opsz,
            });
        }
        has_opsz
    }

    fn font(
        &mut self,
        _index: u32,
        face: &FontFace,
        size: f32,
        device_size: f32,
        variations: &[FontVariation],
    ) -> Ref {
        let typeface = if let Some(entry) = self.typefaces.iter().find(|entry| entry.matches(face))
        {
            entry.id
        } else {
            // Bounds are local font-owner adapter limits, separate from the
            // official 2 MiB image budget. Keep at most 32 identities/64 CTFonts.
            // One oversized font may exceed the identity byte budget alone.
            let bytes: usize = self.typefaces.iter().map(|entry| entry.bytes.len()).sum();
            if self.typefaces.len() >= 32
                || bytes.saturating_add(if face.native_family.is_empty() {
                    face.bytes.len()
                } else {
                    0
                }) > 2 * 1024 * 1024
            {
                self.fonts.clear();
                // Every native font owner has been dropped before its metadata.
                self.font_bounds.clear();
                self.typefaces.clear();
                purge_images();
            }
            self.next_typeface = self
                .next_typeface
                .checked_add(1)
                .expect("typeface identity exhausted");
            self.typefaces.push(TypefaceIdentity {
                id: self.next_typeface,
                native_family: face.native_family.to_owned(),
                weight: face.weight.to_bits(),
                italic: face.italic,
                face_index: face.face_index,
                bytes: if face.native_family.is_empty() {
                    face.bytes.to_vec()
                } else {
                    Vec::new()
                },
            });
            self.next_typeface
        };
        // TypefaceForFontSize: optical size follows the painted size.
        let mut axes = variations.to_vec();
        if self.has_optical_axis(face) {
            if let Some(axis) = axes
                .iter_mut()
                .find(|axis| axis.tag == u32::from_be_bytes(*b"opsz"))
            {
                axis.value = size;
            } else {
                axes.push(FontVariation {
                    tag: u32::from_be_bytes(*b"opsz"),
                    value: size,
                });
            }
        }
        let variations = axes.as_slice();
        let key = (
            typeface,
            size.to_bits(),
            device_size.to_bits(),
            variations
                .iter()
                .map(|v| (v.tag, v.value.to_bits()))
                .collect(),
        );
        if self.fonts.len() >= 64 && !self.fonts.contains_key(&key) {
            self.fonts.clear();
            // Every native font owner has been dropped before its metadata.
            self.font_bounds.clear();
            purge_images();
        }
        self.fonts
            .entry(key)
            .or_insert_with(|| unsafe {
                let base = if !face.native_family.is_empty() {
                    let name = Owned::new(CFStringCreateWithBytes(
                        ptr::null(),
                        face.native_family.as_ptr(),
                        face.native_family.len() as isize,
                        0x08000100,
                        false,
                    ));
                    let matched = match_native_family(name.get(), face.weight, face.italic, size);
                    if CTFontGetSymbolicTraits(matched.get()) & (1 << 13) != 0 {
                        // Original SkFontMgr_mac_ct typeface starts at size0;
                        // its scaler then always makes the exact device copy.
                        // Keep that source optical/trak descriptor for colors.
                        match_native_family(name.get(), face.weight, face.italic, 0.0)
                    } else {
                        matched
                    }
                } else {
                    let data = Owned::new(CFDataCreate(
                        ptr::null(),
                        face.bytes.as_ptr(),
                        face.bytes.len() as isize,
                    ));
                    let index = (face.face_index & 0xffff) as isize;
                    // SkTypeface_mac_ct.cpp::ctfont_from_skdata uses the
                    // singular descriptor for face zero. The plural API can
                    // create a default variable instance with missing outlines
                    // on macOS; its min/max instances may still draw correctly.
                    // Nonzero collection indices keep the existing plural path.
                    let descriptor = if index == 0 {
                        CTFontManagerCreateFontDescriptorFromData(data.get())
                    } else {
                        ptr::null()
                    };
                    if !descriptor.is_null() {
                        let descriptor = Owned::new(descriptor);
                        Owned::new(CTFontCreateWithFontDescriptor(
                            descriptor.get(),
                            size.into(),
                            ptr::null(),
                        ))
                    } else {
                        // Preserve the prior collection support and invalid
                        // index assertion if the singular descriptor is absent.
                        let descriptors =
                            Owned::new(CTFontManagerCreateFontDescriptorsFromData(data.get()));
                        assert!(
                            index < CFArrayGetCount(descriptors.get()),
                            "paint font collection index is out of bounds"
                        );
                        Owned::new(CTFontCreateWithFontDescriptor(
                            CFArrayGetValueAtIndex(descriptors.get(), index),
                            size.into(),
                            ptr::null(),
                        ))
                    }
                };
                if variations.is_empty() {
                    return if device_size == size
                        && CTFontGetSymbolicTraits(base.get()) & (1 << 13) == 0
                    {
                        base
                    } else {
                        exact_copy_preserving_optical_size(base.get(), device_size, None)
                    };
                }
                let mut numbers = Vec::with_capacity(variations.len() * 2);
                let mut keys = Vec::new();
                let mut values = Vec::new();
                for axis in variations {
                    let tag = axis.tag as i32;
                    let value = f64::from(axis.value);
                    let key =
                        Owned::new(CFNumberCreate(ptr::null(), 3, ptr::from_ref(&tag).cast()));
                    let value =
                        Owned::new(CFNumberCreate(ptr::null(), 6, ptr::from_ref(&value).cast()));
                    keys.push(key.get());
                    values.push(value.get());
                    numbers.push(key);
                    numbers.push(value);
                }
                let axes = dictionary(&keys, &values);
                let attributes = dictionary(&[kCTFontVariationAttribute], &[axes.get()]);
                let descriptor = Owned::new(CTFontDescriptorCreateWithAttributes(attributes.get()));
                let varied = Owned::new(CTFontCreateCopyWithAttributes(
                    base.get(),
                    size.into(),
                    ptr::null(),
                    descriptor.get(),
                ));
                if device_size == size {
                    varied
                } else {
                    let opsz = variations
                        .iter()
                        .rev()
                        .find(|axis| axis.tag == u32::from_be_bytes(*b"opsz"))
                        .map(|axis| f64::from(axis.value));
                    exact_copy_preserving_optical_size(varied.get(), device_size, opsz)
                }
            })
            .get()
    }

    // SkGlyphRunPainter::drawForBitmapDevice creates the mask strike once per
    // run (SkStrikeSpec::MakeMask), before processing individual glyphs. This
    // restricted adapter likewise resolves variations, size and CTFont once.
    pub(crate) fn prepare_glyph_run(
        &mut self,
        face_index: u32,
        face: &FontFace,
        size: f32,
        variations: &[FontVariation],
        matrix: Transform,
    ) -> PreparedGlyphRun<'_> {
        // Font-local scale is extracted once; glyph translation stays in
        // device coordinates. Unit-axis cases retain the preceding path exactly.
        let (device_size, remaining) = if Self::supports_unit_axis(matrix) {
            (size, matrix)
        } else {
            let matrices = (SkScalerContextRec {
                fTextSize: size,
                fPost2x2: matrix,
            })
            .computeMatrices(PreMatrixScale::Vertical);
            if let Some(matrices) = matrices {
                (matrices.scale[1], matrices.remaining)
            } else {
                // The caller's original unit-axis gate may subsequently apply
                // a synthetic-italic shear. Preserve that existing path; the
                // caller rejects scaled italic before entering this adapter.
                (size, matrix)
            }
        };
        let font = self.font(face_index, face, size, device_size, variations);
        let native = self
            .font_bounds
            .entry(font as usize)
            .or_insert_with(|| unsafe { native_font_bounds(font) });
        PreparedGlyphRun {
            font,
            remaining,
            coarse_bounds: native
                .bounds
                .and_then(|bounds| device_font_bounds(bounds, remaining)),
            color_glyphs: native.color_glyphs,
            owner: core::marker::PhantomData,
        }
    }

    // SkScalerContext_mac_ct.cpp: Offscreen::getCG, generateMetrics and
    // generateImage; SkTypeface_mac_ct.cpp: onFilterRec (modern gray smoothing).
    // Compatibility driver for the existing A8 unit fixtures. The canvas
    // driver below passes the actual clip representation instead of guessing.
    #[cfg(test)]
    pub(crate) fn draw(
        pixmap: &mut Pixmap,
        strike: &mut SkStrike<ImageKey, GlyphImage>,
        clip: Option<&Mask>,
        clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
        prepared: &PreparedGlyphRun<'_>,
        glyph: u16,
        position: Transform,
        paint: &PreparedGlyphPaint,
        f16_surface: Option<&mut crate::cpu::f16_surface::Surface>,
    ) {
        let owner = crate::src::core::SkColorGlyphClip::CanvasClipOwner::device(
            crate::src::core::SkColorGlyphClip::Bounds([
                0,
                0,
                pixmap.width() as i32,
                pixmap.height() as i32,
            ]),
        );
        // This raw compatibility wrapper belongs only to the A8 fixtures.
        // Production colored calls require the actual complete Canvas owner.
        debug_assert!(!prepared.color_glyphs);
        Self::draw_with_clip_kind(
            pixmap,
            strike,
            clip,
            clip_summary,
            &owner,
            prepared,
            glyph,
            position,
            paint,
            f16_surface,
        )
    }
    pub(crate) fn draw_with_clip_kind(
        pixmap: &mut Pixmap,
        strike: &mut SkStrike<ImageKey, GlyphImage>,
        clip: Option<&Mask>,
        clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
        clip_owner: &crate::src::core::SkColorGlyphClip::CanvasClipOwner,
        prepared: &PreparedGlyphRun<'_>,
        glyph: u16,
        position: Transform,
        paint: &PreparedGlyphPaint,
        f16_surface: Option<&mut crate::cpu::f16_surface::Surface>,
    ) {
        let clip_is_aa = !clip_owner.is_bw();
        // SkDraw::paintMasks chooses BW rectangular clipping before AA-mask
        // multiplication. An opaque rectangle needs bounds intersection only.
        if clip_summary.is_some_and(|summary| summary.getBounds().is_none()) {
            return;
        }
        let clip = clip.filter(|_| {
            (prepared.color_glyphs && clip_is_aa)
                || !clip_summary.is_some_and(|summary| summary.isRect())
        });
        let font = prepared.font;
        let remaining = prepared.remaining;
        let smoothing = paint.smoothing;
        let smooth = paint.smooth;
        let x = position.tx.floor() as i32;
        let y = position.ty.floor() as i32;
        let sub_x = position.tx - x as f32;
        let sub_y = position.ty - y as f32;
        // Reject only when the entire valid font extent misses the device clip.
        // This precedes both phase-image hashing and native per-glyph metrics.
        // Never record an empty image: later visible positions keep their cache.
        if prepared.quick_reject(pixmap, clip_summary, x, y, sub_x, sub_y) {
            return;
        }
        let key = ImageKey {
            glyph,
            sub_x: sub_x.to_bits(),
            sub_y: sub_y.to_bits(),
        };
        if let Some(image) = strike.find_image(&key) {
            if let Some(image) = image {
                Self::paint_image(
                    pixmap,
                    clip,
                    clip_summary,
                    clip_owner,
                    &image,
                    x,
                    y,
                    paint,
                    f16_surface,
                );
            }
            return;
        }
        unsafe {
            let cg_bounds = CTFontGetBoundingRectsForGlyphs(font, 0, &glyph, ptr::null_mut(), 1);
            if cg_bounds.size.width <= 0.0 || cg_bounds.size.height <= 0.0 {
                strike.prepare_image(key, || None, |image| image.storage_bytes());
                return;
            }
            // SkScalerContext_Mac::MatrixToCGAffineTransform and generateMetrics:
            // bounds and text matrix use the linear CTM, with CoreGraphics Y up.
            let matrix = Affine {
                a: f64::from(remaining.sx),
                b: -f64::from(remaining.ky),
                c: -f64::from(remaining.kx),
                d: f64::from(remaining.sy),
                tx: 0.0,
                ty: 0.0,
            };
            let mut min_x = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_y = f64::NEG_INFINITY;
            for bx in [
                cg_bounds.origin.x,
                cg_bounds.origin.x + cg_bounds.size.width,
            ] {
                for by in [
                    cg_bounds.origin.y,
                    cg_bounds.origin.y + cg_bounds.size.height,
                ] {
                    let px = matrix.a * bx + matrix.c * by;
                    let py = matrix.b * bx + matrix.d * by;
                    min_x = min_x.min(px);
                    max_x = max_x.max(px);
                    min_y = min_y.min(py);
                    max_y = max_y.max(py);
                }
            }
            let left = min_x.floor() as i32 - 1;
            let top = (-max_y).floor() as i32 - 1;
            let right = (max_x + f64::from(sub_x)).ceil() as i32 + 1;
            let bottom = (-min_y + f64::from(sub_y)).ceil() as i32 + 1;
            // SkDraw_text::paintMasks intersects device glyph bounds with the
            // raster clip. Reject an offscreen cache miss before creating its
            // bitmap, without admitting a false empty image into the cache.
            // Use the same padded bounds consumed below, including subpixels.
            let mut dst_left = (x + left).max(0);
            let mut dst_top = (y + top).max(0);
            let mut dst_right = (x + right).min(pixmap.width() as i32);
            let mut dst_bottom = (y + bottom).min(pixmap.height() as i32);
            if let Some(bounds) = clip_summary.and_then(|summary| summary.getBounds()) {
                dst_left = dst_left.max(bounds.left());
                dst_top = dst_top.max(bounds.top());
                dst_right = dst_right.min(bounds.right());
                dst_bottom = dst_bottom.min(bounds.bottom());
            }
            if dst_left >= dst_right || dst_top >= dst_bottom {
                return;
            }
            if let Some(clip) = clip {
                if !(dst_top..dst_bottom).any(|row| {
                    clip.row_range(row as u32, dst_left as u32, dst_right as u32)
                        .iter()
                        .any(|&alpha| alpha != 0)
                }) {
                    return;
                }
            }
            let width = (right - left) as usize;
            let height = (bottom - top) as usize;
            let is_color = prepared.color_glyphs;
            let mut image = vec![if is_color { 0u8 } else { 255u8 }; width * height * 4];
            let space = Owned::new(CGColorSpaceCreateDeviceRGB());
            // N32 little-endian, skip-first alpha: B,G,R,X bytes.
            let context = Owned::new(CGBitmapContextCreate(
                image.as_mut_ptr().cast(),
                width,
                height,
                8,
                width * 4,
                space.get(),
                (2 << 12) | if is_color { 2 } else { 6 },
            ));
            CGContextSetAllowsFontSubpixelQuantization(context.get(), false);
            CGContextSetShouldSubpixelQuantizeFonts(context.get(), false);
            CGContextSetAllowsFontSubpixelPositioning(context.get(), true);
            CGContextSetShouldSubpixelPositionFonts(context.get(), true);
            CGContextSetTextDrawingMode(context.get(), 0);
            if is_color {
                let foreground = paint_rgba8(paint.color);
                // CGColorForSkColor forces alpha=1: whole-glyph paint alpha
                // belongs to the later sprite/image composite, exactly once.
                CGContextSetRGBFillColor(
                    context.get(),
                    f64::from(foreground[0]) * f64::from(1.0f32 / 255.0),
                    f64::from(foreground[1]) * f64::from(1.0f32 / 255.0),
                    f64::from(foreground[2]) * f64::from(1.0f32 / 255.0),
                    1.0,
                );
            } else {
                CGContextSetGrayFillColor(context.get(), 0.0, 1.0);
            }
            CGContextSetShouldAntialias(
                context.get(),
                is_color || smoothing != FontSmoothing::kNone,
            );
            CGContextSetShouldSmoothFonts(context.get(), !is_color && smooth);
            CGContextSetTextMatrix(context.get(), matrix);
            // Offscreen::getCG maps device-space glyph origin through fInvTransform
            // because CTFontDrawGlyphs receives positions in text space.
            let device_x = f64::from(-left) + f64::from(sub_x);
            let device_y = f64::from(top) + height as f64 - f64::from(sub_y);
            let determinant = matrix.a * matrix.d - matrix.b * matrix.c;
            let point = Point {
                x: (matrix.d * device_x - matrix.c * device_y) / determinant,
                y: (-matrix.b * device_x + matrix.a * device_y) / determinant,
            };
            CTFontDrawGlyphs(font, &glyph, &point, 1, context.get());
            // Release the context before reading its borrowed pixel storage.
            drop(context);
            // Cache the whole unclipped glyph image, as SkGlyph::setImage does.
            // Clip/paint alpha/integer position remain per-frame operations.
            let pixels = if is_color {
                for pixel in image.chunks_exact_mut(4) {
                    pixel.swap(0, 2);
                }
                GlyphPixels::Argb32(image)
            } else {
                let mut a8 = Vec::with_capacity(width * height);
                for p in image.chunks_exact(4) {
                    let component = |v: u8| {
                        if smooth {
                            255 - ((u32::from(v) * u32::from(v) + 128) / 255) as u8
                        } else {
                            255 - v
                        }
                    };
                    let r = u32::from(component(p[2]));
                    let g = u32::from(component(p[1]));
                    let b = u32::from(component(p[0]));
                    let mut coverage = ((r * 54 + g * 183 + b * 19) >> 8) as u8;
                    if let Some(table) = &paint.table {
                        coverage = table[coverage as usize];
                    }
                    a8.push(coverage);
                }
                GlyphPixels::A8(a8)
            };
            let image = strike
                .prepare_image(
                    key,
                    || {
                        Some(GlyphImage {
                            left,
                            top,
                            width,
                            height,
                            pixels,
                        })
                    },
                    |image| image.storage_bytes(),
                )
                .expect("generated glyph image");
            Self::paint_image(
                pixmap,
                clip,
                clip_summary,
                clip_owner,
                &image,
                x,
                y,
                paint,
                f16_surface,
            );
        }
    }
    fn paint_image(
        pixmap: &mut Pixmap,
        clip: Option<&Mask>,
        clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
        clip_owner: &crate::src::core::SkColorGlyphClip::CanvasClipOwner,
        image: &GlyphImage,
        x: i32,
        y: i32,
        paint: &PreparedGlyphPaint,
        mut f16_surface: Option<&mut crate::cpu::f16_surface::Surface>,
    ) {
        let clip_is_aa = !clip_owner.is_bw();
        // Query original full mask allocation, including transparent padding,
        // before cropping its visible destination. No ink/dense bbox shortcut.
        let whole = [
            i64::from(x) + i64::from(image.left),
            i64::from(y) + i64::from(image.top),
            i64::from(x) + i64::from(image.left) + image.width as i64,
            i64::from(y) + i64::from(image.top) + image.height as i64,
        ];
        if whole[2] <= 0
            || whole[3] <= 0
            || whole[0] >= i64::from(pixmap.width())
            || whole[1] >= i64::from(pixmap.height())
        {
            return;
        }
        let whole_glyph = crate::src::core::SkColorGlyphClip::Bounds(whole.map(|v| v as i32));
        let mut dst_left = (whole[0].max(0)) as i32;
        let mut dst_top = (whole[1].max(0)) as i32;
        let mut dst_right = whole[2].min(i64::from(pixmap.width())) as i32;
        let mut dst_bottom = whole[3].min(i64::from(pixmap.height())) as i32;
        if let Some(bounds) = clip_summary.and_then(|summary| summary.getBounds()) {
            dst_left = dst_left.max(bounds.left());
            dst_top = dst_top.max(bounds.top());
            dst_right = dst_right.min(bounds.right());
            dst_bottom = dst_bottom.min(bounds.bottom());
        }
        if dst_left >= dst_right || dst_top >= dst_bottom {
            return;
        }
        let source_left = (dst_left - x - image.left) as usize;
        let source_top = (dst_top - y - image.top) as usize;
        let visible_width = (dst_right - dst_left) as usize;
        let visible_height = (dst_bottom - dst_top) as usize;
        if let GlyphPixels::Argb32(pixels) = &image.pixels {
            let handles_sprite = clip_owner.composite_mode(f16_surface.is_some(), whole_glyph)
                == crate::src::core::SkColorGlyphClip::CompositeMode::SpriteN32;
            let alpha8 = paint_rgba8(paint.color)[3];
            let format = pixmap.format;
            let stride = pixmap.width() as usize;
            for row in 0..visible_height {
                let source = ((source_top + row) * image.width + source_left) * 4;
                let source = &pixels[source..source + visible_width * 4];
                let pixel_start = (dst_top as usize + row) * stride + dst_left as usize;
                let coverage = clip.map(|mask| {
                    mask.row_range(
                        dst_top as u32 + row as u32,
                        dst_left as u32,
                        dst_right as u32,
                    )
                });
                let bytes =
                    &mut pixmap.data_mut()[pixel_start * 4..(pixel_start + visible_width) * 4];
                for (col, (src, dst)) in source
                    .chunks_exact(4)
                    .zip(bytes.chunks_exact_mut(4))
                    .enumerate()
                {
                    let coverage = coverage.map_or(255, |row| row[col]);
                    if coverage == 0 {
                        continue;
                    }
                    if let Some(surface) = f16_surface.as_mut() {
                        // F16 uses the existing HIGHP image composite. No RGB
                        // tint or gamma. Alpha and clip remain distinct stages.
                        surface.composite_n32_span(
                            pixel_start + col,
                            src,
                            dst,
                            paint.color.alpha.clamp(0.0, 1.0),
                            coverage,
                            format,
                        );
                    } else {
                        let mut source: [u8; 4] = src.try_into().unwrap();
                        if handles_sprite {
                            // BW clip is 0/255; ChooseSprite uses paint alpha
                            // as the global-alpha flag, not AA coverage.
                            color_sprite_pixel(dst, source, alpha8, format);
                        } else {
                            // Linked oracle has SkImageShader::onMakeContext
                            // (nm checked). Integer nearest image -> legacy
                            // shader alpha first, then AA SkBlitRow global alpha.
                            let scale = u32::from(alpha8) + 1;
                            source = source.map(|c| ((u32::from(c) * scale) >> 8) as u8);
                            color_sprite_pixel(dst, source, coverage, format);
                        }
                    }
                }
            }
            return;
        }
        let GlyphPixels::A8(a8) = &image.pixels else {
            unreachable!()
        };
        let stride = pixmap.width() as usize;
        // Unclipped masks are borrowed directly; only clipping needs scratch.
        let mut clipped_row = if clip.is_some() {
            vec![0; visible_width]
        } else {
            Vec::new()
        };
        let format = pixmap.format;
        let format_blitter = if format == crate::PixelFormat::Rgba8888 {
            None
        } else {
            Some(paint.blitter.with_format(format))
        };
        for row in 0..visible_height {
            let source = (source_top + row) * image.width + source_left;
            let image_row = &a8[source..source + visible_width];
            let pixel_start = (dst_top as usize + row) * stride + dst_left as usize;
            let coverage_row = if let Some(clip) = clip {
                let clip_row = clip.row_range(
                    dst_top as u32 + row as u32,
                    dst_left as u32,
                    dst_right as u32,
                );
                if full_coverage_row(clip_row) {
                    image_row
                } else {
                    for ((dst, &src), &alpha) in clipped_row.iter_mut().zip(image_row).zip(clip_row)
                    {
                        *dst = crate::cpu::mask_blitter::mul_div_255_round(src, alpha);
                    }
                    clipped_row.as_slice()
                }
            } else {
                image_row
            };
            let bytes = &mut pixmap.data_mut()[pixel_start * 4..(pixel_start + visible_width) * 4];
            if let Some(surface) = f16_surface.as_mut() {
                for (col, (dst, &coverage)) in
                    bytes.chunks_exact_mut(4).zip(coverage_row).enumerate()
                {
                    surface.blend(pixel_start + col, paint.color, coverage);
                    dst.copy_from_slice(&format.encode(surface.rgba8(pixel_start + col)));
                }
            } else {
                format_blitter
                    .as_ref()
                    .unwrap_or(&paint.blitter)
                    .blend_row(bytes, coverage_row);
            }
        }
    }
}

// ARM macOS SkBlitRow src-alpha / global+src-alpha arithmetic, translating
// SkBlitRow_opts.h and SkBlitRow_D32.cpp. Scalar correctness candidate; the
// source oracle must be compared before production adoption or vectorization.
fn color_sprite_pixel(dst: &mut [u8], src: [u8; 4], alpha: u8, format: crate::PixelFormat) {
    if alpha == 0 {
        return;
    }
    let source = format.swizzle(src);
    if alpha == 255 {
        for c in 0..4 {
            let product = u32::from(dst[c]) * (255 - u32::from(source[3]));
            let p = product + 128;
            dst[c] = (u32::from(source[c]) + ((p + (p >> 8)) >> 8)) as u8;
        }
    } else {
        let scale = u32::from(alpha) + 1;
        let inverse = 65535 - u32::from(source[3]) * scale;
        let inverse = (inverse + (inverse >> 8)) >> 8;
        for c in 0..4 {
            dst[c] = ((u32::from(source[c]) * scale + u32::from(dst[c]) * inverse) >> 8) as u8;
        }
    }
    if format == crate::PixelFormat::Bgrx8888 {
        dst[3] = 0;
    }
}

// SkAAClipBlitter forwards opaque AA runs to its wrapped blitter unchanged.
// The dense-mask Rust adapter recognizes a full row by bounded packed reads.
fn full_coverage_row(row: &[u8]) -> bool {
    let mut index = 0;
    while index + 8 <= row.len() {
        if unsafe { core::ptr::read_unaligned(row.as_ptr().add(index).cast::<u64>()) } != u64::MAX {
            return false;
        }
        index += 8;
    }
    row[index..].iter().all(|&alpha| alpha == 255)
}

pub(crate) use SkScalerContext_Mac as SkScalerContextMac;

#[cfg(test)]
mod matrix_and_optical_size_tests {
    use super::*;

    fn test_face() -> FontFace<'static> {
        FontFace {
            family: "Roboto",
            native_family: "",
            weight: 400.0,
            italic: false,
            bytes: include_bytes!(
                "../../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
            ),
            face_index: 0,
            variations: Vec::new(),
        }
    }

    fn font_with_optical_axis() -> Vec<u8> {
        let face = test_face();
        let raw = ttf_parser::RawFace::parse(face.bytes, 0).unwrap();
        let mut tables: Vec<_> = raw
            .table_records
            .into_iter()
            .map(|r| {
                (
                    r.tag.to_bytes(),
                    face.bytes[r.offset as usize..(r.offset + r.length) as usize].to_vec(),
                )
            })
            .collect();
        let mut fvar = Vec::new();
        fvar.extend_from_slice(&0x00010000_u32.to_be_bytes());
        for value in [16_u16, 2, 1, 20, 0, 8] {
            fvar.extend_from_slice(&value.to_be_bytes());
        }
        fvar.extend_from_slice(b"opsz");
        for value in [8_i32, 14, 72] {
            fvar.extend_from_slice(&(value << 16).to_be_bytes());
        }
        fvar.extend_from_slice(&[0, 0, 0, 1]);
        tables.push((*b"fvar", fvar));
        tables.sort_by_key(|(tag, _)| *tag);
        let mut output = vec![0; 12 + tables.len() * 16];
        output[..4].copy_from_slice(&0x00010000_u32.to_be_bytes());
        output[4..6].copy_from_slice(&(tables.len() as u16).to_be_bytes());
        for (i, (tag, bytes)) in tables.iter().enumerate() {
            while output.len() % 4 != 0 {
                output.push(0);
            }
            let offset = output.len() as u32;
            let at = 12 + i * 16;
            output[at..at + 4].copy_from_slice(tag);
            output[at + 8..at + 12].copy_from_slice(&offset.to_be_bytes());
            output[at + 12..at + 16].copy_from_slice(&(bytes.len() as u32).to_be_bytes());
            output.extend_from_slice(bytes);
        }
        output
    }

    fn old_has_optical_axis(face: &FontFace<'_>) -> bool {
        ttf_parser::Face::parse(face.bytes, face.face_index & 0xffff)
            .ok()
            .is_some_and(|font| {
                font.variation_axes()
                    .into_iter()
                    .any(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"opsz"))
            })
    }

    #[test]
    fn optical_metadata_matches_old_parser_across_sources_and_malformed_tables() {
        let valid = font_with_optical_axis();
        let mut no_opsz = valid.clone();
        let raw = ttf_parser::RawFace::parse(&valid, 0).unwrap();
        let fvar = raw
            .table_records
            .into_iter()
            .find(|r| r.tag == ttf_parser::Tag::from_bytes(b"fvar"))
            .unwrap();
        no_opsz[fvar.offset as usize + 16..fvar.offset as usize + 20].copy_from_slice(b"wght");
        let mut bad_head = valid.clone();
        let head = raw
            .table_records
            .into_iter()
            .find(|r| r.tag == ttf_parser::Tag::from_bytes(b"head"))
            .unwrap();
        bad_head[head.offset as usize + 18..head.offset as usize + 20].fill(0);
        let mut bad_hhea = valid.clone();
        let hhea = raw
            .table_records
            .into_iter()
            .enumerate()
            .find(|(_, r)| r.tag == ttf_parser::Tag::from_bytes(b"hhea"))
            .unwrap()
            .0;
        bad_hhea[12 + hhea * 16 + 12..12 + hhea * 16 + 16].copy_from_slice(&1_u32.to_be_bytes());
        let mut bad_maxp = valid.clone();
        let maxp = raw
            .table_records
            .into_iter()
            .find(|r| r.tag == ttf_parser::Tag::from_bytes(b"maxp"))
            .unwrap();
        bad_maxp[maxp.offset as usize + 4..maxp.offset as usize + 6].fill(0);
        let mut bad_fvar = valid.clone();
        bad_fvar[fvar.offset as usize..fvar.offset as usize + 4].fill(0);
        let mut unsorted = valid.clone();
        let count = raw.table_records.len() as usize;
        for i in 0..count {
            unsorted[12 + i * 16..12 + (i + 1) * 16]
                .copy_from_slice(&valid[12 + (count - 1 - i) * 16..12 + (count - i) * 16]);
        }
        let mut scaler = SkScalerContext_Mac::default();
        for bytes in [
            valid.as_slice(),
            no_opsz.as_slice(),
            bad_head.as_slice(),
            bad_hhea.as_slice(),
            bad_maxp.as_slice(),
            bad_fvar.as_slice(),
            unsorted.as_slice(),
            test_face().bytes,
            b"invalid",
        ] {
            let mut face = test_face();
            face.bytes = bytes;
            face.native_family = "Arial";
            assert_eq!(scaler.has_optical_axis(&face), old_has_optical_axis(&face));
            assert_eq!(scaler.has_optical_axis(&face), old_has_optical_axis(&face));
            for face_index in [0, 1, 0x10000, 0x10001] {
                face.face_index = face_index;
                assert_eq!(scaler.has_optical_axis(&face), old_has_optical_axis(&face));
            }
        }
        // Native-family identity deliberately matches different bytes. Metadata
        // must therefore use actual tables, never that identity alone.
        let mut face = test_face();
        face.native_family = "Arial";
        let identity = TypefaceIdentity {
            id: 1,
            native_family: "Arial".into(),
            weight: face.weight.to_bits(),
            italic: face.italic,
            face_index: 0,
            bytes: Vec::new(),
        };
        assert!(identity.matches(&face));
        face.bytes = &valid;
        assert!(identity.matches(&face));
        assert!(scaler.has_optical_axis(&face));
        face.bytes = &bad_head;
        assert!(!scaler.has_optical_axis(&face));
    }

    #[test]
    fn optical_metadata_handles_duplicate_directory_records_and_collections() {
        let valid = font_with_optical_axis();
        let raw = ttf_parser::RawFace::parse(&valid, 0).unwrap();
        let directory_end = 12 + usize::from(raw.table_records.len()) * 16;
        let fvar = raw.table(ttf_parser::Tag::from_bytes(b"fvar")).unwrap();
        let mut duplicate = valid[..directory_end].to_vec();
        for record in duplicate[12..].chunks_exact_mut(16) {
            let offset = u32::from_be_bytes(record[8..12].try_into().unwrap());
            record[8..12].copy_from_slice(&(offset + 16).to_be_bytes());
        }
        duplicate[4..6].copy_from_slice(&(raw.table_records.len() + 1).to_be_bytes());
        duplicate.extend_from_slice(&[0; 16]);
        duplicate.extend_from_slice(&valid[directory_end..]);
        let offset = duplicate.len() as u32;
        let mut final_fvar = fvar.to_vec();
        final_fvar[16..20].copy_from_slice(b"wght");
        duplicate.extend_from_slice(&final_fvar);
        duplicate[directory_end..directory_end + 4].copy_from_slice(b"fvar");
        duplicate[directory_end + 8..directory_end + 12].copy_from_slice(&offset.to_be_bytes());
        duplicate[directory_end + 12..directory_end + 16]
            .copy_from_slice(&(final_fvar.len() as u32).to_be_bytes());
        let mut scaler = SkScalerContext_Mac::default();
        let mut face = test_face();
        face.bytes = &duplicate;
        assert!(!old_has_optical_axis(&face));
        assert!(!scaler.has_optical_axis(&face));
        let mut invalid_duplicate = duplicate.clone();
        invalid_duplicate[directory_end + 8..directory_end + 12]
            .copy_from_slice(&u32::MAX.to_be_bytes());
        let mut invalid_face = test_face();
        invalid_face.bytes = &invalid_duplicate;
        assert_eq!(
            scaler.has_optical_axis(&invalid_face),
            old_has_optical_axis(&invalid_face)
        );
        let collection =
            include_bytes!("../../../../../src/third_party/skia/resources/fonts/test.ttc");
        face.bytes = collection;
        for face_index in [0, 1, 2, 3, 0x10000, 0x10001] {
            face.face_index = face_index;
            assert_eq!(scaler.has_optical_axis(&face), old_has_optical_axis(&face));
        }
    }

    #[test]
    fn optical_metadata_cache_is_bounded_without_retaining_complete_fonts() {
        let valid = font_with_optical_axis();
        let raw = ttf_parser::RawFace::parse(&valid, 0).unwrap();
        let fvar = raw
            .table_records
            .into_iter()
            .find(|r| r.tag == ttf_parser::Tag::from_bytes(b"fvar"))
            .unwrap();
        let mut scaler = SkScalerContext_Mac::default();
        for name_id in 0..80_u16 {
            let mut bytes = valid.clone();
            bytes[fvar.offset as usize + 34..fvar.offset as usize + 36]
                .copy_from_slice(&name_id.to_be_bytes());
            let mut face = test_face();
            face.bytes = &bytes;
            assert!(scaler.has_optical_axis(&face));
            assert!(scaler.optical_axes.len() <= 32);
            assert!(
                scaler
                    .optical_axes
                    .iter()
                    .map(OpticalAxisMetadata::bytes)
                    .sum::<usize>()
                    <= 64 * 1024
            );
        }
        assert!(
            scaler
                .optical_axes
                .iter()
                .map(OpticalAxisMetadata::bytes)
                .sum::<usize>()
                < valid.len()
        );
    }

    #[test]
    fn optical_metadata_keeps_paint_device_and_explicit_variation_font_keys() {
        let variable = font_with_optical_axis();
        let mut static_face = test_face();
        static_face.native_family = "Arial";
        let mut variable_face = test_face();
        variable_face.native_family = "Arial";
        variable_face.bytes = &variable;
        let explicit = [FontVariation {
            tag: u32::from_be_bytes(*b"opsz"),
            value: 11.5,
        }];
        let mut scaler = SkScalerContext_Mac::default();
        let static_font = scaler.font(0, &static_face, 14.0, 28.0, &explicit);
        let variable_14 = scaler.font(0, &variable_face, 14.0, 28.0, &explicit);
        let variable_7 = scaler.font(0, &variable_face, 7.0, 28.0, &explicit);
        unsafe {
            for font in [static_font, variable_14, variable_7] {
                assert_eq!(CTFontGetSize(font), 28.0);
            }
            assert_eq!(optical_size(static_font), 11.5);
            assert_eq!(optical_size(variable_14), 14.0);
            assert_eq!(optical_size(variable_7), 7.0);
        }
        assert_eq!(scaler.typefaces.len(), 1, "same native-family identity");
        assert_eq!(
            scaler.fonts.len(),
            3,
            "source-dependent opsz andpaintsize remain infontkey"
        );
        assert_eq!(
            scaler.font(0, &static_face, 14.0, 28.0, &explicit),
            static_font
        );
        assert_eq!(
            scaler.font(0, &variable_face, 14.0, 28.0, &explicit),
            variable_14
        );
        assert_eq!(
            scaler.font(0, &variable_face, 7.0, 28.0, &explicit),
            variable_7
        );
    }

    #[test]
    #[ignore = "manual Debug timing; no performance assertion"]
    fn optical_metadata_debug_benchmark() {
        use std::time::Instant;
        let fixture = font_with_optical_axis();
        for (name, bytes) in [("synthetic_opsz", fixture)].into_iter().chain(
            [0, 1, 2, 3, 5, 9, 10, 11].into_iter().map(|i| {
                let path = format!(
                "../../artifacts/live-baidu-fidelity/cpp-production-resource-fresh-2/blob-{i}.bin"
            );
                (
                    Box::leak(format!("baidu_blob{i}").into_boxed_str()) as &str,
                    std::fs::read(path).unwrap(),
                )
            }),
        ) {
            let mut face = test_face();
            face.bytes = &bytes;
            face.native_family = "Arial";
            let mut scaler = SkScalerContext_Mac::default();
            assert_eq!(scaler.has_optical_axis(&face), old_has_optical_axis(&face));
            let mut before = Vec::new();
            let mut after = Vec::new();
            for _ in 0..3 {
                for old in [true, false] {
                    let start = Instant::now();
                    for _ in 0..4096 {
                        std::hint::black_box(if old {
                            old_has_optical_axis(std::hint::black_box(&face))
                        } else {
                            scaler.has_optical_axis(std::hint::black_box(&face))
                        });
                    }
                    let ms = start.elapsed().as_secs_f64() * 1000.0;
                    if old {
                        before.push(ms)
                    } else {
                        after.push(ms)
                    }
                }
            }
            before.sort_by(f64::total_cmp);
            after.sort_by(f64::total_cmp);
            eprintln!("OPTICAL_BENCH name={name} bytes={} has_opsz={} calls=4096 old_face_ms={:.3} new_metadata_ms={:.3} speedup={:.2}",bytes.len(),old_has_optical_axis(&face),before[1],after[1],before[1]/after[1]);
        }
    }

    unsafe fn optical_size(font: Ref) -> f64 {
        let name = b"NSCTFontOpticalSizeAttribute";
        let key = Owned::new(CFStringCreateWithBytes(
            ptr::null(),
            name.as_ptr(),
            name.len() as isize,
            0x08000100,
            false,
        ));
        let value = Owned::new(CTFontCopyAttribute(font, key.get()));
        assert_eq!(CFGetTypeID(value.get()), CFNumberGetTypeID());
        let mut output = 0.0f64;
        assert!(CFNumberGetValue(
            value.get(),
            13,
            ptr::from_mut(&mut output).cast()
        ));
        output
    }

    fn render_cached(
        scaler: &mut SkScalerContext_Mac,
        face: &FontFace<'_>,
        transform: Transform,
        position: Transform,
        color: Color,
        clipped: bool,
        f16: bool,
    ) -> Vec<u8> {
        let prepared = scaler.prepare_glyph_run(0, face, 14.0, &[], transform);
        let paint = PreparedGlyphPaint::new(color, FontSmoothing::kAuto);
        let mut pixmap = Pixmap::new(96, 72).unwrap();
        pixmap.fill(crate::raster::Color::WHITE);
        let mut mask = Mask::new(96, 72).unwrap();
        for (index, alpha) in mask.data_mut().iter_mut().enumerate() {
            *alpha = (index % 256) as u8;
        }
        let mut surface = f16.then(|| crate::cpu::f16_surface::Surface::new(96 * 72));
        if let Some(surface) = &mut surface {
            // Mirror the opaque white starting target.
            for index in 0..96 * 72 {
                surface.blend(
                    index,
                    Color {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                    255,
                );
            }
        }
        let glyph = ttf_parser::Face::parse(face.bytes, 0)
            .unwrap()
            .glyph_index('R')
            .unwrap()
            .0;
        with_glyph_images(&prepared, &paint, |strike| {
            SkScalerContext_Mac::draw(
                &mut pixmap,
                strike,
                clipped.then_some(&mask),
                None,
                &prepared,
                glyph,
                position,
                &paint,
                surface.as_mut(),
            );
        });
        pixmap.data().to_vec()
    }

    #[test]
    fn warm_images_equal_fresh_with_color_phase_transform_clip_and_f16_changes() {
        purge_images();
        let face = test_face();
        let mut scaler = SkScalerContext_Mac::default();
        let black = Color {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        let translucent = Color {
            red: 0.75,
            green: 0.25,
            blue: 0.5,
            alpha: 0.43,
        };
        for (scale, px, py, color, clipped, f16) in [
            (1.0, 12.25, 31.5, black, false, false),
            (1.0, 18.25, 36.5, black, true, false),
            (1.0, -4.75, 36.5, black, false, false),
            (1.0, 18.75, 36.125, black, false, false),
            (2.0, 18.75, 48.125, translucent, true, false),
            (1.0, 18.25, 36.5, translucent, false, true),
        ] {
            let matrix = Transform::from_scale(scale, scale);
            let position = Transform::from_translate(px, py);
            let first = render_cached(&mut scaler, &face, matrix, position, color, clipped, f16);
            let misses = GLYPH_IMAGES.with(|cache| cache.borrow().misses());
            let warm = render_cached(&mut scaler, &face, matrix, position, color, clipped, f16);
            assert_eq!(GLYPH_IMAGES.with(|cache| cache.borrow().misses()), misses);
            assert_eq!(first, warm);
            purge_images();
            let fresh = render_cached(&mut scaler, &face, matrix, position, color, clipped, f16);
            assert_eq!(warm, fresh);
        }
    }

    #[test]
    fn offscreen_miss_does_not_poison_visible_image_and_font_index_is_not_identity() {
        purge_images();
        let face = test_face();
        let mut scaler = SkScalerContext_Mac::default();
        let color = Color {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        let misses = GLYPH_IMAGES.with(|cache| cache.borrow().misses());
        render_cached(
            &mut scaler,
            &face,
            Transform::identity(),
            Transform::from_translate(200.25, 31.5),
            color,
            false,
            false,
        );
        assert_eq!(GLYPH_IMAGES.with(|cache| cache.borrow().misses()), misses);
        let visible = render_cached(
            &mut scaler,
            &face,
            Transform::identity(),
            Transform::from_translate(12.25, 31.5),
            color,
            false,
            false,
        );
        assert!(visible
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
        let regular = scaler.font(0, &face, 14.0, 14.0, &[]);
        let mut other = test_face();
        other.italic = true;
        let italic = scaler.font(0, &other, 14.0, 14.0, &[]);
        assert_ne!(regular, italic);
        assert_eq!(scaler.typefaces.len(), 2);
    }

    #[test]
    fn opaque_aa_row_probe_checks_unaligned_guards_and_every_position() {
        for length in 0..=129 {
            for offset in 0..8 {
                let mut data = vec![0; length + offset + 9];
                data[offset..offset + length].fill(255);
                assert!(full_coverage_row(&data[offset..offset + length]));
                for index in 0..length {
                    data[offset + index] = 254;
                    assert!(!full_coverage_row(&data[offset..offset + length]));
                    data[offset + index] = 255;
                }
            }
        }
    }

    fn render_coarse_scene(
        scaler: &mut SkScalerContext_Mac,
        face: &FontFace<'_>,
        matrix: Transform,
        clipped: bool,
        reject_enabled: bool,
    ) -> Vec<u8> {
        let mut prepared = scaler.prepare_glyph_run(0, face, 14.0, &[], matrix);
        assert!(
            prepared.coarse_bounds.is_some(),
            "test font must have valid bounds"
        );
        if !reject_enabled {
            prepared.coarse_bounds = None;
        }
        let paint = PreparedGlyphPaint::new(
            Color {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            },
            FontSmoothing::kAuto,
        );
        let mut pixmap = Pixmap::new(96, 72).unwrap();
        pixmap.fill(crate::raster::Color::WHITE);
        let mut mask = Mask::new(96, 72).unwrap();
        for y in 5..67 {
            mask.data_mut()[y * 96 + 7..y * 96 + 89].fill(255);
        }
        let summary =
            crate::src::core::SkRasterClip::SkRasterClip::from_mask(96, 72, mask.data()).unwrap();
        let font = ttf_parser::Face::parse(face.bytes, 0).unwrap();
        with_glyph_images(&prepared, &paint, |strike| {
            for ch in ['R', 'f', 'j', 'g', 'W', '_'] {
                let Some(glyph) = font.glyph_index(ch) else {
                    continue;
                };
                for (x, y) in [
                    (-200.25, 30.375),
                    (200.25, 30.375),
                    (44.375, -200.25),
                    (44.375, 200.25),
                    (-5.75, 30.375),
                    (0.125, 30.375),
                    (7.5, 5.125),
                    (88.75, 66.875),
                    (95.5, 71.5),
                    (96.125, 72.125),
                    (115.75, 30.375),
                    (44.375, -0.75),
                    (44.375, 0.125),
                    (44.375, 71.5),
                    (44.375, 72.125),
                    (44.375, 95.75),
                ] {
                    SkScalerContext_Mac::draw(
                        &mut pixmap,
                        strike,
                        clipped.then_some(&mask),
                        clipped.then_some(&summary),
                        &prepared,
                        glyph.0,
                        Transform::from_translate(x, y),
                        &paint,
                        None,
                    );
                }
            }
        });
        pixmap.data().to_vec()
    }

    #[test]
    fn coarse_font_reject_matches_unculled_cached_boundary_phase_scale_and_italic() {
        let mut italic = test_face();
        italic.bytes = include_bytes!(
            "../../../../../src/third_party/skia/resources/fonts/cond-bold-italic.ttf"
        );
        italic.italic = true;
        for face in [test_face(), italic] {
            let mut scaler = SkScalerContext_Mac::default();
            for matrix in [
                Transform::identity(),
                Transform::from_scale(0.5, 0.5),
                Transform::from_scale(2.0, 2.0),
                Transform::from_row(-1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
                Transform::from_row(1.0, 0.0, 0.0, -1.0, 0.0, 0.0),
                Transform::from_row(-1.0, 0.0, 0.0, -1.0, 0.0, 0.0),
                Transform::from_row(0.0, 1.0, -1.0, 0.0, 0.0, 0.0),
                Transform::from_row(0.0, -1.0, 1.0, 0.0, 0.0, 0.0),
                Transform::from_row(0.0, 1.0, 1.0, 0.0, 0.0, 0.0),
                Transform::from_row(0.0, -1.0, -1.0, 0.0, 0.0, 0.0),
                Transform::from_row(1.0, 0.0, -0.21, 1.0, 0.0, 0.0),
            ] {
                for clipped in [false, true] {
                    purge_images();
                    let reference = render_coarse_scene(&mut scaler, &face, matrix, clipped, false);
                    let warm = render_coarse_scene(&mut scaler, &face, matrix, clipped, true);
                    assert_eq!(
                        warm, reference,
                        "warm: matrix={matrix:?}, clipped={clipped}"
                    );
                    purge_images();
                    let cold = render_coarse_scene(&mut scaler, &face, matrix, clipped, true);
                    assert_eq!(
                        cold, reference,
                        "cold: matrix={matrix:?}, clipped={clipped}"
                    );
                }
            }
        }
    }

    #[test]
    fn coarse_font_reject_skips_strike_lookup_and_disables_variable_or_color_bounds() {
        purge_images();
        let face = test_face();
        let mut scaler = SkScalerContext_Mac::default();
        let prepared = scaler.prepare_glyph_run(0, &face, 14.0, &[], Transform::identity());
        let paint = PreparedGlyphPaint::new(
            Color {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            },
            FontSmoothing::kAuto,
        );
        let glyph = ttf_parser::Face::parse(face.bytes, 0)
            .unwrap()
            .glyph_index('R')
            .unwrap()
            .0;
        let mut pixmap = Pixmap::new(96, 72).unwrap();
        with_glyph_images(&prepared, &paint, |strike| {
            // Admit the same phase image first, so an offscreen lookup would
            // increment hits even without generating a new image.
            SkScalerContext_Mac::draw(
                &mut pixmap,
                strike,
                None,
                None,
                &prepared,
                glyph,
                Transform::from_translate(12.25, 30.375),
                &paint,
                None,
            );
            let counters = (strike.hits(), strike.misses());
            SkScalerContext_Mac::draw(
                &mut pixmap,
                strike,
                None,
                None,
                &prepared,
                glyph,
                Transform::from_translate(200.25, 30.375),
                &paint,
                None,
            );
            assert_eq!((strike.hits(), strike.misses()), counters);
        });
        drop(prepared);
        for bytes in [
            include_bytes!("../../../../../src/third_party/skia/resources/fonts/Variable.ttf")
                .as_slice(),
            include_bytes!("../../../../../src/third_party/skia/resources/fonts/sbix.ttf")
                .as_slice(),
        ] {
            let mut invalid = test_face();
            invalid.bytes = bytes;
            let prepared = scaler.prepare_glyph_run(0, &invalid, 14.0, &[], Transform::identity());
            assert!(prepared.coarse_bounds.is_none());
        }
    }

    #[test]
    fn positive_uniform_scale_extends_support_without_accepting_skew_or_scaled_reflection() {
        for scale in [0.25, 0.5, 1.0, 1.5, 2.0, 3.0] {
            assert!(SkScalerContext_Mac::supports_transform(
                Transform::from_row(scale, 0.0, 0.0, scale, 12.5, 36.25)
            ));
        }
        for (sx, ky, kx, sy) in [
            (1.0, 0.0, 0.0, 1.0),
            (-1.0, 0.0, 0.0, 1.0),
            (1.0, 0.0, 0.0, -1.0),
            (-1.0, 0.0, 0.0, -1.0),
            (0.0, 1.0, -1.0, 0.0),
            (0.0, -1.0, 1.0, 0.0),
            (0.0, 1.0, 1.0, 0.0),
            (0.0, -1.0, -1.0, 0.0),
        ] {
            let matrix = Transform::from_row(sx, ky, kx, sy, 4.25, 8.0);
            assert!(SkScalerContext_Mac::supports_unit_axis(matrix));
            assert!(SkScalerContext_Mac::supports_transform(matrix));
        }
        for matrix in [
            Transform::from_scale(2.0, 3.0),
            Transform::from_scale(-2.0, -2.0),
            Transform::from_row(2.0, 0.0, -0.5, 2.0, 0.0, 0.0),
            Transform::from_scale(0.0, 0.0),
            Transform::from_scale(f32::NAN, f32::NAN),
        ] {
            assert!(!SkScalerContext_Mac::supports_transform(matrix));
        }
    }

    #[test]
    fn resized_ctfont_uses_device_size_and_keeps_original_optical_size() {
        let face = test_face();
        let mut scaler = SkScalerContext_Mac::default();
        let size_14_scaled_2 = scaler.font(0, &face, 14.0, 28.0, &[]);
        let size_7_scaled_4 = scaler.font(0, &face, 7.0, 28.0, &[]);
        unsafe {
            assert_eq!(CTFontGetSize(size_14_scaled_2), 28.0);
            assert_eq!(CTFontGetSize(size_7_scaled_4), 28.0);
            assert_eq!(optical_size(size_14_scaled_2), 14.0);
            assert_eq!(optical_size(size_7_scaled_4), 7.0);
        }
        assert_eq!(
            scaler.fonts.len(),
            2,
            "logical size remains in device font cache key"
        );
        assert_eq!(scaler.font(0, &face, 14.0, 28.0, &[]), size_14_scaled_2);
    }

    #[test]
    fn exact_copy_preserves_explicit_opsz_over_device_text_size() {
        let face = test_face();
        let mut scaler = SkScalerContext_Mac::default();
        let base = scaler.font(0, &face, 14.0, 14.0, &[]);
        unsafe {
            let copied = exact_copy_preserving_optical_size(base, 28.0, Some(11.5));
            assert_eq!(CTFontGetSize(copied.get()), 28.0);
            assert_eq!(optical_size(copied.get()), 11.5);
        }
    }
}

#[cfg(test)]
mod native_allocation_bounds_tests {
    use super::*;
    #[test]
    fn per_glyph_bounds_equal_existing_native_mask_allocation() {
        let sources: [&[u8]; 3] = [
            include_bytes!(
                "../../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
            ),
            include_bytes!("../../../../../src/third_party/skia/resources/fonts/Distortable.ttf"),
            include_bytes!("../../../../../src/third_party/skia/resources/fonts/Variable.ttf"),
        ];
        let mut drawn = 0;
        let mut variable_fonts = 0;
        for bytes in sources {
            let parsed = ttf_parser::Face::parse(bytes, 0).unwrap();
            let axes: Vec<_> = parsed.variation_axes().into_iter().collect();
            variable_fonts += usize::from(!axes.is_empty());
            for phase in 0..3 {
                let variations: Vec<_> = axes
                    .iter()
                    .map(|axis| FontVariation {
                        tag: u32::from_be_bytes(axis.tag.to_bytes()),
                        value: match phase {
                            0 => axis.min_value,
                            1 => axis.def_value,
                            _ => axis.max_value,
                        },
                    })
                    .collect();
                let face = FontFace {
                    family: "",
                    native_family: "",
                    weight: 400.,
                    italic: false,
                    bytes,
                    face_index: 0,
                    variations: Vec::new(),
                };
                let mut scaler = SkScalerContext_Mac::default();
                for scale in [0.5, 1., 2.] {
                    let mut prepared = scaler.prepare_glyph_run(
                        0,
                        &face,
                        18.,
                        &variations,
                        Transform::from_scale(scale, scale),
                    );
                    // Exercise the fallback even for Roboto; do not let a
                    // valid whole-font box make the test skip per-glyph metrics.
                    prepared.coarse_bounds = None;
                    for glyph in 1..parsed.number_of_glyphs().min(9) {
                        for offset in [(0., 0.), (0.125, 0.375), (0.9375, 0.8125)] {
                            let position =
                                Transform::from_translate(40. + offset.0, 60. + offset.1);
                            let bounds = prepared
                                .native_allocation_bounds_at(glyph, position)
                                .unwrap();
                            let mut target = Pixmap::new(256, 160).unwrap();
                            target.fill(crate::raster::Color::WHITE);
                            let mut strike = SkStrike::new(0, 0);
                            let paint = PreparedGlyphPaint::new(
                                Color {
                                    red: 0.15,
                                    green: 0.35,
                                    blue: 0.65,
                                    alpha: 1.,
                                },
                                FontSmoothing::kAntialiased,
                            );
                            SkScalerContext_Mac::draw(
                                &mut target,
                                &mut strike,
                                None,
                                None,
                                &prepared,
                                glyph,
                                position,
                                &paint,
                                None,
                            );
                            let key = ImageKey {
                                glyph,
                                sub_x: (position.tx - position.tx.floor()).to_bits(),
                                sub_y: (position.ty - position.ty.floor()).to_bits(),
                            };
                            let image = strike
                                .find_image(&key)
                                .expect("original draw prepared metrics");
                            if let Some(image) = image {
                                assert_eq!(
                                    bounds,
                                    [
                                        40. + f64::from(image.left),
                                        60. + f64::from(image.top),
                                        40. + f64::from(image.left) + image.width as f64,
                                        60. + f64::from(image.top) + image.height as f64
                                    ],
                                    "scale={scale} phase={phase} glyph={glyph}"
                                );
                                for (i, pixel) in target.data().chunks_exact(4).enumerate() {
                                    if pixel == [255; 4] {
                                        continue;
                                    }
                                    let (x, y) = ((i % 256) as f64, (i / 256) as f64);
                                    assert!(
                                        x >= bounds[0]
                                            && y >= bounds[1]
                                            && x < bounds[2]
                                            && y < bounds[3]
                                    );
                                    drawn += 1;
                                }
                            } else {
                                assert_eq!(bounds, [0.; 4]);
                            }
                        }
                    }
                    assert!(prepared
                        .native_allocation_bounds_at(1, Transform::from_translate(f32::NAN, 0.))
                        .is_none());
                }
            }
        }
        assert!(variable_fonts >= 2);
        assert!(drawn > 0);
    }
}
