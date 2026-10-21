// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::src::core::SkBlitter::SkBlitterStorage;
use crate::src::core::SkPixmap::SkPixmap;
use crate::src::core::SkSpriteBlitter::SkSpriteBlitter;
// Source: src/core/SkBlitter_Sprite.cpp:39-40
impl SkSpriteBlitter {
    /// Safety: source storage must remain readable throughout all sprite calls.
    pub unsafe fn new(source: &SkPixmap) -> Self {
        // Native fDst/fPaint/offsets are unusable until setup. Rust initializes
        // those slots without creating a usable sprite or a fake paint owner.
        Self {
            fDst: SkPixmap::default(),
            fSource: source.clone(),
            fLeft: 0,
            fTop: 0,
            fPaint: core::ptr::null(),
            base: SkBlitterStorage::new(),
        }
    }
    // Source: src/core/SkBlitter_Sprite.cpp:42-48
    /// Safety: paint and destination storage must remain live while this sprite
    /// uses them. Destination storage must be writable and valid at addressed pixels.
    pub unsafe fn setup(
        &mut self,
        dst: &SkPixmap,
        left: i32,
        top: i32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> bool {
        self.fDst = dst.clone();
        self.fLeft = left;
        self.fTop = top;
        self.fPaint = paint;
        true
    }
}

// Source: src/core/SkBlitter_Sprite.cpp:50-55
pub(crate) unsafe fn sprite_blit_h<B: crate::src::core::SkBlitter::SkBlitter + ?Sized>(
    mut blitter: core::pin::Pin<&mut B>,
    x: i32,
    y: i32,
    width: i32,
) {
    crate::SkDEBUGFAIL!(@source b"src/core/SkBlitter_Sprite.cpp\0".as_ptr(), 51; b"how did we get here?\0".as_ptr());
    blitter.as_mut().blitRect(x, y, width, 1);
}
// Source: src/core/SkBlitter_Sprite.cpp:57-61
pub(crate) unsafe fn sprite_blit_anti_h() {
    crate::SkDEBUGFAIL!(@source b"src/core/SkBlitter_Sprite.cpp\0".as_ptr(), 58; b"how did we get here?\0".as_ptr());
    // Native release behavior deliberately has no fallback strategy.
}

// Source: src/core/SkBlitter_Sprite.cpp:63-68
pub(crate) unsafe fn sprite_blit_v<B: crate::src::core::SkBlitter::SkBlitter + ?Sized>(
    mut blitter: core::pin::Pin<&mut B>,
    x: i32,
    y: i32,
    height: i32,
    alpha: u8,
) {
    crate::SkDEBUGFAIL!(@source b"src/core/SkBlitter_Sprite.cpp\0".as_ptr(), 64; b"how did we get here?\0".as_ptr());
    crate::src::core::SkBlitter::blitV(blitter, x, y, height, alpha);
}
// Source: src/core/SkBlitter_Sprite.cpp:70-75
pub(crate) unsafe fn sprite_blit_mask<B: crate::src::core::SkBlitter::SkBlitter + ?Sized>(
    mut blitter: core::pin::Pin<&mut B>,
    mask: &crate::src::core::SkMask::SkMask,
    clip: &crate::src::core::SkRect::SkIRect,
) {
    crate::SkDEBUGFAIL!(@source b"src/core/SkBlitter_Sprite.cpp\0".as_ptr(), 71; b"how did we get here?\0".as_ptr());
    crate::src::core::SkBlitter::blitMask(blitter, mask, clip);
}

// Source: src/core/SkBlitter_Sprite.cpp:79-96
pub struct SkSpriteBlitter_Memcpy {
    pub(crate) sprite: SkSpriteBlitter,
}
impl SkSpriteBlitter_Memcpy {
    pub fn Supports(
        dst: &SkPixmap,
        src: &SkPixmap,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> bool {
        use crate::src::core::SkBlendMode::SkBlendMode;
        debug_assert_eq!(
            crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps::new(
                src.colorSpace(),
                src.alphaType(),
                dst.colorSpace(),
                dst.alphaType()
            )
            .fFlags
            .mask(),
            0
        );
        if dst.colorType() != src.colorType() {
            return false;
        }
        if paint.getMaskFilter().is_some()
            || paint.getColorFilter().is_some()
            || paint.getImageFilter().is_some()
        {
            return false;
        }
        if paint.getAlpha() != 0xff {
            return false;
        }
        let mode = paint.asBlendMode();
        mode == Some(SkBlendMode::kSrc) || (mode == Some(SkBlendMode::kSrcOver) && src.isOpaque())
    }
    // Source: src/core/SkBlitter_Sprite.cpp:98-99
    /// Safety: source pixel storage must remain readable for all sprite calls.
    pub unsafe fn new(src: &SkPixmap) -> Self {
        Self {
            sprite: SkSpriteBlitter::new(src),
        }
    }
    // Source: src/core/SkBlitter_Sprite.cpp:101-116
    /// Safety: setup must have supplied valid destination and offsets. Every
    /// source/destination row must cover width pixels and the copied byte ranges
    /// must not overlap, matching native memcpy's requirement.
    pub unsafe fn blitRect(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        width: i32,
        mut height: i32,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        debug_assert_eq!(
            this.sprite.fDst.colorType(),
            this.sprite.fSource.colorType()
        );
        crate::SkASSERT!(@source b"src/core/SkBlitter_Sprite.cpp\0".as_ptr(), 103; @condition b"width > 0 && height > 0\0".as_ptr(); width > 0 && height > 0);
        let mut dst = this.sprite.fDst.writable_addr_xy(x, y);
        let mut src = this
            .sprite
            .fSource
            .addr_xy(x - this.sprite.fLeft, y - this.sprite.fTop);
        let dstRB = this.sprite.fDst.rowBytes();
        let srcRB = this.sprite.fSource.rowBytes();
        let bytesToCopy = (width as usize) << this.sprite.fSource.shiftPerPixel();
        loop {
            let old = height;
            height -= 1;
            if old <= 0 {
                break;
            }
            core::ptr::copy_nonoverlapping(src, dst, bytesToCopy);
            dst = dst.wrapping_add(dstRB);
            src = src.wrapping_add(srcRB);
        }
    }
}

// Rust inheritance forwarding only; native rectangle/setup algorithms are above.
unsafe impl crate::src::core::SkBlitter::SkBlitter for SkSpriteBlitter_Memcpy {
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.sprite.base.allocBlitMemory(sz)
    }
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        sprite_blit_h(core::pin::Pin::new_unchecked(&mut *this), x, y, width)
    }
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, height: i32, alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        sprite_blit_v(
            core::pin::Pin::new_unchecked(&mut *this),
            x,
            y,
            height,
            alpha,
        )
    }
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        sprite_blit_mask(core::pin::Pin::new_unchecked(&mut *this), mask, clip)
    }
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        _: i32,
        _: i32,
        _: &mut [u8],
        _: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        sprite_blit_anti_h()
    }
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32, height: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        SkSpriteBlitter_Memcpy::blitRect(
            core::pin::Pin::new_unchecked(&mut *this),
            x,
            y,
            width,
            height,
        )
    }
}
impl crate::src::core::SkSpriteBlitter::SkSpriteBlitterInterface for SkSpriteBlitter_Memcpy {
    // Source: src/core/SkSpriteBlitter.h:33-33
    // Same real complete object's existing rectangle override, without another
    // pixel loop, owner, base-default fallback or erased-vtable cast.
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32, height: i32) {
        <Self as crate::src::core::SkBlitter::SkBlitter>::blitRect(self, x, y, width, height);
    }

    unsafe fn setup(
        self: core::pin::Pin<&mut Self>,
        dst: &SkPixmap,
        left: i32,
        top: i32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> bool {
        self.get_unchecked_mut().sprite.setup(dst, left, top, paint)
    }
}

impl<'arena> SkRasterPipelineSpriteBlitter<'arena> {
    // Source: src/core/SkBlitter_Sprite.cpp:124-131
    /// Safety: source backing pixels and alloc must live throughout all sprite
    /// calls. Pin the result in its final arena address before calling setup.
    pub unsafe fn new(
        src: &SkPixmap,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        clipShader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    ) -> Self {
        Self {
            sprite: SkSpriteBlitter::new(src),
            fAlloc: alloc,
            fBlitter: None,
            fSrcPtr: crate::src::core::SkRasterPipelineOpContexts::MemoryCtx {
                pixels: core::ptr::null_mut(),
                stride: 0,
            },
            fClipShader: clipShader,
            fPaintColor: core::mem::MaybeUninit::uninit(),
            _pin: core::marker::PhantomPinned,
        }
    }
    // Source: src/core/SkBlitter_Sprite.cpp:133-167
    /// Safety: the destination backing pixels, stable self contexts, and arena
    /// must remain valid through all uses of the resulting pipeline blitter.
    /// Its raw contexts require exclusive synchronized pixel execution.
    pub unsafe fn setup(
        self: core::pin::Pin<&mut Self>,
        dst: &SkPixmap,
        left: i32,
        top: i32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<bool, crate::RasterError> {
        use crate::include::core::SkAlphaType::SkAlphaType::{
            kOpaque_SkAlphaType, kPremul_SkAlphaType,
        };
        use crate::src::core::SkImageInfoPriv::SkColorTypeIsAlphaOnly;
        use crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp as Op;
        let this = self.get_unchecked_mut();
        this.sprite.fDst = dst.clone();
        this.sprite.fLeft = left;
        this.sprite.fTop = top;
        let paintColor = core::ptr::addr_of_mut!(this.fPaintColor)
            .cast::<crate::src::core::SkColor::SkColor4f>();
        paintColor.write(paint.getColor4f());
        let mut p = crate::src::core::SkRasterPipeline::SkRasterPipeline::new(this.fAlloc);
        p.appendLoad(
            this.sprite.fSource.colorType(),
            core::ptr::addr_of_mut!(this.fSrcPtr),
        );
        if SkColorTypeIsAlphaOnly(this.sprite.fSource.colorType()) {
            p.appendSetRGB(
                this.fAlloc,
                &[(*paintColor).fR, (*paintColor).fG, (*paintColor).fB],
            );
            p.append(Op::premul, core::ptr::null_mut());
        }
        if let Some(dstCS) = this.sprite.fDst.colorSpace() {
            let mut srcCS = this.sprite.fSource.colorSpace();
            if srcCS.is_none() || SkColorTypeIsAlphaOnly(this.sprite.fSource.colorType()) {
                srcCS = Some(crate::src::core::SkColorSpace::sk_srgb_singleton().as_ref());
            }
            let srcAT = if this.sprite.fSource.isOpaque() {
                kOpaque_SkAlphaType
            } else {
                kPremul_SkAlphaType
            };
            let steps = (*this.fAlloc).make(
                crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps::new(
                    srcCS,
                    srcAT,
                    Some(dstCS),
                    kPremul_SkAlphaType,
                ),
            );
            (*steps).applyPipeline(&mut p);
        }
        if (*paintColor).fA != 1.0 {
            p.append(
                Op::scale_1_float,
                core::ptr::addr_of_mut!((*paintColor).fA).cast(),
            );
        }
        let is_opaque = this.sprite.fSource.isOpaque() && (*paintColor).fA == 1.0;
        this.fBlitter =
            crate::src::core::SkRasterPipelineBlitter::SkCreateRasterPipelineBlitter_from_pipeline(
                &this.sprite.fDst,
                paint,
                &p,
                is_opaque,
                &mut *this.fAlloc,
                this.fClipShader.clone(),
            )?;
        Ok(this.fBlitter.is_some())
    }
    // Source: src/core/SkBlitter_Sprite.cpp:169-181
    /// Safety: successful setup and all source/destination coordinate bounds
    /// must hold, with self and its contexts pinned throughout execution.
    pub unsafe fn blitRect(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        let this = self.get_unchecked_mut();
        let src_ctx = core::ptr::addr_of_mut!(this.fSrcPtr);
        core::ptr::addr_of_mut!((*src_ctx).stride).write(this.sprite.fSource.rowBytesAsPixels());
        let bpp = this.sprite.fSource.info().bytesPerPixel() as usize;
        let pixels = this
            .sprite
            .fSource
            .writable_addr_xy(-this.sprite.fLeft + x, -this.sprite.fTop + y)
            .wrapping_sub(bpp.wrapping_mul(x as usize))
            .wrapping_sub(
                bpp.wrapping_mul(y as usize)
                    .wrapping_mul((*src_ctx).stride as usize),
            )
            .cast();
        core::ptr::addr_of_mut!((*src_ctx).pixels).write(pixels);
        let mut blitter = this.fBlitter.expect("successful setup required");
        core::pin::Pin::new_unchecked(blitter.as_mut()).blitRect(x, y, width, height);
    }
}
// Source: src/core/SkBlitter_Sprite.cpp:184-188
// Rust reverses native member declaration order to retain C++ reverse teardown.
// Pin keeps source MemoryCtx and paint-alpha addresses stable. The native paint
// color is uninitialized until setup writes it, and is never read beforehand.
pub struct SkRasterPipelineSpriteBlitter<'arena> {
    _pin: core::marker::PhantomPinned,
    fClipShader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    fPaintColor: core::mem::MaybeUninit<crate::src::core::SkColor::SkColor4f>,
    fSrcPtr: crate::src::core::SkRasterPipelineOpContexts::MemoryCtx,
    fBlitter: Option<core::ptr::NonNull<dyn crate::src::core::SkBlitter::SkBlitter + 'arena>>,
    fAlloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    sprite: SkSpriteBlitter,
}

// Rust inheritance adaptation for the actually completed PipelineSprite. It is
// constructed only after setup returned Ok(true); the underlying native object
// stays pinned in its original arena and retains the validated pipeline blitter.
struct PreparedPipelineSprite<'arena> {
    _pin: core::marker::PhantomPinned,
    sprite: core::ptr::NonNull<SkRasterPipelineSpriteBlitter<'arena>>,
}
unsafe impl crate::src::core::SkBlitter::SkBlitter for PreparedPipelineSprite<'_> {
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { this.sprite.as_mut().sprite.base.allocBlitMemory(sz) }
    }
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        sprite_blit_h(core::pin::Pin::new_unchecked(&mut *this), x, y, width)
    }
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, height: i32, alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        sprite_blit_v(
            core::pin::Pin::new_unchecked(&mut *this),
            x,
            y,
            height,
            alpha,
        )
    }
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        sprite_blit_mask(core::pin::Pin::new_unchecked(&mut *this), mask, clip)
    }
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        _: i32,
        _: i32,
        _: &mut [u8],
        _: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        sprite_blit_anti_h()
    }
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32, height: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        core::pin::Pin::new_unchecked(this.sprite.as_mut()).blitRect(x, y, width, height)
    }
}

// Native local SkSpriteBlitter* becomes an enum solely because the actual Rust
// setup signature is fallible/pinned for the pipeline subclass. No state is
// cloned and no alternative algorithm or neighboring sprite class is supplied.
enum SpriteSelection<'arena> {
    Direct(
        core::ptr::NonNull<
            dyn crate::src::core::SkSpriteBlitter::SkSpriteBlitterInterface + 'arena,
        >,
    ),
    Pipeline(core::ptr::NonNull<SkRasterPipelineSpriteBlitter<'arena>>),
}

// Source: src/core/SkBlitter_Sprite.cpp:195-241
/// Safety: destination/source pixel storage, retained paint/shader contexts and
/// arena must remain valid through every use; returned objects stay pinned and
/// mutable calls must be exclusive. All later coordinates obey native bounds.
pub unsafe fn ChooseSprite<'arena>(
    dst: &SkPixmap,
    paint: &crate::src::core::SkPaint::SkPaint,
    source: &SkPixmap,
    left: i32,
    top: i32,
    alloc: core::pin::Pin<&'arena mut crate::src::core::SkArenaAlloc::SkArenaAlloc>,
    clipShader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
) -> Result<
    Option<core::ptr::NonNull<dyn crate::src::core::SkBlitter::SkBlitter + 'arena>>,
    crate::RasterError,
> {
    // Real arena pin is consumed into native raw access; allocate only through
    // short synchronous reborrows, never retaining &mut across setup callbacks.
    let ap = alloc.get_unchecked_mut() as *mut crate::src::core::SkArenaAlloc::SkArenaAlloc;
    use crate::include::core::SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType;
    use crate::src::core::SkSpriteBlitter::SkSpriteBlitterInterface;
    if source.alphaType() == kUnpremul_SkAlphaType {
        return Ok(None);
    }
    let mut blitter: Option<SpriteSelection<'arena>> = None;
    if crate::src::core::SkBlitter::gSkForceRasterPipelineBlitter
        .load(core::sync::atomic::Ordering::Relaxed)
    {
        // Native force switch bypasses all optimized memory blitters.
    } else if crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps::new(
        source.colorSpace(),
        source.alphaType(),
        dst.colorSpace(),
        dst.alphaType(),
    )
    .fFlags
    .mask()
        == 0
        && clipShader.is_none()
    {
        if blitter.is_none() && SkSpriteBlitter_Memcpy::Supports(dst, source, paint) {
            let ptr = (*ap).make(SkSpriteBlitter_Memcpy::new(source));
            blitter = Some(SpriteSelection::Direct(core::ptr::NonNull::new_unchecked(
                ptr as *mut (dyn SkSpriteBlitterInterface + 'arena),
            )));
        }
        if blitter.is_none() {
            match dst.colorType() {
                crate::include::core::SkColorType::kN32_SkColorType => {
                    if let Some(ptr) = SkSpriteBlitter::ChooseL32(source, paint, &mut *ap) {
                        blitter = Some(SpriteSelection::Direct(core::ptr::NonNull::new_unchecked(
                            ptr.as_ptr() as *mut (dyn SkSpriteBlitterInterface + 'arena),
                        )));
                    }
                }
                _ => {}
            }
        }
    }
    if blitter.is_none() && paint.getMaskFilter().is_none() {
        // Actual make_with preserves borrowed arena lifetime; safe make<T>
        // requires 'static and cannot represent the native retained contexts.
        // Complete address-independent header/Arc construction before the arena
        // registers a destructor footer. Setup pins the final placement below.
        let value = SkRasterPipelineSpriteBlitter::new(source, ap, clipShader);
        let ptr = (*ap).make_with(|_, _| value);
        blitter = Some(SpriteSelection::Pipeline(
            core::ptr::NonNull::new_unchecked(ptr),
        ));
    }
    match blitter {
        Some(SpriteSelection::Direct(mut ptr)) => {
            if core::pin::Pin::new_unchecked(ptr.as_mut()).setup(dst, left, top, paint) {
                Ok(Some(core::ptr::NonNull::new_unchecked(ptr.as_ptr()
                    as *mut (dyn crate::src::core::SkBlitter::SkBlitter
                         + 'arena))))
            } else {
                Ok(None)
            }
        }
        Some(SpriteSelection::Pipeline(mut sprite)) => {
            if core::pin::Pin::new_unchecked(sprite.as_mut()).setup(dst, left, top, paint)? {
                let adapter = (*ap).make_with(|_, _| PreparedPipelineSprite {
                    sprite,
                    _pin: core::marker::PhantomPinned,
                });
                Ok(Some(core::ptr::NonNull::new_unchecked(
                    adapter as *mut (dyn crate::src::core::SkBlitter::SkBlitter + 'arena),
                )))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}
