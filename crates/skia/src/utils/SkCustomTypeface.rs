use crate::include::core::{
    SkFontArguments::{Coordinate, SkFontArguments},
    SkFontMetrics::SkFontMetrics,
    SkFontParameters::Axis,
    SkFontStyle::SkFontStyle,
    SkRefCnt::{sk_sp, SkRefCnt, SkRefCntBase, SkRefCntObject},
};
use crate::src::core::{
    SkCanvas::SkCanvas,
    SkDescriptor::SkDescriptor,
    SkDrawable::{SkDrawable, SkDrawableState},
    SkGlyph::SkGlyph,
    SkMatrix::SkMatrix,
    SkPath::SkPath,
    SkPoint::SkPoint,
    SkRect::SkRect,
    SkScalerContext::{
        GeneratedPath, GlyphMetrics, SkScalerContext, SkScalerContextEffects, SkScalerContextRec,
        SkScalerContextVirtual,
    },
    SkStream::{SkDynamicMemoryWStream, SkStream, SkStreamAsset, SkWStream},
    SkTypeface::{LocalizedStrings, SkFontTableTag, SkTypeface, SkTypefaceRef, SkTypefaceVirtuals},
};
use core::{
    mem::{size_of, ManuallyDrop, MaybeUninit},
    ptr::NonNull,
};
use std::sync::Arc;
// Source: include/utils/SkCustomTypeface.h:43-55
struct GlyphRec {
    fPath: ManuallyDrop<SkPath>,
    fDrawable: ManuallyDrop<sk_sp<dyn SkDrawable>>,
    fBounds: SkRect,
    fAdvance: f32,
}
// Native implicit member copy operations; no additional source-line credit.
impl Clone for GlyphRec {
    fn clone(&self) -> Self {
        Self {
            fPath: ManuallyDrop::new((*self.fPath).clone()),
            fDrawable: ManuallyDrop::new((*self.fDrawable).clone()),
            fBounds: self.fBounds,
            fAdvance: self.fAdvance,
        }
    }
    fn clone_from(&mut self, other: &Self) {
        *self.fPath = (*other.fPath).clone();
        *self.fDrawable = (*other.fDrawable).clone();
        self.fBounds = other.fBounds;
        self.fAdvance = other.fAdvance;
    }
}
impl Default for GlyphRec {
    fn default() -> Self {
        Self {
            fPath: ManuallyDrop::new(SkPath::default()),
            fDrawable: ManuallyDrop::new(sk_sp::default()),
            fBounds: SkRect::MakeEmpty(),
            fAdvance: 0.0,
        }
    }
}
impl Drop for GlyphRec {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fDrawable);
            ManuallyDrop::drop(&mut self.fPath);
        }
    }
}
impl GlyphRec {
    fn isDrawable(&self) -> bool {
        debug_assert!(self.fDrawable.is_none() || self.fPath.isEmpty());
        self.fDrawable.is_some()
    }
}
// Source: include/utils/SkCustomTypeface.h:28-28
// Source: include/utils/SkCustomTypeface.h:57-59
pub struct SkCustomTypefaceBuilder {
    fGlyphRecs: Vec<GlyphRec>,
    fMetrics: SkFontMetrics,
    fStyle: SkFontStyle,
}
// Native implicit builder copy operations preserve field declaration order.
impl Clone for SkCustomTypefaceBuilder {
    fn clone(&self) -> Self {
        Self {
            fGlyphRecs: self.fGlyphRecs.clone(),
            fMetrics: self.fMetrics,
            fStyle: self.fStyle,
        }
    }
    fn clone_from(&mut self, other: &Self) {
        self.fGlyphRecs.clone_from(&other.fGlyphRecs);
        self.fMetrics = other.fMetrics;
        self.fStyle = other.fStyle;
    }
}
// Source: src/utils/SkCustomTypeface.cpp:55-57
const kSkShowTextBlitCoverage: bool = false;
// Source: src/utils/SkCustomTypeface.cpp:59-86
fn scale_fontmetrics(src: &SkFontMetrics, sx: f32, sy: f32) -> SkFontMetrics {
    let mut dst = *src;
    dst.fAvgCharWidth *= sx;
    dst.fMaxCharWidth *= sx;
    dst.fXMin *= sx;
    dst.fXMax *= sx;
    dst.fTop *= sy;
    dst.fAscent *= sy;
    dst.fDescent *= sy;
    dst.fBottom *= sy;
    dst.fLeading *= sy;
    dst.fXHeight *= sy;
    dst.fCapHeight *= sy;
    dst.fUnderlineThickness *= sy;
    dst.fUnderlinePosition *= sy;
    dst.fStrikeoutThickness *= sy;
    dst.fStrikeoutPosition *= sy;
    dst
}
// Source: src/utils/SkCustomTypeface.cpp:91-108
struct SkUserTypeface {
    fGlyphRecs: Vec<GlyphRec>,
    fMetrics: SkFontMetrics,
}
// Native SkTypeface publishes immutable glyph records. The unsafe drawable
// builder overload requires the same transferable shared graph as native fonts.
unsafe impl Send for SkUserTypeface {}
unsafe impl Sync for SkUserTypeface {}
impl SkUserTypeface {
    fn new(style: SkFontStyle, metrics: &SkFontMetrics, recs: Vec<GlyphRec>) -> SkTypefaceRef {
        sk_sp::new(SkTypeface::new(
            style,
            false,
            Box::new(Self {
                fGlyphRecs: recs,
                fMetrics: *metrics,
            }),
        ))
    }
}
impl SkUserTypeface {
    // Source: src/utils/SkCustomTypeface.cpp:121-121
    fn onOpenExistingStream(
        &self,
        _: &SkTypeface,
        _: &mut i32,
    ) -> Option<core::pin::Pin<Box<dyn SkStreamAsset>>> {
        None
    }
    // Source: src/utils/SkCustomTypeface.cpp:123-125
    unsafe fn onMakeClone(
        &self,
        owner: &SkTypefaceRef,
        _: &SkFontArguments,
    ) -> Option<SkTypefaceRef> {
        Some(owner.clone())
    }
    // Source: src/utils/SkCustomTypeface.cpp:126-126
    fn onCountGlyphs(&self) -> i32 {
        self.glyphCount()
    }
    // Source: src/utils/SkCustomTypeface.cpp:127-127
    fn onGetUPEM(&self) -> i32 {
        2048
    }
    // Source: src/utils/SkCustomTypeface.cpp:128-131
    fn onComputeBounds(&self, _: &SkTypefaceRef, bounds: &mut SkRect) -> bool {
        *bounds = SkRect::MakeLTRB(
            self.fMetrics.fXMin,
            self.fMetrics.fTop,
            self.fMetrics.fXMax,
            self.fMetrics.fBottom,
        );
        true
    }
    // Source: src/utils/SkCustomTypeface.cpp:135-135
    fn getPostScriptGlyphNames(&self, _: &mut [crate::src::core::SkString::SkString]) {}
    // Source: src/utils/SkCustomTypeface.cpp:136-136
    fn onGlyphMaskNeedsCurrentColor(&self) -> bool {
        false
    }
    // Source: src/utils/SkCustomTypeface.cpp:137-138
    unsafe fn onGetVariationDesignPosition(&self, _: *mut Coordinate, _: usize) -> i32 {
        0
    }
    // Source: src/utils/SkCustomTypeface.cpp:139-141
    fn onGetVariationDesignParameters(&self, _: &mut [Axis]) -> i32 {
        0
    }
    // Source: src/utils/SkCustomTypeface.cpp:142-142
    fn onGetTableTags(&self, _: &mut [SkFontTableTag]) -> i32 {
        0
    }
    // Source: src/utils/SkCustomTypeface.cpp:143-143
    fn onGetTableData(&self, _: SkFontTableTag, _: usize, _: usize, _: Option<&mut [u8]>) -> usize {
        0
    }
}
impl SkUserTypeface {
    // Source: src/utils/SkCustomTypeface.cpp:157-159
    fn glyphCount(&self) -> i32 {
        self.fGlyphRecs.len() as i32
    }
}
impl SkCustomTypefaceBuilder {
    // Source: include/utils/SkCustomTypeface.h:40-40
    pub const FactoryId: u32 = u32::from_be_bytes(*b"user");
    // Source: src/utils/SkCustomTypeface.cpp:150-152
    pub fn new() -> Self {
        Self {
            fGlyphRecs: Vec::new(),
            fMetrics: unsafe { core::mem::zeroed() },
            fStyle: SkFontStyle::default(),
        }
    }
    // Source: src/utils/SkCustomTypeface.cpp:154-156
    pub fn setMetrics(&mut self, fm: &SkFontMetrics, scale: f32) {
        self.fMetrics = scale_fontmetrics(fm, scale, scale);
    }
    // Source: src/utils/SkCustomTypeface.cpp:158-160
    pub fn setFontStyle(&mut self, style: SkFontStyle) {
        self.fStyle = style;
    }
    // Source: src/utils/SkCustomTypeface.cpp:162-168
    fn ensureStorage(&mut self, index: u16) -> &mut GlyphRec {
        if index as usize >= self.fGlyphRecs.len() {
            self.fGlyphRecs
                .resize_with(index as usize + 1, GlyphRec::default);
        }
        &mut self.fGlyphRecs[index as usize]
    }
    // Source: src/utils/SkCustomTypeface.cpp:170-175
    pub fn setGlyph(&mut self, index: u16, advance: f32, path: &SkPath) {
        let rec = self.ensureStorage(index);
        rec.fAdvance = advance;
        *rec.fPath = path.clone();
        *rec.fDrawable = sk_sp::default();
    }
    // Source: src/utils/SkCustomTypeface.cpp:177-184
    /// Safety: drawable and its full retained graph support concurrent access and
    /// destruction on every thread on which the resulting native typeface is used.
    pub unsafe fn setGlyphDrawable(
        &mut self,
        index: u16,
        advance: f32,
        drawable: sk_sp<dyn SkDrawable>,
        bounds: &SkRect,
    ) {
        let rec = self.ensureStorage(index);
        rec.fAdvance = advance;
        *rec.fDrawable = drawable;
        rec.fBounds = *bounds;
        *rec.fPath = SkPath::default();
    }
    // Source: src/utils/SkCustomTypeface.cpp:186-204
    pub fn detach(&mut self) -> Option<SkTypefaceRef> {
        if self.fGlyphRecs.is_empty() {
            return None;
        }
        let mut bounds = SkRect::MakeLTRB(f32::MAX, f32::MAX, -f32::MAX, -f32::MAX);
        for rec in &self.fGlyphRecs {
            bounds.join(if rec.isDrawable() {
                &rec.fBounds
            } else {
                rec.fPath.getBounds()
            });
        }
        self.fMetrics.fTop = bounds.fTop;
        self.fMetrics.fBottom = bounds.fBottom;
        self.fMetrics.fXMin = bounds.fLeft;
        self.fMetrics.fXMax = bounds.fRight;
        Some(SkUserTypeface::new(
            self.fStyle,
            &self.fMetrics,
            core::mem::take(&mut self.fGlyphRecs),
        ))
    }
}
impl SkUserTypeface {
    // Source: src/utils/SkCustomTypeface.cpp:208-211
    fn onFilterRec(&self, rec: &mut SkScalerContextRec) {
        rec.useStrokeForFakeBold();
        rec.set_hinting(crate::include::core::SkFontTypes::SkFontHinting::kNone);
    }
    // Source: src/utils/SkCustomTypeface.cpp:213-218
    fn getGlyphToUnicodeMap(&self, dst: &mut [i32]) {
        let n = (self.glyphCount() as usize).min(dst.len());
        for gid in 0..n {
            dst[gid] = gid as i32;
        }
    }
    // Source: src/utils/SkCustomTypeface.cpp:220-222
    fn onGetAdvancedMetrics(
        &self,
    ) -> Option<Box<crate::src::core::SkAdvancedTypefaceMetrics::SkAdvancedTypefaceMetrics>> {
        None
    }
    // Source: src/utils/SkCustomTypeface.cpp:224-227
    fn onGetFontDescriptor(
        &self,
        _: &SkTypeface,
        desc: &mut crate::src::core::SkFontDescriptor::SkFontDescriptor,
        isLocal: &mut bool,
    ) {
        desc.setFactoryId(SkCustomTypefaceBuilder::FactoryId);
        *isLocal = true;
    }
    // Source: src/utils/SkCustomTypeface.cpp:229-236
    fn onCharsToGlyphs(&self, chars: &[i32], glyphs: &mut [u16]) {
        crate::SkASSERT!(@source b"src/utils/SkCustomTypeface.cpp\0".as_ptr(),231; @condition b"chars.size() == glyphs.size()\0".as_ptr(); (chars.len())==(glyphs.len()));
        let count = self.glyphCount();
        for i in 0..chars.len() {
            glyphs[i] = if chars[i] < count { chars[i] as u16 } else { 0 };
        }
    }
    // Source: src/utils/SkCustomTypeface.cpp:238-240
    fn onGetFamilyName(&self, name: &mut String) {
        name.clear();
    }
    // Source: src/utils/SkCustomTypeface.cpp:242-244
    fn onGetPostScriptName(&self, _: &mut String) -> bool {
        false
    }
    // Source: src/utils/SkCustomTypeface.cpp:246-248
    fn onCreateFamilyNameIterator(&self, _: &SkTypeface) -> Option<Box<dyn LocalizedStrings>> {
        None
    }
}
// Source: src/utils/SkCustomTypeface.cpp:269-282,399-400
struct SkUserScalerContext {
    fMatrix: SkMatrix,
    inherited: SkScalerContext,
}
impl SkUserScalerContext {
    unsafe fn new(
        face: NonNull<SkTypeface>,
        effects: &SkScalerContextEffects<'_>,
        desc: &SkDescriptor,
        path_owner: &sk_sp<dyn crate::src::core::SkPathEffect::SkPathEffect>,
        mask_owner: Option<&Arc<dyn crate::src::core::SkMaskFilter::SkMaskFilter>>,
    ) -> Self {
        let inherited =
            unsafe { SkScalerContext::new(face, effects, desc, path_owner, mask_owner) };
        let fMatrix = inherited.getRec().get_single_matrix();
        Self { fMatrix, inherited }
    }
    // Source: src/utils/SkCustomTypeface.cpp:280-282
    fn userTF(&self) -> &SkUserTypeface {
        unsafe {
            let face = &*self.inherited.getTypeface();
            &*(face.virtuals.as_ref() as *const dyn SkTypefaceVirtuals as *const SkUserTypeface)
        }
    }
}
impl SkScalerContextVirtual for SkUserScalerContext {
    fn base(&self) -> &SkScalerContext {
        &self.inherited
    }
    fn base_mut(&mut self) -> &mut SkScalerContext {
        &mut self.inherited
    }
    // Source: src/utils/SkCustomTypeface.cpp:266-293
    unsafe fn generateMetrics(
        &mut self,
        glyph: &SkGlyph,
        _: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> GlyphMetrics {
        let mut mx = GlyphMetrics::new(glyph.maskFormat());
        let tf = self.userTF();
        let gid = glyph.getGlyphID() as usize;
        if gid >= tf.fGlyphRecs.len() {
            mx.neverRequestPath = true;
            return mx;
        }
        let rec = &tf.fGlyphRecs[gid];
        mx.advance = self.fMatrix.mapPoint(SkPoint::make(rec.fAdvance, 0.0));
        if rec.isDrawable() {
            mx.maskFormat = crate::src::core::SkMask::Format::kARGB32_Format;
            let mut bounds = self.fMatrix.mapRect_value(&rec.fBounds);
            bounds.offset(
                glyph.getSubXFixed() as f32 / 65536.0,
                glyph.getSubYFixed() as f32 / 65536.0,
            );
            mx.bounds = SkRect::Make(&bounds.roundOut());
            mx.neverRequestPath = true;
        } else {
            mx.computeFromPath = true;
        }
        mx
    }
    // Source: src/utils/SkCustomTypeface.cpp:295-315
    unsafe fn generateImage(&mut self, glyph: &SkGlyph, imageBuffer: *mut core::ffi::c_void) {
        let rec = &self.userTF().fGlyphRecs[glyph.getGlyphID() as usize];
        if !rec.isDrawable() {
            unsafe { self.generateImageFromPath(glyph, imageBuffer) }
                .expect("native void image generation cannot return RasterError");
            return;
        }
        let mut canvas = unsafe {
            SkCanvas::MakeRasterDirectN32(
                glyph.width(),
                glyph.height(),
                imageBuffer.cast(),
                glyph.rowBytes(),
            )
        }
        .expect("native raster-direct canvas");
        let raw = unsafe { NonNull::from(canvas.as_mut().get_unchecked_mut()) };
        unsafe {
            SkCanvas::clear_raw(
                raw,
                if kSkShowTextBlitCoverage {
                    0x33FF0000
                } else {
                    0
                },
            )
        }
        .expect("native void raster clear");
        unsafe {
            canvas
                .as_mut()
                .translate(-glyph.left() as f32, -glyph.top() as f32);
            canvas.as_mut().translate(
                glyph.getSubXFixed() as f32 / 65536.0,
                glyph.getSubYFixed() as f32 / 65536.0,
            );
            SkCanvas::drawDrawable_raw(raw, rec.fDrawable.get(), Some(&self.fMatrix))
        }
        .expect("native void drawable rasterization");
    }
    // Source: src/utils/SkCustomTypeface.cpp:317-323
    unsafe fn generatePath(&mut self, glyph: &SkGlyph) -> Option<GeneratedPath> {
        let rec = &self.userTF().fGlyphRecs[glyph.getGlyphID() as usize];
        crate::SkASSERT!(@source b"src/utils/SkCustomTypeface.cpp\0".as_ptr(),320; @condition b"!rec.isDrawable()\0".as_ptr(); !rec.isDrawable());
        Some(GeneratedPath {
            path: rec.fPath.makeTransform(&self.fMatrix),
            modified: false,
        })
    }
    // Source: src/utils/SkCustomTypeface.cpp:325-360
    unsafe fn generateDrawable(&mut self, glyph: &SkGlyph) -> sk_sp<dyn SkDrawable> {
        // Source: src/utils/SkCustomTypeface.cpp:348-377
        struct DrawableMatrixWrapper {
            fDrawable: ManuallyDrop<sk_sp<dyn SkDrawable>>,
            fMatrix: SkMatrix,
            inherited: SkDrawableState,
            fRefCnt: SkRefCnt,
        }
        impl DrawableMatrixWrapper {
            fn new(drawable: sk_sp<dyn SkDrawable>, m: &SkMatrix) -> Self {
                Self {
                    fDrawable: ManuallyDrop::new(drawable),
                    fMatrix: m.clone(),
                    inherited: SkDrawableState::new(),
                    fRefCnt: SkRefCnt::new(),
                }
            }
        }
        unsafe impl SkRefCntObject for DrawableMatrixWrapper {
            // Native complete-object virtual disposal dispatch; this actual concrete
            // Sized type pairs the original raw allocation with its typed adapter.
            fn internal_dispose_raw_proc(
                &self,
            ) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
                crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
            }
            fn ref_cnt_base(&self) -> &SkRefCntBase {
                self.fRefCnt.ref_cnt_base()
            }
            fn ref_(&self) {
                self.fRefCnt.ref_();
            }
        }
        impl SkDrawable for DrawableMatrixWrapper {
            fn drawable_state(&self) -> &SkDrawableState {
                &self.inherited
            }
            fn onGetBounds(&self) -> SkRect {
                self.fMatrix.mapRect_value(&self.fDrawable.getBounds())
            }
            fn onApproximateBytesUsed(&self) -> usize {
                self.fDrawable
                    .approximateBytesUsed()
                    .wrapping_add(size_of::<Self>())
            }
            unsafe fn onDraw(
                &self,
                canvas: NonNull<SkCanvas<'_>>,
            ) -> Result<(), crate::RasterError> {
                if kSkShowTextBlitCoverage {
                    let mut paint = crate::src::core::SkPaint::SkPaint::default();
                    paint.setColor(0x3300FF00);
                    paint.setStyle(crate::src::core::SkPaint::Style::kFill_Style);
                    unsafe { SkCanvas::drawRect_raw(canvas, &self.onGetBounds(), &paint) }?;
                }
                unsafe {
                    SkCanvas::drawDrawable_raw(canvas, self.fDrawable.get(), Some(&self.fMatrix))
                }
            }
        }
        impl Drop for DrawableMatrixWrapper {
            fn drop(&mut self) {
                unsafe { ManuallyDrop::drop(&mut self.fDrawable) }
            }
        }
        let rec = &self.userTF().fGlyphRecs[glyph.getGlyphID() as usize];
        if rec.fDrawable.is_some() {
            let owner: Box<dyn SkDrawable> = Box::new(DrawableMatrixWrapper::new(
                (*rec.fDrawable).clone(),
                &self.fMatrix,
            ));
            unsafe { sk_sp::from_raw(Box::into_raw(owner)) }
        } else {
            sk_sp::default()
        }
    }
    // Source: src/utils/SkCustomTypeface.cpp:362-365
    unsafe fn generateFontMetrics(&mut self, metrics: &mut SkFontMetrics) {
        let point = self.fMatrix.mapPoint(SkPoint::make(1.0, 1.0));
        *metrics = scale_fontmetrics(&self.userTF().fMetrics, point.fX, point.fY);
    }
}
impl SkUserTypeface {
    // Source: src/utils/SkCustomTypeface.cpp:371-375
    unsafe fn onCreateScalerContext(
        &self,
        typeface: NonNull<SkTypeface>,
        effects: &SkScalerContextEffects<'_>,
        desc: &SkDescriptor,
        path_owner: &sk_sp<dyn crate::src::core::SkPathEffect::SkPathEffect>,
        mask_owner: Option<&Arc<dyn crate::src::core::SkMaskFilter::SkMaskFilter>>,
    ) -> Box<dyn SkScalerContextVirtual> {
        Box::new(unsafe {
            SkUserScalerContext::new(typeface, effects, desc, path_owner, mask_owner)
        })
    }
}
// Source: src/utils/SkCustomTypeface.cpp:410-416
const kMaxGlyphCount: i32 = 65536;
const kHeaderSize: usize = 16;
const gHeaderString: &[u8; 17] = b"SkUserTypeface01\0";
const _: () = assert!(gHeaderString.len() == 1 + kHeaderSize);
#[repr(u32)]
enum GlyphType {
    kPath,
    kDrawable,
}
impl SkUserTypeface {
    // Source: src/utils/SkCustomTypeface.cpp:386-422
    fn onOpenStream(
        &self,
        owner: &SkTypeface,
        ttcIndex: &mut i32,
    ) -> Option<core::pin::Pin<Box<dyn SkStreamAsset>>> {
        let mut stream = core::pin::pin!(SkDynamicMemoryWStream::new());
        unsafe {
            stream.as_mut().write(gHeaderString.as_ptr(), kHeaderSize);
            stream.as_mut().write(
                (&self.fMetrics as *const SkFontMetrics).cast(),
                size_of::<SkFontMetrics>(),
            );
        }
        let style = owner.fontStyle();
        unsafe {
            stream.as_mut().write(
                (&style as *const SkFontStyle).cast(),
                size_of::<SkFontStyle>(),
            );
        }
        stream.as_mut().write32(self.glyphCount() as u32);
        for rec in &self.fGlyphRecs {
            stream.as_mut().write32(GlyphType::kPath as u32);
            stream.as_mut().writeScalar(rec.fAdvance);
            unsafe {
                stream
                    .as_mut()
                    .write((&rec.fBounds as *const SkRect).cast(), size_of::<SkRect>());
            }
            let data = if rec.isDrawable() {
                SkPath::RectDefault(
                    &rec.fBounds,
                    crate::include::core::SkPathTypes::SkPathDirection::kCW,
                    0,
                )
                .serialize()
            } else {
                rec.fPath.serialize()
            };
            let sz = data.size();
            debug_assert_eq!(sz & 3, 0);
            unsafe {
                stream
                    .as_mut()
                    .write((&sz as *const usize).cast(), size_of::<usize>());
                stream.as_mut().write(data.data(), sz);
            }
        }
        *ttcIndex = 0;
        Some(stream.as_mut().detachAsStream())
    }
}
// Source: src/utils/SkCustomTypeface.cpp:462-480
struct AutoRestorePosition<'a> {
    fStream: Option<core::pin::Pin<&'a mut dyn SkStream>>,
    fPosition: usize,
}
impl<'a> AutoRestorePosition<'a> {
    fn new(stream: core::pin::Pin<&'a mut dyn SkStream>) -> Self {
        let fPosition = stream.getPosition();
        Self {
            fStream: Some(stream),
            fPosition,
        }
    }
    fn stream(&mut self) -> core::pin::Pin<&mut dyn SkStream> {
        self.fStream
            .as_mut()
            .expect("active native restore guard")
            .as_mut()
    }
    fn markDone(&mut self) {
        self.fStream = None;
    }
}
impl Drop for AutoRestorePosition<'_> {
    fn drop(&mut self) {
        if let Some(stream) = self.fStream.as_mut() {
            stream.as_mut().seek(self.fPosition);
        }
    }
}
impl SkCustomTypefaceBuilder {
    // Source: src/utils/SkCustomTypeface.cpp:442-517
    fn Deserialize(stream: core::pin::Pin<&mut dyn SkStream>) -> Option<SkTypefaceRef> {
        let mut arp = AutoRestorePosition::new(stream);
        let mut header = [0u8; kHeaderSize];
        if unsafe { arp.stream().read(header.as_mut_ptr(), kHeaderSize) } != kHeaderSize
            || header != gHeaderString[..kHeaderSize]
        {
            return None;
        }
        let mut metrics = MaybeUninit::<SkFontMetrics>::uninit();
        if unsafe {
            arp.stream()
                .read(metrics.as_mut_ptr().cast(), size_of::<SkFontMetrics>())
        } != size_of::<SkFontMetrics>()
        {
            return None;
        }
        let metrics = unsafe { metrics.assume_init() };
        let mut style = MaybeUninit::<SkFontStyle>::uninit();
        if unsafe {
            arp.stream()
                .read(style.as_mut_ptr().cast(), size_of::<SkFontStyle>())
        } != size_of::<SkFontStyle>()
        {
            return None;
        }
        let style = unsafe { style.assume_init() };
        let mut glyphCount = 0;
        if !arp.stream().readS32(&mut glyphCount) || glyphCount < 0 || glyphCount > kMaxGlyphCount {
            return None;
        }
        let mut builder = Self::new();
        builder.setMetrics(&metrics, 1.0);
        builder.setFontStyle(style);
        for i in 0..glyphCount {
            let mut gtype = 0;
            if !arp.stream().readU32(&mut gtype) || gtype != GlyphType::kPath as u32 {
                return None;
            }
            let mut advance = 0.0;
            if !arp.stream().readScalar(&mut advance) {
                return None;
            }
            let mut bounds = MaybeUninit::<SkRect>::uninit();
            if unsafe {
                arp.stream()
                    .read(bounds.as_mut_ptr().cast(), size_of::<SkRect>())
            } != size_of::<SkRect>()
            {
                return None;
            }
            let bounds = unsafe { bounds.assume_init() };
            if !bounds.isFinite() {
                return None;
            }
            let mut sz = 0usize;
            if unsafe {
                arp.stream()
                    .read((&mut sz as *mut usize).cast(), size_of::<usize>())
            } != size_of::<usize>()
            {
                return None;
            }
            if crate::src::core::SkStreamPriv::RemainingLengthIsBelow(&*arp.stream(), sz) {
                return None;
            }
            let data = unsafe { crate::src::core::SkData::SkData::MakeUninitialized(sz) };
            if unsafe { arp.stream().read(data.writable_data(), sz) } != sz {
                return None;
            }
            let mut bytesRead = 0;
            let path =
                unsafe { SkPath::ReadFromMemory(data.data(), data.size(), Some(&mut bytesRead)) };
            if let Some(path) = path.filter(|_| bytesRead == data.size()) {
                builder.setGlyph(i as u16, advance, &path);
            } else {
                return None;
            }
        }
        arp.markDone();
        builder.detach()
    }
    // Source: src/utils/SkCustomTypeface.cpp:519-522
    pub fn MakeFromStream(
        mut stream: core::pin::Pin<Box<dyn SkStreamAsset>>,
        _: &SkFontArguments,
    ) -> Option<SkTypefaceRef> {
        Self::Deserialize(stream.as_mut())
    }
}

// Rust virtual bridges carry the actual containing Typeface for inherited style.
// These have no additional native source credit.
impl SkTypefaceVirtuals for SkUserTypeface {
    unsafe fn onCreateScalerContext(
        &self,
        face: NonNull<SkTypeface>,
        effects: &SkScalerContextEffects<'_>,
        desc: &SkDescriptor,
        path: &sk_sp<dyn crate::src::core::SkPathEffect::SkPathEffect>,
        mask: Option<&Arc<dyn crate::src::core::SkMaskFilter::SkMaskFilter>>,
    ) -> Box<dyn SkScalerContextVirtual> {
        unsafe { Self::onCreateScalerContext(self, face, effects, desc, path, mask) }
    }
    fn onFilterRec(&self, rec: &mut SkScalerContextRec) {
        Self::onFilterRec(self, rec)
    }
    fn getGlyphToUnicodeMap(&self, dst: &mut [i32]) {
        Self::getGlyphToUnicodeMap(self, dst)
    }
    fn onGetAdvancedMetrics(
        &self,
    ) -> Option<Box<crate::src::core::SkAdvancedTypefaceMetrics::SkAdvancedTypefaceMetrics>> {
        Self::onGetAdvancedMetrics(self)
    }
    fn onGetFontDescriptor(
        &self,
        owner: &SkTypeface,
        desc: &mut crate::src::core::SkFontDescriptor::SkFontDescriptor,
        local: &mut bool,
    ) {
        Self::onGetFontDescriptor(self, owner, desc, local)
    }
    fn onCharsToGlyphs(&self, chars: &[i32], glyphs: &mut [u16]) {
        Self::onCharsToGlyphs(self, chars, glyphs)
    }
    fn onGetFamilyName(&self, name: &mut String) {
        Self::onGetFamilyName(self, name)
    }
    fn onGetPostScriptName(&self, name: &mut String) -> bool {
        Self::onGetPostScriptName(self, name)
    }
    fn onCreateFamilyNameIterator(&self, owner: &SkTypeface) -> Option<Box<dyn LocalizedStrings>> {
        Self::onCreateFamilyNameIterator(self, owner)
    }
    fn onOpenExistingStream(
        &self,
        owner: &SkTypeface,
        index: &mut i32,
    ) -> Option<core::pin::Pin<Box<dyn SkStreamAsset>>> {
        Self::onOpenExistingStream(self, owner, index)
    }
    unsafe fn onMakeClone(
        &self,
        owner: &SkTypefaceRef,
        args: &SkFontArguments,
    ) -> Option<SkTypefaceRef> {
        unsafe { Self::onMakeClone(self, owner, args) }
    }
    fn onCountGlyphs(&self) -> i32 {
        Self::onCountGlyphs(self)
    }
    fn onGetUPEM(&self) -> i32 {
        Self::onGetUPEM(self)
    }
    fn onComputeBounds(&self, owner: &SkTypefaceRef, bounds: &mut SkRect) -> bool {
        Self::onComputeBounds(self, owner, bounds)
    }
    fn getPostScriptGlyphNames(&self, names: &mut [crate::src::core::SkString::SkString]) {
        Self::getPostScriptGlyphNames(self, names)
    }
    fn onGlyphMaskNeedsCurrentColor(&self) -> bool {
        Self::onGlyphMaskNeedsCurrentColor(self)
    }
    unsafe fn onGetVariationDesignPosition(
        &self,
        coordinates: *mut Coordinate,
        capacity: usize,
    ) -> i32 {
        unsafe { Self::onGetVariationDesignPosition(self, coordinates, capacity) }
    }
    fn onGetVariationDesignParameters(&self, axes: &mut [Axis]) -> i32 {
        Self::onGetVariationDesignParameters(self, axes)
    }
    fn onGetTableTags(&self, tags: &mut [SkFontTableTag]) -> i32 {
        Self::onGetTableTags(self, tags)
    }
    fn onGetTableData(
        &self,
        tag: SkFontTableTag,
        offset: usize,
        length: usize,
        data: Option<&mut [u8]>,
    ) -> usize {
        Self::onGetTableData(self, tag, offset, length, data)
    }
    fn onOpenStream(
        &self,
        owner: &SkTypeface,
        index: &mut i32,
    ) -> Option<core::pin::Pin<Box<dyn SkStreamAsset>>> {
        Self::onOpenStream(self, owner, index)
    }
}
