use std::ops::Deref;

use foundation::{HeapVector, Member, Visitor};
use layoutng_geometry::geometry::logical_offset::LogicalOffset;

use crate::physical_fragment::PhysicalFragment;

// C++ DISALLOW_NEW limits allocation, not value behavior. The stored Member
// remains a GC handle and the logical offset remains a value.
// cpp: layoutng_fragment_tree/logical_fragment_link.h:19-20
// cpp: layoutng_fragment_tree/logical_fragment_link.h:36-37
pub struct LogicalFragmentLink {
    pub fragment: Member<PhysicalFragment>,
    pub offset: LogicalOffset,
}

#[allow(non_snake_case)]
impl LogicalFragmentLink {
    // cpp: layoutng_fragment_tree/logical_fragment_link.h:24-25
    pub fn new(fragment: &PhysicalFragment, offset: LogicalOffset) -> Self {
        Self {
            fragment: Member::from_ptr(
                fragment as *const PhysicalFragment as *mut PhysicalFragment,
            ),
            offset,
        }
    }

    // cpp: layoutng_fragment_tree/logical_fragment_link.h:27
    pub fn Offset(&self) -> &LogicalOffset {
        &self.offset
    }

    // cpp: layoutng_fragment_tree/logical_fragment_link.h:28
    pub fn get(&self) -> *const PhysicalFragment {
        self.fragment.Get()
    }

    // cpp: layoutng_fragment_tree/logical_fragment_link.h:30
    pub fn is_present(&self) -> bool {
        !self.get().is_null()
    }

    // cpp: layoutng_fragment_tree/logical_fragment_link.h:34
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.fragment);
    }
}

impl Default for LogicalFragmentLink {
    // cpp: layoutng_fragment_tree/logical_fragment_link.h:23
    fn default() -> Self {
        Self {
            fragment: Member::default(),
            offset: LogicalOffset::default(),
        }
    }
}

impl Deref for LogicalFragmentLink {
    type Target = PhysicalFragment;

    // cpp: layoutng_fragment_tree/logical_fragment_link.h:31-32
    fn deref(&self) -> &Self::Target {
        unsafe { self.get().as_ref() }.expect("null LogicalFragmentLink")
    }
}

// cpp: layoutng_fragment_tree/logical_fragment_link.h:40
pub type LogicalFragmentLinkVector = HeapVector<LogicalFragmentLink, 4>;

// The C++ vector-traits macro opts this value into zero initialization,
// bytewise move, and bytewise comparison. Copy permission still follows the
// foundation vector trait's default for the concrete Member representation.
// cpp: layoutng_fragment_tree/logical_fragment_link.h:44
pub struct LogicalFragmentLinkVectorTraits;

#[allow(non_upper_case_globals)]
impl LogicalFragmentLinkVectorTraits {
    pub const kCanInitializeWithMemset: bool = true;
    pub const kCanClearUnusedSlotsWithMemset: bool = true;
    pub const kCanMoveWithMemcpy: bool = true;
    pub const kCanCompareWithMemcmp: bool = true;
}
