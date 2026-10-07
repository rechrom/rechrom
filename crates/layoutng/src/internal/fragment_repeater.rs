#![allow(non_snake_case)]

use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;

use super::block_node::BlockNode;
use super::constraint_space::ConstraintSpace;
use super::layout_box::LayoutBox;

// cpp: layoutng/internal/fragment_repeater.h:19-31
pub struct FragmentRepeater {
    is_first_clone_: bool,
    is_last_fragment_: bool,
}

impl FragmentRepeater {
    // cpp: layoutng/internal/fragment_repeater.h:32-33
    pub fn LayoutRepeatableRoot(
        node: &BlockNode,
        space: &ConstraintSpace,
        break_token: *const BlockBreakToken,
    ) -> *const LayoutResult {
        unsafe { FragmentRepeaterLayoutRepeatableRoot(node, space, break_token) }
    }

    // cpp: layoutng/internal/fragment_repeater.h:34
    pub fn FinishRepeatableRoot(node: &BlockNode) {
        unsafe { FragmentRepeaterFinishRepeatableRoot(node) }
    }

    // cpp: layoutng/internal/fragment_repeater.h:37-40
    #[allow(dead_code)]
    fn DeepCloneRepeatableRoot(box_: &mut LayoutBox) {
        unsafe { FragmentRepeaterDeepCloneRepeatableRoot(box_) }
    }

    // cpp: layoutng/internal/fragment_repeater.h:41-42
    #[allow(dead_code)]
    fn new(is_first_clone: bool, is_last_fragment: bool) -> Self {
        Self {
            is_first_clone_: is_first_clone,
            is_last_fragment_: is_last_fragment,
        }
    }

    // cpp: layoutng/internal/fragment_repeater.h:44-48
    #[allow(dead_code)]
    fn CloneChildFragments(&mut self, cloned_fragment: &PhysicalBoxFragment) {
        unsafe { FragmentRepeaterCloneChildFragments(self, cloned_fragment) }
    }

    // cpp: layoutng/internal/fragment_repeater.h:50
    #[allow(dead_code)]
    fn Repeat(&mut self, other: &LayoutResult) -> *const LayoutResult {
        unsafe { FragmentRepeaterRepeat(self, other) }
    }

    // cpp: layoutng/internal/fragment_repeater.h:52-54
    #[allow(dead_code)]
    fn GetClonableLayoutResult(
        &self,
        layout_box: &LayoutBox,
        fragment: &PhysicalBoxFragment,
    ) -> *const LayoutResult {
        unsafe { FragmentRepeaterGetClonableLayoutResult(self, layout_box, fragment) }
    }
}

// All six implementations belong to //src/layoutng_repetition and remain
// external to this first-round package set.
unsafe extern "Rust" {
    fn FragmentRepeaterLayoutRepeatableRoot(
        node: &BlockNode,
        space: &ConstraintSpace,
        break_token: *const BlockBreakToken,
    ) -> *const LayoutResult;
    fn FragmentRepeaterFinishRepeatableRoot(node: &BlockNode);
    fn FragmentRepeaterDeepCloneRepeatableRoot(box_: &mut LayoutBox);
    fn FragmentRepeaterCloneChildFragments(
        repeater: &mut FragmentRepeater,
        cloned_fragment: &PhysicalBoxFragment,
    );
    fn FragmentRepeaterRepeat(
        repeater: &mut FragmentRepeater,
        other: &LayoutResult,
    ) -> *const LayoutResult;
    fn FragmentRepeaterGetClonableLayoutResult(
        repeater: &FragmentRepeater,
        layout_box: &LayoutBox,
        fragment: &PhysicalBoxFragment,
    ) -> *const LayoutResult;
}
