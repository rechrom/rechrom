#![allow(non_snake_case)]

use font_engine::FontBaseline;
use foundation::{
    LayoutUnit, RuntimeEnabledFeatures, TextDirection, To, UnsupportedLayout, Visitor,
};
use layoutng::internal::block_node::BlockNode;
use layoutng::internal::constraint_space::{BaselineAlgorithmType, ConstraintSpace};
use layoutng::internal::layout_pass_scope::LayoutPassScope;
use layoutng::internal::unpositioned_list_marker::UnpositionedListMarker;
use layoutng_fragment_tree::box_fragment_builder::BoxFragmentBuilder;
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_fragment_tree::logical_box_fragment::LogicalBoxFragment;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_fragment_tree::physical_fragment::PhysicalFragment;
use layoutng_fragment_tree::physical_line_box_fragment::PhysicalLineBoxFragment;
use layoutng_geometry::geometry::bfc_offset::BfcOffset;
use layoutng_geometry::geometry::box_strut::{BoxStrut, LineBoxStrut};
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_size::ToLogicalSize;
use layoutng_style::style::computed_style::ComputedStyle;

// C++ defines these methods on the layoutng-owned class in this package.
// Rust keeps the owner there and provides the typed bodies through symbols.
// cpp: layoutng_block/unpositioned_list_marker.cc:20-23
#[no_mangle]
pub fn UnpositionedListMarkerConstructFromBlock(
    node: &BlockNode,
) -> *mut layoutng::internal::layout_box::LayoutBox {
    let marker = node.GetLayoutBox();
    debug_assert!(!marker.is_null() && unsafe { &*marker }.IsLayoutOutsideListMarker());
    marker
}

// cpp: layoutng_block/unpositioned_list_marker.cc:25-37
#[no_mangle]
pub fn UnpositionedListMarkerInlineOffsetFromBlock(
    marker: &UnpositionedListMarker,
    marker_inline_size: LayoutUnit,
) -> LayoutUnit {
    let object = marker.MarkerLayoutObject();
    debug_assert!(!object.is_null());
    let algorithms = LayoutPassScope::Algorithms();
    let callback = if algorithms.is_null() {
        None
    } else {
        unsafe { &*algorithms }
            .list_support
            .outside_marker_inline_offset
    };
    let callback = callback.unwrap_or_else(|| {
        std::panic::panic_any(UnsupportedLayout::new(
            "list layout module is not installed",
        ))
    });
    callback(unsafe { &*object }, marker_inline_size)
}

// cpp: layoutng_block/unpositioned_list_marker.cc:39-53
#[no_mangle]
pub fn UnpositionedListMarkerLayoutFromBlock(
    marker: &UnpositionedListMarker,
    parent_space: &ConstraintSpace,
    parent_style: &ComputedStyle,
    _baseline: FontBaseline,
) -> *const LayoutResult {
    let object = marker.MarkerLayoutObject();
    debug_assert!(!object.is_null());
    let marker_node = BlockNode::new(object);
    let result = marker_node.LayoutAtomicInline(
        parent_space,
        parent_style,
        parent_space.UseFirstLineStyle(),
        BaselineAlgorithmType::kDefault,
    );
    debug_assert!(!result.is_null());
    result
}

// cpp: layoutng_block/unpositioned_list_marker.cc:55-79
#[no_mangle]
pub fn UnpositionedListMarkerContentAlignmentBaselineFromBlock(
    _marker: &UnpositionedListMarker,
    space: &ConstraintSpace,
    _baseline: FontBaseline,
    content: &PhysicalFragment,
) -> Option<LayoutUnit> {
    if content.IsLineBox() {
        let line_box = unsafe { &*To::<PhysicalLineBoxFragment>(content as *const _) };
        if line_box.IsEmptyLineBox() && !line_box.GetBreakToken().is_null() {
            return None;
        }
        return Some(line_box.Metrics().ascent);
    }
    let box_fragment = unsafe { &*To::<PhysicalBoxFragment>(content as *const _) };
    LogicalBoxFragment::new(space.GetWritingDirection(), box_fragment).FirstBaseline()
}

// cpp: layoutng_block/unpositioned_list_marker.cc:81-129
#[no_mangle]
pub fn UnpositionedListMarkerAddToBoxFromBlock(
    marker: &UnpositionedListMarker,
    space: &ConstraintSpace,
    baseline: FontBaseline,
    _content: &PhysicalFragment,
    border_scrollbar_padding: &BoxStrut,
    marker_layout_result: &LayoutResult,
    content_baseline: LayoutUnit,
    block_offset: &mut LayoutUnit,
    builder: *mut BoxFragmentBuilder,
) {
    let marker_physical_fragment = unsafe {
        &*To::<PhysicalBoxFragment>(marker_layout_result.GetPhysicalFragment() as *const _)
    };
    let marker_fragment =
        LogicalBoxFragment::new(space.GetWritingDirection(), marker_physical_fragment);
    let mut marker_offset = LogicalOffset::new(
        marker.InlineOffset(marker_fragment.Size().inline_size),
        *block_offset,
    );
    let marker_metrics = marker_fragment.BaselineMetrics(&LineBoxStrut::default(), baseline);
    let baseline_adjust = content_baseline - marker_metrics.ascent;
    if baseline_adjust >= LayoutUnit::default() {
        marker_offset.block_offset += baseline_adjust;
    } else if !builder.is_null()
        && unsafe { &*builder }.ShouldTextBoxTrimNodeStart()
        && RuntimeEnabledFeatures::TextBoxTrimForNestedListEnabled()
    {
        marker_offset.block_offset += baseline_adjust;
    } else {
        *block_offset -= baseline_adjust;
    }
    marker_offset.inline_offset += marker.ComputeIntrudedFloatOffset(
        space,
        builder,
        border_scrollbar_padding,
        marker_offset.block_offset,
    );
    debug_assert!(!builder.is_null());
    let builder = unsafe { &mut *builder };
    let items_builder = builder.ItemsBuilder();
    if !items_builder.is_null() {
        unsafe { &mut *items_builder }.AddListMarker(marker_physical_fragment, marker_offset);
        return;
    }
    builder.AddResult(
        marker_layout_result,
        marker_offset,
        None,
        None,
        std::ptr::null(),
    );
}

// cpp: layoutng_block/unpositioned_list_marker.cc:131-163
#[no_mangle]
pub fn UnpositionedListMarkerAddToBoxWithoutLineBoxesFromBlock(
    marker: &UnpositionedListMarker,
    space: &ConstraintSpace,
    _baseline: FontBaseline,
    marker_layout_result: &LayoutResult,
    builder: *mut BoxFragmentBuilder,
    intrinsic_block_size: &mut LayoutUnit,
) {
    let marker_physical_fragment = unsafe {
        &*To::<PhysicalBoxFragment>(marker_layout_result.GetPhysicalFragment() as *const _)
    };
    let marker_size = ToLogicalSize(marker_physical_fragment.Size(), space.GetWritingMode());
    let offset = LogicalOffset::new(
        marker.InlineOffset(marker_size.inline_size),
        LayoutUnit::default(),
    );
    debug_assert!(!builder.is_null());
    let builder = unsafe { &mut *builder };
    debug_assert!(builder.ItemsBuilder().is_null());
    builder.AddResult(marker_layout_result, offset, None, None, std::ptr::null());
    if builder.BfcBlockOffset().is_some() {
        *intrinsic_block_size = marker_size.block_size.max(*intrinsic_block_size);
        builder.SetIntrinsicBlockSize(*intrinsic_block_size);
        builder.SetFragmentsTotalBlockSize(marker_size.block_size.max(builder.Size().block_size));
    }
}

// cpp: layoutng_block/unpositioned_list_marker.cc:165-202
#[no_mangle]
pub fn UnpositionedListMarkerComputeIntrudedFloatOffsetFromBlock(
    marker: &UnpositionedListMarker,
    space: &ConstraintSpace,
    builder: *const BoxFragmentBuilder,
    border_scrollbar_padding: &BoxStrut,
    marker_block_offset: LayoutUnit,
) -> LayoutUnit {
    debug_assert!(!builder.is_null());
    let builder = unsafe { &*builder };
    let Some(bfc_block_offset) = builder.BfcBlockOffset() else {
        return LayoutUnit::default();
    };
    let origin_offset = BfcOffset::new(
        builder.BfcLineOffset() + border_scrollbar_padding.inline_start,
        *bfc_block_offset + marker_block_offset,
    );
    let available_size = builder.ChildAvailableSize().inline_size;
    let object = marker.MarkerLayoutObject();
    debug_assert!(!object.is_null());
    let direction = unsafe { &*object }.StyleRef().Direction();
    let opportunity = space.GetExclusionSpace().FindLayoutOpportunityDefault(
        &origin_offset,
        available_size,
        direction,
    );
    if direction == TextDirection::kLtr {
        if opportunity.rect.LineStartOffset() > origin_offset.line_offset {
            return opportunity.rect.LineStartOffset() - origin_offset.line_offset;
        }
    } else if opportunity.rect.LineEndOffset() < origin_offset.line_offset + available_size {
        return origin_offset.line_offset + available_size - opportunity.rect.LineEndOffset();
    }
    LayoutUnit::default()
}

// cpp: layoutng_block/unpositioned_list_marker.cc:204-214
#[cfg(debug_assertions)]
#[no_mangle]
pub fn UnpositionedListMarkerCheckMarginFromBlock(marker: &UnpositionedListMarker) {
    let object = marker.MarkerLayoutObject();
    debug_assert!(!object.is_null());
    debug_assert!(unsafe { &*object }.StyleRef().MarginBlockStart().IsZero());
}

// cpp: layoutng_block/unpositioned_list_marker.cc:216-218
#[no_mangle]
pub fn UnpositionedListMarkerTraceFromBlock(
    marker: &UnpositionedListMarker,
    visitor: &mut Visitor,
) {
    marker.TraceMarkerMember(visitor);
}
