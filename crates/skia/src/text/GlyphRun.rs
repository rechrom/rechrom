use crate::include::core::SkFontTypes::SkTextEncoding;
use crate::src::core::SkFont::{Edging, SkFont};
use crate::src::core::SkPoint::{SkPoint, SkVector};
use crate::src::core::SkRSXform::SkRSXform;
use crate::src::core::SkRect::SkRect;
use crate::src::core::SkTextBlob::{PurgeDelegate, SkTextBlob};
use crate::src::core::SkZip::{SkMakeZip, SkZip};
use std::{marker::PhantomPinned, mem::MaybeUninit, pin::Pin};
// Rust raw span descriptor preserves the native non-owning pointer/length. Safe operations
// inspect metadata only; dereference requires live backing storage for the returned lease.
struct Span<T> {
    ptr: *const T,
    len: usize,
}
impl<T> Copy for Span<T> {}
impl<T> Clone for Span<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Span<T> {
    fn empty() -> Self {
        Self {
            ptr: std::ptr::null(),
            len: 0,
        }
    }
}
impl<T> Span<T> {
    fn new(s: &[T]) -> Self {
        Self {
            ptr: s.as_ptr(),
            len: s.len(),
        }
    }
}
impl<T> Span<T> {
    unsafe fn slice<'a>(&self) -> &'a [T] {
        if self.len == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(self.ptr, self.len)
        }
    }
}
impl GlyphRun {
    // Source: src/text/GlyphRun.h:47-47
    pub fn runSize(&self) -> usize {
        self.fSource.size()
    }
}
impl GlyphRun {
    // Source: src/text/GlyphRun.h:48-48
    pub unsafe fn positions(&self) -> &[SkPoint] {
        self.fSource.get1()
    }
}
impl GlyphRun {
    // Source: src/text/GlyphRun.h:49-49
    pub unsafe fn glyphsIDs(&self) -> &[u16] {
        self.fSource.get0()
    }
}
impl GlyphRun {
    // Source: src/text/GlyphRun.h:50-50
    pub fn source(&self) -> SkZip<u16, SkPoint> {
        self.fSource
    }
}
impl GlyphRun {
    // Source: src/text/GlyphRun.h:51-51
    pub fn font(&self) -> &SkFont {
        &self.fFont
    }
}
impl GlyphRun {
    // Source: src/text/GlyphRun.h:52-52
    pub unsafe fn clusters(&self) -> &[u32] {
        self.fClusters.slice()
    }
}
impl GlyphRun {
    // Source: src/text/GlyphRun.h:53-53
    pub unsafe fn text(&self) -> &[u8] {
        self.fText.slice()
    }
}
impl GlyphRun {
    // Source: src/text/GlyphRun.h:54-54
    pub unsafe fn scaledRotations(&self) -> &[SkVector] {
        self.fScaledRotations.slice()
    }
}
// Adapted source: src/text/GlyphRun.h:36-37: flat Rust run storage declaration separated from native inline methods
// Source: src/text/GlyphRun.h:58-58
// Source: src/text/GlyphRun.h:60-60
// Source: src/text/GlyphRun.h:62-62
// Source: src/text/GlyphRun.h:64-64
// Source: src/text/GlyphRun.h:66-67
pub struct GlyphRun {
    fSource: SkZip<u16, SkPoint>,
    fText: Span<u8>,
    fClusters: Span<u32>,
    fScaledRotations: Span<SkVector>,
    fFont: SkFont,
}
impl GlyphRunList {
    // Source: src/text/GlyphRun.h:88-89
    pub fn canCache(&self) -> bool {
        !self.fOriginalTextBlob.is_null()
    }
}
impl GlyphRunList {
    pub fn runCount(&self) -> usize {
        self.fGlyphRuns.len
    }
}
impl GlyphRunList {
    // Source: src/text/GlyphRun.h:90-103
    pub unsafe fn totalGlyphCount(&self) -> usize {
        let mut count = 0usize;
        for run in self.runs() {
            count = count.wrapping_add(run.runSize())
        }
        count
    }
}
impl GlyphRunList {
    pub unsafe fn maxGlyphRunSize(&self) -> usize {
        let mut size = 0;
        for run in self.runs() {
            size = run.runSize().max(size)
        }
        size
    }
}
impl GlyphRunList {
    // Source: src/text/GlyphRun.h:105-110
    pub unsafe fn hasRSXForm(&self) -> bool {
        for run in self.runs() {
            if !run.scaledRotations().is_empty() {
                return true;
            }
        }
        false
    }
}
impl GlyphRunList {
    // Source: src/text/GlyphRun.h:114-118
    pub fn origin(&self) -> SkPoint {
        self.fOrigin
    }
}
impl GlyphRunList {
    pub fn sourceBounds(&self) -> SkRect {
        self.fSourceBounds
    }
}
impl GlyphRunList {
    pub fn sourceBoundsWithOrigin(&self) -> SkRect {
        self.fSourceBounds.makeOffsetVec(self.fOrigin)
    }
}
impl GlyphRunList {
    pub fn blob(&self) -> *const SkTextBlob {
        self.fOriginalTextBlob
    }
}
impl GlyphRunList {
    pub fn builder(&self) -> *mut GlyphRunBuilder {
        self.fBuilder
    }
}
impl GlyphRunList {
    // Source: src/text/GlyphRun.h:120-126
    pub unsafe fn runs(&self) -> &[GlyphRun] {
        self.fGlyphRuns.slice()
    }
}
impl GlyphRunList {
    pub fn size(&self) -> usize {
        self.fGlyphRuns.len
    }
}
impl GlyphRunList {
    pub fn empty(&self) -> bool {
        self.fGlyphRuns.len == 0
    }
}
// Adapted source: src/text/GlyphRun.h:69-70: early native run-span field and later fields131-135 share one flat Rust storage; preserve physical/member order
// Structural source: src/text/GlyphRun.h:72-72: native public access label is language scaffolding
// Source: src/text/GlyphRun.h:131-135
pub struct GlyphRunList {
    fGlyphRuns: Span<GlyphRun>,
    fOriginalTextBlob: *const SkTextBlob,
    fSourceBounds: SkRect,
    fOrigin: SkPoint,
    fBuilder: *mut GlyphRunBuilder,
}
impl GlyphRunBuilder {
    // Source: src/text/GlyphRun.h:150-150
    pub fn empty(&self) -> bool {
        self.fGlyphRunListStorage.is_empty()
    }
}
impl Default for GlyphRunBuilder {
    fn default() -> Self {
        Self::new()
    }
}
// Uncounted lifetime adaptation: release scalar scratch/list views and run fonts before
// backing buffers, preserving native reverse member destruction. No view is dereferenced.
impl Drop for GlyphRunBuilder {
    fn drop(&mut self) {
        drop(std::mem::take(&mut self.fScratchGlyphIDs));
        drop(self.fGlyphRunList.take());
        while let Some(run) = self.fGlyphRunListStorage.pop() {
            drop(run)
        }
        drop(self.fScaledRotations.take());
        drop(self.fPositions.take());
    }
}
// Adapted source: src/text/GlyphRun.h:137-138: flat pinned builder storage declaration separated from native inline empty accessor
// Source: src/text/GlyphRun.h:170-173
// Source: src/text/GlyphRun.h:175-176
// Source: src/text/GlyphRun.h:180-180
// Source: src/text/GlyphRun.h:182-182
pub struct GlyphRunBuilder {
    fMaxTotalRunSize: i32,
    fPositions: Option<Box<[MaybeUninit<SkPoint>]>>,
    fMaxScaledRotations: i32,
    fScaledRotations: Option<Box<[MaybeUninit<SkVector>]>>,
    fGlyphRunListStorage: Vec<GlyphRun>,
    fGlyphRunList: Option<GlyphRunList>,
    fScratchGlyphIDs: Vec<u16>,
    pinned: PhantomPinned,
}
impl GlyphRunBuilder {
    // Default construction realizes the native in-class initializers and default constructors.
    pub fn new() -> Self {
        Self {
            fMaxTotalRunSize: 0,
            fPositions: None,
            fMaxScaledRotations: 0,
            fScaledRotations: None,
            fGlyphRunListStorage: Vec::new(),
            fGlyphRunList: None,
            fScratchGlyphIDs: Vec::new(),
            pinned: PhantomPinned,
        }
    }
}
impl GlyphRun {
    // Source: src/text/GlyphRun.cpp:26-36
    /// Safety: backing spans must remain valid while this run or any of its views is used.
    pub unsafe fn new(
        font: &SkFont,
        positions: &[SkPoint],
        glyph_ids: &[u16],
        text: &[u8],
        clusters: &[u32],
        scaled_rotations: &[SkVector],
    ) -> Self {
        Self {
            fSource: SkMakeZip(glyph_ids, positions),
            fText: Span::new(text),
            fClusters: Span::new(clusters),
            fScaledRotations: Span::new(scaled_rotations),
            fFont: font.clone(),
        }
    }
}
impl GlyphRun {
    // Source: src/text/GlyphRun.cpp:38-42
    /// Safety: all source backing owners must outlive this new view as well.
    pub unsafe fn with_font(that: &Self, font: &SkFont) -> Self {
        Self {
            fSource: that.fSource,
            fText: that.fText,
            fClusters: that.fClusters,
            fScaledRotations: Span::empty(),
            fFont: font.clone(),
        }
    }
}
impl GlyphRunList {
    // Source: src/text/GlyphRun.cpp:45-54
    /// Safety: run/blob owners and the pinned builder must outlive the list and its views.
    pub unsafe fn new(
        blob: *const SkTextBlob,
        bounds: SkRect,
        origin: SkPoint,
        runs: &[GlyphRun],
        builder: *mut GlyphRunBuilder,
    ) -> Self {
        Self {
            fGlyphRuns: Span::new(runs),
            fOriginalTextBlob: blob,
            fSourceBounds: bounds,
            fOrigin: origin,
            fBuilder: builder,
        }
    }
}
impl GlyphRunList {
    // Source: src/text/GlyphRun.cpp:56-64
    pub unsafe fn from_run(
        run: &GlyphRun,
        bounds: &SkRect,
        origin: SkPoint,
        builder: *mut GlyphRunBuilder,
    ) -> Self {
        Self::new(
            std::ptr::null(),
            *bounds,
            origin,
            std::slice::from_ref(run),
            builder,
        )
    }
}
impl GlyphRunList {
    // Source: src/text/GlyphRun.cpp:66-69
    pub unsafe fn uniqueID(&self) -> u64 {
        if self.fOriginalTextBlob.is_null() {
            0
        } else {
            (*self.fOriginalTextBlob).uniqueID() as u64
        }
    }
}
impl GlyphRunList {
    // Source: src/text/GlyphRun.cpp:71-78
    pub unsafe fn anyRunsLCD(&self) -> bool {
        for run in self.runs() {
            if run.font().getEdging() == Edging::kSubpixelAntiAlias {
                return true;
            }
        }
        false
    }
}
impl GlyphRunList {
    // Source: src/text/GlyphRun.cpp:80-85
    pub unsafe fn temporaryShuntBlobNotifyAddedToCache(
        &self,
        cache_id: u32,
        purge_delegate: PurgeDelegate,
    ) {
        crate::SkASSERT!(@source b"src/text/GlyphRun.cpp\0".as_ptr(),82; @condition b"fOriginalTextBlob != nullptr\0".as_ptr(); !self.fOriginalTextBlob.is_null());
        (*self.fOriginalTextBlob).notifyAddedToCache(cache_id, purge_delegate)
    }
}
impl GlyphRunList {
    // Source: src/text/GlyphRun.cpp:87-115
    /// Safety: all run backing spans and native font/scaler leases remain live.
    /// All run buffers must be fully initialized before any callback can unwind.
    pub unsafe fn makeBlob(&self) -> crate::include::core::SkRefCnt::sk_sp<SkTextBlob> {
        let mut builder = crate::src::core::SkTextBlob::SkTextBlobBuilder::new();
        for run in self.runs() {
            let buffer;
            if run.scaledRotations().is_empty() {
                if run.text().is_empty() {
                    buffer = *builder.allocRunPos(run.font(), run.runSize() as i32, None);
                } else {
                    buffer = *builder.allocRunTextPos(
                        run.font(),
                        run.runSize() as i32,
                        run.text().len() as i32,
                        None,
                    );
                    let text = run.text();
                    if !text.is_empty() {
                        std::ptr::copy_nonoverlapping(text.as_ptr(), buffer.utf8text, text.len());
                    }
                    let clusters = run.clusters();
                    if !clusters.is_empty() {
                        std::ptr::copy_nonoverlapping(
                            clusters.as_ptr(),
                            buffer.clusters,
                            clusters.len(),
                        );
                    }
                }
                let positions = run.positions();
                if !positions.is_empty() {
                    std::ptr::copy_nonoverlapping(
                        positions.as_ptr(),
                        buffer.points(),
                        positions.len(),
                    );
                }
            } else {
                buffer = *builder.allocRunRSXform(run.font(), run.runSize() as i32);
                for (i, (pos, sr)) in run
                    .positions()
                    .iter()
                    .zip(run.scaledRotations())
                    .enumerate()
                {
                    buffer.xforms().add(i).write(SkRSXform {
                        fSCos: sr.fX,
                        fSSin: sr.fY,
                        fTx: pos.fX,
                        fTy: pos.fY,
                    });
                }
            }
            let ids = run.glyphsIDs();
            if !ids.is_empty() {
                std::ptr::copy_nonoverlapping(ids.as_ptr(), buffer.glyphs, ids.len());
            }
        }
        builder.make()
    }
}

// Source: src/text/GlyphRun.cpp:118-185
// Adapted Result propagates the existing real scaler error. No fallback bounds
// replace a missing/erroring backend. All source numeric and branch order stays.
unsafe fn glyphrun_source_bounds(
    font: &SkFont,
    paint: &crate::src::core::SkPaint::SkPaint,
    source: SkZip<u16, SkPoint>,
    scaledRotations: &[SkVector],
) -> Result<SkRect, crate::RasterError> {
    use crate::src::core::{
        SkFontPriv::SkFontPriv,
        SkMatrix::SkMatrix,
        SkStrikeSpec::{SkBulkGlyphMetrics, SkStrikeSpec},
    };
    debug_assert!(source.size() != 0);
    let fontBounds = SkFontPriv::GetFontBounds(font);
    let ids = source.get0();
    let positions = source.get1();
    if fontBounds.isEmpty() {
        let (spec, scale) = SkStrikeSpec::MakeCanonicalized(font, Some(paint))?;
        let mut metrics = SkBulkGlyphMetrics::new(&spec);
        let glyphs = metrics.glyphs(ids);
        if scaledRotations.is_empty() {
            let scaleAndTranslateRect = |r: &SkRect, pos: &SkPoint| {
                SkRect::MakeLTRB(
                    r.fLeft * scale + pos.fX,
                    r.fTop * scale + pos.fY,
                    r.fRight * scale + pos.fX,
                    r.fBottom * scale + pos.fY,
                )
            };
            let mut bounds = SkRect::MakeEmpty();
            for (pos, glyph) in positions.iter().zip(glyphs) {
                let rect = glyph.as_ref().rect();
                if !rect.isEmpty() {
                    bounds.join(&scaleAndTranslateRect(&rect, pos));
                }
            }
            return Ok(bounds);
        } else {
            let mut bounds = SkRect::MakeEmpty();
            for ((pos, sr), glyph) in positions.iter().zip(scaledRotations).zip(glyphs) {
                let rect = glyph.as_ref().rect();
                if !rect.isEmpty() {
                    let mut xform = SkMatrix::default();
                    // Preserve the pinned source's aggregate initializer order,
                    // including pos in the first two SkRSXform fields.
                    xform.setRSXform(&SkRSXform {
                        fSCos: pos.fX,
                        fSSin: pos.fY,
                        fTx: sr.fX,
                        fTy: sr.fY,
                    });
                    xform.preScale(scale, scale);
                    bounds.join(&xform.mapRect_value(&rect));
                }
            }
            return Ok(bounds);
        }
    }
    if scaledRotations.is_empty() {
        let mut bounds = SkRect::BoundsOrEmpty(positions);
        bounds.fLeft += fontBounds.fLeft;
        bounds.fTop += fontBounds.fTop;
        bounds.fRight += fontBounds.fRight;
        bounds.fBottom += fontBounds.fBottom;
        Ok(bounds)
    } else {
        let mut bounds = SkRect::MakeEmpty();
        for (pos, sr) in positions.iter().zip(scaledRotations) {
            let xform = SkRSXform {
                fSCos: pos.fX,
                fSSin: pos.fY,
                fTx: sr.fX,
                fTy: sr.fY,
            };
            let mut matrix = SkMatrix::default();
            matrix.setRSXform(&xform);
            bounds.join(&matrix.mapRect_value(&fontBounds));
        }
        Ok(bounds)
    }
}

impl GlyphRunBuilder {
    // Source: src/text/GlyphRun.cpp:187-193
    /// Safety: run backing and this pinned builder outlive the returned raw-span list.
    pub unsafe fn makeGlyphRunList(
        self: Pin<&mut Self>,
        run: &GlyphRun,
        paint: &crate::src::core::SkPaint::SkPaint,
        origin: SkPoint,
    ) -> Result<GlyphRunList, crate::RasterError> {
        let bounds =
            glyphrun_source_bounds(run.font(), paint, run.source(), run.scaledRotations())?;
        Ok(GlyphRunList::from_run(
            run,
            &bounds,
            origin,
            self.get_unchecked_mut(),
        ))
    }
}

// Source: src/text/GlyphRun.cpp:195-209
/// Safety: buffer has glyphIDs.len() writable point slots and stays live for 'a.
/// All font/strike and synchronous glyph read leases hold.
unsafe fn draw_text_positions<'a>(
    font: &SkFont,
    glyphIDs: &[u16],
    origin: SkPoint,
    buffer: *mut SkPoint,
) -> Result<&'a [SkPoint], crate::RasterError> {
    use crate::src::core::SkStrikeSpec::{SkBulkGlyphMetrics, SkStrikeSpec};
    let spec = SkStrikeSpec::MakeWithNoDevice(
        font,
        None,
        crate::src::core::SkScalerContext::SkScalerContextFlags::FAKE_GAMMA_AND_BOOST_CONTRAST,
    )?;
    let mut storage = SkBulkGlyphMetrics::new(&spec);
    let glyphs = storage.glyphs(glyphIDs);
    let mut cursor = buffer;
    let mut end = origin;
    for glyph in glyphs {
        cursor.write(end);
        cursor = cursor.add(1);
        let advance = glyph.as_ref().advanceVector();
        end.fX += advance.fX;
        end.fY += advance.fY;
    }
    Ok(if glyphIDs.is_empty() {
        &[]
    } else {
        std::slice::from_raw_parts(buffer, glyphIDs.len())
    })
}

impl GlyphRunBuilder {
    // Source: src/text/GlyphRun.cpp:211-229
    /// Safety: encoded input and font leases remain live; all prior builder
    /// views expire. This builder stays pinned until the returned view expires.
    pub unsafe fn textToGlyphRunList(
        mut self: Pin<&mut Self>,
        font: &SkFont,
        paint: &crate::src::core::SkPaint::SkPaint,
        bytes: *const std::ffi::c_void,
        byteLength: usize,
        origin: SkPoint,
        encoding: SkTextEncoding,
    ) -> Result<&GlyphRunList, crate::RasterError> {
        let (ids_ptr, ids_len) = self
            .as_mut()
            .textToGlyphIDs(font, bytes, byteLength, encoding);
        let mut bounds = SkRect::MakeEmpty();
        let this = self.get_unchecked_mut();
        this.prepare_buffers(ids_len as i32, 0);
        if ids_len != 0 {
            let ids = std::slice::from_raw_parts(ids_ptr, ids_len);
            let buffer = this
                .fPositions
                .as_mut()
                .unwrap()
                .as_mut_ptr()
                .cast::<SkPoint>();
            let positions = draw_text_positions(font, ids, SkPoint::make(0.0, 0.0), buffer)?;
            // Same native makeGlyphRun insertion, projected only to the disjoint
            // run-list field while the position/id allocation leases are live.
            this.fGlyphRunListStorage
                .push(GlyphRun::new(font, positions, ids, &[], &[], &[]));
            let run = &this.fGlyphRunListStorage[0];
            bounds =
                glyphrun_source_bounds(run.font(), paint, run.source(), run.scaledRotations())?;
        }
        Ok(Pin::new_unchecked(this).setGlyphRunList(std::ptr::null(), &bounds, origin))
    }

    // Source: src/text/GlyphRun.cpp:231-289
    /// Safety: blob data and native font/strike leases remain live; previous
    /// builder views expire before this call. Builder stays pinned.
    pub unsafe fn blobToGlyphRunList(
        mut self: Pin<&mut Self>,
        blob: &SkTextBlob,
        origin: SkPoint,
    ) -> Result<&GlyphRunList, crate::RasterError> {
        use crate::src::core::{
            SkFontPriv::SkFontPriv, SkTextBlob::GlyphPositioning,
            SkTextBlobPriv::SkTextBlobRunIterator,
        };
        self.as_mut().initialize(blob);
        let this = self.get_unchecked_mut();
        let mut positionCursor = this
            .fPositions
            .as_mut()
            .map_or(std::ptr::null_mut(), |p| p.as_mut_ptr().cast::<SkPoint>());
        let mut rotationCursor = this
            .fScaledRotations
            .as_mut()
            .map_or(std::ptr::null_mut(), |p| p.as_mut_ptr().cast::<SkVector>());
        let mut it = SkTextBlobRunIterator::new(blob);
        while !it.done() {
            let size = it.glyphCount() as usize;
            if size == 0 || !SkFontPriv::IsFinite(it.font()) {
                it.next();
                continue;
            }
            let font = it.font();
            let ids = std::slice::from_raw_parts(it.glyphs(), size);
            let positions;
            let mut rotations: &[SkVector] = &[];
            match it.positioning() {
                GlyphPositioning::kDefault_Positioning => {
                    positions = draw_text_positions(font, ids, *it.offset(), positionCursor)?;
                    positionCursor = positionCursor.add(positions.len());
                }
                GlyphPositioning::kHorizontal_Positioning => {
                    let start = positionCursor;
                    for x in std::slice::from_raw_parts(it.pos(), ids.len()) {
                        positionCursor.write(SkPoint::make(*x, it.offset().fY));
                        positionCursor = positionCursor.add(1);
                    }
                    positions = std::slice::from_raw_parts(start, size);
                }
                GlyphPositioning::kFull_Positioning => {
                    positions = std::slice::from_raw_parts(it.points(), size);
                }
                GlyphPositioning::kRSXform_Positioning => {
                    let start = positionCursor;
                    let rotationsStart = rotationCursor;
                    for x in std::slice::from_raw_parts(it.xforms(), size) {
                        positionCursor.write(SkPoint::make(x.fTx, x.fTy));
                        positionCursor = positionCursor.add(1);
                        rotationCursor.write(SkPoint::make(x.fSCos, x.fSSin));
                        rotationCursor = rotationCursor.add(1);
                    }
                    // Construct typed slices only after native scratch bytes are initialized.
                    positions = std::slice::from_raw_parts(start, size);
                    rotations = std::slice::from_raw_parts(rotationsStart, size);
                }
            }
            let clusters = it.clusters();
            let text = if it.textSize() == 0 {
                &[]
            } else {
                std::slice::from_raw_parts(it.text(), it.textSize() as usize)
            };
            let clusters = if clusters.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(clusters, size)
            };
            this.fGlyphRunListStorage.push(GlyphRun::new(
                font, positions, ids, text, clusters, rotations,
            ));
            it.next();
        }
        Ok(Pin::new_unchecked(this).setGlyphRunList(blob, blob.bounds(), origin))
    }
}

impl GlyphRunBuilder {
    // Source: src/text/GlyphRun.cpp:291-303
    /// Safety: expires previous views; xform count fits int. Builder remains pinned.
    pub unsafe fn convertRSXForm(
        self: Pin<&mut Self>,
        xforms: &[SkRSXform],
    ) -> (&[SkPoint], &[SkVector]) {
        debug_assert!(xforms.len() <= i32::MAX as usize);
        let count = xforms.len() as i32;
        let this = self.get_unchecked_mut();
        this.prepare_buffers(count, count);
        let p = this
            .fPositions
            .as_mut()
            .map_or(std::ptr::null_mut(), |p| p.as_mut_ptr().cast::<SkPoint>());
        let sr = this
            .fScaledRotations
            .as_mut()
            .map_or(std::ptr::null_mut(), |p| p.as_mut_ptr().cast::<SkVector>());
        for (i, x) in xforms.iter().enumerate() {
            p.add(i).write(SkPoint::make(x.fTx, x.fTy));
            sr.add(i).write(SkPoint::make(x.fSCos, x.fSSin))
        }
        if count == 0 {
            (&[], &[])
        } else {
            (
                std::slice::from_raw_parts(p, count as usize),
                std::slice::from_raw_parts(sr, count as usize),
            )
        }
    }
}
impl GlyphRunBuilder {
    // Source: src/text/GlyphRun.cpp:305-315
    // Source: src/text/GlyphRun.cpp:317-318
    /// Safety: blob's complete immutable run storage stays live through this call;
    /// all previous views into this builder expire before preparation mutates storage.
    pub unsafe fn initialize(self: Pin<&mut Self>, blob: &SkTextBlob) {
        use crate::src::core::SkTextBlob::GlyphPositioning;
        use crate::src::core::SkTextBlobPriv::SkTextBlobRunIterator;
        let mut position_count = 0i32;
        let mut rsxform_count = 0i32;
        let mut it = SkTextBlobRunIterator::new(blob);
        while !it.done() {
            if it.positioning() != GlyphPositioning::kFull_Positioning {
                position_count = position_count.wrapping_add(it.glyphCount() as i32);
            }
            if it.positioning() == GlyphPositioning::kRSXform_Positioning {
                rsxform_count = rsxform_count.wrapping_add(it.glyphCount() as i32);
            }
            it.next();
        }
        self.get_unchecked_mut()
            .prepare_buffers(position_count, rsxform_count);
    }
}
impl GlyphRunBuilder {
    // Source: src/text/GlyphRun.cpp:320-324
    // Source: src/text/GlyphRun.cpp:326-329
    // Source: src/text/GlyphRun.cpp:331-332
    fn prepare_buffers(&mut self, position_count: i32, rsxform_count: i32) {
        if position_count > self.fMaxTotalRunSize {
            self.fMaxTotalRunSize = position_count;
            self.fPositions = Some(Box::<[SkPoint]>::new_uninit_slice(
                self.fMaxTotalRunSize as usize,
            ))
        }
        if rsxform_count > self.fMaxScaledRotations {
            self.fMaxScaledRotations = rsxform_count;
            self.fScaledRotations =
                Some(Box::<[SkVector]>::new_uninit_slice(rsxform_count as usize))
        }
        while let Some(run) = self.fGlyphRunListStorage.pop() {
            drop(run)
        }
    }
}
impl GlyphRunBuilder {
    // Source: src/text/GlyphRun.cpp:334-348
    /// Safety: expires prior scratch IDs; bytes is readable/aligned for encoding.
    pub unsafe fn textToGlyphIDs(
        self: Pin<&mut Self>,
        font: &SkFont,
        bytes: *const std::ffi::c_void,
        byte_length: usize,
        encoding: SkTextEncoding,
    ) -> (*const u16, usize) {
        let this = self.get_unchecked_mut();
        if encoding != SkTextEncoding::kGlyphID {
            let count = font.countText(bytes, byte_length, encoding) as i32;
            if count > 0 {
                this.fScratchGlyphIDs.resize(count as usize, 0);
                font.textToGlyphs(bytes, byte_length, encoding, &mut this.fScratchGlyphIDs);
                (this.fScratchGlyphIDs.as_ptr(), this.fScratchGlyphIDs.len())
            } else {
                (std::ptr::null(), 0)
            }
        } else {
            (bytes.cast(), byte_length / 2)
        }
    }
}
impl GlyphRunBuilder {
    // Source: src/text/GlyphRun.cpp:350-356
    // Source: src/text/GlyphRun.cpp:359-368
    /// Safety: expires prior run-list view; all backing owners remain live for subsequent use.
    pub unsafe fn makeGlyphRun(
        self: Pin<&mut Self>,
        font: &SkFont,
        glyph_ids: &[u16],
        positions: &[SkPoint],
        text: &[u8],
        clusters: &[u32],
        scaled_rotations: &[SkVector],
    ) {
        if !glyph_ids.is_empty() {
            self.get_unchecked_mut()
                .fGlyphRunListStorage
                .push(GlyphRun::new(
                    font,
                    positions,
                    glyph_ids,
                    text,
                    clusters,
                    scaled_rotations,
                ))
        }
    }
}
impl GlyphRunBuilder {
    // Source: src/text/GlyphRun.cpp:370-374
    /// Safety: previous list/views expire; nonnull blob stays live and builder stays pinned.
    pub unsafe fn setGlyphRunList(
        self: Pin<&mut Self>,
        blob: *const SkTextBlob,
        bounds: &SkRect,
        origin: SkPoint,
    ) -> &GlyphRunList {
        let this = self.get_unchecked_mut();
        let list = GlyphRunList::new(
            blob,
            *bounds,
            origin,
            &this.fGlyphRunListStorage,
            this as *const Self as *mut Self,
        );
        this.fGlyphRunList = Some(list);
        this.fGlyphRunList.as_ref().unwrap()
    }
}

impl GlyphRunBuilder {
    // Source: src/text/GlyphRun.h:145-145
    /// Safety: same encoded-byte lease and pinning contract as textToGlyphRunList.
    pub unsafe fn textToGlyphRunList_utf8(
        self: Pin<&mut Self>,
        font: &SkFont,
        paint: &crate::src::core::SkPaint::SkPaint,
        bytes: *const std::ffi::c_void,
        byteLength: usize,
        origin: SkPoint,
    ) -> Result<&GlyphRunList, crate::RasterError> {
        unsafe {
            self.textToGlyphRunList(
                font,
                paint,
                bytes,
                byteLength,
                origin,
                SkTextEncoding::kUTF8,
            )
        }
    }
}
