//! Fragmentation-specific `ConstraintSpace` projection.

use foundation::{MakeGarbageCollected, Member};

use super::{ConstraintSpace, FragmentationType};

impl ConstraintSpace {
    // cpp: layoutng/internal/constraint_space.h:142-152
    #[inline(always)]
    pub fn CloneWithoutFragmentation(&self) -> Self {
        clone_without_fragmentation(self)
    }
}

// Keep the sampled symbol package-qualified. Rust's symbol for an inherent
// method names the type but not the module containing its impl.
#[inline(never)]
fn clone_without_fragmentation(space: &ConstraintSpace) -> ConstraintSpace {
    debug_assert!(space.HasBlockFragmentation());
    let mut copy = space.clone();
    let rare_data = copy.rare_data_.Get();
    debug_assert!(!rare_data.is_null());
    let mut cloned_rare_data = unsafe { &*rare_data }.clone();
    cloned_rare_data.block_direction_fragmentation_type = FragmentationType::kFragmentNone;
    cloned_rare_data.is_block_fragmentation_forced_off = true;
    copy.rare_data_ = Member::from_ptr(MakeGarbageCollected(cloned_rare_data));
    copy
}
