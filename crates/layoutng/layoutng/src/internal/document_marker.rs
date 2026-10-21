#![allow(non_snake_case, non_upper_case_globals)]

use foundation::{HeapVector, Member, Visitor};

// cpp: layoutng/internal/document_marker.h:43-55
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkerTypeIndex {
    kSpellingMarkerIndex = 0,
    kGrammarMarkerIndex,
    kTextMatchMarkerIndex,
    kCompositionMarkerIndex,
    kActiveSuggestionMarkerIndex,
    kSuggestionMarkerIndex,
    kTextFragmentMarkerIndex,
    kCustomHighlightMarkerIndex,
    kGlicMarkerIndex,
    kPreviewStylusGestureMarkerIndex,
    kMarkerTypeIndexesCount,
}

// A newtype retains the source enum's uint32 mask representation, including
// the value constructed by isolating the lowest bit of a mask.
// cpp: layoutng/internal/document_marker.h:57-68
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkerType(pub u32);

impl MarkerType {
    pub const kSpelling: Self = Self(1 << MarkerTypeIndex::kSpellingMarkerIndex as u32);
    pub const kGrammar: Self = Self(1 << MarkerTypeIndex::kGrammarMarkerIndex as u32);
    pub const kTextMatch: Self = Self(1 << MarkerTypeIndex::kTextMatchMarkerIndex as u32);
    pub const kComposition: Self = Self(1 << MarkerTypeIndex::kCompositionMarkerIndex as u32);
    pub const kActiveSuggestion: Self =
        Self(1 << MarkerTypeIndex::kActiveSuggestionMarkerIndex as u32);
    pub const kSuggestion: Self = Self(1 << MarkerTypeIndex::kSuggestionMarkerIndex as u32);
    pub const kTextFragment: Self = Self(1 << MarkerTypeIndex::kTextFragmentMarkerIndex as u32);
    pub const kCustomHighlight: Self =
        Self(1 << MarkerTypeIndex::kCustomHighlightMarkerIndex as u32);
    pub const kGlic: Self = Self(1 << MarkerTypeIndex::kGlicMarkerIndex as u32);
    pub const kPreviewStylusGesture: Self =
        Self(1 << MarkerTypeIndex::kPreviewStylusGestureMarkerIndex as u32);
}

// cpp: layoutng/internal/document_marker.h:70-110
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkerTypesIterator {
    remaining_types_: u32,
}

#[allow(non_snake_case)]
impl MarkerTypesIterator {
    // cpp: layoutng/internal/document_marker.h:78-80
    pub fn new(marker_types: u32) -> Self {
        Self {
            remaining_types_: marker_types,
        }
    }

    // cpp: layoutng/internal/document_marker.h:82-84
    pub fn Equals(&self, other: &Self) -> bool {
        self.remaining_types_ == other.remaining_types_
    }

    // cpp: layoutng/internal/document_marker.h:86-95
    pub fn Increment(&mut self) -> &mut Self {
        debug_assert!(self.remaining_types_ != 0);
        self.remaining_types_ &= self.remaining_types_.wrapping_sub(1);
        self
    }

    // cpp: layoutng/internal/document_marker.h:97-106
    pub fn Current(&self) -> MarkerType {
        debug_assert!(self.remaining_types_ != 0);
        MarkerType(self.remaining_types_ & (!self.remaining_types_).wrapping_add(1))
    }
}

impl Iterator for MarkerTypesIterator {
    type Item = MarkerType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_types_ == 0 {
            return None;
        }
        let value = self.Current();
        self.Increment();
        Some(value)
    }
}

// cpp: layoutng/internal/document_marker.h:112-178
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MarkerTypes {
    mask_: u32,
}

#[allow(non_snake_case)]
impl MarkerTypes {
    // cpp: layoutng/internal/document_marker.h:116
    pub fn new(mask: u32) -> Self {
        Self { mask_: mask }
    }

    // cpp: layoutng/internal/document_marker.h:118-120
    pub fn All() -> Self {
        Self::new((1u32 << MarkerTypeIndex::kMarkerTypeIndexesCount as u32) - 1)
    }

    // cpp: layoutng/internal/document_marker.h:122-124
    pub fn AllBut(types: &Self) -> Self {
        Self::new(Self::All().mask_ & !types.mask_)
    }

    // cpp: layoutng/internal/document_marker.h:126-129
    pub fn HighlightPseudos() -> Self {
        Self::new(
            MarkerType::kTextFragment.0
                | MarkerType::kSpelling.0
                | MarkerType::kGrammar.0
                | MarkerType::kCustomHighlight.0,
        )
    }

    // cpp: layoutng/internal/document_marker.h:131-149
    pub fn ActiveSuggestion() -> Self {
        Self::new(MarkerType::kActiveSuggestion.0)
    }
    pub fn Composition() -> Self {
        Self::new(MarkerType::kComposition.0)
    }
    pub fn PreviewStylusGesture() -> Self {
        Self::new(MarkerType::kPreviewStylusGesture.0)
    }
    pub fn Grammar() -> Self {
        Self::new(MarkerType::kGrammar.0)
    }
    pub fn Misspelling() -> Self {
        Self::new(MarkerType::kSpelling.0 | MarkerType::kGrammar.0)
    }
    pub fn Spelling() -> Self {
        Self::new(MarkerType::kSpelling.0)
    }
    pub fn TextMatch() -> Self {
        Self::new(MarkerType::kTextMatch.0)
    }
    pub fn Suggestion() -> Self {
        Self::new(MarkerType::kSuggestion.0)
    }
    pub fn TextFragment() -> Self {
        Self::new(MarkerType::kTextFragment.0)
    }
    pub fn CustomHighlight() -> Self {
        Self::new(MarkerType::kCustomHighlight.0)
    }
    pub fn Glic() -> Self {
        Self::new(MarkerType::kGlic.0)
    }

    // cpp: layoutng/internal/document_marker.h:151-154
    pub fn Contains(&self, marker_type: MarkerType) -> bool {
        self.mask_ & marker_type.0 != 0
    }
    pub fn Intersects(&self, types: &Self) -> bool {
        self.mask_ & types.mask_ != 0
    }

    // cpp: layoutng/internal/document_marker.h:155-160
    pub fn IsOneMarkerType(&self) -> Option<MarkerType> {
        if self.mask_.is_power_of_two() {
            Some(MarkerType(self.mask_))
        } else {
            None
        }
    }

    // cpp: layoutng/internal/document_marker.h:161-163
    pub fn Equals(&self, other: &Self) -> bool {
        self.mask_ == other.mask_
    }

    // cpp: layoutng/internal/document_marker.h:165-171
    pub fn Add(&self, types: &Self) -> Self {
        Self::new(self.mask_ | types.mask_)
    }
    pub fn Subtract(&self, types: &Self) -> Self {
        Self::new(self.mask_ & !types.mask_)
    }

    // cpp: layoutng/internal/document_marker.h:173-174
    pub fn begin(&self) -> MarkerTypesIterator {
        MarkerTypesIterator::new(self.mask_)
    }
    pub fn end(&self) -> MarkerTypesIterator {
        MarkerTypesIterator::new(0)
    }
}

impl IntoIterator for MarkerTypes {
    type Item = MarkerType;
    type IntoIter = MarkerTypesIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.begin()
    }
}

// cpp: layoutng/internal/document_marker.h:188-191
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkerOffsets {
    pub start_offset: u32,
    pub end_offset: u32,
}

// C++ subclasses derive from this GC base and provide GetType virtually.
// The Rust base keeps offsets and the unresolved dispatch contract; concrete
// marker subclasses belong to their own source packages.
// The default virtual destructor uses Rust's ordinary value destruction.
// cpp: layoutng/internal/document_marker.h:41-42
// cpp: layoutng/internal/document_marker.h:180-212
// cpp: layoutng/internal/document_marker.cc:36
pub struct DocumentMarker {
    start_offset_: u32,
    end_offset_: u32,
}

#[allow(non_snake_case)]
impl DocumentMarker {
    // cpp: layoutng/internal/document_marker.h:207
    // cpp: layoutng/internal/document_marker.cc:38-41
    // Safety: the base must be embedded in a concrete marker whose type
    // dispatch is supplied by the external marker implementation.
    pub unsafe fn new_for_subclass(start_offset: u32, end_offset: u32) -> Self {
        debug_assert!(start_offset < end_offset);
        Self {
            start_offset_: start_offset,
            end_offset_: end_offset,
        }
    }

    // cpp: layoutng/internal/document_marker.h:182-184
    pub fn GetType(&self) -> MarkerType {
        unsafe { DocumentMarkerGetType(self) }
    }

    // cpp: layoutng/internal/document_marker.h:185-186
    pub fn StartOffset(&self) -> u32 {
        self.start_offset_
    }
    pub fn EndOffset(&self) -> u32 {
        self.end_offset_
    }

    // cpp: layoutng/internal/document_marker.h:193-196
    // cpp: layoutng/internal/document_marker.cc:43-87
    pub fn ComputeOffsetsAfterShift(
        &self,
        offset: u32,
        old_length: u32,
        new_length: u32,
    ) -> Option<MarkerOffsets> {
        let mut result = MarkerOffsets {
            start_offset: self.StartOffset(),
            end_offset: self.EndOffset(),
        };
        let replaced_end = offset.wrapping_add(old_length);
        if self.StartOffset() > offset || (self.StartOffset() == offset && old_length == 0) {
            if self.StartOffset() <= replaced_end {
                result.start_offset = offset.wrapping_add(new_length);
            } else {
                result.start_offset = self
                    .StartOffset()
                    .wrapping_add(new_length)
                    .wrapping_sub(old_length);
            }
        }
        if self.EndOffset() > offset {
            if self.EndOffset() < replaced_end {
                result.end_offset = offset;
            } else {
                result.end_offset = self
                    .EndOffset()
                    .wrapping_add(new_length)
                    .wrapping_sub(old_length);
            }
        }
        if result.start_offset >= result.end_offset {
            return None;
        }
        Some(result)
    }

    // cpp: layoutng/internal/document_marker.h:200-201
    pub fn SetStartOffset(&mut self, offset: u32) {
        self.start_offset_ = offset;
    }
    pub fn SetEndOffset(&mut self, offset: u32) {
        self.end_offset_ = offset;
    }

    // cpp: layoutng/internal/document_marker.h:202
    // cpp: layoutng/internal/document_marker.cc:89-92
    pub fn ShiftOffsets(&mut self, delta: i32) {
        self.start_offset_ = self.start_offset_.wrapping_add(delta as u32);
        self.end_offset_ = self.end_offset_.wrapping_add(delta as u32);
    }

    // cpp: layoutng/internal/document_marker.h:204
    pub fn Trace(&self, _visitor: &mut Visitor) {}
}

// cpp: layoutng/internal/document_marker.h:214
pub type DocumentMarkerVector = HeapVector<Member<DocumentMarker>>;

unsafe extern "Rust" {
    fn DocumentMarkerGetType(marker: &DocumentMarker) -> MarkerType;
}
