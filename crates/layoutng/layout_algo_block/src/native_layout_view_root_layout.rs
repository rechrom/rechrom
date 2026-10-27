//! Root block-layout entry owned by the block algorithm package.

use super::{
    BlockNode, ConstraintSpaceBuilder, LayoutBox, LayoutUnit, LayoutView, MinMaxSizesFloatInput,
    SizeType, ToLogicalSize,
};

impl LayoutView {
    // cpp: layoutng/internal/layout_view_data.cc:34-45
    #[inline(never)]
    pub fn ComputeMinimumWidth(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        let style = self.StyleRef();
        let mode = style.GetWritingMode();
        let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
            mode,
            style.GetWritingDirection(),
            true,
            true,
            false,
        );
        builder.SetContainsAnnotations(self.contains_annotations_);
        let space = builder.ToConstraintSpace();
        BlockNode::new(self as *const LayoutView as *mut LayoutBox)
            .ComputeMinMaxSizes(
                mode,
                SizeType::kIntrinsic,
                &space,
                MinMaxSizesFloatInput::default(),
            )
            .sizes
            .min_size
    }

    // cpp: layoutng/internal/layout_view_layout.cc:113-126
    #[inline(never)]
    pub fn LayoutRoot(&self) {
        self.CheckIsNotDestroyed();
        let style = self.StyleRef();
        let writing_mode = style.GetWritingMode();
        let size = ToLogicalSize(self.InitialContainingBlockSize(), writing_mode);
        let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
            writing_mode,
            style.GetWritingDirection(),
            true,
            true,
            false,
        );
        builder.SetAvailableSize(size);
        builder.SetIsFixedInlineSize(true);
        builder.SetIsFixedBlockSize(true);
        builder.SetContainsAnnotations(self.contains_annotations_);
        let space = builder.ToConstraintSpace();
        BlockNode::new(self as *const LayoutView as *mut LayoutBox).Layout(
            &space,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        );
    }
}
