// C++: foundation/blink_base/heap/collection_support/heap_vector.h:24-182.
// SmallVec preserves WTF::Vector's inline-capacity contract. A traced
// BasicHeapVector visits its live elements directly; spilled backing storage is
// released when its owner is dropped rather than registered as a separate
// layout-heap object.
use crate::gc_heap::{Traceable, Visitor};
use crate::wtf_size_t::WtfSizeT;
use smallvec::SmallVec;
use std::fmt;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};

// cpp: foundation/blink_base/heap/collection_support/heap_vector.h:23-165
#[repr(transparent)]
pub struct BasicHeapVector<T, const INLINE_CAPACITY: usize, const GCED: bool> {
    elements: SmallVec<[T; INLINE_CAPACITY]>,
}

// cpp: foundation/blink_base/heap/collection_support/heap_vector.h:166-181
pub type HeapVector<T, const INLINE_CAPACITY: usize = 0> =
    BasicHeapVector<T, INLINE_CAPACITY, false>;
pub type GCedHeapVector<T, const INLINE_CAPACITY: usize = 0> =
    BasicHeapVector<T, INLINE_CAPACITY, true>;

impl<T, const N: usize, const GCED: bool> BasicHeapVector<T, N, GCED> {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            elements: SmallVec::new(),
        }
    }

    pub fn with_size(size: WtfSizeT) -> Self
    where
        T: Default,
    {
        let mut vector = Self::new();
        vector.elements.resize_with(size as usize, T::default);
        vector
    }

    pub fn with_value(size: WtfSizeT, value: T) -> Self
    where
        T: Clone,
    {
        let mut vector = Self::new();
        vector.elements.resize(size as usize, value);
        vector
    }

    #[inline(always)]
    pub fn size(&self) -> WtfSizeT {
        WtfSizeT::try_from(self.elements.len()).expect("WTF vector size exceeds 32 bits")
    }

    pub fn at(&self, index: WtfSizeT) -> &T {
        &self.elements[index as usize]
    }

    pub fn at_mut(&mut self, index: WtfSizeT) -> &mut T {
        &mut self.elements[index as usize]
    }

    pub fn IsEmpty(&self) -> bool {
        self.elements.is_empty()
    }

    // cpp: foundation/blink_base/wtf/vector.h:2344-2355
    pub fn Shrink(&mut self, size: WtfSizeT) {
        assert!(size <= self.size());
        self.elements.truncate(size as usize);
    }

    // cpp: foundation/blink_base/wtf/vector.h:1541
    #[inline(always)]
    pub fn empty(&self) -> bool {
        self.elements.is_empty()
    }

    #[inline(always)]
    pub fn ReserveInitialCapacity(&mut self, capacity: WtfSizeT) {
        if self.elements.capacity() < capacity as usize {
            self.elements
                .reserve(capacity as usize - self.elements.len());
        }
    }

    #[inline(always)]
    pub fn push_back(&mut self, value: T) {
        assert!(
            self.elements.len() < WtfSizeT::MAX as usize,
            "WTF vector size exceeds 32 bits"
        );
        self.elements.push(value);
    }

    // cpp: foundation/blink_base/wtf/vector.h:1698-1723,2491
    #[inline(always)]
    pub fn emplace_back(&mut self, value: T) -> &mut T {
        self.push_back(value);
        self.elements.last_mut().expect("just appended an element")
    }

    pub fn extend_from_slice(&mut self, values: &[T])
    where
        T: Clone,
    {
        self.elements.extend(values.iter().cloned());
    }

    pub fn splice<I>(&mut self, range: std::ops::Range<usize>, replace_with: I)
    where
        I: IntoIterator<Item = T>,
    {
        let position = range.start;
        self.elements.drain(range);
        for (offset, value) in replace_with.into_iter().enumerate() {
            self.elements.insert(position + offset, value);
        }
    }

    pub fn AppendVector<const OTHER_N: usize, const OTHER_GCED: bool>(
        &mut self,
        other: &BasicHeapVector<T, OTHER_N, OTHER_GCED>,
    ) where
        T: Clone,
    {
        let new_size = self
            .elements
            .len()
            .checked_add(other.elements.len())
            .expect("WTF vector size overflow");
        assert!(
            new_size <= WtfSizeT::MAX as usize,
            "WTF vector size exceeds 32 bits"
        );
        self.elements.extend(other.elements.iter().cloned());
    }

    // cpp: foundation/blink_base/wtf/vector.h:1746-1765,2655-2661
    pub fn InsertVector<const OTHER_N: usize, const OTHER_GCED: bool>(
        &mut self,
        position: WtfSizeT,
        other: &BasicHeapVector<T, OTHER_N, OTHER_GCED>,
    ) where
        T: Clone,
    {
        let position = position as usize;
        assert!(position <= self.elements.len());
        let new_size = self
            .elements
            .len()
            .checked_add(other.elements.len())
            .expect("WTF vector size overflow");
        assert!(
            new_size <= WtfSizeT::MAX as usize,
            "WTF vector size exceeds 32 bits"
        );
        self.elements.reserve(other.elements.len());
        for (offset, value) in other.elements.iter().cloned().enumerate() {
            self.elements.insert(position + offset, value);
        }
    }
}

impl<T, const N: usize, const GCED: bool> Default for BasicHeapVector<T, N, GCED> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone, const N: usize, const GCED: bool> Clone for BasicHeapVector<T, N, GCED> {
    fn clone(&self) -> Self {
        Self {
            elements: self.elements.clone(),
        }
    }
}

impl<T: fmt::Debug, const N: usize, const GCED: bool> fmt::Debug for BasicHeapVector<T, N, GCED> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.elements.fmt(f)
    }
}

impl<T: PartialEq, const N: usize, const GCED: bool> PartialEq for BasicHeapVector<T, N, GCED> {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
    }
}

impl<T: Eq, const N: usize, const GCED: bool> Eq for BasicHeapVector<T, N, GCED> {}

impl<T, const N: usize, const GCED: bool> Deref for BasicHeapVector<T, N, GCED> {
    type Target = SmallVec<[T; N]>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}

impl<T, const N: usize, const GCED: bool> DerefMut for BasicHeapVector<T, N, GCED> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elements
    }
}

impl<T, const N: usize, const GCED: bool> From<Vec<T>> for BasicHeapVector<T, N, GCED> {
    fn from(elements: Vec<T>) -> Self {
        assert!(
            elements.len() <= WtfSizeT::MAX as usize,
            "WTF vector size exceeds 32 bits"
        );
        Self {
            elements: SmallVec::from_vec(elements),
        }
    }
}

impl<T, const N: usize, const GCED: bool> From<BasicHeapVector<T, N, GCED>> for Vec<T> {
    fn from(vector: BasicHeapVector<T, N, GCED>) -> Self {
        vector.elements.into_vec()
    }
}

impl<T, const N: usize, const GCED: bool> FromIterator<T> for BasicHeapVector<T, N, GCED> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            elements: iter.into_iter().collect(),
        }
    }
}

impl<T, const N: usize, const GCED: bool> IntoIterator for BasicHeapVector<T, N, GCED> {
    type Item = T;
    type IntoIter = smallvec::IntoIter<[T; N]>;
    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a, T, const N: usize, const GCED: bool> IntoIterator for &'a BasicHeapVector<T, N, GCED> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

impl<'a, T, const N: usize, const GCED: bool> IntoIterator for &'a mut BasicHeapVector<T, N, GCED> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter_mut()
    }
}

impl<T: Traceable, const N: usize, const GCED: bool> Traceable for BasicHeapVector<T, N, GCED> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        for element in &self.elements {
            visitor.Trace(element);
        }
    }
}
