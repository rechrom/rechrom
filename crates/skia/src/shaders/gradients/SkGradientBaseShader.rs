// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2022 Google LLC. BSD-style license.
use crate::include::effects::SkGradient::ColorSpace;
use crate::include::private::SkTPin::SkTPin;
use crate::src::core::SkColor::SkColor4f;
use crate::src::core::SkColorData::SkPMColor4f;

use crate::include::core::SkTileMode::SkTileMode;
use crate::include::effects::SkGradient::{InPremul, Interpolation, SkGradient};
use crate::src::core::SkColorSpace::SkColorSpace;
use crate::src::core::SkMatrix::SkMatrix;
use std::sync::Arc;

// Source: src/shaders/gradients/SkGradientBaseShader.h:35-36
// Source: src/shaders/gradients/SkGradientBaseShader.h:39-41
// Native STArray inline optimization becomes retained contiguous owned buffers;
// returned descriptors borrow only this scope until their factory consumes them.
// Adapted private field order: native position allocation releases before color allocation.
pub struct SkGradientScope {
    // Source: src/shaders/gradients/SkGradientBaseShader.h:40-40
    fPositionStorage: Vec<f32>,
    // Source: src/shaders/gradients/SkGradientBaseShader.h:39-39
    fColorStorage: Vec<SkColor4f>,
}
impl SkGradientScope {
    pub fn new() -> Self {
        Self {
            fColorStorage: Vec::new(),
            fPositionStorage: Vec::new(),
        }
    }
}

// Source: src/shaders/gradients/SkGradientBaseShader.h:148-151
// AutoSTMalloc's inline-or-owned allocation is retained as one contiguous f32 backing store.
// Pointer fields become offsets so moving the Rust owner cannot invalidate inline references.
enum StopStorage {
    Inline([f32; 20]),
    Heap(Box<[f32]>),
}
impl StopStorage {
    fn new(floats: usize) -> Self {
        if floats <= 20 {
            Self::Inline([0.0; 20])
        } else {
            Self::Heap(vec![0.0; floats].into_boxed_slice())
        }
    }
    fn ptr(&self) -> *const f32 {
        match self {
            Self::Inline(x) => x.as_ptr(),
            Self::Heap(x) => x.as_ptr(),
        }
    }
    fn mut_ptr(&mut self) -> *mut f32 {
        match self {
            Self::Inline(x) => x.as_mut_ptr(),
            Self::Heap(x) => x.as_mut_ptr(),
        }
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.h:87-88
// Source: src/shaders/gradients/SkGradientBaseShader.h:134-140
// Source: src/shaders/gradients/SkGradientBaseShader.h:153-153
// Rust drops fields in declaration order: release stop storage before the retained color space,
// matching C++ reverse member destruction (storage follows colorSpace upstream).
pub struct SkGradientBaseShader {
    storage: StopStorage,
    fPtsToUnit: SkMatrix,
    fTileMode: SkTileMode,
    fColorSpace: Arc<SkColorSpace>,
    fInterpolation: Interpolation,
    fColorCount: usize,
    fFirstStopIsImplicit: bool,
    fLastStopIsImplicit: bool,
    fColorsAreOpaque: bool,
    position_offset: Option<usize>,
}
// Source: src/shaders/gradients/SkGradientBaseShader.h:50-62
// Source: src/shaders/gradients/SkGradientBaseShader.h:72-72
// Source: src/shaders/gradients/SkGradientBaseShader.h:104-127
impl SkGradientBaseShader {
    pub const kDegenerateThreshold: f32 = 1.0 / (1 << 15) as f32;
    pub fn r#type(&self) -> crate::src::shaders::SkShaderBase::ShaderType {
        crate::src::shaders::SkShaderBase::ShaderType::kGradientBase
    }
    pub fn interpolateInPremul(&self) -> bool {
        self.fInterpolation.fInPremul == InPremul::kYes
    }
    pub fn getGradientMatrix(&self) -> &SkMatrix {
        &self.fPtsToUnit
    }
    pub fn getColorCount(&self) -> usize {
        self.fColorCount
    }
    pub fn getPositions(&self) -> Option<&[f32]> {
        self.position_offset.map(|offset| unsafe {
            std::slice::from_raw_parts(self.storage.ptr().add(offset), self.fColorCount)
        })
    }
    pub fn getInterpolation(&self) -> &Interpolation {
        &self.fInterpolation
    }
    pub fn getPos(&self, i: usize) -> f32 {
        crate::SkASSERT!(@source b"src/shaders/gradients/SkGradientBaseShader.h\0".as_ptr(),105; @condition b"i < fColorCount\0".as_ptr(); i < self.fColorCount);
        match self.getPositions() {
            Some(p) => p[i],
            None => i as f32 / (self.fColorCount - 1) as f32,
        }
    }
    pub fn getLegacyColor(&self, i: usize) -> crate::src::core::SkColor::SkColor {
        crate::SkASSERT!(@source b"src/shaders/gradients/SkGradientBaseShader.h\0".as_ptr(),110; @condition b"i < fColorCount\0".as_ptr(); i < self.fColorCount);
        self.colors()[i].toSkColor()
    }
    pub fn colors(&self) -> &[SkColor4f] {
        unsafe { std::slice::from_raw_parts(self.storage.ptr().cast(), self.fColorCount) }
    }
    pub fn positions(&self) -> &[f32] {
        self.getPositions().unwrap_or(&[])
    }
    pub fn firstStopIsImplicit(&self) -> bool {
        self.fFirstStopIsImplicit
    }
    pub fn lastStopIsImplicit(&self) -> bool {
        self.fLastStopIsImplicit
    }
    pub fn colorSpace(&self) -> &Arc<SkColorSpace> {
        &self.fColorSpace
    }
    pub fn interpolation(&self) -> &Interpolation {
        &self.fInterpolation
    }
    pub fn colorsAreOpaque(&self) -> bool {
        self.fColorsAreOpaque
    }
    pub fn getTileMode(&self) -> SkTileMode {
        self.fTileMode
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.h:163-168
// STArray's four-element inline ownership is retained. Moving the owner cannot leave self pointers.
pub struct InlineStorage<T: Copy> {
    inline: [std::mem::MaybeUninit<T>; 4],
    heap: Option<Vec<T>>,
    len: usize,
}
impl<T: Copy> InlineStorage<T> {
    pub fn new() -> Self {
        Self {
            inline: [std::mem::MaybeUninit::uninit(); 4],
            heap: None,
            len: 0,
        }
    }
    pub fn from_slice(values: &[T]) -> Self {
        let mut result = Self::new();
        result.reserve_exact(values.len());
        for &v in values {
            result.push(v);
        }
        result
    }
    pub fn reserve_exact(&mut self, count: usize) {
        if let Some(heap) = &mut self.heap {
            if count > heap.capacity() {
                heap.reserve_exact(count - heap.len());
            }
        } else if count > 4 {
            let mut heap = Vec::with_capacity(count);
            heap.extend_from_slice(self.as_slice());
            self.heap = Some(heap);
        }
    }
    pub fn push(&mut self, value: T) {
        if self.heap.is_none() && self.len == 4 {
            self.reserve_exact(8);
        }
        if let Some(heap) = &mut self.heap {
            heap.push(value);
        } else {
            self.inline[self.len].write(value);
        }
        self.len += 1;
    }
    pub fn as_slice(&self) -> &[T] {
        if let Some(heap) = &self.heap {
            heap
        } else {
            unsafe { std::slice::from_raw_parts(self.inline.as_ptr().cast(), self.len) }
        }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        if let Some(heap) = &mut self.heap {
            heap
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.inline.as_mut_ptr().cast(), self.len) }
        }
    }
}
pub type ColorStorage = InlineStorage<SkPMColor4f>;
pub type PositionStorage = InlineStorage<f32>;
// Source: src/shaders/gradients/SkGradientBaseShader.h:158-158
// Source: src/shaders/gradients/SkGradientBaseShader.h:166-169
// Reverse C++ member destruction: intermediate color space, position storage, color storage.
pub struct SkColor4fXformer<'a> {
    pub fIntermediateColorSpace: Option<Arc<SkColorSpace>>,
    fPositionStorage: PositionStorage,
    pub fColors: ColorStorage,
    borrowed_positions: Option<&'a [f32]>,
    owned_positions: bool,
}
impl SkColor4fXformer<'_> {
    pub fn positions(&self) -> Option<&[f32]> {
        if self.owned_positions {
            Some(self.fPositionStorage.as_slice())
        } else {
            self.borrowed_positions
        }
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.h:172-172
pub use super::SkConicalGradient::SkRegisterConicalGradientShaderFlattenable;
// Source: src/shaders/gradients/SkGradientBaseShader.h:173-173
pub use super::SkLinearGradient::SkRegisterLinearGradientShaderFlattenable;
// Source: src/shaders/gradients/SkGradientBaseShader.h:174-174
pub use super::SkRadialGradient::SkRegisterRadialGradientShaderFlattenable;
// Source: src/shaders/gradients/SkGradientBaseShader.h:175-175
pub use super::SkSweepGradient::SkRegisterSweepGradientShaderFlattenable;

// Source: src/shaders/gradients/SkGradientBaseShader.cpp:45-68
const kHasPosition_GSF: u32 = 0x80000000;
const kHasLegacyLocalMatrix_GSF: u32 = 0x40000000;
const kHasColorSpace_GSF: u32 = 0x20000000;
const kTileModeShift_GSF: u32 = 8;
const kTileModeMask_GSF: u32 = 0xF;
const kInterpolationColorSpaceShift_GSF: u32 = 4;
const kInterpolationColorSpaceMask_GSF: u32 = 0xF;
const kInterpolationHueMethodShift_GSF: u32 = 1;
const kInterpolationHueMethodMask_GSF: u32 = 0x7;
const kInterpolationInPremul_GSF: u32 = 1;

impl SkGradientBaseShader {
    // Source: src/shaders/gradients/SkGradientBaseShader.h:75-75
    // Source: src/shaders/gradients/SkGradientBaseShader.cpp:70-113
    pub fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        let mut flags = 0;
        if self.getPositions().is_some() {
            flags |= kHasPosition_GSF;
        }
        // The canonical Rust base constructor normalizes null to real sRGB,
        // so fColorSpace is a retained nonnull Arc (the native invariant here).
        let color_space_data = self.fColorSpace.serialize();
        flags |= kHasColorSpace_GSF;
        if self.fInterpolation.fInPremul == InPremul::kYes {
            flags |= kInterpolationInPremul_GSF;
        }
        crate::SkASSERT!(@source b"src/shaders/gradients/SkGradientBaseShader.cpp\0".as_ptr(),82; @condition b"static_cast<uint32_t>(fTileMode) <= kTileModeMask_GSF\0".as_ptr(); (self.fTileMode as u32)<=kTileModeMask_GSF);
        flags |= (self.fTileMode as u32) << kTileModeShift_GSF;
        crate::SkASSERT!(@source b"src/shaders/gradients/SkGradientBaseShader.cpp\0".as_ptr(),84; @condition b"static_cast<uint32_t>(fInterpolation.fColorSpace) <= kInterpolationColorSpaceMask_GSF\0".as_ptr(); (self.fInterpolation.fColorSpace as u32)<=kInterpolationColorSpaceMask_GSF);
        flags |= (self.fInterpolation.fColorSpace as u32) << kInterpolationColorSpaceShift_GSF;
        crate::SkASSERT!(@source b"src/shaders/gradients/SkGradientBaseShader.cpp\0".as_ptr(),86; @condition b"static_cast<uint32_t>(fInterpolation.fHueMethod) <= kInterpolationHueMethodMask_GSF\0".as_ptr(); (self.fInterpolation.fHueMethod as u32)<=kInterpolationHueMethodMask_GSF);
        flags |= (self.fInterpolation.fHueMethod as u32) << kInterpolationHueMethodShift_GSF;
        buffer.writeUInt(flags);
        let mut start = 0;
        let mut count = self.fColorCount;
        if self.fFirstStopIsImplicit {
            count -= 1;
            start += 1;
        }
        if self.fLastStopIsImplicit {
            count -= 1;
        }
        buffer.writeColor4fArray(&self.colors()[start..start + count]);
        buffer.writeDataAsByteArray(Some(color_space_data.as_ref()));
        if let Some(positions) = self.getPositions() {
            buffer.writeScalarArray(&positions[start..start + count]);
        }

        Ok(())
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:115-123
fn validate_array<T: Clone>(
    buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    count: usize,
    array: &mut Vec<T>,
    initialized: T,
) -> bool {
    if !buffer.validateCanReadN::<T>(count) {
        return false;
    }
    // Native resized scratch is immediately overwritten by readArray. Rust's
    // initialized allocation avoids forming references to uninitialized values.
    array.resize(count, initialized);
    true
}
// Adapted source: src/shaders/gradients/SkGradientBaseShader.cpp:125-175: native open enum descriptor and ValidGradient gate use deferred checked Rust enum conversion.
// Rust representation adaptation of the native descriptor's open enums. Keep
// malformed discriminants as integers until the factory has consumed geometry;
// reject them at native ValidGradient's gate without invalidating the reader.
// No invalid Rust enum is constructed and buffer cursor behavior is preserved.
pub struct DecodedGradient<'a> {
    colors: &'a [SkColor4f],
    positions: &'a [f32],
    color_space: Option<Arc<SkColorSpace>>,
    tile: u32,
    color: u32,
    hue: u32,
    premul: InPremul,
}
impl<'a> DecodedGradient<'a> {
    pub fn typed(self) -> Option<SkGradient<'a>> {
        use crate::include::effects::SkGradient::{Colors, HueMethod};
        let tile = match self.tile {
            0 => SkTileMode::kClamp,
            1 => SkTileMode::kRepeat,
            2 => SkTileMode::kMirror,
            3 => SkTileMode::kDecal,
            _ => return None,
        };
        if self.color >= crate::include::effects::SkGradient::kColorSpaceCount as u32
            || self.hue >= crate::include::effects::SkGradient::kHueMethodCount as u32
        {
            return None;
        }
        let color = unsafe { core::mem::transmute::<u8, ColorSpace>(self.color as u8) };
        let hue = unsafe { core::mem::transmute::<u8, HueMethod>(self.hue as u8) };
        Some(SkGradient::new(
            Colors::new(self.colors, self.positions, tile, self.color_space),
            Interpolation {
                fInPremul: self.premul,
                fColorSpace: color,
                fHueMethod: hue,
            },
        ))
    }
}
impl SkGradientScope {
    // Source: src/shaders/gradients/SkGradientBaseShader.h:37-37
    // Source: src/shaders/gradients/SkGradientBaseShader.cpp:125-175
    pub fn unflatten<'scope>(
        &'scope mut self,
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
        legacy: &mut SkMatrix,
    ) -> Option<DecodedGradient<'scope>> {
        let flags = buffer.readUInt();
        let tile = (flags >> kTileModeShift_GSF) & kTileModeMask_GSF;
        let color = (flags >> kInterpolationColorSpaceShift_GSF) & kInterpolationColorSpaceMask_GSF;
        let hue = (flags >> kInterpolationHueMethodShift_GSF) & kInterpolationHueMethodMask_GSF;
        let premul = if flags & kInterpolationInPremul_GSF != 0 {
            InPremul::kYes
        } else {
            InPremul::kNo
        };
        let count = buffer.getArrayCount() as usize;
        if !(validate_array(
            buffer,
            count,
            &mut self.fColorStorage,
            SkColor4f {
                fR: 0.0,
                fG: 0.0,
                fB: 0.0,
                fA: 0.0,
            },
        ) && buffer.readColor4fArray(&mut self.fColorStorage))
        {
            return None;
        }
        let mut color_space = None;
        if flags & kHasColorSpace_GSF != 0 {
            if let Some(data) = buffer.readByteArrayAsData() {
                let bytes = if data.size() == 0 {
                    &[][..]
                } else {
                    unsafe { core::slice::from_raw_parts(data.data(), data.size()) }
                };
                color_space = SkColorSpace::Deserialize(bytes);
            }
        }
        let positions = if flags & kHasPosition_GSF != 0 {
            if !(validate_array(buffer, count, &mut self.fPositionStorage, 0.0)
                && buffer.readScalarArray(&mut self.fPositionStorage))
            {
                return None;
            }
            &self.fPositionStorage[..]
        } else {
            &[]
        };
        if flags & kHasLegacyLocalMatrix_GSF != 0 {
            crate::SkASSERT!(@source b"src/shaders/gradients/SkGradientBaseShader.cpp\0".as_ptr(),164; @condition b"buffer.isVersionLT(SkPicturePriv::Version::kNoShaderLocalMatrix)\0".as_ptr(); buffer.isVersionLT(crate::src::core::SkPicturePriv::kNoShaderLocalMatrix));
            buffer.readMatrix(legacy);
        } else {
            *legacy = SkMatrix::I().clone();
        }
        if buffer.isValid() {
            Some(DecodedGradient {
                colors: &self.fColorStorage,
                positions,
                color_space,
                tile,
                color,
                hue,
                premul,
            })
        } else {
            None
        }
    }
}

// Source: src/shaders/gradients/SkGradientBaseShader.cpp:179-181
pub fn srgb_if_null(cs: Option<Arc<SkColorSpace>>) -> Arc<SkColorSpace> {
    cs.unwrap_or_else(SkColorSpace::MakeSRGB)
}
// Source: src/shaders/gradients/SkGradientBaseShader.h:47-47
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:183-310
impl SkGradientBaseShader {
    pub fn new(desc: &SkGradient<'_>, ptsToUnit: &SkMatrix) -> Self {
        let (colors, pos) = color_pos_spans(desc);
        let matrix = ptsToUnit.clone();
        matrix.getType();
        crate::SkASSERT!(@source b"src/shaders/gradients/SkGradientBaseShader.cpp\0".as_ptr(),193; @condition b"colors.size() > 1\0".as_ptr(); colors.len() > 1);
        let mut result = Self {
            fPtsToUnit: matrix,
            fTileMode: desc.colors().tileMode(),
            fColorSpace: srgb_if_null(desc.colors().colorSpace().clone()),
            fInterpolation: *desc.interpolation(),
            fColorCount: colors.len(),
            fFirstStopIsImplicit: false,
            fLastStopIsImplicit: false,
            fColorsAreOpaque: true,
            storage: StopStorage::new(0),
            position_offset: None,
        };
        if !pos.is_empty() {
            result.fFirstStopIsImplicit = pos[0] > 0.0;
            result.fLastStopIsImplicit = pos[pos.len() - 1] != 1.0;
            result.fColorCount +=
                result.fFirstStopIsImplicit as usize + result.fLastStopIsImplicit as usize;
        }
        let count = result.fColorCount;
        result.storage = StopStorage::new(count * (4 + if pos.is_empty() { 0 } else { 1 }));
        result.position_offset = if pos.is_empty() {
            None
        } else {
            Some(count * 4)
        };
        // Each SkColor4f has four repr(C) floats. Both arrays share this allocation, as upstream.
        unsafe {
            let base_ptr = result.storage.mut_ptr();
            let color_ptr = base_ptr.cast::<SkColor4f>();
            let mut out = color_ptr;
            if result.fFirstStopIsImplicit {
                *out = colors[0];
                out = out.add(1);
            }
            for i in 0..colors.len() {
                *out.add(i) = colors[i];
                result.fColorsAreOpaque = result.fColorsAreOpaque && colors[i].fA == 1.0;
            }
            if result.fLastStopIsImplicit {
                *out.add(colors.len()) = colors[colors.len() - 1];
            }
            if !pos.is_empty() {
                let positions = base_ptr.add(count * 4);
                let mut prev = 0.0;
                *positions = prev;
                let start_index = if result.fFirstStopIsImplicit { 0 } else { 1 };
                let end = pos.len() + result.fLastStopIsImplicit as usize;
                let mut uniform_stops = true;
                let uniform_step = pos[start_index] - prev;
                let mut written = 1;
                for i in start_index..end {
                    let mut curr = 1.0;
                    if i != pos.len() {
                        curr = SkTPin(pos[i], prev, 1.0);
                        if curr == 1.0 && result.fLastStopIsImplicit {
                            result.fLastStopIsImplicit = false;
                        }
                    }
                    uniform_stops &= (uniform_step - (curr - prev)).abs() <= 1.0 / 4096.0;
                    prev = curr;
                    *positions.add(written) = prev;
                    written += 1;
                }
                if uniform_stops {
                    result.position_offset = None;
                } else {
                    let mut i = 0;
                    let mut deduped_count = 0;
                    for j in 1..=count {
                        if j == count || *positions.add(i) != *positions.add(j) {
                            crate::SkASSERT!(@source b"src/shaders/gradients/SkGradientBaseShader.cpp\0".as_ptr(),282; @condition b"j >= i\0".as_ptr(); j >= i);
                            let duplicate = j - i > 1;
                            let ignore_left = duplicate
                                && result.fTileMode != SkTileMode::kClamp
                                && *positions.add(i) == 0.0;
                            if !ignore_left {
                                *positions.add(deduped_count) = *positions.add(i);
                                *color_ptr.add(deduped_count) = *color_ptr.add(i);
                                deduped_count += 1;
                            }
                            let ignore_right = result.fTileMode != SkTileMode::kClamp
                                && *positions.add(j - 1) == 1.0;
                            if duplicate && !ignore_right {
                                *positions.add(deduped_count) = *positions.add(j - 1);
                                *color_ptr.add(deduped_count) = *color_ptr.add(j - 1);
                                deduped_count += 1;
                            }
                            i = j;
                        }
                    }
                    result.fColorCount = deduped_count;
                }
            }
        }
        result
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:314-327
unsafe fn add_stop_color(
    ctx: *mut crate::src::core::SkRasterPipelineOpContexts::GradientCtx,
    stop: usize,
    fs: [f32; 4],
    bs: [f32; 4],
) {
    unsafe {
        for i in 0..4 {
            *(*ctx).factors[i].add(stop) = fs[i];
            *(*ctx).biases[i].add(stop) = bs[i];
        }
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:329-333
unsafe fn add_const_color(
    ctx: *mut crate::src::core::SkRasterPipelineOpContexts::GradientCtx,
    stop: usize,
    color: [f32; 4],
) {
    unsafe {
        add_stop_color(ctx, stop, [0.0; 4], color);
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:338-362
unsafe fn init_stop_evenly(
    ctx: *mut crate::src::core::SkRasterPipelineOpContexts::GradientCtx,
    gap_count: f32,
    stop: usize,
    left: [f32; 4],
    right: [f32; 4],
) {
    let mut factor = [0.0; 4];
    let mut bias = [0.0; 4];
    for i in 0..4 {
        factor[i] = (right[i] - left[i]) * gap_count;
        bias[i] = left[i] - factor[i] * (stop as f32 / gap_count);
    }
    unsafe {
        add_stop_color(ctx, stop, factor, bias);
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:366-389
unsafe fn init_stop_pos(
    ctx: *mut crate::src::core::SkRasterPipelineOpContexts::GradientCtx,
    stop: usize,
    t_l: f32,
    gap_reciprocal: f32,
    left: [f32; 4],
    right: [f32; 4],
) {
    crate::SkASSERT!(@source b"src/shaders/gradients/SkGradientBaseShader.cpp\0".as_ptr(),374; @condition b"SkIsFinite(gapReciprocal)\0".as_ptr(); gap_reciprocal.is_finite());
    let mut factor = [0.0; 4];
    let mut bias = [0.0; 4];
    for i in 0..4 {
        factor[i] = (right[i] - left[i]) * gap_reciprocal;
        bias[i] = left[i] - factor[i] * t_l;
    }
    unsafe {
        *(*ctx).ts.add(stop) = t_l;
        add_stop_color(ctx, stop, factor, bias);
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.h:91-95
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:391-498
impl SkGradientBaseShader {
    /// Safety: arena and pipeline remain valid until execution; caller supplies at least two stops.
    pub unsafe fn AppendGradientFillStages(
        p: *mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        colors: &[SkPMColor4f],
        positions: Option<&[f32]>,
    ) {
        use crate::src::core::SkRasterPipelineOpContexts::{
            EvenlySpaced2StopGradientCtx, GradientCtx,
        };
        use crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp as Op;
        let count = colors.len();
        let rgba = |i: usize| {
            let c = colors[i];
            [c.fR, c.fG, c.fB, c.fA]
        };
        unsafe {
            if count == 2 && positions.is_none() {
                let left = rgba(0);
                let right = rgba(1);
                let mut factor = [0.0; 4];
                for i in 0..4 {
                    factor[i] = right[i] - left[i];
                }
                let ctx = (*alloc).make(EvenlySpaced2StopGradientCtx { factor, bias: left });
                (*p).append(Op::evenly_spaced_2_stop_gradient, ctx.cast());
                return;
            }
            let ctx = (*alloc)
                .makeBytesAlignedTo(
                    std::mem::size_of::<GradientCtx>(),
                    std::mem::align_of::<GradientCtx>(),
                )
                .cast::<GradientCtx>();
            // Rust adaptation: make every field valid before publishing typed references.
            // In particular, evenly-spaced gradients do not use or assign ts upstream.
            ctx.write(GradientCtx {
                stopCount: 0,
                factors: [std::ptr::null_mut(); 4],
                biases: [std::ptr::null_mut(); 4],
                ts: std::ptr::null_mut(),
            });
            let factor_bias_floats = (count + 1).max(8);
            let ts_count = count + 1;
            let to_alloc = 2 * 4 * factor_bias_floats + ts_count;
            let mut buffer = (*alloc).makeArrayDefault::<f32>(to_alloc);
            for i in 0..4 {
                (*ctx).factors[i] = buffer;
                buffer = buffer.add(factor_bias_floats);
                (*ctx).biases[i] = buffer;
                buffer = buffer.add(factor_bias_floats);
            }
            if positions.is_none() {
                let stop_count = count;
                let gap_count = (stop_count - 1) as f32;
                let mut left = rgba(0);
                let mut i = 0;
                while (i as f32) < gap_count {
                    let right = rgba(i + 1);
                    init_stop_evenly(ctx, gap_count, i, left, right);
                    left = right;
                    i += 1;
                }
                add_const_color(ctx, stop_count - 1, left);
                (*ctx).stopCount = stop_count;
                (*p).append(Op::evenly_spaced_gradient, ctx.cast());
                return;
            }
            let positions = positions.unwrap();
            (*ctx).ts = buffer;
            let (first, last) = if count > 2 {
                (
                    if rgba(0) != rgba(1) { 0 } else { 1 },
                    if rgba(count - 2) != rgba(count - 1) {
                        count - 1
                    } else {
                        count - 2
                    },
                )
            } else {
                (0, 1)
            };
            let mut stop_count = 0;
            let mut t_l = positions[first];
            let mut left = rgba(first);
            add_const_color(ctx, stop_count, left);
            stop_count += 1;
            for i in first..last {
                let t_r = positions[i + 1];
                let right = rgba(i + 1);
                crate::SkASSERT!(@source b"src/shaders/gradients/SkGradientBaseShader.cpp\0".as_ptr(),481; @condition b"t_l <= t_r\0".as_ptr(); t_l <= t_r);
                if t_l < t_r {
                    let scale = 1.0 / (t_r - t_l);
                    if scale.is_finite() {
                        init_stop_pos(ctx, stop_count, t_l, scale, left, right);
                        stop_count += 1;
                    }
                }
                t_l = t_r;
                left = right;
            }
            *(*ctx).ts.add(stop_count) = t_l;
            add_const_color(ctx, stop_count, left);
            stop_count += 1;
            (*ctx).stopCount = stop_count;
            (*p).append(Op::gradient, ctx.cast());
        }
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.h:97-102
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:500-567
impl SkGradientBaseShader {
    /// Safety: arena/pipeline must live through execution; arena retains the transform's context fields.
    pub unsafe fn AppendInterpolatedToDstStages(
        p: *mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        colors_are_opaque: bool,
        interpolation: &Interpolation,
        intermediate: Option<&SkColorSpace>,
        dst: Option<&SkColorSpace>,
    ) {
        use crate::include::core::SkAlphaType::SkAlphaType::{
            kPremul_SkAlphaType, kUnpremul_SkAlphaType,
        };
        use crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps;
        use crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp as Op;
        let mut color_is_premul = interpolation.fInPremul == InPremul::kYes;
        unsafe {
            if color_is_premul && !colors_are_opaque {
                match interpolation.fColorSpace {
                    ColorSpace::kLab | ColorSpace::kOKLab | ColorSpace::kOKLabGamutMap => {
                        (*p).append(Op::unpremul, std::ptr::null_mut());
                        color_is_premul = false;
                    }
                    ColorSpace::kLCH
                    | ColorSpace::kOKLCH
                    | ColorSpace::kOKLCHGamutMap
                    | ColorSpace::kHSL
                    | ColorSpace::kHWB => {
                        (*p).append(Op::unpremul_polar, std::ptr::null_mut());
                        color_is_premul = false;
                    }
                    _ => {}
                }
            }
            match interpolation.fColorSpace {
                ColorSpace::kLab => (*p).append(Op::css_lab_to_xyz, std::ptr::null_mut()),
                ColorSpace::kOKLab => {
                    (*p).append(Op::css_oklab_to_linear_srgb, std::ptr::null_mut())
                }
                ColorSpace::kOKLabGamutMap => {
                    (*p).append(Op::css_oklab_gamut_map_to_linear_srgb, std::ptr::null_mut())
                }
                ColorSpace::kLCH => {
                    (*p).append(Op::css_hcl_to_lab, std::ptr::null_mut());
                    (*p).append(Op::css_lab_to_xyz, std::ptr::null_mut());
                }
                ColorSpace::kOKLCH => {
                    (*p).append(Op::css_hcl_to_lab, std::ptr::null_mut());
                    (*p).append(Op::css_oklab_to_linear_srgb, std::ptr::null_mut());
                }
                ColorSpace::kOKLCHGamutMap => {
                    (*p).append(Op::css_hcl_to_lab, std::ptr::null_mut());
                    (*p).append(Op::css_oklab_gamut_map_to_linear_srgb, std::ptr::null_mut());
                }
                ColorSpace::kHSL => (*p).append(Op::css_hsl_to_srgb, std::ptr::null_mut()),
                ColorSpace::kHWB => (*p).append(Op::css_hwb_to_srgb, std::ptr::null_mut()),
                _ => {}
            }
            let srgb = SkColorSpace::MakeSRGB();
            let dst = dst.or(Some(srgb.as_ref()));
            let mut intermediate_alpha = if color_is_premul {
                kPremul_SkAlphaType
            } else {
                kUnpremul_SkAlphaType
            };
            let mut dst_alpha = kPremul_SkAlphaType;
            if colors_are_opaque {
                intermediate_alpha = kUnpremul_SkAlphaType;
                dst_alpha = kUnpremul_SkAlphaType;
            }
            let steps = (*alloc).make(SkColorSpaceXformSteps::new(
                intermediate,
                intermediate_alpha,
                dst,
                dst_alpha,
            ));
            (*steps).applyPipeline(&mut *p);
        }
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.h:82-84
pub trait SkGradientShader {
    fn gradient_base(&self) -> &SkGradientBaseShader;
    /// Safety: keep arena and both native pipelines alive while appending.
    unsafe fn append_gradient_stages(
        &self,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        pipeline: *mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
        post: *mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
    );

    // Source: src/shaders/gradients/SkGradientBaseShader.h:80-80
    // Source: src/shaders/gradients/SkGradientBaseShader.cpp:569-625
    /// Safety: preserve StageRec's arena, shader and pixel/context leases. Stage
    /// construction may append before returning Err; callers must not publish that program.
    unsafe fn append_gradient_pipeline(
        &self,
        rec: &crate::src::core::SkEffectPriv::SkStageRec<'_>,
        mRec: &crate::src::shaders::SkShaderBase::MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        use crate::src::core::SkRasterPipelineOpContexts::DecalTileCtx;
        use crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp as Op;
        let base = self.gradient_base();
        let p = rec.fPipeline;
        let alloc = rec.fAlloc;
        let mut decal_ctx: *mut DecalTileCtx = std::ptr::null_mut();
        let Some(_new_mrec) = mRec.apply(rec, &base.fPtsToUnit) else {
            return Ok(false);
        };
        let mut storage = core::pin::pin!(
            crate::src::core::SkArenaAlloc::SkSTArenaAlloc::<256>::new(0)
        );
        let mut post =
            crate::src::core::SkRasterPipeline::SkRasterPipeline::new(storage.as_mut().arena());
        self.append_gradient_stages(alloc, p, &mut post);
        match base.fTileMode {
            SkTileMode::kMirror => (*p).append(Op::mirror_x_1, std::ptr::null_mut()),
            SkTileMode::kRepeat => (*p).append(Op::repeat_x_1, std::ptr::null_mut()),
            SkTileMode::kDecal | SkTileMode::kClamp => {
                if base.fTileMode == SkTileMode::kDecal {
                    // Rust initialization adaptation: all POD members are valid
                    // before any stage forms a whole-context reference.
                    decal_ctx = (*alloc).make(DecalTileCtx::new(
                        [0; 16],
                        f32::from_bits(1.0f32.to_bits() + 1),
                        0.0,
                    ));
                    (*p).append(Op::decal_x, decal_ctx.cast());
                }
                if base.position_offset.is_none() {
                    (*p).append(Op::clamp_x_1, std::ptr::null_mut());
                }
            }
        }
        // This is the same Arc owner supplied by StageRec; the clone corresponds
        // to the native retained intermediate-color-space reference.
        let transformed = SkColor4fXformer::new(base, rec.fDstCS.cloned(), false)?;
        SkGradientBaseShader::AppendGradientFillStages(
            p,
            alloc,
            transformed.fColors.as_slice(),
            transformed.positions(),
        );
        SkGradientBaseShader::AppendInterpolatedToDstStages(
            p,
            alloc,
            base.fColorsAreOpaque,
            &base.fInterpolation,
            transformed.fIntermediateColorSpace.as_deref(),
            rec.fDstCS.map(Arc::as_ref),
        );
        if !decal_ctx.is_null() {
            (*p).append(Op::check_decal_mask, decal_ctx.cast());
        }
        (*p).extend(&post);
        Ok(true)
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:627-629
impl SkGradientBaseShader {
    pub fn isOpaque(&self) -> bool {
        self.fColorsAreOpaque && self.fTileMode != SkTileMode::kDecal
    }
    // Source: src/shaders/gradients/SkGradientBaseShader.cpp:631-649
    pub fn onAsLuminanceColor(&self, lum: &mut SkColor4f) -> bool {
        let mut color = SkColor4f {
            fR: 0.0,
            fG: 0.0,
            fB: 0.0,
            fA: 1.0,
        };
        for c in self.colors() {
            color.fR += c.fR;
            color.fG += c.fG;
            color.fB += c.fB;
        }
        let scale = 1.0 / self.fColorCount as f32;
        color.fR *= scale;
        color.fG *= scale;
        color.fB *= scale;
        *lum = color;
        true
    }
}

// C++ std::min/max preserve the first operand for unordered comparisons.
fn cmin(a: f32, b: f32) -> f32 {
    if b < a {
        b
    } else {
        a
    }
}
fn cmax(a: f32, b: f32) -> f32 {
    if a < b {
        b
    } else {
        a
    }
}

// Source: src/shaders/gradients/SkGradientBaseShader.cpp:651-703
pub fn intermediate_color_space(
    cs: ColorSpace,
    dst: Option<Arc<SkColorSpace>>,
) -> Option<Arc<SkColorSpace>> {
    use crate::src::core::SkColorSpace::{
        SkNamedGamut as Gamut, SkNamedPrimaries, SkNamedTransferFn as TF,
    };
    match cs {
        ColorSpace::kDestination => dst,
        ColorSpace::kSRGBLinear => Some(SkColorSpace::MakeSRGBLinear()),
        ColorSpace::kSRGB | ColorSpace::kHSL | ColorSpace::kHWB => Some(SkColorSpace::MakeSRGB()),
        ColorSpace::kLab | ColorSpace::kLCH => SkColorSpace::MakeRGB(&TF::kLinear, &Gamut::kXYZ),
        ColorSpace::kOKLab
        | ColorSpace::kOKLabGamutMap
        | ColorSpace::kOKLCH
        | ColorSpace::kOKLCHGamutMap => Some(SkColorSpace::MakeSRGBLinear()),
        ColorSpace::kDisplayP3 => SkColorSpace::MakeRGB(&TF::kSRGB, &Gamut::kDisplayP3),
        ColorSpace::kRec2020 => SkColorSpace::MakeRGB(&TF::kRec2020, &Gamut::kRec2020),
        ColorSpace::kProphotoRGB => {
            static MATRIX: std::sync::OnceLock<
                crate::modules::skcms::src::skcms_public::skcms_Matrix3x3,
            > = std::sync::OnceLock::new();
            let matrix = MATRIX.get_or_init(|| {
                let mut matrix = crate::modules::skcms::src::skcms_public::skcms_Matrix3x3 {
                    vals: [[0.0; 3]; 3],
                };
                SkNamedPrimaries::kProPhotoRGB.toXYZD50(&mut matrix);
                matrix
            });
            SkColorSpace::MakeRGB(&TF::kProPhotoRGB, matrix)
        }
        ColorSpace::kA98RGB => SkColorSpace::MakeRGB(&TF::kA98RGB, &Gamut::kAdobeRGB),
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:705-706
pub type ConvertColorProc = fn(SkPMColor4f, &mut bool) -> SkPMColor4f;
pub type PremulColorProc = fn(SkPMColor4f) -> SkPMColor4f;
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:708-730
pub fn srgb_to_hsl(rgb: SkPMColor4f, hueIsPowerless: &mut bool) -> SkPMColor4f {
    let mx = cmax(cmax(rgb.fR, rgb.fG), rgb.fB);
    let mn = cmin(cmin(rgb.fR, rgb.fG), rgb.fB);
    let (mut hue, mut sat, light) = (0.0, 0.0, (mn + mx) / 2.0);
    let d = mx - mn;
    if d != 0.0 {
        sat = if light == 0.0 || light == 1.0 {
            0.0
        } else {
            (mx - light) / cmin(light, 1.0 - light)
        };
        if mx == rgb.fR {
            hue = (rgb.fG - rgb.fB) / d + if rgb.fG < rgb.fB { 6.0 } else { 0.0 };
        } else if mx == rgb.fG {
            hue = (rgb.fB - rgb.fR) / d + 2.0;
        } else {
            hue = (rgb.fR - rgb.fG) / d + 4.0;
        }
        hue *= 60.0;
    }
    if sat == 0.0 {
        *hueIsPowerless = true;
    }
    SkPMColor4f {
        fR: hue,
        fG: sat * 100.0,
        fB: light * 100.0,
        fA: rgb.fA,
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:732-737
pub fn srgb_to_hwb(rgb: SkPMColor4f, hueIsPowerless: &mut bool) -> SkPMColor4f {
    let hsl = srgb_to_hsl(rgb, hueIsPowerless);
    let white = cmin(cmin(rgb.fR, rgb.fG), rgb.fB);
    let black = 1.0 - cmax(cmax(rgb.fR, rgb.fG), rgb.fB);
    SkPMColor4f {
        fR: hsl.fR,
        fG: white * 100.0,
        fB: black * 100.0,
        fA: rgb.fA,
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:739-752
pub fn xyzd50_to_lab(xyz: SkPMColor4f, _: &mut bool) -> SkPMColor4f {
    const D50: [f32; 3] = [
        0.3457f32 / 0.3585f32,
        1.0,
        (1.0f32 - 0.3457f32 - 0.3585f32) / 0.3585f32,
    ];
    const E: f32 = 216.0f32 / 24389.0;
    const K: f32 = 24389.0f32 / 27.0;
    let channels = [xyz.fR, xyz.fG, xyz.fB];
    let mut f = [0.0f32; 3];
    for i in 0..3 {
        let v = channels[i] / D50[i];
        f[i] = if v > E {
            v.cbrt()
        } else {
            (K * v + 16.0) / 116.0
        };
    }
    SkPMColor4f {
        fR: 116.0 * f[1] - 16.0,
        fG: 500.0 * (f[0] - f[1]),
        fB: 200.0 * (f[1] - f[2]),
        fA: xyz.fA,
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:756-766
pub fn xyzd50_to_hcl(xyz: SkPMColor4f, hueIsPowerless: &mut bool) -> SkPMColor4f {
    let lab = xyzd50_to_lab(xyz, hueIsPowerless);
    let hue = lab.fB.atan2(lab.fG) * (180.0f32 / std::f32::consts::PI);
    let chroma = (lab.fG * lab.fG + lab.fB * lab.fB).sqrt();
    const MAX_CHROMA: f32 = 1e-2;
    if chroma <= MAX_CHROMA {
        *hueIsPowerless = true;
    }
    SkPMColor4f {
        fR: if hue >= 0.0 { hue } else { hue + 360.0 },
        fG: chroma,
        fB: lab.fR,
        fA: xyz.fA,
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:769-780
pub fn lin_srgb_to_oklab(rgb: SkPMColor4f, _: &mut bool) -> SkPMColor4f {
    let mut l = 0.4122214708f32 * rgb.fR + 0.5363325363f32 * rgb.fG + 0.0514459929f32 * rgb.fB;
    let mut m = 0.2119034982f32 * rgb.fR + 0.6806995451f32 * rgb.fG + 0.1073969566f32 * rgb.fB;
    let mut s = 0.0883024619f32 * rgb.fR + 0.2817188376f32 * rgb.fG + 0.6299787005f32 * rgb.fB;
    l = l.cbrt();
    m = m.cbrt();
    s = s.cbrt();
    SkPMColor4f {
        fR: 0.2104542553f32 * l + 0.7936177850f32 * m - 0.0040720468f32 * s,
        fG: 1.9779984951f32 * l - 2.4285922050f32 * m + 0.4505937099f32 * s,
        fB: 0.0259040371f32 * l + 0.7827717662f32 * m - 0.8086757660f32 * s,
        fA: rgb.fA,
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:784-794
pub fn lin_srgb_to_okhcl(rgb: SkPMColor4f, hueIsPowerless: &mut bool) -> SkPMColor4f {
    let lab = lin_srgb_to_oklab(rgb, hueIsPowerless);
    let hue = lab.fB.atan2(lab.fG) * (180.0f32 / std::f32::consts::PI);
    let chroma = (lab.fG * lab.fG + lab.fB * lab.fB).sqrt();
    const MAX_CHROMA: f32 = 1e-6;
    if chroma <= MAX_CHROMA {
        *hueIsPowerless = true;
    }
    SkPMColor4f {
        fR: if hue >= 0.0 { hue } else { hue + 360.0 },
        fG: chroma,
        fB: lab.fR,
        fA: rgb.fA,
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:796-798
pub fn premul_polar(hsl: SkPMColor4f) -> SkPMColor4f {
    SkPMColor4f {
        fR: hsl.fR,
        fG: hsl.fG * hsl.fA,
        fB: hsl.fB * hsl.fA,
        fA: hsl.fA,
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:800-802
pub fn premul_rgb(rgb: SkPMColor4f) -> SkPMColor4f {
    SkPMColor4f {
        fR: rgb.fR * rgb.fA,
        fG: rgb.fG * rgb.fA,
        fB: rgb.fB * rgb.fA,
        fA: rgb.fA,
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:804-815
pub fn color_space_is_polar(cs: ColorSpace) -> bool {
    matches!(
        cs,
        ColorSpace::kLCH | ColorSpace::kOKLCH | ColorSpace::kHSL | ColorSpace::kHWB
    )
}
impl<'a> SkColor4fXformer<'a> {
    // Source: src/shaders/gradients/SkGradientBaseShader.cpp:840-868
    pub fn new(
        shader: &'a SkGradientBaseShader,
        dst: Option<Arc<SkColorSpace>>,
        force_explicit_positions: bool,
    ) -> Result<Self, crate::src::core::SkRasterPipeline::MissingStage> {
        use crate::include::core::{
            SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType,
            SkColorType::SkColorType::kRGBA_F32_SkColorType,
        };
        let color_count = shader.colors().len();
        let intermediate = intermediate_color_space(shader.interpolation().fColorSpace, dst);
        let info = crate::src::core::SkImageInfo::SkImageInfo::Make(
            color_count as i32,
            1,
            kRGBA_F32_SkColorType,
            kUnpremul_SkAlphaType,
        );
        let dst_info = info.makeColorSpace(intermediate.clone());
        let src_info = info.makeColorSpace(Some(Arc::clone(shader.colorSpace())));
        // Rust adaptation: initialized output elements permit a valid typed slice
        // during conversion. This is local storage initialization, not arena zeroing.
        let mut colors = ColorStorage::new();
        colors.reserve_exact(color_count);
        for _ in 0..color_count {
            colors.push(SkPMColor4f {
                fR: 0.0,
                fG: 0.0,
                fB: 0.0,
                fA: 0.0,
            });
        }
        let converted = unsafe {
            crate::src::core::SkConvertPixels::SkConvertPixels(
                &dst_info,
                colors.as_mut_slice().as_mut_ptr().cast(),
                info.minRowBytes(),
                &src_info,
                shader.colors().as_ptr().cast(),
                info.minRowBytes(),
            )
        }?;
        // Original SkAssertResult requires success for these same-size float images.
        crate::SkAssertResult!(@source b"src/shaders/gradients/SkGradientBaseShader.cpp\0".as_ptr(),863; @condition b"SkConvertPixels(dstInfo, fColors.begin(), info.minRowBytes(), srcInfo, shader->colors().data(), info.minRowBytes())\0".as_ptr(); converted);
        Ok(Self::from_intermediate(
            shader,
            intermediate,
            colors,
            force_explicit_positions,
        ))
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:871-1017
impl<'a> SkColor4fXformer<'a> {
    // Constructor continuation after the native conversion prefix above.
    pub fn from_intermediate(
        shader: &'a SkGradientBaseShader,
        intermediate: Option<Arc<SkColorSpace>>,
        colors: ColorStorage,
        force_explicit_positions: bool,
    ) -> Self {
        use crate::include::effects::SkGradient::HueMethod;
        let mut result = Self {
            fColors: colors,
            fPositionStorage: PositionStorage::new(),
            borrowed_positions: shader.getPositions(),
            owned_positions: false,
            fIntermediateColorSpace: intermediate,
        };
        let mut color_count = result.fColors.as_slice().len();
        let interpolation = shader.interpolation();
        debug_assert_eq!(color_count, shader.getColorCount());
        let convert_fn: Option<ConvertColorProc> = match interpolation.fColorSpace {
            ColorSpace::kHSL => Some(srgb_to_hsl),
            ColorSpace::kHWB => Some(srgb_to_hwb),
            ColorSpace::kLab => Some(xyzd50_to_lab),
            ColorSpace::kLCH => Some(xyzd50_to_hcl),
            ColorSpace::kOKLab | ColorSpace::kOKLabGamutMap => Some(lin_srgb_to_oklab),
            ColorSpace::kOKLCH | ColorSpace::kOKLCHGamutMap => Some(lin_srgb_to_okhcl),
            _ => None,
        };
        let mut powerless = InlineStorage::new();
        let mut any_powerless = false;
        for _ in 0..color_count {
            powerless.push(false);
        }
        if let Some(convert) = convert_fn {
            for i in 0..color_count {
                result.fColors.as_mut_slice()[i] = convert(
                    result.fColors.as_slice()[i],
                    &mut powerless.as_mut_slice()[i],
                );
                any_powerless = any_powerless || powerless.as_slice()[i];
            }
        }
        if any_powerless {
            let mut new_colors = ColorStorage::new();
            let mut new_positions = PositionStorage::new();
            for i in 0..color_count {
                let current = result.fColors.as_slice()[i];
                let position = shader.getPos(i);
                if !powerless.as_slice()[i] {
                    new_colors.push(current);
                    new_positions.push(position);
                    continue;
                }
                let hue_from = |color: SkPMColor4f, hue_color: SkPMColor4f| SkPMColor4f {
                    fR: hue_color.fR,
                    fG: color.fG,
                    fB: color.fB,
                    fA: color.fA,
                };
                if i != 0 {
                    new_positions.push(position);
                    new_colors.push(hue_from(current, result.fColors.as_slice()[i - 1]));
                }
                if i != color_count - 1 {
                    new_positions.push(position);
                    new_colors.push(hue_from(current, result.fColors.as_slice()[i + 1]));
                }
            }
            std::mem::swap(&mut result.fColors, &mut new_colors);
            std::mem::swap(&mut result.fPositionStorage, &mut new_positions);
            result.owned_positions = true;
            color_count = result.fColors.as_slice().len();
        }
        if color_space_is_polar(interpolation.fColorSpace) {
            let mut delta = 0.0;
            for i in 0..color_count - 1 {
                let h1 = result.fColors.as_slice()[i].fR;
                let h2 = &mut result.fColors.as_mut_slice()[i + 1].fR;
                *h2 += delta;
                match interpolation.fHueMethod {
                    HueMethod::kShorter => {
                        if *h2 - h1 > 180.0 {
                            *h2 -= 360.0;
                            delta -= 360.0;
                        } else if *h2 - h1 < -180.0 {
                            *h2 += 360.0;
                            delta += 360.0;
                        }
                    }
                    HueMethod::kLonger => {
                        if (i == 0 && shader.firstStopIsImplicit())
                            || (i == color_count - 2 && shader.lastStopIsImplicit())
                        {
                        } else if 0.0 < *h2 - h1 && *h2 - h1 < 180.0 {
                            *h2 -= 360.0;
                            delta -= 360.0;
                        } else if -180.0 < *h2 - h1 && *h2 - h1 <= 0.0 {
                            *h2 += 360.0;
                            delta += 360.0;
                        }
                    }
                    HueMethod::kIncreasing => {
                        if *h2 < h1 {
                            *h2 += 360.0;
                            delta += 360.0;
                        }
                    }
                    HueMethod::kDecreasing => {
                        if h1 < *h2 {
                            *h2 -= 360.0;
                            delta -= 360.0;
                        }
                    }
                }
            }
        }
        let premul_fn: Option<PremulColorProc> = if interpolation.fInPremul == InPremul::kYes {
            Some(match interpolation.fColorSpace {
                ColorSpace::kHSL | ColorSpace::kHWB | ColorSpace::kLCH | ColorSpace::kOKLCH => {
                    premul_polar
                }
                _ => premul_rgb,
            })
        } else {
            None
        };
        if let Some(premul) = premul_fn {
            for i in 0..color_count {
                result.fColors.as_mut_slice()[i] = premul(result.fColors.as_slice()[i]);
            }
        }
        if force_explicit_positions && result.positions().is_none() {
            result.fPositionStorage.reserve_exact(color_count);
            let scale = 1.0 / (color_count - 1) as f32;
            for i in 0..color_count {
                result.fPositionStorage.push(i as f32 * scale);
            }
            result.owned_positions = true;
        }
        result
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.h:77-77
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:1019-1038
impl SkGradientBaseShader {
    /// Safety: nonnull output pointers in info must hold fColorCount writable entries.
    pub unsafe fn commonAsAGradient(
        &self,
        info: &mut crate::src::shaders::SkShaderBase::GradientInfo,
    ) {
        let count = i32::try_from(self.fColorCount).expect("gradient stop count fits i32");
        if info.fColorCount >= count {
            unsafe {
                if !info.fColors.is_null() {
                    for i in 0..self.fColorCount {
                        *info.fColors.add(i) = self.colors()[i];
                    }
                }
                if !info.fColorOffsets.is_null() {
                    for i in 0..self.fColorCount {
                        *info.fColorOffsets.add(i) = self.getPos(i);
                    }
                }
            }
        }
        info.fColorCount = count;
        info.fTileMode = self.fTileMode;
        info.fPremulInterp = self.interpolateInPremul();
    }
    // Source: src/shaders/gradients/SkGradientBaseShader.h:64-66
    // Source: src/shaders/gradients/SkGradientBaseShader.cpp:1042-1048
    pub fn ValidGradient(
        colors: &[SkColor4f],
        tile_mode: SkTileMode,
        interpolation: &Interpolation,
    ) -> bool {
        !colors.is_empty()
            && (tile_mode as usize) < 4
            && (interpolation.fColorSpace as usize)
                < crate::include::effects::SkGradient::kColorSpaceCount
            && (interpolation.fHueMethod as usize)
                < crate::include::effects::SkGradient::kHueMethodCount
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:1050-1099
pub fn average_gradient_color(colors: &[SkColor4f], pos: &[f32]) -> SkColor4f {
    let mut blend = [0.0f32; 4];
    for i in 0..colors.len() - 1 {
        let c0 = [colors[i].fR, colors[i].fG, colors[i].fB, colors[i].fA];
        let c1 = [
            colors[i + 1].fR,
            colors[i + 1].fG,
            colors[i + 1].fB,
            colors[i + 1].fA,
        ];
        let w;
        if !pos.is_empty() {
            let p0 = SkTPin(pos[i], 0.0, 1.0);
            let p1 = SkTPin(pos[i + 1], p0, 1.0);
            w = p1 - p0;
            if i == 0 && p0 > 0.0 {
                for j in 0..4 {
                    blend[j] += p0 * c0[j];
                }
            }
            if i == colors.len() - 2 && p1 < 1.0 {
                for j in 0..4 {
                    blend[j] += (1.0 - p1) * c1[j];
                }
            }
        } else {
            w = 1.0 / (colors.len() - 1) as f32;
        }
        for j in 0..4 {
            blend[j] += (0.5 * w) * (c1[j] + c0[j]);
        }
    }
    SkColor4f {
        fR: blend[0],
        fG: blend[1],
        fB: blend[2],
        fA: blend[3],
    }
}
// Source: src/shaders/gradients/SkGradientBaseShader.h:68-68
// Source: src/shaders/gradients/SkGradientBaseShader.cpp:1104-1124
impl SkGradientBaseShader {
    pub fn MakeDegenerateGradient(
        colors: &crate::include::effects::SkGradient::Colors<'_>,
    ) -> Option<Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        use crate::src::shaders::SkColorShader::Color4f;
        match colors.tileMode() {
            SkTileMode::kDecal => Some(crate::src::shaders::SkEmptyShader::Empty()),
            SkTileMode::kRepeat | SkTileMode::kMirror => Color4f(
                &average_gradient_color(colors.colors(), colors.positions()),
                colors.colorSpace().clone(),
            ),
            SkTileMode::kClamp => Color4f(
                colors
                    .colors()
                    .last()
                    .expect("gradient has at least one stop"),
                colors.colorSpace().clone(),
            ),
        }
    }
}

// Source: src/shaders/gradients/SkGradientBaseShader.h:178-184
// Adapted nullable span boundary: Colors already borrows a validated color span
// and either an empty position slice (native null) or its equal-length span.
// The native legacy raw-pointer macro has no pinned in-tree callers. Reusing
// the same conversion at the actual Rust base producer preserves absence,
// count and caller backing lifetime without exposing raw invalid pointers.
fn color_pos_spans<'a>(gradient: &'a SkGradient<'_>) -> (&'a [SkColor4f], &'a [f32]) {
    let colors = gradient.colors().colors();
    let mut pos: &[f32] = &[];
    if !gradient.colors().positions().is_empty() {
        pos = gradient.colors().positions();
    }
    (colors, pos)
}

// Source: src/shaders/gradients/SkGradientBaseShader.h:187-200
macro_rules! gradient_factory_early_exit {
    ($gradient:expr, $matrix:expr) => {{
        let colors = $gradient.colors();
        let interpolation = $gradient.interpolation();
        if !super::SkGradientBaseShader::SkGradientBaseShader::ValidGradient(
            colors.colors(),
            colors.tileMode(),
            interpolation,
        ) {
            return None;
        }
        if colors.colors().len() == 1 {
            return crate::src::shaders::SkColorShader::Color4f(
                &colors.colors()[0],
                colors.colorSpace().clone(),
            );
        }
        if let Some(matrix) = $matrix {
            if matrix.invert().is_none() {
                return None;
            }
        }
    }};
}
pub(crate) use gradient_factory_early_exit;
