use std::cell::{Cell, UnsafeCell};
use std::fmt;

use foundation::{
    DynamicTo, GCedHeapVector, HeapVector, IsA, LayoutUnit, MakeGarbageCollected, Member,
    PhysicalOffset, PhysicalRect, PhysicalSize, String, StringBuilder, TextDirection,
    ThreadAffinity, ThreadingTrait, To, Visitor, WritingMode,
};
use layoutng::internal::anchor_map::AnchorMap;
use layoutng::internal::block_node::BlockNode;
use layoutng::internal::form_node_metadata::TextControlElement;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_box_model_object::LayoutBoxModelObject;
use layoutng::internal::layout_box_model_object::PaintLayer;
use layoutng::internal::layout_node_metadata::{Element, Node};
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::oof_positioned_node::FragmentedOofData;
use layoutng::internal::oof_positioned_node::LogicalOofPositionedNodeToPhysical;
use layoutng::internal::oof_positioned_node::PhysicalOofPositionedNode;
use layoutng::internal::oof_positioned_node::{
    MulticolWithPendingOofs, OofContainingBlock, OofInlineContainer,
    PhysicalOofNodeForFragmentation, RelativeInsetToPhysical,
};
use layoutng::internal::snap_area::SnapArea;
use layoutng::internal::split_axis_item::SplitAxisItem;
use layoutng::internal::style_variant::{StyleVariant, UsesFirstLineStyle};
use layoutng::internal::trigger_scoped_name::TriggerScopedNameMap;

use crate::break_token::BreakToken;
use crate::fragment_builder::FragmentBuilder;
use crate::fragment_data::FragmentData;
use crate::inline_cursor::InlineCursor;
use crate::physical_box_fragment::PhysicalBoxFragment;
use crate::physical_fragment_link::PhysicalFragmentLink;
use crate::physical_line_box_fragment::PhysicalLineBoxFragment;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::logical_size::ToPhysicalSize;
use layoutng_geometry::geometry::physical_size_string::PhysicalSizeStringExt;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;

// physical_offset.h declares ToString(), but its definition is not present
// in the supplied checkout. Keep the missing owner visible at link time.
trait PhysicalOffsetStringExt {
    fn ToString(&self) -> String;
}

impl PhysicalOffsetStringExt for PhysicalOffset {
    fn ToString(&self) -> String {
        unsafe { PhysicalOffsetToString(self) }
    }
}

unsafe extern "Rust" {
    fn PhysicalOffsetToString(offset: &PhysicalOffset) -> String;
}

// cpp: layoutng_fragment_tree/physical_fragment.h:66-71
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FragmentType {
    kFragmentBox = 0,
    kFragmentLineBox = 1,
}

// cpp: layoutng_fragment_tree/physical_fragment.h:72-119
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BoxType {
    kNormalBox = 0,
    kInlineBox = 1,
    kColumnBox = 2,
    kPageContainer = 3,
    kPageBorderBox = 4,
    kPageMargin = 5,
    kPageArea = 6,
    kAtomicInline = 7,
    kFloating = 8,
    kOutOfFlowPositioned = 9,
    kBlockFlowRoot = 10,
    kRenderedLegend = 11,
}

#[allow(non_upper_case_globals)]
impl BoxType {
    pub const kMinimumFormattingContextRoot: Self = Self::kAtomicInline;
}

// Four separate bytes mirror the source's zero-length bitfield separator.
// A Cell permits only the intentional late flag updates through const views.
// cpp: layoutng_fragment_tree/physical_fragment.h:678-718
#[repr(C)]
pub(crate) struct PhysicalFragmentFlags([Cell<u8>; 4]);

impl Clone for PhysicalFragmentFlags {
    fn clone(&self) -> Self {
        Self(std::array::from_fn(|index| Cell::new(self.0[index].get())))
    }
}

impl Default for PhysicalFragmentFlags {
    fn default() -> Self {
        Self(std::array::from_fn(|_| Cell::new(0)))
    }
}

impl PhysicalFragmentFlags {
    pub(crate) const TYPE: (usize, u8) = (0, 0);
    pub(crate) const SUB_TYPE: (usize, u8) = (0, 1);
    pub(crate) const STYLE_VARIANT: (usize, u8) = (0, 5);
    pub(crate) const IS_HIDDEN_FOR_PAINT: (usize, u8) = (0, 7);
    pub(crate) const HAS_FLOATING_DESCENDANTS_FOR_PAINT: (usize, u8) = (1, 0);
    pub(crate) const HAS_ADJOINING_OBJECT_DESCENDANTS: (usize, u8) = (1, 1);
    pub(crate) const DEPENDS_ON_PERCENTAGE_BLOCK_SIZE: (usize, u8) = (1, 2);
    pub(crate) const HAS_RUNNING_ANCHOR_TRANSFORM_ANIMATION: (usize, u8) = (1, 3);
    pub(crate) const CHILDREN_VALID: (usize, u8) = (1, 4);
    pub(crate) const HAS_PROPAGATED_DESCENDANTS: (usize, u8) = (1, 5);
    pub(crate) const HAS_HANGING: (usize, u8) = (1, 6);
    pub(crate) const IS_OPAQUE: (usize, u8) = (1, 7);
    pub(crate) const IS_BLOCK_IN_INLINE: (usize, u8) = (2, 0);
    pub(crate) const IS_LINE_FOR_PARALLEL_FLOW: (usize, u8) = (2, 1);
    pub(crate) const IS_MATH_FRACTION: (usize, u8) = (2, 2);
    pub(crate) const IS_MATH_OPERATOR: (usize, u8) = (2, 3);
    pub(crate) const MAY_HAVE_DESCENDANT_ABOVE_BLOCK_START: (usize, u8) = (2, 4);
    pub(crate) const IS_FIELDSET_CONTAINER: (usize, u8) = (2, 5);
    pub(crate) const IS_TABLE_PART: (usize, u8) = (2, 6);
    pub(crate) const IS_PAINTED_ATOMICALLY: (usize, u8) = (2, 7);
    pub(crate) const HAS_COLLAPSED_BORDERS: (usize, u8) = (3, 0);
    pub(crate) const HAS_FIRST_BASELINE: (usize, u8) = (3, 1);
    pub(crate) const HAS_LAST_BASELINE: (usize, u8) = (3, 2);
    pub(crate) const USE_LAST_BASELINE_FOR_INLINE_BASELINE: (usize, u8) = (3, 3);
    pub(crate) const HAS_FRAGMENTED_OUT_OF_FLOW_DATA: (usize, u8) = (3, 4);
    pub(crate) const HAS_OUT_OF_FLOW_FRAGMENT_CHILD: (usize, u8) = (3, 5);
    pub(crate) const HAS_OUT_OF_FLOW_IN_FRAGMENTAINER_SUBTREE: (usize, u8) = (3, 6);
    pub(crate) const BASE_DIRECTION: (usize, u8) = (3, 7);

    pub(crate) fn get(&self, field: (usize, u8)) -> bool {
        self.0[field.0].get() & (1 << field.1) != 0
    }

    pub(crate) fn set(&self, field: (usize, u8), value: bool) {
        let byte = self.0[field.0].get();
        self.0[field.0].set((byte & !(1 << field.1)) | ((value as u8) << field.1));
    }

    pub(crate) fn get_bits(&self, index: usize, offset: u8, width: u8) -> u8 {
        (self.0[index].get() >> offset) & ((1 << width) - 1)
    }

    pub(crate) fn set_bits(&self, index: usize, offset: u8, width: u8, value: u8) {
        let mask = ((1 << width) - 1) << offset;
        let byte = self.0[index].get();
        self.0[index].set((byte & !mask) | ((value << offset) & mask));
    }
}

const _: () = assert!(std::mem::size_of::<PhysicalFragmentFlags>() == 4);

// cpp: layoutng_fragment_tree/physical_fragment.h:121-138
pub struct PropagatedData {
    pub sticky_descendants: Member<GCedHeapVector<SplitAxisItem<LayoutBoxModelObject>>>,
    pub snap_areas: Member<GCedHeapVector<SnapArea>>,
    pub scroll_initial_target: Member<LayoutObject>,
    pub named_triggers: Member<TriggerScopedNameMap>,
}

#[allow(non_snake_case)]
impl PropagatedData {
    pub fn new(
        sticky_descendants: *const GCedHeapVector<SplitAxisItem<LayoutBoxModelObject>>,
        snap_areas: *const GCedHeapVector<SnapArea>,
        scroll_initial_target: Member<LayoutObject>,
        named_triggers: *const TriggerScopedNameMap,
    ) -> Self {
        Self {
            sticky_descendants: Member::from_ptr(sticky_descendants as *mut _),
            snap_areas: Member::from_ptr(snap_areas as *mut _),
            scroll_initial_target,
            named_triggers: Member::from_ptr(named_triggers as *mut _),
        }
    }

    // cpp: layoutng_fragment_tree/physical_fragment.cc:797-802
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.sticky_descendants);
        visitor.Trace(&self.snap_areas);
        visitor.Trace(&self.scroll_initial_target);
        visitor.Trace(&self.named_triggers);
    }
}

// cpp: layoutng_fragment_tree/physical_fragment.h:608-623
pub struct OofData {
    oof_positioned_descendants_: HeapVector<PhysicalOofPositionedNode>,
    anchor_map_: Member<AnchorMap>,
}

impl Default for OofData {
    fn default() -> Self {
        Self {
            oof_positioned_descendants_: HeapVector::default(),
            anchor_map_: Member::default(),
        }
    }
}

impl Clone for OofData {
    fn clone(&self) -> Self {
        Self {
            oof_positioned_descendants_: self.oof_positioned_descendants_.clone(),
            anchor_map_: self.anchor_map_.clone(),
        }
    }
}

#[allow(non_snake_case)]
impl OofData {
    pub fn OofPositionedDescendants(&self) -> &HeapVector<PhysicalOofPositionedNode> {
        &self.oof_positioned_descendants_
    }

    pub fn OofPositionedDescendantsMut(&mut self) -> &mut HeapVector<PhysicalOofPositionedNode> {
        &mut self.oof_positioned_descendants_
    }

    pub fn SetAnchorMap(&mut self, anchor_map: *mut AnchorMap) {
        self.anchor_map_ = Member::from_ptr(anchor_map);
    }

    pub fn GetAnchorMap(&self) -> *const AnchorMap {
        self.anchor_map_.Get()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:617
    // cpp: layoutng_out_of_flow/out_of_flow_fragment_builder.cc:260-265
    pub fn EnsureAnchorMap(&mut self) -> &mut AnchorMap {
        if self.anchor_map_.Get().is_null() {
            self.anchor_map_ = Member::from_ptr(MakeGarbageCollected(AnchorMap::default()));
        }
        unsafe { &mut *self.anchor_map_.Get() }
    }

    // cpp: layoutng_fragment_tree/physical_fragment.cc:792-795
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.oof_positioned_descendants_);
        visitor.Trace(&self.anchor_map_);
    }
}

// cpp: layoutng_fragment_tree/physical_fragment.h:64-65
// cpp: layoutng_fragment_tree/physical_fragment.h:675-723
#[repr(C)]
pub struct PhysicalFragment {
    pub(crate) layout_object_: UnsafeCell<Member<LayoutObject>>,
    pub(crate) size_: PhysicalSize,
    pub(crate) flags_: PhysicalFragmentFlags,
    pub(crate) propagated_data_: Member<PropagatedData>,
    pub(crate) break_token_: Member<BreakToken>,
    pub(crate) oof_data_: Member<OofData>,
}

// cpp: layoutng_fragment_tree/physical_fragment.cc:307-330
#[allow(non_snake_case)]
fn PhysicalContainingBlockWithSize(
    builder: &FragmentBuilder,
    outer_size: PhysicalSize,
    inner_size: PhysicalSize,
    containing_block: &OofContainingBlock<LogicalOffset>,
) -> OofContainingBlock<PhysicalOffset> {
    OofContainingBlock::new(
        containing_block.Offset().ConvertToPhysical(
            builder.Style().GetWritingDirection(),
            outer_size,
            inner_size,
        ),
        RelativeInsetToPhysical(
            containing_block.RelativeOffset(),
            builder.Style().GetWritingDirection(),
        ),
        containing_block.Fragment(),
        containing_block.ClippedContainerBlockOffset(),
        containing_block.IsInsideColumnSpanner(),
    )
}

#[allow(non_snake_case)]
fn PhysicalContainingBlock(
    builder: &FragmentBuilder,
    size: PhysicalSize,
    containing_block: &OofContainingBlock<LogicalOffset>,
) -> OofContainingBlock<PhysicalOffset> {
    let fragment = containing_block.Fragment();
    let containing_block_size = if fragment.is_null() {
        size
    } else {
        unsafe { &*fragment }.Size()
    };
    PhysicalContainingBlockWithSize(builder, size, containing_block_size, containing_block)
}

// cpp: layoutng_fragment_tree/physical_fragment.cc:52-109
#[allow(non_snake_case)]
fn StringForBoxType(fragment: &PhysicalFragment) -> String {
    let mut result = StringBuilder::default();
    let kind = match fragment.GetBoxType() {
        BoxType::kNormalBox => "",
        BoxType::kInlineBox => "inline",
        BoxType::kColumnBox => "column",
        BoxType::kPageContainer => "page container",
        BoxType::kPageBorderBox => "page border box",
        BoxType::kPageMargin => "page margin",
        BoxType::kPageArea => "page area",
        BoxType::kAtomicInline => "atomic-inline",
        BoxType::kFloating => "floating",
        BoxType::kOutOfFlowPositioned => "out-of-flow-positioned",
        BoxType::kBlockFlowRoot => "block-flow-root",
        BoxType::kRenderedLegend => "rendered-legend",
    };
    result.Append(kind);
    if fragment.IsBlockFlow() {
        if !kind.is_empty() {
            result.Append(" ");
        }
        result.Append("block-flow");
    }
    if fragment.IsFieldsetContainer() {
        if !kind.is_empty() || fragment.IsBlockFlow() {
            result.Append(" ");
        }
        result.Append("fieldset-container");
    }
    if fragment.IsBox()
        && unsafe { &*To::<PhysicalBoxFragment>(fragment as *const PhysicalFragment) }
            .IsInlineFormattingContext()
    {
        if !kind.is_empty() || fragment.IsBlockFlow() || fragment.IsFieldsetContainer() {
            result.Append(" ");
        }
        result.Append("children-inline");
    }
    result.ToString()
}

// cpp: layoutng_fragment_tree/physical_fragment.h:424-440
pub type DumpFlags = i32;

#[allow(non_upper_case_globals)]
impl PhysicalFragment {
    pub const DumpHeaderText: DumpFlags = 0x1;
    pub const DumpSubtree: DumpFlags = 0x2;
    pub const DumpIndentation: DumpFlags = 0x4;
    pub const DumpType: DumpFlags = 0x8;
    pub const DumpOffset: DumpFlags = 0x10;
    pub const DumpSize: DumpFlags = 0x20;
    pub const DumpTextOffsets: DumpFlags = 0x40;
    pub const DumpSelfPainting: DumpFlags = 0x80;
    pub const DumpNodeName: DumpFlags = 0x100;
    pub const DumpItems: DumpFlags = 0x200;
    pub const DumpLegacyDescendants: DumpFlags = 0x400;
    pub const DumpBreakInfo: DumpFlags = 0x800;
    pub const DumpAll: DumpFlags = -1;
}

// cpp: layoutng_fragment_tree/physical_fragment.cc:111-305
struct FragmentTreeDumper<'a> {
    builder_: &'a mut StringBuilder,
    target_fragment_: *const PhysicalFragment,
    flags_: DumpFlags,
    target_fragment_found_: bool,
}

#[allow(non_snake_case)]
impl FragmentTreeDumper<'_> {
    fn has(&self, flag: DumpFlags) -> bool {
        self.flags_ & flag != 0
    }

    fn Append(
        &mut self,
        fragment: &PhysicalFragment,
        offset: Option<PhysicalOffset>,
        attributes: &mut Vec<String>,
        indent: usize,
    ) {
        self.AppendIndentation(indent, fragment as *const _);
        let mut has_content = false;
        if fragment.IsBox() {
            let box_fragment = unsafe { &*(fragment as *const _ as *const PhysicalBoxFragment) };
            if box_fragment.IsLayoutObjectDestroyedOrMoved() {
                self.builder_.Append("DEAD LAYOUT OBJECT!\n");
                return;
            }
            let layout_object = box_fragment.GetLayoutObject();
            if self.has(PhysicalFragment::DumpType) {
                self.builder_.Append("Box");
                let box_type = StringForBoxType(fragment);
                has_content = true;
                if !box_type.is_empty() {
                    attributes.push(box_type);
                }
                if self.has(PhysicalFragment::DumpSelfPainting)
                    && box_fragment.HasSelfPaintingLayer()
                {
                    attributes.push(String::from("self paint"));
                }
                if self.has(PhysicalFragment::DumpBreakInfo) && !box_fragment.IsFirstForNode() {
                    attributes.push(String::from("resumed"));
                }
            }
            self.AppendAttributes(attributes);
            has_content = self.AppendOffsetAndSize(fragment, offset, has_content);
            if self.has(PhysicalFragment::DumpNodeName) && !layout_object.is_null() {
                if has_content {
                    self.builder_.Append(" ");
                }
                self.builder_
                    .Append(&unsafe { &*layout_object }.DebugName());
            }
            if self.has(PhysicalFragment::DumpBreakInfo) {
                let token = box_fragment.GetBreakToken();
                if !token.is_null() {
                    self.builder_
                        .Append(&unsafe { &*token }.ToStringWithSkipNodeInfo(true));
                }
            }
            self.builder_.Append("\n");

            let mut has_fragment_items = false;
            if self.has(PhysicalFragment::DumpItems) {
                let items = box_fragment.Items();
                if !items.is_null() {
                    let mut cursor =
                        InlineCursor::from_container_and_items(box_fragment, unsafe { &*items });
                    self.AppendCursor(&mut cursor, indent + 2);
                    has_fragment_items = true;
                }
            }
            if self.has(PhysicalFragment::DumpSubtree) {
                for child in box_fragment.Children() {
                    if has_fragment_items && child.IsLineBox() {
                        continue;
                    }
                    self.Append(&child, Some(child.Offset()), &mut Vec::new(), indent + 2);
                }
            }
            return;
        }

        if fragment.IsLineBox() {
            if self.has(PhysicalFragment::DumpType) {
                self.builder_.Append("LineBox");
                has_content = true;
            }
        } else if self.has(PhysicalFragment::DumpType) {
            self.builder_.Append("Unknown fragment type");
            has_content = true;
        }
        self.AppendOffsetAndSize(fragment, offset, has_content);
        self.builder_.Append("\n");
    }

    fn AppendAttributes(&mut self, attributes: &[String]) {
        for attribute in attributes {
            self.builder_.Append(" (");
            self.builder_.Append(attribute);
            self.builder_.Append(")");
        }
    }

    fn AppendCursor(&mut self, cursor: &mut InlineCursor, indent: usize) {
        while cursor.IsNotNull() {
            let current = cursor.Current();
            let box_fragment = current.BoxFragment();
            if !box_fragment.is_null() && !unsafe { &*box_fragment }.IsInlineBox() {
                let mut attributes = Vec::new();
                if unsafe { &*current.Item() }.IsHiddenForPaint() {
                    attributes.push(String::from("hidden"));
                }
                self.Append(
                    unsafe { &*box_fragment },
                    Some(current.OffsetInContainerFragment()),
                    &mut attributes,
                    indent,
                );
                cursor.MoveToNextSkippingChildren();
                continue;
            }
            let displayed: *const PhysicalFragment = if box_fragment.is_null() {
                unsafe { &*current.Item() }.LineBoxFragment() as *const PhysicalFragment
            } else {
                box_fragment as *const PhysicalFragment
            };
            self.AppendIndentation(indent, displayed);
            if unsafe { &*current.Item() }.IsLayoutObjectDestroyedOrMoved() {
                self.builder_.Append("DEAD LAYOUT OBJECT!\n");
                return;
            }
            self.builder_
                .Append(&unsafe { &*current.Item() }.ToString());
            if self.has(PhysicalFragment::DumpOffset) {
                self.builder_.Append(" offset:");
                self.builder_
                    .Append(&current.OffsetInContainerFragment().ToString());
            }
            if self.has(PhysicalFragment::DumpSize) {
                self.builder_.Append(" size:");
                self.builder_.Append(&current.Size().ToString());
            }
            self.builder_.Append("\n");
            if self.has(PhysicalFragment::DumpSubtree) && current.HasChildren() {
                let mut descendants = cursor.CursorForDescendants();
                self.AppendCursor(&mut descendants, indent + 2);
            }
            cursor.MoveToNextSkippingChildren();
        }
    }

    fn AppendOffsetAndSize(
        &mut self,
        fragment: &PhysicalFragment,
        offset: Option<PhysicalOffset>,
        mut has_content: bool,
    ) -> bool {
        if self.has(PhysicalFragment::DumpOffset) {
            if has_content {
                self.builder_.Append(" ");
            }
            self.builder_.Append("offset:");
            if let Some(offset) = offset {
                self.builder_.Append(&offset.ToString());
            } else {
                self.builder_.Append("unplaced");
            }
            has_content = true;
        }
        if self.has(PhysicalFragment::DumpSize) {
            if has_content {
                self.builder_.Append(" ");
            }
            self.builder_.Append("size:");
            self.builder_.Append(&fragment.Size().ToString());
            has_content = true;
        }
        has_content
    }

    fn AppendIndentation(&mut self, indent: usize, fragment: *const PhysicalFragment) {
        if self.has(PhysicalFragment::DumpIndentation) {
            let mut start = 0;
            if !fragment.is_null() && fragment == self.target_fragment_ {
                self.builder_.Append("*");
                self.target_fragment_found_ = true;
                start = 1;
            }
            for _ in start..indent {
                self.builder_.Append(" ");
            }
        }
    }
}

// C++ stores a span and returns a temporary link after resolving its latest
// fragment generation. Rust yields that link by value to keep its lifetime
// tied to the iterator step.
// cpp: layoutng_fragment_tree/physical_fragment.h:467-481
// cpp: layoutng_fragment_tree/physical_fragment.h:530-543
pub struct PostLayoutChildLinkList<'a> {
    buffer_: &'a [PhysicalFragmentLink],
}

impl<'a> PostLayoutChildLinkList<'a> {
    pub fn new(buffer: &'a [PhysicalFragmentLink]) -> Self {
        Self { buffer_: buffer }
    }

    pub fn iter(&self) -> PostLayoutChildLinkIterator<'a> {
        PostLayoutChildLinkIterator {
            current_: 0,
            buffer_: self.buffer_,
        }
    }

    pub fn size(&self) -> usize {
        self.buffer_.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer_.is_empty()
    }
}

// cpp: layoutng_fragment_tree/physical_fragment.h:483-529
pub struct PostLayoutChildLinkIterator<'a> {
    current_: usize,
    buffer_: &'a [PhysicalFragmentLink],
}

impl Iterator for PostLayoutChildLinkIterator<'_> {
    type Item = PhysicalFragmentLink;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_ < self.buffer_.len() {
            let current = &self.buffer_[self.current_];
            self.current_ += 1;
            let fragment = current.fragment.Get();
            if unsafe { &*fragment }.IsLayoutObjectDestroyedOrMoved() {
                continue;
            }
            let post_layout = unsafe { &*fragment }.PostLayout();
            if !post_layout.is_null() {
                return Some(PhysicalFragmentLink {
                    fragment: Member::from_ptr(post_layout as *mut _),
                    offset: current.offset,
                });
            }
        }
        None
    }
}

// cpp: layoutng_fragment_tree/physical_fragment.cc:40-50
#[repr(C)]
struct SameSizeAsPhysicalFragment {
    layout_object: Member<LayoutObject>,
    size: PhysicalSize,
    flags: [u8; 4],
    members: [Member<PhysicalFragment>; 3],
}

const _: () = assert!(
    std::mem::size_of::<PhysicalFragment>() == std::mem::size_of::<SameSizeAsPhysicalFragment>()
);

impl ThreadingTrait for PhysicalFragment {
    // cpp: layoutng_fragment_tree/physical_fragment.h:725-728
    const kAffinity: ThreadAffinity = ThreadAffinity::kMainThreadOnly;
}

impl Clone for PhysicalFragment {
    // cpp: layoutng_fragment_tree/physical_fragment.h:145
    // cpp: layoutng_fragment_tree/physical_fragment.cc:398-440
    fn clone(&self) -> Self {
        let oof_data = if self.oof_data_.Get().is_null() {
            Member::default()
        } else {
            Member::from_ptr(self.CloneOofData())
        };
        let copy = Self {
            layout_object_: UnsafeCell::new(unsafe { &*self.layout_object_.get() }.clone()),
            size_: self.size_,
            flags_: self.flags_.clone(),
            propagated_data_: self.propagated_data_.clone(),
            break_token_: self.break_token_.clone(),
            oof_data_: oof_data,
        };
        assert!(!copy.layout_object_ptr().is_null());
        debug_assert!(self.flags_.get(PhysicalFragmentFlags::CHILDREN_VALID));
        debug_assert!(copy.flags_.get(PhysicalFragmentFlags::CHILDREN_VALID));
        copy
    }
}

#[allow(non_snake_case)]
impl PhysicalFragment {
    // cpp: layoutng_fragment_tree/physical_fragment.h:140-143
    // cpp: layoutng_fragment_tree/physical_fragment.cc:334-396
    pub fn from_builder(
        builder: &mut FragmentBuilder,
        _block_or_line_writing_mode: WritingMode,
        fragment_type: FragmentType,
        sub_type: u32,
    ) -> Self {
        let flags = PhysicalFragmentFlags::default();
        flags.set(
            PhysicalFragmentFlags::TYPE,
            fragment_type == FragmentType::kFragmentLineBox,
        );
        flags.set_bits(0, 1, 4, sub_type as u8);
        flags.set_bits(0, 5, 2, builder.style_variant_ as u8);
        flags.set(
            PhysicalFragmentFlags::IS_HIDDEN_FOR_PAINT,
            builder.is_hidden_for_paint_,
        );
        flags.set(
            PhysicalFragmentFlags::HAS_RUNNING_ANCHOR_TRANSFORM_ANIMATION,
            builder.has_running_anchor_transform_animation_,
        );
        flags.set(PhysicalFragmentFlags::CHILDREN_VALID, true);
        flags.set(PhysicalFragmentFlags::IS_OPAQUE, builder.is_opaque_);
        flags.set(
            PhysicalFragmentFlags::IS_BLOCK_IN_INLINE,
            builder.is_block_in_inline_,
        );
        flags.set(
            PhysicalFragmentFlags::IS_LINE_FOR_PARALLEL_FLOW,
            builder.is_line_for_parallel_flow_,
        );
        flags.set(
            PhysicalFragmentFlags::MAY_HAVE_DESCENDANT_ABOVE_BLOCK_START,
            builder.may_have_descendant_above_block_start_,
        );
        flags.set(
            PhysicalFragmentFlags::HAS_COLLAPSED_BORDERS,
            builder.has_collapsed_borders_,
        );
        let has_fragmented_out_of_flow_data =
            !builder.oof_positioned_fragmentainer_descendants_.is_empty()
                || !builder.multicols_with_pending_oofs_.is_empty();
        flags.set(
            PhysicalFragmentFlags::HAS_FRAGMENTED_OUT_OF_FLOW_DATA,
            has_fragmented_out_of_flow_data,
        );
        flags.set(
            PhysicalFragmentFlags::HAS_OUT_OF_FLOW_FRAGMENT_CHILD,
            builder.HasOutOfFlowFragmentChild(),
        );
        flags.set(
            PhysicalFragmentFlags::HAS_OUT_OF_FLOW_IN_FRAGMENTAINER_SUBTREE,
            builder.HasOutOfFlowInFragmentainerSubtree(),
        );
        let propagated_data = if !builder.sticky_descendants_.is_null()
            || !builder.snap_areas_.is_null()
            || !builder.scroll_start_target_.is_null()
            || !builder.named_triggers_.is_null()
        {
            Member::from_ptr(MakeGarbageCollected(PropagatedData::new(
                builder.sticky_descendants_,
                builder.snap_areas_,
                Member::from_ptr(builder.scroll_start_target_ as *mut _),
                builder.named_triggers_,
            )))
        } else {
            Member::default()
        };
        let mut result = Self {
            layout_object_: UnsafeCell::new(Member::from_ptr(builder.layout_object_)),
            size_: ToPhysicalSize(builder.size_, builder.GetWritingMode()),
            flags_: flags,
            propagated_data_: propagated_data,
            break_token_: Member::from_ptr(builder.break_token_ as *mut _),
            oof_data_: Member::default(),
        };
        if !builder.oof_positioned_descendants_.is_empty()
            || !builder.GetAnchorMap().is_null()
            || has_fragmented_out_of_flow_data
        {
            result.oof_data_ = Member::from_ptr(result.OofDataFromBuilder(builder));
        }
        assert!(!builder.layout_object_.is_null());
        debug_assert!(!result.IsLineForParallelFlow() || result.break_token_.Get().is_null());
        result.flags_.set(
            PhysicalFragmentFlags::HAS_FLOATING_DESCENDANTS_FOR_PAINT,
            builder.has_floating_descendants_for_paint_,
        );
        result.flags_.set(
            PhysicalFragmentFlags::HAS_ADJOINING_OBJECT_DESCENDANTS,
            builder.has_adjoining_object_descendants_,
        );
        result.flags_.set(
            PhysicalFragmentFlags::DEPENDS_ON_PERCENTAGE_BLOCK_SIZE,
            Self::DependsOnPercentageBlockSizeForBuilder(builder),
        );
        result
            .flags_
            .set(PhysicalFragmentFlags::CHILDREN_VALID, true);
        result
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:668
    // cpp: layoutng_fragment_tree/physical_fragment.cc:757-790
    fn DependsOnPercentageBlockSizeForBuilder(builder: &FragmentBuilder) -> bool {
        if builder.node_.IsNull() || builder.node_.IsInline() {
            return builder.has_descendant_that_depends_on_percentage_block_size_;
        }
        let node: BlockNode = builder.node_.clone().into();
        if builder.has_descendant_that_depends_on_percentage_block_size_
            && node.UseParentPercentageResolutionBlockSizeForChildren()
        {
            return true;
        }
        let style = builder.Style();
        if style.LogicalHeight().MayHavePercentDependence()
            || style.LogicalMinHeight().MayHavePercentDependence()
            || style.LogicalMaxHeight().MayHavePercentDependence()
        {
            return true;
        }
        false
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:672
    // cpp: layoutng_fragment_tree/physical_fragment.cc:31-38
    pub(crate) fn ClearOofData(&mut self) {
        let oof = self.oof_data_.Get();
        if oof.is_null() {
            return;
        }
        if self.HasChildAnchors() {
            unsafe { &mut *oof }.OofPositionedDescendantsMut().clear();
        } else {
            self.oof_data_ = Member::default();
        }
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:653
    // cpp: layoutng_fragment_tree/physical_fragment.cc:467-474
    pub fn GetFragmentedOofData(&self) -> *const FragmentedOofData {
        if !self
            .flags_
            .get(PhysicalFragmentFlags::HAS_FRAGMENTED_OUT_OF_FLOW_DATA)
        {
            return std::ptr::null();
        }
        let data = self.oof_data_.Get() as *const FragmentedOofData;
        debug_assert!(
            !unsafe { &*data }.multicols_with_pending_oofs.is_empty()
                || !unsafe { &*data }
                    .oof_positioned_fragmentainer_descendants
                    .is_empty()
        );
        data
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:657
    // cpp: layoutng_fragment_tree/physical_fragment.cc:476-479
    pub fn HasNestedMulticolsWithOOFs(&self) -> bool {
        let data = self.GetFragmentedOofData();
        !data.is_null() && !unsafe { &*data }.multicols_with_pending_oofs.is_empty()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:673
    // cpp: layoutng_fragment_tree/physical_fragment.cc:585-591
    fn CloneOofData(&self) -> *mut OofData {
        let oof = self.oof_data_.Get();
        debug_assert!(!oof.is_null());
        if !self
            .flags_
            .get(PhysicalFragmentFlags::HAS_FRAGMENTED_OUT_OF_FLOW_DATA)
        {
            return MakeGarbageCollected(unsafe { &*oof }.clone());
        }
        let fragmented = self.GetFragmentedOofData();
        debug_assert!(!fragmented.is_null());
        MakeGarbageCollected(unsafe { &*fragmented }.clone()) as *mut OofData
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:670
    // cpp: layoutng_fragment_tree/physical_fragment.cc:491-522
    fn OofDataFromBuilder(&self, builder: &mut FragmentBuilder) -> *mut OofData {
        let mut oof_data: *mut OofData = std::ptr::null_mut();
        if self
            .flags_
            .get(PhysicalFragmentFlags::HAS_FRAGMENTED_OUT_OF_FLOW_DATA)
        {
            oof_data = self.FragmentedOofDataFromBuilder(builder);
        }
        let converter = WritingModeConverter::new(
            foundation::WritingDirectionMode::new(
                builder.Style().GetWritingMode(),
                builder.Direction(),
            ),
            self.Size(),
        );
        if !builder.oof_positioned_descendants_.is_empty() {
            if oof_data.is_null() {
                oof_data = MakeGarbageCollected(OofData::default());
            }
            let descendants = unsafe { &mut *oof_data }.OofPositionedDescendantsMut();
            descendants.reserve(builder.oof_positioned_descendants_.len());
            for descendant in &builder.oof_positioned_descendants_ {
                descendants.push(LogicalOofPositionedNodeToPhysical(descendant, &converter));
            }
        }
        if !builder.anchor_map_.is_null() {
            if oof_data.is_null() {
                oof_data = MakeGarbageCollected(OofData::default());
            }
            unsafe { &mut *oof_data }.SetAnchorMap(builder.anchor_map_);
        }
        oof_data
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:671
    // cpp: layoutng_fragment_tree/physical_fragment.cc:524-583
    fn FragmentedOofDataFromBuilder(&self, builder: &mut FragmentBuilder) -> *mut OofData {
        debug_assert!(self
            .flags_
            .get(PhysicalFragmentFlags::HAS_FRAGMENTED_OUT_OF_FLOW_DATA));
        debug_assert_eq!(
            self.flags_
                .get(PhysicalFragmentFlags::HAS_FRAGMENTED_OUT_OF_FLOW_DATA),
            !builder.oof_positioned_fragmentainer_descendants_.is_empty()
                || !builder.multicols_with_pending_oofs_.is_empty()
        );
        let fragmented_data = MakeGarbageCollected(FragmentedOofData::default());
        let data = unsafe { &mut *fragmented_data };
        data.oof_positioned_fragmentainer_descendants
            .reserve(builder.oof_positioned_fragmentainer_descendants_.len());
        let size = self.Size();
        let writing_direction = builder.GetWritingDirection();
        let converter = WritingModeConverter::new(writing_direction, size);
        for descendant in &builder.oof_positioned_fragmentainer_descendants_ {
            let inline_container = OofInlineContainer::new(
                descendant.InlineContainer(),
                converter.ToPhysicalOffset(
                    descendant.InlineContainerInfo().RelativeOffset(),
                    PhysicalSize::default(),
                ),
            );
            let fixedpos_inline_container = OofInlineContainer::new(
                descendant.fixedpos_inline_container.Container(),
                converter.ToPhysicalOffset(
                    descendant.fixedpos_inline_container.RelativeOffset(),
                    PhysicalSize::default(),
                ),
            );
            let containing_fragment = descendant.containing_block.Fragment();
            let containing_block_size = if containing_fragment.is_null() {
                size
            } else {
                unsafe { &*containing_fragment }.Size()
            };
            let containing_block_converter =
                WritingModeConverter::new(writing_direction, containing_block_size);
            data.oof_positioned_fragmentainer_descendants.push(
                PhysicalOofNodeForFragmentation::new(
                    descendant.Node(),
                    descendant
                        .StaticPosition()
                        .ConvertToPhysical(&containing_block_converter),
                    descendant.RequiresContentBeforeBreaking(),
                    inline_container,
                    PhysicalContainingBlockWithSize(
                        builder,
                        size,
                        containing_block_size,
                        &descendant.containing_block,
                    ),
                    PhysicalContainingBlock(builder, size, &descendant.fixedpos_containing_block),
                    fixedpos_inline_container,
                ),
            );
        }
        for (key, value) in builder.multicols_with_pending_oofs_.iter() {
            let value = unsafe { &*value.Get() };
            let fixedpos_inline_container = OofInlineContainer::new(
                value.fixedpos_inline_container.Container(),
                converter.ToPhysicalOffset(
                    value.fixedpos_inline_container.RelativeOffset(),
                    PhysicalSize::default(),
                ),
            );
            data.multicols_with_pending_oofs.insert(
                key.clone(),
                Member::from_ptr(MakeGarbageCollected(MulticolWithPendingOofs::new(
                    value.multicol_offset.ConvertToPhysical(
                        builder.Style().GetWritingDirection(),
                        size,
                        PhysicalSize::default(),
                    ),
                    PhysicalContainingBlock(builder, size, &value.fixedpos_containing_block),
                    fixedpos_inline_container,
                ))),
            );
        }
        fragmented_data as *mut OofData
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:228
    // cpp: layoutng_fragment_tree/physical_fragment.cc:442-444
    pub fn IsBlockFlow(&self) -> bool {
        !self.IsLineBox() && unsafe { &*self.layout_object_ptr() }.IsLayoutBlockFlow()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:249
    // cpp: layoutng_fragment_tree/physical_fragment.cc:446-451
    pub fn IsTextControlContainer(&self) -> bool {
        let element = if self.IsCSSBox() {
            DynamicTo::<Element>(unsafe { &*self.layout_object_ptr() }.GetNode())
        } else {
            std::ptr::null_mut()
        };
        !element.is_null()
            && unsafe { &*element }.InputIsTextControlContainer()
            && IsA::<TextControlElement>(unsafe { &*element }.OwnerShadowHost())
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:250
    // cpp: layoutng_fragment_tree/physical_fragment.cc:453-458
    pub fn IsTextControlPlaceholder(&self) -> bool {
        let element = if self.IsCSSBox() {
            DynamicTo::<Element>(unsafe { &*self.layout_object_ptr() }.GetNode())
        } else {
            std::ptr::null_mut()
        };
        !element.is_null()
            && unsafe { &*element }.InputIsTextControlPlaceholder()
            && IsA::<TextControlElement>(unsafe { &*element }.OwnerShadowHost())
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:661
    // cpp: layoutng_fragment_tree/physical_fragment.cc:481-489
    pub fn NeedsOOFPositionedInfoPropagation(&self) -> bool {
        let fragmented = self.GetFragmentedOofData();
        debug_assert_eq!(
            !self.oof_data_.Get().is_null(),
            self.HasOutOfFlowPositionedDescendants()
                || self.HasChildAnchors()
                || (!fragmented.is_null()
                    && unsafe { &*fragmented }.NeedsOOFPositionedInfoPropagation())
        );
        !self.oof_data_.Get().is_null()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:367
    // cpp: layoutng_fragment_tree/physical_fragment.cc:593-603
    pub fn IsMonolithic(&self) -> bool {
        if self.IsLineBox() {
            return !self.IsBlockInInline();
        }
        let box_fragment = DynamicTo::<PhysicalBoxFragment>(self as *const Self);
        if !box_fragment.is_null() {
            return unsafe { &*box_fragment }.IsMonolithic();
        }
        false
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:372
    // cpp: layoutng_fragment_tree/physical_fragment.cc:605-610
    pub fn IsImplicitAnchor(&self) -> bool {
        let element = DynamicTo::<Element>(self.GetNode());
        !element.is_null() && unsafe { &*element }.InputMayBeImplicitAnchor()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:398
    // cpp: layoutng_fragment_tree/physical_fragment.cc:612-619
    pub fn GetFragmentData(&self) -> *const FragmentData {
        let box_object = DynamicTo::<LayoutBox>(self.GetLayoutObject());
        if box_object.is_null() {
            debug_assert!(self.GetLayoutObject().is_null());
            return std::ptr::null();
        }
        let box_fragment = To::<PhysicalBoxFragment>(self as *const Self);
        unsafe { &*box_object }.FragmentDataFromPhysicalFragment(unsafe { &*box_fragment })
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:414
    // cpp: layoutng_fragment_tree/physical_fragment.cc:621-627
    pub fn PostLayout(&self) -> *const PhysicalFragment {
        let box_fragment = DynamicTo::<PhysicalBoxFragment>(self as *const Self);
        if !box_fragment.is_null() {
            return unsafe { &*box_fragment }.PostLayout() as *const PhysicalFragment;
        }
        self
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:418
    // cpp: layoutng_fragment_tree/physical_fragment.cc:675-679
    pub fn ConvertChildToLogical(&self, rect: &PhysicalRect) -> LogicalRect {
        WritingModeConverter::new(self.Style().GetWritingDirection(), self.Size())
            .ToLogicalRect(*rect)
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:422
    // cpp: layoutng_fragment_tree/physical_fragment.h:733-735
    // cpp: layoutng_fragment_tree/physical_fragment.cc:629-673
    pub fn CheckType(&self) {
        #[cfg(debug_assertions)]
        {
            let layout_object = unsafe { &*self.layout_object_ptr() };
            match self.Type() {
                FragmentType::kFragmentBox => {
                    if self.IsInlineBox() {
                        debug_assert!(layout_object.IsLayoutInline());
                    } else {
                        debug_assert!(layout_object.IsBox());
                    }
                    if self.IsFragmentainerBox()
                        || matches!(
                            self.GetBoxType(),
                            BoxType::kPageContainer
                                | BoxType::kPageBorderBox
                                | BoxType::kPageMargin
                        )
                    {
                        debug_assert!(layout_object.IsLayoutBlockFlow());
                        debug_assert!(!self.IsFloating());
                        debug_assert!(!self.IsOutOfFlowPositioned());
                        debug_assert!(!self.IsAtomicInline());
                        debug_assert!(!self.IsFormattingContextRoot());
                        return;
                    }
                    if layout_object.IsLayoutOutsideListMarker() {
                        debug_assert!(!self.IsFloating());
                        debug_assert!(!self.IsOutOfFlowPositioned());
                        debug_assert!(
                            self.IsAtomicInline()
                                || (self.IsBox() && self.GetBoxType() == BoxType::kBlockFlowRoot)
                        );
                        return;
                    }
                    debug_assert_eq!(self.IsFloating(), layout_object.IsFloating());
                    debug_assert_eq!(
                        self.IsOutOfFlowPositioned(),
                        layout_object.IsOutOfFlowPositioned()
                    );
                    debug_assert_eq!(self.IsAtomicInline(), layout_object.IsAtomicInline());
                }
                FragmentType::kFragmentLineBox => {
                    debug_assert!(layout_object.IsLayoutBlockFlow());
                    debug_assert!(!self.IsFloating());
                    debug_assert!(!self.IsOutOfFlowPositioned());
                    debug_assert!(!self.IsInlineBox());
                    debug_assert!(!self.IsAtomicInline());
                }
            }
        }
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:420
    // cpp: layoutng_fragment_tree/physical_fragment.cc:681-692
    pub fn ToString(&self) -> String {
        let mut output = StringBuilder::default();
        output.Append("Type: '");
        output.AppendNumber(self.Type() as u8);
        output.Append("' Size: '");
        output.Append(&self.Size().ToString());
        output.Append("'");
        if self.IsBox() {
            output.Append(", BoxType: '");
            output.Append(&StringForBoxType(self));
            output.Append("'");
        }
        output.ReleaseString()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:442-449
    // cpp: layoutng_fragment_tree/physical_fragment.cc:694-705
    pub fn DumpFragmentTree(
        &self,
        flags: DumpFlags,
        target: *const PhysicalFragment,
        offset: Option<PhysicalOffset>,
        indent: usize,
    ) -> String {
        let mut builder = StringBuilder::default();
        if flags & Self::DumpHeaderText != 0 {
            builder.Append(".:: LayoutNG Physical Fragment Tree ::.\n");
        }
        FragmentTreeDumper {
            builder_: &mut builder,
            target_fragment_: target,
            flags_: flags,
            target_fragment_found_: false,
        }
        .Append(self, offset, &mut Vec::new(), indent);
        builder.ToString()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:451-458
    // cpp: layoutng_fragment_tree/physical_fragment.cc:707-713
    pub fn DumpFragmentTreeFromRoot(
        root: &LayoutObject,
        flags: DumpFlags,
        target: *const PhysicalFragment,
    ) -> String {
        let root_box = To::<LayoutBox>(root as *const LayoutObject);
        let root_box = unsafe { &*root_box };
        debug_assert_eq!(root_box.PhysicalFragmentCount(), 1);
        unsafe { &*root_box.GetPhysicalFragment(0) }.DumpFragmentTree(flags, target, None, 2)
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:460-461
    // cpp: layoutng_fragment_tree/physical_fragment.cc:715-733
    pub fn Trace(&self, visitor: &mut Visitor) {
        match self.Type() {
            FragmentType::kFragmentBox => {
                let box_fragment = self as *const Self as *const PhysicalBoxFragment;
                unsafe { &*box_fragment }.TraceAfterDispatch(visitor);
            }
            FragmentType::kFragmentLineBox => {
                let line = self as *const Self as *const PhysicalLineBoxFragment;
                unsafe { &*line }.TraceAfterDispatch(visitor);
            }
        }
    }

    pub fn TraceAfterDispatch(&self, visitor: &mut Visitor) {
        visitor.Trace(unsafe { &*self.layout_object_.get() });
        visitor.Trace(&self.propagated_data_);
        visitor.Trace(&self.break_token_);
        visitor.Trace(&self.oof_data_);
    }
    pub(crate) fn layout_object_ptr(&self) -> *mut LayoutObject {
        unsafe { &*self.layout_object_.get() }.Get()
    }

    pub(crate) fn sub_type(&self) -> u8 {
        self.flags_.get_bits(0, 1, 4)
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:147-154
    pub fn Type(&self) -> FragmentType {
        if self.flags_.get(PhysicalFragmentFlags::TYPE) {
            FragmentType::kFragmentLineBox
        } else {
            FragmentType::kFragmentBox
        }
    }

    pub fn IsContainer(&self) -> bool {
        matches!(
            self.Type(),
            FragmentType::kFragmentBox | FragmentType::kFragmentLineBox
        )
    }

    pub fn IsBox(&self) -> bool {
        self.Type() == FragmentType::kFragmentBox
    }

    pub fn IsLineBox(&self) -> bool {
        self.Type() == FragmentType::kFragmentLineBox
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:156-175
    pub fn GetBoxType(&self) -> BoxType {
        debug_assert!(self.IsBox());
        match self.sub_type() {
            0 => BoxType::kNormalBox,
            1 => BoxType::kInlineBox,
            2 => BoxType::kColumnBox,
            3 => BoxType::kPageContainer,
            4 => BoxType::kPageBorderBox,
            5 => BoxType::kPageMargin,
            6 => BoxType::kPageArea,
            7 => BoxType::kAtomicInline,
            8 => BoxType::kFloating,
            9 => BoxType::kOutOfFlowPositioned,
            10 => BoxType::kBlockFlowRoot,
            11 => BoxType::kRenderedLegend,
            _ => unreachable!("invalid physical box subtype"),
        }
    }

    pub fn IsInlineBox(&self) -> bool {
        self.IsBox() && self.GetBoxType() == BoxType::kInlineBox
    }

    pub fn IsColumnBox(&self) -> bool {
        self.IsBox() && self.GetBoxType() == BoxType::kColumnBox
    }

    pub fn IsFragmentainerBoxType(box_type: BoxType) -> bool {
        box_type == BoxType::kColumnBox || box_type == BoxType::kPageArea
    }

    pub fn IsFragmentainerBox(&self) -> bool {
        self.IsBox() && Self::IsFragmentainerBoxType(self.GetBoxType())
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:177-210
    pub fn IsAtomicInline(&self) -> bool {
        self.IsBox() && self.GetBoxType() == BoxType::kAtomicInline
    }

    pub fn IsBlockInInline(&self) -> bool {
        self.flags_.get(PhysicalFragmentFlags::IS_BLOCK_IN_INLINE)
    }

    pub fn IsLineForParallelFlow(&self) -> bool {
        self.flags_
            .get(PhysicalFragmentFlags::IS_LINE_FOR_PARALLEL_FLOW)
    }

    pub fn IsInline(&self) -> bool {
        self.IsInlineBox() || self.IsAtomicInline()
    }

    pub fn IsFloating(&self) -> bool {
        self.IsBox() && self.GetBoxType() == BoxType::kFloating
    }

    pub fn IsOutOfFlowPositioned(&self) -> bool {
        self.IsBox() && self.GetBoxType() == BoxType::kOutOfFlowPositioned
    }

    pub fn IsFloatingOrOutOfFlowPositioned(&self) -> bool {
        self.IsFloating() || self.IsOutOfFlowPositioned()
    }

    pub fn IsRenderedLegend(&self) -> bool {
        self.IsBox() && self.GetBoxType() == BoxType::kRenderedLegend
    }

    pub fn IsMathMLFraction(&self) -> bool {
        self.IsBox() && self.flags_.get(PhysicalFragmentFlags::IS_MATH_FRACTION)
    }

    pub fn IsMathMLOperator(&self) -> bool {
        self.IsBox() && self.flags_.get(PhysicalFragmentFlags::IS_MATH_OPERATOR)
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:212-226
    pub fn IsCSSBox(&self) -> bool {
        !self.IsLineBox() && !self.IsFragmentainerBox()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:236
    pub fn IsTablePart(&self) -> bool {
        self.flags_.get(PhysicalFragmentFlags::IS_TABLE_PART)
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:256-274
    pub fn IsFieldsetContainer(&self) -> bool {
        self.flags_
            .get(PhysicalFragmentFlags::IS_FIELDSET_CONTAINER)
    }

    pub fn IsPaintedAtomically(&self) -> bool {
        self.flags_
            .get(PhysicalFragmentFlags::IS_PAINTED_ATOMICALLY)
    }

    pub fn HasCollapsedBorders(&self) -> bool {
        self.flags_
            .get(PhysicalFragmentFlags::HAS_COLLAPSED_BORDERS)
    }

    pub fn IsFormattingContextRoot(&self) -> bool {
        self.IsBox() && self.GetBoxType() >= BoxType::kMinimumFormattingContextRoot
    }

    pub fn MayHaveDescendantAboveBlockStart(&self) -> bool {
        self.flags_
            .get(PhysicalFragmentFlags::MAY_HAVE_DESCENDANT_ABOVE_BLOCK_START)
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:279-289
    pub fn Size(&self) -> PhysicalSize {
        self.size_
    }

    pub fn LocalRect(&self) -> PhysicalRect {
        PhysicalRect::new(PhysicalOffset::default(), self.size_)
    }

    pub fn GetStyleVariant(&self) -> StyleVariant {
        StyleVariant::try_from(self.flags_.get_bits(0, 5, 2))
            .expect("invalid fragment style variant")
    }

    pub fn UsesFirstLineStyle(&self) -> bool {
        UsesFirstLineStyle(self.GetStyleVariant())
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:363
    pub fn IsOpaque(&self) -> bool {
        self.flags_.get(PhysicalFragmentFlags::IS_OPAQUE)
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:376
    pub fn IsAnchor(&self) -> bool {
        self.IsExplicitAnchor() || self.IsImplicitAnchor()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:384-396
    pub fn GetLayoutObject(&self) -> *const LayoutObject {
        if self.IsCSSBox() {
            self.layout_object_ptr()
        } else {
            std::ptr::null()
        }
    }

    pub fn GetMutableLayoutObject(&self) -> *mut LayoutObject {
        if self.IsCSSBox() {
            self.layout_object_ptr()
        } else {
            std::ptr::null_mut()
        }
    }

    pub fn GetSelfOrContainerLayoutObject(&self) -> *const LayoutObject {
        self.layout_object_ptr()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:403-407
    pub fn IsLayoutObjectDestroyedOrMoved(&self) -> bool {
        self.layout_object_ptr().is_null()
    }

    pub fn LayoutObjectWillBeDestroyed(&self) {
        unsafe { &mut *self.layout_object_.get() }.Clear();
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:545
    pub fn GetBreakToken(&self) -> *const BreakToken {
        self.break_token_.Get()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:549-569
    pub fn HasFloatingDescendantsForPaint(&self) -> bool {
        self.flags_
            .get(PhysicalFragmentFlags::HAS_FLOATING_DESCENDANTS_FOR_PAINT)
    }

    pub fn HasAdjoiningObjectDescendants(&self) -> bool {
        self.flags_
            .get(PhysicalFragmentFlags::HAS_ADJOINING_OBJECT_DESCENDANTS)
    }

    pub fn DependsOnPercentageBlockSize(&self) -> bool {
        self.flags_
            .get(PhysicalFragmentFlags::DEPENDS_ON_PERCENTAGE_BLOCK_SIZE)
    }

    pub fn HasRunningAnchorTransformAnimation(&self) -> bool {
        self.flags_
            .get(PhysicalFragmentFlags::HAS_RUNNING_ANCHOR_TRANSFORM_ANIMATION)
    }

    // Return a borrowed sequence rather than manufacturing an unrooted GC
    // allocation for the empty case. The source's Persistent empty vector is
    // observationally identical to an empty slice for these readers.
    // cpp: layoutng_fragment_tree/physical_fragment.h:572-574
    // cpp: layoutng_fragment_tree/physical_fragment.cc:736-745
    pub fn StickyDescendants(&self) -> &[SplitAxisItem<LayoutBoxModelObject>] {
        let propagated = self.propagated_data_.Get();
        if propagated.is_null() {
            return &[];
        }
        let descendants = unsafe { &*propagated }.sticky_descendants.Get();
        if descendants.is_null() {
            return &[];
        }
        unsafe { &*descendants }.as_slice()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:575-580
    pub fn HasConsumedStickyDescendants(&self) -> bool {
        self.StickyDescendants()
            .iter()
            .any(|item| !item.GetIfConsumed().is_null())
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:581-585
    pub fn HasPendingStickyDescendants(&self) -> bool {
        self.StickyDescendants()
            .iter()
            .any(|item| !item.GetIfPending().is_null())
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:586-589
    pub fn ScrollInitialTarget(&self) -> Member<LayoutObject> {
        let propagated = self.propagated_data_.Get();
        if propagated.is_null() {
            Member::default()
        } else {
            unsafe { &*propagated }.scroll_initial_target.clone()
        }
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:590-596
    pub fn PropagatedScrollInitialTarget(&self) -> Member<LayoutObject> {
        let target = self.ScrollInitialTarget();
        if target.Get().is_null() || self.IsScrollContainer() {
            Member::default()
        } else {
            target
        }
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:597-598
    // cpp: layoutng_fragment_tree/physical_fragment.cc:747-755
    pub fn SnapAreas(&self) -> &[SnapArea] {
        let propagated = self.propagated_data_.Get();
        if propagated.is_null() {
            return &[];
        }
        let snap_areas = unsafe { &*propagated }.snap_areas.Get();
        if snap_areas.is_null() {
            return &[];
        }
        unsafe { &*snap_areas }.as_slice()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:599-602
    pub fn HasPendingSnapAreas(&self) -> bool {
        self.SnapAreas().iter().any(|area| area.IsPending())
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:603-607
    pub fn HasPropagatedLayoutObjects(&self) -> bool {
        self.HasPendingStickyDescendants()
            || !self.PropagatedScrollInitialTarget().Get().is_null()
            || self.HasPendingSnapAreas()
            || !self.NamedTriggers().is_null()
            || self.HasChildAnchors()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:627-629
    pub fn HasOutOfFlowFragmentChild(&self) -> bool {
        self.flags_
            .get(PhysicalFragmentFlags::HAS_OUT_OF_FLOW_FRAGMENT_CHILD)
    }

    // C++ exposes a mutable span through a const fragment. A raw slice
    // pointer retains that capability without creating an aliased &mut in
    // safe Rust. The caller must establish exclusive access before mutation.
    // cpp: layoutng_fragment_tree/physical_fragment.h:638
    // cpp: layoutng_fragment_tree/physical_fragment.cc:460-465
    pub fn OutOfFlowPositionedDescendants(&self) -> *mut [PhysicalOofPositionedNode] {
        let oof = self.oof_data_.Get();
        if !self.HasOutOfFlowPositionedDescendants() {
            return std::ptr::slice_from_raw_parts_mut(std::ptr::NonNull::dangling().as_ptr(), 0);
        }
        let descendants = &unsafe { &*oof }.oof_positioned_descendants_;
        std::ptr::slice_from_raw_parts_mut(
            descendants.as_ptr() as *mut PhysicalOofPositionedNode,
            descendants.len(),
        )
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:641
    pub fn HasAnchorsToPropagate(&self) -> bool {
        self.HasChildAnchors() || self.IsAnchor()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:642-647
    pub fn GetAnchorMap(&self) -> *const AnchorMap {
        if !self.HasChildAnchors() {
            return std::ptr::null();
        }
        unsafe { &*self.oof_data_.Get() }.GetAnchorMap()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:649-651
    pub fn NamedTriggers(&self) -> *const TriggerScopedNameMap {
        let propagated = self.propagated_data_.Get();
        if propagated.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*propagated }.named_triggers.Get()
        }
    }
}

// cpp: layoutng_fragment_tree/physical_fragment.h:731-732
// cpp: layoutng_fragment_tree/physical_fragment.cc:804-806
impl fmt::Display for PhysicalFragment {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "{}", self.ToString())
    }
}

// cpp: layoutng_fragment_tree/physical_fragment.h:730
// cpp: layoutng_fragment_tree/physical_fragment.cc:808-813
pub struct PhysicalFragmentPointer(pub *const PhysicalFragment);

impl fmt::Display for PhysicalFragmentPointer {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_null() {
            out.write_str("<null>")
        } else {
            write!(out, "{}", unsafe { &*self.0 })
        }
    }
}
