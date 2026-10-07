use std::ops::Deref;

use foundation::{Member, PhysicalOffset, ThreadAffinity, ThreadingTrait, Visitor};

use crate::physical_fragment::PhysicalFragment;

// This value is stored in a C-style fragment array. Its fields have no Drop
// behavior; fragment reference counting remains with PhysicalFragment.
// cpp: layoutng_fragment_tree/physical_fragment_link.h:25-27
// cpp: layoutng_fragment_tree/physical_fragment_link.h:39-40
pub struct PhysicalFragmentLink {
    pub fragment: Member<PhysicalFragment>,
    pub offset: PhysicalOffset,
}

#[allow(non_snake_case)]
impl PhysicalFragmentLink {
    // cpp: layoutng_fragment_tree/physical_fragment_link.h:30
    pub fn Offset(&self) -> PhysicalOffset {
        self.offset
    }

    // cpp: layoutng_fragment_tree/physical_fragment_link.h:31
    pub fn get(&self) -> *const PhysicalFragment {
        self.fragment.Get()
    }

    // cpp: layoutng_fragment_tree/physical_fragment_link.h:33
    pub fn is_present(&self) -> bool {
        !self.get().is_null()
    }

    // cpp: layoutng_fragment_tree/physical_fragment_link.h:37
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.fragment);
    }
}

impl Deref for PhysicalFragmentLink {
    type Target = PhysicalFragment;

    // cpp: layoutng_fragment_tree/physical_fragment_link.h:34-35
    fn deref(&self) -> &Self::Target {
        unsafe { self.get().as_ref() }.expect("null PhysicalFragmentLink")
    }
}

// cpp: layoutng_fragment_tree/physical_fragment_link.h:43-47
impl ThreadingTrait for PhysicalFragmentLink {
    const kAffinity: ThreadAffinity = ThreadAffinity::kMainThreadOnly;
}

// cpp: layoutng_fragment_tree/physical_fragment_link.h:49-57
pub struct PhysicalFragmentLinkVectorTraits;

#[allow(non_upper_case_globals)]
impl PhysicalFragmentLinkVectorTraits {
    pub const kCanInitializeWithMemset: bool = true;
    pub const kCanClearUnusedSlotsWithMemset: bool = true;
    pub const kCanMoveWithMemcpy: bool = true;
    pub const kCanCopyWithMemcpy: bool = true;
    pub const kCanTraceConcurrently: bool = true;
}
