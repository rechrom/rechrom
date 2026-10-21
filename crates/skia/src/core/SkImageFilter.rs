#![allow(non_snake_case)]
use super::SkImageFilterTypes::{
    DeviceSpace, LayerSpace, Mapping, MatrixCapability, ParameterSpace,
};
use super::SkImageFilter_Base::SkImageFilter_Base;
use super::SkRect::{SkIRect, SkRect};
use std::sync::Arc;
// Source: include/core/SkImageFilter.h:37-40
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapDirection {
    kForward_MapDirection,
    kReverse_MapDirection,
}
// Rust trait collects virtual/default methods before the separate base constructor definition.
// Rust trait composition replaces the native public/base inheritance and keeps input owners intact.
// The trait includes the actual DAG, rendering and retained color-filter/local-matrix entry paths.
pub trait SkImageFilter: super::SkImageFilter_Base::SkImageFilterBaseHooks {
    fn base(&self) -> &SkImageFilter_Base;
    // Source: include/core/SkImageFilter.h:81-81
    // Source: src/core/SkImageFilter.cpp:46-46
    fn countInputs(&self) -> usize {
        self.base().fInputs.len()
    }
    // Source: include/core/SkImageFilter.h:87-87
    // Source: src/core/SkImageFilter.cpp:52-55
    fn getInput(&self, i: usize) -> Option<&Arc<dyn SkImageFilter>> {
        crate::SkASSERT!(@source b"src/core/SkImageFilter.cpp\0".as_ptr(),53; @condition b"i < this->countInputs()\0".as_ptr(); i < self.countInputs());
        self.base().fInputs[i].as_ref()
    }
    // Source: src/core/SkImageFilter.cpp:57-59
    fn isColorFilterNode(
        &self,
        filter: &mut Option<Arc<dyn super::SkColorFilter::SkColorFilter>>,
    ) -> bool {
        self.onIsColorFilterNode(filter)
    }
    // Source: include/core/SkImageFilter.h:67-69
    fn asColorFilter(
        &self,
        filter: &mut Option<Arc<dyn super::SkColorFilter::SkColorFilter>>,
    ) -> bool {
        self.isColorFilterNode(filter)
    }
    // Source: include/core/SkImageFilter.h:54-55
    // Source: src/core/SkImageFilter.cpp:61-80
    fn filterBounds(
        &self,
        src: SkIRect,
        ctm: &super::SkMatrix::SkMatrix,
        direction: MapDirection,
        input: Option<SkIRect>,
    ) -> Result<SkIRect, crate::RasterError> {
        Ok({
            let mapping = Mapping::new(super::SkM44::SkM44::from(ctm));
            if direction == MapDirection::kReverse_MapDirection {
                let desired = LayerSpace::new(src);
                let content = input.map(LayerSpace::new);
                *self
                    .onGetInputLayerBounds(&mapping, &desired, content)?
                    .geometry()
            } else {
                crate::SkASSERT!(@source b"src/core/SkImageFilter.cpp\0".as_ptr(),75; @condition b"!inputRect\0".as_ptr(); input.is_none());
                let output = self.onGetOutputLayerBounds(&mapping, Some(LayerSpace::new(src)))?;
                match output {
                    Some(r) => *r.geometry(),
                    None => super::SkRectPriv::SkRectPriv::MakeILarge(),
                }
            }
        })
    }
    // Source: include/core/SkImageFilter.h:90-90
    // Source: src/core/SkImageFilter.cpp:82-96
    fn computeFastBounds(&self, src: &SkRect) -> Result<SkRect, crate::RasterError> {
        compute_fast_bounds_base(self, src)
    }
    // Source: include/core/SkImageFilter.h:93-93
    // Source: src/core/SkImageFilter.cpp:98-100
    fn canComputeFastBounds(&self) -> Result<bool, crate::RasterError> {
        Ok({ !self.affectsTransparentBlack()? })
    }
    // Source: src/core/SkImageFilter_Base.h:119-119
    // Source: src/core/SkImageFilter.cpp:102-116
    fn affectsTransparentBlack(&self) -> Result<bool, crate::RasterError> {
        Ok({
            if self.onAffectsTransparentBlack()? {
                return Ok(true);
            } else if self.ignoreInputsAffectsTransparentBlack() {
                return Ok(false);
            }
            for i in 0..self.countInputs() {
                if let Some(input) = self.getInput(i) {
                    if input.affectsTransparentBlack()? {
                        return Ok(true);
                    }
                }
            }
            false
        })
    }
    // Source: include/core/SkImageFilter.h:76-76
    // Source: src/core/SkImageFilter.cpp:118-128
    // Arc/output-option and fallible scalar evaluation adapt native retained pointers.
    fn asAColorFilter(
        &self,
        filter: &mut Option<Arc<dyn super::SkColorFilter::SkColorFilter>>,
    ) -> Result<bool, crate::RasterError> {
        if !self.isColorFilterNode(filter) {
            return Ok(false);
        }
        if self.getInput(0).is_some()
            || filter
                .as_ref()
                .expect("native successful color-filter node supplies a retained filter")
                .affectsTransparentBlack()?
        {
            // Native unreferences its output on rejection; clearing avoids a dangling handle.
            drop(filter.take());
            return Ok(false);
        }
        Ok(true)
    }
    // Source: src/core/SkImageFilter_Base.h:36-36
    // Source: src/core/SkImageFilter.cpp:231-263
    /// Safety: retain Context's complete native source/backend draw leases.
    unsafe fn filterImage(
        &self,
        context: &super::SkImageFilterTypes::Context<'_>,
    ) -> Result<super::SkImageFilterTypes::FilterResult, crate::RasterError> {
        context.markVisitedImageFilter();
        let mut result = super::SkImageFilterTypes::FilterResult::default();
        if context.desiredOutput().isEmpty() || !context.mapping().layerMatrix().isFinite() {
            return Ok(result);
        }
        let sourceInKey = self.base().fUsesSrcInput && context.source().has_image();
        let sourceImage = context.source().image();
        let sourceId = if sourceInKey {
            sourceImage.unwrap().uniqueID()
        } else {
            0
        };
        let subset = if sourceInKey {
            *sourceImage.unwrap().subset()
        } else {
            SkIRect::make_wh(0, 0)
        };
        let key = super::SkImageFilterCache::SkImageFilterCacheKey::new(
            self.base().fUniqueID,
            &context.mapping().layerMatrix().asM33(),
            *context.desiredOutput().geometry(),
            sourceId,
            subset,
        );
        if let Some(cache) = context.backend().cache() {
            if cache.get(&key, &mut result) {
                context.markCacheHit();
                return Ok(result);
            }
        }
        result = self.onFilterImage(context)?;
        if let Some(cache) = context.backend().cache() {
            cache.set_with_filter_identity(
                &key,
                self.base() as *const SkImageFilter_Base as usize,
                &result,
            );
        }
        Ok(result)
    }
    // Source: src/core/SkImageFilter_Base.h:43-48
    // Source: src/core/SkImageFilter.cpp:265-300
    /// Safety: src retains an initialized immutable readable backing lease for
    /// the full CPU DAG and returned image; backend's offscreen pixels and draw
    /// callbacks obey their native exclusive leases. Output views must not alias.
    /// Result propagates actual raster construction/evaluation errors rather
    /// than converting a rejected stage to a transparent output.
    unsafe fn makeImageWithFilter(
        &self,
        backend: Arc<dyn super::SkImageFilterTypes::Backend>,
        src: Arc<dyn crate::src::image::SkImage::SkImage>,
        subset: &SkIRect,
        clipBounds: &SkIRect,
        outSubset: Option<&mut SkIRect>,
        offset: Option<&mut super::SkPoint::SkIPoint>,
    ) -> Result<Option<Arc<dyn crate::src::image::SkImage::SkImage>>, crate::RasterError> {
        use super::SkImageFilterTypes::{Context, FilterResult, Stats};
        let (Some(outSubset), Some(offset)) = (outSubset, offset) else {
            return Ok(None);
        };
        if !src.bounds().contains_rect(subset) {
            return Ok(None);
        }
        let Some(srcSpecialImage) = backend.makeImage(subset, Some(src.clone())) else {
            return Ok(None);
        };
        let stats = core::cell::Cell::new(Stats::default());
        let mapping = Mapping::new(super::SkM44::SkM44::default());
        let source = FilterResult::new(Some(srcSpecialImage), LayerSpace::new(subset.top_left()));
        let context = Context::new(
            backend,
            &mapping,
            LayerSpace::new(*clipBounds),
            &source,
            src.imageInfo().refColorSpace(),
            Some(&stats),
        );
        // Context owns the source copy. Release the temporary immediately so
        // no extra source-image owner remains through filter evaluation.
        drop(source);
        let (result, origin) = self.filterImage(&context)?.imageAndOffset(&context)?;
        *offset = *origin.geometry();
        stats.get().reportStats();
        let Some(result) = result else {
            return Ok(None);
        };
        crate::SkASSERT!(@source b"src/core/SkImageFilter.cpp\0".as_ptr(),296; @condition b"clipBounds.contains(SkIRect::MakeXYWH(offset->fX, offset->fY, result->width(), result->height()))\0".as_ptr(); clipBounds.contains_rect(&SkIRect::make_xywh(offset.fX,offset.fY,result.width(),result.height())));
        *outSubset = *result.subset();
        Ok(result.asImage())
    }

    // Source: src/core/SkImageFilter_Base.h:80-83
    // Source: src/core/SkImageFilter.cpp:302-318
    fn getInputBounds(
        &self,
        mapping: &Mapping,
        desired: &DeviceSpace<SkIRect>,
        known: Option<ParameterSpace<SkRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        Ok({
            let desiredBounds = mapping.deviceToLayer(desired);
            let content = known.as_ref().map(|r| mapping.paramToLayer(r).roundOut());
            self.onGetInputLayerBounds(mapping, &desiredBounds, content)?
        })
    }
    // Source: src/core/SkImageFilter_Base.h:113-115
    // Source: src/core/SkImageFilter.cpp:320-335
    fn getOutputBounds(
        &self,
        mapping: &Mapping,
        content: &ParameterSpace<SkRect>,
    ) -> Result<Option<DeviceSpace<SkIRect>>, crate::RasterError> {
        Ok({
            let layer = mapping.paramToLayer(content);
            let output = self.onGetOutputLayerBounds(mapping, Some(layer.roundOut()))?;
            output.map(|out| mapping.layerToDevice(&out))
        })
    }
    // Source: src/core/SkImageFilter_Base.h:128-128
    // Source: src/core/SkImageFilter.cpp:337-346
    fn getCTMCapability(&self) -> MatrixCapability {
        let mut result = self.onGetCTMCapability();
        for i in 0..self.countInputs() {
            if let Some(input) = self.getInput(i) {
                let child = input.getCTMCapability();
                if (child as u8) < (result as u8) {
                    result = child;
                }
            }
        }
        result
    }
    // Source: src/core/SkImageFilter_Base.h:183-187
    // Source: src/core/SkImageFilter.cpp:348-370
    fn getChildInputLayerBounds(
        &self,
        index: usize,
        mapping: &Mapping,
        desired: &LayerSpace<SkIRect>,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        Ok({
            if let Some(child) = self.getInput(index) {
                child.onGetInputLayerBounds(mapping, desired, content)?
            } else {
                let mut visible = *desired;
                if let Some(bounds) = content {
                    if !visible.intersect(&bounds) {
                        return Ok(LayerSpace::<SkIRect>::Empty());
                    }
                }
                visible
            }
        })
    }
    // Source: src/core/SkImageFilter_Base.h:188-191
    // Source: src/core/SkImageFilter.cpp:372-381
    fn getChildOutputLayerBounds(
        &self,
        index: usize,
        mapping: &Mapping,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        Ok({
            match self.getInput(index) {
                Some(child) => child.onGetOutputLayerBounds(mapping, content)?,
                None => content,
            }
        })
    }
    // Source: src/core/SkImageFilter_Base.h:200-200
    // Source: src/core/SkImageFilter.cpp:383-386
    /// Safety: the child inherits Context's real source/backend draw leases.
    unsafe fn getChildOutput(
        &self,
        index: usize,
        ctx: &super::SkImageFilterTypes::Context<'_>,
    ) -> Result<super::SkImageFilterTypes::FilterResult, crate::RasterError> {
        match self.getInput(index) {
            Some(input) => input.filterImage(ctx),
            None => Ok(ctx.source().clone()),
        }
    }
}
// Source: src/core/SkImageFilter.cpp:138-146
fn next_image_filter_unique_id() -> u32 {
    use std::sync::atomic::{AtomicI32, Ordering};
    static NEXT_ID: AtomicI32 = AtomicI32::new(1);
    loop {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        if id != 0 {
            return id as u32;
        }
    }
}
// Source: src/core/SkImageFilter.cpp:148-161
impl SkImageFilter_Base {
    pub fn new(inputs: &[Option<Arc<dyn SkImageFilter>>], usesSrc: Option<bool>) -> Self {
        let mut base = Self {
            fInputs: super::SkImageFilter_Base::FilterInputs::new(inputs.len()),
            fUsesSrcInput: usesSrc.unwrap_or(false),
            fUniqueID: next_image_filter_unique_id(),
        };
        for (index, input) in inputs.iter().enumerate() {
            if usesSrc.is_none() && input.as_ref().map_or(true, |i| i.base().usesSource()) {
                base.fUsesSrcInput = true;
            }
            base.fInputs[index] = input.clone();
        }
        base
    }
}

// Source: src/core/SkImageFilter_Base.h:177-177
// Source: src/core/SkImageFilter.cpp:163-165
// Native base destruction purges this exact base-subobject identity before
// input fields are destroyed. Get's default is kYes, so a missing cache is
// initialized even on first filter destruction, just as in the pinned source.
impl Drop for SkImageFilter_Base {
    fn drop(&mut self) {
        let cache = super::SkImageFilterCache::SkImageFilterCache::Get(
            super::SkImageFilterCache::CreateIfNecessary::kYes,
        )
        .expect("native Get(kYes) retains the global cache");
        cache.purge_by_filter_identity(self as *const Self as usize);
    }
}

// Source: src/core/SkImageFilter_Base.h:143-144
// Source: src/core/SkImageFilter.cpp:167-175
impl SkImageFilter_Base {
    pub fn Unflatten(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> Result<(Option<Arc<dyn SkImageFilter>>, Option<SkRect>), crate::RasterError> {
        let mut common = crate::src::core::SkImageFilter_Base::Common::default();
        Ok(if !common.unflatten(buffer, 1)? {
            (None, None)
        } else {
            (common.getInput(0), common.cropRect())
        })
    }
}
// Source: src/core/SkImageFilter.cpp:177-218
impl crate::src::core::SkImageFilter_Base::Common {
    pub fn unflatten(
        &mut self,
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
        expected_count: i32,
    ) -> Result<bool, crate::RasterError> {
        let count = buffer.readInt();
        if !buffer.validate(count >= 0) {
            return Ok(false);
        }
        if !buffer.validate(expected_count < 0 || count == expected_count) {
            return Ok(false);
        }
        #[cfg(SK_BUILD_FOR_FUZZER)]
        if count > 4 {
            return Ok(false);
        }
        crate::SkASSERT!(@source b"src/core/SkImageFilter.cpp\0".as_ptr(),192; @condition b"fInputs.empty()\0".as_ptr(); self.fInputs.is_empty());
        for _ in 0..count {
            self.fInputs.push(if buffer.readBool() {
                buffer.readImageFilter()?
            } else {
                None
            });
            if !buffer.isValid() {
                return Ok(false);
            }
        }
        if buffer.isVersionLT(super::SkPicturePriv::kRemoveDeprecatedCropRect) {
            let rect = buffer.readRect();
            if !buffer.isValid() || !buffer.validate(super::SkValidationUtils::SkIsValidRect(&rect))
            {
                return Ok(false);
            }
            let flags = buffer.readUInt();
            if !buffer.isValid() || !buffer.validate(flags == 0 || flags == 0x0f) {
                return Ok(false);
            }
            if flags == 0x0f {
                self.fCropRect = Some(rect);
            }
        }
        Ok(buffer.isValid())
    }
}
// Source: src/core/SkImageFilter_Base.h:179-179
// Source: src/core/SkImageFilter.cpp:220-229
impl SkImageFilter_Base {
    pub fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeInt(self.fInputs.len() as i32);
        for i in 0..self.fInputs.len() {
            let input = self.fInputs[i].as_ref();
            buffer.writeBool(input.is_some());
            if let Some(input) = input {
                buffer.writeFlattenable(Some(&**input))?;
            }
        }

        Ok(())
    }
}
impl SkImageFilter_Base {
    // Source: src/core/SkImageFilter_Base.h:207-207
    // Source: src/core/SkImageFilter.cpp:388-393
    pub fn PurgeCache() {
        if let Some(cache) = super::SkImageFilterCache::SkImageFilterCache::Get(
            super::SkImageFilterCache::CreateIfNecessary::kNo,
        ) {
            cache.purge();
        }
    }
}

// Source: include/core/SkImageFilter.h:99-99
// Source: src/core/SkImageFilter.cpp:130-132
// Borrowing the Arc handle expresses native refMe() without inventing a second owner or self-reference.
pub fn makeWithLocalMatrix(
    filter: &Arc<dyn SkImageFilter>,
    matrix: &super::SkMatrix::SkMatrix,
) -> Option<Arc<dyn SkImageFilter>> {
    super::SkLocalMatrixImageFilter::Make(matrix, Some(refMe(filter)))
}

// Source: include/core/SkImageFilter.h:101-105
// Transfer the factory's actual Arc ImageFilter owner; never reconstruct an
// intrusive counter or downcast a borrowed trait object into a second owner.
pub fn Deserialize(
    data: &[u8],
    procs: Option<&crate::include::core::SkSerialProcs::SkDeserialProcs>,
) -> Result<Option<Arc<dyn SkImageFilter>>, crate::RasterError> {
    match super::SkFlattenable::Deserialize(
        super::SkFlattenable::Type::kSkImageFilter_Type,
        data,
        procs,
    )? {
        None => Ok(None),
        Some(super::SkFlattenable::FlattenableOwner::ImageFilter(owner)) => Ok(Some(owner)),
        Some(_) => {
            unreachable!("ImageFilter factory must return its actual typed ImageFilter owner")
        }
    }
}
// Source: include/core/SkImageFilter.h:109-111
// The original retained owner is explicit because &self cannot reconstruct Arc.
pub fn refMe(owner: &Arc<dyn SkImageFilter>) -> Arc<dyn SkImageFilter> {
    Arc::clone(owner)
}

// Source: src/core/SkImageFilter.cpp:82-96
// Borrowed qualified-base call adapter: derived overrides can invoke the same
// native base algorithm without redispatching their own virtual override.
// Extracted from the existing trait default with zero repeated source credit.
pub(crate) fn compute_fast_bounds_base<F: SkImageFilter + ?Sized>(
    filter: &F,
    src: &SkRect,
) -> Result<SkRect, crate::RasterError> {
    Ok({
        if filter.countInputs() == 0 {
            return Ok(*src);
        }
        let mut combined = match filter.getInput(0) {
            Some(input) => input.computeFastBounds(src)?,
            None => *src,
        };
        for i in 1..filter.countInputs() {
            if let Some(input) = filter.getInput(i) {
                combined.join(&input.computeFastBounds(src)?);
            } else {
                combined.join(src);
            }
        }
        combined
    })
}
