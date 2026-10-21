// Rust concrete f32/f64 specializations retain upstream unsigned-bit arithmetic and ULP policy.

// Source: src/utils/SkFloatUtils.h:54-171

#[derive(Clone, Copy, Debug)]
pub struct SkFloatingPointF32<const ULPS: u32> {
    bits: u32,
}
impl<const ULPS: u32> SkFloatingPointF32<ULPS> {
    pub const kBitCount: usize = 32;
    pub const kFractionBitCount: usize = 23;
    pub const kExponentBitCount: usize = Self::kBitCount - 1 - Self::kFractionBitCount;
    pub const kSignBitMask: u32 = 1 << (32 - 1);
    pub const kFractionBitMask: u32 = !0 >> (Self::kExponentBitCount + 1);
    pub const kExponentBitMask: u32 = !(Self::kSignBitMask | Self::kFractionBitMask);
    pub const kMaxUlps: usize = ULPS as usize;
    #[inline]
    pub fn new(x: f32) -> Self {
        Self { bits: x.to_bits() }
    }
    #[inline]
    pub fn exponent_bits(&self) -> u32 {
        Self::kExponentBitMask & self.bits
    }
    #[inline]
    pub fn fraction_bits(&self) -> u32 {
        Self::kFractionBitMask & self.bits
    }
    #[inline]
    pub fn is_nan(&self) -> bool {
        self.exponent_bits() == Self::kExponentBitMask && self.fraction_bits() != 0
    }
    #[inline]
    pub fn AlmostEquals(&self, rhs: &Self) -> bool {
        if self.is_nan() || rhs.is_nan() {
            return false;
        }
        let dist = Self::DistanceBetweenSignAndMagnitudeNumbers(self.bits, rhs.bits);
        dist as u64 <= Self::kMaxUlps as u64
    }
    #[inline]
    fn SignAndMagnitudeToBiased(sam: u32) -> u32 {
        if Self::kSignBitMask & sam != 0 {
            (!sam).wrapping_add(1)
        } else {
            Self::kSignBitMask | sam
        }
    }
    #[inline]
    fn DistanceBetweenSignAndMagnitudeNumbers(sam1: u32, sam2: u32) -> u32 {
        let biased1 = Self::SignAndMagnitudeToBiased(sam1);
        let biased2 = Self::SignAndMagnitudeToBiased(sam2);
        if biased1 >= biased2 {
            biased1 - biased2
        } else {
            biased2 - biased1
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SkFloatingPointF64<const ULPS: u32> {
    bits: u64,
}
impl<const ULPS: u32> SkFloatingPointF64<ULPS> {
    pub const kBitCount: usize = 64;
    pub const kFractionBitCount: usize = 52;
    pub const kExponentBitCount: usize = Self::kBitCount - 1 - Self::kFractionBitCount;
    pub const kSignBitMask: u64 = 1 << (64 - 1);
    pub const kFractionBitMask: u64 = !0 >> (Self::kExponentBitCount + 1);
    pub const kExponentBitMask: u64 = !(Self::kSignBitMask | Self::kFractionBitMask);
    pub const kMaxUlps: usize = ULPS as usize;
    #[inline]
    pub fn new(x: f64) -> Self {
        Self { bits: x.to_bits() }
    }
    #[inline]
    pub fn exponent_bits(&self) -> u64 {
        Self::kExponentBitMask & self.bits
    }
    #[inline]
    pub fn fraction_bits(&self) -> u64 {
        Self::kFractionBitMask & self.bits
    }
    #[inline]
    pub fn is_nan(&self) -> bool {
        self.exponent_bits() == Self::kExponentBitMask && self.fraction_bits() != 0
    }
    #[inline]
    pub fn AlmostEquals(&self, rhs: &Self) -> bool {
        if self.is_nan() || rhs.is_nan() {
            return false;
        }
        let dist = Self::DistanceBetweenSignAndMagnitudeNumbers(self.bits, rhs.bits);
        dist as u64 <= Self::kMaxUlps as u64
    }
    #[inline]
    fn SignAndMagnitudeToBiased(sam: u64) -> u64 {
        if Self::kSignBitMask & sam != 0 {
            (!sam).wrapping_add(1)
        } else {
            Self::kSignBitMask | sam
        }
    }
    #[inline]
    fn DistanceBetweenSignAndMagnitudeNumbers(sam1: u64, sam2: u64) -> u64 {
        let biased1 = Self::SignAndMagnitudeToBiased(sam1);
        let biased2 = Self::SignAndMagnitudeToBiased(sam2);
        if biased1 >= biased2 {
            biased1 - biased2
        } else {
            biased2 - biased1
        }
    }
}

// Native declaration: src/utils/SkFloatUtils.h:182; implementation lives in core/SkScalar.rs.
pub use crate::src::core::SkScalar::SkFloatInterpFunc;
