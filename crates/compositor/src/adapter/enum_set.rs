//! Platform-neutral typed bitset boundary for `base::EnumSet`.
//! This adapter preserves the value/range/iteration semantics from
//! `base/containers/enum_set.h`; it is not counted as a cc source translation.

use std::cmp::Ordering;
use std::marker::PhantomData;
use std::ops::{BitAnd, BitOr, Sub};

/// A source enum whose domain includes every integer in the selected set range.
pub trait EnumValue: Copy + Eq {
    fn to_integer(self) -> i32;
    fn from_integer(value: i32) -> Self;
}

/// Bits are indexed relative to MIN. Unlike an enum-keyed tree set, this also
/// retains the source bitmask ordering and contiguous range semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EnumSet<E, const MIN: i32, const MAX: i32> {
    // Every EnumSet domain used by the translated compositor core fits in the
    // source's inline IntegralBitset path. Keep the bits inline as well: this
    // type is copied in property-tree and damage hot paths.
    bits: u64,
    marker: PhantomData<E>,
}

impl<E: EnumValue, const MIN: i32, const MAX: i32> Default for EnumSet<E, MIN, MAX> {
    fn default() -> Self {
        assert!(MIN <= MAX);
        assert!(Self::value_count() <= u64::BITS as usize);
        Self {
            bits: 0,
            marker: PhantomData,
        }
    }
}

impl<E: EnumValue, const MIN: i32, const MAX: i32> EnumSet<E, MIN, MAX> {
    pub fn value_count() -> usize {
        (MAX as i64 - MIN as i64 + 1) as usize
    }
    fn in_range(value: E) -> bool {
        value.to_integer() >= MIN && value.to_integer() <= MAX
    }
    fn to_index(value: E) -> usize {
        assert!(Self::in_range(value));
        (value.to_integer() as i64 - MIN as i64) as usize
    }
    fn from_index(index: usize) -> E {
        debug_assert!(index < Self::value_count());
        E::from_integer((MIN as i64 + index as i64) as i32)
    }
    fn test(&self, index: usize) -> bool {
        self.bits & (1 << index) != 0
    }
    fn set(&mut self, index: usize) {
        self.bits |= 1 << index;
    }
    fn reset(&mut self, index: usize) {
        self.bits &= !(1 << index);
    }
    fn mask_last_word(&mut self) {
        let count = Self::value_count();
        if count < u64::BITS as usize {
            self.bits &= (1 << count) - 1;
        }
    }
    pub fn new(values: impl IntoIterator<Item = E>) -> Self {
        let mut result = Self::default();
        for value in values {
            result.Put(value);
        }
        result
    }
    pub fn All() -> Self {
        let mut result = Self::default();
        result.bits = u64::MAX;
        result.mask_last_word();
        result
    }
    pub fn FromRange(start: E, end: E) -> Self {
        let mut result = Self::default();
        result.PutRange(start, end);
        result
    }
    pub fn FromEnumBitmask(bitmask: u64) -> Self {
        assert!(MIN >= 0 && MAX < 64);
        let mut result = Self::default();
        result.bits = bitmask >> MIN;
        result.mask_last_word();
        result
    }
    pub fn ToEnumBitmask(&self) -> u64 {
        assert!(MIN >= 0 && MAX < 64);
        self.bits << MIN
    }
    pub fn GetNth64bitWordBitmask(&self, n: usize) -> Option<u64> {
        assert!(MIN >= 0);
        if MAX as usize / 64 < n {
            return None;
        }
        let mut result = 0;
        let start = n.checked_mul(64)?;
        for bit in 0..64 {
            let integer = start + bit;
            if integer >= MIN as usize
                && integer <= MAX as usize
                && self.test(integer - MIN as usize)
            {
                result |= 1 << bit;
            }
        }
        Some(result)
    }
    pub fn Put(&mut self, value: E) {
        self.set(Self::to_index(value));
    }
    pub fn PutAll(&mut self, other: &Self) {
        self.bits |= other.bits;
    }
    pub fn PutRange(&mut self, start: E, end: E) {
        assert!(start.to_integer() <= end.to_integer());
        let end_index_inclusive = Self::to_index(end);
        for current in Self::to_index(start)..=end_index_inclusive {
            self.set(current);
        }
    }
    pub fn RetainAll(&mut self, other: &Self) {
        self.bits &= other.bits;
    }
    pub fn Remove(&mut self, value: E) {
        if Self::in_range(value) {
            self.reset(Self::to_index(value));
        }
    }
    pub fn RemoveAll(&mut self, other: &Self) {
        self.bits &= !other.bits;
    }
    pub fn Clear(&mut self) {
        self.bits = 0;
    }
    pub fn PutOrRemove(&mut self, value: E, should_be_present: bool) {
        if should_be_present {
            self.Put(value);
        } else {
            self.Remove(value);
        }
    }
    pub fn Has(&self, value: E) -> bool {
        Self::in_range(value) && self.test(Self::to_index(value))
    }
    pub fn HasAll(&self, other: &Self) -> bool {
        self.bits & other.bits == other.bits
    }
    pub fn HasAny(&self, other: &Self) -> bool {
        self.bits & other.bits != 0
    }
    pub fn empty(&self) -> bool {
        self.bits == 0
    }
    pub fn size(&self) -> usize {
        self.bits.count_ones() as usize
    }
    pub fn ToString(&self) -> String {
        (0..Self::value_count())
            .rev()
            .map(|index| if self.test(index) { '1' } else { '0' })
            .collect()
    }
    pub fn begin(&self) -> EnumSetIterator<'_, E, MIN, MAX> {
        let mut index = 0;
        while index < Self::value_count() && !self.test(index) {
            index += 1;
        }
        EnumSetIterator { set: self, index }
    }
    pub fn end(&self) -> EnumSetIterator<'_, E, MIN, MAX> {
        EnumSetIterator {
            set: self,
            index: Self::value_count(),
        }
    }
    pub fn Union(mut self, other: &Self) -> Self {
        self.PutAll(other);
        self
    }
    pub fn Intersection(mut self, other: &Self) -> Self {
        self.RetainAll(other);
        self
    }
    pub fn Difference(mut self, other: &Self) -> Self {
        self.RemoveAll(other);
        self
    }
}

pub struct EnumSetIterator<'a, E, const MIN: i32, const MAX: i32> {
    set: &'a EnumSet<E, MIN, MAX>,
    index: usize,
}

impl<E, const MIN: i32, const MAX: i32> PartialEq for EnumSetIterator<'_, E, MIN, MAX> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl<E, const MIN: i32, const MAX: i32> Eq for EnumSetIterator<'_, E, MIN, MAX> {}

impl<E: EnumValue, const MIN: i32, const MAX: i32> Iterator for EnumSetIterator<'_, E, MIN, MAX> {
    type Item = E;
    fn next(&mut self) -> Option<E> {
        while self.index < EnumSet::<E, MIN, MAX>::value_count() && !self.set.test(self.index) {
            self.index += 1;
        }
        if self.index == EnumSet::<E, MIN, MAX>::value_count() {
            return None;
        }
        let value = EnumSet::<E, MIN, MAX>::from_index(self.index);
        self.index += 1;
        while self.index < EnumSet::<E, MIN, MAX>::value_count() && !self.set.test(self.index) {
            self.index += 1;
        }
        Some(value)
    }
}
impl<'a, E: EnumValue, const MIN: i32, const MAX: i32> IntoIterator for &'a EnumSet<E, MIN, MAX> {
    type Item = E;
    type IntoIter = EnumSetIterator<'a, E, MIN, MAX>;
    fn into_iter(self) -> Self::IntoIter {
        self.begin()
    }
}
impl<E: EnumValue, const MIN: i32, const MAX: i32> PartialOrd for EnumSet<E, MIN, MAX> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<E: EnumValue, const MIN: i32, const MAX: i32> Ord for EnumSet<E, MIN, MAX> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bits.cmp(&other.bits)
    }
}
impl<E: EnumValue, const MIN: i32, const MAX: i32> BitOr for EnumSet<E, MIN, MAX> {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        self.Union(&other)
    }
}
impl<E: EnumValue, const MIN: i32, const MAX: i32> BitAnd for EnumSet<E, MIN, MAX> {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        self.Intersection(&other)
    }
}
impl<E: EnumValue, const MIN: i32, const MAX: i32> Sub for EnumSet<E, MIN, MAX> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self.Difference(&other)
    }
}
