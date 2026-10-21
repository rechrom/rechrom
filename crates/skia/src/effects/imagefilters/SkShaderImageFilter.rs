#![allow(non_snake_case)]
use crate::include::core::SkTileMode::SkTileMode;
use crate::src::core::{
    SkImageFilter::SkImageFilter,
    SkImageFilterTypes::{LayerSpace, Mapping, MatrixCapability},
    SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
    SkRect::{SkIRect, SkRect},
};
use crate::src::shaders::SkShader::SkShader;
use std::sync::Arc;
// Source: src/effects/imagefilters/SkShaderImageFilter.cpp:64-65
// Boolean projection of native Dither enum, not a substitute shader or rendering backend.
struct SkShaderImageFilter {
    fShader: Arc<dyn SkShader>,
    fDither: bool,
    base: SkImageFilter_Base,
}
// Source: src/effects/imagefilters/SkShaderImageFilter.cpp:31-36
impl SkShaderImageFilter {
    fn new(shader: Arc<dyn SkShader>, dither: bool) -> Self {
        Self {
            base: SkImageFilter_Base::new(&[], None),
            fShader: shader,
            fDither: dither,
        }
    }
}
// Source: src/effects/imagefilters/SkShaderImageFilter.cpp:70-82
pub fn Shader(
    shader: Option<Arc<dyn SkShader>>,
    dither: bool,
    crop: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    let Some(shader) = shader else {
        return super::SkCropImageFilter::Empty();
    };
    let filter: Arc<dyn SkImageFilter> = Arc::new(SkShaderImageFilter::new(shader, dither));
    if let Some(rect) = crop {
        super::SkCropImageFilter::Crop(rect, SkTileMode::kDecal, Some(filter))
    } else {
        Some(filter)
    }
}
// Source: src/effects/imagefilters/SkShaderImageFilter.cpp:47-47
impl crate::src::core::SkFlattenable::SkFlattenableView for SkShaderImageFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkShaderImageFilter")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkImageFilter_Type
    }
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(writer)?;
        Ok(())
    }
}
// Source: src/effects/imagefilters/SkShaderImageFilter.cpp:84-89
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterShaderImageFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkShaderImageFilter",
            SkShaderImageFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkPaintImageFilter",
            SkShaderImageFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkPaintImageFilterImpl",
            SkShaderImageFilter::CreateProc,
        );
    }
}
// Source: src/effects/imagefilters/SkShaderImageFilter.cpp:91-109
impl SkShaderImageFilter {
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut common = crate::src::core::SkImageFilter_Base::Common::default();
        if !common.unflatten(buffer, 0)? {
            return Ok(None);
        }
        let (shader, dither) = if buffer
            .isVersionLT(crate::src::core::SkPicturePriv::kShaderImageFilterSerializeShader)
        {
            let paint = buffer.readPaint()?;
            let shader = if paint.getShader().is_some() {
                paint.refShader()
            } else {
                crate::src::shaders::SkColorShader::Color4f(&paint.getColor4f(), None)
            };
            (shader, paint.isDither())
        } else {
            let shader = buffer.readShader()?;
            (shader, buffer.readBool())
        };
        Ok(Shader(shader, dither, common.cropRect())
            .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkShaderImageFilter.cpp:111-115
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        buffer.writeFlattenable(Some(&*self.fShader))?;
        buffer.writeBool(self.fDither);
        Ok(())
    }
}
impl SkImageFilterBaseHooks for SkShaderImageFilter {
    // Source: src/effects/imagefilters/SkShaderImageFilter.cpp:49-49
    fn onAffectsTransparentBlack(&self) -> Result<bool, crate::RasterError> {
        Ok({ true })
    }
    // Source: src/effects/imagefilters/SkShaderImageFilter.cpp:51-51
    fn onGetCTMCapability(&self) -> MatrixCapability {
        MatrixCapability::kComplex
    }
    // Source: src/effects/imagefilters/SkShaderImageFilter.cpp:53-53
    // Rust virtual forwarding preserves the existing real native implementation.
    unsafe fn onFilterImage(
        &self,
        ctx: &crate::src::core::SkImageFilterTypes::Context<'_>,
    ) -> Result<crate::src::core::SkImageFilterTypes::FilterResult, crate::RasterError> {
        SkShaderImageFilter::onFilterImage(self, ctx)
    }
    // Source: src/effects/imagefilters/SkShaderImageFilter.cpp:123-129
    fn onGetInputLayerBounds(
        &self,
        _: &Mapping,
        _: &LayerSpace<SkIRect>,
        _: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        Ok({ LayerSpace::<SkIRect>::Empty() })
    }
    // Source: src/effects/imagefilters/SkShaderImageFilter.cpp:131-138
    fn onGetOutputLayerBounds(
        &self,
        _: &Mapping,
        _: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        Ok({ LayerSpace::<SkIRect>::Unbounded() })
    }
}
impl SkImageFilter for SkShaderImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/effects/imagefilters/SkShaderImageFilter.cpp:38-40
    fn computeFastBounds(&self, _: &SkRect) -> Result<SkRect, crate::RasterError> {
        Ok({ crate::src::core::SkRectPriv::SkRectPriv::MakeLargeS32() })
    }
}

impl SkShaderImageFilter {
    // Source: src/effects/imagefilters/SkShaderImageFilter.cpp:119-122
    /// Safety: retain shader backing and the native Backend offscreen/COW lease
    /// for this synchronous draw. The canonical DAG dispatch forwards to this actual body.
    unsafe fn onFilterImage(
        &self,
        ctx: &crate::src::core::SkImageFilterTypes::Context<'_>,
    ) -> Result<crate::src::core::SkImageFilterTypes::FilterResult, crate::RasterError> {
        let dither = self.fDither;
        unsafe {
            crate::src::core::SkImageFilterTypes::FilterResult::MakeFromShader(
                ctx,
                self.fShader.clone(),
                dither,
            )
        }
    }
}
