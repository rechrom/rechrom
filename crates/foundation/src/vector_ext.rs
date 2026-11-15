// C++: foundation/blink_base/wtf/vector.h. The translated Vector<T> is
// currently Vec<T>; this trait retains the source's 32-bit index/size
// boundary for callers that still use C++-named collection operations.
use crate::WtfSizeT;

pub trait VectorExt<T> {
    fn size(&self) -> WtfSizeT;
    fn at(&self, index: WtfSizeT) -> &T;
    fn empty(&self) -> bool;
    fn push_back(&mut self, value: T);
    fn ReserveInitialCapacity(&mut self, capacity: WtfSizeT);
    fn Contains(&self, value: &T) -> bool
    where
        T: PartialEq;
}

impl<T> VectorExt<T> for Vec<T> {
    // cpp: foundation/blink_base/wtf/vector.h:1538-1541
    fn size(&self) -> WtfSizeT {
        WtfSizeT::try_from(self.len()).expect("WTF vector size exceeds 32 bits")
    }

    // cpp: foundation/blink_base/wtf/vector.h:1545-1557
    fn at(&self, index: WtfSizeT) -> &T {
        &self[index as usize]
    }

    // cpp: foundation/blink_base/wtf/vector.h:1541
    fn empty(&self) -> bool {
        self.is_empty()
    }

    // cpp: foundation/blink_base/wtf/vector.h:1696-1719,2474-2490
    fn push_back(&mut self, value: T) {
        assert!(
            self.len() < WtfSizeT::MAX as usize,
            "WTF vector size exceeds 32 bits"
        );
        self.push(value);
    }

    // cpp: foundation/blink_base/wtf/vector.h:1666-1668,2417-2433
    fn ReserveInitialCapacity(&mut self, capacity: WtfSizeT) {
        assert!(self.is_empty(), "initial capacity requires an empty vector");
        assert_eq!(self.capacity(), 0, "initial capacity requires a new vector");
        if self.capacity() < capacity as usize {
            self.reserve(capacity as usize - self.len());
        }
    }

    // cpp: foundation/blink_base/wtf/vector.h:1635,2208-2220
    fn Contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.contains(value)
    }
}
