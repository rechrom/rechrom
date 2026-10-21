use foundation::{
    HeapHashMap, IsHorizontalWritingMode, LayoutUnit, MakeGarbageCollected, Member, PhysicalSize,
    String, TextDirection, ToLineWritingMode, Visitor, WritingDirectionMode, WritingMode,
};
use layoutng::internal::inline_node::InlineNode;
use layoutng::internal::layout_pass_scope::LayoutPassScope;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;

use crate::box_fragment_builder::BoxFragmentBuilder;
use crate::fragment_item::FragmentItem;
use crate::fragment_items::FragmentItems;
use crate::inline_break_token::InlineBreakToken;
use crate::logical_line_container::LogicalLineContainer;
use crate::logical_line_item::{LogicalLineItem, LogicalLineItems};
use crate::physical_box_fragment::PhysicalBoxFragment;
use crate::physical_fragment::PhysicalFragment;
use crate::physical_line_box_fragment::PhysicalLineBoxFragment;

// cpp: layoutng_fragment_tree/fragment_items_builder.h:25-40
pub struct FragmentItemWithOffset {
    pub item: FragmentItem,
    pub offset: LogicalOffset,
}

// cpp: layoutng_fragment_tree/fragment_items_builder.h:129-130
pub type ItemWithOffset = FragmentItemWithOffset;
pub type ItemWithOffsetList = Vec<ItemWithOffset>;

#[allow(non_snake_case)]
impl FragmentItemWithOffset {
    pub fn new(offset: LogicalOffset, item: FragmentItem) -> Self {
        Self { item, offset }
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.item);
    }
}

// cpp: layoutng_fragment_tree/fragment_items_builder.h:109-117
pub struct AddPreviousItemsResult {
    pub inline_break_token: *const InlineBreakToken,
    pub used_block_size: LayoutUnit,
    pub line_count: u32,
    pub succeeded: bool,
}

// The implementation belongs to the excluded //src/layoutng_inline package.
// Keep its declared fragment-tree method as a typed link for later assembly.
extern "Rust" {
    fn FragmentItemsBuilderNewForInline(
        node: &InlineNode,
        writing_direction: WritingDirectionMode,
        is_block_fragmented: bool,
    ) -> FragmentItemsBuilder;
    fn FragmentItemsBuilderAddPreviousItemsFromInline(
        builder: &mut FragmentItemsBuilder,
        container: &PhysicalBoxFragment,
        items: &FragmentItems,
        end_item: &FragmentItem,
        container_builder: &mut BoxFragmentBuilder,
        max_lines: u32,
    ) -> AddPreviousItemsResult;
}

impl Default for AddPreviousItemsResult {
    fn default() -> Self {
        Self {
            inline_break_token: std::ptr::null(),
            used_block_size: LayoutUnit::default(),
            line_count: 0,
            succeeded: false,
        }
    }
}

// cpp: layoutng_fragment_tree/fragment_items_builder.h:45-46
// cpp: layoutng_fragment_tree/fragment_items_builder.h:157-178
pub struct FragmentItemsBuilder {
    pub(crate) items_: Vec<FragmentItemWithOffset>,
    pub(crate) text_content_: String,
    pub(crate) first_line_text_content_: String,
    current_line_container_: *mut LogicalLineContainer,
    current_line_fragment_: *const PhysicalFragment,
    line_container_map_: HeapHashMap<Member<PhysicalFragment>, Member<LogicalLineContainer>>,
    line_container_pool_: *mut LogicalLineContainer,
    pub(crate) node_: InlineNode,
    writing_direction_: WritingDirectionMode,
    pub(crate) is_converted_to_physical_: bool,
    is_line_items_pool_acquired_: bool,
}

// cpp: layoutng_fragment_tree/fragment_items_builder.h:49
// cpp: layoutng_fragment_tree/fragment_items_builder.cc:19-21
#[allow(non_snake_case)]
impl FragmentItemsBuilder {
    // cpp: layoutng_fragment_tree/fragment_items_builder.h:50-52
    // The definition is in //src/layoutng_inline/fragment_items_builder_inline.cc.
    pub fn new_for_inline(
        node: &InlineNode,
        writing_direction: WritingDirectionMode,
        is_block_fragmented: bool,
    ) -> Self {
        unsafe { FragmentItemsBuilderNewForInline(node, writing_direction, is_block_fragmented) }
    }

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:123-128
    pub fn AddPreviousItems(
        &mut self,
        container: &PhysicalBoxFragment,
        items: &FragmentItems,
        end_item: &FragmentItem,
        container_builder: &mut BoxFragmentBuilder,
        max_lines: u32,
    ) -> AddPreviousItemsResult {
        unsafe {
            FragmentItemsBuilderAddPreviousItemsFromInline(
                self,
                container,
                items,
                end_item,
                container_builder,
                max_lines,
            )
        }
    }

    pub fn new(writing_direction: WritingDirectionMode) -> Self {
        Self {
            items_: Vec::new(),
            text_content_: String::default(),
            first_line_text_content_: String::default(),
            current_line_container_: std::ptr::null_mut(),
            current_line_fragment_: std::ptr::null(),
            line_container_map_: HeapHashMap::default(),
            line_container_pool_: MakeGarbageCollected(LogicalLineContainer::default()),
            node_: InlineNode::null(),
            writing_direction_: writing_direction,
            is_converted_to_physical_: false,
            is_line_items_pool_acquired_: false,
        }
    }

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:55-61
    pub fn GetWritingDirection(&self) -> WritingDirectionMode {
        self.writing_direction_
    }

    pub fn GetWritingMode(&self) -> WritingMode {
        self.writing_direction_.GetWritingMode()
    }

    pub fn Direction(&self) -> TextDirection {
        self.writing_direction_.Direction()
    }

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:63
    pub fn Size(&self) -> u32 {
        self.items_.len() as u32
    }

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:65-73
    pub fn TextContent(&self, first_line: bool) -> &String {
        if first_line && !self.first_line_text_content_.IsNull() {
            &self.first_line_text_content_
        } else {
            &self.text_content_
        }
    }

    pub fn TextContentLengthMax(&self) -> u32 {
        self.TextContent(false)
            .length()
            .max(self.TextContent(true).length()) as u32
    }

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:96
    // cpp: layoutng_fragment_tree/fragment_items_builder.cc:37-48
    pub fn ReleaseCurrentLogicalLineContainer(&mut self) {
        if self.current_line_container_.is_null() {
            return;
        }
        if self.current_line_container_ == self.line_container_pool_ {
            debug_assert!(self.is_line_items_pool_acquired_);
            self.is_line_items_pool_acquired_ = false;
        } else {
            unsafe { &mut *self.current_line_container_ }.Clear();
        }
        self.current_line_container_ = std::ptr::null_mut();
    }

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:151
    // cpp: layoutng_fragment_tree/fragment_items_builder.cc:50-59
    fn MoveCurrentLogicalLineItemsToMap(&mut self) {
        if self.current_line_container_.is_null() {
            debug_assert!(self.current_line_fragment_.is_null());
            return;
        }
        debug_assert!(!self.current_line_fragment_.is_null());
        self.line_container_map_.insert(
            Member::from_ptr(self.current_line_fragment_ as *mut _),
            Member::from_ptr(self.current_line_container_),
        );
        self.current_line_fragment_ = std::ptr::null();
        self.current_line_container_ = std::ptr::null_mut();
    }

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:95
    // cpp: layoutng_fragment_tree/fragment_items_builder.cc:61-70
    pub fn AcquireLogicalLineContainer(&mut self) -> *mut LogicalLineContainer {
        if !self.line_container_pool_.is_null() && !self.is_line_items_pool_acquired_ {
            self.is_line_items_pool_acquired_ = true;
            return self.line_container_pool_;
        }
        self.MoveCurrentLogicalLineItemsToMap();
        debug_assert!(self.current_line_container_.is_null());
        self.current_line_container_ = MakeGarbageCollected(LogicalLineContainer::default());
        self.current_line_container_
    }

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:97-98
    // cpp: layoutng_fragment_tree/fragment_items_builder.cc:72-82
    pub fn GetLogicalLineItems(&self, line: &PhysicalLineBoxFragment) -> &LogicalLineItems {
        if line as *const _ as *const PhysicalFragment == self.current_line_fragment_ {
            debug_assert!(!self.current_line_container_.is_null());
            return unsafe { &*(&*self.current_line_container_).BaseLine() };
        }
        let container = self
            .line_container_map_
            .get(&Member::from_ptr(line as *const _ as *mut PhysicalFragment))
            .expect("line fragment must have an associated logical container")
            .Get();
        debug_assert!(!container.is_null());
        unsafe { &*(&*container).BaseLine() }
    }

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:99-100
    // cpp: layoutng_fragment_tree/fragment_items_builder.cc:84-91
    pub fn AssociateLogicalLineContainer(
        &mut self,
        line_container: *mut LogicalLineContainer,
        line_fragment: &PhysicalFragment,
    ) {
        debug_assert!(
            self.current_line_container_.is_null()
                || self.current_line_container_ == line_container
        );
        self.current_line_container_ = line_container;
        debug_assert!(self.current_line_fragment_.is_null());
        self.current_line_fragment_ = line_fragment;
    }

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:101-102
    // cpp: layoutng_fragment_tree/fragment_items_builder.cc:93-164
    pub fn AddLine(&mut self, line_fragment: &PhysicalLineBoxFragment, offset: LogicalOffset) {
        debug_assert!(!self.is_converted_to_physical_);
        let line_fragment_ptr = line_fragment as *const _ as *const PhysicalFragment;
        if line_fragment_ptr == self.current_line_fragment_ {
            debug_assert!(!self.current_line_container_.is_null());
            self.current_line_fragment_ = std::ptr::null();
        } else {
            self.MoveCurrentLogicalLineItemsToMap();
            debug_assert!(self.current_line_container_.is_null());
            self.current_line_container_ = self
                .line_container_map_
                .Take(&Member::from_ptr(line_fragment_ptr as *mut _))
                .Get();
            debug_assert!(!self.current_line_container_.is_null());
        }
        let line_container = unsafe { &mut *self.current_line_container_ };
        let line_items = unsafe { &mut *line_container.BaseLine() };
        let size_before = self.items_.len();
        let estimated_size = size_before + line_container.EstimatedFragmentItemCount() as usize;
        let old_capacity = self.items_.capacity();
        if estimated_size > old_capacity {
            self.items_
                .reserve(estimated_size.max(old_capacity * 2) - size_before);
        }
        let line_start_index = self.items_.len();
        self.items_.push(FragmentItemWithOffset::new(
            offset,
            FragmentItem::from_line_fragment(line_fragment),
        ));
        self.AddItems(line_items.AsSpan());

        for annotation_line in line_container.AnnotationLineList() {
            let annotation_line_start_index = self.items_.len();
            let line_height = annotation_line.metrics.LineHeight();
            let annotation_items = unsafe { &mut *annotation_line.get() };
            let first = annotation_items.FirstInFlowChild();
            if first.is_null() {
                continue;
            }
            if line_fragment.IsHiddenForPaint() {
                for item in annotation_items.iter_mut() {
                    item.is_hidden_for_paint = true;
                }
            }
            let line_offset = *unsafe { &*first }.Offset();
            let last = annotation_items.LastInFlowChild();
            let inline_size = unsafe { &*last }.rect.InlineEndOffset() - line_offset.inline_offset;
            let size = if IsHorizontalWritingMode(self.GetWritingMode()) {
                PhysicalSize::new(inline_size, line_height)
            } else {
                PhysicalSize::new(line_height, inline_size)
            };
            self.items_.push(FragmentItemWithOffset::new(
                line_offset,
                FragmentItem::from_annotation_line(size, line_fragment),
            ));
            self.AddItems(annotation_items.AsSpan());
            let descendants_count = (self.items_.len() - annotation_line_start_index) as u32;
            self.items_[annotation_line_start_index]
                .item
                .SetDescendantsCount(descendants_count);
        }
        let item_count = self.items_.len() - line_start_index;
        let line_item = &mut self.items_[line_start_index].item;
        debug_assert_eq!(line_item.DescendantsCount(), 1);
        line_item.SetDescendantsCount(item_count as u32);
        line_item.SetLineTextFitScale(line_container.TextFitScale());
        self.ReleaseCurrentLogicalLineContainer();
        debug_assert!(self.items_.len() <= estimated_size);
    }

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:153
    // cpp: layoutng_fragment_tree/fragment_items_builder.cc:166-205
    fn AddItems(&mut self, child_span: &mut [LogicalLineItem]) {
        debug_assert!(!self.is_converted_to_physical_);
        let writing_mode = self.GetWritingMode();
        let mut i = 0;
        while i < child_span.len() {
            let child = &mut child_span[i];
            debug_assert!(child.out_of_flow_positioned_box.Get().is_null());
            if !child.CanCreateFragmentItem() {
                i += 1;
                continue;
            }
            if child.children_count <= 1 {
                self.items_.push(FragmentItemWithOffset::new(
                    child.rect.offset,
                    FragmentItem::from_logical_line_item(std::mem::take(child), writing_mode),
                ));
                i += 1;
                continue;
            }
            let children_count = child.children_count as usize;
            let box_start_index = self.items_.len();
            self.items_.push(FragmentItemWithOffset::new(
                child.rect.offset,
                FragmentItem::from_logical_line_item(std::mem::take(child), writing_mode),
            ));
            assert!(children_count >= 1);
            self.AddItems(&mut child_span[i + 1..i + children_count]);
            i += children_count;
            let count = self.items_.len() - box_start_index;
            let box_item = &mut self.items_[box_start_index].item;
            debug_assert_eq!(box_item.DescendantsCount(), 1);
            box_item.SetDescendantsCount(count as u32);
        }
    }

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:105-106
    // cpp: layoutng_fragment_tree/fragment_items_builder.cc:207-216
    pub fn AddListMarker(&mut self, marker: &PhysicalBoxFragment, offset: LogicalOffset) {
        debug_assert!(!self.is_converted_to_physical_);
        self.items_.push(FragmentItemWithOffset::new(
            offset,
            FragmentItem::from_box_fragment(marker, TextDirection::kLtr),
        ));
    }

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:155
    // cpp: layoutng_fragment_tree/fragment_items_builder.cc:218-259
    fn ConvertToPhysical(&mut self, outer_size: PhysicalSize) {
        if self.is_converted_to_physical_ {
            return;
        }
        let converter = WritingModeConverter::new(self.GetWritingDirection(), outer_size);
        let mut line_converter = WritingModeConverter::new(
            WritingDirectionMode::new(
                ToLineWritingMode(self.GetWritingMode()),
                TextDirection::kLtr,
            ),
            PhysicalSize::default(),
        );
        let mut i = 0;
        while i < self.items_.len() {
            let line_data = {
                let item_with_offset = &mut self.items_[i];
                item_with_offset.item.SetOffset(
                    converter
                        .ToPhysicalOffset(item_with_offset.offset, item_with_offset.item.Size()),
                );
                (item_with_offset.item.Type() == FragmentItem::kLine).then(|| {
                    (
                        item_with_offset.item.DescendantsCount(),
                        *item_with_offset.item.RectInContainerFragment(),
                    )
                })
            };
            if let Some((mut descendants_count, line_box_bounds)) = line_data {
                debug_assert!(descendants_count > 0);
                if descendants_count > 0 {
                    line_converter.SetOuterSize(line_box_bounds.size);
                    while descendants_count > 1 {
                        descendants_count -= 1;
                        i += 1;
                        assert!(i < self.items_.len());
                        let descendant = &mut self.items_[i];
                        descendant.item.SetOffset(
                            line_converter
                                .ToPhysicalOffset(descendant.offset, descendant.item.Size())
                                + line_box_bounds.offset,
                        );
                    }
                }
            }
            i += 1;
        }
        self.is_converted_to_physical_ = true;
    }

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:133
    // cpp: layoutng_fragment_tree/fragment_items_builder.cc:261-283
    pub fn MoveChildrenInDirection(&mut self, offset: LayoutUnit, is_block_direction: bool) {
        debug_assert!(!self.is_converted_to_physical_);
        let mut i = 0;
        while i < self.items_.len() {
            let item = &mut self.items_[i];
            if is_block_direction {
                item.offset.block_offset += offset;
            } else {
                item.offset.inline_offset += offset;
            }
            if item.item.Type() == FragmentItem::kLine {
                i += item.item.DescendantsCount() as usize;
                debug_assert!(i <= self.items_.len());
            } else {
                i += 1;
            }
        }
    }

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:140
    // cpp: layoutng_fragment_tree/fragment_items_builder.cc:301-305
    pub fn Items(&mut self, outer_size: PhysicalSize) -> &[FragmentItemWithOffset] {
        self.ConvertToPhysical(outer_size);
        &self.items_
    }

    // The InlineNode constructor and AddPreviousItems bodies are implemented
    // in the unselected layoutng_inline package. AddPreviousItems is forwarded
    // through the typed declaration above until that package is connected.

    // cpp: layoutng_fragment_tree/fragment_items_builder.h:147-148
    // cpp: layoutng_fragment_tree/fragment_items_builder.cc:285-299
    pub unsafe fn ToFragmentItems(
        &mut self,
        outer_size: PhysicalSize,
        data: *mut FragmentItems,
    ) -> Option<PhysicalSize> {
        debug_assert!(!self.text_content_.IsNull());
        self.ConvertToPhysical(outer_size);
        let mut new_size = None;
        if self.node_.IsSvgText() {
            let algorithms = LayoutPassScope::Algorithms();
            if algorithms.is_null() || unsafe { &*algorithms }.svg_support.layout_text.is_none() {
                std::panic::panic_any(layoutng::UnsupportedLayout::new(
                    "SVG text layout module is not installed",
                ));
            }
            let node = &self.node_ as *const InlineNode;
            let items = &mut self.items_ as *mut Vec<FragmentItemWithOffset>;
            let builder = self as *mut FragmentItemsBuilder;
            new_size = Some(unsafe { &*algorithms }
                .svg_support
                .layout_text
                .expect("checked SVG text layout callback")(
                node, builder, items
            ));
        }
        unsafe { std::ptr::write(data, FragmentItems::from_builder(self)) };
        new_size
    }
}

// cpp: layoutng_fragment_tree/fragment_items_builder.h:53
// cpp: layoutng_fragment_tree/fragment_items_builder.cc:23-35
impl Drop for FragmentItemsBuilder {
    fn drop(&mut self) {
        self.ReleaseCurrentLogicalLineContainer();
        debug_assert!(!self.line_container_pool_.is_null());
        unsafe { &mut *self.line_container_pool_ }.Clear();
        for (_line, container) in self.line_container_map_.iter() {
            let container = container.Get();
            if container != self.line_container_pool_ {
                unsafe { &mut *container }.Clear();
            }
        }
    }
}
