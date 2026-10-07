// Bind the source Trace methods to the layout heap without moving their
// translated bodies out of their owning files.
// cpp: foundation/blink_base/heap/trace_traits.h:20-28
use foundation::{Traceable, Visitor};

use crate::internal::anchor_map::{AnchorMap, PhysicalAnchorReference};
use crate::internal::anchor_position_scroll_data::{AdjustmentData, AnchorPositionScrollData};
use crate::internal::block_node::BlockNode;
use crate::internal::column_spanner_path::ColumnSpannerPath;
use crate::internal::constraint_space::{ConstraintSpace, RareData};
use crate::internal::css::counters_attachment_context::CounterEntry;
use crate::internal::css::out_of_flow_data::ScrollOffsetPair;
use crate::internal::css::out_of_flow_data::{OutOfFlowData, RememberedScrollOffsets};
use crate::internal::css::successful_position_fallback::SuccessfulPositionFallback;
use crate::internal::custom_scrollbar::CustomScrollbar;
use crate::internal::devtools_flex_info::DevtoolsFlexInfo;
use crate::internal::document_marker::DocumentMarker;
use crate::internal::early_break::EarlyBreak;
use crate::internal::exclusions::exclusion_area::{ExclusionArea, ExclusionShapeData};
use crate::internal::exclusions::exclusion_space::{
    ClosedArea, DerivedGeometry, ExclusionSpace, ExclusionSpaceInternal, Shelf,
};
use crate::internal::exclusions::layout_opportunity::LayoutOpportunity;
use crate::internal::exclusions::shape_exclusions::ShapeExclusions;
use crate::internal::form_node_metadata::{
    HTMLAreaElement, HTMLButtonElement, HTMLDivElement, HTMLElement, HTMLFieldSetElement,
    HTMLFormControlElement, HTMLFormControlElementWithState, HTMLImageElement, HTMLInputElement,
    HTMLLegendElement, HTMLMarqueeElement, HTMLOutputElement, HTMLSelectElement,
    HTMLTextAreaElement, SliderThumbElement, SpinButtonElement, TextControlElement,
    TextControlInnerEditorElement,
};
use crate::internal::fragmentation_utils::FlexColumnBreakInfo;
use crate::internal::gap::gap_geometry::GapGeometry;
use crate::internal::grid_item::{GridItemData, GridItemDataVirtualItemContributions, GridItems};
use crate::internal::grid_lanes_item_group::{GridLanesItemGroup, VirtualItems};
use crate::internal::grid_layout_data::{
    GridLayoutData, GridLayoutSubtree, GridLayoutTree, GridTreeNode,
};
use crate::internal::grid_track_collection::{GridLayoutTrackCollection, GridTrackBaselines};
use crate::internal::hyphen_result::HyphenResult;
use crate::internal::inline_item::InlineItem;
use crate::internal::inline_item_result::{InlineItemResult, OptionalPositionedFloat};
use crate::internal::inline_item_segment::InlineItemSegment;
use crate::internal::inline_item_segment::InlineItemSegments;
use crate::internal::inline_item_span::InlineItemSpan;
use crate::internal::inline_node::FloatingObject;
use crate::internal::inline_node_data::InlineNodeData;
use crate::internal::layout_block::LayoutBlock;
use crate::internal::layout_block_flow::LayoutBlockFlow;
use crate::internal::layout_box::LayoutBox;
use crate::internal::layout_box_model_object::LayoutBoxModelObject;
use crate::internal::layout_custom_scrollbar_part::LayoutCustomScrollbarPart;
use crate::internal::layout_inline::LayoutInline;
use crate::internal::layout_input_node::LayoutInputNode;
use crate::internal::layout_node_data::LayoutBoxRareData;
use crate::internal::layout_node_metadata::{ContainerNode, Element, Node, Text};
use crate::internal::layout_object::LayoutObject;
use crate::internal::layout_object_child_list::LayoutObjectChildList;
use crate::internal::layout_replaced::LayoutInputReplaced;
use crate::internal::layout_scrollable_area::PaintLayerScrollableArea;
use crate::internal::layout_text::LayoutText;
use crate::internal::layout_text_combine::LayoutTextCombine;
use crate::internal::layout_view::LayoutView;
use crate::internal::line_clamp_data::LineClampAncestorChain;
use crate::internal::mathml_paint_info::MathMLPaintInfo;
use crate::internal::measure_cache::MeasureCache;
use crate::internal::min_max_sizes_cache::MinMaxSizesCache;
use crate::internal::naming_scope::NamingScope;
use crate::internal::node_rare_data_field::NodeRareDataField;
use crate::internal::non_overflowing_scroll_range::NonOverflowingScrollRange;
use crate::internal::oof_positioned_node::{
    FragmentedOofData, MulticolWithPendingOofs, OofContainingBlock, OofInlineContainer,
    OofPositionedNode, PhysicalOofNodeForFragmentation,
};
use crate::internal::overflow_model::BoxOverflowModel;
use crate::internal::positioned_float::PositionedFloat;
use crate::internal::shapes::shape_outside_info::ShapeOutsideInfo;
use crate::internal::snap_area::SnapArea;
use crate::internal::split_axis_item::SplitAxisItem;
use crate::internal::sticky_position_scrolling_constraints::PerAxisData;
use crate::internal::svg_inline_node_data::{SvgInlineNodeData, SvgTextContentRange};
use crate::internal::table_borders::{Edge, TableBorders};
use crate::internal::table_fragment_data::TableColumnGeometry;
use crate::internal::table_layout_algorithm_types::TableGroupedChildren;
use crate::internal::text_fit_scale::TextFitScale;
use crate::internal::text_overflow_post_layout_snapshot::TextOverflowPostLayoutSnapshot;
use crate::internal::unpositioned_list_marker::UnpositionedListMarker;
use crate::internal::used_font::UsedFont;
use crate::internal::variable_length_transform_result::VariableLengthTransformResult;
use foundation::PhysicalOffset;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::static_position::{LogicalStaticPosition, PhysicalStaticPosition};
use std::ops::AddAssign;

macro_rules! trace_source_method {
    ($($type:ty),+ $(,)?) => {
        $(impl Traceable for $type {
            fn Trace(&self, visitor: &mut Visitor<'_>) {
                <$type>::Trace(self, visitor);
            }
        })+
    };
}

trace_source_method!(
    LayoutObject,
    LayoutBoxModelObject,
    LayoutBox,
    LayoutBlock,
    LayoutBlockFlow,
    LayoutView,
    ConstraintSpace,
    RareData,
    EarlyBreak,
    ExclusionArea,
    ExclusionShapeData,
    ClosedArea,
    DerivedGeometry,
    ExclusionSpace,
    ExclusionSpaceInternal,
    Shelf,
    LayoutOpportunity,
    ShapeExclusions,
    LayoutBoxRareData,
    LayoutObjectChildList,
    BoxOverflowModel,
    AnchorMap,
    PhysicalAnchorReference,
    AdjustmentData,
    AnchorPositionScrollData,
    ColumnSpannerPath,
    CounterEntry,
    OutOfFlowData,
    RememberedScrollOffsets,
    SuccessfulPositionFallback,
    CustomScrollbar,
    DevtoolsFlexInfo,
    DocumentMarker,
    FlexColumnBreakInfo,
    GapGeometry,
    GridItemData,
    GridItemDataVirtualItemContributions,
    GridItems,
    GridLanesItemGroup,
    VirtualItems,
    GridLayoutData,
    GridLayoutSubtree,
    GridLayoutTree,
    GridTreeNode,
    GridLayoutTrackCollection,
    GridTrackBaselines,
    HyphenResult,
    InlineItemResult,
    OptionalPositionedFloat,
    InlineItemSegments,
    InlineItemSpan,
    FloatingObject,
    LayoutCustomScrollbarPart,
    LayoutInline,
    ContainerNode,
    Element,
    Node,
    PaintLayerScrollableArea,
    LayoutTextCombine,
    LineClampAncestorChain,
    MathMLPaintInfo,
    MeasureCache,
    MinMaxSizesCache,
    NamingScope,
    NodeRareDataField,
    NonOverflowingScrollRange,
    PositionedFloat,
    ShapeOutsideInfo,
    SnapArea,
    PerAxisData,
    Edge,
    TableBorders,
    TableColumnGeometry,
    TableGroupedChildren,
    TextFitScale,
    TextOverflowPostLayoutSnapshot,
    UnpositionedListMarker,
    UsedFont,
    LayoutInputNode,
    FragmentedOofData,
);

// cpp: layoutng/internal/layout_replaced.h:152-160
// LayoutInputReplaced adds no managed fields; trace the inherited LayoutBox.
impl Traceable for LayoutInputReplaced {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        let box_: &LayoutBox = self;
        box_.Trace(visitor);
    }
}

impl<T: Copy> Traceable for OofContainingBlock<T> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        OofContainingBlock::Trace(self, visitor);
    }
}

impl<T: Copy + Default + AddAssign> Traceable for OofInlineContainer<T> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        OofInlineContainer::Trace(self, visitor);
    }
}

impl<T: Copy + Default + AddAssign> Traceable for MulticolWithPendingOofs<T> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        MulticolWithPendingOofs::Trace(self, visitor);
    }
}

impl<T> Traceable for SplitAxisItem<T> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        SplitAxisItem::Trace(self, visitor);
    }
}

impl Traceable for OofPositionedNode<LogicalOffset, LogicalStaticPosition> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        OofPositionedNode::<LogicalOffset, LogicalStaticPosition>::Trace(self, visitor);
    }
}

impl Traceable for OofPositionedNode<PhysicalOffset, PhysicalStaticPosition> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        OofPositionedNode::<PhysicalOffset, PhysicalStaticPosition>::Trace(self, visitor);
    }
}

impl Traceable for PhysicalOofNodeForFragmentation {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.TraceAfterDispatch(visitor);
    }
}

// C++ HTML nodes inherit Element's managed fields at offset zero.
// cpp: layoutng/internal/form_node_metadata.h:15-51
impl Traceable for HTMLElement {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        Element::Trace(&self.element, visitor);
    }
}
impl Traceable for HTMLAreaElement {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        Element::Trace(&self.html.element, visitor);
    }
}
impl Traceable for HTMLImageElement {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        Element::Trace(&self.html.element, visitor);
    }
}

// The form subclasses embed the traced Element at offset zero. Their own
// managed Member fields are visited in the source-derived Trace order.
// cpp: layoutng/internal/form_node_metadata.h:53-163
// cpp: layoutng_forms/form_node_metadata.cc:72-75,83-86
macro_rules! trace_form_base {
    ($($type:ty => $base:ident),+ $(,)?) => {
        $(impl Traceable for $type {
            fn Trace(&self, visitor: &mut Visitor<'_>) {
                self.$base.Trace(visitor);
            }
        })+
    };
}
trace_form_base!(
    HTMLFormControlElement => html,
    HTMLFormControlElementWithState => control,
    HTMLSelectElement => state,
    HTMLButtonElement => control,
    HTMLFieldSetElement => control,
    HTMLOutputElement => control,
    HTMLLegendElement => html,
    HTMLMarqueeElement => html,
    HTMLDivElement => html,
    TextControlInnerEditorElement => div,
    SpinButtonElement => div,
    SliderThumbElement => div,
);
impl Traceable for TextControlElement {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(self.InnerEditorMember());
        self.state.Trace(visitor);
    }
}
impl Traceable for HTMLInputElement {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(self.UploadButtonMember());
        visitor.Trace(self.SpinButtonMember());
        self.text_control.Trace(visitor);
    }
}
impl Traceable for HTMLTextAreaElement {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.text_control.Trace(visitor);
    }
}

// These value records contain no managed pointer fields in their source.
// cpp: layoutng/internal/inline_item_segment.h:37-77
impl Traceable for InlineItemSegment {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}
// cpp: layoutng/internal/css/out_of_flow_data.h:30-40
impl Traceable for ScrollOffsetPair {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}
// cpp: layoutng/internal/variable_length_transform_result.h:26-29
impl Traceable for VariableLengthTransformResult {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}

// Text has no Trace override in C++; its Node base owns the GC edges.
// cpp: layoutng/internal/layout_node_metadata.h:255-264
impl Traceable for Text {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        Node::Trace(&self.node, visitor);
    }
}

// BlockNode is a zero-extra-field LayoutInputNode subtype. C++ traces the
// inherited box_ edge through LayoutInputNode::Trace.
// cpp: layoutng/internal/layout_input_node_data.cc:210-210
impl Traceable for BlockNode {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        LayoutInputNode::Trace(&self.base, visitor);
    }
}

// The InlineItemsData tag dispatches to this source-defined derived trace.
// cpp: layoutng_fragment_tree/inline_items_data.cc:29-33
impl Traceable for InlineNodeData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.TraceAfterDispatch(visitor);
    }
}
