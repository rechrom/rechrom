use crate::{
    kFirstHighPriorityCSSProperty, kLastHighPriorityCSSProperty, kNumCSSPropertyIDs, CSSPropertyID,
};

const STORAGE_CHUNKS: usize = (kNumCSSPropertyIDs as usize + 63) / 64;
const FIRST_HIGH_PRIORITY: usize = kFirstHighPriorityCSSProperty as usize;
const LAST_HIGH_PRIORITY: usize = kLastHighPriorityCSSProperty as usize;
const HIGH_PRIORITY_MASK: u64 =
    ((1u64 << (LAST_HIGH_PRIORITY + 1)) - 1) & !((1u64 << FIRST_HIGH_PRIORITY) - 1);

// cpp: foundation/style_values/css/properties/css_bitset.h:25-43,156-157,186
// The production specialization has the same inline 13-chunk storage as the C++
// CSSBitset. Smaller test specializations use the prefix of that storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CSSBitsetBase<const BITS: usize> {
    chunks_: [u64; STORAGE_CHUNKS],
}

pub type CSSBitset = CSSBitsetBase<{ kNumCSSPropertyIDs as usize }>;

#[allow(non_snake_case)]
impl<const BITS: usize> CSSBitsetBase<BITS> {
    const CHECK_BITS: () = assert!(BITS > 0 && BITS <= kNumCSSPropertyIDs as usize);
    const CHUNKS: usize = (BITS + 63) / 64;

    // cpp: foundation/style_values/css/properties/css_bitset.h:40-55,159-169
    pub const fn new() -> Self {
        let _ = Self::CHECK_BITS;
        Self {
            chunks_: [0; STORAGE_CHUNKS],
        }
    }

    pub const fn FromList(list: &[CSSPropertyID]) -> Self {
        let mut set = Self::new();
        let mut i = 0;
        while i < list.len() {
            let bit = list[i] as usize;
            assert!(bit < BITS);
            set.chunks_[bit / 64] |= 1u64 << (bit % 64);
            i += 1;
        }
        set
    }

    // cpp: foundation/style_values/css/properties/css_bitset.h:61-85
    pub fn HighPriorityBits(&self) -> u64 {
        self.chunks_[0] & HIGH_PRIORITY_MASK
    }

    pub fn Set(&mut self, id: CSSPropertyID) {
        let bit = id as usize;
        assert!(bit < BITS);
        self.chunks_[bit / 64] |= 1u64 << (bit % 64);
    }

    pub fn Has(&self, id: CSSPropertyID) -> bool {
        let bit = id as usize;
        assert!(bit < BITS);
        self.chunks_[bit / 64] & (1u64 << (bit % 64)) != 0
    }

    pub fn HasAny(&self) -> bool {
        self.chunks_[..Self::CHUNKS].iter().any(|chunk| *chunk != 0)
    }

    pub fn Reset(&mut self) {
        self.chunks_[..Self::CHUNKS].fill(0);
    }

    // cpp: foundation/style_values/css/properties/css_bitset.h:144-153
    pub fn begin(&self) -> CSSBitsetIterator<'_, BITS> {
        CSSBitsetIterator::new(&self.chunks_, false)
    }

    pub fn end(&self) -> CSSBitsetIterator<'_, BITS> {
        CSSBitsetIterator::end(&self.chunks_)
    }

    pub fn BeginAfterHighPriority(&self) -> CSSBitsetIterator<'_, BITS> {
        CSSBitsetIterator::new(&self.chunks_, true)
    }
}

impl<const BITS: usize> Default for CSSBitsetBase<BITS> {
    fn default() -> Self {
        Self::new()
    }
}

// cpp: foundation/style_values/css/properties/css_bitset.h:88-142
pub struct CSSBitsetIterator<'a, const BITS: usize> {
    chunks_: &'a [u64; STORAGE_CHUNKS],
    chunk_index_: usize,
    chunk_: u64,
}

impl<'a, const BITS: usize> CSSBitsetIterator<'a, BITS> {
    fn new(chunks: &'a [u64; STORAGE_CHUNKS], after_high_priority: bool) -> Self {
        let chunk = if after_high_priority {
            chunks[0] & !HIGH_PRIORITY_MASK
        } else {
            chunks[0]
        };
        Self {
            chunks_: chunks,
            chunk_index_: 0,
            chunk_: chunk,
        }
    }

    fn end(chunks: &'a [u64; STORAGE_CHUNKS]) -> Self {
        Self {
            chunks_: chunks,
            chunk_index_: (BITS + 63) / 64,
            chunk_: 0,
        }
    }
}

impl<const BITS: usize> Iterator for CSSBitsetIterator<'_, BITS> {
    type Item = CSSPropertyID;

    fn next(&mut self) -> Option<Self::Item> {
        while self.chunk_ == 0 {
            self.chunk_index_ += 1;
            if self.chunk_index_ >= (BITS + 63) / 64 {
                return None;
            }
            self.chunk_ = self.chunks_[self.chunk_index_];
        }
        let index = self.chunk_index_ * 64 + self.chunk_.trailing_zeros() as usize;
        self.chunk_ &= self.chunk_ - 1;
        if index >= BITS {
            return None;
        }
        // css_property_id.rs declares every integer discriminant from 0 to
        // kNumCSSPropertyIDs - 1; the generator checks that contiguity.
        Some(unsafe { std::mem::transmute::<i32, CSSPropertyID>(index as i32) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_bits_and_chunk_boundaries() {
        assert_eq!(std::mem::size_of::<CSSBitset>(), 13 * 8);
        let mut set = CSSBitset::new();
        for id in [
            CSSPropertyID::kInvalid,
            CSSPropertyID::kZoom,
            CSSPropertyID::kInternalForcedVisitedColor,
            CSSPropertyID::kAccentColor,
            CSSPropertyID::kAdditiveSymbols,
            CSSPropertyID::kAliasGridGap,
        ] {
            set.Set(id);
            assert!(set.Has(id));
        }
        assert_eq!(
            set.HighPriorityBits(),
            1u64 << CSSPropertyID::kZoom as usize
        );
        assert_eq!(
            set.begin().collect::<Vec<_>>(),
            [
                CSSPropertyID::kInvalid,
                CSSPropertyID::kZoom,
                CSSPropertyID::kInternalForcedVisitedColor,
                CSSPropertyID::kAccentColor,
                CSSPropertyID::kAdditiveSymbols,
                CSSPropertyID::kAliasGridGap,
            ]
        );
        assert_eq!(
            set.BeginAfterHighPriority().collect::<Vec<_>>(),
            [
                CSSPropertyID::kInvalid,
                CSSPropertyID::kInternalForcedVisitedColor,
                CSSPropertyID::kAccentColor,
                CSSPropertyID::kAdditiveSymbols,
                CSSPropertyID::kAliasGridGap,
            ]
        );
        set.Reset();
        assert!(!set.HasAny());
        assert_eq!(set.begin().count(), 0);
    }

    #[test]
    fn small_const_set() {
        const SET: CSSBitsetBase<64> =
            CSSBitsetBase::FromList(&[CSSPropertyID::kColorScheme, CSSPropertyID::kAccentColor]);
        assert_eq!(
            SET.begin().collect::<Vec<_>>(),
            [CSSPropertyID::kColorScheme, CSSPropertyID::kAccentColor,]
        );
        assert_eq!(
            SET.BeginAfterHighPriority().collect::<Vec<_>>(),
            [CSSPropertyID::kAccentColor,]
        );
    }
}
