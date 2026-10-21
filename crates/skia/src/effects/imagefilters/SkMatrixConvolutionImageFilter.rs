// Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.h:22-22
pub const kLargeKernelSize: i32 = 256;
// Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.h:27-27
pub const kSmallKernelSize: i32 = 64;
// Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.h:31-31
pub const kMaxUniformKernelSize: i32 = 28;

use crate::include::core::{
    SkSamplingOptions::{SkFilterMode, SkSamplingOptions},
    SkSize::SkISize,
    SkTileMode::SkTileMode,
};
use crate::src::core::{
    SkBitmap::SkBitmap,
    SkFlattenable::{Factory, FactoryResult, FlattenableOwner, SkFlattenableView, Type},
    SkImageFilter::SkImageFilter,
    SkImageFilterTypes::{
        Builder, Context, FilterResult, IVector, LayerSpace, Mapping, ShaderFlags,
    },
    SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
    SkKnownRuntimeEffects::{GetKnownRuntimeEffect, StableKey},
    SkM44::SkV2,
    SkPoint::SkIPoint,
    SkReadBuffer::SkReadBuffer,
    SkRect::{SkIRect, SkRect},
    SkRuntimeEffect::{ChildPtr, SkRuntimeEffectBuilder},
    SkWriteBuffer::SkWriteBuffer,
};
use crate::src::shaders::SkShader::SkShader;
use std::sync::Arc;
// Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:52-56
const _: () = assert!(kLargeKernelSize % 4 == 0);
const _: () = assert!(kSmallKernelSize <= kLargeKernelSize);
const _: () = assert!(kMaxUniformKernelSize % 4 == 0);
// Native fields retain declaration order; Vec substitutes the owning TArray.
// Derived owners are destroyed before base input owners, as in the native class.
// Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:127-143
// Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:61-61
pub struct SkMatrixConvolutionImageFilter {
    fKernel: core::mem::ManuallyDrop<Vec<f32>>,
    fKernelSize: LayerSpace<SkISize>,
    fKernelOffset: LayerSpace<IVector>,
    fGain: f32,
    fBias: f32,
    fConvolveAlpha: bool,
    fKernelBitmap: core::mem::ManuallyDrop<SkBitmap>,
    fInnerBias: f32,
    fInnerGain: f32,
    base: SkImageFilter_Base,
}
// Native reverse derived-member teardown precedes inherited input teardown.
// ManuallyDrop adds no state; guard destruction also retains order on Rust unwind.
impl Drop for SkMatrixConvolutionImageFilter {
    fn drop(&mut self) {
        struct Owner<'a, T>(&'a mut core::mem::ManuallyDrop<T>);
        impl<T> Drop for Owner<'_, T> {
            fn drop(&mut self) {
                unsafe {
                    core::mem::ManuallyDrop::drop(self.0);
                }
            }
        }
        drop((Owner(&mut self.fKernelBitmap), Owner(&mut self.fKernel)));
    }
}
// Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:63-81
impl SkMatrixConvolutionImageFilter {
    fn new(
        size: SkISize,
        kernel: &[f32],
        gain: f32,
        bias: f32,
        offset: SkIPoint,
        alpha: bool,
        input: Option<Arc<dyn SkImageFilter>>,
    ) -> Self {
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp\0".as_ptr(),74; @condition b"SkSafeMath::Mul(kernelSize.fWidth, kernelSize.fHeight) <= kLargeKernelSize\0".as_ptr(); (size.fWidth as usize).saturating_mul(size.fHeight as usize)<=kLargeKernelSize as usize);
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp\0".as_ptr(),75; @condition b"kernelSize.fWidth >= 1 && kernelSize.fHeight >= 1\0".as_ptr(); size.fWidth>=1&&size.fHeight>=1);
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp\0".as_ptr(),76; @condition b"kernelOffset.fX >= 0 && kernelOffset.fX < kernelSize.fWidth\0".as_ptr(); offset.fX>=0&&offset.fX<size.fWidth);
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp\0".as_ptr(),77; @condition b"kernelOffset.fY >= 0 && kernelOffset.fY < kernelSize.fHeight\0".as_ptr(); offset.fY>=0&&offset.fY<size.fHeight);
        let base = SkImageFilter_Base::new(&[input], None);
        let fKernel = kernel[..(size.fWidth * size.fHeight) as usize].to_vec();
        let mut innerGain = core::mem::MaybeUninit::<f32>::uninit();
        let mut innerBias = core::mem::MaybeUninit::<f32>::uninit();
        let bitmap = unsafe {
            create_kernel_bitmap(size, kernel, innerGain.as_mut_ptr(), innerBias.as_mut_ptr())
        };
        Self {
            fKernel: core::mem::ManuallyDrop::new(fKernel),
            fKernelSize: LayerSpace::new(size),
            fKernelOffset: LayerSpace::new(IVector::new(offset.fX, offset.fY)),
            fGain: gain,
            fBias: bias,
            fConvolveAlpha: alpha,
            fKernelBitmap: core::mem::ManuallyDrop::new(bitmap),
            fInnerBias: unsafe { innerBias.assume_init() },
            fInnerGain: unsafe { innerGain.assume_init() },
            base,
        }
    }
}
// Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:148-153
fn adjust(rect: &LayerSpace<SkIRect>, dl: i32, dt: i32, dr: i32, db: i32) -> LayerSpace<SkIRect> {
    let mut adjusted = *rect.geometry();
    adjusted.adjust(dl, dt, dr, db);
    LayerSpace::new(adjusted)
}
// Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:155-163
fn quantize_by_kernel_size(size: i32) -> (i32, StableKey) {
    if size < kMaxUniformKernelSize {
        (kMaxUniformKernelSize, StableKey::kMatrixConvUniforms)
    } else if size <= kSmallKernelSize {
        (kSmallKernelSize, StableKey::kMatrixConvTexSm)
    } else {
        (kLargeKernelSize, StableKey::kMatrixConvTexLg)
    }
}
// Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:165-224
unsafe fn create_kernel_bitmap(
    size: SkISize,
    kernel: &[f32],
    innerGain: *mut f32,
    innerBias: *mut f32,
) -> SkBitmap {
    let length = size.fWidth * size.fHeight;
    let (quantized, key) = quantize_by_kernel_size(length);
    if key == StableKey::kMatrixConvUniforms {
        innerGain.write(1.0);
        innerBias.write(0.0);
        return SkBitmap::default();
    }
    let mut min = kernel[0];
    let mut max = kernel[0];
    for i in 1..length as usize {
        if kernel[i] < min {
            min = kernel[i];
        }
        if kernel[i] > max {
            max = kernel[i];
        }
    }
    innerGain.write(max - min);
    innerBias.write(min);
    if crate::src::core::SkScalar::SkScalarNearlyZero(
        *innerGain,
        crate::src::core::SkScalar::SK_ScalarNearlyZero,
    ) {
        innerGain.write(1.0);
    }
    let mut bitmap = SkBitmap::default();
    let info = crate::src::core::SkImageInfo::SkImageInfo::Make(
        quantized,
        1,
        crate::include::core::SkColorType::SkColorType::kAlpha_8_SkColorType,
        crate::include::core::SkAlphaType::SkAlphaType::kPremul_SkAlphaType,
    );
    if !bitmap.tryAllocPixels_info(&info) {
        return SkBitmap::default();
    }
    for i in 0..length {
        bitmap
            .getAddr8(i, 0)
            .write(crate::src::core::SkScalar::SkScalarRoundToInt(
                255.0 * (kernel[i as usize] - min) / *innerGain,
            ) as u8);
    }
    for i in length..quantized {
        bitmap.getAddr8(i, 0).write(0);
    }
    bitmap.setImmutable();
    bitmap
}
// Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:228-267
/// The optional slice expresses native nullable kernel; a present slice contains
/// at least the validated kernel area. No substitute kernel is manufactured.
pub fn MatrixConvolution(
    size: SkISize,
    kernel: Option<&[f32]>,
    gain: f32,
    bias: f32,
    offset: SkIPoint,
    tile: SkTileMode,
    alpha: bool,
    input: Option<Arc<dyn SkImageFilter>>,
    crop: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    if size.fWidth < 1 || size.fHeight < 1 {
        return None;
    }
    if (size.fWidth as usize).saturating_mul(size.fHeight as usize) > kLargeKernelSize as usize {
        return None;
    }
    let kernel = kernel?;
    assert!(
        kernel.len() >= (size.fWidth * size.fHeight) as usize,
        "native kernel array must contain its full area"
    );
    if offset.fX < 0 || offset.fX >= size.fWidth || offset.fY < 0 || offset.fY >= size.fHeight {
        return None;
    }
    let mut filter = input;
    if let Some(crop) = crop {
        if tile != SkTileMode::kDecal {
            filter = super::SkCropImageFilter::Crop(crop, tile, filter);
        }
    }
    filter = Some(Arc::new(SkMatrixConvolutionImageFilter::new(
        size, kernel, gain, bias, offset, alpha, filter,
    )));
    if let Some(crop) = crop {
        filter = super::SkCropImageFilter::Crop(crop, SkTileMode::kDecal, filter);
    }
    filter
}
// Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:269-274
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterMatrixConvolutionImageFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkMatrixConvolutionImageFilter",
            SkMatrixConvolutionImageFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkMatrixConvolutionImageFilterImpl",
            SkMatrixConvolutionImageFilter::CreateProc,
        );
    }
}
impl SkMatrixConvolutionImageFilter {
    // Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:276-317
    pub fn CreateProc(buffer: &mut SkReadBuffer<'_>) -> FactoryResult {
        let mut common = crate::src::core::SkImageFilter_Base::Common::default();
        if !common.unflatten(buffer, 1)? {
            return Ok(None);
        }
        let size = SkISize::Make(buffer.readInt(), buffer.readInt());
        let count = buffer.getArrayCount();
        let area = size.fWidth as i64 * size.fHeight as i64;
        if !buffer.validate(area == count as i64) {
            return Ok(None);
        }
        if !buffer.validateCanReadN::<f32>(count as usize) {
            return Ok(None);
        }
        let mut kernel = vec![0.0; count as usize];
        if !buffer.readScalarArray(&mut kernel) {
            return Ok(None);
        }
        let gain = buffer.readScalar();
        let bias = buffer.readScalar();
        let offset = SkIPoint::make(buffer.readInt(), buffer.readInt());
        let tile = if buffer
            .isVersionLT(crate::src::core::SkPicturePriv::kConvolutionImageFilterTilingUpdate)
        {
            let raw = buffer.read32LE(SkTileMode::kLastTileMode as u32);
            unsafe { core::mem::transmute::<u32, SkTileMode>(raw) }
        } else {
            SkTileMode::kDecal
        };
        let alpha = buffer.readBool();
        if !buffer.isValid() {
            return Ok(None);
        }
        Ok(MatrixConvolution(
            size,
            Some(&kernel),
            gain,
            bias,
            offset,
            tile,
            alpha,
            common.getInput(0),
            common.cropRect(),
        )
        .map(FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:319-329
    fn flatten_impl(&self, buffer: &mut dyn SkWriteBuffer) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        buffer.writeInt(self.fKernelSize.width());
        buffer.writeInt(self.fKernelSize.height());
        buffer.writeScalarArray(&self.fKernel);
        buffer.writeScalar(self.fGain);
        buffer.writeScalar(self.fBias);
        buffer.writeInt(self.fKernelOffset.x());
        buffer.writeInt(self.fKernelOffset.y());
        buffer.writeBool(self.fConvolveAlpha);
        Ok(())
    }
    // Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:333-340
    fn boundsSampledByKernel(&self, bounds: &LayerSpace<SkIRect>) -> LayerSpace<SkIRect> {
        adjust(
            bounds,
            -self.fKernelOffset.x(),
            -self.fKernelOffset.y(),
            self.fKernelSize.width() - self.fKernelOffset.x() - 1,
            self.fKernelSize.height() - self.fKernelOffset.y() - 1,
        )
    }
    // Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:342-349
    fn boundsAffectedByKernel(&self, bounds: &LayerSpace<SkIRect>) -> LayerSpace<SkIRect> {
        adjust(
            bounds,
            self.fKernelOffset.x() - self.fKernelSize.width() + 1,
            self.fKernelOffset.y() - self.fKernelSize.height() + 1,
            self.fKernelOffset.x(),
            self.fKernelOffset.y(),
        )
    }
    // Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:353-389
    unsafe fn createShader(
        &self,
        ctx: &Context<'_>,
        input: Option<Arc<dyn SkShader>>,
    ) -> Option<Arc<dyn SkShader>> {
        let length = self.fKernelSize.width() * self.fKernelSize.height();
        let (_, key) = quantize_by_kernel_size(length);
        let texture = key != StableKey::kMatrixConvUniforms;
        if texture && self.fKernelBitmap.empty() {
            return None;
        }
        let effect = GetKnownRuntimeEffect(key)
            .expect("native known convolution effect")
            .clone();
        let mut builder = SkRuntimeEffectBuilder::new(effect);
        builder
            .child(b"child")
            .assign(input.map(ChildPtr::Shader).unwrap_or(ChildPtr::Empty));
        if texture {
            let image = ctx.backend().getCachedBitmap(&self.fKernelBitmap)?;
            let shader = image
                .makeRawShader_default(&SkSamplingOptions::with_filter(SkFilterMode::kNearest));
            builder
                .child(b"kernel")
                .assign(shader.map(ChildPtr::Shader).unwrap_or(ChildPtr::Empty));
            builder.uniform(b"innerGainAndBias").assign(&SkV2 {
                x: self.fInnerGain,
                y: self.fInnerBias,
            });
        } else {
            let mut padded = [0.0; kMaxUniformKernelSize as usize];
            padded[..length as usize].copy_from_slice(&self.fKernel);
            builder
                .uniform(b"kernel")
                .set(padded.as_ptr(), kMaxUniformKernelSize);
        }
        builder.uniform(b"size").assign(self.fKernelSize.geometry());
        // The native int2 uniform is two contiguous initialized ints; this array
        // projects IVector without assuming Rust's non-repr(C) struct layout.
        builder
            .uniform(b"offset")
            .assign(&[self.fKernelOffset.x(), self.fKernelOffset.y()]);
        builder.uniform(b"gainAndBias").assign(&SkV2 {
            x: self.fGain,
            y: self.fBias / 255.0,
        });
        builder
            .uniform(b"convolveAlpha")
            .assign(&(if self.fConvolveAlpha { 1i32 } else { 0i32 }));
        builder.makeShader(None)
    }
}
// Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:90-90
impl SkFlattenableView for SkMatrixConvolutionImageFilter {
    fn getFactory(&self) -> Option<Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkMatrixConvolutionImageFilter")
    }
    fn getFlattenableType(&self) -> Type {
        Type::kSkImageFilter_Type
    }
    fn flatten(&self, b: &mut dyn SkWriteBuffer) -> Result<(), crate::RasterError> {
        self.flatten_impl(b)?;
        Ok(())
    }
}
impl SkImageFilterBaseHooks for SkMatrixConvolutionImageFilter {
    // Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:92-106
    fn onAffectsTransparentBlack(&self) -> Result<bool, crate::RasterError> {
        Ok(true)
    }
    // Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:391-419
    unsafe fn onFilterImage(
        &self,
        context: &Context<'_>,
    ) -> Result<FilterResult, crate::RasterError> {
        let required = self.boundsSampledByKernel(context.desiredOutput());
        let child = self.getChildOutput(0, &context.withNewDesiredOutput(required))?;
        let bounds = if self.fConvolveAlpha && self.fBias != 0.0 {
            *context.desiredOutput()
        } else {
            let mut bounds = self.boundsAffectedByKernel(&child.layerBounds());
            if !bounds.intersect(context.desiredOutput()) {
                return Ok(FilterResult::default());
            }
            bounds
        };
        let mut builder = Builder::new(context);
        builder.add_sampled(
            child,
            Some(self.boundsSampledByKernel(&bounds)),
            ShaderFlags::kSampledRepeatedly,
            FilterResult::kDefaultSampling,
        );
        builder.eval(
            |inputs| Ok(self.createShader(context, inputs[0].clone())),
            Some(bounds),
        )
    }
    // Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:421-429
    fn onGetInputLayerBounds(
        &self,
        mapping: &Mapping,
        desired: &LayerSpace<SkIRect>,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        self.getChildInputLayerBounds(0, mapping, &self.boundsSampledByKernel(desired), content)
    }
    // Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:431-450
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        if self.fConvolveAlpha && self.fBias != 0.0 {
            return Ok(LayerSpace::<SkIRect>::Unbounded());
        }
        Ok(self
            .getChildOutputLayerBounds(0, mapping, content)?
            .map(|b| self.boundsAffectedByKernel(&b)))
    }
}
impl SkImageFilter for SkMatrixConvolutionImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp:452-457
    fn computeFastBounds(&self, _bounds: &SkRect) -> Result<SkRect, crate::RasterError> {
        Ok(crate::src::core::SkRectPriv::SkRectPriv::MakeLargeS32())
    }
}
