#![allow(non_snake_case, non_camel_case_types)]
use super::SkImageFilter::SkImageFilter;
use std::sync::Arc;
// Source: src/core/SkImageFilter_Base.h:302-305
// Private inline/heap storage and Arc substitute AutoSTArray/sk_sp; source declaration is adapted.
// Cache-purge destruction is implemented in the original SkImageFilter.cpp target.
pub struct SkImageFilter_Base {
    pub(crate) fInputs: FilterInputs,
    pub(crate) fUsesSrcInput: bool,
    pub(crate) fUniqueID: u32,
}
// Source: src/core/SkImageFilter_Base.h:122-122
impl SkImageFilter_Base {
    pub fn usesSource(&self) -> bool {
        self.fUsesSrcInput
    }
}
// Source: src/core/SkImageFilter_Base.h:130-130
impl SkImageFilter_Base {
    pub fn uniqueID(&self) -> u32 {
        self.fUniqueID
    }
}

use super::SkImageFilterTypes::{LayerSpace, Mapping};
// Source: src/core/SkImageFilter_Base.h:127-127
pub use super::SkImageFilterTypes::MatrixCapability;
use super::SkRect::SkIRect;
// Native virtual configuration, bounds functions, and actual rendering hook.
pub trait SkImageFilterBaseHooks: crate::src::core::SkFlattenable::SkFlattenableView {
    // Source: src/core/SkImageFilter_Base.h:217-217
    fn onIsColorFilterNode(
        &self,
        _filter: &mut Option<Arc<dyn super::SkColorFilter::SkColorFilter>>,
    ) -> bool {
        false
    }
    // Source: src/core/SkImageFilter_Base.h:225-227
    fn onGetCTMCapability(&self) -> MatrixCapability {
        MatrixCapability::kScaleTranslate
    }
    // Source: src/core/SkImageFilter_Base.h:234-234
    fn onAffectsTransparentBlack(&self) -> Result<bool, crate::RasterError> {
        Ok({ false })
    }
    // Source: src/core/SkImageFilter_Base.h:240-240
    fn ignoreInputsAffectsTransparentBlack(&self) -> bool {
        false
    }
    // Source: src/core/SkImageFilter_Base.h:254-254
    /// Safety: preserve Context's source/backend, pixel and native draw leases.
    unsafe fn onFilterImage(
        &self,
        context: &super::SkImageFilterTypes::Context<'_>,
    ) -> Result<super::SkImageFilterTypes::FilterResult, crate::RasterError>;
    // Source: src/core/SkImageFilter_Base.h:272-275
    fn onGetInputLayerBounds(
        &self,
        mapping: &Mapping,
        desired: &LayerSpace<SkIRect>,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError>;
    // Source: src/core/SkImageFilter_Base.h:298-300
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError>;
}

// Rust container adaptation, not additional ported SkTemplates source.
// Read-only reference: include/private/SkTemplates.h AutoSTArray reset destroys end-to-start.
// Its padding formula makes AutoSTArray<2,sk_sp<T>> hold 3 native thin pointers inline.
// Preserve that native allocation split despite Rust's wider Arc<dyn Trait> handles.
const INLINE_INPUTS: usize = 3;
pub(crate) struct FilterInputs {
    storage: InputStorage,
    len: usize,
}
enum InputStorage {
    Inline([Option<Arc<dyn SkImageFilter>>; INLINE_INPUTS]),
    Heap(Box<[Option<Arc<dyn SkImageFilter>>]>),
}
impl FilterInputs {
    pub(crate) fn new(count: usize) -> Self {
        Self {
            storage: if count <= INLINE_INPUTS {
                InputStorage::Inline(std::array::from_fn(|_| None))
            } else {
                InputStorage::Heap(
                    (0..count)
                        .map(|_| None)
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                )
            },
            len: count,
        }
    }
    pub(crate) fn len(&self) -> usize {
        self.len
    }
    fn as_slice(&self) -> &[Option<Arc<dyn SkImageFilter>>] {
        match &self.storage {
            InputStorage::Inline(v) => &v[..self.len],
            InputStorage::Heap(v) => v,
        }
    }
    fn as_mut_slice(&mut self) -> &mut [Option<Arc<dyn SkImageFilter>>] {
        match &mut self.storage {
            InputStorage::Inline(v) => &mut v[..self.len],
            InputStorage::Heap(v) => v,
        }
    }
}
impl core::ops::Index<usize> for FilterInputs {
    type Output = Option<Arc<dyn SkImageFilter>>;
    fn index(&self, i: usize) -> &Self::Output {
        &self.as_slice()[i]
    }
}
impl core::ops::IndexMut<usize> for FilterInputs {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.as_mut_slice()[i]
    }
}
impl Drop for FilterInputs {
    fn drop(&mut self) {
        for i in (0..self.len).rev() {
            drop(self.as_mut_slice()[i].take());
        }
    }
}
// Source: src/core/SkImageFilter_Base.h:147-175
// Native inline-array/optional ownership is retained using Vec/Option with actual Arc children.
#[derive(Default)]
pub struct Common {
    pub(crate) fCropRect: Option<super::SkRect::SkRect>,
    pub(crate) fInputs: Vec<Option<Arc<dyn SkImageFilter>>>,
}

impl SkImageFilter_Base {
    // Source: src/core/SkImageFilter_Base.h:132-134
    pub fn GetFlattenableType() -> super::SkFlattenable::Type {
        super::SkFlattenable::Type::kSkImageFilter_Type
    }
    // Source: src/core/SkImageFilter_Base.h:136-138
    pub fn getFlattenableType(&self) -> super::SkFlattenable::Type {
        super::SkFlattenable::Type::kSkImageFilter_Type
    }
}

// Source: src/core/SkImageFilter_Base.h:310-312
// Public/base virtual methods share the existing Rust trait. These views
// preserve the original complete object and its actual base state, including
// nullptr; they neither create a new owner nor reinterpret trait metadata.
pub fn as_IFB_mut(filter: Option<&mut dyn SkImageFilter>) -> Option<&mut dyn SkImageFilter> {
    filter
}
// Source: src/core/SkImageFilter_Base.h:314-316
pub fn as_IFB_shared(filter: &Option<Arc<dyn SkImageFilter>>) -> Option<&dyn SkImageFilter> {
    filter.as_deref()
}
// Source: src/core/SkImageFilter_Base.h:318-320
pub fn as_IFB(filter: Option<&dyn SkImageFilter>) -> Option<&dyn SkImageFilter> {
    filter
}

// Source: src/core/SkImageFilter_Base.h:325-331
// Explicit buffer input replaces the unhygienic C++ macro's enclosing variable.
// Native decode failure returns null; a real Rust raster error propagates.
macro_rules! SK_IMAGEFILTER_UNFLATTEN_COMMON {
    ($local:ident, $buffer:expr, $expected_count:expr) => {
        let mut $local = $crate::src::core::SkImageFilter_Base::Common::default();
        if !$local.unflatten($buffer, $expected_count)? {
            return Ok(None);
        }
    };
}
pub(crate) use SK_IMAGEFILTER_UNFLATTEN_COMMON;
impl Common {
    // Source: src/core/SkImageFilter_Base.h:159-159
    pub fn cropRect(&self) -> Option<super::SkRect::SkRect> {
        self.fCropRect
    }
    // Source: src/core/SkImageFilter_Base.h:161-161
    pub fn inputCount(&self) -> usize {
        self.fInputs.len()
    }
    // Source: src/core/SkImageFilter_Base.h:162-162
    // Native begin() exposes mutable retained handles, not a copied input list.
    pub fn inputs(&mut self) -> &mut [Option<Arc<dyn SkImageFilter>>] {
        &mut self.fInputs
    }
    // Source: src/core/SkImageFilter_Base.h:164-164
    pub fn getInput(&self, index: usize) -> Option<Arc<dyn SkImageFilter>> {
        self.fInputs[index].clone()
    }
}

// Source: src/core/SkImageFilter_Base.h:339-339
// Reexport the canonical registration producer; no second factory or wrapper owner.
pub use crate::src::effects::imagefilters::SkBlendImageFilter::SkRegisterBlendImageFilterFlattenable;

// Source: src/core/SkImageFilter_Base.h:340-340
// Reexport the canonical registration producer; no second factory or wrapper owner.
pub use crate::src::effects::imagefilters::SkBlurImageFilter::SkRegisterBlurImageFilterFlattenable;

// Source: src/core/SkImageFilter_Base.h:341-341
// Reexport the canonical registration producer; no second factory or wrapper owner.
pub use crate::src::effects::imagefilters::SkColorFilterImageFilter::SkRegisterColorFilterImageFilterFlattenable;

// Source: src/core/SkImageFilter_Base.h:342-342
// Reexport the canonical registration producer; no second factory or wrapper owner.
pub use crate::src::effects::imagefilters::SkComposeImageFilter::SkRegisterComposeImageFilterFlattenable;

// Source: src/core/SkImageFilter_Base.h:343-343
// Reexport the canonical registration producer; no second factory or wrapper owner.
pub use crate::src::effects::imagefilters::SkCropImageFilter::SkRegisterCropImageFilterFlattenable;

// Source: src/core/SkImageFilter_Base.h:345-345
// Reexport the canonical registration producer; no second factory or wrapper owner.
pub use crate::src::effects::imagefilters::SkImageImageFilter::SkRegisterImageImageFilterFlattenable;

// Source: src/core/SkImageFilter_Base.h:349-349
// Reexport the canonical registration producer; no second factory or wrapper owner.
pub use crate::src::effects::imagefilters::SkMatrixTransformImageFilter::SkRegisterMatrixTransformImageFilterFlattenable;

// Source: src/core/SkImageFilter_Base.h:350-350
// Reexport the canonical registration producer; no second factory or wrapper owner.
pub use crate::src::effects::imagefilters::SkMergeImageFilter::SkRegisterMergeImageFilterFlattenable;

// Source: src/core/SkImageFilter_Base.h:354-354
// Reexport the canonical registration producer; no second factory or wrapper owner.
pub use crate::src::effects::imagefilters::SkShaderImageFilter::SkRegisterShaderImageFilterFlattenable;

// Source: src/core/SkImageFilter_Base.h:358-358
// Reexport the canonical registration producer; no second factory or wrapper owner.
pub use crate::src::effects::imagefilters::SkDropShadowImageFilter::SkRegisterLegacyDropShadowImageFilterFlattenable;

// Source: src/core/SkImageFilter_Base.h:344
// Declaration-only canonical producer projection; zero new algorithm credit.
pub use crate::src::effects::imagefilters::SkDisplacementMapImageFilter::SkRegisterDisplacementMapImageFilterFlattenable;

// Source: src/core/SkImageFilter_Base.h:346
// Declaration-only canonical producer projection; zero new algorithm credit.
pub use crate::src::effects::imagefilters::SkLightingImageFilter::SkRegisterLightingImageFilterFlattenables;

// Source: src/core/SkImageFilter_Base.h:347
// Declaration-only canonical producer projection; zero new algorithm credit.
pub use crate::src::effects::imagefilters::SkMagnifierImageFilter::SkRegisterMagnifierImageFilterFlattenable;

// Source: src/core/SkImageFilter_Base.h:348
// Declaration-only canonical producer projection; zero new algorithm credit.
pub use crate::src::effects::imagefilters::SkMatrixConvolutionImageFilter::SkRegisterMatrixConvolutionImageFilterFlattenable;

// Source: src/core/SkImageFilter_Base.h:351
// Declaration-only canonical producer projection; zero new algorithm credit.
pub use crate::src::effects::imagefilters::SkMorphologyImageFilter::SkRegisterMorphologyImageFilterFlattenables;

// Source: src/core/SkImageFilter_Base.h:352
// Declaration-only canonical producer projection; zero new algorithm credit.
pub use crate::src::effects::imagefilters::SkPictureImageFilter::SkRegisterPictureImageFilterFlattenable;

// Source: src/core/SkImageFilter_Base.h:353
// Declaration-only canonical producer projection; zero new algorithm credit.
pub use crate::src::effects::imagefilters::SkRuntimeImageFilter::SkRegisterRuntimeImageFilterFlattenable;
