use crate::include::core::SkFontArguments::{Coordinate, SkFontArguments};
use crate::include::core::SkFontParameters::Axis;
use crate::include::core::SkFontStyle::{SkFontStyle, Slant};

use crate::src::core::SkRect::SkRect;

use crate::src::core::SkScalerContext::SkScalerContextRec;

use std::sync::{Arc, OnceLock};

// Source: include/core/SkTypeface.h:54-54
// Native sk_sp retains the same complete Typeface and its actual weak parent.
pub type SkTypefaceRef = crate::include::core::SkRefCnt::sk_sp<SkTypeface>;

// Source: include/core/SkTypeface.h:40-40
pub type SkTypefaceID = u32;

// Source: include/core/SkTypeface.h:43-43
pub type SkFontTableTag = u32;

// Source: include/core/SkTypeface.h:127-131
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SerializeBehavior {
    kDoIncludeData,
    kDontIncludeData,
    kIncludeDataIfLocal,
}

// Source: include/core/SkTypeface.h:153-153
pub type SkTypefaceStreamSanitizerProc =
    fn(Arc<crate::src::core::SkData::SkData>) -> Option<Arc<crate::src::core::SkData::SkData>>;

// Source: include/core/SkTypeface.h:104-104
impl SkTypeface {
    pub fn uniqueID(&self) -> SkTypefaceID {
        self.fUniqueID
    }
}

// Source: include/core/SkTypeface.h:261-275
// Owned UTF-8 strings and a unique boxed iterator replace SkString and native
// delete-this ownership. Rust moves enforce the original noncopyable iterator.
pub struct LocalizedString {
    pub fString: String,
    pub fLanguage: String,
}
pub trait LocalizedStrings {
    fn next(&mut self, localized_string: &mut LocalizedString) -> bool;
    fn unref(self: Box<Self>) {
        drop(self);
    }
}

// Source: include/core/SkTypeface.h:344-346
// UNVERIFIED: native nonvirtual dispatch to the actual containing backend.
impl SkTypeface {
    pub fn filterRec(&self, rec: &mut SkScalerContextRec) {
        self.virtuals.onFilterRec(rec);
    }
}

// Source: include/core/SkTypeface.h:348-350
impl SkTypeface {
    pub fn getFontDescriptor(
        &self,
        desc: &mut crate::src::core::SkFontDescriptor::SkFontDescriptor,
        is_local: &mut bool,
    ) {
        self.virtuals.onGetFontDescriptor(self, desc, is_local);
    }
}

// Source: include/core/SkTypeface.h:352-354
impl SkTypeface {
    pub fn internal_private_getCTFontRef(&self) -> *mut core::ffi::c_void {
        self.virtuals.onGetCTFontRef()
    }
}

// Source: include/core/SkTypeface.h:364-364
impl SkTypeface {
    pub(crate) const MAX_REASONABLE_TABLE_COUNT: i32 = (1 << 16) - 1;
}

// Source: include/core/SkTypeface.h:372-372
// Source: include/core/SkTypeface.h:374-374
impl SkTypeface {
    pub(crate) fn setIsFixedPitch(&mut self, value: bool) {
        self.fIsFixedPitch = value;
    }
    pub(crate) fn setFontStyle(&mut self, value: SkFontStyle) {
        self.fStyle = value;
    }
}

// Rust dispatch objects express the original protected virtual calls. Core identity, style,
// span limiting, cache ownership and algorithms stay on SkTypeface; no platform font manager lives here.
pub trait SkTypefaceVirtuals: Send + Sync {
    // Source: include/core/SkTypeface.h:369-369
    /// Safety: arguments' native pointer/count arrays remain initialized and
    /// readable through this call. `owner` is the actual containing typeface.
    unsafe fn onMakeClone(
        &self,
        owner: &SkTypefaceRef,
        args: &SkFontArguments,
    ) -> Option<SkTypefaceRef>;
    // Source: include/core/SkTypeface.h:376-376
    // Source: include/core/SkTypeface.h:378-378
    fn onGetFontStyle(&self, stored: SkFontStyle) -> SkFontStyle {
        default_font_style(stored)
    }
    fn onGetFixedPitch(&self, stored: bool) -> bool {
        default_fixed_pitch(stored)
    }
    // Source: include/core/SkTypeface.h:381-382
    /// The returned actual scaler retains the same synchronous typeface/backing lease.
    /// Explicit canonical owner handles preserve native effect ref increments.
    unsafe fn onCreateScalerContext(
        &self,
        typeface: core::ptr::NonNull<SkTypeface>,
        effects: &crate::src::core::SkScalerContext::SkScalerContextEffects<'_>,
        desc: &crate::src::core::SkDescriptor::SkDescriptor,
        path_owner: &crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkPathEffect::SkPathEffect,
        >,
        mask_owner: Option<&Arc<dyn crate::src::core::SkMaskFilter::SkMaskFilter>>,
    ) -> Box<dyn crate::src::core::SkScalerContext::SkScalerContextVirtual>;
    // Source: include/core/SkTypeface.h:383-384
    // Source: src/core/SkTypeface.cpp:358-363
    // Native default is a fatal "Not implemented.", never null/empty success.
    // UNVERIFIED: Rust process abort preserves nonreturning termination; native
    // SkAbort's platform logging/stack formatting remains outside this adaptation.
    unsafe fn onCreateScalerContextAsProxyTypeface(
        &self,
        _effects: &crate::src::core::SkScalerContext::SkScalerContextEffects<'_>,
        _desc: &crate::src::core::SkDescriptor::SkDescriptor,
        _proxy_typeface: core::ptr::NonNull<SkTypeface>,
    ) -> Box<dyn crate::src::core::SkScalerContext::SkScalerContextVirtual> {
        let _ = std::io::Write::write_all(&mut std::io::stderr(), b"Not implemented.\n");
        std::process::abort()
    }
    // Source: include/core/SkTypeface.h:385-385
    fn onFilterRec(&self, rec: &mut SkScalerContextRec);
    // Source: include/core/SkTypeface.h:389-389
    fn onGetAdvancedMetrics(
        &self,
    ) -> Option<Box<super::SkAdvancedTypefaceMetrics::SkAdvancedTypefaceMetrics>>;
    // Source: include/core/SkTypeface.h:393-393
    fn getPostScriptGlyphNames(&self, dst: &mut [crate::src::core::SkString::SkString]);
    // Source: include/core/SkTypeface.h:398-398
    fn getGlyphToUnicodeMap(&self, dst: &mut [i32]);
    // Source: include/core/SkTypeface.h:400-400
    fn onOpenStream(
        &self,
        _owner: &SkTypeface,
        ttc_index: &mut i32,
    ) -> Option<core::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>>;
    // Source: include/core/SkTypeface.h:402-402
    fn onOpenExistingStream(
        &self,
        owner: &SkTypeface,
        ttc_index: &mut i32,
    ) -> Option<core::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>> {
        default_open_existing_stream(self, owner, ttc_index)
    }
    // Source: include/core/SkTypeface.h:404-404
    fn onGlyphMaskNeedsCurrentColor(&self) -> bool;
    // Source: include/core/SkTypeface.h:406-407
    /// Safety: coordinates covers capacity writable output entries; it may be
    /// uninitialized. Implementations write fields without reading old entries.
    unsafe fn onGetVariationDesignPosition(
        &self,
        coordinates: *mut Coordinate,
        capacity: usize,
    ) -> i32;
    // Source: include/core/SkTypeface.h:409-409
    fn onGetVariationDesignParameters(&self, parameters: &mut [Axis]) -> i32;
    // Source: include/core/SkTypeface.h:411-412
    fn onIsSyntheticBold(&self) -> bool {
        default_synthetic_bold()
    }
    fn onIsSyntheticOblique(&self) -> bool {
        default_synthetic_oblique()
    }
    // Source: include/core/SkTypeface.h:414-414
    fn onGetFontDescriptor(
        &self,
        owner: &SkTypeface,
        desc: &mut crate::src::core::SkFontDescriptor::SkFontDescriptor,
        is_local: &mut bool,
    );
    // Source: include/core/SkTypeface.h:416-417
    // Source: include/core/SkTypeface.h:419-421
    fn onCharsToGlyphs(&self, chars: &[i32], glyphs: &mut [u16]);
    fn onCountGlyphs(&self) -> i32;
    fn onGetUPEM(&self) -> i32;
    fn onGetKerningPairAdjustments(&self, glyphs: &[u16], adjustments: &mut [i32]) -> bool {
        default_kerning(glyphs, adjustments)
    }
    // Source: include/core/SkTypeface.h:426-428
    fn onGetFamilyName(&self, name: &mut String);
    fn onGetPostScriptName(&self, name: &mut String) -> bool;
    fn onGetResourceName(&self, name: &mut String) -> i32 {
        default_resource_name(name)
    }
    // Source: include/core/SkTypeface.h:431-431
    fn onCreateFamilyNameIterator(&self, owner: &SkTypeface) -> Option<Box<dyn LocalizedStrings>>;
    // Source: include/core/SkTypeface.h:433-435
    fn onGetTableTags(&self, tags: &mut [SkFontTableTag]) -> i32;
    fn onGetTableData(
        &self,
        tag: SkFontTableTag,
        offset: usize,
        length: usize,
        data: Option<&mut [u8]>,
    ) -> usize;
    // Source: include/core/SkTypeface.h:436
    fn onCopyTableData(
        &self,
        typeface: &SkTypeface,
        tag: SkFontTableTag,
    ) -> Option<Arc<crate::src::core::SkData::SkData>> {
        typeface.defaultCopyTableData(tag)
    }
    // Source: include/core/SkTypeface.h:438
    fn onComputeBounds(&self, typeface: &SkTypefaceRef, bounds: &mut SkRect) -> bool {
        typeface.defaultComputeBounds(bounds)
    }
    // Source: include/core/SkTypeface.h:440-440
    fn onGetCTFontRef(&self) -> *mut core::ffi::c_void {
        core::ptr::null_mut()
    }
}

// Source: include/core/SkTypeface.h:460-464
// The native cached rectangle and actual three-state SkOnce remain distinct.
// The sole actual SkWeakRefCnt parent is constructed first and destroyed last.
pub struct SkTypeface {
    fUniqueID: SkTypefaceID,
    fStyle: SkFontStyle,
    fBounds: core::cell::UnsafeCell<SkRect>,
    fBoundsOnce: crate::include::private::SkOnce::SkOnce,
    fIsFixedPitch: bool,
    pub(crate) virtuals: core::mem::ManuallyDrop<Box<dyn SkTypefaceVirtuals>>,
    // Source: include/core/SkTypeface.h:466-466
    // Adapted field order: native parent survives all derived/member cleanup.
    parent: crate::include::private::SkWeakRefCnt::SkWeakRefCnt,
}

// Source: include/core/SkTypeface.h:54-54,466-466
impl core::ops::Deref for SkTypeface {
    type Target = crate::include::private::SkWeakRefCnt::SkWeakRefCnt;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
unsafe impl crate::include::core::SkRefCnt::SkRefCntObject for SkTypeface {
    fn ref_cnt_base(&self) -> &crate::include::core::SkRefCnt::SkRefCntBase {
        self.parent.ref_cnt_base()
    }
    fn ref_(&self) {
        self.parent.ref_();
    }
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::private::SkWeakRefCnt::SkWeakRefCntStrongDispose::<Self>
    }
}
unsafe impl crate::include::private::SkWeakRefCnt::SkWeakRefCntObject for SkTypeface {
    fn weak_ref_cnt(&self) -> &crate::include::private::SkWeakRefCnt::SkWeakRefCnt {
        &self.parent
    }
    fn weak_final_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::private::SkWeakRefCnt::SkWeakRefCntFinalDispose::<Self>
    }
    // Native inherited weak_dispose is empty. The backend stays alive after
    // strong zero whenever external weak references still keep this object live.
}

// Source: src/core/SkTypeface.cpp:60-60
// The boxed virtual implementation holds the real native derived members.
// Destroy it while the containing SkTypeface base is still alive. Its concrete
// Drop releases derived members; Rust then destroys the remaining base fields.
impl Drop for SkTypeface {
    fn drop(&mut self) {
        unsafe {
            core::mem::ManuallyDrop::drop(&mut self.virtuals);
        }
    }
}

// fBounds is mutated only by its actual native SkOnce winner, and read only
// after that flag's acquire publication. Arc receivers keep this allocation
// live and stationary throughout initialization, waiting, and the returned copy.
unsafe impl Sync for SkTypeface {}

fn default_synthetic_oblique() -> bool {
    false
}

impl Utf32Storage {
    fn reset(&mut self, count: usize) -> &mut [i32] {
        *self = if count <= 256 {
            Self::Inline([0; 256])
        } else {
            Self::Heap(vec![0; count])
        };
        match self {
            Self::Inline(v) => &mut v[..count],
            Self::Heap(v) => v.as_mut_slice(),
        }
    }
}

// Source: src/core/SkTypeface.cpp:57-58
// Source: src/core/SkTypeface.cpp:60-60
impl SkTypeface {
    pub fn new(
        style: SkFontStyle,
        is_fixed_pitch: bool,
        virtuals: Box<dyn SkTypefaceVirtuals>,
    ) -> Self {
        Self {
            // Source: src/core/SkTypeface.cpp:57-58
            parent: crate::include::private::SkWeakRefCnt::SkWeakRefCnt::new(),
            fUniqueID: crate::src::core::SkTypefaceCache::NewTypefaceID(),
            fStyle: style,
            // Preserve the existing initialized Rust rectangle required by
            // backend &mut SkRect parameters; native publication is still SkOnce.
            fBounds: core::cell::UnsafeCell::new(SkRect::MakeEmpty()),
            fBoundsOnce: crate::include::private::SkOnce::SkOnce::new(),
            fIsFixedPitch: is_fixed_pitch,
            virtuals: core::mem::ManuallyDrop::new(virtuals),
        }
    }
}

// Source: src/core/SkTypeface.cpp:66-71
struct SkEmptyTypeface;

impl SkEmptyTypeface {
    fn Make() -> SkTypefaceRef {
        static INSTANCE: OnceLock<SkTypefaceRef> = OnceLock::new();
        INSTANCE.get_or_init(Self::make_instance).clone()
    }
    // Source: src/core/SkTypeface.cpp:73-73
    const FactoryId: u32 = u32::from_be_bytes(*b"emty");
    // Source: src/core/SkTypeface.cpp:74-80
    fn MakeFromStream(
        stream: core::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>,
        _args: &SkFontArguments,
    ) -> Option<SkTypefaceRef> {
        if crate::src::core::SkStream::SkStreamAsset::getLength(stream.as_ref().get_ref()) == 0 {
            return Some(Self::Make());
        }
        None
    }
    // Source: src/core/SkTypeface.cpp:83-83
    fn make_instance() -> SkTypefaceRef {
        crate::include::core::SkRefCnt::sk_sp::new(SkTypeface::new(
            SkFontStyle::default(),
            true,
            Box::new(Self),
        ))
    }
}

impl SkTypefaceVirtuals for SkEmptyTypeface {
    // Source: src/core/SkTypeface.cpp:101-101
    fn getPostScriptGlyphNames(&self, _: &mut [crate::src::core::SkString::SkString]) {}
    // Source: src/core/SkTypeface.cpp:85-85
    fn onOpenStream(
        &self,
        _owner: &SkTypeface,
        _ttc_index: &mut i32,
    ) -> Option<core::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>> {
        None
    }
    // Source: src/core/SkTypeface.cpp:86-88
    unsafe fn onMakeClone(
        &self,
        owner: &SkTypefaceRef,
        _args: &SkFontArguments,
    ) -> Option<SkTypefaceRef> {
        Some(owner.clone())
    }
    // Source: src/core/SkTypeface.cpp:89-93
    unsafe fn onCreateScalerContext(
        &self,
        typeface: core::ptr::NonNull<SkTypeface>,
        effects: &crate::src::core::SkScalerContext::SkScalerContextEffects<'_>,
        desc: &crate::src::core::SkDescriptor::SkDescriptor,
        path_owner: &crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkPathEffect::SkPathEffect,
        >,
        mask_owner: Option<&Arc<dyn crate::src::core::SkMaskFilter::SkMaskFilter>>,
    ) -> Box<dyn crate::src::core::SkScalerContext::SkScalerContextVirtual> {
        unsafe {
            crate::src::core::SkScalerContext::SkScalerContext::MakeEmpty(
                typeface, effects, desc, path_owner, mask_owner,
            )
        }
    }
    // Source: src/core/SkTypeface.cpp:94-94
    fn onFilterRec(&self, _rec: &mut SkScalerContextRec) {}
    // Source: src/core/SkTypeface.cpp:95-97
    fn onGetAdvancedMetrics(
        &self,
    ) -> Option<Box<super::SkAdvancedTypefaceMetrics::SkAdvancedTypefaceMetrics>> {
        None
    }
    // Source: src/core/SkTypeface.cpp:98-101
    fn onGetFontDescriptor(
        &self,
        _owner: &SkTypeface,
        desc: &mut crate::src::core::SkFontDescriptor::SkFontDescriptor,
        is_local: &mut bool,
    ) {
        desc.setFactoryId(Self::FactoryId);
        *is_local = false;
    }
    // Source: src/core/SkTypeface.cpp:102-105
    fn onCharsToGlyphs(&self, _chars: &[i32], glyphs: &mut [u16]) {
        glyphs.fill(0);
    }
    fn onCountGlyphs(&self) -> i32 {
        0
    }
    // Source: src/core/SkTypeface.cpp:107-107
    fn getGlyphToUnicodeMap(&self, _dst: &mut [i32]) {}
    // Source: src/core/SkTypeface.cpp:108-108
    fn onGetUPEM(&self) -> i32 {
        0
    }
    // Source: src/core/SkTypeface.cpp:109
    fn onComputeBounds(&self, _typeface: &SkTypefaceRef, _bounds: &mut SkRect) -> bool {
        false
    }
    // Source: src/core/SkTypeface.cpp:115-120
    fn onGetFamilyName(&self, name: &mut String) {
        name.clear();
    }
    fn onGetPostScriptName(&self, _name: &mut String) -> bool {
        false
    }
    // Source: src/core/SkTypeface.cpp:121-123
    fn onCreateFamilyNameIterator(&self, _owner: &SkTypeface) -> Option<Box<dyn LocalizedStrings>> {
        Some(Box::new(EmptyLocalizedStrings))
    }
    // Source: src/core/SkTypeface.cpp:124-126
    fn onGlyphMaskNeedsCurrentColor(&self) -> bool {
        false
    }
    // Source: src/core/SkTypeface.cpp:127-131
    unsafe fn onGetVariationDesignPosition(
        &self,
        _coordinates: *mut Coordinate,
        _capacity: usize,
    ) -> i32 {
        0
    }
    // Source: src/core/SkTypeface.cpp:132-135
    fn onGetVariationDesignParameters(&self, _parameters: &mut [Axis]) -> i32 {
        0
    }
    // Source: src/core/SkTypeface.cpp:136-140
    fn onGetTableTags(&self, _tags: &mut [SkFontTableTag]) -> i32 {
        0
    }
    fn onGetTableData(
        &self,
        _tag: SkFontTableTag,
        _offset: usize,
        _length: usize,
        _data: Option<&mut [u8]>,
    ) -> usize {
        0
    }
}

// Source: src/core/SkTypeface.cpp:111-114
// The nested native iterator type is hoisted to module scope in Rust.
struct EmptyLocalizedStrings;
impl LocalizedStrings for EmptyLocalizedStrings {
    fn next(&mut self, _localized_string: &mut LocalizedString) -> bool {
        false
    }
}

// Source: src/core/SkTypeface.cpp:144-146
// Source: src/core/SkTypeface.cpp:148-156
impl SkTypeface {
    pub fn MakeEmpty() -> SkTypefaceRef {
        SkEmptyTypeface::Make()
    }
    pub fn Equal(facea: Option<&Self>, faceb: Option<&Self>) -> bool {
        let a = facea.map_or(std::ptr::null(), |v| v as *const Self);
        let b = faceb.map_or(std::ptr::null(), |v| v as *const Self);
        if a == b {
            return true;
        }
        if a.is_null() || b.is_null() {
            return false;
        }
        facea.unwrap().uniqueID() == faceb.unwrap().uniqueID()
    }
}

// Source: include/core/SkTypeface.h:120-120
// Source: src/core/SkTypeface.cpp:189-191
impl SkTypeface {
    /// Safety: native argument arrays remain valid during the backend call.
    pub unsafe fn makeClone(self: &SkTypefaceRef, args: &SkFontArguments) -> Option<SkTypefaceRef> {
        unsafe { self.virtuals.onMakeClone(self, args) }
    }
}

// Source: src/core/SkTypeface.cpp:201-233
impl SkTypeface {
    /// Safety: the native font backend must initialize all output coordinates
    /// it reports, and duplicated font streams must supply their declared length.
    pub unsafe fn serialize(
        &self,
        wstream: core::pin::Pin<&mut dyn crate::src::core::SkStream::SkWStream>,
        behavior: SerializeBehavior,
    ) -> bool {
        use crate::src::core::SkFontDescriptor::SkFontDescriptor;
        let mut is_local_data = false;
        let mut desc = SkFontDescriptor::new();
        self.virtuals
            .onGetFontDescriptor(self, &mut desc, &mut is_local_data);
        if desc.getFactoryId() == 0 {
            #[cfg(sk_debug)]
            {
                use std::io::Write;
                let mut output = std::io::stderr().lock();
                let _ = output.write_all(b"Factory was not set for ");
                let name = unsafe { std::ffi::CStr::from_ptr(desc.getFamilyName()) };
                let _ = output.write_all(name.to_bytes());
                let _ = output.write_all(b".\n");
            }
        }
        let should_serialize_data = match behavior {
            SerializeBehavior::kDoIncludeData => true,
            SerializeBehavior::kDontIncludeData => false,
            SerializeBehavior::kIncludeDataIfLocal => is_local_data,
        };
        if should_serialize_data {
            let mut index = 0;
            desc.setStream(self.openStream(Some(&mut index)));
            if desc.hasStream() {
                desc.setCollectionIndex(index);
            }
            let mut num_axes = self.getVariationDesignPosition(&mut []);
            if num_axes > 0 {
                num_axes = unsafe {
                    self.getVariationDesignPosition_raw(
                        desc.setVariationCoordinates(num_axes),
                        num_axes as usize,
                    )
                };
                if num_axes <= 0 {
                    desc.setVariationCoordinates(0);
                }
            }
        }
        unsafe { desc.serialize(wstream) }
    }
    // Source: src/core/SkTypeface.cpp:235-238
    /// Safety: same native initialized-output/full-stream contract as serialize.
    pub unsafe fn serializeData(
        &self,
        behavior: SerializeBehavior,
    ) -> Option<Arc<crate::src::core::SkData::SkData>> {
        let mut stream = core::pin::pin!(crate::src::core::SkStream::SkDynamicMemoryWStream::new());
        if unsafe { self.serialize(stream.as_mut(), behavior) } {
            Some(stream.as_mut().detachAsData())
        } else {
            None
        }
    }
    // Source: include/core/SkTypeface.h:136-136
    pub unsafe fn serialize_default(
        &self,
        stream: core::pin::Pin<&mut dyn crate::src::core::SkStream::SkWStream>,
    ) -> bool {
        unsafe { self.serialize(stream, SerializeBehavior::kIncludeDataIfLocal) }
    }
    // Source: include/core/SkTypeface.h:142-142
    pub unsafe fn serializeData_default(&self) -> Option<Arc<crate::src::core::SkData::SkData>> {
        unsafe { self.serializeData(SerializeBehavior::kIncludeDataIfLocal) }
    }
}

// Source: src/core/SkTypeface.cpp:286-288
impl SkTypeface {
    pub fn glyphMaskNeedsCurrentColor(&self) -> bool {
        self.virtuals.onGlyphMaskNeedsCurrentColor()
    }
}

impl SkTypeface {
    // Source: src/core/SkTypeface.cpp:290-294
    pub fn getVariationDesignPosition(&self, coordinates: &mut [Coordinate]) -> i32 {
        unsafe {
            self.virtuals
                .onGetVariationDesignPosition(coordinates.as_mut_ptr(), coordinates.len())
        }
    }
    // Source: src/core/SkTypeface.cpp:290-294
    /// Safety: coordinates covers capacity writable entries. Entries may be
    /// uninitialized, as in native font-descriptor array allocation. Only fields
    /// successfully produced by the concrete backend may subsequently be read.
    pub unsafe fn getVariationDesignPosition_raw(
        &self,
        coordinates: *mut Coordinate,
        capacity: usize,
    ) -> i32 {
        unsafe {
            self.virtuals
                .onGetVariationDesignPosition(coordinates, capacity)
        }
    }
    // Source: src/core/SkTypeface.cpp:296-300
    pub fn getVariationDesignParameters(&self, parameters: &mut [Axis]) -> i32 {
        self.virtuals.onGetVariationDesignParameters(parameters)
    }
}

// Source: src/core/SkTypeface.cpp:302-304
// Source: src/core/SkTypeface.cpp:306-308
// Source: src/core/SkTypeface.cpp:310-312
// Source: src/core/SkTypeface.cpp:314-317
impl SkTypeface {
    pub fn countTables(&self) -> i32 {
        self.virtuals.onGetTableTags(&mut [])
    }
    pub fn readTableTags(&self, tags: &mut [SkFontTableTag]) -> i32 {
        self.virtuals.onGetTableTags(tags)
    }
    pub fn getTableSize(&self, tag: SkFontTableTag) -> usize {
        self.virtuals
            .onGetTableData(tag, 0, u32::MAX as usize, None)
    }
    pub fn getTableData(
        &self,
        tag: SkFontTableTag,
        offset: usize,
        length: usize,
        data: Option<&mut [u8]>,
    ) -> usize {
        self.virtuals
            .onGetTableData(tag, offset, length, data.map(|v| &mut v[..length]))
    }
}

impl SkTypeface {
    // Source: src/core/SkTypeface.cpp:319-321
    pub fn copyTableData(
        &self,
        tag: SkFontTableTag,
    ) -> Option<Arc<crate::src::core::SkData::SkData>> {
        self.virtuals.onCopyTableData(self, tag)
    }
    // Source: src/core/SkTypeface.cpp:323-331
    fn defaultCopyTableData(
        &self,
        tag: SkFontTableTag,
    ) -> Option<Arc<crate::src::core::SkData::SkData>> {
        let size = self.getTableSize(tag);
        if size != 0 {
            // Rust slice writes require initialized u8. Zero initialization adapts
            // only native unspecified bytes if a backend returns a partial read.
            let data = crate::src::core::SkData::SkData::MakeZeroInitialized(size);
            let bytes = unsafe { std::slice::from_raw_parts_mut(data.writable_data(), size) };
            let _ = self.getTableData(tag, 0, size, Some(bytes));
            Some(data)
        } else {
            None
        }
    }
    // Source: src/core/SkTypeface.cpp:333-340
    pub fn openStream(
        &self,
        ttc_index: Option<&mut i32>,
    ) -> Option<core::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>> {
        // Rust initializes the unused scratch output before forming &mut i32.
        let mut ttc_index_storage = 0;
        let ttc_index = ttc_index.unwrap_or(&mut ttc_index_storage);
        self.virtuals.onOpenStream(self, ttc_index)
    }
    // Source: src/core/SkTypeface.cpp:342-349
    pub fn openExistingStream(
        &self,
        ttc_index: Option<&mut i32>,
    ) -> Option<core::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>> {
        let mut ttc_index_storage = 0;
        let ttc_index = ttc_index.unwrap_or(&mut ttc_index_storage);
        self.virtuals.onOpenExistingStream(self, ttc_index)
    }
    // Source: src/core/SkTypeface.cpp:351-356
    /// Safety: retain the concrete typeface and all backend leases while the scaler is used.
    pub unsafe fn createScalerContext(
        self: &SkTypefaceRef,
        effects: &crate::src::core::SkScalerContext::SkScalerContextEffects<'_>,
        desc: &crate::src::core::SkDescriptor::SkDescriptor,
        path_owner: &crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkPathEffect::SkPathEffect,
        >,
        mask_owner: Option<&Arc<dyn crate::src::core::SkMaskFilter::SkMaskFilter>>,
    ) -> Box<dyn crate::src::core::SkScalerContext::SkScalerContextVirtual> {
        unsafe {
            self.virtuals.onCreateScalerContext(
                core::ptr::NonNull::from(&**self),
                effects,
                desc,
                path_owner,
                mask_owner,
            )
        }
    }
}
// Source: src/core/SkTypeface.cpp:365-369
// Source: src/core/SkTypeface.cpp:371-375
impl SkTypeface {
    pub fn unicharsToGlyphs(&self, uni: &[i32], glyphs: &mut [u16]) {
        let n = uni.len().min(glyphs.len());
        if n != 0 {
            self.virtuals.onCharsToGlyphs(&uni[..n], &mut glyphs[..n]);
        }
    }
    pub fn unicharToGlyph(&self, uni: i32) -> u16 {
        let mut glyphs = [0];
        self.virtuals.onCharsToGlyphs(&[uni], &mut glyphs);
        glyphs[0]
    }
}

// The native inline-or-heap scratch buffer becomes owned Rust storage. Its unwritten
// tail is zero-initialized; only the counted prefix is passed to the virtual call.
// Source: src/core/SkTypeface.cpp:378-380
struct SkConvertToUTF32 {
    storage: Utf32Storage,
}

impl SkConvertToUTF32 {
    // Source: src/core/SkTypeface.cpp:382-408
    unsafe fn convert(
        &mut self,
        text: *const std::ffi::c_void,
        byte_length: usize,
        encoding: crate::include::core::SkFontTypes::SkTextEncoding,
    ) -> *const i32 {
        use crate::include::core::SkFontTypes::SkTextEncoding;
        use crate::src::core::SkUTF;
        match encoding {
            SkTextEncoding::kUTF8 => {
                let storage = self.storage.reset(byte_length);
                let mut ptr = text.cast::<u8>();
                let end = ptr.add(byte_length);
                let mut i = 0;
                while ptr < end {
                    storage[i] = SkUTF::NextUTF8(&mut ptr, end);
                    i += 1
                }
                storage.as_ptr()
            }
            SkTextEncoding::kUTF16 => {
                let storage = self.storage.reset(byte_length);
                let mut ptr = text.cast::<u16>();
                let end = ptr.add(byte_length >> 1);
                let mut i = 0;
                while ptr < end {
                    storage[i] = SkUTF::NextUTF16(&mut ptr, end);
                    i += 1
                }
                storage.as_ptr()
            }
            SkTextEncoding::kUTF32 => text.cast(),
            SkTextEncoding::kGlyphID => {
                crate::SK_ABORT!(@source b"src/core/SkTypeface.cpp\0".as_ptr(),405; "unexpected enum")
            }
            _ => {
                crate::SK_ABORT!(@source b"src/core/SkTypeface.cpp\0".as_ptr(),405; "unexpected enum")
            }
        }
    }
}

// Source: src/core/SkTypeface.cpp:410-412
enum Utf32Storage {
    Inline([i32; 256]),
    Heap(Vec<i32>),
}

impl SkTypeface {
    // Source: src/core/SkTypeface.cpp:415-419
    // Source: src/core/SkTypeface.cpp:421-421
    // Source: src/core/SkTypeface.cpp:423-426
    // Source: src/core/SkTypeface.cpp:428-431
    // Source: src/core/SkTypeface.cpp:433-434
    // Source: src/core/SkTypeface.cpp:436-438
    pub unsafe fn textToGlyphs(
        &self,
        text: *const std::ffi::c_void,
        byte_length: usize,
        encoding: crate::include::core::SkFontTypes::SkTextEncoding,
        glyphs: &mut [u16],
    ) -> usize {
        if byte_length == 0 {
            return 0;
        }
        debug_assert!(!text.is_null());
        let count = crate::src::core::SkFontPriv::SkFontPriv::CountTextElements(
            text,
            byte_length,
            encoding,
        );
        if count > glyphs.len() {
            return count;
        }
        if encoding == crate::include::core::SkFontTypes::SkTextEncoding::kGlyphID {
            std::ptr::copy_nonoverlapping(
                text.cast::<u8>(),
                glyphs.as_mut_ptr().cast::<u8>(),
                count << 1,
            );
            return count;
        }
        let mut storage = SkConvertToUTF32 {
            storage: Utf32Storage::Inline([0; 256]),
        };
        let uni = storage.convert(text, byte_length, encoding);
        self.unicharsToGlyphs(std::slice::from_raw_parts(uni, count), glyphs);
        count
    }
}

// Source: src/core/SkTypeface.cpp:440-442
// Source: src/core/SkTypeface.cpp:444-444
// Source: src/core/SkTypeface.cpp:446-447
// Source: src/core/SkTypeface.cpp:449-450
// Source: src/core/SkTypeface.cpp:454-456
// Source: src/core/SkTypeface.cpp:458-460
impl SkTypeface {
    pub fn countGlyphs(&self) -> i32 {
        self.virtuals.onCountGlyphs()
    }
    pub fn getUnitsPerEm(&self) -> i32 {
        self.virtuals.onGetUPEM()
    }
    pub fn getKerningPairAdjustments(&self, glyphs: &[u16], adjustments: &mut [i32]) -> bool {
        if glyphs.len() <= 1 || adjustments.is_empty() {
            return self.virtuals.onGetKerningPairAdjustments(&[], &mut []);
        }
        let n = (glyphs.len() - 1).min(adjustments.len());
        self.virtuals
            .onGetKerningPairAdjustments(&glyphs[..n + 1], &mut adjustments[..n])
    }
}

// Source: src/core/SkTypeface.cpp:462-464
impl SkTypeface {
    pub fn createFamilyNameIterator(&self) -> Option<Box<dyn LocalizedStrings>> {
        self.virtuals.onCreateFamilyNameIterator(self)
    }
}

// Source: src/core/SkTypeface.cpp:466-469
// Source: src/core/SkTypeface.cpp:471-473
// Source: src/core/SkTypeface.cpp:475-477
impl SkTypeface {
    pub fn getFamilyName(&self, name: &mut String) {
        self.virtuals.onGetFamilyName(name);
    }
    pub fn getPostScriptName(&self, name: &mut String) -> bool {
        self.virtuals.onGetPostScriptName(name)
    }
    pub fn getResourceName(&self, name: &mut String) -> i32 {
        self.virtuals.onGetResourceName(name)
    }
}

// Source: src/core/SkTypeface.cpp:479-481
fn default_resource_name(_name: &mut String) -> i32 {
    0
}

// Source: src/core/SkTypeface.cpp:483-485
impl SkTypeface {
    pub fn fontStyle(&self) -> SkFontStyle {
        self.virtuals.onGetFontStyle(self.fStyle)
    }
}

// Source: src/core/SkTypeface.cpp:487-489
fn default_font_style(stored: SkFontStyle) -> SkFontStyle {
    stored
}

// Source: src/core/SkTypeface.cpp:491-493
// Source: src/core/SkTypeface.cpp:495-497
// Source: src/core/SkTypeface.cpp:499-501
impl SkTypeface {
    pub fn isBold(&self) -> bool {
        self.virtuals.onGetFontStyle(self.fStyle).weight() >= SkFontStyle::kSemiBold_Weight
    }
    pub fn isItalic(&self) -> bool {
        self.virtuals.onGetFontStyle(self.fStyle).slant() != Slant::kUpright_Slant
    }
    pub fn isFixedPitch(&self) -> bool {
        self.virtuals.onGetFixedPitch(self.fIsFixedPitch)
    }
}

// Source: src/core/SkTypeface.cpp:503-505
fn default_fixed_pitch(stored: bool) -> bool {
    stored
}

// Source: src/core/SkTypeface.cpp:507-508
impl SkTypeface {
    pub fn isSyntheticBold(&self) -> bool {
        self.virtuals.onIsSyntheticBold()
    }
    pub fn isSyntheticOblique(&self) -> bool {
        self.virtuals.onIsSyntheticOblique()
    }
}

// Source: src/core/SkTypeface.cpp:509-510
fn default_synthetic_bold() -> bool {
    false
}

// Source: src/core/SkTypeface.cpp:512-514
impl SkTypeface {
    pub(crate) fn baseGlyphToUnicodeMap(dst: &mut [i32]) {
        dst.fill(0);
    }
}

// Source: src/core/SkTypeface.cpp:516-538
impl SkTypeface {
    pub fn getAdvancedMetrics(
        &self,
    ) -> Option<Box<super::SkAdvancedTypefaceMetrics::SkAdvancedTypefaceMetrics>> {
        use super::SkAdvancedTypefaceMetrics::{FontFlags, FontType};
        use crate::src::sfnt::{
            SkOTTable_OS_2::SkOTTableOS2,
            SkOTTable_OS_2_V2::{SkOTTableOS2_V2, SkOTTableOS2_V2_Type_Field},
        };
        let mut result = self.virtuals.onGetAdvancedMetrics();
        if let Some(metrics) = result.as_mut() {
            if metrics.fPostScriptName.isEmpty() {
                // Existing typeface name protocol uses String; project bytes into the actual
                // retained SkString field after the same native fallback dispatch.
                let mut name = String::new();
                if !self.getPostScriptName(&mut name) {
                    self.getFamilyName(&mut name);
                }
                metrics.fPostScriptName.set_view(name.as_bytes());
            }
            if metrics.fType == FontType::kTrueType_Font || metrics.fType == FontType::kCFF_Font {
                let os2Tag = u32::from_be(SkOTTableOS2::TAG);
                let fsTypeOffset = core::mem::offset_of!(SkOTTableOS2_V2, fsType);
                let mut fsType = core::mem::MaybeUninit::<SkOTTableOS2_V2_Type_Field>::uninit();
                // Native getTableData exposes only the fully read field. The current byte
                // slice projection requires initialized storage before a potentially short read.
                let mut bytes = [0u8; core::mem::size_of::<SkOTTableOS2_V2_Type_Field>()];
                if self.getTableData(os2Tag, fsTypeOffset, bytes.len(), Some(&mut bytes))
                    == bytes.len()
                {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            bytes.as_ptr(),
                            fsType.as_mut_ptr().cast(),
                            bytes.len(),
                        );
                    }
                    let fsType = unsafe { fsType.assume_init() };
                    if fsType.Bitmap()
                        || (fsType.Restricted() && !(fsType.PreviewPrint() || fsType.Editable()))
                    {
                        metrics.fFlags |= FontFlags::kNotEmbeddable_FontFlag;
                    }
                    if fsType.NoSubsetting() {
                        metrics.fFlags |= FontFlags::kNotSubsettable_FontFlag;
                    }
                }
            }
        }
        result
    }
}

// Source: src/core/SkTypeface.cpp:540-542
fn default_kerning(_glyphs: &[u16], _adjustments: &mut [i32]) -> bool {
    false
}

// Source: src/core/SkTypeface.cpp:544-546
fn default_open_existing_stream<T: SkTypefaceVirtuals + ?Sized>(
    typeface: &T,
    owner: &SkTypeface,
    ttc_index: &mut i32,
) -> Option<core::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>> {
    typeface.onOpenStream(owner, ttc_index)
}

impl SkTypeface {
    // Source: src/core/SkTypeface.cpp:550-557
    pub fn getBounds(self: &SkTypefaceRef) -> SkRect {
        unsafe {
            crate::include::private::SkOnce::SkOnce::call_once(&self.fBoundsOnce, || {
                let bounds = &mut *self.fBounds.get();
                if !self.virtuals.onComputeBounds(self, bounds) {
                    bounds.setEmpty();
                }
            });
            *self.fBounds.get()
        }
    }
    // Source: src/core/SkTypeface.cpp:559-589
    fn defaultComputeBounds(self: &SkTypefaceRef, bounds: &mut SkRect) -> bool {
        use crate::src::core::{
            SkDescriptor::{SkAutoDescriptor, SkDescriptor},
            SkScalerContext::{self, SkScalerContextEffects, SkScalerContextRec},
        };
        let text_size = 2048.0;
        let inv_text_size = 1.0 / text_size;
        let mut font = crate::src::core::SkFont::SkFont::default();
        font.setTypeface(Some(self.clone()));
        font.setSize(text_size);
        font.setLinearMetrics(true);
        // This exact source call creates a default Paint without Shader/ColorFilter.
        // Its luminance evaluation is infallible; an invariant violation is surfaced,
        // never substituted with empty metrics or an unavailable-backend fallback.
        let (rec, _effects) = SkScalerContext::MakeRecAndEffectsFromFont(&font)
            .expect("native default Paint has no fallible color-filter producer");
        let mut ad = SkAutoDescriptor::default();
        let no_effects = SkScalerContextEffects::default();
        // Source explicitly selects noeffects. Its size/empty-buffer descriptor is
        // exactly calculate_size_and_flatten's no-effect case + generate_descriptor.
        ad.reset(std::mem::size_of::<SkScalerContextRec>() + SkDescriptor::compute_overhead(1));
        SkScalerContext::DescriptorBufferGiveRec(&rec, ad.get_desc_mut());
        let path_owner = crate::include::core::SkRefCnt::sk_sp::default();
        let mut ctx =
            unsafe { self.createScalerContext(&no_effects, ad.get_desc(), &path_owner, None) };
        // Native generateFontMetrics must initialize every field. Avoid constructing
        // an initialized Rust reference to uninitialized storage: backend receives
        // initialized temporary fields, whose final values remain backend-owned.
        let mut metrics =
            unsafe { std::mem::zeroed::<crate::include::core::SkFontMetrics::SkFontMetrics>() };
        unsafe {
            ctx.getFontMetrics(&mut metrics);
        }
        let fm = metrics;
        if !fm.hasBounds() {
            return false;
        }
        bounds.setLTRB(
            fm.fXMin * inv_text_size,
            fm.fTop * inv_text_size,
            fm.fXMax * inv_text_size,
            fm.fBottom * inv_text_size,
        );
        true
    }
}

// Source: src/core/SkTypeface.cpp:162-185
// Source: include/core/SkTypeface.h:357-357
pub type FactoryId = u32;
pub type TypefaceDecoder = unsafe fn(
    core::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>,
    &SkFontArguments,
) -> Option<SkTypefaceRef>;
#[derive(Clone, Copy)]
struct DecoderProc {
    id: u32,
    makeFromStream: TypefaceDecoder,
}
struct DecoderRegistry(core::cell::UnsafeCell<Vec<DecoderProc>>);
// Native registration is an externally serialized startup operation. The unsafe
// Register contract excludes registration concurrent with any decoder access.
unsafe impl Sync for DecoderRegistry {}
unsafe fn empty_decoder(
    stream: core::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>,
    args: &SkFontArguments,
) -> Option<SkTypefaceRef> {
    SkEmptyTypeface::MakeFromStream(stream, args)
}
unsafe fn custom_decoder(
    stream: core::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>,
    args: &SkFontArguments,
) -> Option<SkTypefaceRef> {
    crate::src::utils::SkCustomTypeface::SkCustomTypefaceBuilder::MakeFromStream(stream, args)
}
#[cfg(SK_TYPEFACE_FACTORY_CORETEXT)]
unsafe fn coretext_decoder(
    stream: core::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>,
    args: &SkFontArguments,
) -> Option<SkTypefaceRef> {
    unsafe {
        crate::src::ports::SkTypeface_mac_ct::SkTypeface_Mac::MakeFromStream(
            crate::src::core::SkStream::TransferableStreamAsset::from_native_transferable(stream),
            args,
        )
    }
}
fn decoders() -> &'static DecoderRegistry {
    static DECODERS: OnceLock<DecoderRegistry> = OnceLock::new();
    DECODERS.get_or_init(||DecoderRegistry(core::cell::UnsafeCell::new(vec![
  DecoderProc{id:SkEmptyTypeface::FactoryId,makeFromStream:empty_decoder},
  DecoderProc{id:crate::src::utils::SkCustomTypeface::SkCustomTypefaceBuilder::FactoryId,makeFromStream:custom_decoder},
  #[cfg(SK_TYPEFACE_FACTORY_CORETEXT)]
  DecoderProc{id:crate::src::ports::SkTypeface_mac_ct::SkTypeface_Mac::FactoryId,makeFromStream:coretext_decoder},
  #[cfg(SK_TYPEFACE_FACTORY_DIRECTWRITE)]
  DecoderProc{id:crate::src::ports::SkTypeface_win_dw::DWriteFontTypeface::FactoryId,makeFromStream:crate::src::ports::SkTypeface_win_dw::DWriteFontTypeface::MakeFromStream},
  #[cfg(SK_TYPEFACE_FACTORY_FREETYPE)]
  DecoderProc{id:crate::src::ports::SkTypeface_FreeType::SkTypeface_FreeType::FactoryId,makeFromStream:crate::src::ports::SkTypeface_FreeType::SkTypeface_FreeType::MakeFromStream},
  #[cfg(SK_TYPEFACE_FACTORY_FONTATIONS)]
  DecoderProc{id:crate::include::ports::SkTypeface_fontations::SkTypefaces::Fontations::FactoryId,makeFromStream:crate::src::ports::SkTypeface_fontations_factory::SkTypeface_Make_Fontations},
 ])))
}
impl SkTypeface {
    // Source: src/core/SkTypeface.cpp:195-199
    /// Safety: registration is externally serialized and precedes concurrent use.
    /// Factory preserves the complete native transferable stream/backing contract.
    pub unsafe fn Register(id: u32, make: TypefaceDecoder) {
        unsafe { &mut *decoders().0.get() }.push(DecoderProc {
            id,
            makeFromStream: make,
        });
    }
    // Source: src/core/SkTypeface.cpp:240-282
    pub fn MakeDeserialize(
        stream: core::pin::Pin<&mut dyn crate::src::core::SkStream::SkStream>,
        lastResortMgr: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkFontMgr::SkFontMgr,
        >,
        sanitizer: Option<SkTypefaceStreamSanitizerProc>,
    ) -> Option<SkTypefaceRef> {
        let mut desc = crate::src::core::SkFontDescriptor::SkFontDescriptor::new();
        if !crate::src::core::SkFontDescriptor::SkFontDescriptor::Deserialize(
            stream, &mut desc, sanitizer,
        ) {
            return None;
        }
        if desc.hasStream() {
            for proc in unsafe { &*decoders().0.get() } {
                if proc.id == desc.getFactoryId() {
                    let args = desc.getFontArguments();
                    return unsafe {
                        (proc.makeFromStream)(
                            desc.detachStream().expect("native descriptor owns stream"),
                            &args,
                        )
                    };
                }
            }
            #[cfg(sk_debug)]
            {
                use std::io::Write;
                let id = desc.getFactoryId();
                let mut output = std::io::stderr();
                let _ = output.write_all(b"Could not find factory ");
                let _ = output.write_all(&[
                    (id >> 24) as u8,
                    (id >> 16) as u8,
                    (id >> 8) as u8,
                    id as u8,
                ]);
                let _ = output.write_all(b" for ");
                let name = unsafe { core::ffi::CStr::from_ptr(desc.getFamilyName()) };
                let _ = output.write_all(name.to_bytes());
                let _ = output.write_all(b".\n");
            }
            if let Some(manager) = lastResortMgr.get() {
                let args = desc.getFontArguments();
                if let Some(face) = unsafe { manager.makeFromStream(desc.detachStream(), &args) } {
                    return Some(face);
                }
            }
        }
        if let Some(manager) = lastResortMgr.get() {
            return unsafe { manager.legacyMakeTypeface(desc.getFamilyName(), desc.getStyle()) };
        }
        Some(SkEmptyTypeface::Make())
    }
}
