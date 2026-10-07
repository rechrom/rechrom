// C++: layoutng_inline/physical_line_box_fragment_builder.cc.
#![allow(non_snake_case)]

use foundation::{LayoutUnit, MakeGarbageCollected, TextDirection};
use layoutng_fragment_tree::physical_fragment::{
    FragmentType, PhysicalFragment, PhysicalFragmentFlags,
};
use layoutng_fragment_tree::physical_line_box_fragment::PhysicalLineBoxFragment;

use crate::line_box_fragment_builder::LineBoxFragmentBuilder;

// cpp: layoutng_inline/physical_line_box_fragment_builder.cc:10-15
#[unsafe(no_mangle)]
pub extern "Rust" fn PhysicalLineBoxFragmentCreateFromInline(
    builder: &mut LineBoxFragmentBuilder,
) -> *const PhysicalLineBoxFragment {
    debug_assert_eq!(builder.base_.children_.len(), 0);
    MakeGarbageCollected(PhysicalLineBoxFragmentFromBuilder(builder))
}

// cpp: layoutng_inline/physical_line_box_fragment_builder.cc:17-30
fn PhysicalLineBoxFragmentFromBuilder(
    builder: &mut LineBoxFragmentBuilder,
) -> PhysicalLineBoxFragment {
    let writing_mode = builder.base_.GetWritingMode();
    let line_box_type = builder.line_box_type_;
    let base = PhysicalFragment::from_builder(
        &mut builder.base_,
        writing_mode,
        FragmentType::kFragmentLineBox,
        line_box_type as u32,
    );
    let result = PhysicalLineBoxFragment {
        base_: base,
        metrics_: builder.metrics_,
    };
    debug_assert!(!result.metrics_.IsEmpty() || result.IsEmptyLineBox());
    result.base_.flags_.set(
        PhysicalFragmentFlags::BASE_DIRECTION,
        builder.base_direction_ == TextDirection::kRtl,
    );
    result.base_.flags_.set(
        PhysicalFragmentFlags::HAS_HANGING,
        builder.hang_inline_size_ != LayoutUnit::default(),
    );
    result.base_.flags_.set(
        PhysicalFragmentFlags::HAS_PROPAGATED_DESCENDANTS,
        result.HasFloatingDescendantsForPaint()
            || result.HasOutOfFlowPositionedDescendants()
            || builder.base_.unpositioned_list_marker_.is_present(),
    );
    result
}
