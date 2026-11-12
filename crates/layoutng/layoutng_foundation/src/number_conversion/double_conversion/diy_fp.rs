// Copyright 2010 the V8 project authors. All rights reserved.
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
// Source: diy-fp.h:41-133. Nonnegative uint64 significand/binary exponent.
#[derive(Clone, Copy)]
pub(super) struct DiyFp {
    pub f: u64,
    pub e: i32,
}
impl DiyFp {
    pub fn new(f: u64, e: i32) -> Self {
        Self { f, e }
    }
    pub fn Minus(a: Self, b: Self) -> Self {
        debug_assert_eq!(a.e, b.e);
        debug_assert!(a.f >= b.f);
        Self::new(a.f - b.f, a.e)
    }
    // Source: diy-fp.h:77-101. Round the high half of the 128-bit product.
    pub fn Multiply(&mut self, other: Self) {
        let a = self.f >> 32;
        let b = self.f & 0xffff_ffff;
        let c = other.f >> 32;
        let d = other.f & 0xffff_ffff;
        let ac = a * c;
        let bc = b * c;
        let ad = a * d;
        let bd = b * d;
        let tmp = (bd >> 32) + (ad & 0xffff_ffff) + (bc & 0xffff_ffff) + (1 << 31);
        self.e += other.e + 64;
        self.f = ac + (ad >> 32) + (bc >> 32) + (tmp >> 32);
    }
    pub fn Times(mut a: Self, b: Self) -> Self {
        a.Multiply(b);
        a
    }
    pub fn Normalize(&mut self) {
        debug_assert_ne!(self.f, 0);
        while self.f & 0xffc0_0000_0000_0000 == 0 {
            self.f <<= 10;
            self.e -= 10;
        }
        while self.f & 0x8000_0000_0000_0000 == 0 {
            self.f <<= 1;
            self.e -= 1;
        }
    }
    pub fn Normalized(mut a: Self) -> Self {
        a.Normalize();
        a
    }
}
