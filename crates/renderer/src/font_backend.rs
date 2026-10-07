use std::ffi::{c_void, CString};
use std::sync::Arc;

use font_engine::fonts::font_backend::{
    FontBackend, FontBackendFactory, FontBackendGlyphMetrics, FontBackendMetrics, FontVariation,
};

#[repr(C)]
#[derive(Clone, Copy)]
struct NativeVariation {
    tag: u32,
    value: f32,
}

#[repr(C)]
struct NativeFontMetrics {
    ascent: f32,
    descent: f32,
    leading: f32,
    x_height: f32,
    cap_height: f32,
    underline_position: f32,
    underline_thickness: f32,
    has_underline_position: bool,
    has_underline_thickness: bool,
}

#[repr(C)]
struct NativeGlyphMetrics {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    advance: f32,
}

unsafe extern "C" {
    fn LayoutngFontBackendCreate(
        bytes: *const u8,
        byte_count: usize,
        face_index: u32,
        variations: *const NativeVariation,
        variation_count: usize,
        native_family: *const i8,
        metrics_family: *const i8,
        weight: f64,
        italic: bool,
    ) -> *mut c_void;
    fn LayoutngFontBackendDestroy(backend: *mut c_void);
    fn LayoutngFontBackendWithVariations(
        backend: *mut c_void,
        variations: *const NativeVariation,
        variation_count: usize,
    ) -> *mut c_void;
    fn LayoutngFontBackendMetrics(
        backend: *mut c_void,
        size: f32,
        synthetic_bold: bool,
        synthetic_italic: bool,
    ) -> NativeFontMetrics;
    fn LayoutngFontBackendGlyphMetrics(
        backend: *mut c_void,
        glyph: u16,
        size: f32,
        synthetic_bold: bool,
        synthetic_italic: bool,
    ) -> NativeGlyphMetrics;
}

pub struct SkiaFontBackendFactory;

struct SkiaFontBackend(*mut c_void);

impl Drop for SkiaFontBackend {
    fn drop(&mut self) {
        unsafe { LayoutngFontBackendDestroy(self.0) }
    }
}

fn native_variations(axes: &[FontVariation]) -> Vec<NativeVariation> {
    axes.iter()
        .map(|axis| NativeVariation {
            tag: axis.tag,
            value: axis.value,
        })
        .collect()
}

impl FontBackendFactory for SkiaFontBackendFactory {
    fn Create(
        &self,
        bytes: &[u8],
        face_index: u32,
        variations: &[FontVariation],
        native_family: &str,
        metrics_family: &str,
        weight: f64,
        italic: bool,
    ) -> Arc<dyn FontBackend> {
        let native_family = CString::new(native_family).expect("font family contains NUL");
        let metrics_family = CString::new(metrics_family).expect("metrics family contains NUL");
        let axes = native_variations(variations);
        let handle = unsafe {
            LayoutngFontBackendCreate(
                bytes.as_ptr(),
                bytes.len(),
                face_index,
                axes.as_ptr(),
                axes.len(),
                native_family.as_ptr(),
                metrics_family.as_ptr(),
                weight,
                italic,
            )
        };
        assert!(!handle.is_null(), "Skia rejected font backend face");
        Arc::new(SkiaFontBackend(handle))
    }
}

impl FontBackend for SkiaFontBackend {
    fn Metrics(
        &self,
        size: f32,
        synthetic_bold: bool,
        synthetic_italic: bool,
    ) -> FontBackendMetrics {
        let metrics =
            unsafe { LayoutngFontBackendMetrics(self.0, size, synthetic_bold, synthetic_italic) };
        FontBackendMetrics {
            ascent: metrics.ascent,
            descent: metrics.descent,
            leading: metrics.leading,
            x_height: metrics.x_height,
            cap_height: metrics.cap_height,
            underline_position: metrics.underline_position,
            underline_thickness: metrics.underline_thickness,
            has_underline_position: metrics.has_underline_position,
            has_underline_thickness: metrics.has_underline_thickness,
        }
    }

    fn GlyphMetrics(
        &self,
        glyph: u16,
        size: f32,
        synthetic_bold: bool,
        synthetic_italic: bool,
    ) -> FontBackendGlyphMetrics {
        let metrics = unsafe {
            LayoutngFontBackendGlyphMetrics(self.0, glyph, size, synthetic_bold, synthetic_italic)
        };
        FontBackendGlyphMetrics {
            x: metrics.x,
            y: metrics.y,
            width: metrics.width,
            height: metrics.height,
            advance: metrics.advance,
        }
    }

    fn WithVariations(&self, variations: &[FontVariation]) -> Arc<dyn FontBackend> {
        let axes = native_variations(variations);
        let handle =
            unsafe { LayoutngFontBackendWithVariations(self.0, axes.as_ptr(), axes.len()) };
        assert!(!handle.is_null(), "Skia rejected font variation axes");
        Arc::new(SkiaFontBackend(handle))
    }
}
