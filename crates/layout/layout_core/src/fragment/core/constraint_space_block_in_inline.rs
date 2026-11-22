//! Block-in-inline `ConstraintSpace` projection.

use foundation::{MakeGarbageCollected, Member};

use super::ConstraintSpace;

impl ConstraintSpace {
    // cpp: layoutng/internal/constraint_space.cc:29-55
    #[inline(always)]
    pub fn CloneForBlockInInlineIfNeeded<'a>(
        &'a self,
        space: &'a mut Option<ConstraintSpace>,
    ) -> &'a ConstraintSpace {
        if !self.ShouldTextBoxTrimNodeEnd() {
            debug_assert!(!self.ShouldForceTextBoxTrimEnd());
            return self;
        }
        clone_for_block_in_inline(self, space)
    }
}

// Keep the allocating slow-path symbol package-qualified; inherent method
// symbols omit the module in which their impl is physically defined.
#[inline(never)]
fn clone_for_block_in_inline<'a>(
    source: &'a ConstraintSpace,
    space: &'a mut Option<ConstraintSpace>,
) -> &'a ConstraintSpace {
    *space = Some(source.clone());
    let copy = space.as_mut().expect("block-in-inline constraint clone");
    let original_rare_data = copy.rare_data_.Get();
    debug_assert!(!original_rare_data.is_null());
    let mut rare_data = unsafe { &*original_rare_data }.clone();
    if source.ShouldForceTextBoxTrimEnd() {
        rare_data.should_force_text_box_trim_end = false;
    } else {
        rare_data.should_text_box_trim_node_end = false;
        rare_data.should_text_box_trim_fragmentainer_end = false;
    }
    copy.rare_data_ = Member::from_ptr(MakeGarbageCollected(rare_data));
    copy
}
