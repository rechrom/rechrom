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
// Source: bignum.h:39-173; bignum.cc:35-117,154-566,645-793.
// Chromium's fixed-capacity, 28-bit bigits plus implicit low-zero exponent.
const kBigitSize: usize = 28;
const kBigitMask: u32 = (1 << kBigitSize) - 1;
const kBigitCapacity: usize = 3584 / kBigitSize;
pub(super) struct Bignum {
    used: usize,
    exponent: usize,
    bigits: [u32; kBigitCapacity],
}
impl Bignum {
    pub fn new() -> Self {
        Self {
            used: 0,
            exponent: 0,
            bigits: [0; kBigitCapacity],
        }
    }
    fn EnsureCapacity(size: usize) {
        assert!(size <= kBigitCapacity);
    }
    fn Zero(&mut self) {
        self.used = 0;
        self.exponent = 0;
    }
    fn BigitLength(&self) -> usize {
        self.used + self.exponent
    }
    fn IsClamped(&self) -> bool {
        self.used == 0 || self.bigits[self.used - 1] != 0
    }
    fn Clamp(&mut self) {
        while self.used > 0 && self.bigits[self.used - 1] == 0 {
            self.used -= 1;
        }
        if self.used == 0 {
            self.exponent = 0;
        }
    }
    pub fn AssignUInt16(&mut self, value: u16) {
        self.Zero();
        if value > 0 {
            self.bigits[0] = value as u32;
            self.used = 1;
        }
    }
    pub fn AssignUInt64(&mut self, mut value: u64) {
        self.Zero();
        while value > 0 {
            self.bigits[self.used] = value as u32 & kBigitMask;
            value >>= kBigitSize;
            self.used += 1;
        }
    }
    pub fn AssignBignum(&mut self, other: &Self) {
        self.exponent = other.exponent;
        self.bigits[..other.used].copy_from_slice(&other.bigits[..other.used]);
        self.used = other.used;
    }
    pub fn AssignDecimalString(&mut self, value: &[u8]) {
        self.Zero();
        let mut pos = 0;
        let mut length = value.len();
        while length >= 19 {
            let digits = ReadUInt64(&value[pos..pos + 19]);
            pos += 19;
            length -= 19;
            self.MultiplyByPowerOfTen(19);
            self.AddUInt64(digits);
        }
        let digits = ReadUInt64(&value[pos..pos + length]);
        self.MultiplyByPowerOfTen(length as i32);
        self.AddUInt64(digits);
        self.Clamp();
    }
    fn AddUInt64(&mut self, operand: u64) {
        if operand == 0 {
            return;
        }
        let mut other = Self::new();
        other.AssignUInt64(operand);
        self.AddBignum(&other);
    }
    fn AddBignum(&mut self, other: &Self) {
        debug_assert!(self.IsClamped() && other.IsClamped());
        self.Align(other);
        Self::EnsureCapacity(1 + self.BigitLength().max(other.BigitLength()) - self.exponent);
        let mut carry = 0;
        let mut pos = other.exponent - self.exponent;
        for i in self.used..pos {
            self.bigits[i] = 0;
        }
        for i in 0..other.used {
            let mine = if pos < self.used { self.bigits[pos] } else { 0 };
            let sum = mine + other.bigits[i] + carry;
            self.bigits[pos] = sum & kBigitMask;
            carry = sum >> kBigitSize;
            pos += 1;
        }
        while carry != 0 {
            let mine = if pos < self.used { self.bigits[pos] } else { 0 };
            let sum = mine + carry;
            self.bigits[pos] = sum & kBigitMask;
            carry = sum >> kBigitSize;
            pos += 1;
        }
        self.used = self.used.max(pos);
        debug_assert!(self.IsClamped());
    }
    fn SubtractBignum(&mut self, other: &Self) {
        debug_assert!(Self::Compare(other, self) <= 0);
        self.Align(other);
        let offset = other.exponent - self.exponent;
        let mut borrow = 0;
        let mut i = 0;
        while i < other.used {
            let diff = self.bigits[i + offset]
                .wrapping_sub(other.bigits[i])
                .wrapping_sub(borrow);
            self.bigits[i + offset] = diff & kBigitMask;
            borrow = diff >> 31;
            i += 1;
        }
        while borrow != 0 {
            let diff = self.bigits[i + offset].wrapping_sub(borrow);
            self.bigits[i + offset] = diff & kBigitMask;
            borrow = diff >> 31;
            i += 1;
        }
        self.Clamp();
    }
    pub fn ShiftLeft(&mut self, shift: i32) {
        debug_assert!(shift >= 0);
        if self.used == 0 {
            return;
        }
        self.exponent += shift as usize / kBigitSize;
        Self::EnsureCapacity(self.used + 1);
        self.BigitsShiftLeft(shift as usize % kBigitSize);
    }
    pub fn MultiplyByUInt32(&mut self, factor: u32) {
        if factor == 1 {
            return;
        }
        if factor == 0 {
            self.Zero();
            return;
        }
        if self.used == 0 {
            return;
        }
        let mut carry = 0u64;
        for i in 0..self.used {
            let product = factor as u64 * self.bigits[i] as u64 + carry;
            self.bigits[i] = product as u32 & kBigitMask;
            carry = product >> kBigitSize;
        }
        while carry != 0 {
            Self::EnsureCapacity(self.used + 1);
            self.bigits[self.used] = carry as u32 & kBigitMask;
            self.used += 1;
            carry >>= kBigitSize;
        }
    }
    pub fn MultiplyByUInt64(&mut self, factor: u64) {
        if factor == 1 {
            return;
        }
        if factor == 0 {
            self.Zero();
            return;
        }
        if self.used == 0 {
            return;
        }
        let low = factor & 0xffff_ffff;
        let high = factor >> 32;
        let mut carry = 0u64;
        for i in 0..self.used {
            let product_low = low * self.bigits[i] as u64;
            let product_high = high * self.bigits[i] as u64;
            let tmp = (carry & kBigitMask as u64) + product_low;
            self.bigits[i] = tmp as u32 & kBigitMask;
            carry =
                (carry >> kBigitSize) + (tmp >> kBigitSize) + (product_high << (32 - kBigitSize));
        }
        while carry != 0 {
            Self::EnsureCapacity(self.used + 1);
            self.bigits[self.used] = carry as u32 & kBigitMask;
            self.used += 1;
            carry >>= kBigitSize;
        }
    }
    pub fn MultiplyByPowerOfTen(&mut self, exponent: i32) {
        const kFive1To12: [u32; 12] = [
            5, 25, 125, 625, 3125, 15625, 78125, 390625, 1953125, 9765625, 48828125, 244140625,
        ];
        debug_assert!(exponent >= 0);
        if exponent == 0 || self.used == 0 {
            return;
        }
        let mut remaining = exponent;
        while remaining >= 27 {
            self.MultiplyByUInt64(0x6765c793_fa10079d);
            remaining -= 27;
        }
        while remaining >= 13 {
            self.MultiplyByUInt32(1220703125);
            remaining -= 13;
        }
        if remaining > 0 {
            self.MultiplyByUInt32(kFive1To12[remaining as usize - 1]);
        }
        self.ShiftLeft(exponent);
    }
    pub fn Times10(&mut self) {
        self.MultiplyByUInt32(10);
    }
    // Source: bignum.cc:354-429. Comba multiplication, retaining its in-place
    // overlap order rather than constructing a second arbitrary precision model.
    fn Square(&mut self) {
        let product_length = 2 * self.used;
        Self::EnsureCapacity(product_length);
        assert!(self.used < 1 << (2 * (32 - kBigitSize)));
        let mut accumulator = 0u64;
        let copy_offset = self.used;
        for i in 0..self.used {
            self.bigits[copy_offset + i] = self.bigits[i];
        }
        for i in 0..self.used {
            for j in 0..=i {
                accumulator +=
                    self.bigits[copy_offset + i - j] as u64 * self.bigits[copy_offset + j] as u64;
            }
            self.bigits[i] = accumulator as u32 & kBigitMask;
            accumulator >>= kBigitSize;
        }
        for i in self.used..product_length {
            let mut index1 = self.used - 1;
            let mut index2 = i - index1;
            while index2 < self.used {
                accumulator += self.bigits[copy_offset + index1] as u64
                    * self.bigits[copy_offset + index2] as u64;
                index1 = index1.wrapping_sub(1);
                index2 += 1;
            }
            self.bigits[i] = accumulator as u32 & kBigitMask;
            accumulator >>= kBigitSize;
        }
        debug_assert_eq!(accumulator, 0);
        self.used = product_length;
        self.exponent *= 2;
        self.Clamp();
    }
    pub fn AssignPowerUInt16(&mut self, mut base: u16, power: i32) {
        debug_assert!(base != 0 && power >= 0);
        if power == 0 {
            self.AssignUInt16(1);
            return;
        }
        self.Zero();
        let mut shifts = 0;
        while base & 1 == 0 {
            base >>= 1;
            shifts += 1;
        }
        let mut bits = 0;
        let mut tmp_base = base;
        while tmp_base != 0 {
            tmp_base >>= 1;
            bits += 1;
        }
        Self::EnsureCapacity((bits * power) as usize / kBigitSize + 2);
        let mut mask = 1;
        while power >= mask {
            mask <<= 1;
        }
        mask >>= 2;
        let mut value = base as u64;
        let mut delayed = false;
        while mask != 0 && value <= 0xffff_ffff {
            value *= value;
            if power & mask != 0 {
                debug_assert!(bits > 0);
                let high_mask = !((1u64 << (64 - bits)) - 1);
                if value & high_mask == 0 {
                    value *= base as u64;
                } else {
                    delayed = true;
                }
            }
            mask >>= 1;
        }
        self.AssignUInt64(value);
        if delayed {
            self.MultiplyByUInt32(base as u32);
        }
        while mask != 0 {
            self.Square();
            if power & mask != 0 {
                self.MultiplyByUInt32(base as u32);
            }
            mask >>= 1;
        }
        self.ShiftLeft(shifts * power);
    }
    // Source: bignum.cc:498-566. Small quotient digit extraction.
    pub fn DivideModuloIntBignum(&mut self, other: &Self) -> u16 {
        debug_assert!(self.IsClamped() && other.IsClamped() && other.used > 0);
        if self.BigitLength() < other.BigitLength() {
            return 0;
        }
        self.Align(other);
        let mut result = 0u16;
        while self.BigitLength() > other.BigitLength() {
            debug_assert!(other.bigits[other.used - 1] >= (1 << kBigitSize) / 16);
            let top = self.bigits[self.used - 1];
            debug_assert!(top < 0x10000);
            result += top as u16;
            self.SubtractTimes(other, top);
        }
        let this_bigit = self.bigits[self.used - 1];
        let other_bigit = other.bigits[other.used - 1];
        if other.used == 1 {
            let quotient = this_bigit / other_bigit;
            self.bigits[self.used - 1] = this_bigit - other_bigit * quotient;
            debug_assert!(quotient < 0x10000);
            result += quotient as u16;
            self.Clamp();
            return result;
        }
        let estimate = this_bigit / (other_bigit + 1);
        debug_assert!(estimate < 0x10000);
        result += estimate as u16;
        self.SubtractTimes(other, estimate);
        if other_bigit * (estimate + 1) > this_bigit {
            return result;
        }
        while Self::Compare(other, self) <= 0 {
            self.SubtractBignum(other);
            result += 1;
        }
        result
    }
    fn BigitOrZero(&self, index: usize) -> u32 {
        if index >= self.BigitLength() || index < self.exponent {
            0
        } else {
            self.bigits[index - self.exponent]
        }
    }
    pub fn Compare(a: &Self, b: &Self) -> i32 {
        debug_assert!(a.IsClamped() && b.IsClamped());
        if a.BigitLength() < b.BigitLength() {
            return -1;
        }
        if a.BigitLength() > b.BigitLength() {
            return 1;
        }
        for i in (a.exponent.min(b.exponent)..a.BigitLength()).rev() {
            let x = a.BigitOrZero(i);
            let y = b.BigitOrZero(i);
            if x < y {
                return -1;
            }
            if x > y {
                return 1;
            }
        }
        0
    }
    pub fn PlusCompare(a: &Self, b: &Self, c: &Self) -> i32 {
        if a.BigitLength() < b.BigitLength() {
            return Self::PlusCompare(b, a, c);
        }
        if a.BigitLength() + 1 < c.BigitLength() {
            return -1;
        }
        if a.BigitLength() > c.BigitLength() {
            return 1;
        }
        if a.exponent >= b.BigitLength() && a.BigitLength() < c.BigitLength() {
            return -1;
        }
        let mut borrow = 0u32;
        let min_exponent = a.exponent.min(b.exponent).min(c.exponent);
        for i in (min_exponent..c.BigitLength()).rev() {
            let sum = a.BigitOrZero(i) + b.BigitOrZero(i);
            let chunk = c.BigitOrZero(i);
            if sum > chunk + borrow {
                return 1;
            }
            borrow = chunk + borrow - sum;
            if borrow > 1 {
                return -1;
            }
            borrow <<= kBigitSize;
        }
        if borrow == 0 {
            0
        } else {
            -1
        }
    }
    fn Align(&mut self, other: &Self) {
        if self.exponent <= other.exponent {
            return;
        }
        let zeros = self.exponent - other.exponent;
        Self::EnsureCapacity(self.used + zeros);
        for i in (0..self.used).rev() {
            self.bigits[i + zeros] = self.bigits[i];
        }
        self.bigits[..zeros].fill(0);
        self.used += zeros;
        self.exponent -= zeros;
    }
    fn BigitsShiftLeft(&mut self, shift: usize) {
        debug_assert!(shift < kBigitSize);
        let mut carry = 0;
        for i in 0..self.used {
            let new_carry = self.bigits[i] >> (kBigitSize - shift);
            self.bigits[i] = ((self.bigits[i] << shift) + carry) & kBigitMask;
            carry = new_carry;
        }
        if carry != 0 {
            self.bigits[self.used] = carry;
            self.used += 1;
        }
    }
    fn SubtractTimes(&mut self, other: &Self, factor: u32) {
        debug_assert!(self.exponent <= other.exponent);
        if factor < 3 {
            for _ in 0..factor {
                self.SubtractBignum(other);
            }
            return;
        }
        let mut borrow = 0u32;
        let diff = other.exponent - self.exponent;
        for i in 0..other.used {
            let product = factor as u64 * other.bigits[i] as u64;
            let remove = borrow as u64 + product;
            let difference =
                self.bigits[i + diff].wrapping_sub((remove & kBigitMask as u64) as u32);
            self.bigits[i + diff] = difference & kBigitMask;
            borrow = ((difference >> 31) as u64 + (remove >> kBigitSize)) as u32;
        }
        for i in other.used + diff..self.used {
            if borrow == 0 {
                return;
            }
            let difference = self.bigits[i].wrapping_sub(borrow);
            self.bigits[i] = difference & kBigitMask;
            borrow = difference >> 31;
        }
        self.Clamp();
    }
}
fn ReadUInt64(buffer: &[u8]) -> u64 {
    let mut result = 0;
    for &byte in buffer {
        debug_assert!(byte.is_ascii_digit());
        result = result * 10 + (byte - b'0') as u64;
    }
    result
}
