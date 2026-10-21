use foundation::{DisplayAdjustmentContext, EBoxOrient, EContinue, EDisplay, EFloat, EPosition};

use super::computed_style::{ComputedStyle, ComputedStyleBuilder};
use super::computed_style_constants::PseudoId;

// cpp: layoutng_style/style/display_adjustment.cc:37-90
fn EquivalentBlockDisplay(display: EDisplay) -> EDisplay {
    match display {
        EDisplay::kFlowRootListItem
        | EDisplay::kBlock
        | EDisplay::kTable
        | EDisplay::kWebkitBox
        | EDisplay::kFlex
        | EDisplay::kGrid
        | EDisplay::kBlockMath
        | EDisplay::kBlockRuby
        | EDisplay::kListItem
        | EDisplay::kFlowRoot
        | EDisplay::kLayoutCustom
        | EDisplay::kGridLanes => display,
        EDisplay::kInlineTable => EDisplay::kTable,
        EDisplay::kWebkitInlineBox => EDisplay::kWebkitBox,
        EDisplay::kInlineFlex => EDisplay::kFlex,
        EDisplay::kInlineGrid => EDisplay::kGrid,
        EDisplay::kMath => EDisplay::kBlockMath,
        EDisplay::kRuby => EDisplay::kBlockRuby,
        EDisplay::kInlineLayoutCustom => EDisplay::kLayoutCustom,
        EDisplay::kInlineListItem => EDisplay::kListItem,
        EDisplay::kInlineFlowRootListItem => EDisplay::kFlowRootListItem,
        EDisplay::kInlineGridLanes => EDisplay::kGridLanes,
        EDisplay::kContents
        | EDisplay::kInline
        | EDisplay::kInlineBlock
        | EDisplay::kTableRowGroup
        | EDisplay::kTableHeaderGroup
        | EDisplay::kTableFooterGroup
        | EDisplay::kTableRow
        | EDisplay::kTableColumnGroup
        | EDisplay::kTableColumn
        | EDisplay::kTableCell
        | EDisplay::kTableCaption
        | EDisplay::kRubyText => EDisplay::kBlock,
        EDisplay::kNone => unreachable!("none is excluded before display adjustment"),
    }
}

// cpp: layoutng_style/style/display_adjustment.cc:92-147
fn EquivalentInlineDisplay(display: EDisplay) -> EDisplay {
    match display {
        EDisplay::kFlowRootListItem => EDisplay::kInlineFlowRootListItem,
        EDisplay::kBlock | EDisplay::kFlowRoot => EDisplay::kInlineBlock,
        EDisplay::kTable => EDisplay::kInlineTable,
        EDisplay::kWebkitBox => EDisplay::kWebkitInlineBox,
        EDisplay::kFlex => EDisplay::kInlineFlex,
        EDisplay::kGrid => EDisplay::kInlineGrid,
        EDisplay::kGridLanes => EDisplay::kInlineGridLanes,
        EDisplay::kBlockMath => EDisplay::kMath,
        EDisplay::kBlockRuby => EDisplay::kRuby,
        EDisplay::kListItem => EDisplay::kInlineListItem,
        EDisplay::kLayoutCustom => EDisplay::kInlineLayoutCustom,
        EDisplay::kInlineFlex
        | EDisplay::kInlineFlowRootListItem
        | EDisplay::kInlineGrid
        | EDisplay::kInlineLayoutCustom
        | EDisplay::kInlineListItem
        | EDisplay::kInlineGridLanes
        | EDisplay::kInlineTable
        | EDisplay::kMath
        | EDisplay::kRuby
        | EDisplay::kWebkitInlineBox
        | EDisplay::kContents
        | EDisplay::kInline
        | EDisplay::kInlineBlock
        | EDisplay::kTableRowGroup
        | EDisplay::kTableHeaderGroup
        | EDisplay::kTableFooterGroup
        | EDisplay::kTableRow
        | EDisplay::kTableColumnGroup
        | EDisplay::kTableColumn
        | EDisplay::kTableCell
        | EDisplay::kTableCaption
        | EDisplay::kRubyText => display,
        EDisplay::kNone => unreachable!("none is excluded before display adjustment"),
    }
}

// cpp: layoutng_style/style/display_adjustment.cc:149-242
#[allow(non_snake_case)]
fn AdjustDisplay(
    builder: &mut ComputedStyleBuilder,
    layout_parent_style: &ComputedStyle,
    context: &DisplayAdjustmentContext,
) {
    let is_immediate_canvas_child = context.is_immediate_canvas_child;

    if (layout_parent_style.BlockifiesChildren() && !context.is_input_file_shadow_child)
        || is_immediate_canvas_child
    {
        builder.SetIsInBlockifyingDisplay();
        if builder.Display() != EDisplay::kContents {
            let display = EquivalentBlockDisplay(builder.Display());
            builder.SetDisplay(display);
            if !builder.HasOutOfFlowPosition() {
                builder.SetIsFlexOrGridOrCustomItem();
            }
        }
        if layout_parent_style.IsDisplayFlex()
            || layout_parent_style.IsDisplayWebkitBox()
            || layout_parent_style.IsDisplayGrid()
            || layout_parent_style.IsDisplayGridLanes()
            || layout_parent_style.IsDisplayMath()
            || is_immediate_canvas_child
        {
            builder.SetIsInsideDisplayIgnoringFloatingChildren();
        }

        if is_immediate_canvas_child {
            builder.SetPosition(EPosition::kStatic);
        }
    }

    if layout_parent_style.InlinifiesChildren()
        && !builder.HasOutOfFlowPosition()
        && context.should_be_inlinified
        && !is_immediate_canvas_child
    {
        if builder.IsFloating() {
            builder.SetFloating(EFloat::kNone);
        }
        debug_assert!(!builder.IsFloating());
        builder.SetIsInInlinifyingDisplay();
        let display = EquivalentInlineDisplay(builder.Display());
        builder.SetDisplay(display);
    }

    if builder.StyleType() == PseudoId::kPseudoIdScrollMarkerGroup {
        let display = EquivalentBlockDisplay(builder.Display());
        builder.SetDisplay(display);
    }

    if builder.Display() == EDisplay::kBlock {
        return;
    }

    if builder.Display() == EDisplay::kInline
        && builder.StyleType() == PseudoId::kPseudoIdNone
        && builder.GetWritingMode() != layout_parent_style.GetWritingMode()
    {
        builder.SetDisplay(EDisplay::kInlineBlock);
    }

    if builder.Display() == EDisplay::kTableColumn
        || builder.Display() == EDisplay::kTableColumnGroup
        || builder.Display() == EDisplay::kTableFooterGroup
        || builder.Display() == EDisplay::kTableHeaderGroup
        || builder.Display() == EDisplay::kTableRow
        || builder.Display() == EDisplay::kTableRowGroup
    {
        builder.SetWritingMode(layout_parent_style.GetWritingMode());
        builder.SetTextOrientation(layout_parent_style.GetTextOrientation());
    }

    if context.is_at_media_shadow_boundary {
        let display = EquivalentBlockDisplay(builder.Display());
        builder.SetDisplay(display);
    }

    if builder.BoxOrient() == EBoxOrient::kVertical
        && (builder.WebkitLineClamp() != 0
            || builder.Continue() == EContinue::kCollapse
            || builder.Continue() == EContinue::kWebkitLegacy)
    {
        if builder.Display() == EDisplay::kWebkitBox {
            builder.SetDisplay(EDisplay::kFlowRoot);
            builder.SetIsSpecifiedDisplayWebkitBox();
        } else if builder.Display() == EDisplay::kWebkitInlineBox {
            builder.SetDisplay(EDisplay::kInlineBlock);
            builder.SetIsSpecifiedDisplayWebkitBox();
        }
    }
}

// cpp: layoutng_style/style/display_adjustment.cc:246-261
#[allow(non_snake_case)]
pub fn AdjustComputedDisplayForLayout(
    builder: &mut ComputedStyleBuilder,
    layout_parent_style: &ComputedStyle,
    context: &DisplayAdjustmentContext,
) {
    if builder.Display() == EDisplay::kNone {
        return;
    }
    if context.is_root
        || (builder.Display() != EDisplay::kContents
            && (builder.HasOutOfFlowPosition() || builder.IsFloating()))
    {
        let display = EquivalentBlockDisplay(builder.Display());
        builder.SetDisplay(display);
    }
    AdjustDisplay(builder, layout_parent_style, context);
}
