// Copyright 2012 the V8 project authors. All rights reserved.
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright
//       notice, this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above
//       copyright notice, this list of conditions and the following
//       disclaimer in the documentation and/or other materials provided
//       with the distribution.
//     * Neither the name of Google Inc. nor the names of its
//       contributors may be used to endorse or promote products derived
//       from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: ieee.h:43-302, binary64 helpers used by shortest and StrtodTrimmed.
use super::diy_fp::DiyFp;
#[derive(Clone, Copy)]
pub(super) struct Double(pub u64);
impl Double {
    pub const kSignificandSize: i32 = 53;
    pub const kHiddenBit: u64 = 0x0010_0000_0000_0000;
    const kSignificandMask: u64 = 0x000f_ffff_ffff_ffff;
    const kExponentMask: u64 = 0x7ff0_0000_0000_0000;
    const kExponentBias: i32 = 0x3ff + 52;
    const kDenormalExponent: i32 = -Self::kExponentBias + 1;
    pub fn new(value: f64) -> Self {
        Self(value.to_bits())
    }
    pub fn value(self) -> f64 {
        f64::from_bits(self.0)
    }
    pub fn Exponent(self) -> i32 {
        if self.IsDenormal() {
            Self::kDenormalExponent
        } else {
            ((self.0 & Self::kExponentMask) >> 52) as i32 - Self::kExponentBias
        }
    }
    pub fn Significand(self) -> u64 {
        let f = self.0 & Self::kSignificandMask;
        if self.IsDenormal() {
            f
        } else {
            f + Self::kHiddenBit
        }
    }
    pub fn IsDenormal(self) -> bool {
        self.0 & Self::kExponentMask == 0
    }
    pub fn IsSpecial(self) -> bool {
        self.0 & Self::kExponentMask == Self::kExponentMask
    }
    pub fn IsNan(self) -> bool {
        self.IsSpecial() && self.0 & Self::kSignificandMask != 0
    }
    pub fn Sign(self) -> i32 {
        if self.0 & (1 << 63) == 0 {
            1
        } else {
            -1
        }
    }
    pub fn AsDiyFp(self) -> DiyFp {
        debug_assert!(self.Sign() > 0 && !self.IsSpecial());
        DiyFp::new(self.Significand(), self.Exponent())
    }
    pub fn AsNormalizedDiyFp(self) -> DiyFp {
        debug_assert!(self.value() > 0.0);
        let mut f = self.Significand();
        let mut e = self.Exponent();
        while f & Self::kHiddenBit == 0 {
            f <<= 1;
            e -= 1;
        }
        DiyFp::new(f << 11, e - 11)
    }
    pub fn UpperBoundary(self) -> DiyFp {
        debug_assert!(self.Sign() > 0);
        DiyFp::new(self.Significand() * 2 + 1, self.Exponent() - 1)
    }
    pub fn LowerBoundaryIsCloser(self) -> bool {
        self.0 & Self::kSignificandMask == 0 && self.Exponent() != Self::kDenormalExponent
    }
    pub fn NormalizedBoundaries(self) -> (DiyFp, DiyFp) {
        let v = self.AsDiyFp();
        let plus = DiyFp::Normalized(DiyFp::new((v.f << 1) + 1, v.e - 1));
        let mut minus = if self.LowerBoundaryIsCloser() {
            DiyFp::new((v.f << 2) - 1, v.e - 2)
        } else {
            DiyFp::new((v.f << 1) - 1, v.e - 1)
        };
        minus.f <<= minus.e - plus.e;
        minus.e = plus.e;
        (minus, plus)
    }
    pub fn NextDouble(self) -> f64 {
        if self.0 == Self::kExponentMask {
            return self.value();
        }
        if self.Sign() < 0 && self.Significand() == 0 {
            return 0.0;
        }
        Self(if self.Sign() < 0 {
            self.0 - 1
        } else {
            self.0 + 1
        })
        .value()
    }
    pub fn Infinity() -> f64 {
        Self(Self::kExponentMask).value()
    }
    pub fn SignificandSizeForOrderOfMagnitude(order: i32) -> i32 {
        if order >= Self::kDenormalExponent + Self::kSignificandSize {
            Self::kSignificandSize
        } else if order <= Self::kDenormalExponent {
            0
        } else {
            order - Self::kDenormalExponent
        }
    }
    // Source: ieee.h:273-301. Encode an already-rounded DiyFp.
    pub fn FromDiyFp(diy: DiyFp) -> Self {
        let mut f = diy.f;
        let mut e = diy.e;
        while f > Self::kHiddenBit + Self::kSignificandMask {
            f >>= 1;
            e += 1;
        }
        if e >= 0x7ff - Self::kExponentBias {
            return Self(Self::kExponentMask);
        }
        if e < Self::kDenormalExponent {
            return Self(0);
        }
        while e > Self::kDenormalExponent && f & Self::kHiddenBit == 0 {
            f <<= 1;
            e -= 1;
        }
        let biased = if e == Self::kDenormalExponent && f & Self::kHiddenBit == 0 {
            0
        } else {
            (e + Self::kExponentBias) as u64
        };
        Self((f & Self::kSignificandMask) | (biased << 52))
    }
}
