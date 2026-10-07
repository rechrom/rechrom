// C++: font_engine/text/native/opentype_font.h/.cc. Native handles are owned
// by RAII wrappers; sized copies retain the same immutable HarfBuzz face.
use super::harfbuzz as hb;
use super::rustybuzz_shaper::shape_run_with_specified_size;
#[cfg(test)]
use crate::text_shaper::ShapedGlyph;
use crate::text_shaper::{ShapedRun, TextDirection};
use crate::SharedFontBytes;
use std::ffi::{c_char, c_void};
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub use super::harfbuzz::HbFont as HarfBuzzFont;

// cpp: font_engine/text/native/opentype_font.h:14-27
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontMetric {
    kHorizontalAscender,
    kHorizontalDescender,
    kHorizontalLineGap,
    kVerticalAscender,
    kVerticalDescender,
    kVerticalLineGap,
    kXHeight,
    kCapHeight,
    kUnderlineOffset,
    kUnderlineSize,
    kStrikeoutOffset,
    kStrikeoutSize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphExtents {
    pub x_bearing: f64,
    pub y_bearing: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpenTypeVariation {
    pub tag: u32,
    pub value: f32,
}

struct Handle<T> {
    value: NonNull<T>,
    destroy: unsafe extern "C" fn(*mut T),
}

impl<T> Handle<T> {
    fn new(value: *mut T, destroy: unsafe extern "C" fn(*mut T)) -> Self {
        Self {
            value: NonNull::new(value).expect("HarfBuzz allocation failed"),
            destroy,
        }
    }

    fn as_ptr(&self) -> *mut T {
        self.value.as_ptr()
    }
}

impl<T> Drop for Handle<T> {
    fn drop(&mut self) {
        unsafe { (self.destroy)(self.as_ptr()) }
    }
}

static NEXT_FACE_IDENTITY: AtomicU64 = AtomicU64::new(1);

// cpp: font_engine/text/native/opentype_font.cc:18-24
fn next_face_identity() -> u32 {
    let id = NEXT_FACE_IDENTITY.fetch_add(1, Ordering::Relaxed);
    u32::try_from(id).expect("OpenType face identity exhausted")
}

// cpp: font_engine/text/native/opentype_font.cc:25-39
fn set_size(font: *mut hb::HbFont, size: f64) {
    assert!(
        size.is_finite() && (0.0..=16_000_000.0).contains(&size),
        "invalid OpenType font size"
    );
    let scale = (size * 65536.0).round().clamp(0.0, i32::MAX as f64) as i32;
    unsafe {
        hb::hb_font_set_scale(font, scale, scale);
        hb::hb_font_set_ptem(font, size as f32);
    }
}

// cpp: font_engine/text/native/opentype_font.cc:40-61
fn font_name(face: *mut hb::HbFace, id: u32) -> String {
    unsafe {
        let mut length = 0u32;
        let mut language: *const c_void = ptr::null();
        let mut needed = hb::hb_ot_name_get_utf8(face, id, language, &mut length, ptr::null_mut());
        if needed == 0 {
            let mut count = 0u32;
            let names = hb::hb_ot_name_list_names(face, &mut count);
            if !names.is_null() {
                for name in std::slice::from_raw_parts(names, count as usize) {
                    if name.name_id != id {
                        continue;
                    }
                    language = name.language;
                    needed =
                        hb::hb_ot_name_get_utf8(face, id, language, &mut length, ptr::null_mut());
                    if needed != 0 {
                        break;
                    }
                }
            }
        }
        if needed == 0 {
            return String::new();
        }
        assert!(needed != u32::MAX, "font name too long");
        let mut name = vec![0u8; needed as usize + 1];
        length = name.len() as u32;
        hb::hb_ot_name_get_utf8(face, id, language, &mut length, name.as_mut_ptr().cast());
        name.truncate(length as usize);
        String::from_utf8(name).expect("HarfBuzz returned non-UTF8 font name")
    }
}

// cpp: font_engine/text/native/opentype_font.cc:63-88
#[cfg(test)]
fn direction(direction: TextDirection) -> i32 {
    match direction {
        TextDirection::kLtr => hb::DIRECTION_LTR,
        TextDirection::kRtl => hb::DIRECTION_RTL,
        TextDirection::kTtb => hb::DIRECTION_TTB,
        TextDirection::kBtt => hb::DIRECTION_BTT,
    }
}

fn metric_tag(metric: FontMetric) -> u32 {
    use FontMetric::*;
    hb::tag(match metric {
        kHorizontalAscender => *b"hasc",
        kHorizontalDescender => *b"hdsc",
        kHorizontalLineGap => *b"hlgp",
        kVerticalAscender => *b"vasc",
        kVerticalDescender => *b"vdsc",
        kVerticalLineGap => *b"vlgp",
        kXHeight => *b"xhgt",
        kCapHeight => *b"cpht",
        kUnderlineOffset => *b"undo",
        kUnderlineSize => *b"unds",
        kStrikeoutOffset => *b"stro",
        kStrikeoutSize => *b"strs",
    })
}

// cpp: font_engine/text/native/opentype_font.h:35-77
pub struct OpenTypeFont {
    source_bytes: SharedFontBytes,
    face_index: u32,
    size: f64,
    specified_size: f64,
    blob: Handle<hb::HbBlob>,
    face: Handle<hb::HbFace>,
    font: Handle<hb::HbFont>,
    variations: Vec<hb::HbVariation>,
    uses_open_type_horizontal_advances: bool,
    identity: u32,
}

impl OpenTypeFont {
    // cpp: font_engine/text/native/opentype_font.cc:100-142
    pub fn new(bytes: &[u8], face_index: u32, size: f64, variations: &[OpenTypeVariation]) -> Self {
        Self::new_shared(SharedFontBytes::from(bytes), face_index, size, variations)
    }

    /// Retain immutable font data without copying it into Rust or HarfBuzz buffers.
    pub fn new_shared(
        bytes: impl Into<SharedFontBytes>,
        face_index: u32,
        size: f64,
        variations: &[OpenTypeVariation],
    ) -> Self {
        let bytes = bytes.into();
        assert!(
            !bytes.is_empty()
                && bytes.len() <= u32::MAX as usize
                && size.is_finite()
                && (0.0..=16_000_000.0).contains(&size),
            "invalid OpenType font request"
        );
        let blob = Handle::new(
            unsafe {
                hb::hb_blob_create_or_fail(
                    bytes.as_ptr().cast::<c_char>(),
                    bytes.len() as u32,
                    hb::MEMORY_MODE_READONLY,
                    Box::into_raw(Box::new(bytes.clone())).cast(),
                    Some(release_shared_font_bytes),
                )
            },
            hb::hb_blob_destroy,
        );
        assert!(
            (face_index & 0xffff) < unsafe { hb::hb_face_count(blob.as_ptr()) },
            "invalid OpenType font or face index"
        );
        let face = Handle::new(
            unsafe { hb::hb_face_create(blob.as_ptr(), face_index) },
            hb::hb_face_destroy,
        );
        assert!(
            unsafe { hb::hb_face_get_glyph_count(face.as_ptr()) } != 0,
            "font contains no glyphs"
        );
        unsafe { hb::hb_face_make_immutable(face.as_ptr()) };
        let has_table = |tag| {
            let table = Handle::new(
                unsafe { hb::hb_face_reference_table(face.as_ptr(), tag) },
                hb::hb_blob_destroy,
            );
            unsafe { hb::hb_blob_get_length(table.as_ptr()) != 0 }
        };
        let uses_open_type_horizontal_advances =
            has_table(hb::tag(*b"trak")) && !has_table(hb::tag(*b"sbix"));
        let font = create_font(face.as_ptr(), size, variations, size);
        Self {
            source_bytes: bytes,
            face_index,
            size,
            specified_size: size,
            blob,
            face,
            font,
            variations: copy_variations(variations),
            uses_open_type_horizontal_advances,
            identity: next_face_identity(),
        }
    }

    // cpp: font_engine/text/native/opentype_font.cc:146-179
    pub fn FaceIdentity(&self) -> u32 {
        self.identity
    }

    pub fn WithSize(&self, size: f64) -> Arc<Self> {
        let variations: Vec<_> = self
            .variations
            .iter()
            .map(|v| OpenTypeVariation {
                tag: v.tag,
                value: v.value,
            })
            .collect();
        self.WithSizeAndVariations(size, &variations)
    }

    pub fn WithSizeAndVariations(&self, size: f64, variations: &[OpenTypeVariation]) -> Arc<Self> {
        let specified_size = if self.size > 0.0 {
            self.specified_size * size / self.size
        } else {
            size
        };
        self.WithSizeVariationsAndSpecifiedSize(size, variations, specified_size)
    }

    // Blink HarfBuzzFace::GetScaledFont uses the unzoomed specified CSS size
    // for AAT tracking (ptem), while the glyph scale uses the computed size.
    pub fn WithSizeVariationsAndSpecifiedSize(
        &self,
        size: f64,
        variations: &[OpenTypeVariation],
        specified_size: f64,
    ) -> Arc<Self> {
        assert!(
            specified_size.is_finite() && specified_size >= 0.0 && specified_size <= 16_000_000.0
        );

        assert!(
            size.is_finite() && (0.0..=16_000_000.0).contains(&size),
            "invalid OpenType font size"
        );
        let blob = Handle::new(
            unsafe { hb::hb_blob_reference(self.blob.as_ptr()) },
            hb::hb_blob_destroy,
        );
        let face = Handle::new(
            unsafe { hb::hb_face_reference(self.face.as_ptr()) },
            hb::hb_face_destroy,
        );
        let font = create_font(face.as_ptr(), size, variations, specified_size);
        Arc::new(Self {
            source_bytes: self.source_bytes.clone(),
            face_index: self.face_index,
            size,
            specified_size,
            blob,
            face,
            font,
            variations: copy_variations(variations),
            uses_open_type_horizontal_advances: self.uses_open_type_horizontal_advances,
            identity: self.identity,
        })
    }

    // cpp: font_engine/text/native/opentype_font.cc:180-204
    pub fn HasVariationAxis(&self, tag: u32) -> bool {
        if tag == 0 {
            return false;
        }
        let mut info = hb::HbVarAxisInfo::default();
        unsafe { hb::hb_ot_var_find_axis_info(self.face.as_ptr(), tag, &mut info) != 0 }
    }

    pub fn UsesOpenTypeHorizontalAdvances(&self) -> bool {
        self.uses_open_type_horizontal_advances
    }

    /// The returned handle is borrowed and cannot outlive this OpenTypeFont.
    pub fn BorrowHarfBuzzFont(&self) -> *mut HarfBuzzFont {
        self.font.as_ptr()
    }

    pub fn SourceBytes(&self) -> &[u8] {
        &self.source_bytes
    }

    pub fn FaceIndex(&self) -> u32 {
        self.face_index
    }

    pub fn SpecifiedSize(&self) -> f64 {
        self.specified_size
    }

    pub fn Size(&self) -> f64 {
        self.size
    }

    pub fn Variations(&self) -> Vec<OpenTypeVariation> {
        self.variations
            .iter()
            .map(|axis| OpenTypeVariation {
                tag: axis.tag,
                value: axis.value,
            })
            .collect()
    }

    pub fn FamilyName(&self) -> String {
        let family = font_name(self.face.as_ptr(), hb::NAME_ID_TYPOGRAPHIC_FAMILY);
        if family.is_empty() {
            font_name(self.face.as_ptr(), hb::NAME_ID_FONT_FAMILY)
        } else {
            family
        }
    }

    pub fn PostScriptName(&self) -> String {
        font_name(self.face.as_ptr(), hb::NAME_ID_POSTSCRIPT_NAME)
    }
    pub fn UnitsPerEm(&self) -> u32 {
        unsafe { hb::hb_face_get_upem(self.face.as_ptr()) }
    }
    pub fn GlyphCount(&self) -> u32 {
        unsafe { hb::hb_face_get_glyph_count(self.face.as_ptr()) }
    }

    // cpp: font_engine/text/native/opentype_font.cc:205-239
    pub fn GlyphForCharacter(&self, character: u32) -> Option<u32> {
        assert!(
            character <= 0x10ffff && !(0xd800..=0xdfff).contains(&character),
            "invalid Unicode scalar"
        );
        let mut glyph = 0;
        (unsafe { hb::hb_font_get_nominal_glyph(self.font.as_ptr(), character, &mut glyph) } != 0)
            .then_some(glyph)
    }

    fn CheckGlyph(&self, glyph: u32) {
        assert!(glyph < self.GlyphCount(), "glyph index outside font");
    }

    pub fn HorizontalAdvance(&self, glyph: u32) -> f64 {
        self.CheckGlyph(glyph);
        unsafe { hb::hb_font_get_glyph_h_advance(self.font.as_ptr(), glyph) as f64 / 65536.0 }
    }

    pub fn VerticalAdvance(&self, glyph: u32) -> f64 {
        self.CheckGlyph(glyph);
        unsafe { hb::hb_font_get_glyph_v_advance(self.font.as_ptr(), glyph) as f64 / 65536.0 }
    }

    pub fn Extents(&self, glyph: u32) -> Option<GlyphExtents> {
        self.CheckGlyph(glyph);
        let mut extents = hb::HbGlyphExtents::default();
        if unsafe { hb::hb_font_get_glyph_extents(self.font.as_ptr(), glyph, &mut extents) } == 0 {
            return None;
        }
        Some(GlyphExtents {
            x_bearing: extents.x_bearing as f64 / 65536.0,
            y_bearing: extents.y_bearing as f64 / 65536.0,
            width: extents.width as f64 / 65536.0,
            height: extents.height as f64 / 65536.0,
        })
    }

    pub fn Metric(&self, metric: FontMetric) -> Option<f64> {
        let mut position = 0;
        (unsafe {
            hb::hb_ot_metrics_get_position(self.font.as_ptr(), metric_tag(metric), &mut position)
        } != 0)
            .then_some(position as f64 / 65536.0)
    }

    // cpp: font_engine/text/native/opentype_font.cc:240-248
    pub fn CopyTable(&self, tag: u32) -> Vec<u8> {
        let table = Handle::new(
            unsafe { hb::hb_face_reference_table(self.face.as_ptr(), tag) },
            hb::hb_blob_destroy,
        );
        let mut length = 0u32;
        let bytes = unsafe { hb::hb_blob_get_data(table.as_ptr(), &mut length) };
        if length == 0 {
            return Vec::new();
        }
        assert!(!bytes.is_null(), "HarfBuzz returned null table data");
        unsafe { std::slice::from_raw_parts(bytes.cast::<u8>(), length as usize) }.to_vec()
    }

    // cpp: font_engine/text/native/opentype_font.cc:249-283
    pub fn ShapeRun(
        &self,
        utf8: &str,
        direction_value: TextDirection,
        language: &str,
        script: &str,
    ) -> ShapedRun {
        shape_run_with_specified_size(
            self.SourceBytes(),
            self.FaceIndex(),
            self.Size(),
            self.specified_size,
            &self.Variations(),
            utf8,
            direction_value,
            language,
            script,
        )
    }

    #[cfg(test)]
    pub fn ShapeRunNativeForTesting(
        &self,
        utf8: &str,
        direction_value: TextDirection,
        language: &str,
        script: &str,
    ) -> ShapedRun {
        assert!(
            utf8.len() <= i32::MAX as usize
                && language.len() <= i32::MAX as usize
                && script.len() <= i32::MAX as usize,
            "text shaping request too large"
        );
        let native_direction = direction(direction_value);
        let buffer = Handle::new(unsafe { hb::hb_buffer_create() }, hb::hb_buffer_destroy);
        unsafe {
            hb::hb_buffer_set_direction(buffer.as_ptr(), native_direction);
            hb::hb_buffer_set_language(
                buffer.as_ptr(),
                hb::hb_language_from_string(language.as_ptr().cast(), language.len() as i32),
            );
            if !script.is_empty() {
                hb::hb_buffer_set_script(
                    buffer.as_ptr(),
                    hb::hb_script_from_string(script.as_ptr().cast(), script.len() as i32),
                );
            }
            hb::hb_buffer_add_utf8(
                buffer.as_ptr(),
                utf8.as_ptr().cast(),
                utf8.len() as i32,
                0,
                utf8.len() as i32,
            );
            hb::hb_buffer_guess_segment_properties(buffer.as_ptr());
            assert!(
                hb::hb_buffer_allocation_successful(buffer.as_ptr()) != 0
                    && hb::hb_shape_full(
                        self.font.as_ptr(),
                        buffer.as_ptr(),
                        ptr::null(),
                        0,
                        ptr::null()
                    ) != 0,
                "HarfBuzz shaping failed"
            );
            let mut count = 0u32;
            let info = hb::hb_buffer_get_glyph_infos(buffer.as_ptr(), &mut count);
            let positions = hb::hb_buffer_get_glyph_positions(buffer.as_ptr(), ptr::null_mut());
            let mut result = ShapedRun {
                glyphs: Vec::with_capacity(count as usize),
                advance: 0.0,
            };
            if count == 0 {
                return result;
            }
            assert!(
                !info.is_null() && !positions.is_null(),
                "HarfBuzz returned null glyph data"
            );
            let infos = std::slice::from_raw_parts(info, count as usize);
            let positions = std::slice::from_raw_parts(positions, count as usize);
            for (info, position) in infos.iter().zip(positions) {
                result.glyphs.push(ShapedGlyph {
                    id: info.codepoint,
                    cluster: info.cluster,
                    x_advance: position.x_advance as f64 / 65536.0,
                    y_advance: position.y_advance as f64 / 65536.0,
                    x_offset: position.x_offset as f64 / 65536.0,
                    y_offset: position.y_offset as f64 / 65536.0,
                });
                result.advance += if native_direction == hb::DIRECTION_LTR
                    || native_direction == hb::DIRECTION_RTL
                {
                    position.x_advance as f64 / 65536.0
                } else {
                    position.y_advance as f64 / 65536.0
                };
            }
            result
        }
    }
}

// HarfBuzz calls this on allocation failure or when the last blob/table/face
// reference is destroyed. The backing allocation outlives every native handle.
unsafe extern "C" fn release_shared_font_bytes(data: *mut c_void) {
    drop(Box::from_raw(data.cast::<SharedFontBytes>()));
}

fn copy_variations(variations: &[OpenTypeVariation]) -> Vec<hb::HbVariation> {
    variations
        .iter()
        .map(|variation| {
            assert!(
                variation.tag != 0 && variation.value.is_finite(),
                "invalid OpenType variation"
            );
            hb::HbVariation {
                tag: variation.tag,
                value: variation.value,
            }
        })
        .collect()
}

fn create_font(
    face: *mut hb::HbFace,
    size: f64,
    variations: &[OpenTypeVariation],
    specified_size: f64,
) -> Handle<hb::HbFont> {
    let font = Handle::new(unsafe { hb::hb_font_create(face) }, hb::hb_font_destroy);
    assert!(
        font.as_ptr() != unsafe { hb::hb_font_get_empty() },
        "HarfBuzz font allocation failed"
    );
    unsafe { hb::hb_ot_font_set_funcs(font.as_ptr()) };
    let variations = copy_variations(variations);
    unsafe {
        hb::hb_font_set_variations(
            font.as_ptr(),
            if variations.is_empty() {
                ptr::null()
            } else {
                variations.as_ptr()
            },
            variations.len() as u32,
        )
    };
    set_size(font.as_ptr(), size);
    unsafe {
        hb::hb_font_set_ptem(font.as_ptr(), specified_size as f32);
    }
    unsafe { hb::hb_font_make_immutable(font.as_ptr()) };
    font
}

#[cfg(test)]
mod shared_bytes_tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn native_blob_retains_mapping_until_last_native_reference() {
        use std::{
            io::Write,
            sync::atomic::{AtomicUsize, Ordering},
        };
        let reference = include_bytes!(
            "../../../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
        );
        let path =
            std::env::temp_dir().join(format!("harfbuzz-font-map-{}.ttf", std::process::id()));
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        std::fs::remove_file(path).unwrap();
        file.write_all(reference).unwrap();
        // SAFETY: This unlinked file has no other handles, and its sole writable
        // handle is closed before any font consumer receives the mapping.
        let mut bytes = unsafe { SharedFontBytes::map_readonly(&file) }.unwrap();
        drop(file);
        let observer = Arc::new(AtomicUsize::new(0));
        bytes.observe_mapping_drop(observer.clone());
        let original_pointer = bytes.as_ptr();
        let font = OpenTypeFont::new_shared(bytes, 0, 16.0, &[]);
        let sized = font.WithSize(24.0);
        assert_eq!(font.SourceBytes().as_ptr(), original_pointer);
        assert_eq!(sized.SourceBytes().as_ptr(), original_pointer);
        let retained_blob = Handle::new(
            unsafe { hb::hb_blob_reference(font.blob.as_ptr()) },
            hb::hb_blob_destroy,
        );
        drop(font);
        let glyph = sized.GlyphForCharacter('A' as u32).unwrap();
        assert!(sized.HorizontalAdvance(glyph) > 0.0);
        drop(sized);
        assert_eq!(observer.load(Ordering::SeqCst), 0);
        let mut length = 0;
        let pointer = unsafe { hb::hb_blob_get_data(retained_blob.as_ptr(), &mut length) };
        assert_eq!(pointer.cast::<u8>(), original_pointer);
        let retained = unsafe { std::slice::from_raw_parts(pointer.cast::<u8>(), length as usize) };
        assert_eq!(retained, reference);
        drop(retained_blob);
        assert_eq!(observer.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn native_blob_retains_shared_backing_after_font_owner_is_dropped() {
        let bytes: Arc<[u8]> = include_bytes!(
            "../../../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
        )
        .as_slice()
        .into();
        let weak = Arc::downgrade(&bytes);
        let original_pointer = bytes.as_ptr();
        let original_length = bytes.len();
        let font = OpenTypeFont::new_shared(bytes, 0, 16.0, &[]);
        let sized = font.WithSize(24.0);
        assert_eq!(font.SourceBytes().as_ptr(), original_pointer);
        assert_eq!(sized.SourceBytes().as_ptr(), original_pointer);
        let retained_blob = Handle::new(
            unsafe { hb::hb_blob_reference(font.blob.as_ptr()) },
            hb::hb_blob_destroy,
        );
        drop(font);
        let glyph = sized.GlyphForCharacter('A' as u32).unwrap();
        assert!(sized.HorizontalAdvance(glyph) > 0.0);
        drop(sized);
        assert!(weak.upgrade().is_some());
        let mut length = 0;
        let native_pointer = unsafe { hb::hb_blob_get_data(retained_blob.as_ptr(), &mut length) };
        assert_eq!(native_pointer.cast::<u8>(), original_pointer);
        assert_eq!(length as usize, original_length);
        let retained =
            unsafe { std::slice::from_raw_parts(native_pointer.cast::<u8>(), length as usize) };
        assert_eq!(
            retained,
            include_bytes!(
                "../../../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
            )
        );
        drop(retained_blob);
        assert!(weak.upgrade().is_none());
    }
}
