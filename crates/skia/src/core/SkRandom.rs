// Copyright 2006 Google LLC. BSD-style license.
// Source: src/core/SkRandom.h:17-17
pub type SkScalar = crate::src::core::SkScalar::SkScalar;
// Source: src/core/SkRandom.h:27-27
// Source: src/core/SkRandom.h:169-170
#[derive(Clone, Copy)]
pub struct SkRandom {
    fK: u32,
    fJ: u32,
}
impl Default for SkRandom {
    fn default() -> Self {
        Self::new(0)
    }
}
impl SkRandom {
    // Source: src/core/SkRandom.h:29-31
    pub fn new(seed: u32) -> Self {
        let mut this = Self { fK: 0, fJ: 0 };
        this.init(seed);
        this
    }
    // Source: src/core/SkRandom.h:33-38
    pub fn assign(&mut self, rand: &Self) -> &mut Self {
        self.fK = rand.fK;
        self.fJ = rand.fJ;
        self
    }
    // Source: src/core/SkRandom.h:42-46
    pub fn nextU(&mut self) -> u32 {
        self.fK = Self::kKMul
            .wrapping_mul(self.fK & 0xffff)
            .wrapping_add(self.fK >> 16);
        self.fJ = Self::kJMul
            .wrapping_mul(self.fJ & 0xffff)
            .wrapping_add(self.fJ >> 16);
        ((self.fK << 16) | (self.fK >> 16)).wrapping_add(self.fJ)
    }
    // Source: src/core/SkRandom.h:50-50
    pub fn nextS(&mut self) -> i32 {
        self.nextU() as i32
    }
    // Source: src/core/SkRandom.h:55-59
    pub fn nextF(&mut self) -> f32 {
        let floatint = 0x3f800000 | (self.nextU() >> 9);
        f32::from_bits(floatint) - 1.0
    }
    // Source: src/core/SkRandom.h:64-66
    pub fn nextRangeF(&mut self, min: f32, max: f32) -> f32 {
        min + self.nextF() * (max - min)
    }
    // Source: src/core/SkRandom.h:72-75
    pub fn nextBits(&mut self, bitCount: u32) -> u32 {
        crate::SkASSERT!(@source b"src/core/SkRandom.h\0".as_ptr(), 73; @condition b"bitCount > 0 && bitCount <= 32\0".as_ptr(); bitCount>0&&bitCount<=32);
        self.nextU() >> (32 - bitCount)
    }
    // Source: src/core/SkRandom.h:80-88
    pub fn nextRangeU(&mut self, min: u32, max: u32) -> u32 {
        crate::SkASSERT!(@source b"src/core/SkRandom.h\0".as_ptr(), 81; @condition b"min <= max\0".as_ptr(); min<=max);
        let range = max.wrapping_sub(min).wrapping_add(1);
        if range == 0 {
            self.nextU()
        } else {
            min.wrapping_add(self.nextU() % range)
        }
    }
    // Source: src/core/SkRandom.h:93-96
    pub fn nextULessThan(&mut self, count: u32) -> u32 {
        crate::SkASSERT!(@source b"src/core/SkRandom.h\0".as_ptr(), 94; @condition b"count > 0\0".as_ptr(); count>0);
        self.nextRangeU(0, count - 1)
    }
    // Source: src/core/SkRandom.h:101-101
    pub fn nextUScalar1(&mut self) -> f32 {
        crate::include::private::SkFixed::SkFixedToFloat(self.nextUFixed1())
    }
    // Source: src/core/SkRandom.h:106-108
    pub fn nextRangeScalar(&mut self, min: f32, max: f32) -> f32 {
        self.nextUScalar1() * (max - min) + min
    }
    // Source: src/core/SkRandom.h:113-113
    pub fn nextSScalar1(&mut self) -> f32 {
        crate::include::private::SkFixed::SkFixedToFloat(self.nextSFixed1())
    }
    // Source: src/core/SkRandom.h:117-117
    pub fn nextBool(&mut self) -> bool {
        self.nextU() >= 0x80000000
    }
    // Source: src/core/SkRandom.h:121-124
    pub fn nextBiasedBool(&mut self, fractionTrue: f32) -> bool {
        crate::SkASSERT!(@source b"src/core/SkRandom.h\0".as_ptr(), 122; @condition b"fractionTrue >= 0 && fractionTrue <= 1\0".as_ptr(); fractionTrue>=0.0&&fractionTrue<=1.0);
        self.nextUScalar1() <= fractionTrue
    }
    // Source: src/core/SkRandom.h:128-128
    pub fn setSeed(&mut self, seed: u32) {
        self.init(seed);
    }
    // Source: src/core/SkRandom.h:134-144
    fn init(&mut self, seed: u32) {
        self.fK = Self::NextLCG(seed);
        if self.fK == 0 {
            self.fK = Self::NextLCG(self.fK);
        }
        self.fJ = Self::NextLCG(self.fK);
        if self.fJ == 0 {
            self.fJ = Self::NextLCG(self.fJ);
        }
        crate::SkASSERT!(@source b"src/core/SkRandom.h\0".as_ptr(), 143; @condition b"0 != fK && 0 != fJ\0".as_ptr(); self.fK!=0&&self.fJ!=0);
    }
    // Source: src/core/SkRandom.h:145-145
    fn NextLCG(seed: u32) -> u32 {
        Self::kMul.wrapping_mul(seed).wrapping_add(Self::kAdd)
    }
    // Source: src/core/SkRandom.h:150-150
    fn nextUFixed1(&mut self) -> i32 {
        (self.nextU() >> 16) as i32
    }
    // Source: src/core/SkRandom.h:155-155
    fn nextSFixed1(&mut self) -> i32 {
        self.nextS() >> 15
    }
    // Source: src/core/SkRandom.h:160-166
    const kMul: u32 = 1664525;
    const kAdd: u32 = 1013904223;
    const kKMul: u32 = 30345;
    const kJMul: u32 = 18000;
}
