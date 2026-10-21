// Copyright 2013 Google Inc. BSD-style license. UNVERIFIED native translation.
use crate::{
    include::core::{SkAlphaType::SkAlphaType, SkColorType::SkColorType, SkSize::SkISize},
    src::{
        core::{
            SkBitmap::SkBitmap,
            SkColor::SkColor4f,
            SkEffectPriv::SkStageRec,
            SkImageInfo::SkImageInfo,
            SkMatrix::SkMatrix,
            SkPoint::SkPoint,
            SkRasterPipelineOpContexts::PerlinNoiseCtx,
            SkRasterPipelineOpList::SkRasterPipelineOp,
            SkScalar::{
                SkScalarCeilToScalar, SkScalarFloorToScalar, SkScalarRoundToInt, SkScalarTruncToInt,
            },
        },
        shaders::{
            SkColorShader,
            SkPerlinNoiseShaderType::SkPerlinNoiseShaderType,
            SkShader::SkShader,
            SkShaderBase::{MatrixRec, ShaderType, SkShaderBase},
        },
    },
};
use core::{marker::PhantomPinned, mem::ManuallyDrop, pin::Pin};
use std::sync::{Arc, OnceLock};
// Source: src/shaders/SkPerlinNoiseShaderImpl.h:36-39
const kBlockSize: usize = 256;
const kBlockMask: usize = kBlockSize - 1;
const kPerlinNoise: i32 = 4096;
const kRandMaximum: i32 = i32::MAX;
// Source: src/shaders/SkPerlinNoiseShaderImpl.h:42-43
// Source: src/shaders/SkPerlinNoiseShaderImpl.h:56-59
#[derive(Clone, Copy, Debug, Default)]
pub struct StitchData {
    pub fWidth: i32,
    pub fWrapX: i32,
    pub fHeight: i32,
    pub fWrapY: i32,
}
impl StitchData {
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:45-49
    pub fn new(w: f32, h: f32) -> Self {
        let width = SkScalarRoundToInt(w).min(i32::MAX - kPerlinNoise);
        let height = SkScalarRoundToInt(h).min(i32::MAX - kPerlinNoise);
        Self {
            fWidth: width,
            fWrapX: kPerlinNoise + width,
            fHeight: height,
            fWrapY: kPerlinNoise + height,
        }
    }
}
// Source: src/shaders/SkPerlinNoiseShaderImpl.h:51-54
impl PartialEq for StitchData {
    fn eq(&self, other: &Self) -> bool {
        self.fWidth == other.fWidth
            && self.fWrapX == other.fWrapX
            && self.fHeight == other.fHeight
            && self.fWrapY == other.fWrapY
    }
}
impl Eq for StitchData {}
// Source: src/shaders/SkPerlinNoiseShaderImpl.h:62-62
// Source: src/shaders/SkPerlinNoiseShaderImpl.h:97-106
// Rust contiguous native field group. Pin retains self-backed bitmap addresses;
// explicit teardown destroys the two bitmap references before their data arrays.
pub struct PaintingData {
    pub fSeed: i32,
    pub fLatticeSelector: [u8; kBlockSize],
    pub fNoise: [[[u16; 2]; kBlockSize]; 4],
    pub fTileSize: SkISize,
    pub fBaseFrequency: SkPoint,
    pub fStitchDataInit: StitchData,
    fPermutationsBitmap: ManuallyDrop<SkBitmap>,
    fNoiseBitmap: ManuallyDrop<SkBitmap>,
    _pin: PhantomPinned,
}
// Native painting data is immutable once published by SkOnce. Mutating bitmap
// installation requires the exclusive pinned owner; no mutable array projection
// is exposed from the shader's shared OnceLock. Bitmap pointers borrow this same
// stable backing allocation and never point to a second noise owner.
unsafe impl Send for PaintingData {}
unsafe impl Sync for PaintingData {}
impl PaintingData {
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:63-74
    pub fn new(tileSize: &SkISize, seed: f32, x: f32, y: f32) -> Pin<Box<Self>> {
        // Native arrays/seed are fully overwritten by init before publication; Rust
        // initializes their construction storage so no uninitialized reference exists.
        let mut data = Box::pin(Self {
            fSeed: 0,
            fLatticeSelector: [0; kBlockSize],
            fNoise: [[[0; 2]; kBlockSize]; 4],
            fTileSize: SkISize::Make(0, 0),
            fBaseFrequency: SkPoint::Make(0.0, 0.0),
            fStitchDataInit: StitchData::default(),
            fPermutationsBitmap: ManuallyDrop::new(SkBitmap::default()),
            fNoiseBitmap: ManuallyDrop::new(SkBitmap::default()),
            _pin: PhantomPinned,
        });
        let p = unsafe { data.as_mut().get_unchecked_mut() };
        p.fBaseFrequency.set(x, y);
        p.fTileSize.set(
            SkScalarRoundToInt(tileSize.fWidth as f32),
            SkScalarRoundToInt(tileSize.fHeight as f32),
        );
        p.init(seed);
        if !p.fTileSize.isEmpty() {
            p.stitch();
        }
        data
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:76-84
    /// Safety: no bitmap reader aliases the exclusive pixel installation interval.
    pub unsafe fn generateBitmaps(self: Pin<&mut Self>) {
        let p = self.get_unchecked_mut();
        let info = SkImageInfo::MakeA8(kBlockSize as i32, 1);
        p.fPermutationsBitmap.installPixels_no_release(
            &info,
            p.fLatticeSelector.as_mut_ptr().cast(),
            info.minRowBytes(),
        );
        p.fPermutationsBitmap.setImmutable();
        let info = SkImageInfo::Make(
            kBlockSize as i32,
            4,
            SkColorType::kRGBA_8888_SkColorType,
            SkAlphaType::kPremul_SkAlphaType,
        );
        p.fNoiseBitmap.installPixels_no_release(
            &info,
            p.fNoise.as_mut_ptr().cast(),
            info.minRowBytes(),
        );
        p.fNoiseBitmap.setImmutable();
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:86-95
    /// Safety: native bitmap copy retains the original bitmap pixel references,
    /// whose external bytes still belong to `that`; keep that allocation alive for
    /// every bitmap read on the copy. Noise arrays themselves are copied by value.
    pub unsafe fn copy_native(that: &Self) -> Pin<Box<Self>> {
        Box::pin(Self {
            fSeed: that.fSeed,
            fLatticeSelector: that.fLatticeSelector,
            fNoise: that.fNoise,
            fTileSize: that.fTileSize,
            fBaseFrequency: that.fBaseFrequency,
            fStitchDataInit: that.fStitchDataInit,
            fPermutationsBitmap: ManuallyDrop::new((*that.fPermutationsBitmap).clone()),
            fNoiseBitmap: ManuallyDrop::new((*that.fNoiseBitmap).clone()),
            _pin: PhantomPinned,
        })
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:108-121
    fn random(&mut self) -> i32 {
        const kRandAmplitude: i32 = 16807;
        const kRandQ: i32 = 127773;
        const kRandR: i32 = 2836;
        let mut result = kRandAmplitude * (self.fSeed % kRandQ) - kRandR * (self.fSeed / kRandQ);
        if result <= 0 {
            result += kRandMaximum;
        }
        self.fSeed = result;
        result
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:124-187
    fn init(&mut self, seed: f32) {
        self.fSeed = SkScalarTruncToInt(seed);
        if self.fSeed <= 0 {
            self.fSeed = -(self.fSeed % (kRandMaximum - 1)) + 1;
        }
        if self.fSeed > kRandMaximum - 1 {
            self.fSeed = kRandMaximum - 1;
        }
        for channel in 0..4 {
            for i in 0..kBlockSize {
                self.fLatticeSelector[i] = i as u8;
                self.fNoise[channel][i][0] = (self.random() % (2 * kBlockSize as i32)) as u16;
                self.fNoise[channel][i][1] = (self.random() % (2 * kBlockSize as i32)) as u16;
            }
        }
        for i in (1..kBlockSize).rev() {
            let k = self.fLatticeSelector[i];
            let j = self.random() % kBlockSize as i32;
            crate::SkASSERT!(@source b"src/shaders/SkPerlinNoiseShaderImpl.h\0".as_ptr(),144; @condition b"j >= 0\0".as_ptr(); j>=0);
            crate::SkASSERT!(@source b"src/shaders/SkPerlinNoiseShaderImpl.h\0".as_ptr(),145; @condition b"j < kBlockSize\0".as_ptr(); j<kBlockSize as i32);
            self.fLatticeSelector[i] = self.fLatticeSelector[j as usize];
            self.fLatticeSelector[j as usize] = k;
        }
        {
            let mut noise = [[[0u16; 2]; kBlockSize]; 4];
            for i in 0..kBlockSize {
                for channel in 0..4 {
                    for j in 0..2 {
                        noise[channel][i][j] = self.fNoise[channel][i][j];
                    }
                }
            }
            for i in 0..kBlockSize {
                for channel in 0..4 {
                    for j in 0..2 {
                        self.fNoise[channel][i][j] =
                            noise[channel][self.fLatticeSelector[i] as usize][j];
                    }
                }
            }
        }
        const kHalfMax16bits: f32 = 32767.5;
        const kInvBlockSizef: f32 = 1.0 / kBlockSize as f32;
        for channel in 0..4 {
            for i in 0..kBlockSize {
                let mut gradient = SkPoint::Make(
                    (self.fNoise[channel][i][0] as i32 - kBlockSize as i32) as f32 * kInvBlockSizef,
                    (self.fNoise[channel][i][1] as i32 - kBlockSize as i32) as f32 * kInvBlockSizef,
                );
                gradient.normalize();
                self.fNoise[channel][i][0] =
                    SkScalarRoundToInt((gradient.fX + 1.0) * kHalfMax16bits) as u16;
                self.fNoise[channel][i][1] =
                    SkScalarRoundToInt((gradient.fY + 1.0) * kHalfMax16bits) as u16;
            }
        }
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:190-225
    fn stitch(&mut self) {
        let tileWidth = self.fTileSize.width() as f32;
        let tileHeight = self.fTileSize.height() as f32;
        crate::SkASSERT!(@source b"src/shaders/SkPerlinNoiseShaderImpl.h\0".as_ptr(),193; @condition b"tileWidth > 0 && tileHeight > 0\0".as_ptr(); tileWidth>0.0&&tileHeight>0.0);
        if self.fBaseFrequency.fX != 0.0 {
            let low = SkScalarFloorToScalar(tileWidth * self.fBaseFrequency.fX) / tileWidth;
            let high = SkScalarCeilToScalar(tileWidth * self.fBaseFrequency.fX) / tileWidth;
            if self.fBaseFrequency.fX / low < high / self.fBaseFrequency.fX {
                self.fBaseFrequency.fX = low;
            } else {
                self.fBaseFrequency.fX = high;
            }
        }
        if self.fBaseFrequency.fY != 0.0 {
            let low = SkScalarFloorToScalar(tileHeight * self.fBaseFrequency.fY) / tileHeight;
            let high = SkScalarCeilToScalar(tileHeight * self.fBaseFrequency.fY) / tileHeight;
            if self.fBaseFrequency.fY / low < high / self.fBaseFrequency.fY {
                self.fBaseFrequency.fY = low;
            } else {
                self.fBaseFrequency.fY = high;
            }
        }
        self.fStitchDataInit = StitchData::new(
            tileWidth * self.fBaseFrequency.fX,
            tileHeight * self.fBaseFrequency.fY,
        );
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:228-231
    pub fn getPermutationsBitmap(&self) -> &SkBitmap {
        crate::SkASSERT!(@source b"src/shaders/SkPerlinNoiseShaderImpl.h\0".as_ptr(),229; @condition b"!fPermutationsBitmap.drawsNothing()\0".as_ptr(); !self.fPermutationsBitmap.drawsNothing());
        &self.fPermutationsBitmap
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:232-235
    pub fn getNoiseBitmap(&self) -> &SkBitmap {
        crate::SkASSERT!(@source b"src/shaders/SkPerlinNoiseShaderImpl.h\0".as_ptr(),233; @condition b"!fNoiseBitmap.drawsNothing()\0".as_ptr(); !self.fNoiseBitmap.drawsNothing());
        &self.fNoiseBitmap
    }
}
impl Drop for PaintingData {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fNoiseBitmap);
            ManuallyDrop::drop(&mut self.fPermutationsBitmap);
        }
    }
}
// Uncredited lexical bridge for native class-scoped constant references.
const kMaxOctaves: i32 = SkPerlinNoiseShader::kMaxOctaves;
// Source: src/shaders/SkPerlinNoiseShaderImpl.h:34-34
// Source: src/shaders/SkPerlinNoiseShaderImpl.h:267-276
// OnceLock combines the native once flag and its unique PaintingData owner;
// it publishes exactly one stable allocation and retains it through shader drop.
pub struct SkPerlinNoiseShader {
    fType: SkPerlinNoiseShaderType,
    fBaseFrequencyX: f32,
    fBaseFrequencyY: f32,
    fNumOctaves: i32,
    fSeed: f32,
    fTileSize: SkISize,
    fStitchTiles: bool,
    fPaintingData: OnceLock<Pin<Box<PaintingData>>>,
}
impl SkPerlinNoiseShader {
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:238-238
    pub const kMaxOctaves: i32 = 255;
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:247-247
    fn header_type(&self) -> ShaderType {
        ShaderType::kPerlinNoise
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:248-248
    fn header_isOpaque(&self) -> bool {
        false
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:250-250
    pub fn noiseType(&self) -> SkPerlinNoiseShaderType {
        self.fType
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:251-251
    pub fn numOctaves(&self) -> i32 {
        self.fNumOctaves
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:252-252
    pub fn stitchTiles(&self) -> bool {
        self.fStitchTiles
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:253-253
    pub fn tileSize(&self) -> SkISize {
        self.fTileSize
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.h:255-257
    pub fn getPaintingData(&self) -> Pin<Box<PaintingData>> {
        PaintingData::new(
            &self.fTileSize,
            self.fSeed,
            self.fBaseFrequencyX,
            self.fBaseFrequencyY,
        )
    }
}
// Source: src/shaders/SkPerlinNoiseShaderImpl.h:265-265
impl crate::src::core::SkFlattenable::SkFlattenableView for SkPerlinNoiseShader {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkPerlinNoiseShader")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkShader_Type
    }
    fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(buffer)?;
        Ok(())
    }
}
impl SkPerlinNoiseShader {
    // Source: src/shaders/SkPerlinNoiseShaderImpl.cpp:26-47
    pub fn new(
        kind: SkPerlinNoiseShaderType,
        x: f32,
        y: f32,
        octaves: i32,
        seed: f32,
        tile: Option<&SkISize>,
    ) -> Self {
        let size = tile.copied().unwrap_or(SkISize::Make(0, 0));
        let result = Self {
            fType: kind,
            fBaseFrequencyX: x,
            fBaseFrequencyY: y,
            fNumOctaves: if octaves > kMaxOctaves {
                kMaxOctaves
            } else {
                octaves
            },
            fSeed: seed,
            fTileSize: size,
            fStitchTiles: !size.isEmpty(),
            fPaintingData: OnceLock::new(),
        };
        crate::SkASSERT!(@source b"src/shaders/SkPerlinNoiseShaderImpl.cpp\0".as_ptr(),40; @condition b"numOctaves >= 0 && numOctaves <= kMaxOctaves\0".as_ptr(); octaves>=0&&octaves<=kMaxOctaves);
        crate::SkASSERT!(@source b"src/shaders/SkPerlinNoiseShaderImpl.cpp\0".as_ptr(),41; @condition b"fBaseFrequencyX >= 0\0".as_ptr(); x>=0.0);
        crate::SkASSERT!(@source b"src/shaders/SkPerlinNoiseShaderImpl.cpp\0".as_ptr(),42; @condition b"fBaseFrequencyY >= 0\0".as_ptr(); y>=0.0);
        const _: () = assert!(kBlockSize == 256);
        result
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.cpp:49-71
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let kind = buffer.read32LE(SkPerlinNoiseShaderType::kLast as u32);
        let x = buffer.readScalar();
        let y = buffer.readScalar();
        let octaves = buffer.read32LE(kMaxOctaves as u32) as i32;
        let seed = buffer.readScalar();
        let tile = SkISize::Make(buffer.readInt(), buffer.readInt());
        let shader = match kind {
            0 => MakeFractalNoise(x, y, octaves, seed, Some(&tile)),
            1 => MakeTurbulence(x, y, octaves, seed, Some(&tile)),
            _ => {
                buffer.validate(false);
                None
            }
        };
        Ok(shader.map(crate::src::core::SkFlattenable::FlattenableOwner::Shader))
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.cpp:73-81
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeInt(self.fType as i32);
        buffer.writeScalar(self.fBaseFrequencyX);
        buffer.writeScalar(self.fBaseFrequencyY);
        buffer.writeInt(self.fNumOctaves);
        buffer.writeScalar(self.fSeed);
        buffer.writeInt(self.fTileSize.fWidth);
        buffer.writeInt(self.fTileSize.fHeight);
        Ok(())
    }
}
impl SkShaderBase for SkPerlinNoiseShader {
    // Native inline body is retained above; this is virtual forwarding only.
    fn r#type(&self) -> ShaderType {
        self.header_type()
    }
    // Source: src/shaders/SkPerlinNoiseShaderImpl.cpp:83-107
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        mRec: &MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        let newMRec = mRec.apply(rec, &SkMatrix::default());
        if newMRec.is_none() {
            return Ok(false);
        }
        let data = self.fPaintingData.get_or_init(|| self.getPaintingData());
        let ctx = (*rec.fAlloc).make(PerlinNoiseCtx {
            noiseType: self.fType,
            baseFrequencyX: data.fBaseFrequency.fX,
            baseFrequencyY: data.fBaseFrequency.fY,
            stitchDataInX: data.fStitchDataInit.fWidth as f32,
            stitchDataInY: data.fStitchDataInit.fHeight as f32,
            stitching: self.fStitchTiles,
            numOctaves: self.fNumOctaves,
            latticeSelector: data.fLatticeSelector.as_ptr(),
            noiseData: data.fNoise.as_ptr().cast(),
        });
        (*rec.fPipeline).append(SkRasterPipelineOp::perlin_noise, ctx.cast());
        Ok(true)
    }
}
impl SkShader for SkPerlinNoiseShader {
    // Native inline body is retained above; this is virtual forwarding only.
    fn isOpaque(&self) -> bool {
        self.header_isOpaque()
    }
}
// Source: src/shaders/SkPerlinNoiseShaderImpl.cpp:111-126
fn valid_input(x: f32, y: f32, octaves: i32, tile: Option<&SkISize>, seed: f32) -> bool {
    if !(x >= 0.0 && y >= 0.0) {
        return false;
    }
    if !(octaves >= 0 && octaves <= kMaxOctaves) {
        return false;
    }
    if tile.is_some_and(|tile| !(tile.width() >= 0 && tile.height() >= 0)) {
        return false;
    }
    if !seed.is_finite() {
        return false;
    }
    true
}
// Source: src/shaders/SkPerlinNoiseShaderImpl.cpp:128-132
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterPerlinNoiseShaderFlattenable() {
    unsafe {
        use crate::src::core::SkFlattenable::Register;
        Register(b"SkPerlinNoiseShader", SkPerlinNoiseShader::CreateProc);
        Register(b"SkPerlinNoiseShaderImpl", SkPerlinNoiseShader::CreateProc);
    }
}
// Source: src/shaders/SkPerlinNoiseShaderImpl.cpp:135-158
// Source: include/effects/SkPerlinNoiseShader.h:45-47
pub fn MakeFractalNoise(
    x: f32,
    y: f32,
    octaves: i32,
    seed: f32,
    tile: Option<&SkISize>,
) -> Option<Arc<dyn SkShader>> {
    if !valid_input(x, y, octaves, tile, seed) {
        return None;
    }
    if octaves == 0 {
        return SkColorShader::Color4f(
            &SkColor4f {
                fR: 0.5,
                fG: 0.5,
                fB: 0.5,
                fA: 0.5,
            },
            None,
        );
    }
    Some(Arc::new(SkPerlinNoiseShader::new(
        SkPerlinNoiseShaderType::kFractalNoise,
        x,
        y,
        octaves,
        seed,
        tile,
    )))
}
// Source: src/shaders/SkPerlinNoiseShaderImpl.cpp:160-180
// Source: include/effects/SkPerlinNoiseShader.h:48-50
pub fn MakeTurbulence(
    x: f32,
    y: f32,
    octaves: i32,
    seed: f32,
    tile: Option<&SkISize>,
) -> Option<Arc<dyn SkShader>> {
    if !valid_input(x, y, octaves, tile, seed) {
        return None;
    }
    if octaves == 0 {
        return SkColorShader::Color4f(
            &SkColor4f {
                fR: 0.0,
                fG: 0.0,
                fB: 0.0,
                fA: 0.0,
            },
            None,
        );
    }
    Some(Arc::new(SkPerlinNoiseShader::new(
        SkPerlinNoiseShaderType::kTurbulence,
        x,
        y,
        octaves,
        seed,
        tile,
    )))
}
