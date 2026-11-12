use std::cell::UnsafeCell;
use std::ops::Deref;
use std::sync::atomic::{AtomicU32, Ordering};

use foundation::graphics_types;

use foundation::{
    gfx, AtomicString, DynamicTo, EVisibility, GCedHeapVector, HeapVector, LayoutUnit, Length,
    MakeGarbageCollected, MakeGarbageCollectedWithAdditionalBytes, Member, PhysicalOffset,
    PhysicalRect, RuntimeEnabledFeatures, To, UnsupportedLayout, ValueForLength, Visitor,
    WritingMode,
};
use graphics_types::graphics::overlay_scrollbar_clip_behavior::OverlayScrollbarClipBehavior;
use layoutng::internal::block_node::BlockNode;
use layoutng::internal::disable_layout_side_effects_scope::DisableLayoutSideEffectsScope;
use layoutng::internal::frame_set_layout_data::FrameSetLayoutData;
use layoutng::internal::gap::gap_geometry::{GapDecorationInkOutsets, GapGeometry};
use layoutng::internal::ink_overflow::{InkOverflow, InkOverflowType};
use layoutng::internal::layout_block_flow::LayoutBlockFlow;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_box_core_services::ApplyVisibleOverflowToClipRect;
use layoutng::internal::layout_node_metadata::ContentLockBlocksChildLayout;
use layoutng::internal::layout_node_metadata::{ContainerNode, Element, Node};
use layoutng::internal::layout_object::{HitTestLocation, HitTestResult};
use layoutng::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng::internal::layout_pass_scope::LayoutPassScope;
use layoutng::internal::layout_view::LayoutView;
use layoutng::internal::mathml_paint_info::MathMLPaintInfo;
use layoutng::internal::outline_geometry::OutlineOutsetExtent;
use layoutng::internal::outline_info::LayoutOutlineInfo;
use layoutng::internal::outline_rect_collector::{OutlineRectCollector, UnionOutlineRectCollector};
use layoutng::internal::pagination_utils::{GetPageArea, GetPageBorderBox};
use layoutng::internal::scrollable_overflow_calculator::ScrollableOverflowCalculator;
use layoutng::internal::table_borders::TableBorders;
use layoutng::internal::table_fragment_data::{
    CollapsedTableBordersGeometry, GCedTableColumnGeometries,
};
use layoutng_geometry::geometry::box_sides::PhysicalBoxSides;
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::logical_size::{ToLogicalSize, ToPhysicalSize};
use layoutng_geometry::geometry::overflow_clip_axes::{
    kNoOverflowClip, kOverflowClipBothAxis, kOverflowClipX, kOverflowClipY, OverflowClipAxes,
};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;
use layoutng_style::style::gap_data_list::GapDataList;
use layoutng_style::style::grid_enums::GridTrackSizingDirection;
use layoutng_style::style::outline_type::{
    OutlineType, ShouldIncludeBlockInkOverflow, ShouldIncludeBlockInkOverflowForAnchorOnly,
};
use layoutng_style::style::style_overflow_clip_margin::ReferenceBox;

use crate::block_break_token::BlockBreakToken;
use crate::box_fragment_builder::BoxFragmentBuilder;
use crate::fragment_item::ItemType as FragmentItemType;
use crate::fragment_items::FragmentItems;
use crate::inline_cursor::InlineCursor;
use crate::physical_fragment::{
    BoxType, FragmentType, PhysicalFragment, PhysicalFragmentFlags, PostLayoutChildLinkList,
};
use crate::physical_fragment_link::PhysicalFragmentLink;
use crate::physical_fragment_rare_data::{FieldId, PhysicalFragmentRareData, RareField};

// FragmentItems occupies aligned trailing bytes, outside the fixed-size box
// fragment. The base stays first for the source's inherited layout contract.
// cpp: layoutng_fragment_tree/physical_box_fragment.h:40-75
// cpp: layoutng_fragment_tree/physical_box_fragment.h:627-739
// cpp: layoutng_fragment_tree/physical_box_fragment.cc:52-61
#[repr(C)]
pub struct PhysicalBoxFragment {
    pub(crate) base_: UnsafeCell<PhysicalFragment>,
    bit_field_: AtomicU32,
    first_baseline_: LayoutUnit,
    last_baseline_: LayoutUnit,
    rare_data_: UnsafeCell<Member<PhysicalFragmentRareData>>,
    ink_overflow_: UnsafeCell<InkOverflow>,
    children_: UnsafeCell<HeapVector<PhysicalFragmentLink>>,
}

// cpp: layoutng_fragment_tree/physical_box_fragment.h:739-745
impl foundation::DowncastFrom<PhysicalFragment> for PhysicalBoxFragment {
    fn AllowFrom(fragment: &PhysicalFragment) -> bool {
        fragment.Type() == FragmentType::kFragmentBox
    }
}

const _: () = assert!(std::mem::offset_of!(PhysicalBoxFragment, base_) == 0);

#[cfg(debug_assertions)]
static ALLOW_POST_LAYOUT_COUNT: AtomicU32 = AtomicU32::new(0);

// cpp: layoutng_fragment_tree/physical_box_fragment.h:77-89
// cpp: layoutng_fragment_tree/physical_box_fragment.cc:46-48
// cpp: layoutng_fragment_tree/physical_box_fragment.cc:1783-1790
#[cfg(debug_assertions)]
pub struct AllowPostLayoutScope;

#[cfg(debug_assertions)]
#[allow(non_snake_case)]
impl AllowPostLayoutScope {
    pub fn new() -> Self {
        ALLOW_POST_LAYOUT_COUNT.fetch_add(1, Ordering::Relaxed);
        Self
    }

    pub fn IsAllowed() -> bool {
        ALLOW_POST_LAYOUT_COUNT.load(Ordering::Relaxed) != 0
    }
}

#[cfg(debug_assertions)]
impl Drop for AllowPostLayoutScope {
    fn drop(&mut self) {
        debug_assert!(ALLOW_POST_LAYOUT_COUNT.load(Ordering::Relaxed) != 0);
        ALLOW_POST_LAYOUT_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
}

// The source restricts these mutations to the enclosing fragment's factory.
// cpp: layoutng_fragment_tree/physical_box_fragment.h:485-511
// cpp: layoutng_fragment_tree/physical_box_fragment.cc:859-862
pub struct MutableForStyleRecalc<'a> {
    fragment_: &'a PhysicalBoxFragment,
}

// cpp: layoutng_fragment_tree/physical_box_fragment.cc:887-890
pub struct MutableForContainerLayout<'a> {
    fragment_: &'a PhysicalBoxFragment,
}

// cpp: layoutng_fragment_tree/physical_box_fragment.h:539-567
pub struct MutableForCloning<'a> {
    fragment_: &'a PhysicalBoxFragment,
}

// cpp: layoutng_fragment_tree/physical_box_fragment.h:515-535
pub struct MutableForPainting<'a> {
    fragment_: &'a PhysicalBoxFragment,
}

// cpp: layoutng_fragment_tree/physical_box_fragment.h:123-143
pub struct MutableChildrenForOutOfFlow<'a> {
    fragment_: &'a PhysicalBoxFragment,
}

// The methods of this adapter are defined by the unselected out-of-flow
// package. Keep the source-visible handle and access to its fragment so that
// that package can supply the operations without duplicating this class.
// cpp: layoutng_fragment_tree/physical_box_fragment.h:581-610
pub struct MutableForOofFragmentation<'a> {
    pub fragment_: &'a PhysicalBoxFragment,
}

#[allow(non_snake_case)]
impl MutableForOofFragmentation<'_> {
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:929-943
    pub fn AddChildFragmentainer(
        &self,
        child_fragment: &PhysicalBoxFragment,
        child_offset: LogicalOffset,
    ) {
        debug_assert!(self.fragment_.IsFragmentationContextRoot());
        debug_assert!(child_fragment.IsFragmentainerBox());
        let converter = WritingModeConverter::new(
            self.fragment_.Style().GetWritingDirection(),
            self.fragment_.Size(),
        );
        let link = PhysicalFragmentLink {
            offset: converter.ToPhysicalOffset(child_offset, child_fragment.Size()),
            fragment: Member::from_ptr(
                child_fragment as *const PhysicalBoxFragment as *mut PhysicalFragment,
            ),
        };
        unsafe { &mut *self.fragment_.children_.get() }.push(link);
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:945-981
    pub fn Merge(&self, placeholder_fragmentainer: &PhysicalBoxFragment) {
        debug_assert!(placeholder_fragmentainer.IsFragmentainerBox());
        for new_child in unsafe { &*placeholder_fragmentainer.children_.get() } {
            unsafe { &mut *self.fragment_.children_.get() }.push(PhysicalFragmentLink {
                fragment: new_child.fragment.clone(),
                offset: new_child.offset,
            });
            debug_assert!(new_child.IsOutOfFlowPositioned());
            self.fragment_
                .base_mut()
                .flags_
                .set(PhysicalFragmentFlags::HAS_OUT_OF_FLOW_FRAGMENT_CHILD, true);
        }
        let new_break_token = placeholder_fragmentainer.GetBreakToken();
        if !new_break_token.is_null() {
            let old_break_token = self.fragment_.GetBreakToken();
            if old_break_token.is_null() {
                self.fragment_.base_mut().break_token_ =
                    Member::from_ptr(new_break_token as *mut crate::break_token::BreakToken);
            } else {
                unsafe { &*old_break_token }
                    .GetMutableForOofFragmentation()
                    .Merge(unsafe { &*new_break_token });
            }
        }
        let new_anchor_map = placeholder_fragmentainer.GetAnchorMap();
        if !new_anchor_map.is_null() {
            if self.fragment_.base_mut().oof_data_.Get().is_null() {
                self.fragment_.base_mut().oof_data_ = Member::from_ptr(MakeGarbageCollected(
                    crate::physical_fragment::OofData::default(),
                ));
            }
            let oof_data = self.fragment_.base_mut().oof_data_.Get();
            let anchor_map = unsafe { &mut *oof_data }.EnsureAnchorMap();
            for entry in unsafe { &*new_anchor_map } {
                anchor_map.SetReference(entry.key, entry.value);
            }
        }
        self.UpdateOverflow();
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:983-988
    pub fn UpdateOverflow(&self) {
        let overflow = ScrollableOverflowCalculator::RecalculateScrollableOverflowForFragment(
            self.fragment_,
            true,
        );
        self.fragment_
            .GetMutableForStyleRecalc()
            .SetScrollableOverflow(overflow);
    }
}

#[allow(non_snake_case)]
impl MutableChildrenForOutOfFlow<'_> {
    pub fn Children(&self) -> &mut [PhysicalFragmentLink] {
        unsafe { &mut *self.fragment_.children_.get() }.as_mut_slice()
    }
}

#[allow(non_snake_case)]
impl MutableForStyleRecalc<'_> {
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:864-875
    pub fn SetScrollableOverflow(&self, scrollable_overflow: PhysicalRect) {
        let has_overflow = scrollable_overflow
            != PhysicalRect::new(PhysicalOffset::default(), self.fragment_.Size());
        if has_overflow {
            let field = self.fragment_.EnsureRareField(FieldId::kScrollableOverflow);
            unsafe {
                *field.value_.scrollable_overflow = scrollable_overflow;
            }
        } else if self.fragment_.HasScrollableOverflow() {
            let rare = self.fragment_.rare_data_mut().expect("rare data exists");
            rare.RemoveField(FieldId::kScrollableOverflow);
        }
    }
}

#[allow(non_snake_case)]
impl MutableForContainerLayout<'_> {
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:892-896
    pub fn SetMargins(&self, margins: PhysicalBoxStrut) {
        let field = self.fragment_.EnsureRareField(FieldId::kMargins);
        unsafe {
            *field.value_.margins = margins;
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:898-906
    pub fn SetOffsetFromRootFragmentationContext(&self, offset: PhysicalOffset) {
        let id = FieldId::kOffsetFromRootFragmentationContext;
        if offset.IsZero() && self.fragment_.GetRareField(id).is_none() {
            return;
        }
        let field = self.fragment_.EnsureRareField(id);
        unsafe {
            *field.value_.offset_from_root_fragmentation_context = offset;
        }
    }
}

#[allow(non_snake_case)]
impl MutableForCloning<'_> {
    pub fn ClearIsFirstForNode(&self) {
        self.fragment_
            .set_flag(PhysicalBoxFragment::FIRST_FOR_NODE_BIT, false);
    }

    pub fn ClearPropagatedOOFs(&self) {
        self.fragment_.base_mut().ClearOofData();
    }

    pub fn SetBreakToken(&self, token: *const BlockBreakToken) {
        self.fragment_.base_mut().break_token_ =
            Member::from_ptr(token as *mut crate::break_token::BreakToken);
    }

    pub fn Children(&self) -> &mut [PhysicalFragmentLink] {
        debug_assert!(self.fragment_.ChildrenValid());
        unsafe { &mut *self.fragment_.children_.get() }.as_mut_slice()
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:915-927
    pub fn ReplaceChildren(&self, new_fragment: &PhysicalBoxFragment) {
        debug_assert!(!new_fragment.HasItems() && !self.fragment_.HasItems());
        let children = unsafe { &mut *self.fragment_.children_.get() };
        children.clear();
        for child in new_fragment.Children() {
            children.push(PhysicalFragmentLink {
                fragment: child.fragment.clone(),
                offset: child.offset,
            });
        }
        self.fragment_.base_mut().propagated_data_ = new_fragment.deref().propagated_data_.clone();
    }
}

#[allow(non_snake_case)]
impl MutableForPainting<'_> {
    // Source: core/layout/physical_box_fragment.h:523 and
    // core/layout/physical_box_fragment.cc:1001-1036.
    pub fn RecalcInkOverflow(&self) {
        self.fragment_.RecalcInkOverflow();
    }

    pub fn RecalcInkOverflowWithContents(&self, contents: PhysicalRect) {
        self.fragment_.RecalcInkOverflowWithContents(contents);
    }
}

impl Deref for PhysicalBoxFragment {
    type Target = PhysicalFragment;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.base_.get() }
    }
}

// Source-local helpers used by point positioning and gap-decoration overflow.
// cpp: layoutng_fragment_tree/physical_box_fragment.cc:63-74
fn IsFlexibleBoxWithSingleChildElement(layout_object: &LayoutObject) -> bool {
    if !RuntimeEnabledFeatures::UsePositionForPointInFlexibleBoxWithSingleChildElementEnabled() {
        return false;
    }
    let node = layout_object.GetNode();
    if node.is_null() || !layout_object.IsFlexibleBox() {
        return false;
    }
    let container = DynamicTo::<ContainerNode>(node);
    if container.is_null() {
        return false;
    }
    let mut only_element: *const Element = std::ptr::null();
    let mut child = unsafe { &*container }.firstChild();
    while !child.is_null() {
        let element = DynamicTo::<Element>(child);
        if !element.is_null() {
            if !only_element.is_null() {
                return false;
            }
            only_element = element;
        }
        child = unsafe { &*child }.nextSibling();
    }
    !only_element.is_null()
}

// cpp: layoutng_fragment_tree/physical_box_fragment.cc:76-95
fn ShouldUsePositionForPointInBlockFlowDirection(layout_object: &LayoutObject) -> bool {
    let block_flow = DynamicTo::<LayoutBlockFlow>(layout_object as *const LayoutObject);
    if block_flow.is_null() && !IsFlexibleBoxWithSingleChildElement(layout_object) {
        return false;
    }
    !layout_object.StyleRef().SpecifiesColumns()
}

// cpp: layoutng_fragment_tree/physical_box_fragment.cc:97-101
fn IsHitTestCandidate(fragment: &PhysicalBoxFragment) -> bool {
    fragment.Size().height != LayoutUnit::default()
        && fragment.Style().Visibility() == EVisibility::kVisible
        && !fragment.IsFloatingOrOutOfFlowPositioned()
}

// cpp: layoutng_fragment_tree/physical_box_fragment.cc:119-141
fn MaxGapDecorationsWidth(width_value: &GapDataList<i32>) -> i32 {
    let widths = width_value.GetGapDataList();
    assert!(!widths.is_empty());
    let first = &widths[0];
    let mut maximum = if first.IsRepeaterData() {
        *first
            .GetValueRepeater()
            .RepeatedValues()
            .first()
            .expect("repeater has at least one value")
    } else {
        first.GetValue()
    };
    for width in widths.iter() {
        if !width.IsRepeaterData() {
            maximum = maximum.max(width.GetValue());
        } else {
            for value in width.GetValueRepeater().RepeatedValues().iter() {
                maximum = maximum.max(*value);
            }
        }
    }
    maximum
}

// cpp: layoutng_fragment_tree/physical_box_fragment.cc:143-175
fn MaxGapDecorationInsetOutset(
    cap_inset: &Length,
    junction_inset: &Length,
    cross_gap_width: LayoutUnit,
) -> LayoutUnit {
    let mut maximum = LayoutUnit::default();
    if !cap_inset.IsOverlapJoin() {
        let resolved = ValueForLength(cap_inset, LayoutUnit::default());
        maximum = maximum.max(-resolved);
    }
    if !junction_inset.IsOverlapJoin() {
        let resolved = ValueForLength(junction_inset, cross_gap_width);
        maximum = maximum.max(-resolved);
    }
    maximum
}

// cpp: layoutng_fragment_tree/physical_box_fragment.cc:103-117
fn ApplyOverflowClip(
    axes: OverflowClipAxes,
    no_overflow: &PhysicalRect,
    result: &mut PhysicalRect,
) {
    if axes & kOverflowClipX != 0 {
        result.offset.left = no_overflow.offset.left;
        result.size.width = no_overflow.size.width;
    }
    if axes & kOverflowClipY != 0 {
        result.offset.top = no_overflow.offset.top;
        result.size.height = no_overflow.size.height;
    }
}

#[allow(non_snake_case)]
impl PhysicalBoxFragment {
    // Source: core/layout/physical_box_fragment.cc:1097-1199. Border-shape
    // still requires its concrete painter; outline geometry is collected
    // through the source LayoutObject owner rather than estimated from style.
    fn ComputeSelfInkOverflow(&self) -> PhysicalRect {
        debug_assert_eq!(self.PostLayout(), self as *const Self);
        let style = self.Style();
        let mut ink_overflow = self.LocalRect();
        if self.IsTableRow() {
            let owner = self.GetSelfOrContainerLayoutObject();
            let objects = unsafe { &*owner }.FactorySet();
            let table_virtuals = unsafe { objects.as_ref() }
                .and_then(|objects| objects.table_virtuals)
                .expect("source table virtual dispatch is not installed");
            for child in self.PostLayoutChildren().iter() {
                let child_fragment = DynamicTo::<PhysicalBoxFragment>(child.get());
                debug_assert!(!child_fragment.is_null());
                let child_fragment = unsafe { &*child_fragment };
                if !child_fragment.IsTableCell() {
                    continue;
                }
                let child_layout_object = To::<LayoutBox>(child_fragment.GetLayoutObject());
                debug_assert!(!child_layout_object.is_null());
                if (table_virtuals.computed_row_span)(unsafe { &*child_layout_object }) == 1 {
                    continue;
                }
                let mut child_rect = if child_fragment.CanUseFragmentsForInkOverflow() {
                    child_fragment.InkOverflowRect()
                } else {
                    unsafe { &*child_layout_object }.VisualOverflowRect()
                };
                child_rect.offset += child.offset;
                ink_overflow.Unite(&child_rect);
            }
        }
        if !style.HasVisualOverflowingEffect() {
            return ink_overflow;
        }

        ink_overflow.Expand(&style.BoxDecorationOutsets());
        if style.HasBorderShape() {
            std::panic::panic_any(UnsupportedLayout::new(
                "border-shape self ink overflow requires BorderShapePainter",
            ));
        }
        if style.HasOutline() && self.IsOutlineOwner() {
            let mut collector = UnionOutlineRectCollector::default();
            let mut info = LayoutOutlineInfo::default();
            self.AddSelfOutlineRects(
                PhysicalOffset::default(),
                style.OutlineRectsShouldIncludeBlockInkOverflow(),
                &mut collector,
                Some(&mut info),
            );
            let mut rect = collector.Rect();
            if !style.HasBorderShape() || style.OutlineStyleIsAuto() {
                rect.Inflate(LayoutUnit::from_signed(OutlineOutsetExtent(style, &info)));
            }
            ink_overflow.Unite(&rect);
        }

        let gap_geometry = self.GetGapGeometry();
        if !gap_geometry.is_null() {
            let gap_geometry = unsafe { &*gap_geometry };
            let inline_thickness =
                LayoutUnit::from_signed(MaxGapDecorationsWidth(style.ColumnRuleWidth()));
            let block_thickness =
                LayoutUnit::from_signed(MaxGapDecorationsWidth(style.RowRuleWidth()));
            let col_crossing_gap_size =
                gap_geometry.GetCrossingGapSize(GridTrackSizingDirection::kForColumns);
            let row_crossing_gap_size =
                gap_geometry.GetCrossingGapSize(GridTrackSizingDirection::kForRows);
            let outsets = GapDecorationInkOutsets {
                block_start: MaxGapDecorationInsetOutset(
                    style.ColumnRuleInsetCapStart(),
                    style.ColumnRuleInsetJunctionStart(),
                    col_crossing_gap_size,
                ),
                block_end: MaxGapDecorationInsetOutset(
                    style.ColumnRuleInsetCapEnd(),
                    style.ColumnRuleInsetJunctionEnd(),
                    col_crossing_gap_size,
                ),
                inline_start: MaxGapDecorationInsetOutset(
                    style.RowRuleInsetCapStart(),
                    style.RowRuleInsetJunctionStart(),
                    row_crossing_gap_size,
                ),
                inline_end: MaxGapDecorationInsetOutset(
                    style.RowRuleInsetCapEnd(),
                    style.RowRuleInsetJunctionEnd(),
                    row_crossing_gap_size,
                ),
            };
            let rect = gap_geometry.ComputeInkOverflowForGaps(
                style.GetWritingDirection(),
                &self.Size(),
                inline_thickness,
                block_thickness,
                &outsets,
            );
            ink_overflow.Unite(&rect);
        }
        ink_overflow
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1208-1227
    pub fn AddSelfOutlineRects(
        &self,
        additional_offset: PhysicalOffset,
        outline_type: OutlineType,
        collector: &mut dyn OutlineRectCollector,
        info: Option<&mut LayoutOutlineInfo>,
    ) {
        if let Some(info) = info {
            *info = if self.IsSvgText() {
                LayoutOutlineInfo::GetUnzoomedFromStyle(self.Style())
            } else {
                LayoutOutlineInfo::GetFromStyle(self.Style())
            };
        }

        let layout_object = self.GetLayoutObject();
        let node = if layout_object.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*layout_object }.GetNode()
        };
        let outline_type = if ShouldIncludeBlockInkOverflow(outline_type)
            && !node.is_null()
            && unsafe { &*node }.InputIsHTMLAnchorElement()
        {
            OutlineType::kIncludeBlockInkOverflowForAnchor
        } else {
            outline_type
        };
        self.AddOutlineRectsInternal(additional_offset, outline_type, false, collector);
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1229-1235
    fn AddOutlineRects(
        &self,
        additional_offset: PhysicalOffset,
        outline_type: OutlineType,
        collector: &mut dyn OutlineRectCollector,
    ) {
        self.AddOutlineRectsInternal(additional_offset, outline_type, true, collector);
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1237-1291
    fn AddOutlineRectsInternal(
        &self,
        additional_offset: PhysicalOffset,
        outline_type: OutlineType,
        inline_container_relative: bool,
        collector: &mut dyn OutlineRectCollector,
    ) {
        debug_assert_eq!(self.PostLayout(), self as *const Self);
        if self.IsInlineBox() {
            self.AddOutlineRectsForInlineBox(
                additional_offset,
                outline_type,
                inline_container_relative,
                collector,
            );
            return;
        }
        debug_assert!(self.IsOutlineOwner());

        if !self.IsAnonymousBlockFlow() || self.GetBoxType() == BoxType::kPageBorderBox {
            if self.IsSvgText() {
                let layout_object = self.GetLayoutObject();
                debug_assert!(!layout_object.is_null());
                let algorithms = LayoutPassScope::Algorithms();
                let object_bounding_box = unsafe { algorithms.as_ref() }
                    .and_then(|algorithms| algorithms.svg_support.text_object_bounding_box)
                    .expect("source SVG text ObjectBoundingBox dispatch is not installed");
                collector.AddRect(&PhysicalRect::EnclosingRect(&object_bounding_box(unsafe {
                    &*layout_object
                })));
            } else {
                collector.AddRect(&PhysicalRect::new(additional_offset, self.Size()));
            }
        }

        if ShouldIncludeBlockInkOverflow(outline_type) && !self.HasNonVisibleOverflow() {
            let mut child_collector = collector.ForDescendantCollector();
            self.AddOutlineRectsForNormalChildren(
                child_collector.as_mut(),
                PhysicalOffset::default(),
                outline_type,
                self.GetLayoutObject(),
            );
            collector.CombineWithOffset(child_collector.as_mut(), &additional_offset);

            if ShouldIncludeBlockInkOverflowForAnchorOnly(outline_type) {
                for child in self.PostLayoutChildren().iter() {
                    if child.IsOutOfFlowPositioned() {
                        self.AddOutlineRectsForDescendant(
                            child.get(),
                            child.offset,
                            collector,
                            additional_offset,
                            outline_type,
                            self.GetLayoutObject(),
                        );
                    }
                }
            }
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1293-1373
    fn AddOutlineRectsForInlineBox(
        &self,
        mut additional_offset: PhysicalOffset,
        outline_type: OutlineType,
        container_relative: bool,
        collector: &mut dyn OutlineRectCollector,
    ) {
        debug_assert_eq!(self.PostLayout(), self as *const Self);
        debug_assert!(self.IsInlineBox());

        let container = self.InlineContainerFragmentIfOutlineOwner();
        if container.is_null() {
            return;
        }
        let container = unsafe { &*container };
        let layout_object = self.GetLayoutObject();
        debug_assert!(!layout_object.is_null());
        debug_assert!(unsafe { &*layout_object }.IsLayoutInline());

        let mut cursor_collector = collector.ForDescendantCollector();
        let mut cursor = InlineCursor::from_fragment(container);
        cursor.MoveToLayoutObject(unsafe { &*layout_object });
        debug_assert!(cursor.IsNotNull());
        if cursor.IsNull() {
            return;
        }
        let this_offset_in_container = cursor.Current().OffsetInContainerFragment();
        while cursor.IsNotNull() {
            let current = *cursor.Current();
            if !current.Size().IsZero() {
                let fragment = current.BoxFragment();
                debug_assert!(!fragment.is_null());
                if !fragment.is_null()
                    && !unsafe { &*fragment }.IsOpaque()
                    && !unsafe { &*fragment }.IsSvg()
                {
                    cursor_collector.AddRect(&current.RectInContainerFragment());
                }
            }

            if current.HasChildren() {
                let mut descendants = cursor.CursorForDescendants();
                container.AddOutlineRectsForCursor(
                    cursor_collector.as_mut(),
                    PhysicalOffset::default(),
                    outline_type,
                    layout_object,
                    &mut descendants,
                );
            }
            cursor.MoveToNextForSameLayoutObject();
        }
        if cursor_collector.IsEmpty() {
            return;
        }

        if !container_relative {
            additional_offset -= this_offset_in_container;
        }
        collector.CombineWithOffset(cursor_collector.as_mut(), &additional_offset);

        if ShouldIncludeBlockInkOverflowForAnchorOnly(outline_type) && !self.HasNonVisibleOverflow()
        {
            for child in container.PostLayoutChildren().iter() {
                if !child.IsOutOfFlowPositioned() {
                    continue;
                }
                let child_layout_object = child.GetLayoutObject();
                if child_layout_object.is_null()
                    || unsafe { &*child_layout_object }.ContainerForAbsolutePosition()
                        != layout_object.cast_mut()
                {
                    continue;
                }
                container.AddOutlineRectsForDescendant(
                    child.get(),
                    child.offset,
                    collector,
                    additional_offset,
                    outline_type,
                    layout_object,
                );
            }
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1375-1405
    fn AddOutlineRectsForNormalChildren(
        &self,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: PhysicalOffset,
        outline_type: OutlineType,
        containing_block: *const LayoutObject,
    ) {
        debug_assert_eq!(self.PostLayout(), self as *const Self);
        let items = self.Items();
        if !items.is_null() {
            let mut cursor = InlineCursor::new_with_items(self, unsafe { &*items });
            self.AddOutlineRectsForCursor(
                collector,
                additional_offset,
                outline_type,
                containing_block,
                &mut cursor,
            );
            return;
        }

        for child in self.PostLayoutChildren().iter() {
            if child.IsOutOfFlowPositioned() {
                continue;
            }
            self.AddOutlineRectsForDescendant(
                child.get(),
                child.offset,
                collector,
                additional_offset,
                outline_type,
                containing_block,
            );
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1406-1464
    fn AddOutlineRectsForCursor(
        &self,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: PhysicalOffset,
        outline_type: OutlineType,
        containing_block: *const LayoutObject,
        cursor: &mut InlineCursor,
    ) {
        let text_combine =
            if !containing_block.is_null() && unsafe { &*containing_block }.IsLayoutTextCombine() {
                Some(unsafe { &*containing_block })
            } else {
                None
            };
        while cursor.IsNotNull() {
            let current = *cursor.Current();
            let item = unsafe { &*current.Item() };
            if item.IsLayoutObjectDestroyedOrMoved() {
                cursor.MoveToNext();
                continue;
            }
            match item.Type() {
                FragmentItemType::kLine => {
                    // PhysicalLineBoxFragment is not a box descendant and the
                    // source descendant helper drops it after DynamicTo.
                }
                FragmentItemType::kGeneratedText | FragmentItemType::kText => {
                    let mut rect = if item.IsSvgText() {
                        PhysicalRect::EnclosingRect(&current.ObjectBoundingBox(cursor))
                    } else if ShouldIncludeBlockInkOverflow(outline_type) {
                        current.RectInContainerFragment()
                    } else {
                        cursor.MoveToNext();
                        continue;
                    };
                    if let Some(text_combine) = text_combine {
                        let objects = unsafe { &*text_combine }.FactorySet();
                        let adjust = unsafe { objects.as_ref() }
                            .and_then(|objects| objects.adjust_text_combine_bounding_box)
                            .expect("source text-combine bounding-box dispatch is not installed");
                        rect = adjust(text_combine, &rect);
                    }
                    rect.offset += additional_offset;
                    collector.AddRect(&rect);
                }
                FragmentItemType::kBox => {
                    let child_box = item.PostLayoutBoxFragment();
                    if !child_box.is_null() {
                        self.AddOutlineRectsForDescendant(
                            child_box as *const PhysicalFragment,
                            current.OffsetInContainerFragment(),
                            collector,
                            additional_offset,
                            outline_type,
                            containing_block,
                        );
                        cursor.MoveToNextSkippingChildren();
                        continue;
                    }
                }
                FragmentItemType::kInvalid => unreachable!("invalid FragmentItem in outline walk"),
            }
            cursor.MoveToNext();
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1466-1519
    fn AddOutlineRectsForDescendant(
        &self,
        descendant: *const PhysicalFragment,
        descendant_offset: PhysicalOffset,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: PhysicalOffset,
        outline_type: OutlineType,
        containing_block: *const LayoutObject,
    ) {
        if descendant.is_null() {
            return;
        }
        let descendant = unsafe { &*descendant };
        if descendant.IsListMarker() {
            return;
        }
        let descendant_box = DynamicTo::<PhysicalBoxFragment>(descendant);
        if descendant_box.is_null() {
            return;
        }
        let descendant_box = unsafe { &*descendant_box };
        if descendant_box.HasLayer() {
            let descendant_layout_object = descendant_box.GetLayoutObject();
            debug_assert!(!descendant_layout_object.is_null());
            let mut descendant_collector = collector.ForDescendantCollector();
            descendant_box.AddOutlineRectsInternal(
                PhysicalOffset::default(),
                outline_type,
                true,
                descendant_collector.as_mut(),
            );
            collector.CombineWithDescendant(
                descendant_collector.as_mut(),
                unsafe { &*descendant_layout_object },
                containing_block.cast(),
                &additional_offset,
            );
            return;
        }
        if !descendant_box.IsInlineBox() {
            descendant_box.AddSelfOutlineRects(
                additional_offset + descendant_offset,
                outline_type,
                collector,
                None,
            );
            return;
        }
        if descendant_box.IsOutlineOwner() {
            self.AddOutlineRectsForInlineNormalChildren(
                descendant_box,
                collector,
                PhysicalOffset::default(),
                outline_type,
            );
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1505-1519
    // cpp: core/layout/layout_box_model_object.cc:380-395
    fn AddOutlineRectsForInlineNormalChildren(
        &self,
        inline_fragment: &PhysicalBoxFragment,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: PhysicalOffset,
        outline_type: OutlineType,
    ) {
        let container = inline_fragment.InlineContainerFragmentIfOutlineOwner();
        if container.is_null() {
            return;
        }
        let container = unsafe { &*container };
        let layout_object = inline_fragment.GetLayoutObject();
        if layout_object.is_null() {
            return;
        }

        let mut cursor = InlineCursor::from_fragment(container);
        cursor.MoveToLayoutObject(unsafe { &*layout_object });
        while cursor.IsNotNull() {
            if cursor.Current().HasChildren() {
                let mut children = cursor.CursorForDescendants();
                while children.IsNotNull() {
                    let current = *children.Current();
                    let item = unsafe { &*current.Item() };
                    if item.Type() == FragmentItemType::kBox {
                        let child_box = item.PostLayoutBoxFragment();
                        if !child_box.is_null() && !unsafe { &*child_box }.IsOutOfFlowPositioned() {
                            container.AddOutlineRectsForDescendant(
                                child_box as *const PhysicalFragment,
                                current.OffsetInContainerFragment(),
                                collector,
                                additional_offset,
                                outline_type,
                                layout_object,
                            );
                        }
                    }
                    children.MoveToNextSkippingChildren();
                }
            }
            cursor.MoveToNextForSameLayoutObject();
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:739-744
    pub fn AllowFrom(fragment: &PhysicalFragment) -> bool {
        fragment.Type() == FragmentType::kFragmentBox
    }

    const HAS_ITEMS_BIT: u32 = 0;
    const INLINE_FORMATTING_CONTEXT_BIT: u32 = 1;
    const BORDER_TOP_BIT: u32 = 2;
    const BORDER_RIGHT_BIT: u32 = 3;
    const BORDER_BOTTOM_BIT: u32 = 4;
    const BORDER_LEFT_BIT: u32 = 5;
    const INK_OVERFLOW_SHIFT: u32 = 6;
    const FIRST_FOR_NODE_BIT: u32 = 9;
    const FRAGMENTATION_CONTEXT_ROOT_BIT: u32 = 10;
    const MONOLITHIC_BIT: u32 = 11;
    const MONOLITHIC_OVERFLOW_PROPAGATION_DISABLED_BIT: u32 = 12;
    const MOVED_CHILDREN_BIT: u32 = 13;

    // The GC allocator accepts additional bytes after the fixed object.
    // Reserve the maximum alignment padding as well because Rust may give
    // this repr(C) value a lower alignment than FragmentItems.
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:337-353
    fn AdditionalByteSize(has_fragment_items: bool) -> usize {
        if has_fragment_items {
            std::mem::align_of::<FragmentItems>() - 1 + std::mem::size_of::<FragmentItems>()
        } else {
            0
        }
    }

    // SAFETY: `ptr` must point to an allocation with AdditionalByteSize(true)
    // bytes after a PhysicalBoxFragment. The caller initializes or reads the
    // returned slot only when the fragment's item-presence bit is set.
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:66-75
    unsafe fn ComputeItemsAddressFor(ptr: *mut Self) -> *mut FragmentItems {
        let address = unsafe { ptr.add(1) } as usize;
        let align = std::mem::align_of::<FragmentItems>();
        let aligned = (address + align - 1) & !(align - 1);
        aligned as *mut FragmentItems
    }

    fn ComputeItemsAddress(&self) -> *mut FragmentItems {
        debug_assert!(self.HasItems());
        unsafe { Self::ComputeItemsAddressFor(self as *const Self as *mut Self) }
    }

    fn flag(&self, bit: u32) -> bool {
        self.bit_field_.load(Ordering::Relaxed) & (1 << bit) != 0
    }

    fn set_flag(&self, bit: u32, value: bool) {
        let mask = 1 << bit;
        if value {
            self.bit_field_.fetch_or(mask, Ordering::Relaxed);
        } else {
            self.bit_field_.fetch_and(!mask, Ordering::Relaxed);
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:69-74
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:461-481
    fn copy_from(
        other: &Self,
        _has_scrollable_overflow: bool,
        _scrollable_overflow: PhysicalRect,
        items_storage: *mut FragmentItems,
    ) -> Self {
        let children = unsafe { &*other.children_.get() };
        let mut copied_children = HeapVector::default();
        copied_children.ReserveInitialCapacity(children.len() as u32);
        for child in children.iter() {
            copied_children.push(PhysicalFragmentLink {
                fragment: child.fragment.clone(),
                offset: child.offset,
            });
        }
        let rare = other
            .rare_data()
            .map(|data| {
                Member::from_ptr(MakeGarbageCollected(PhysicalFragmentRareData::copy_from(
                    data,
                )))
            })
            .unwrap_or_default();
        if other.HasItems() {
            unsafe {
                items_storage.write((*other.Items()).clone());
            }
        }
        Self {
            base_: UnsafeCell::new(other.deref().clone()),
            bit_field_: AtomicU32::new(other.bit_field_.load(Ordering::Relaxed)),
            first_baseline_: other.first_baseline_,
            last_baseline_: other.last_baseline_,
            rare_data_: UnsafeCell::new(rare),
            ink_overflow_: UnsafeCell::new(InkOverflow::new(other.InkOverflowType(), unsafe {
                &*other.ink_overflow_.get()
            })),
            children_: UnsafeCell::new(copied_children),
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:47-48
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:263-271
    pub fn Clone(other: &Self) -> *const Self {
        let additional = Self::AdditionalByteSize(other.HasItems());
        unsafe {
            MakeGarbageCollectedWithAdditionalBytes(additional, |ptr: *mut Self| {
                let items_storage = Self::ComputeItemsAddressFor(ptr);
                ptr.write(Self::copy_from(
                    other,
                    other.HasScrollableOverflow(),
                    other.ScrollableOverflow(),
                    items_storage,
                ));
            })
        }
    }

    // The allocated trailing storage follows the source's inline item layout.
    // cpp: layoutng_fragment_tree/physical_box_fragment.h:42-45
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:179-260
    pub fn Create(
        builder: &mut BoxFragmentBuilder,
        block_or_line_writing_mode: WritingMode,
    ) -> *const Self {
        let writing_direction = builder.GetWritingDirection();
        let borders = builder
            .ApplicableBorders()
            .ConvertToPhysical(writing_direction);
        let scrollbar = builder
            .ApplicableScrollbar()
            .ConvertToPhysical(writing_direction);
        let padding = builder
            .ApplicablePadding()
            .ConvertToPhysical(writing_direction);
        let physical_size = ToPhysicalSize(*builder.Size(), builder.GetWritingMode());
        let inflow_bounds = builder.inflow_bounds_.map(|rect| {
            WritingModeConverter::new(writing_direction, physical_size).ToPhysicalRect(rect)
        });
        #[cfg(debug_assertions)]
        {
            if builder.needs_inflow_bounds_explicitly_set_
                && !builder.node_.IsNull()
                && builder.node_.IsScrollContainer()
                && !builder.IsFragmentainerBoxType()
            {
                debug_assert!(builder.is_inflow_bounds_explicitly_set_);
            }
            if builder.needs_may_have_descendant_above_block_start_explicitly_set_ {
                debug_assert!(builder.is_may_have_descendant_above_block_start_explicitly_set_);
            }
        }
        let mut scrollable_overflow = PhysicalRect::new(PhysicalOffset::default(), physical_size);
        if builder.ShouldCalculateScrollableOverflow() {
            let block_node = BlockNode::from(builder.node_.clone());
            let mut calculator = ScrollableOverflowCalculator::new(
                &block_node,
                !builder.IsFragmentainerBoxType(),
                builder.GetConstraintSpace().HasBlockFragmentation(),
                &borders,
                &scrollbar,
                &padding,
                physical_size,
                writing_direction,
            );
            let items_builder = builder.ItemsBuilder();
            if !items_builder.is_null() {
                calculator.AddBuilderItems(
                    builder.GetLayoutObject(),
                    unsafe { &mut *items_builder }.Items(physical_size),
                );
            }
            for child in &builder.children_ {
                let box_fragment = DynamicTo::<Self>(child.fragment.Get());
                if box_fragment.is_null() {
                    continue;
                }
                let box_fragment = unsafe { &*box_fragment };
                calculator.AddChild(
                    box_fragment,
                    child.offset.ConvertToPhysical(
                        writing_direction,
                        physical_size,
                        box_fragment.Size(),
                    ),
                );
            }
            if !builder.table_collapsed_borders_.is_null() {
                calculator.AddTableSelfRect();
            }
            scrollable_overflow = calculator.Result(inflow_bounds);
        }
        let has_scrollable_overflow =
            scrollable_overflow != PhysicalRect::new(PhysicalOffset::default(), physical_size);
        let items_builder = builder.ItemsBuilder();
        let has_items = !items_builder.is_null() && unsafe { &*items_builder }.Size() != 0;
        let additional = Self::AdditionalByteSize(has_items);
        unsafe {
            MakeGarbageCollectedWithAdditionalBytes(additional, |ptr: *mut Self| {
                let items_storage = Self::ComputeItemsAddressFor(ptr);
                ptr.write(Self::from_builder(
                    builder,
                    has_scrollable_overflow,
                    scrollable_overflow,
                    (!borders.IsZero()).then_some(&borders),
                    (!scrollbar.IsZero()).then_some(&scrollbar),
                    (!padding.IsZero()).then_some(&padding),
                    inflow_bounds,
                    has_items,
                    block_or_line_writing_mode,
                    items_storage,
                ));
            })
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:50-53
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:273-335
    pub fn CloneWithPostLayoutFragments(other: &Self) -> *const Self {
        let clone = Self::Clone(other);
        #[cfg(debug_assertions)]
        let _allow_post_layout = AllowPostLayoutScope::new();
        let result = unsafe { &*clone };
        for child in result.GetMutableForCloning().Children() {
            child.fragment = Member::from_ptr(unsafe { &*child.get() }.PostLayout() as *mut _);
            debug_assert!(!child.get().is_null());
            let mut box_fragment = DynamicTo::<Self>(child.get()) as *const Self;
            if box_fragment.is_null() {
                continue;
            }
            if unsafe { &*box_fragment }.GetBoxType()
                == crate::physical_fragment::BoxType::kPageContainer
            {
                box_fragment = GetPageArea(GetPageBorderBox(unsafe { &*box_fragment })) as *const _;
            }
            if !unsafe { &*box_fragment }.IsFragmentainerBox() {
                continue;
            }
            for fragmentainer_child in unsafe { &*box_fragment }.GetMutableForCloning().Children() {
                let old_child = unsafe { &*(fragmentainer_child.get() as *const Self) };
                fragmentainer_child.fragment = Member::from_ptr(old_child.PostLayout() as *mut _);
            }
        }
        if result.HasItems() {
            for item in unsafe { &*result.Items() }.Items() {
                let box_fragment = item.BoxFragment();
                if box_fragment.is_null() {
                    continue;
                }
                let latest = unsafe { &*box_fragment }.PostLayout();
                debug_assert!(!latest.is_null());
                item.GetMutableForCloning()
                    .ReplaceBoxFragment(unsafe { &*latest });
            }
        }
        clone
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:55-67
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:355-459
    fn from_builder(
        builder: &mut BoxFragmentBuilder,
        has_scrollable_overflow: bool,
        scrollable_overflow: PhysicalRect,
        borders: Option<&PhysicalBoxStrut>,
        scrollbar: Option<&PhysicalBoxStrut>,
        padding: Option<&PhysicalBoxStrut>,
        inflow_bounds: Option<PhysicalRect>,
        has_fragment_items: bool,
        block_or_line_writing_mode: WritingMode,
        items_storage: *mut FragmentItems,
    ) -> Self {
        let box_type = builder.GetBoxType();
        let mut base = PhysicalFragment::from_builder(
            &mut builder.base_,
            block_or_line_writing_mode,
            FragmentType::kFragmentBox,
            box_type as u32,
        );
        // Chromium reads layout_object_ here, which also owns anonymous
        // fragmentainers. GetLayoutObject() intentionally excludes them.
        let layout_object = base.GetSelfOrContainerLayoutObject();
        debug_assert!(!layout_object.is_null());
        debug_assert!(unsafe { &*layout_object }.IsBoxModelObject());
        debug_assert!(
            builder.break_token_.is_null() || unsafe { &*builder.break_token_ }.IsBlockType()
        );
        let converter = WritingModeConverter::new(
            foundation::WritingDirectionMode::new(block_or_line_writing_mode, builder.Direction()),
            base.Size(),
        );
        let mut children = HeapVector::default();
        children.ReserveInitialCapacity(builder.children_.len() as u32);
        for child in builder.children_.iter_mut() {
            let size = unsafe { &*child.fragment.Get() }.Size();
            let offset = converter.ToPhysicalOffset(child.offset, size);
            children.push(PhysicalFragmentLink {
                fragment: std::mem::take(&mut child.fragment),
                offset,
            });
        }

        if has_fragment_items {
            let items_builder = builder.ItemsBuilder();
            debug_assert!(!items_builder.is_null());
            debug_assert_eq!(
                unsafe { &*items_builder }.GetWritingMode(),
                block_or_line_writing_mode
            );
            debug_assert_eq!(unsafe { &*items_builder }.Direction(), builder.Direction());
            let new_size =
                unsafe { (&mut *items_builder).ToFragmentItems(base.Size(), items_storage) };
            if let Some(size) = new_size {
                base.size_ = size;
            }
        }

        let rare_fields_size = has_scrollable_overflow as u32
            + builder.frame_set_layout_data_.is_some() as u32
            + (!builder.mathml_paint_info_.is_null()) as u32
            + builder.table_grid_rect_.is_some() as u32
            + (!builder.table_collapsed_borders_.is_null()) as u32
            + builder.table_collapsed_borders_geometry_.is_some() as u32
            + builder.table_cell_column_index_.is_some() as u32
            + if builder.table_section_row_offsets_.is_empty() {
                0
            } else {
                2
            }
            + (!builder.page_name_.IsNull()) as u32
            + borders.is_some() as u32
            + scrollbar.is_some() as u32
            + padding.is_some() as u32
            + inflow_bounds.is_some() as u32
            + builder.Style().MayHaveMargin() as u32;
        let has_rare = rare_fields_size > 0
            || !builder.table_column_geometries_.is_empty()
            || !builder.reading_flow_nodes_.is_empty()
            || !builder.gap_geometry_.is_null();
        let rare_data = if has_rare {
            Member::from_ptr(MakeGarbageCollected(
                PhysicalFragmentRareData::from_builder(
                    has_scrollable_overflow.then_some(&scrollable_overflow),
                    borders,
                    scrollbar,
                    padding,
                    inflow_bounds,
                    builder,
                    rare_fields_size,
                ),
            ))
        } else {
            Member::default()
        };

        base.flags_.set(
            PhysicalFragmentFlags::IS_FIELDSET_CONTAINER,
            builder.is_fieldset_container_,
        );
        base.flags_
            .set(PhysicalFragmentFlags::IS_TABLE_PART, builder.is_table_part_);
        base.flags_.set(
            PhysicalFragmentFlags::IS_PAINTED_ATOMICALLY,
            builder.space_.IsPaintedAtomically(),
        );
        base.flags_.set(
            PhysicalFragmentFlags::IS_MATH_FRACTION,
            builder.is_math_fraction_,
        );
        base.flags_.set(
            PhysicalFragmentFlags::IS_MATH_OPERATOR,
            builder.is_math_operator_,
        );

        let allow_baseline = !unsafe { &*layout_object }.ShouldApplyLayoutContainment()
            || unsafe { &*layout_object }.IsTableCell();
        let first_baseline = if allow_baseline {
            builder.first_baseline_
        } else {
            None
        };
        let last_baseline = if allow_baseline {
            builder.last_baseline_
        } else {
            None
        };
        base.flags_.set(
            PhysicalFragmentFlags::HAS_FIRST_BASELINE,
            first_baseline.is_some(),
        );
        base.flags_.set(
            PhysicalFragmentFlags::HAS_LAST_BASELINE,
            last_baseline.is_some(),
        );
        base.flags_.set(
            PhysicalFragmentFlags::USE_LAST_BASELINE_FOR_INLINE_BASELINE,
            builder.use_last_baseline_for_inline_baseline_,
        );

        let initial_flags = ((has_fragment_items as u32) << Self::HAS_ITEMS_BIT)
            | ((builder.is_fragmentation_context_root_ as u32)
                << Self::FRAGMENTATION_CONTEXT_ROOT_BIT)
            | ((builder.is_monolithic_ as u32) << Self::MONOLITHIC_BIT)
            | ((builder
                .GetConstraintSpace()
                .IsMonolithicOverflowPropagationDisabled() as u32)
                << Self::MONOLITHIC_OVERFLOW_PROPAGATION_DISABLED_BIT)
            | ((builder.has_moved_children_ as u32) << Self::MOVED_CHILDREN_BIT);
        let result = Self {
            base_: UnsafeCell::new(base),
            bit_field_: AtomicU32::new(initial_flags),
            first_baseline_: first_baseline.unwrap_or_else(LayoutUnit::Min),
            last_baseline_: last_baseline.unwrap_or_else(LayoutUnit::Min),
            rare_data_: UnsafeCell::new(rare_data),
            ink_overflow_: UnsafeCell::new(InkOverflow::default()),
            children_: UnsafeCell::new(children),
        };
        result.SetInkOverflowType(InkOverflowType::kNotSet);
        result.set_flag(Self::FIRST_FOR_NODE_BIT, builder.is_first_for_node_);
        let sides = PhysicalBoxSides::from_line_logical(
            builder.sides_to_include_,
            builder.GetWritingMode(),
        );
        result.set_flag(Self::BORDER_TOP_BIT, sides.top);
        result.set_flag(Self::BORDER_RIGHT_BIT, sides.right);
        result.set_flag(Self::BORDER_BOTTOM_BIT, sides.bottom);
        result.set_flag(Self::BORDER_LEFT_BIT, sides.left);
        result.set_flag(
            Self::INLINE_FORMATTING_CONTEXT_BIT,
            builder.is_inline_formatting_context_,
        );
        #[cfg(debug_assertions)]
        result.CheckIntegrity();
        result
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:720-722
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1876-1922
    #[cfg(debug_assertions)]
    fn CheckIntegrity(&self) {
        let mut has_inflow_blocks = false;
        let mut has_inlines = false;
        let mut has_line_boxes = false;
        let mut has_floats = false;
        let mut has_list_markers = false;
        for child in self.Children() {
            if child.IsFloating() {
                has_floats = true;
            } else if child.IsOutOfFlowPositioned() { /* May be in tree without items. */
            } else if child.IsLineBox() {
                has_line_boxes = true;
            } else if child.IsListMarker() {
                has_list_markers = true;
            } else if child.IsInline() {
                has_inlines = true;
            } else {
                has_inflow_blocks = true;
            }
        }
        if has_line_boxes || has_inlines {
            debug_assert!(self.IsInlineFormattingContext());
        }
        let object = self.GetSelfOrContainerLayoutObject();
        debug_assert!(!object.is_null());
        if !object.is_null() && ContentLockBlocksChildLayout(unsafe { &*object }.GetNode()) {
            return;
        }
        if has_line_boxes {
            debug_assert!(self.HasItems());
        }
        if has_line_boxes {
            debug_assert!(!has_inlines && !has_inflow_blocks);
            debug_assert!(!has_floats || !self.IsFirstForNode());
            debug_assert!(!has_list_markers);
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:466-469
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1792-1873
    #[cfg(debug_assertions)]
    pub fn CheckSameForSimplifiedLayout(&self, other: &Self, check_no_fragmentation: bool) {
        debug_assert_eq!(
            self.GetSelfOrContainerLayoutObject(),
            other.GetSelfOrContainerLayoutObject()
        );
        debug_assert_eq!(self.Size(), other.Size());
        if check_no_fragmentation {
            debug_assert!(self.GetBreakToken().is_null() && other.GetBreakToken().is_null());
        }

        let stable_base_flags = [
            PhysicalFragmentFlags::IS_HIDDEN_FOR_PAINT,
            PhysicalFragmentFlags::IS_OPAQUE,
            PhysicalFragmentFlags::IS_BLOCK_IN_INLINE,
            PhysicalFragmentFlags::IS_MATH_FRACTION,
            PhysicalFragmentFlags::IS_MATH_OPERATOR,
            PhysicalFragmentFlags::HAS_ADJOINING_OBJECT_DESCENDANTS,
            PhysicalFragmentFlags::MAY_HAVE_DESCENDANT_ABOVE_BLOCK_START,
            PhysicalFragmentFlags::IS_FIELDSET_CONTAINER,
            PhysicalFragmentFlags::IS_TABLE_PART,
            PhysicalFragmentFlags::IS_PAINTED_ATOMICALLY,
            PhysicalFragmentFlags::HAS_COLLAPSED_BORDERS,
        ];
        for field in stable_base_flags {
            debug_assert_eq!(self.flags_.get(field), other.flags_.get(field));
        }
        debug_assert_eq!(self.Type(), other.Type());
        debug_assert_eq!(self.GetBoxType(), other.GetBoxType());
        debug_assert_eq!(self.GetStyleVariant(), other.GetStyleVariant());
        debug_assert_eq!(
            self.IsFragmentationContextRoot(),
            other.IsFragmentationContextRoot()
        );
        if !self.IsOutOfFlowPositioned() {
            debug_assert_eq!(
                self.DependsOnPercentageBlockSize(),
                other.DependsOnPercentageBlockSize()
            );
        }
        debug_assert_eq!(self.HasItems(), other.HasItems());
        debug_assert_eq!(
            self.IsInlineFormattingContext(),
            other.IsInlineFormattingContext()
        );
        debug_assert_eq!(self.SidesToInclude(), other.SidesToInclude());
        debug_assert_eq!(self.FirstBaseline(), other.FirstBaseline());
        debug_assert_eq!(self.LastBaseline(), other.LastBaseline());

        if self.IsTable() {
            debug_assert_eq!(self.TableGridRect(), other.TableGridRect());
            let columns = self.TableColumnGeometries();
            let other_columns = other.TableColumnGeometries();
            if columns.is_null() {
                debug_assert!(other_columns.is_null());
            } else {
                debug_assert!(!other_columns.is_null());
                debug_assert!(unsafe { &*columns } == unsafe { &*other_columns });
            }
            debug_assert_eq!(self.TableCollapsedBorders(), other.TableCollapsedBorders());
            let geometry = self.TableCollapsedBordersGeometry();
            let other_geometry = other.TableCollapsedBordersGeometry();
            if geometry.is_null() {
                debug_assert!(other_geometry.is_null());
            } else {
                debug_assert!(!other_geometry.is_null());
                unsafe { &*geometry }.CheckSameForSimplifiedLayout(unsafe { &*other_geometry });
            }
        }
        if self.IsTableCell() {
            debug_assert_eq!(self.TableCellColumnIndex(), other.TableCellColumnIndex());
        }
        debug_assert_eq!(self.Borders(), other.Borders());
        debug_assert_eq!(self.Padding(), other.Padding());
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:622-625
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1925-1929
    #[cfg(debug_assertions)]
    pub fn AssertFragmentTreeSelf(&self) {
        debug_assert!(!self.IsInlineBox());
        debug_assert!(!self.OwnerLayoutBox().is_null());
        debug_assert_eq!(self as *const Self, self.PostLayout());
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1931-1958
    #[cfg(debug_assertions)]
    pub fn AssertFragmentTreeChildren(&self, allow_destroyed_or_moved: bool) {
        let items = self.Items();
        if !items.is_null() {
            let mut cursor = InlineCursor::new_with_items(self, unsafe { &*items });
            while cursor.IsNotNull() {
                let item = unsafe { &*cursor.Current().Item() };
                if item.IsLayoutObjectDestroyedOrMoved() {
                    debug_assert!(allow_destroyed_or_moved);
                    cursor.MoveToNext();
                    continue;
                }
                let box_fragment = item.BoxFragment();
                if !box_fragment.is_null() {
                    let box_fragment = unsafe { &*box_fragment };
                    debug_assert!(!box_fragment.IsLayoutObjectDestroyedOrMoved());
                    if !box_fragment.IsInlineBox() {
                        box_fragment.AssertFragmentTreeSelf();
                    }
                }
                cursor.MoveToNext();
            }
        }
        for child in self.Children() {
            if child.IsLayoutObjectDestroyedOrMoved() {
                debug_assert!(allow_destroyed_or_moved);
                continue;
            }
            let box_fragment = DynamicTo::<Self>(child.get());
            if !box_fragment.is_null() {
                unsafe { &*box_fragment }.AssertFragmentTreeSelf();
            }
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:98-104
    pub fn Children(&self) -> &[PhysicalFragmentLink] {
        debug_assert!(self.ChildrenValid());
        unsafe { &*self.children_.get() }.as_slice()
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:106-111
    pub fn ReadingFlowNodes(&self) -> *const GCedHeapVector<Member<Node>> {
        let rare = unsafe { &*self.rare_data_.get() }.Get();
        if rare.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*rare }.reading_flow_nodes_.Get()
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:115-118
    pub fn PostLayoutChildren(&self) -> PostLayoutChildLinkList<'_> {
        debug_assert!(self.ChildrenValid());
        PostLayoutChildLinkList::new(self.Children())
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:139-143
    pub fn GetMutableChildrenForOutOfFlow(&self) -> MutableChildrenForOutOfFlow<'_> {
        debug_assert!(self.ChildrenValid());
        MutableChildrenForOutOfFlow { fragment_: self }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:120-121
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:630-639
    pub fn SetChildrenInvalid(&self) {
        if !self.ChildrenValid() {
            return;
        }
        for child in unsafe { &mut *self.children_.get() }.iter_mut() {
            child.fragment = Member::default();
        }
        self.deref()
            .flags_
            .set(PhysicalFragmentFlags::CHILDREN_VALID, false);
    }

    pub fn ChildrenValid(&self) -> bool {
        self.deref()
            .flags_
            .get(PhysicalFragmentFlags::CHILDREN_VALID)
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:146-153
    pub fn HasItems(&self) -> bool {
        self.flag(Self::HAS_ITEMS_BIT)
    }

    pub fn Items(&self) -> *const FragmentItems {
        if !self.HasItems() {
            return std::ptr::null();
        }
        self.ComputeItemsAddress()
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:155-169
    pub fn FirstBaseline(&self) -> Option<LayoutUnit> {
        self.deref()
            .flags_
            .get(PhysicalFragmentFlags::HAS_FIRST_BASELINE)
            .then_some(self.first_baseline_)
    }

    pub fn LastBaseline(&self) -> Option<LayoutUnit> {
        self.deref()
            .flags_
            .get(PhysicalFragmentFlags::HAS_LAST_BASELINE)
            .then_some(self.last_baseline_)
    }

    pub fn UseLastBaselineForInlineBaseline(&self) -> bool {
        self.deref()
            .flags_
            .get(PhysicalFragmentFlags::USE_LAST_BASELINE_FOR_INLINE_BASELINE)
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:173
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:56-60
    pub fn ForceInlineBaselineSynthesis(&self) -> bool {
        self.UseLastBaselineForInlineBaseline()
            && self.IsScrollContainer()
            && self.Style().IsOverflowValueScrollableBlock()
            && !self
                .Style()
                .ShouldIgnoreOverflowPropertyForInlineBlockBaseline()
    }

    fn rare_data(&self) -> Option<&PhysicalFragmentRareData> {
        unsafe { (&*self.rare_data_.get()).Get().as_ref() }
    }

    fn rare_data_mut(&self) -> Option<&mut PhysicalFragmentRareData> {
        unsafe { (&mut *self.rare_data_.get()).Get().as_mut() }
    }

    fn base_mut(&self) -> &mut PhysicalFragment {
        unsafe { &mut *self.base_.get() }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:675
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:851-857
    fn EnsureRareField(&self, id: FieldId) -> &mut RareField {
        if self.rare_data().is_none() {
            let new_data = foundation::MakeGarbageCollected(PhysicalFragmentRareData::new(1));
            unsafe {
                *self.rare_data_.get() = Member::from_ptr(new_data);
            }
        }
        self.rare_data_mut()
            .expect("rare data allocated")
            .EnsureField(id)
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:668-674
    fn GetRareField(&self, id: FieldId) -> Option<&RareField> {
        self.rare_data().and_then(|rare| rare.GetField(id))
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:175-232
    pub fn GetGapGeometry(&self) -> *const GapGeometry {
        self.rare_data()
            .map_or(std::ptr::null(), |rare| rare.gap_geometry_.Get())
    }

    pub fn TableGridRect(&self) -> LogicalRect {
        let field = self
            .GetRareField(FieldId::kTableGridRect)
            .expect("missing table grid rect");
        unsafe { *field.value_.table_grid_rect }
    }

    pub fn TableColumnGeometries(&self) -> *const GCedTableColumnGeometries {
        self.rare_data()
            .map_or(std::ptr::null(), |rare| rare.table_column_geometries_.Get())
    }

    pub fn TableCollapsedBorders(&self) -> *const TableBorders {
        self.rare_data()
            .map_or(std::ptr::null(), |rare| rare.table_collapsed_borders_.Get())
    }

    pub fn TableCollapsedBordersGeometry(&self) -> *const CollapsedTableBordersGeometry {
        self.GetRareField(FieldId::kTableCollapsedBordersGeometry)
            .and_then(|field| unsafe { field.value_.table_collapsed_borders_geometry.as_ref() })
            .map_or(std::ptr::null(), |value| &**value as *const _)
    }

    pub fn TableCellColumnIndex(&self) -> u32 {
        let field = self
            .GetRareField(FieldId::kTableCellColumnIndex)
            .expect("missing table column index");
        unsafe { *field.value_.table_cell_column_index }
    }

    pub fn TableSectionStartRowIndex(&self) -> Option<u32> {
        debug_assert!(self.IsTableSection());
        self.GetRareField(FieldId::kTableSectionStartRowIndex)
            .map(|field| unsafe { *field.value_.table_section_start_row_index })
    }

    pub fn TableSectionRowOffsets(&self) -> Option<&[LayoutUnit]> {
        debug_assert!(self.IsTableSection());
        self.GetRareField(FieldId::kTableSectionRowOffsets)
            .map(|field| unsafe { field.value_.table_section_row_offsets.as_slice() })
    }

    pub fn PropagatedPageName(&self) -> AtomicString {
        self.GetRareField(FieldId::kPageName)
            .map_or_else(AtomicString::default, |field| unsafe {
                (*field.value_.page_name).clone()
            })
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:234-318
    pub fn ScrollableOverflow(&self) -> PhysicalRect {
        self.GetRareField(FieldId::kScrollableOverflow).map_or_else(
            || PhysicalRect::new(PhysicalOffset::default(), self.Size()),
            |field| unsafe { *field.value_.scrollable_overflow },
        )
    }

    pub fn HasScrollableOverflow(&self) -> bool {
        self.GetRareField(FieldId::kScrollableOverflow).is_some()
    }

    pub fn HasBorders(&self) -> bool {
        self.GetRareField(FieldId::kBorders).is_some()
    }
    pub fn HasScrollbar(&self) -> bool {
        self.GetRareField(FieldId::kScrollbar).is_some()
    }
    pub fn HasPadding(&self) -> bool {
        self.GetRareField(FieldId::kPadding).is_some()
    }
    pub fn HasInflowBounds(&self) -> bool {
        self.GetRareField(FieldId::kInflowBounds).is_some()
    }

    pub fn Borders(&self) -> PhysicalBoxStrut {
        self.GetRareField(FieldId::kBorders)
            .map_or_else(PhysicalBoxStrut::default, |field| unsafe {
                *field.value_.borders
            })
    }

    pub fn Scrollbar(&self) -> PhysicalBoxStrut {
        self.GetRareField(FieldId::kScrollbar)
            .map_or_else(PhysicalBoxStrut::default, |field| unsafe {
                *field.value_.scrollbar
            })
    }

    pub fn Padding(&self) -> PhysicalBoxStrut {
        self.GetRareField(FieldId::kPadding)
            .map_or_else(PhysicalBoxStrut::default, |field| unsafe {
                *field.value_.padding
            })
    }

    pub fn Margins(&self) -> PhysicalBoxStrut {
        self.GetRareField(FieldId::kMargins)
            .map_or_else(PhysicalBoxStrut::default, |field| unsafe {
                *field.value_.margins
            })
    }

    pub fn ContentOffset(&self) -> PhysicalOffset {
        let mut offset = PhysicalOffset::default();
        if self.HasBorders() {
            offset += self.Borders().Offset();
        }
        if self.HasPadding() {
            offset += self.Padding().Offset();
        }
        offset
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:490-495
    pub fn ContentRect(&self) -> PhysicalRect {
        let mut rect = PhysicalRect::new(PhysicalOffset::default(), self.Size());
        rect.Contract(&(self.Borders() + self.Padding()));
        debug_assert!(rect.size.width >= LayoutUnit::default());
        debug_assert!(rect.size.height >= LayoutUnit::default());
        rect
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:347-350
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:498-522
    pub fn OwnerLayoutBox(&self) -> *const LayoutBox {
        let owner = DynamicTo::<LayoutBox>(self.GetSelfOrContainerLayoutObject());
        #[cfg(debug_assertions)]
        {
            debug_assert!(!owner.is_null());
            if !owner.is_null() {
                if self.IsFragmentainerBox() {
                    if unsafe { &*owner }.IsLayoutView() {
                        debug_assert_eq!(
                            self.GetBoxType(),
                            crate::physical_fragment::BoxType::kPageArea
                        );
                        debug_assert!(
                            unsafe { &*(owner as *const LayoutView) }.ShouldUsePaginatedLayout()
                        );
                    } else {
                        debug_assert!(self.IsColumnBox());
                    }
                } else {
                    debug_assert!(unsafe { &*owner }
                        .PhysicalFragments()
                        .into_iter()
                        .any(|fragment| std::ptr::eq(fragment, self)));
                    debug_assert_eq!(
                        self.IsFirstForNode(),
                        self as *const Self == unsafe { &*owner }.GetPhysicalFragment(0)
                    );
                }
            }
        }
        owner
    }

    pub fn MutableOwnerLayoutBox(&self) -> *mut LayoutBox {
        self.OwnerLayoutBox() as *mut _
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:351-351
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:528-566
    pub fn OffsetFromOwnerLayoutBox(&self) -> PhysicalOffset {
        debug_assert!(self.IsCSSBox());
        let owner = unsafe { &*self.OwnerLayoutBox() };
        debug_assert!(owner.PhysicalFragments().Contains(self));
        if owner.PhysicalFragmentCount() <= 1 {
            return PhysicalOffset::default();
        }

        let containing_block = owner.ContainingBlock();
        debug_assert!(!containing_block.is_null());
        if unsafe { &*containing_block }
            .StyleRef()
            .IsLeftToRightDirection()
        {
            debug_assert_eq!(
                self.IsFirstForNode(),
                std::ptr::eq(self, owner.GetPhysicalFragment(0))
            );
            if self.IsFirstForNode() {
                return PhysicalOffset::default();
            }
            let fragment_data = owner.FragmentDataFromPhysicalFragment(self);
            debug_assert!(!fragment_data.is_null());
            return unsafe { &*fragment_data }.PaintOffset() - owner.FirstFragment().PaintOffset();
        }

        let fragment_data = owner.FragmentDataFromPhysicalFragment(self);
        debug_assert!(!fragment_data.is_null());
        unsafe { &*fragment_data }.PaintOffset() - owner.FragmentList().back().PaintOffset()
    }

    pub fn InflowBounds(&self) -> Option<PhysicalRect> {
        self.GetRareField(FieldId::kInflowBounds)
            .map(|field| unsafe { *field.value_.inflow_bounds })
    }

    pub fn OffsetFromRootFragmentationContext(&self) -> PhysicalOffset {
        self.GetRareField(FieldId::kOffsetFromRootFragmentationContext)
            .map_or_else(PhysicalOffset::default, |field| unsafe {
                *field.value_.offset_from_root_fragmentation_context
            })
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:497
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:877-885
    pub fn GetMutableForStyleRecalc(&self) -> MutableForStyleRecalc<'_> {
        debug_assert!(!self.GetSelfOrContainerLayoutObject().is_null());
        MutableForStyleRecalc { fragment_: self }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:511
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:908-913
    pub fn GetMutableForContainerLayout(&self) -> MutableForContainerLayout<'_> {
        debug_assert!(!self.GetSelfOrContainerLayoutObject().is_null());
        MutableForContainerLayout { fragment_: self }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:566-569
    pub fn GetMutableForCloning(&self) -> MutableForCloning<'_> {
        MutableForCloning { fragment_: self }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:608-610
    // cpp: layoutng_out_of_flow/out_of_flow_fragment_builder.cc:277-283
    pub fn GetMutableForOofFragmentation(&self) -> MutableForOofFragmentation<'_> {
        MutableForOofFragmentation { fragment_: self }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:534-536
    pub fn GetMutableForPainting(&self) -> MutableForPainting<'_> {
        MutableForPainting { fragment_: self }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:430-438
    pub fn GetBreakToken(&self) -> *const BlockBreakToken {
        let token = self.deref().GetBreakToken();
        if token.is_null() {
            return std::ptr::null();
        }
        assert!(unsafe { &*token }.IsBlockType());
        token as *const BlockBreakToken
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:425-426
    pub fn SidesToInclude(&self) -> PhysicalBoxSides {
        PhysicalBoxSides::new(
            self.IncludeBorderTop(),
            self.IncludeBorderRight(),
            self.IncludeBorderBottom(),
            self.IncludeBorderLeft(),
        )
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:653-664
    pub fn IncludeBorderTop(&self) -> bool {
        self.flag(Self::BORDER_TOP_BIT)
    }

    pub fn IncludeBorderRight(&self) -> bool {
        self.flag(Self::BORDER_RIGHT_BIT)
    }

    pub fn IncludeBorderBottom(&self) -> bool {
        self.flag(Self::BORDER_BOTTOM_BIT)
    }

    pub fn IncludeBorderLeft(&self) -> bool {
        self.flag(Self::BORDER_LEFT_BIT)
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:434-448
    pub fn IsFirstForNode(&self) -> bool {
        self.flag(Self::FIRST_FOR_NODE_BIT)
    }

    pub fn IsOnlyForNode(&self) -> bool {
        self.IsFirstForNode() && self.GetBreakToken().is_null()
    }

    pub fn HasDescendantsForTablePart(&self) -> bool {
        debug_assert!(self.IsTablePart() || self.IsTableCell());
        !unsafe { &*self.children_.get() }.is_empty() || self.NeedsOOFPositionedInfoPropagation()
    }

    pub fn IsFragmentationContextRoot(&self) -> bool {
        self.flag(Self::FRAGMENTATION_CONTEXT_ROOT_BIT)
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:77-79
    pub fn IsPaginatedRoot(&self) -> bool {
        self.IsFragmentationContextRoot()
            && unsafe { &*self.GetSelfOrContainerLayoutObject() }.IsLayoutView()
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:453-463
    pub fn IsMonolithic(&self) -> bool {
        self.flag(Self::MONOLITHIC_BIT)
    }

    pub fn IsMonolithicOverflowPropagationDisabled(&self) -> bool {
        self.flag(Self::MONOLITHIC_OVERFLOW_PROPAGATION_DISABLED_BIT)
    }

    pub fn HasMovedChildren(&self) -> bool {
        self.flag(Self::MOVED_CHILDREN_BIT)
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:471-483
    pub fn GetFrameSetLayoutData(&self) -> *const FrameSetLayoutData {
        self.GetRareField(FieldId::kFrameSetLayoutData)
            .and_then(|field| unsafe { field.value_.frame_set_layout_data.as_ref() })
            .map_or(std::ptr::null(), |value| &**value as *const _)
    }

    pub fn HasExtraMathMLPainting(&self) -> bool {
        if self.IsMathMLFraction() {
            return true;
        }
        self.rare_data()
            .is_some_and(|rare| !rare.mathml_paint_info_.Get().is_null())
    }

    pub fn GetMathMLPaintInfo(&self) -> &MathMLPaintInfo {
        unsafe {
            &*self
                .rare_data()
                .expect("missing rare data")
                .mathml_paint_info_
                .Get()
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:94
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:568-628
    pub fn PostLayout(&self) -> *const Self {
        if DisableLayoutSideEffectsScope::IsDisabled() {
            return self as *const _;
        }
        let layout_object = self.GetLayoutObject();
        if layout_object.is_null() {
            return self as *const _;
        }
        let box_ptr = DynamicTo::<LayoutBox>(layout_object);
        if box_ptr.is_null() {
            debug_assert!(self.IsInlineBox());
            return self as *const _;
        }
        let box_ = unsafe { &*box_ptr };
        let count = box_.PhysicalFragmentCount();
        if count == 0 {
            #[cfg(debug_assertions)]
            debug_assert!(AllowPostLayoutScope::IsAllowed());
            return std::ptr::null();
        }
        let post_layout = if count == 1 {
            box_.GetPhysicalFragment(0)
        } else {
            let token = self.GetBreakToken();
            if !token.is_null() {
                let index = unsafe { &*token }.SequenceNumber() as usize;
                if index < count {
                    box_.GetPhysicalFragment(index)
                } else {
                    std::ptr::null()
                }
            } else {
                box_.PhysicalFragments().back() as *const _
            }
        };
        if post_layout == self as *const _ {
            return post_layout;
        }
        #[cfg(debug_assertions)]
        debug_assert!(AllowPostLayoutScope::IsAllowed());
        post_layout
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:62-64
    pub fn CanUseFragmentsForInkOverflow(&self) -> bool {
        !unsafe { &*self.GetSelfOrContainerLayoutObject() }.IsLayoutReplaced()
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:368-377
    pub fn InkOverflowType(&self) -> InkOverflowType {
        match (self.bit_field_.load(Ordering::Relaxed) >> Self::INK_OVERFLOW_SHIFT) & 7 {
            0 => InkOverflowType::kNotSet,
            1 => InkOverflowType::kInvalidated,
            2 => InkOverflowType::kNone,
            3 => InkOverflowType::kSmallSelf,
            4 => InkOverflowType::kSelf,
            5 => InkOverflowType::kSmallContents,
            6 => InkOverflowType::kContents,
            7 => InkOverflowType::kSelfAndContents,
            _ => unreachable!(),
        }
    }

    pub fn IsInkOverflowComputed(&self) -> bool {
        !matches!(
            self.InkOverflowType(),
            InkOverflowType::kNotSet | InkOverflowType::kInvalidated
        )
    }

    pub fn HasInkOverflow(&self) -> bool {
        self.InkOverflowType() != InkOverflowType::kNone
    }

    fn SetInkOverflowType(&self, overflow_type: InkOverflowType) {
        let mask = 7 << Self::INK_OVERFLOW_SHIFT;
        let new_bits = (overflow_type as u32) << Self::INK_OVERFLOW_SHIFT;
        self.bit_field_
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
                Some((old & !mask) | new_bits)
            })
            .ok();
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:676-680
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:990-995
    fn SetInkOverflow(&self, self_rect: PhysicalRect, contents: PhysicalRect) {
        let current = self.InkOverflowType();
        let next = unsafe { &mut *self.ink_overflow_.get() }.Set(
            current,
            &self_rect,
            &contents,
            &self.Size(),
        );
        self.SetInkOverflowType(next);
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1202-1206
    #[cfg(debug_assertions)]
    pub fn InvalidateInkOverflow(&self) {
        let current = self.InkOverflowType();
        let next = unsafe { &mut *self.ink_overflow_.get() }.Invalidate(current);
        self.SetInkOverflowType(next);
    }

    // Source: core/layout/physical_box_fragment.cc:996-999.
    fn RecalcInkOverflowWithContents(&self, contents: PhysicalRect) {
        let self_rect = self.ComputeSelfInkOverflow();
        self.SetInkOverflow(self_rect, contents);
    }

    // Source: core/layout/physical_box_fragment.cc:1001-1036.
    fn RecalcInkOverflow(&self) {
        debug_assert!(self.CanUseFragmentsForInkOverflow());
        let layout_object = self.GetSelfOrContainerLayoutObject();
        debug_assert!(!layout_object.is_null());

        let mut contents_rect = PhysicalRect::default();
        if !unsafe { &*layout_object }.ChildPrePaintBlockedByDisplayLock() {
            contents_rect = self.RecalcContentsInkOverflow();
        }
        self.RecalcInkOverflowWithContents(contents_rect);

        if self.IsFragmentainerBox() {
            return;
        }
        let owner = self.MutableOwnerLayoutBox();
        debug_assert!(!owner.is_null());
        if !self.GetBreakToken().is_null() {
            debug_assert_ne!(
                self as *const Self,
                unsafe { &*owner }.PhysicalFragments().back() as *const _
            );
            return;
        }
        debug_assert_eq!(
            self as *const Self,
            unsafe { &*owner }.PhysicalFragments().back() as *const _
        );
        debug_assert_eq!(owner.cast::<LayoutObject>(), self.GetMutableLayoutObject());
        debug_assert!(unsafe { &*owner }.PhysicalFragments().Contains(self));
        unsafe { &mut *owner }.CopyVisualOverflowFromFragments();
    }

    // Source: core/layout/physical_box_fragment.cc:1040-1095.
    fn RecalcContentsInkOverflow(&self) -> PhysicalRect {
        debug_assert!(!self.GetSelfOrContainerLayoutObject().is_null());
        debug_assert!(
            !unsafe { &*self.GetSelfOrContainerLayoutObject() }.ChildPrePaintBlockedByDisplayLock()
        );
        let mut contents_rect = PhysicalRect::default();
        if !self.Items().is_null() {
            let mut cursor = InlineCursor::from_fragment(self);
            let mut child_inline_context =
                crate::inline_paint_context::InlinePaintContext::default();
            contents_rect = crate::fragment_item::FragmentItem::RecalcInkOverflowForCursor(
                &mut cursor,
                &mut child_inline_context,
            );
            let layout_object = self.GetLayoutObject();
            if !layout_object.is_null()
                && unsafe { &*layout_object }.RuntimeClass() == LayoutObjectClass::TextCombine
            {
                let mut text_combine_cursor = InlineCursor::from_fragment(self);
                let objects = unsafe { &*layout_object }.FactorySet();
                let recalc = unsafe { objects.as_ref() }
                    .and_then(|objects| objects.recalc_text_combine_contents_ink_overflow)
                    .expect("source text-combine ink-overflow dispatch is not installed");
                let combined = recalc(
                    unsafe { &*layout_object },
                    (&mut text_combine_cursor as *mut InlineCursor).cast(),
                );
                contents_rect.Unite(&combined);
            }
            if !self.HasFloatingDescendantsForPaint() {
                return contents_rect;
            }
        }

        for child in self.PostLayoutChildren().iter() {
            let child_fragment = DynamicTo::<PhysicalBoxFragment>(child.get());
            if child_fragment.is_null() || unsafe { &*child_fragment }.HasSelfPaintingLayer() {
                continue;
            }
            let child_fragment = unsafe { &*child_fragment };
            debug_assert!(!child_fragment.IsOutOfFlowPositioned());
            let mut child_rect;
            if child_fragment.CanUseFragmentsForInkOverflow() {
                child_fragment.GetMutableForPainting().RecalcInkOverflow();
                child_rect = child_fragment.InkOverflowRect();
            } else {
                let child_layout_object = child_fragment.MutableOwnerLayoutBox();
                debug_assert!(!child_layout_object.is_null());
                debug_assert!(!unsafe { &*child_layout_object }.CanUseFragmentsForVisualOverflow());
                unsafe { &mut *child_layout_object }.RecalcVisualOverflow();
                child_rect = unsafe { &*child_layout_object }.VisualOverflowRect();
            }
            child_rect.offset += child.offset;
            contents_rect.Unite(&child_rect);
        }
        contents_rect
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:390-393
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:641-659
    pub fn SelfInkOverflowRect(&self) -> PhysicalRect {
        if !self.CanUseFragmentsForInkOverflow() {
            let owner = DynamicTo::<LayoutBox>(self.GetLayoutObject());
            return unsafe { &*owner }.SelfVisualOverflowRect();
        }
        if !self.HasInkOverflow() {
            return self.LocalRect();
        }
        unsafe { &*self.ink_overflow_.get() }.SelfRect(self.InkOverflowType(), &self.Size())
    }

    pub fn ContentsInkOverflowRect(&self) -> PhysicalRect {
        if !self.CanUseFragmentsForInkOverflow() {
            let owner = DynamicTo::<LayoutBox>(self.GetLayoutObject());
            return unsafe { &*owner }.ContentsVisualOverflowRect();
        }
        if !self.HasInkOverflow() {
            return self.LocalRect();
        }
        unsafe { &*self.ink_overflow_.get() }.Contents(self.InkOverflowType(), &self.Size())
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:661-700
    pub fn InkOverflowRect(&self) -> PhysicalRect {
        if !self.CanUseFragmentsForInkOverflow() {
            let owner = DynamicTo::<LayoutBox>(self.GetLayoutObject());
            return unsafe { &*owner }.VisualOverflowRect();
        }
        if !self.HasInkOverflow() {
            return self.LocalRect();
        }
        let overflow = unsafe { &*self.ink_overflow_.get() };
        let self_rect = overflow.SelfRect(self.InkOverflowType(), &self.Size());
        if self.Style().HasMask() {
            return self_rect;
        }
        let axes = self.GetOverflowClipAxes();
        if axes == kNoOverflowClip {
            let mut result = self_rect;
            result.Unite(&overflow.Contents(self.InkOverflowType(), &self.Size()));
            return result;
        }
        if axes == kOverflowClipBothAxis {
            if self.ShouldApplyOverflowClipMargin() {
                let contents = overflow.Contents(self.InkOverflowType(), &self.Size());
                if !contents.IsEmpty() {
                    let mut result = self.LocalRect();
                    result.Expand(&self.OverflowClipMarginOutsets());
                    result.Intersect(&contents);
                    result.Unite(&self_rect);
                    return result;
                }
            }
            return self_rect;
        }
        let mut result = overflow.Contents(self.InkOverflowType(), &self.Size());
        result.Unite(&self_rect);
        ApplyOverflowClip(axes, &self_rect, &mut result);
        result
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:423
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:924-954
    pub fn OverflowClipMarginOutsets(&self) -> PhysicalBoxStrut {
        let margin = self
            .Style()
            .OverflowClipMargin()
            .as_ref()
            .expect("overflow clip margin required");
        debug_assert!(self.ShouldApplyOverflowClipMargin());
        debug_assert!(!self.IsScrollContainer());
        let mut outsets = PhysicalBoxStrut::default();
        match margin.GetReferenceBox() {
            ReferenceBox::kBorderBox => {}
            ReferenceBox::kPaddingBox => outsets -= self.Borders(),
            ReferenceBox::kContentBox => {
                outsets -= self.Borders();
                outsets -= self.Padding();
            }
        }
        outsets += PhysicalBoxStrut::with_value(margin.GetMargin());
        outsets.TruncateSides(&self.SidesToInclude());
        outsets
    }

    // Rust uses a distinct name for the overload that takes an incoming token.
    // cpp: layoutng_fragment_tree/physical_box_fragment.h:353-361
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:702-707
    pub fn OverflowClipRect(&self, behavior: OverlayScrollbarClipBehavior) -> PhysicalRect {
        let object = self.GetLayoutObject();
        debug_assert!(!object.is_null() && unsafe { &*object }.IsBox());
        unsafe { &*To::<LayoutBox>(object) }.OverflowClipRectWithBehavior(behavior)
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:709-770
    pub fn OverflowClipRectForFragment(
        &self,
        incoming: *const BlockBreakToken,
        behavior: OverlayScrollbarClipBehavior,
    ) -> PhysicalRect {
        let mut clip_rect = self.OverflowClipRect(behavior);
        if incoming.is_null() && self.GetBreakToken().is_null() {
            return clip_rect;
        }
        let writing_direction = self.Style().GetWritingDirection();
        let box_ = unsafe { &*To::<LayoutBox>(self.GetLayoutObject()) };
        let converter = WritingModeConverter::new(writing_direction, box_.StitchedSize());
        let mut stitched_offset = LogicalOffset::default();
        if !incoming.is_null() {
            stitched_offset.block_offset = unsafe { &*incoming }.ConsumedBlockSize();
        }
        let logical_rect = LogicalRect::new(
            stitched_offset,
            ToLogicalSize(self.Size(), writing_direction.GetWritingMode()),
        );
        let physical_rect = converter.ToPhysicalRect(logical_rect);
        if !unsafe { &*self.GetLayoutObject() }.IsPrintingForLayout() {
            let overflow_clip = box_.GetOverflowClipAxes();
            let mut overflow_rect = physical_rect;
            if overflow_clip != kOverflowClipBothAxis {
                ApplyVisibleOverflowToClipRect(overflow_clip, &mut overflow_rect);
            } else if box_.ShouldApplyOverflowClipMargin() {
                overflow_rect.Expand(&self.OverflowClipMarginOutsets());
            }
            clip_rect.Intersect(&overflow_rect);
        }
        clip_rect.offset -= physical_rect.offset;
        clip_rect
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:397-398
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:772-781
    pub fn MayIntersect(
        &self,
        result: &HitTestResult,
        location: &HitTestLocation,
        accumulated_offset: PhysicalOffset,
    ) -> bool {
        let box_ = DynamicTo::<LayoutBox>(self.GetLayoutObject());
        if !box_.is_null() {
            return unsafe { &*box_ }.MayIntersect(result, location, &accumulated_offset);
        }
        true
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:402-405
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:818-849
    pub fn InlineContainerFragmentIfOutlineOwner(&self) -> *const Self {
        debug_assert!(self.IsInlineBox());
        let layout_object = self.GetLayoutObject();
        debug_assert!(!layout_object.is_null());
        debug_assert!(unsafe { &*layout_object }.IsLayoutInline());
        let mut cursor = InlineCursor::default();
        cursor.MoveToLayoutObject(unsafe { &*layout_object });
        debug_assert!(cursor.IsNotNull());
        if cursor.Current().BoxFragment() == self as *const Self {
            return cursor.ContainerFragment() as *const _;
        }
        if !cursor.IsBlockFragmented() {
            return std::ptr::null();
        }
        let mut previous_index = cursor.ContainerFragmentIndex();
        loop {
            cursor.MoveToNextForSameLayoutObject();
            debug_assert!(cursor.IsNotNull());
            let index = cursor.ContainerFragmentIndex();
            if cursor.Current().BoxFragment() == self as *const Self {
                return if index != previous_index {
                    cursor.ContainerFragment() as *const _
                } else {
                    std::ptr::null()
                };
            }
            previous_index = index;
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:402-404
    pub fn IsOutlineOwner(&self) -> bool {
        !self.IsInlineBox() || !self.InlineContainerFragmentIfOutlineOwner().is_null()
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:363-366
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:783-803
    pub fn PixelSnappedOverscrollContentOffset(&self) -> gfx::Vector2d {
        debug_assert!(!self.GetLayoutObject().is_null());
        if !self.IsNonOverlayOverscrollScrollContainer() {
            return gfx::Vector2d::default();
        }
        // Source deliberately fails here: pseudo-box traversal is unsupported.
        assert!(!self.IsNonOverlayOverscrollScrollContainer());
        gfx::Vector2d::default()
    }

    pub fn PixelSnappedScrolledContentOffset(&self) -> gfx::Vector2d {
        debug_assert!(!self.GetLayoutObject().is_null());
        if self.IsScrollContainer() {
            unsafe { &*To::<LayoutBox>(self.GetLayoutObject()) }.PixelSnappedScrolledContentOffset()
        } else {
            gfx::Vector2d::default()
        }
    }

    pub fn ScrollSize(&self) -> foundation::PhysicalSize {
        debug_assert!(!self.GetLayoutObject().is_null());
        let box_ = unsafe { &*To::<LayoutBox>(self.GetLayoutObject()) };
        foundation::PhysicalSize::new(box_.ScrollWidth(), box_.ScrollHeight())
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:92
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1962-1970
    pub fn TraceAfterDispatch(&self, visitor: &mut Visitor) {
        visitor.Trace(unsafe { &*self.children_.get() });
        visitor.Trace(unsafe { &*self.rare_data_.get() });
        if self.HasItems() {
            visitor.Trace(unsafe { &*self.Items() });
        }
        self.deref().TraceAfterDispatch(visitor);
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:343-345
    pub fn IsInlineFormattingContext(&self) -> bool {
        self.flag(Self::INLINE_FORMATTING_CONTEXT_BIT)
    }
}

// Drop the placement-constructed trailing FragmentItems explicitly.
// cpp: layoutng_fragment_tree/physical_box_fragment.h:76
// cpp: layoutng_fragment_tree/physical_box_fragment.cc:483-488
impl Drop for PhysicalBoxFragment {
    fn drop(&mut self) {
        if self.HasInkOverflow() {
            let overflow_type = self.InkOverflowType();
            let new_type = self.ink_overflow_.get_mut().Reset(overflow_type);
            self.SetInkOverflowType(new_type);
        }
        if self.HasItems() {
            unsafe { std::ptr::drop_in_place(self.ComputeItemsAddress()) };
        }
    }
}

#[cfg(feature = "translation_in_progress")]
#[path = "../../layout_algo_inline/src/physical_box_editing_queries.rs"]
mod layout_algo_inline_editing_queries;
