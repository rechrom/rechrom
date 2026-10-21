#![allow(non_snake_case)]

use foundation::{
    DynamicTo, EBoxSizing, IsParallelWritingMode, LayoutUnit, RuntimeEnabledFeatures,
};
use layoutng_fragment_tree::box_fragment_builder::BoxFragmentBuilder;
use layoutng_geometry::geometry::bfc_offset::BfcOffset;
use layoutng_style::style::computed_style::ComputedStyle;

use super::block_node::BlockNode;
use super::constraint_space_builder::ConstraintSpaceBuilder;
use super::form_node_metadata::HTMLElement;
use super::layout_input_node::LayoutInputNode;
use super::layout_node_metadata::ElementType;

// cpp: layoutng/internal/space_utils.h:19-21
// cpp: layoutng/internal/space_utils.cc:19-27
pub fn AdjustToClearance(clearance_offset: LayoutUnit, offset: &mut BfcOffset) -> bool {
    if clearance_offset > offset.block_offset {
        offset.block_offset = clearance_offset;
        return true;
    }
    false
}

// cpp: layoutng/internal/space_utils.h:32-34
// cpp: layoutng/internal/space_utils.cc:29-77
pub fn SetOrthogonalFallbackInlineSize(
    parent_style: &ComputedStyle,
    child: LayoutInputNode,
    builder: &mut ConstraintSpaceBuilder,
) {
    debug_assert!(!IsParallelWritingMode(
        parent_style.GetWritingMode(),
        child.Style().GetWritingMode()
    ));

    let containing_block_size = child.InitialContainingBlockSize();
    let mut fallback_size = if parent_style.IsHorizontalWritingMode() {
        containing_block_size.height
    } else {
        containing_block_size.width
    };

    let mut size = LayoutUnit::Max();
    if parent_style.LogicalHeight().IsFixed() {
        size = LayoutUnit::from_f32(parent_style.LogicalHeight().Pixels());
    }
    if parent_style.LogicalMaxHeight().IsFixed() {
        let max_height = LayoutUnit::from_f32(parent_style.LogicalMaxHeight().Pixels());
        if max_height < size {
            size = max_height;
        }
    }
    if parent_style.LogicalMinHeight().IsFixed() {
        let min_height = LayoutUnit::from_f32(parent_style.LogicalMinHeight().Pixels());
        if min_height > size {
            size = min_height;
        }
    }

    if parent_style.BoxSizing() == EBoxSizing::kBorderBox {
        if !parent_style.PaddingBlockStart().IsFixed() || !parent_style.PaddingBlockEnd().IsFixed()
        {
            builder.SetOrthogonalFallbackInlineSize(fallback_size);
            return;
        }

        let border_padding = LayoutUnit::from_f32(
            parent_style.BorderBlockStartWidth() as f32
                + parent_style.BorderBlockEndWidth() as f32
                + parent_style.PaddingBlockStart().Pixels()
                + parent_style.PaddingBlockEnd().Pixels(),
        );
        size -= border_padding;
        size = size.ClampNegativeToZero();
    }

    if size < fallback_size {
        fallback_size = size;
    }
    builder.SetOrthogonalFallbackInlineSize(fallback_size);
}

// cpp: layoutng/internal/space_utils.h:36-45
pub fn SetOrthogonalFallbackInlineSizeIfNeeded(
    parent_style: &ComputedStyle,
    child: LayoutInputNode,
    builder: &mut ConstraintSpaceBuilder,
) {
    if IsParallelWritingMode(
        parent_style.GetWritingMode(),
        child.Style().GetWritingMode(),
    ) {
        return;
    }
    SetOrthogonalFallbackInlineSize(parent_style, child, builder);
}

// cpp: layoutng/internal/space_utils.h:50-50
// cpp: layoutng/internal/space_utils.cc:79-102
pub fn ShouldBlockContainerChildStretchAutoInlineSize(child: &BlockNode) -> bool {
    if child.IsReplaced() || child.IsTable() {
        return false;
    }

    let element = DynamicTo::<HTMLElement>(child.GetDOMNode());
    if !element.is_null() {
        // The base Element stores the concrete virtual identity, whereas
        // HTMLElement::GetElementType identifies only the base type.
        let element = unsafe { &*element };
        if matches!(
            element.element.GetElementType(),
            ElementType::kHTMLButtonElement
                | ElementType::kHTMLInputElement
                | ElementType::kHTMLSelectElement
                | ElementType::kHTMLTextAreaElement
        ) {
            if !RuntimeEnabledFeatures::BaseAppearanceInlineSizingEnabled() {
                return false;
            }
            if !element.element.InputSupportsBaseAppearance() {
                return false;
            }
        }
    }
    true
}

// cpp: layoutng/internal/space_utils.h:55-57
// cpp: layoutng/internal/space_utils.cc:104-121
pub fn SetTextBoxTrimOnChildSpaceBuilderWithKnownContent(
    fragment_builder: &BoxFragmentBuilder,
    known_to_have_successive_content: bool,
    space_builder: &mut ConstraintSpaceBuilder,
) {
    space_builder.SetShouldTextBoxTrimNodeStart(fragment_builder.ShouldTextBoxTrimNodeStart());
    space_builder.SetShouldTextBoxTrimFragmentainerStart(
        fragment_builder.ShouldTextBoxTrimFragmentainerStart(),
    );
    space_builder
        .SetShouldTextBoxTrimFragmentainerEnd(fragment_builder.ShouldTextBoxTrimFragmentainerEnd());
    if fragment_builder.ShouldTextBoxTrimEnd() {
        space_builder.SetShouldTextBoxTrimNodeEnd(
            fragment_builder.ShouldTextBoxTrimNodeEnd() && !known_to_have_successive_content,
        );
    }
}

// cpp: layoutng/internal/space_utils.h:59-63
pub fn SetTextBoxTrimOnChildSpaceBuilder(
    fragment_builder: &BoxFragmentBuilder,
    space_builder: &mut ConstraintSpaceBuilder,
) {
    SetTextBoxTrimOnChildSpaceBuilderWithKnownContent(fragment_builder, false, space_builder);
}
