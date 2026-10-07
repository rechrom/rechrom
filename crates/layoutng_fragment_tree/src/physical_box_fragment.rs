use std::cell::UnsafeCell;
use std::ops::Deref;
use std::sync::atomic::{AtomicU32, Ordering};

unsafe extern "Rust" {
    fn PhysicalBoxFragmentComputeSelfInkOverflow(fragment: &PhysicalBoxFragment) -> PhysicalRect;
}
use foundation::graphics_types;

use foundation::{
    gfx, AtomicString, DynamicTo, EVisibility, GCedHeapVector, HeapVector, LayoutUnit, Length,
    MakeGarbageCollected, MakeGarbageCollectedWithAdditionalBytes, Member, PhysicalOffset,
    PhysicalRect, RuntimeEnabledFeatures, To, ValueForLength, Visitor, WritingMode,
};
use graphics_types::graphics::overlay_scrollbar_clip_behavior::OverlayScrollbarClipBehavior;
use layoutng::internal::block_node::BlockNode;
use layoutng::internal::disable_layout_side_effects_scope::DisableLayoutSideEffectsScope;
use layoutng::internal::frame_set_layout_data::FrameSetLayoutData;
use layoutng::internal::gap::gap_geometry::GapGeometry;
use layoutng::internal::ink_overflow::{InkOverflow, InkOverflowType};
use layoutng::internal::layout_block_flow::LayoutBlockFlow;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_box_core_services::ApplyVisibleOverflowToClipRect;
use layoutng::internal::layout_node_metadata::ContentLockBlocksChildLayout;
use layoutng::internal::layout_node_metadata::{ContainerNode, Element, Node};
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_object::{HitTestLocation, HitTestResult};
use layoutng::internal::layout_view::LayoutView;
use layoutng::internal::mathml_paint_info::MathMLPaintInfo;
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
use layoutng_style::style::style_overflow_clip_margin::ReferenceBox;

use crate::block_break_token::BlockBreakToken;
use crate::box_fragment_builder::BoxFragmentBuilder;
use crate::fragment_items::FragmentItems;
use crate::inline_cursor::InlineCursor;
use crate::physical_fragment::{
    FragmentType, PhysicalFragment, PhysicalFragmentFlags, PostLayoutChildLinkList,
};
use crate::physical_fragment_link::PhysicalFragmentLink;
use crate::physical_fragment_rare_data::{FieldId, PhysicalFragmentRareData, RareField};

// FragmentItems occupies aligned trailing bytes, outside the fixed-size box
// fragment. The base stays first for the source's inherited layout contract.
// cpp: layoutng_fragment_tree/physical_box_fragment.h:40-75
// cpp: layoutng_fragment_tree/physical_box_fragment.h:627-739
// cpp: layoutng_fragment_tree/physical_box_fragment.cc:83-92
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
// cpp: layoutng_fragment_tree/physical_box_fragment.cc:53
// cpp: layoutng_fragment_tree/physical_box_fragment.cc:956-963
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
        ALLOW_POST_LAYOUT_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
}

// The source restricts these mutations to the enclosing fragment's factory.
// cpp: layoutng_fragment_tree/physical_box_fragment.h:485-511
// cpp: layoutng_fragment_tree/physical_box_fragment.cc:846-849
pub struct MutableForStyleRecalc<'a> {
    fragment_: &'a PhysicalBoxFragment,
}

// cpp: layoutng_fragment_tree/physical_box_fragment.cc:871-874
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
    // cpp: layoutng_out_of_flow/out_of_flow_fragment_builder.cc:202-215
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

    // cpp: layoutng_out_of_flow/out_of_flow_fragment_builder.cc:217-247
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

    // cpp: layoutng_out_of_flow/out_of_flow_fragment_builder.cc:249-258
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:851-862
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:876-881
    pub fn SetMargins(&self, margins: PhysicalBoxStrut) {
        let field = self.fragment_.EnsureRareField(FieldId::kMargins);
        unsafe {
            *field.value_.margins = margins;
        }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:882-892
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

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:899-911
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
    pub fn RecalcInkOverflowWithContents(&self, contents: PhysicalRect) {
        self.fragment_.RecalcInkOverflowWithContents(contents);
    }
    // The no-argument recalculation and debug invalidation bodies are absent
    // from the supplied C++ tree; their declarations remain in status.
}

impl Deref for PhysicalBoxFragment {
    type Target = PhysicalFragment;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.base_.get() }
    }
}

// Source-local helpers used by point positioning and gap-decoration overflow.
// Their callers have missing source definitions; the supplied behavior stays
// available for the later cross-package connection.
// cpp: layoutng_fragment_tree/physical_box_fragment.cc:94-120
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

// cpp: layoutng_fragment_tree/physical_box_fragment.cc:122-141
fn ShouldUsePositionForPointInBlockFlowDirection(layout_object: &LayoutObject) -> bool {
    let block_flow = DynamicTo::<LayoutBlockFlow>(layout_object as *const LayoutObject);
    if block_flow.is_null() && !IsFlexibleBoxWithSingleChildElement(layout_object) {
        return false;
    }
    !layout_object.StyleRef().SpecifiesColumns()
}

// cpp: layoutng_fragment_tree/physical_box_fragment.cc:143-147
fn IsHitTestCandidate(fragment: &PhysicalBoxFragment) -> bool {
    fragment.Size().height != LayoutUnit::default()
        && fragment.Style().Visibility() == EVisibility::kVisible
        && !fragment.IsFloatingOrOutOfFlowPositioned()
}

// cpp: layoutng_fragment_tree/physical_box_fragment.cc:165-187
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

// cpp: layoutng_fragment_tree/physical_box_fragment.cc:189-221
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

// cpp: layoutng_fragment_tree/physical_box_fragment.cc:149-163
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.h:685
    // The declaration has no definition in the supplied C++ checkout.
    fn ComputeSelfInkOverflow(&self) -> PhysicalRect {
        unsafe { PhysicalBoxFragmentComputeSelfInkOverflow(self) }
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:372-388
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:496-516
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:42-51
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:226-306
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:372-388
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:309-370
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:390-495
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
        debug_assert!(!base.GetLayoutObject().is_null());
        debug_assert!(unsafe { &*base.GetLayoutObject() }.IsBoxModelObject());
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

        let allow_baseline = !unsafe { &*base.GetLayoutObject() }.ShouldApplyLayoutContainment()
            || unsafe { &*base.GetLayoutObject() }.IsTableCell();
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1049-1096
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
        let object = self.GetLayoutObject();
        debug_assert!(!object.is_null());
        if ContentLockBlocksChildLayout(unsafe { &*object }.GetNode()) {
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:965-1046
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1098-1102
    #[cfg(debug_assertions)]
    pub fn AssertFragmentTreeSelf(&self) {
        debug_assert!(!self.IsInlineBox());
        debug_assert!(!self.OwnerLayoutBox().is_null());
        debug_assert_eq!(self as *const Self, self.PostLayout());
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1104-1131
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:625-634
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:838-844
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

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:525-531
    pub fn ContentRect(&self) -> PhysicalRect {
        let mut rect = PhysicalRect::new(PhysicalOffset::default(), self.Size());
        rect.Contract(&(self.Borders() + self.Padding()));
        debug_assert!(rect.size.width >= LayoutUnit::default());
        debug_assert!(rect.size.height >= LayoutUnit::default());
        rect
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:347-350
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:533-561
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:864-869
    pub fn GetMutableForStyleRecalc(&self) -> MutableForStyleRecalc<'_> {
        debug_assert!(!self.GetLayoutObject().is_null());
        MutableForStyleRecalc { fragment_: self }
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:511
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:892-897
    pub fn GetMutableForContainerLayout(&self) -> MutableForContainerLayout<'_> {
        debug_assert!(!self.GetLayoutObject().is_null());
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
        self.IsFragmentationContextRoot() && unsafe { &*self.GetLayoutObject() }.IsLayoutView()
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:563-623
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
        !unsafe { &*self.GetLayoutObject() }.IsLayoutReplaced()
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:913-917
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

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:616-618
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:919-922
    fn RecalcInkOverflowWithContents(&self, contents: PhysicalRect) {
        // ComputeSelfInkOverflow is declared in the header, but its source
        // definition is absent. Keep the call explicit for later connection.
        let self_rect = self.ComputeSelfInkOverflow();
        self.SetInkOverflow(self_rect, contents);
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.h:390-393
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:636-654
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

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:656-695
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:697-702
    pub fn OverflowClipRect(&self, behavior: OverlayScrollbarClipBehavior) -> PhysicalRect {
        let object = self.GetLayoutObject();
        debug_assert!(!object.is_null() && unsafe { &*object }.IsBox());
        unsafe { &*To::<LayoutBox>(object) }.OverflowClipRectWithBehavior(behavior)
    }

    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:704-765
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:767-776
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:810-836
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:778-808
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
    // cpp: layoutng_fragment_tree/physical_box_fragment.cc:1135-1143
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
// cpp: layoutng_fragment_tree/physical_box_fragment.cc:518-523
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
