#![cfg(target_vendor = "apple")]
use crate::src::utils::mac::SkUniqueCFRef::SkUniqueCFRef;
use core::ffi::{c_char, c_void};
// Native SDK ABI bindings. Each opaque CF type retains its actual type identity;
// these are imported objects, never Rust-constructed backend stand-ins.
macro_rules! cf_opaque {($($name:ident),*)=>{$(#[repr(C)]pub struct $name{_opaque:[u8;0]})*};}
cf_opaque!(
    CTFont,
    CTFontDescriptor,
    CFArray,
    CFBoolean,
    CFDictionary,
    CFNumber,
    CFString,
    CGFont,
    CGPath,
    CGContext,
    CGColor,
    CGColorSpace
);
pub type CTFontRef = *mut CTFont;
pub type CTFontDescriptorRef = *mut CTFontDescriptor;
pub type CFArrayRef = *mut CFArray;
pub type CFDictionaryRef = *mut CFDictionary;
pub type CFNumberRef = *mut CFNumber;
pub type CFStringRef = *mut CFString;
pub type CGFloat = f64;
pub type CFIndex = isize;
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    pub(crate) fn CFGetTypeID(value: *const c_void) -> usize;
    fn CFArrayGetTypeID() -> usize;
    fn CFBooleanGetTypeID() -> usize;
    fn CFDictionaryGetTypeID() -> usize;
    fn CFNumberGetTypeID() -> usize;
    fn CFCopyTypeIDDescription(id: usize) -> CFStringRef;
    fn CFStringGetLength(value: CFStringRef) -> CFIndex;
    fn CFStringGetMaximumSizeForEncoding(length: CFIndex, encoding: u32) -> CFIndex;
    fn CFStringGetCString(
        value: CFStringRef,
        buffer: *mut c_char,
        capacity: CFIndex,
        encoding: u32,
    ) -> u8;
    fn CFNumberGetValue(value: CFNumberRef, kind: isize, out: *mut c_void) -> u8;
    pub(crate) fn CFArrayGetCount(value: CFArrayRef) -> CFIndex;
}
// Source: src/ports/SkTypeface_mac_ct.h:48-51
#[derive(Clone, Copy, Default)]
pub struct OpszVariation {
    pub isSet: bool,
    pub value: f64,
}
// Source: src/ports/SkTypeface_mac_ct.h:53-57
pub struct CTFontVariation {
    pub variation: core::mem::ManuallyDrop<SkUniqueCFRef<CFDictionary>>,
    pub wrongOpszVariation: core::mem::ManuallyDrop<SkUniqueCFRef<CFDictionary>>,
    pub opsz: OpszVariation,
}
// Source: src/ports/SkTypeface_mac_ct.h:53-57
// Implicit native member destruction runs in reverse declaration order.
impl Drop for CTFontVariation {
    fn drop(&mut self) {
        unsafe {
            core::mem::ManuallyDrop::drop(&mut self.wrongOpszVariation);
            core::mem::ManuallyDrop::drop(&mut self.variation);
        }
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:71-80
/// Safety: src is a live CFString for the complete conversion.
pub unsafe fn SkStringFromCFString(src: CFStringRef, dst: &mut String) {
    const UTF8: u32 = 0x08000100;
    let length = unsafe { CFStringGetMaximumSizeForEncoding(CFStringGetLength(src), UTF8) } + 1;
    let mut storage = vec![0u8; length as usize];
    unsafe {
        CFStringGetCString(src, storage.as_mut_ptr().cast(), length, UTF8);
    }
    let length = unsafe { core::ffi::CStr::from_ptr(storage.as_ptr().cast()) }
        .to_bytes()
        .len();
    storage.truncate(length);
    *dst = String::from_utf8(storage).expect("CoreFoundation UTF-8 conversion");
}
// Source: src/ports/SkTypeface_mac_ct.cpp:82-87
pub unsafe fn SkCFTypeIDDescription(id: usize) -> String {
    let description = unsafe { SkUniqueCFRef::from_raw(CFCopyTypeIDDescription(id)) };
    let mut result = String::new();
    unsafe {
        SkStringFromCFString(description.get(), &mut result);
    }
    result
}
// Source: src/ports/SkTypeface_mac_ct.cpp:89-95
pub trait CFConcreteType {
    unsafe fn type_id() -> usize;
}
macro_rules! cf_type_id {
    ($ty:ty,$func:ident) => {
        impl CFConcreteType for $ty {
            unsafe fn type_id() -> usize {
                unsafe { $func() }
            }
        }
    };
}
cf_type_id!(CFArray, CFArrayGetTypeID);
cf_type_id!(CFBoolean, CFBooleanGetTypeID);
cf_type_id!(CFDictionary, CFDictionaryGetTypeID);
cf_type_id!(CFNumber, CFNumberGetTypeID);
// Source: src/ports/SkTypeface_mac_ct.cpp:105-125
pub unsafe fn SkCFDynamicCastNamed<T: CFConcreteType>(
    value: *const c_void,
    out: &mut *mut T,
    name: Option<&core::ffi::CStr>,
) -> bool {
    if value.is_null() {
        if let Some(name) = name {
            #[cfg(sk_debug)]
            unsafe {
                crate::SkDebugf!(@source b"src/ports/SkTypeface_mac_ct.cpp\0".as_ptr(),111; b"%s not present\n\0".as_ptr(),name.as_ptr());
            }
        }
        return false;
    }
    if unsafe { CFGetTypeID(value) } != unsafe { T::type_id() } {
        if let Some(name) = name {
            #[cfg(sk_debug)]
            {
                let actual = unsafe { SkCFTypeIDDescription(CFGetTypeID(value)) };
                let expected = unsafe { SkCFTypeIDDescription(T::type_id()) };
                let actual = crate::src::core::SkString::SkString::from_view(actual.as_bytes());
                let expected = crate::src::core::SkString::SkString::from_view(expected.as_bytes());
                unsafe {
                    crate::SkDebugf!(@source b"src/ports/SkTypeface_mac_ct.cpp\0".as_ptr(),117; b"%s is a %s but expected a %s\n\0".as_ptr(),name.as_ptr(),actual.c_str(),expected.c_str());
                }
            }
        }
        return false;
    }
    *out = value.cast_mut().cast();
    true
}
// Existing no-name callers retain their source interface; no new source credit.
pub unsafe fn SkCFDynamicCast<T: CFConcreteType>(value: *const c_void, out: &mut *mut T) -> bool {
    unsafe { SkCFDynamicCastNamed(value, out, None) }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:127-136
pub trait CFNumberValue {
    const CF_NUMBER_TYPE: isize;
}
macro_rules! cf_number_type {
    ($ty:ty,$value:expr) => {
        impl CFNumberValue for $ty {
            const CF_NUMBER_TYPE: isize = $value;
        }
    };
}
// SDK CFNumber.h native char/short/int/long/longlong/float/double tags7..13.
cf_number_type!(i8, 7);
cf_number_type!(i16, 8);
cf_number_type!(i32, 9);
cf_number_type!(isize, 10);
cf_number_type!(i64, 11);
cf_number_type!(f32, 12);
cf_number_type!(f64, 13);
// Source: src/ports/SkTypeface_mac_ct.cpp:138-154
pub unsafe fn SkCFNumberDynamicCastNamed<T: CFNumberValue>(
    value: *const c_void,
    out: &mut T,
    cf_number: Option<&mut CFNumberRef>,
    name: Option<&core::ffi::CStr>,
) -> bool {
    let mut number = core::ptr::null_mut();
    if !unsafe { SkCFDynamicCastNamed::<CFNumber>(value, &mut number, name) } {
        return false;
    }
    if unsafe { CFNumberGetValue(number, T::CF_NUMBER_TYPE, (out as *mut T).cast()) } == 0 {
        if let Some(name) = name {
            #[cfg(sk_debug)]
            unsafe {
                crate::SkDebugf!(@source b"src/ports/SkTypeface_mac_ct.cpp\0".as_ptr(),146; b"%s CFNumber not extractable\n\0".as_ptr(),name.as_ptr());
            }
        }
        return false;
    }
    if let Some(result) = cf_number {
        *result = number;
    }
    true
}
// No-name caller overload is Rust language glue only; no source credit.
pub unsafe fn SkCFNumberDynamicCast<T: CFNumberValue>(
    value: *const c_void,
    out: &mut T,
    cf_number: Option<&mut CFNumberRef>,
) -> bool {
    unsafe { SkCFNumberDynamicCastNamed(value, out, cf_number, None) }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:156-158
pub fn SkTypeface_GetCTFontRef(
    face: Option<&crate::src::core::SkTypeface::SkTypeface>,
) -> CTFontRef {
    face.map_or(core::ptr::null_mut(), |face| {
        face.internal_private_getCTFontRef().cast()
    })
}
// Narrow native resource ownership projections. CTFont is an immutable CoreText
// font, with thread-safe reference management/queries. Axes are exclusively the
// immutable CFArray/property-list graph returned by that retained CTFont's
// descriptor/variation query; this private wrapper cannot adopt arbitrary arrays.
// Neither wrapper adds a reference counter or mutable access. Native CGContexts
// remain in the separately locked scaler, never in this shared font resource.
#[repr(transparent)]
pub struct MacFontOwner(SkUniqueCFRef<CTFont>);
impl MacFontOwner {
    pub fn get(&self) -> CTFontRef {
        self.0.get()
    }
}
unsafe impl Send for MacFontOwner {}
unsafe impl Sync for MacFontOwner {}
#[repr(transparent)]
struct MacAxesOwner(SkUniqueCFRef<CFArray>);
impl MacAxesOwner {
    fn get(&self) -> CFArrayRef {
        self.0.get()
    }
}
unsafe impl Send for MacAxesOwner {}
unsafe impl Sync for MacAxesOwner {}
// Source: src/ports/SkTypeface_mac_ct.h:90-92
// Source: src/ports/SkTypeface_mac_ct.h:132-138
// Native shared mutex + once are expressed as actual synchronized Rust owners.
// Immutable CF font/axis objects are never cloned into a second font producer.
pub struct SkTypeface_Mac {
    fVariationAxes: std::sync::OnceLock<MacAxesOwner>,
    fStream: std::sync::Mutex<Option<crate::src::core::SkStream::TransferableStreamAsset>>,
    pub fFontRef: MacFontOwner,
    pub fOpszVariation: OpszVariation,
    pub fHasColorGlyphs: bool,
    fIsFromStream: bool,
}
#[link(name = "CoreText", kind = "framework")]
unsafe extern "C" {
    fn CTFontGetSymbolicTraits(font: CTFontRef) -> u32;
    fn CTFontCopyFontDescriptor(font: CTFontRef) -> CTFontDescriptorRef;
    fn CTFontDescriptorCopyAttribute(desc: CTFontDescriptorRef, name: CFStringRef) -> *mut c_void;
    fn CTFontCopyVariationAxes(font: CTFontRef) -> CFArrayRef;
    fn CTFontCopyVariation(font: CTFontRef) -> CFDictionaryRef;
    fn CTFontGetUnitsPerEm(font: CTFontRef) -> u32;
    fn CTFontCopyLocalizedName(
        font: CTFontRef,
        key: CFStringRef,
        language: *mut CFStringRef,
    ) -> CFStringRef;
    static kCTFontFamilyNameKey: CFStringRef;
    fn CTFontCopyAvailableTables(font: CTFontRef, options: u32) -> CFArrayRef;
    fn CTFontCopyTable(font: CTFontRef, tag: u32, options: u32) -> *mut CFData;
    fn CTFontCopyGraphicsFont(font: CTFontRef, attributes: *mut CTFontDescriptorRef)
        -> *mut CGFont;
    static kCTFontVariationAxesAttribute: CFStringRef;
    static kCTFontVariationAxisIdentifierKey: CFStringRef;
    static kCTFontVariationAxisDefaultValueKey: CFStringRef;
}
#[repr(C)]
pub struct CFData {
    _opaque: [u8; 0],
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFArrayGetValueAtIndex(value: CFArrayRef, index: CFIndex) -> *const c_void;
    fn CFDictionaryGetValue(value: CFDictionaryRef, key: *const c_void) -> *const c_void;
    fn CFDataGetLength(value: *mut CFData) -> CFIndex;
    fn CFDataGetBytePtr(value: *mut CFData) -> *const u8;
    fn CFRelease(value: *const c_void);
}
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGFontCopyTableForTag(font: *mut CGFont, tag: u32) -> *mut CFData;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFDictionaryGetValueIfPresent(
        dict: CFDictionaryRef,
        key: *const c_void,
        value: *mut *const c_void,
    ) -> u8;
    fn CFNumberIsFloatType(value: CFNumberRef) -> u8;
}
#[link(name = "CoreText", kind = "framework")]
unsafe extern "C" {
    static kCTFontTraitsAttribute: CFStringRef;
    static kCTFontWeightTrait: CFStringRef;
    static kCTFontWidthTrait: CFStringRef;
    static kCTFontSlantTrait: CFStringRef;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFEqual(a: *const c_void, b: *const c_void) -> u8;
    fn CFRetain(value: *const c_void) -> *const c_void;
}
unsafe extern "C" {
    fn atexit(callback: extern "C" fn()) -> i32;
}
// Source: src/ports/SkTypeface_mac_ct.cpp:160-165
fn find_by_CTFontRef(cached: &crate::src::core::SkTypeface::SkTypeface, font: CTFontRef) -> bool {
    unsafe { CFEqual(font.cast(), cached.internal_private_getCTFontRef().cast()) != 0 }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:167-203
static MAC_TYPEFACE_CACHE: std::sync::OnceLock<
    std::sync::Mutex<Option<crate::src::core::SkTypefaceCache::SkTypefaceCache>>,
> = std::sync::OnceLock::new();
extern "C" fn release_mac_typeface_cache() {
    if let Some(cache) = MAC_TYPEFACE_CACHE.get() {
        let owned = cache.lock().unwrap_or_else(|e| e.into_inner()).take();
        drop(owned);
    }
}
impl SkTypeface_Mac {
    /// Safety: font is an actual immutable CTFont +1 reference. Any provided stream
    /// carries its explicit native complete-graph transferability proof; canonical
    /// font/backing resources remain valid for every owner and cached derived stream.
    pub unsafe fn Make(
        font: SkUniqueCFRef<CTFont>,
        opsz: OpszVariation,
        provided: Option<crate::src::core::SkStream::TransferableStreamAsset>,
    ) -> Option<crate::src::core::SkTypeface::SkTypefaceRef> {
        let cache = MAC_TYPEFACE_CACHE.get_or_init(|| {
            let cache = std::sync::Mutex::new(Some(
                crate::src::core::SkTypefaceCache::SkTypefaceCache::default(),
            ));
            unsafe {
                atexit(release_mac_typeface_cache);
            }
            cache
        });
        crate::SkASSERT!(@source b"src/ports/SkTypeface_mac_ct.cpp\0".as_ptr(),174; @condition b"font\0".as_ptr(); !font.is_null());
        let from_stream = provided.is_some();
        let raw = font.get();
        let create = move || unsafe {
            let desc = SkUniqueCFRef::from_raw(CTFontCopyFontDescriptor(font.get()));
            let style = SkCTFontDescriptorGetSkFontStyle(desc.get(), from_stream);
            let fixed = CTFontGetSymbolicTraits(font.get()) & (1 << 10) != 0;
            let backend = Self::new_backend(font, opsz, provided);
            crate::include::core::SkRefCnt::sk_sp::new(
                crate::src::core::SkTypeface::SkTypeface::new(style, fixed, Box::new(backend)),
            )
        };
        if from_stream {
            return Some(create());
        }
        let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
        let cache = guard
            .as_mut()
            .expect("native cache lives until static destruction");
        if let Some(face) = cache.findByProcAndRef(|face| find_by_CTFontRef(face, raw)) {
            return Some(face);
        }
        let face = create();
        cache.add(face.clone());
        Some(face)
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:205-210
pub unsafe fn SkMakeTypefaceFromCTFont(
    font: CTFontRef,
) -> Option<crate::src::core::SkTypeface::SkTypefaceRef> {
    unsafe {
        CFRetain(font.cast());
        SkTypeface_Mac::Make(
            SkUniqueCFRef::from_raw(font),
            OpszVariation::default(),
            None,
        )
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:212-217
unsafe fn find_dict_CGFloat(dict: CFDictionaryRef, name: CFStringRef, out: &mut f64) -> bool {
    let mut number = core::ptr::null();
    unsafe {
        CFDictionaryGetValueIfPresent(dict, name.cast(), &mut number) != 0
            && CFNumberIsFloatType(number.cast_mut().cast()) != 0
            && CFNumberGetValue(number.cast_mut().cast(), 16, (out as *mut f64).cast()) != 0
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:219-260
// Hoisted template specializations retain native expression order and rounding.
fn interpolate_int_to_float(value: i32, mapping: &[(i32, f64)]) -> f64 {
    if value < mapping[0].0 {
        return mapping[0].1;
    }
    for i in 0..mapping.len() - 1 {
        if value < mapping[i + 1].0 {
            let (lo, hi) = (mapping[i], mapping[i + 1]);
            debug_assert!(lo.0 < hi.0);
            debug_assert!(lo.1 <= hi.1);
            return lo.1 + (((value - lo.0) as f64 * (hi.1 - lo.1)) / (hi.0 - lo.0) as f64);
        }
    }
    mapping[mapping.len() - 1].1
}
fn interpolate_float_to_int(value: f64, mapping: &[(f64, i32)]) -> i32 {
    if value < mapping[0].0 {
        return mapping[0].1;
    }
    for i in 0..mapping.len() - 1 {
        if value < mapping[i + 1].0 {
            let (lo, hi) = (mapping[i], mapping[i + 1]);
            debug_assert!(lo.0 < hi.0);
            debug_assert!(lo.1 <= hi.1);
            return (lo.1 as f64 + (((value - lo.0) * (hi.1 - lo.1) as f64) / (hi.0 - lo.0)) + 0.5)
                as i32;
        }
    }
    mapping[mapping.len() - 1].1
}
// Source: src/ports/SkTypeface_mac_ct.cpp:268-286
pub fn SkCTFontCTWeightForCSSWeight(weight: i32) -> f64 {
    static MAPPING: std::sync::OnceLock<[(i32, f64); 11]> = std::sync::OnceLock::new();
    let mapping = MAPPING.get_or_init(|| {
        let weights = crate::src::utils::mac::SkCTFont::SkCTFontGetNSFontWeightMapping();
        core::array::from_fn(|i| (i as i32 * 100, weights[i]))
    });
    interpolate_int_to_float(weight, mapping)
}
// Source: src/ports/SkTypeface_mac_ct.cpp:294-321
fn ct_weight_to_fontstyle(weight: f64, from_data: bool) -> i32 {
    static MAPPING: std::sync::OnceLock<([(f64, i32); 11], [(f64, i32); 11])> =
        std::sync::OnceLock::new();
    let mapping = MAPPING.get_or_init(|| {
        let native = crate::src::utils::mac::SkCTFont::SkCTFontGetNSFontWeightMapping();
        let data = crate::src::utils::mac::SkCTFont::SkCTFontGetDataFontWeightMapping();
        (
            core::array::from_fn(|i| (native[i], i as i32 * 100)),
            core::array::from_fn(|i| (data[i], i as i32 * 100)),
        )
    });
    interpolate_float_to_int(weight, if from_data { &mapping.1 } else { &mapping.0 })
}
// Source: src/ports/SkTypeface_mac_ct.cpp:324-335
pub fn SkCTFontCTWidthForCSSWidth(width: i32) -> f64 {
    interpolate_int_to_float(width, &[(0, -0.5), (10, 0.5)])
}
// Source: src/ports/SkTypeface_mac_ct.cpp:338-349
fn ct_width_to_fontstyle(width: f64) -> i32 {
    interpolate_float_to_int(width, &[(-0.5, 0), (0.5, 10)])
}
// Source: src/ports/SkTypeface_mac_ct.cpp:351-374
pub unsafe fn SkCTFontDescriptorGetSkFontStyle(
    desc: CTFontDescriptorRef,
    from_data: bool,
) -> crate::include::core::SkFontStyle::SkFontStyle {
    use crate::include::core::SkFontStyle::{SkFontStyle, Slant};
    let traits = unsafe {
        SkUniqueCFRef::from_raw(CTFontDescriptorCopyAttribute(desc, kCTFontTraitsAttribute))
    };
    let mut dict = core::ptr::null_mut();
    if !unsafe { SkCFDynamicCast::<CFDictionary>(traits.get(), &mut dict) } {
        return SkFontStyle::default();
    }
    let (mut weight, mut width, mut slant) = (0.0, 0.0, 0.0);
    if !unsafe { find_dict_CGFloat(dict, kCTFontWeightTrait, &mut weight) } {
        weight = 0.0;
    }
    if !unsafe { find_dict_CGFloat(dict, kCTFontWidthTrait, &mut width) } {
        width = 0.0;
    }
    if !unsafe { find_dict_CGFloat(dict, kCTFontSlantTrait, &mut slant) } {
        slant = 0.0;
    }
    SkFontStyle::new(
        ct_weight_to_fontstyle(weight, from_data),
        ct_width_to_fontstyle(width),
        if slant != 0.0 {
            Slant::kItalic_Slant
        } else {
            Slant::kUpright_Slant
        },
    )
}
#[repr(C)]
pub struct CFCharacterSet {
    _opaque: [u8; 0],
}
#[link(name = "CoreText", kind = "framework")]
unsafe extern "C" {
    fn CTFontCopyCharacterSet(font: CTFontRef) -> *mut CFCharacterSet;
    fn CTFontCopyAttribute(font: CTFontRef, key: CFStringRef) -> *mut c_void;
    static kCTFontFormatAttribute: CFStringRef;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFCharacterSetCreateBitmapRepresentation(
        allocator: *const c_void,
        set: *mut CFCharacterSet,
    ) -> *mut CFData;
}
// Source: src/ports/SkTypeface_mac_ct.cpp:379-396
unsafe fn populate_glyph_to_unicode_slow(font: CTFontRef, out: &mut [i32]) {
    out.fill(0);
    let mut remaining = out.len();
    let mut character = 0u16;
    while remaining > 0 {
        let mut glyph = core::mem::MaybeUninit::<u16>::uninit();
        if unsafe { CTFontGetGlyphsForCharacters(font, &character, glyph.as_mut_ptr(), 1) } {
            let glyph = unsafe { glyph.assume_init() } as usize;
            if glyph < out.len() && out[glyph] == 0 {
                out[glyph] = character as i32;
                remaining -= 1;
            }
        }
        character = character.wrapping_add(1);
        if character == 0 {
            break;
        }
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:398-398
const kPlaneSize: usize = 1 << 13;
// Source: src/ports/SkTypeface_mac_ct.cpp:400-434
unsafe fn get_plane_glyph_map(bits: *const u8, font: CTFontRef, out: &mut [i32], plane: u8) {
    let origin = (plane as i32) << 16;
    for i in 0..kPlaneSize {
        let mask = unsafe { bits.add(i).read() };
        if mask == 0 {
            continue;
        }
        for j in 0..8 {
            if mask & (1u8 << j) == 0 {
                continue;
            }
            let offset = ((i as u16) << 3) | j as u16;
            let codepoint = origin | offset as i32;
            let mut utf16 = [offset, 0u16];
            let count = if origin != 0 {
                unsafe { crate::src::core::SkUTF::ToUTF16(codepoint, utf16.as_mut_ptr()) }
            } else {
                1
            };
            let mut glyphs = [0u16; 2];
            if unsafe {
                CTFontGetGlyphsForCharacters(
                    font,
                    utf16.as_ptr(),
                    glyphs.as_mut_ptr(),
                    count as isize,
                )
            } {
                crate::SkASSERT!(@source b"src/ports/SkTypeface_mac_ct.cpp\0".as_ptr(),424; @condition b"glyphs[1] == 0\0".as_ptr(); (glyphs[1])==(0));
                let glyph = glyphs[0] as usize;
                if glyph < out.len() && out[glyph] < 0x20 {
                    out[glyph] = codepoint;
                }
            }
        }
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:436-481
unsafe fn populate_glyph_to_unicode(font: CTFontRef, out: &mut [i32]) {
    out.fill(0);
    let set = unsafe { SkUniqueCFRef::from_raw(CTFontCopyCharacterSet(font)) };
    if set.is_null() {
        unsafe {
            populate_glyph_to_unicode_slow(font, out);
        }
        return;
    }
    let bitmap = unsafe {
        SkUniqueCFRef::from_raw(CFCharacterSetCreateBitmapRepresentation(
            core::ptr::null(),
            set.get(),
        ))
    };
    if bitmap.is_null() {
        return;
    }
    let length = unsafe { CFDataGetLength(bitmap.get()) } as usize;
    if length == 0 {
        return;
    }
    debug_assert!(length >= kPlaneSize);
    let mut bits = unsafe { CFDataGetBytePtr(bitmap.get()) };
    unsafe {
        get_plane_glyph_map(bits, font, out, 0);
    }
    if length <= kPlaneSize {
        return;
    }
    let mut extra = ((length - kPlaneSize) / (1 + kPlaneSize)) as i32;
    debug_assert_eq!(length, kPlaneSize + extra as usize * (1 + kPlaneSize));
    while extra > 0 {
        unsafe {
            bits = bits.add(kPlaneSize);
            let plane = bits.read();
            bits = bits.add(1);
            debug_assert!(plane >= 1 && plane <= 16);
            get_plane_glyph_map(bits, font, out, plane);
        }
        extra -= 1;
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:491-574
impl SkTypeface_Mac {
    pub unsafe fn onGetAdvancedMetrics(
        &self,
    ) -> Option<Box<crate::src::core::SkAdvancedTypefaceMetrics::SkAdvancedTypefaceMetrics>> {
        use crate::src::core::SkAdvancedTypefaceMetrics::{
            FontFlags, FontType, SkAdvancedTypefaceMetrics, StyleFlags,
        };
        use crate::src::ports::SkScalerContext_mac_ct::CGRect;
        use crate::src::sfnt::SkOTTable_OS_2_V4::{SkOTTableOS2_V4, SkOTTableOS2_V4_Type};
        #[link(name = "CoreText", kind = "framework")]
        unsafe extern "C" {
            fn CTFontGetSlantAngle(font: CTFontRef) -> f64;
            fn CTFontGetAscent(font: CTFontRef) -> f64;
            fn CTFontGetDescent(font: CTFontRef) -> f64;
            fn CTFontGetCapHeight(font: CTFontRef) -> f64;
            fn CTFontGetBoundingBox(font: CTFontRef) -> CGRect;
            fn CTFontGetBoundingRectsForGlyphs(
                font: CTFontRef,
                orientation: u32,
                glyphs: *const u16,
                rects: *mut CGRect,
                count: isize,
            ) -> CGRect;
        }
        #[link(name = "CoreGraphics", kind = "framework")]
        unsafe extern "C" {
            fn CGRectGetMinX(r: CGRect) -> f64;
            fn CGRectGetMaxX(r: CGRect) -> f64;
            fn CGRectGetMinY(r: CGRect) -> f64;
            fn CGRectGetMaxY(r: CGRect) -> f64;
        }
        let ct_font = unsafe {
            crate::src::utils::mac::SkCTFontCreateExactCopy::SkCTFontCreateExactCopy(
                self.fFontRef.get(),
                CTFontGetUnitsPerEm(self.fFontRef.get()) as f64,
                self.fOpszVariation,
            )
        };
        let mut info = Box::new(SkAdvancedTypefaceMetrics::default());
        let axes = self.getVariationAxes();
        if !axes.is_null() && unsafe { CFArrayGetCount(axes) } > 0 {
            info.fFlags |= FontFlags::kVariable_FontFlag;
        }
        let mut fs_type = core::mem::MaybeUninit::<SkOTTableOS2_V4_Type>::uninit();
        if core::mem::size_of::<SkOTTableOS2_V4_Type>()
            == unsafe {
                self.onGetTableData(
                    u32::from_be_bytes(*b"OS/2"),
                    core::mem::offset_of!(SkOTTableOS2_V4, fsType),
                    core::mem::size_of::<SkOTTableOS2_V4_Type>(),
                    fs_type.as_mut_ptr().cast(),
                )
            }
        {
            crate::src::sfnt::SkOTUtils::SetAdvancedTypefaceFlags(
                unsafe { fs_type.assume_init() },
                &mut info,
            );
        }
        const GLYF: u32 = u32::from_be_bytes(*b"glyf");
        const LOCA: u32 = u32::from_be_bytes(*b"loca");
        const CFF: u32 = u32::from_be_bytes(*b"CFF ");
        if unsafe { self.onGetTableData(GLYF, 0, u32::MAX as usize, core::ptr::null_mut()) } != 0
            && unsafe { self.onGetTableData(LOCA, 0, u32::MAX as usize, core::ptr::null_mut()) }
                != 0
        {
            info.fType = FontType::kTrueType_Font;
        } else if unsafe { self.onGetTableData(CFF, 0, u32::MAX as usize, core::ptr::null_mut()) }
            != 0
        {
            info.fType = FontType::kCFF_Font;
        } else {
            return Some(info);
        }
        let symbolic = unsafe { CTFontGetSymbolicTraits(ct_font.get()) };
        if symbolic & (1 << 10) != 0 {
            info.fStyle |= StyleFlags::kFixedPitch_Style;
        }
        if symbolic & 1 != 0 {
            info.fStyle |= StyleFlags::kItalic_Style;
        }
        let stylistic = symbolic & 0xF0000000;
        if stylistic >= 0x10000000 && stylistic <= 0x50000000 {
            info.fStyle |= StyleFlags::kSerif_Style;
        } else if stylistic & 0xA0000000 != 0 {
            info.fStyle |= StyleFlags::kScript_Style;
        }
        info.fItalicAngle = unsafe { CTFontGetSlantAngle(ct_font.get()) } as i16;
        info.fAscent = unsafe { CTFontGetAscent(ct_font.get()) } as i16;
        info.fDescent = unsafe { CTFontGetDescent(ct_font.get()) } as i16;
        info.fCapHeight = unsafe { CTFontGetCapHeight(ct_font.get()) } as i16;
        let bbox = unsafe { CTFontGetBoundingBox(ct_font.get()) };
        let mut rect = crate::src::core::SkRect::SkRect::default();
        rect.setLTRB(
            unsafe { CGRectGetMinX(bbox) } as f32,
            unsafe { CGRectGetMaxY(bbox) } as f32,
            unsafe { CGRectGetMaxX(bbox) } as f32,
            unsafe { CGRectGetMinY(bbox) } as f32,
        );
        rect.roundOutTo(&mut info.fBBox);
        let mut min_width = i16::MAX;
        info.fStemV = 0;
        const STEM_CHARS: [u16; 4] = [b'i' as u16, b'I' as u16, b'!' as u16, b'1' as u16];
        let count = STEM_CHARS.len();
        let mut glyphs = core::mem::MaybeUninit::<[u16; 4]>::uninit();
        let mut rects = core::mem::MaybeUninit::<[CGRect; 4]>::uninit();
        if unsafe {
            CTFontGetGlyphsForCharacters(
                ct_font.get(),
                STEM_CHARS.as_ptr(),
                glyphs.as_mut_ptr().cast(),
                count as isize,
            )
        } {
            unsafe {
                CTFontGetBoundingRectsForGlyphs(
                    ct_font.get(),
                    0,
                    glyphs.as_ptr().cast(),
                    rects.as_mut_ptr().cast(),
                    count as isize,
                );
            }
            let rects = unsafe { rects.assume_init() };
            for i in 0..count {
                let width = rects[i].size.width as i16;
                if width > 0 && width < min_width {
                    min_width = width;
                    info.fStemV = min_width;
                }
            }
        }
        Some(info)
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:483-489
impl SkTypeface_Mac {
    pub unsafe fn getGlyphToUnicodeMap(&self, dst: &mut [i32]) {
        let font = unsafe {
            crate::src::utils::mac::SkCTFontCreateExactCopy::SkCTFontCreateExactCopy(
                self.fFontRef.get(),
                CTFontGetUnitsPerEm(self.fFontRef.get()) as f64,
                self.fOpszVariation,
            )
        };
        let count = (unsafe { CTFontGetGlyphCount(font.get()) } as usize).min(dst.len());
        unsafe {
            populate_glyph_to_unicode(font.get(), &mut dst[..count]);
        }
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:576-603
unsafe fn get_font_type_tag(font: CTFontRef) -> u32 {
    use crate::src::sfnt::SkSFNTHeader::*;
    let format = unsafe {
        SkUniqueCFRef::from_raw(
            CTFontCopyAttribute(font, kCTFontFormatAttribute).cast::<CFNumber>(),
        )
    };
    if format.is_null() {
        return 0;
    }
    let mut value = core::mem::MaybeUninit::<i32>::uninit();
    if unsafe { CFNumberGetValue(format.get(), 3, value.as_mut_ptr().cast()) } == 0 {
        return 0;
    }
    // SDK CTFontFormat enum: unrecognized0, OpenTypePostScript1,
    // OpenTypeTrueType2, TrueType3, PostScript4, Bitmap5.
    match unsafe { value.assume_init() } {
        1 => fontType_OpenTypeCFF::TAG,
        2 => fontType_WindowsTrueType::TAG,
        3 | 5 => fontType_MacTrueType::TAG,
        4 => fontType_PostScript::TAG,
        _ => 0,
    }
}
impl SkTypeface_Mac {
    // Source: src/ports/SkTypeface_mac_ct.h:68-79
    // Construction of the actual backend fields. Canonical Typeface::new/Make
    // stays pending until the complete shared virtual protocol is implemented.
    unsafe fn new_backend(
        font: SkUniqueCFRef<CTFont>,
        opsz: OpszVariation,
        provided: Option<crate::src::core::SkStream::TransferableStreamAsset>,
    ) -> Self {
        debug_assert!(!font.is_null());
        let from_stream = provided.is_some();
        let color = unsafe { CTFontGetSymbolicTraits(font.get()) } & (1 << 13) != 0;
        Self {
            fVariationAxes: std::sync::OnceLock::new(),
            fStream: std::sync::Mutex::new(provided),
            fFontRef: MacFontOwner(font),
            fOpszVariation: opsz,
            fHasColorGlyphs: color,
            fIsFromStream: from_stream,
        }
    }
    // Source: src/ports/SkTypeface_mac_ct.cpp:605-709
    pub unsafe fn onOpenStream(
        &self,
        ttc_index: &mut i32,
    ) -> Option<crate::src::core::SkStream::TransferableStreamAsset> {
        use crate::src::core::SkStream::{SkMemoryStream, TransferableStreamAsset};
        use crate::src::sfnt::SkSFNTHeader::*;
        *ttc_index = 0;
        let mut stream = self.fStream.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(stream) = stream.as_ref() {
            return stream.duplicate();
        }
        let mut font_type = unsafe { get_font_type_tag(self.fFontRef.get()) };
        let num_tables = unsafe { self.onGetTableTags(&mut []) };
        let mut tags = vec![0u32; num_tables as usize];
        unsafe {
            self.onGetTableTags(&mut tags);
        }
        if font_type == 0 {
            font_type = fontType_WindowsTrueType::TAG;
            let mut could_cff = false;
            for &tag in &tags {
                if tag == u32::from_be_bytes(*b"CFF ") || tag == u32::from_be_bytes(*b"CFF2") {
                    could_cff = true;
                }
            }
            if could_cff {
                font_type = fontType_OpenTypeCFF::TAG;
            }
        }
        if font_type == fontType_PostScript::TAG {
            let mut could_type1 = false;
            for &tag in &tags {
                if tag == u32::from_be_bytes(*b"TYP1") || tag == u32::from_be_bytes(*b"CID ") {
                    could_type1 = true;
                }
            }
            if !could_type1 {
                font_type = fontType_OpenTypeCFF::TAG;
            }
        }
        let mut sizes = Vec::with_capacity(tags.len());
        let mut total = core::mem::size_of::<SkSFNTHeader>()
            + core::mem::size_of::<TableDirectoryEntry>() * tags.len();
        for &tag in &tags {
            let size =
                unsafe { self.onGetTableData(tag, 0, u32::MAX as usize, core::ptr::null_mut()) };
            total = total.wrapping_add(size.wrapping_add(3) & !3);
            sizes.push(size);
        }
        let data = crate::src::core::SkData::SkData::MakeZeroInitialized(total);
        let start = data.writable_data();
        let (mut selector, mut search) = (0u16, 1u16);
        while (search as i32) < (num_tables >> 1) {
            selector += 1;
            search = search.wrapping_shl(1);
        }
        search = search.wrapping_shl(4);
        let shift = (num_tables as u16).wrapping_shl(4).wrapping_sub(search);
        unsafe {
            start.cast::<SkSFNTHeader>().write_unaligned(SkSFNTHeader {
                fontType: font_type,
                numTables: (num_tables as u16).to_be(),
                searchRange: search.to_be(),
                entrySelector: selector.to_be(),
                rangeShift: shift.to_be(),
            });
        }
        let mut entry = unsafe {
            start
                .add(core::mem::size_of::<SkSFNTHeader>())
                .cast::<TableDirectoryEntry>()
        };
        let mut ptr = unsafe { entry.add(tags.len()).cast::<u8>() };
        for (tag, size) in tags.into_iter().zip(sizes) {
            unsafe {
                self.onGetTableData(tag, 0, size, ptr);
                let offset = ptr.offset_from(start) as usize;
                debug_assert!(offset <= u32::MAX as usize && size <= u32::MAX as usize);
                let sum = crate::src::sfnt::SkOTUtils::CalcTableChecksum(ptr.cast(), size);
                entry.write_unaligned(TableDirectoryEntry {
                    tag: tag.to_be(),
                    checksum: sum.to_be(),
                    offset: (offset as u32).to_be(),
                    logicalLength: (size as u32).to_be(),
                });
                ptr = ptr.add(size.wrapping_add(3) & !3);
                entry = entry.add(1);
            }
        }
        // Proven concrete producer: sole-owned initialized Data allocation; no external
        // callbacks/borrowed data, immutable shared bytes and independent offsets in
        // every MemoryStream descendant. Cache access remains under this mutex.
        let native = Box::into_pin(Box::new(SkMemoryStream::from_data(Some(data))));
        *stream = Some(unsafe { TransferableStreamAsset::from_native_transferable(native) });
        stream
            .as_ref()
            .expect("native freshly installed stream")
            .duplicate()
    }
    // Source: src/ports/SkTypeface_mac_ct.cpp:711-715
    pub fn onOpenExistingStream(
        &self,
        ttc_index: &mut i32,
    ) -> Option<crate::src::core::SkStream::TransferableStreamAsset> {
        *ttc_index = 0;
        self.fStream
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .and_then(|stream| stream.duplicate())
    }
    // Source: src/ports/SkTypeface_mac_ct.cpp:717-722
    pub fn onGlyphMaskNeedsCurrentColor(&self) -> bool {
        self.fHasColorGlyphs
    }
    // Source: src/ports/SkTypeface_mac_ct.cpp:724-739
    pub fn getVariationAxes(&self) -> CFArrayRef {
        self.fVariationAxes
            .get_or_init(|| unsafe {
                let desc = SkUniqueCFRef::from_raw(CTFontCopyFontDescriptor(self.fFontRef.get()));
                let mut attribute = SkUniqueCFRef::from_raw(CTFontDescriptorCopyAttribute(
                    desc.get(),
                    kCTFontVariationAxesAttribute,
                ));
                let mut array = core::ptr::null_mut();
                if !attribute.is_null() && SkCFDynamicCast::<CFArray>(attribute.get(), &mut array) {
                    attribute.release();
                    return MacAxesOwner(SkUniqueCFRef::from_raw(array));
                }
                MacAxesOwner(SkUniqueCFRef::from_raw(CTFontCopyVariationAxes(
                    self.fFontRef.get(),
                )))
            })
            .get()
    }
    // Source: src/ports/SkTypeface_mac_ct.cpp:741-788
    pub unsafe fn onGetVariationDesignPosition(
        &self,
        coordinates: *mut crate::include::core::SkFontArguments::Coordinate,
        capacity: usize,
    ) -> i32 {
        let axes = self.getVariationAxes();
        if axes.is_null() {
            return -1;
        }
        let count = unsafe { CFArrayGetCount(axes) };
        if capacity < count as usize {
            return count as i32;
        }
        let variation =
            unsafe { SkUniqueCFRef::from_raw(CTFontCopyVariation(self.fFontRef.get())) };
        if variation.is_null() {
            return -1;
        }
        for i in 0..count {
            let mut dictionary = core::ptr::null_mut();
            if !unsafe {
                SkCFDynamicCast::<CFDictionary>(CFArrayGetValueAtIndex(axes, i), &mut dictionary)
            } {
                return -1;
            }
            let mut tag = 0i64;
            let mut number = core::ptr::null_mut();
            let raw = unsafe {
                CFDictionaryGetValue(dictionary, kCTFontVariationAxisIdentifierKey.cast())
            };
            if !unsafe { SkCFNumberDynamicCast(raw, &mut tag, Some(&mut number)) } {
                return -1;
            }
            unsafe {
                core::ptr::addr_of_mut!((*coordinates.add(i as usize)).axis).write(tag as u32);
            }
            let raw = unsafe { CFDictionaryGetValue(variation.get(), number.cast()) };
            let mut value = 0.0f64;
            if !raw.is_null() {
                if !unsafe { SkCFNumberDynamicCast(raw, &mut value, None) } {
                    return -1;
                }
            } else {
                let raw = unsafe {
                    CFDictionaryGetValue(dictionary, kCTFontVariationAxisDefaultValueKey.cast())
                };
                if !unsafe { SkCFNumberDynamicCast(raw, &mut value, None) } {
                    return -1;
                }
            }
            unsafe {
                core::ptr::addr_of_mut!((*coordinates.add(i as usize)).value).write(value as f32);
            }
        }
        count as i32
    }
    // Source: src/ports/SkTypeface_mac_ct.cpp:790-792
    pub fn onGetUPEM(&self) -> i32 {
        unsafe { CTFontGetUnitsPerEm(self.fFontRef.get()) as i32 }
    }
    // Source: src/ports/SkTypeface_mac_ct.cpp:794-817
    pub unsafe fn onCreateFamilyNameIterator(
        &self,
        owner: &crate::src::core::SkTypeface::SkTypeface,
    ) -> Box<dyn crate::src::core::SkTypeface::LocalizedStrings> {
        if let Some(iter) =
            crate::src::sfnt::SkOTUtils::LocalizedStrings_NameTable::MakeForFamilyNames(owner)
        {
            return iter;
        }
        let mut raw_language = core::ptr::null_mut();
        let name = unsafe {
            SkUniqueCFRef::from_raw(CTFontCopyLocalizedName(
                self.fFontRef.get(),
                kCTFontFamilyNameKey,
                &mut raw_language,
            ))
        };
        let language = unsafe { SkUniqueCFRef::from_raw(raw_language) };
        let mut family = String::new();
        let mut locale = String::new();
        if !language.is_null() {
            unsafe {
                SkStringFromCFString(language.get(), &mut locale);
            }
        } else {
            locale = "und".to_owned();
        }
        if !name.is_null() {
            unsafe {
                SkStringFromCFString(name.get(), &mut family);
            }
        }
        Box::new(crate::src::sfnt::SkOTUtils::LocalizedStrings_SingleName::new(family, locale))
    }
    // Source: src/ports/SkTypeface_mac_ct.cpp:819-833
    pub unsafe fn onGetTableTags(&self, tags: &mut [u32]) -> i32 {
        let array =
            unsafe { SkUniqueCFRef::from_raw(CTFontCopyAvailableTables(self.fFontRef.get(), 0)) };
        if array.is_null() {
            return 0;
        }
        let count = unsafe { CFArrayGetCount(array.get()) };
        let n = count.min(tags.len() as isize);
        for i in 0..n {
            tags[i as usize] = unsafe { CFArrayGetValueAtIndex(array.get(), i) } as usize as u32;
        }
        count as i32
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:838-846
unsafe fn copy_table_from_font(font: CTFontRef, tag: u32) -> SkUniqueCFRef<CFData> {
    let mut data = unsafe { SkUniqueCFRef::from_raw(CTFontCopyTable(font, tag, 0)) };
    if data.is_null() {
        let graphics =
            unsafe { SkUniqueCFRef::from_raw(CTFontCopyGraphicsFont(font, core::ptr::null_mut())) };
        unsafe {
            data.reset(CGFontCopyTableForTag(graphics.get(), tag));
        }
    }
    data
}

impl SkTypeface_Mac {
    // Source: src/ports/SkTypeface_mac_ct.cpp:848-866
    pub unsafe fn onGetTableData(
        &self,
        tag: u32,
        offset: usize,
        mut length: usize,
        dst: *mut u8,
    ) -> usize {
        let data = unsafe { copy_table_from_font(self.fFontRef.get(), tag) };
        if data.is_null() {
            return 0;
        }
        let size = unsafe { CFDataGetLength(data.get()) } as usize;
        if offset >= size {
            return 0;
        }
        if length > size - offset {
            length = size - offset;
        }
        if !dst.is_null() && length != 0 {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    CFDataGetBytePtr(data.get()).add(offset),
                    dst,
                    length,
                );
            }
        }
        length
    }
    // Source: src/ports/SkTypeface_mac_ct.cpp:868-879
    pub unsafe fn onCopyTableData(
        &self,
        tag: u32,
    ) -> Option<std::sync::Arc<crate::src::core::SkData::SkData>> {
        let mut data = unsafe { copy_table_from_font(self.fFontRef.get(), tag) };
        if data.is_null() {
            return None;
        }
        let bytes = unsafe { CFDataGetBytePtr(data.get()) };
        let length = unsafe { CFDataGetLength(data.get()) } as usize;
        unsafe extern "C" fn release(_: *const u8, ctx: *mut c_void) {
            unsafe {
                CFRelease(ctx.cast());
            }
        }
        Some(unsafe {
            crate::src::core::SkData::SkData::MakeWithProc(
                bytes,
                length,
                Some(release),
                data.release().cast(),
            )
        })
    }
}
#[link(name = "CoreText", kind = "framework")]
unsafe extern "C" {
    fn CTFontCopyFamilyName(font: CTFontRef) -> CFStringRef;
    fn CTFontCopyFullName(font: CTFontRef) -> CFStringRef;
    fn CTFontCopyPostScriptName(font: CTFontRef) -> CFStringRef;
    fn CTFontGetGlyphsForCharacters(
        font: CTFontRef,
        characters: *const u16,
        glyphs: *mut u16,
        count: CFIndex,
    ) -> bool;
    fn CTFontGetGlyphCount(font: CTFontRef) -> CFIndex;
    static kCTFontVariationAxisMinimumValueKey: CFStringRef;
    static kCTFontVariationAxisMaximumValueKey: CFStringRef;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFBooleanGetValue(value: *mut CFBoolean) -> u8;
}
unsafe extern "C" {
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}
impl SkTypeface_Mac {
    // Source: src/ports/SkTypeface_mac_ct.cpp:881-885
    pub unsafe fn onCreateScalerContext(
        &self,
        typeface: core::ptr::NonNull<crate::src::core::SkTypeface::SkTypeface>,
        effects: &crate::src::core::SkScalerContext::SkScalerContextEffects<'_>,
        desc: &crate::src::core::SkDescriptor::SkDescriptor,
        path: &crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkPathEffect::SkPathEffect,
        >,
        mask: Option<&std::sync::Arc<dyn crate::src::core::SkMaskFilter::SkMaskFilter>>,
    ) -> Box<dyn crate::src::core::SkScalerContext::SkScalerContextVirtual> {
        Box::new(unsafe {
            crate::src::ports::SkScalerContext_mac_ct::SkScalerContext_Mac::new(
                typeface, effects, desc, path, mask, self,
            )
        })
    }
    // Source: src/ports/SkTypeface_mac_ct.cpp:887-987
    pub fn onFilterRec(&self, rec: &mut crate::src::core::SkScalerContext::SkScalerContextRec) {
        use crate::include::core::SkFontTypes::SkFontHinting;
        use crate::src::core::SkMask::Format;
        use crate::src::core::SkScalerContext::flags;
        use crate::src::utils::mac::SkCTFont::{SkCTFontGetSmoothBehavior, SkCTFontSmoothBehavior};
        rec.useStrokeForFakeBold();
        if rec.fFlags & (flags::LCD_BGR_ORDER | flags::LCD_VERTICAL) != 0 {
            rec.fMaskFormat = Format::kA8_Format;
            rec.set_hinting(SkFontHinting::kNormal);
        }
        rec.fFlags &= !(flags::FORCE_AUTOHINTING | flags::LCD_BGR_ORDER | flags::LCD_VERTICAL);
        let smooth = SkCTFontGetSmoothBehavior();
        if rec.get_hinting() != SkFontHinting::kNone {
            rec.set_hinting(SkFontHinting::kNormal);
        }
        if smooth == SkCTFontSmoothBehavior::none {
            rec.set_hinting(SkFontHinting::kNone);
        }
        if rec.fMaskFormat == Format::kLCD16_Format {
            if smooth == SkCTFontSmoothBehavior::subpixel {
                rec.fMaskFormat = Format::kLCD16_Format;
                rec.set_hinting(SkFontHinting::kNormal);
            } else {
                rec.fMaskFormat = Format::kA8_Format;
                if smooth != SkCTFontSmoothBehavior::none {
                    rec.set_hinting(SkFontHinting::kNormal);
                }
            }
        }
        if self.fHasColorGlyphs {
            rec.fMaskFormat = Format::kARGB32_Format;
        }
        if rec.fMaskFormat == Format::kA8_Format && rec.get_hinting() == SkFontHinting::kNone {
            // Default native build does not define SK_GAMMA_APPLY_TO_A8.
            rec.ignore_pre_blend();
        } else {
            use crate::src::core::SkColor::{SkColorGetB, SkColorGetG, SkColorGetR, SkColorSetRGB};
            let mut color = rec.get_luminance_color();
            if smooth == SkCTFontSmoothBehavior::some {
                color = SkColorSetRGB(
                    SkColorGetR(color) * 1 / 2,
                    SkColorGetG(color) * 1 / 2,
                    SkColorGetB(color) * 1 / 2,
                );
            } else if smooth == SkCTFontSmoothBehavior::subpixel {
                color = SkColorSetRGB(
                    SkColorGetR(color) * 3 / 4,
                    SkColorGetG(color) * 3 / 4,
                    SkColorGetB(color) * 3 / 4,
                );
            }
            rec.set_luminance_color(color);
            rec.set_contrast(0.0);
        }
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:990-997
unsafe fn get_str(value: CFStringRef, out: &mut String) -> Option<&str> {
    if value.is_null() {
        return None;
    }
    unsafe {
        SkStringFromCFString(value, out);
        CFRelease(value.cast());
    }
    Some(out.as_str())
}
impl SkTypeface_Mac {
    // Source: src/ports/SkTypeface_mac_ct.cpp:999-1001
    pub unsafe fn onGetFamilyName(&self, name: &mut String) {
        unsafe {
            get_str(CTFontCopyFamilyName(self.fFontRef.get()), name);
        }
    }
    // Source: src/ports/SkTypeface_mac_ct.cpp:1003-1012
    pub unsafe fn onGetPostScriptName(&self, name: Option<&mut String>) -> bool {
        let value =
            unsafe { SkUniqueCFRef::from_raw(CTFontCopyPostScriptName(self.fFontRef.get())) };
        if value.is_null() {
            return false;
        }
        if let Some(name) = name {
            unsafe {
                SkStringFromCFString(value.get(), name);
            }
        }
        true
    }
    // Source: src/ports/SkTypeface_mac_ct.cpp:1014-1024
    pub unsafe fn onGetFontDescriptor(
        &self,
        owner: &crate::src::core::SkTypeface::SkTypeface,
        desc: &mut crate::src::core::SkFontDescriptor::SkFontDescriptor,
        is_local: &mut bool,
    ) {
        let mut text = String::new();
        desc.setFamilyName(
            unsafe { get_str(CTFontCopyFamilyName(self.fFontRef.get()), &mut text) }.unwrap_or(""),
        );
        desc.setFullName(
            unsafe { get_str(CTFontCopyFullName(self.fFontRef.get()), &mut text) }.unwrap_or(""),
        );
        desc.setPostscriptName(
            unsafe { get_str(CTFontCopyPostScriptName(self.fFontRef.get()), &mut text) }
                .unwrap_or(""),
        );
        desc.setStyle(owner.fontStyle());
        desc.setFactoryId(Self::FactoryId);
        *is_local = self.fIsFromStream;
    }
    // Source: src/ports/SkTypeface_mac_ct.cpp:1026-1069
    /// Safety: native Unicode inputs are valid for SkUTF::ToUTF16; the font and
    /// output backing remain live for CT's synchronous initialization/compaction.
    pub unsafe fn onCharsToGlyphs(&self, characters: &[i32], glyphs: &mut [u16]) {
        debug_assert_eq!(characters.len(), glyphs.len());
        assert!(characters.len() <= i32::MAX as usize);
        let count = characters.len();
        let mut storage = Vec::<core::mem::MaybeUninit<u16>>::with_capacity(2 * count);
        let src = storage.as_mut_ptr().cast::<u16>();
        let mut cursor = src;
        for &character in characters {
            let written = unsafe { crate::src::core::SkUTF::ToUTF16(character, cursor) };
            cursor = unsafe { cursor.add(written) };
        }
        let src_count = unsafe { cursor.offset_from(src) } as usize;
        let mut glyph_storage = Vec::<core::mem::MaybeUninit<u16>>::new();
        let mac_glyphs = if src_count > count {
            glyph_storage.reserve_exact(src_count);
            glyph_storage.as_mut_ptr().cast::<u16>()
        } else {
            glyphs.as_mut_ptr()
        };
        unsafe {
            CTFontGetGlyphsForCharacters(self.fFontRef.get(), src, mac_glyphs, src_count as isize);
        }
        if src_count > count {
            let mut extra = 0;
            for i in 0..count {
                glyphs[i] = unsafe { mac_glyphs.add(i + extra).read() };
                if crate::src::core::SkUTF::IsLeadingSurrogateUTF16(unsafe {
                    src.add(i + extra).read()
                }) {
                    extra += 1;
                }
            }
        }
    }
    // Source: src/ports/SkTypeface_mac_ct.cpp:1071-1073
    pub fn onCountGlyphs(&self) -> i32 {
        let count = unsafe { CTFontGetGlyphCount(self.fFontRef.get()) };
        debug_assert!(count <= i32::MAX as isize);
        count as i32
    }
}
// Actual CFDictionary/CFAllocator SDK callback layouts, importing native retain
// callbacks. Rust maps neither dictionary entries nor allocation owners to a
// second container/reference counter.
type CFCallbackRetain = unsafe extern "C" fn(*const c_void, *const c_void) -> *const c_void;
type CFCallbackRelease = unsafe extern "C" fn(*const c_void, *const c_void);
type CFCallbackDescription = unsafe extern "C" fn(*const c_void) -> CFStringRef;
type CFCallbackEqual = unsafe extern "C" fn(*const c_void, *const c_void) -> u8;
#[repr(C)]
struct DictionaryKeys {
    version: isize,
    retain: Option<CFCallbackRetain>,
    release: Option<CFCallbackRelease>,
    copyDescription: Option<CFCallbackDescription>,
    equal: Option<CFCallbackEqual>,
    hash: Option<unsafe extern "C" fn(*const c_void) -> usize>,
}
#[repr(C)]
struct DictionaryValues {
    version: isize,
    retain: Option<CFCallbackRetain>,
    release: Option<CFCallbackRelease>,
    copyDescription: Option<CFCallbackDescription>,
    equal: Option<CFCallbackEqual>,
}
#[repr(C)]
struct CFAllocator {
    _opaque: [u8; 0],
}
#[repr(C)]
struct AllocatorContext {
    version: isize,
    info: *mut c_void,
    retain: Option<unsafe extern "C" fn(*const c_void) -> *const c_void>,
    release: Option<unsafe extern "C" fn(*const c_void)>,
    copyDescription: Option<unsafe extern "C" fn(*const c_void) -> CFStringRef>,
    allocate: Option<unsafe extern "C" fn(isize, usize, *mut c_void) -> *mut c_void>,
    reallocate: Option<unsafe extern "C" fn(*mut c_void, isize, usize, *mut c_void) -> *mut c_void>,
    deallocate: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
    preferredSize: Option<unsafe extern "C" fn(isize, usize, *mut c_void) -> isize>,
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFTypeDictionaryKeyCallBacks: DictionaryKeys;
    static kCFTypeDictionaryValueCallBacks: DictionaryValues;
    fn CFDictionaryCreateMutable(
        allocator: *const c_void,
        capacity: CFIndex,
        keys: *const DictionaryKeys,
        values: *const DictionaryValues,
    ) -> CFDictionaryRef;
    fn CFDictionaryAddValue(dict: CFDictionaryRef, key: *const c_void, value: *const c_void);
    fn CFDictionarySetValue(dict: CFDictionaryRef, key: *const c_void, value: *const c_void);
    fn CFNumberCreate(allocator: *const c_void, kind: isize, value: *const c_void) -> CFNumberRef;
    fn CFAllocatorCreate(
        allocator: *const c_void,
        context: *const AllocatorContext,
    ) -> *mut CFAllocator;
    fn CFDataCreateWithBytesNoCopy(
        allocator: *const c_void,
        bytes: *const u8,
        length: isize,
        bytes_deallocator: *mut CFAllocator,
    ) -> *mut CFData;
}
#[link(name = "CoreText", kind = "framework")]
unsafe extern "C" {
    static kCTFontVariationAttribute: CFStringRef;
    fn CTFontDescriptorCreateWithAttributes(attributes: CFDictionaryRef) -> CTFontDescriptorRef;
    fn CTFontCreateCopyWithAttributes(
        font: CTFontRef,
        size: f64,
        matrix: *const crate::src::ports::SkScalerContext_mac_ct::CGAffineTransform,
        attributes: CTFontDescriptorRef,
    ) -> CTFontRef;
    fn CTFontManagerCreateFontDescriptorFromData(data: *mut CFData) -> CTFontDescriptorRef;
    fn CTFontCreateWithFontDescriptor(
        desc: CTFontDescriptorRef,
        size: f64,
        matrix: *const crate::src::ports::SkScalerContext_mac_ct::CGAffineTransform,
    ) -> CTFontRef;
}
impl Default for CTFontVariation {
    fn default() -> Self {
        Self {
            variation: core::mem::ManuallyDrop::new(SkUniqueCFRef::default()),
            wrongOpszVariation: core::mem::ManuallyDrop::new(SkUniqueCFRef::default()),
            opsz: OpszVariation::default(),
        }
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:1075-1178
unsafe fn ctvariation_from_SkFontArguments(
    font: CTFontRef,
    axes: CFArrayRef,
    args: &crate::include::core::SkFontArguments::SkFontArguments,
) -> CTFontVariation {
    let mut opsz = OpszVariation::default();
    let opsz_tag = u32::from_be_bytes(*b"opsz") as i64;
    if axes.is_null() {
        return CTFontVariation::default();
    }
    let count = unsafe { CFArrayGetCount(axes) };
    let old = unsafe { SkUniqueCFRef::from_raw(CTFontCopyVariation(font)) };
    let position = args.getVariationDesignPosition();
    let variation = unsafe {
        SkUniqueCFRef::from_raw(CFDictionaryCreateMutable(
            core::ptr::null(),
            count,
            &kCFTypeDictionaryKeyCallBacks,
            &kCFTypeDictionaryValueCallBacks,
        ))
    };
    let mut wrong = SkUniqueCFRef::default();
    for i in 0..count {
        let mut dict = core::ptr::null_mut();
        if !unsafe { SkCFDynamicCast::<CFDictionary>(CFArrayGetValueAtIndex(axes, i), &mut dict) } {
            return CTFontVariation::default();
        }
        let mut tag = 0i64;
        let mut number = core::ptr::null_mut();
        if !unsafe {
            SkCFNumberDynamicCast(
                CFDictionaryGetValue(dict, kCTFontVariationAxisIdentifierKey.cast()),
                &mut tag,
                Some(&mut number),
            )
        } {
            return CTFontVariation::default();
        }
        let (mut min, mut max, mut def) = (0.0f64, 0.0f64, 0.0f64);
        if !unsafe {
            SkCFNumberDynamicCast(
                CFDictionaryGetValue(dict, kCTFontVariationAxisMinimumValueKey.cast()),
                &mut min,
                None,
            )
        } || !unsafe {
            SkCFNumberDynamicCast(
                CFDictionaryGetValue(dict, kCTFontVariationAxisMaximumValueKey.cast()),
                &mut max,
                None,
            )
        } || !unsafe {
            SkCFNumberDynamicCast(
                CFDictionaryGetValue(dict, kCTFontVariationAxisDefaultValueKey.cast()),
                &mut def,
                None,
            )
        } {
            return CTFontVariation::default();
        }
        let mut value = def;
        let mut have_current = false;
        let mut current = 0.0f64;
        if !old.is_null() {
            let raw = unsafe { CFDictionaryGetValue(old.get(), number.cast()) };
            if !raw.is_null() {
                if !unsafe { SkCFNumberDynamicCast(raw, &mut value, None) } {
                    return CTFontVariation::default();
                }
                current = value;
                have_current = true;
            }
        }
        for j in (0..position.coordinateCount).rev() {
            let coordinate = unsafe { position.coordinates.add(j as usize).read() };
            if coordinate.axis as i64 == tag {
                value = crate::include::private::SkTPin::SkTPin(coordinate.value as f64, min, max);
                if tag == opsz_tag {
                    opsz.isSet = true;
                }
                break;
            }
        }
        if tag == opsz_tag {
            opsz.value = value;
            if have_current && value == current {
                let mut wrong_value = (max - min) / 2.0 + min;
                if wrong_value == current {
                    wrong_value = (max - min) / 4.0 + min;
                }
                unsafe {
                    wrong.reset(CFDictionaryCreateMutable(
                        core::ptr::null(),
                        0,
                        &kCFTypeDictionaryKeyCallBacks,
                        &kCFTypeDictionaryValueCallBacks,
                    ));
                }
                let wrong_number = unsafe {
                    SkUniqueCFRef::from_raw(CFNumberCreate(
                        core::ptr::null(),
                        13,
                        (&wrong_value as *const f64).cast(),
                    ))
                };
                unsafe {
                    CFDictionarySetValue(wrong.get(), number.cast(), wrong_number.get().cast());
                }
            }
        }
        let value_number = unsafe {
            SkUniqueCFRef::from_raw(CFNumberCreate(
                core::ptr::null(),
                13,
                (&value as *const f64).cast(),
            ))
        };
        unsafe {
            CFDictionaryAddValue(variation.get(), number.cast(), value_number.get().cast());
        }
    }
    CTFontVariation {
        variation: core::mem::ManuallyDrop::new(variation),
        wrongOpszVariation: core::mem::ManuallyDrop::new(wrong),
        opsz,
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:1180-1228
impl SkTypeface_Mac {
    pub unsafe fn onMakeClone(
        &self,
        args: &crate::include::core::SkFontArguments::SkFontArguments,
    ) -> Option<crate::src::core::SkTypeface::SkTypefaceRef> {
        let variation = unsafe {
            ctvariation_from_SkFontArguments(self.fFontRef.get(), self.getVariationAxes(), args)
        };
        let mut variant = SkUniqueCFRef::default();
        if !variation.variation.is_null() {
            let attributes = unsafe {
                SkUniqueCFRef::from_raw(CFDictionaryCreateMutable(
                    core::ptr::null(),
                    0,
                    &kCFTypeDictionaryKeyCallBacks,
                    &kCFTypeDictionaryValueCallBacks,
                ))
            };
            let mut font = self.fFontRef.get();
            let mut wrong_font = SkUniqueCFRef::default();
            if !variation.wrongOpszVariation.is_null() {
                unsafe {
                    CFDictionarySetValue(
                        attributes.get(),
                        kCTFontVariationAttribute.cast(),
                        variation.wrongOpszVariation.get().cast(),
                    );
                }
                let desc = unsafe {
                    SkUniqueCFRef::from_raw(CTFontDescriptorCreateWithAttributes(attributes.get()))
                };
                unsafe {
                    wrong_font.reset(CTFontCreateCopyWithAttributes(
                        font,
                        0.0,
                        core::ptr::null(),
                        desc.get(),
                    ));
                }
                font = wrong_font.get();
            }
            unsafe {
                CFDictionarySetValue(
                    attributes.get(),
                    kCTFontVariationAttribute.cast(),
                    variation.variation.get().cast(),
                );
            }
            let desc = unsafe {
                SkUniqueCFRef::from_raw(CTFontDescriptorCreateWithAttributes(attributes.get()))
            };
            unsafe {
                variant.reset(CTFontCreateCopyWithAttributes(
                    font,
                    0.0,
                    core::ptr::null(),
                    desc.get(),
                ));
            }
        } else {
            unsafe {
                variant.reset(CFRetain(self.fFontRef.get().cast()).cast_mut().cast());
            }
        }
        if variant.is_null() {
            return None;
        }
        let stream = self.fStream.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            Self::Make(
                variant,
                variation.opsz,
                stream.as_ref().and_then(|stream| stream.duplicate()),
            )
        }
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:1230-1238
unsafe fn skdata_from_skstreamasset(
    mut stream: crate::src::core::SkStream::TransferableStreamAsset,
) -> Option<std::sync::Arc<crate::src::core::SkData::SkData>> {
    use crate::src::core::SkStream::{SkStream, SkStreamAsset, TransferableStreamAsset};
    let size = SkStreamAsset::getLength(stream.as_asset());
    let base = SkStream::getMemoryBase(stream.as_asset_mut());
    if !base.is_null() {
        // The outer Box carries Rust's fat unique owner metadata; the same native
        // stream and its recursive transfer proof survive until this Data's last drop.
        unsafe extern "C" fn release(_: *const u8, ctx: *mut c_void) {
            unsafe {
                drop(Box::from_raw(ctx.cast::<TransferableStreamAsset>()));
            }
        }
        let owner = Box::into_raw(Box::new(stream));
        return Some(unsafe {
            crate::src::core::SkData::SkData::MakeWithProc(base, size, Some(release), owner.cast())
        });
    }
    crate::src::core::SkData::SkData::MakeFromStream(stream.as_asset_mut(), size)
}
// Source: src/ports/SkTypeface_mac_ct.cpp:1240-1261
unsafe fn cfdata_from_skdata(
    data: std::sync::Arc<crate::src::core::SkData::SkData>,
) -> SkUniqueCFRef<CFData> {
    let addr = data.data();
    let size = data.size();
    unsafe extern "C" fn deallocate(_: *mut c_void, info: *mut c_void) {
        crate::SkASSERT!(@source b"src/ports/SkTypeface_mac_ct.cpp\0".as_ptr(),1253; @condition b"info\0".as_ptr(); !info.is_null());
        unsafe {
            drop(std::sync::Arc::from_raw(
                info.cast::<crate::src::core::SkData::SkData>(),
            ));
        }
    }
    let context = AllocatorContext {
        version: 0,
        info: std::sync::Arc::into_raw(data).cast_mut().cast(),
        retain: None,
        release: None,
        copyDescription: None,
        allocate: None,
        reallocate: None,
        deallocate: Some(deallocate),
        preferredSize: None,
    };
    let allocator =
        unsafe { SkUniqueCFRef::from_raw(CFAllocatorCreate(core::ptr::null(), &context)) };
    unsafe {
        SkUniqueCFRef::from_raw(CFDataCreateWithBytesNoCopy(
            core::ptr::null(),
            addr,
            size as isize,
            allocator.get(),
        ))
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:1263-1280
unsafe fn ctfont_from_skdata(
    data: std::sync::Arc<crate::src::core::SkData::SkData>,
    ttc_index: i32,
) -> SkUniqueCFRef<CTFont> {
    if ttc_index != 0 {
        return SkUniqueCFRef::default();
    }
    let data = unsafe { cfdata_from_skdata(data) };
    let desc =
        unsafe { SkUniqueCFRef::from_raw(CTFontManagerCreateFontDescriptorFromData(data.get())) };
    if desc.is_null() {
        return SkUniqueCFRef::default();
    }
    unsafe {
        SkUniqueCFRef::from_raw(CTFontCreateWithFontDescriptor(
            desc.get(),
            0.0,
            core::ptr::null(),
        ))
    }
}
// Source: src/ports/SkTypeface_mac_ct.cpp:1282-1324
impl SkTypeface_Mac {
    /// Safety: arguments' raw coordinate backing is valid through the call. The
    /// stream has an actual proven transferable owner, not merely a trait cast.
    pub unsafe fn MakeFromStream(
        stream: crate::src::core::SkStream::TransferableStreamAsset,
        args: &crate::include::core::SkFontArguments::SkFontArguments,
    ) -> Option<crate::src::core::SkTypeface::SkTypefaceRef> {
        let index = args.getCollectionIndex();
        if index != 0 {
            return None;
        }
        let data = unsafe {
            skdata_from_skstreamasset(
                stream
                    .duplicate()
                    .expect("native stream duplicate precondition"),
            )
        }?;
        let font = unsafe { ctfont_from_skdata(data, index) };
        if font.is_null() {
            return None;
        }
        let mut variation = CTFontVariation::default();
        let variant;
        if args.getVariationDesignPosition().coordinateCount == 0 {
            variant = font;
        } else {
            let axes = unsafe { SkUniqueCFRef::from_raw(CTFontCopyVariationAxes(font.get())) };
            variation = unsafe { ctvariation_from_SkFontArguments(font.get(), axes.get(), args) };
            if !variation.variation.is_null() {
                let attributes = unsafe {
                    SkUniqueCFRef::from_raw(CFDictionaryCreateMutable(
                        core::ptr::null(),
                        0,
                        &kCFTypeDictionaryKeyCallBacks,
                        &kCFTypeDictionaryValueCallBacks,
                    ))
                };
                unsafe {
                    CFDictionaryAddValue(
                        attributes.get(),
                        kCTFontVariationAttribute.cast(),
                        variation.variation.get().cast(),
                    );
                }
                let desc = unsafe {
                    SkUniqueCFRef::from_raw(CTFontDescriptorCreateWithAttributes(attributes.get()))
                };
                variant = unsafe {
                    SkUniqueCFRef::from_raw(CTFontCreateCopyWithAttributes(
                        font.get(),
                        0.0,
                        core::ptr::null(),
                        desc.get(),
                    ))
                };
            } else {
                variant = font;
            }
        }
        if variant.is_null() {
            return None;
        }
        unsafe { Self::Make(variant, variation.opsz, Some(stream)) }
    }
}

impl SkTypeface_Mac {
    // Source: src/ports/SkTypeface_mac_ct.cpp:1326-1392
    pub unsafe fn onGetVariationDesignParameters(
        &self,
        parameters: &mut [crate::include::core::SkFontParameters::Axis],
    ) -> i32 {
        let axes = self.getVariationAxes();
        if axes.is_null() {
            return -1;
        }
        let count = unsafe { CFArrayGetCount(axes) };
        if parameters.len() < count as usize {
            return count as i32;
        }
        static HIDDEN_KEY: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        let hidden_key = *HIDDEN_KEY.get_or_init(|| unsafe {
            dlsym(
                (-2isize) as *mut c_void,
                b"kCTFontVariationAxisHiddenKey\0".as_ptr().cast(),
            ) as usize
        }) as *const CFStringRef;
        for i in 0..count {
            let mut dict = core::ptr::null_mut();
            if !unsafe {
                SkCFDynamicCast::<CFDictionary>(CFArrayGetValueAtIndex(axes, i), &mut dict)
            } {
                return -1;
            }
            let (mut tag, mut min, mut max, mut def) = (0i64, 0f64, 0f64, 0f64);
            if !unsafe {
                SkCFNumberDynamicCast(
                    CFDictionaryGetValue(dict, kCTFontVariationAxisIdentifierKey.cast()),
                    &mut tag,
                    None,
                )
            } {
                return -1;
            }
            if !unsafe {
                SkCFNumberDynamicCast(
                    CFDictionaryGetValue(dict, kCTFontVariationAxisMinimumValueKey.cast()),
                    &mut min,
                    None,
                )
            } || !unsafe {
                SkCFNumberDynamicCast(
                    CFDictionaryGetValue(dict, kCTFontVariationAxisMaximumValueKey.cast()),
                    &mut max,
                    None,
                )
            } || !unsafe {
                SkCFNumberDynamicCast(
                    CFDictionaryGetValue(dict, kCTFontVariationAxisDefaultValueKey.cast()),
                    &mut def,
                    None,
                )
            } {
                return -1;
            }
            let axis = &mut parameters[i as usize];
            axis.tag = tag as u32;
            axis.min = min as f32;
            axis.max = max as f32;
            axis.def = def as f32;
            axis.setHidden(false);
            if !hidden_key.is_null() {
                let value = unsafe { CFDictionaryGetValue(dict, (*hidden_key).cast()) };
                if !value.is_null() {
                    let mut boolean = core::ptr::null_mut();
                    let mut number = 0i32;
                    if unsafe { SkCFDynamicCast::<CFBoolean>(value, &mut boolean) } {
                        axis.setHidden(unsafe { CFBooleanGetValue(boolean) } != 0);
                    } else if unsafe { SkCFNumberDynamicCast(value, &mut number, None) } {
                        axis.setHidden(number != 0);
                    } else {
                        return -1;
                    }
                }
            }
        }
        count as i32
    }
}

// Source: src/ports/SkTypeface_mac_ct.h:86-86
impl SkTypeface_Mac {
    pub const FactoryId: u32 = u32::from_be_bytes(*b"ctxt");
}
// Source: src/ports/SkTypeface_mac_ct.h:104-120,122-123,125-126,128
// Virtual base projection retains the actual containing canonical owner where
// native inherited base state is read. No duplicate Typeface/style or weak
// backpointer is manufactured. Mac itself acquires Send/Sync from its proven
// immutable resource owners and the actual transferable-stream mutex.
impl crate::src::core::SkTypeface::SkTypefaceVirtuals for SkTypeface_Mac {
    // Source: src/ports/SkTypeface_mac_ct.h:124-124
    fn getPostScriptGlyphNames(&self, _: &mut [crate::src::core::SkString::SkString]) {}
    unsafe fn onMakeClone(
        &self,
        _owner: &crate::src::core::SkTypeface::SkTypefaceRef,
        args: &crate::include::core::SkFontArguments::SkFontArguments,
    ) -> Option<crate::src::core::SkTypeface::SkTypefaceRef> {
        unsafe { Self::onMakeClone(self, args) }
    }
    unsafe fn onCreateScalerContext(
        &self,
        typeface: core::ptr::NonNull<crate::src::core::SkTypeface::SkTypeface>,
        effects: &crate::src::core::SkScalerContext::SkScalerContextEffects<'_>,
        desc: &crate::src::core::SkDescriptor::SkDescriptor,
        path: &crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkPathEffect::SkPathEffect,
        >,
        mask: Option<&std::sync::Arc<dyn crate::src::core::SkMaskFilter::SkMaskFilter>>,
    ) -> Box<dyn crate::src::core::SkScalerContext::SkScalerContextVirtual> {
        unsafe { Self::onCreateScalerContext(self, typeface, effects, desc, path, mask) }
    }
    // Source: src/ports/SkTypeface_mac_ct.h:120-120
    fn onGetAdvancedMetrics(
        &self,
    ) -> Option<Box<crate::src::core::SkAdvancedTypefaceMetrics::SkAdvancedTypefaceMetrics>> {
        unsafe { Self::onGetAdvancedMetrics(self) }
    }
    fn onFilterRec(&self, rec: &mut crate::src::core::SkScalerContext::SkScalerContextRec) {
        Self::onFilterRec(self, rec)
    }
    fn getGlyphToUnicodeMap(&self, dst: &mut [i32]) {
        unsafe { Self::getGlyphToUnicodeMap(self, dst) }
    }
    fn onOpenStream(
        &self,
        _: &crate::src::core::SkTypeface::SkTypeface,
        index: &mut i32,
    ) -> Option<core::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>> {
        unsafe { Self::onOpenStream(self, index) }.map(|stream| stream.into_asset())
    }
    fn onOpenExistingStream(
        &self,
        _: &crate::src::core::SkTypeface::SkTypeface,
        index: &mut i32,
    ) -> Option<core::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>> {
        Self::onOpenExistingStream(self, index).map(|stream| stream.into_asset())
    }
    fn onGlyphMaskNeedsCurrentColor(&self) -> bool {
        Self::onGlyphMaskNeedsCurrentColor(self)
    }
    unsafe fn onGetVariationDesignPosition(
        &self,
        out: *mut crate::include::core::SkFontArguments::Coordinate,
        capacity: usize,
    ) -> i32 {
        unsafe { Self::onGetVariationDesignPosition(self, out, capacity) }
    }
    fn onGetVariationDesignParameters(
        &self,
        out: &mut [crate::include::core::SkFontParameters::Axis],
    ) -> i32 {
        unsafe { Self::onGetVariationDesignParameters(self, out) }
    }
    fn onGetFontDescriptor(
        &self,
        owner: &crate::src::core::SkTypeface::SkTypeface,
        desc: &mut crate::src::core::SkFontDescriptor::SkFontDescriptor,
        local: &mut bool,
    ) {
        unsafe { Self::onGetFontDescriptor(self, owner, desc, local) }
    }
    fn onCharsToGlyphs(&self, chars: &[i32], glyphs: &mut [u16]) {
        // Safe Rust cannot permit the native invalid-Unicode out-of-range compaction
        // precondition to become an uninitialized/out-of-bounds read. Valid native
        // codepoints run the exact UTF16/CT/compaction body; no glyph is synthesized.
        assert!(
            chars.iter().all(|&value| value as u32 <= 0x10ffff),
            "native UTF32 input precondition"
        );
        unsafe { Self::onCharsToGlyphs(self, chars, glyphs) }
    }
    fn onCountGlyphs(&self) -> i32 {
        Self::onCountGlyphs(self)
    }
    fn onGetUPEM(&self) -> i32 {
        Self::onGetUPEM(self)
    }
    fn onGetFamilyName(&self, name: &mut String) {
        unsafe { Self::onGetFamilyName(self, name) }
    }
    fn onGetPostScriptName(&self, name: &mut String) -> bool {
        unsafe { Self::onGetPostScriptName(self, Some(name)) }
    }
    fn onCreateFamilyNameIterator(
        &self,
        owner: &crate::src::core::SkTypeface::SkTypeface,
    ) -> Option<Box<dyn crate::src::core::SkTypeface::LocalizedStrings>> {
        Some(unsafe { Self::onCreateFamilyNameIterator(self, owner) })
    }
    fn onGetTableTags(&self, tags: &mut [u32]) -> i32 {
        unsafe { Self::onGetTableTags(self, tags) }
    }
    fn onGetTableData(
        &self,
        tag: u32,
        offset: usize,
        length: usize,
        data: Option<&mut [u8]>,
    ) -> usize {
        let ptr = if let Some(data) = data {
            assert!(data.len() >= length);
            data.as_mut_ptr()
        } else {
            core::ptr::null_mut()
        };
        unsafe { Self::onGetTableData(self, tag, offset, length, ptr) }
    }
    fn onCopyTableData(
        &self,
        _owner: &crate::src::core::SkTypeface::SkTypeface,
        tag: u32,
    ) -> Option<std::sync::Arc<crate::src::core::SkData::SkData>> {
        unsafe { Self::onCopyTableData(self, tag) }
    }
    fn onGetCTFontRef(&self) -> *mut c_void {
        self.fFontRef.get().cast()
    }
}
