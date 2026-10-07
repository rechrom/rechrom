use font_engine::ShapeResultView;
use foundation::graphics_types;
use foundation::{
    gfx, DynamicTo, LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize, RuntimeEnabledFeatures,
    StringView, TextDirection, To, WritingDirectionMode,
};
use graphics_types::graphics::paint::display_item_client::DisplayItemClient;
use layoutng::internal::inline_item::InlineItem;
use layoutng::internal::layout_block_flow::LayoutBlockFlow;
use layoutng::internal::layout_inline::LayoutInline;
use layoutng::internal::layout_node_metadata::Node;
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_text::LayoutText;
use layoutng::internal::style_variant::{StyleVariant, UsesFirstLineStyle};
use layoutng::internal::text_offset_range::TextOffsetRange;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::fragment_item::{FragmentItem, FragmentItemType};
use crate::fragment_items::FragmentItems;
use crate::inline_break_token::InlineBreakToken;
use crate::physical_box_fragment::PhysicalBoxFragment;

// C++ defines these LayoutInline members in inline_cursor.cc. Rust cannot add
// inherent methods to the later layoutng crate's type, so retain their bodies
// here as free functions for that crate to forward to.
// cpp: layoutng_fragment_tree/inline_cursor.cc:28-42
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInlineHasInlineFragments(inline: &LayoutInline) -> bool {
    inline.FirstInlineFragmentItemIndex() != 0
}

#[allow(non_snake_case)]
pub fn LayoutInlinePhysicalLinesBoundingBox(inline: &LayoutInline) -> PhysicalRect {
    let mut cursor = InlineCursor::default();
    cursor.MoveToIncludingCulledInline(unsafe {
        &*(inline as *const LayoutInline as *const LayoutObject)
    });
    let mut bounding_box = PhysicalRect::default();
    while cursor.IsNotNull() {
        bounding_box.UniteIfNonZero(&cursor.Current().RectInContainerFragment());
        cursor.MoveToNextForSameLayoutObject();
    }
    bounding_box
}

// cpp: layoutng_fragment_tree/inline_cursor.cc:46-50
fn IsBidiControl(text: StringView) -> bool {
    if text.length() != 1 {
        return false;
    }
    let ch = text[0] as u32;
    ch == 0x061c
        || (0x200e..=0x200f).contains(&ch)
        || (0x202a..=0x202e).contains(&ch)
        || (0x2066..=0x2069).contains(&ch)
}

// C++ retains both an item pointer and its iterator. The index is the Rust
// equivalent of the iterator into the cursor's current ItemsSpan.
// cpp: layoutng_fragment_tree/inline_cursor.h:47-66
// cpp: layoutng_fragment_tree/inline_cursor.h:213-240
// cpp: layoutng_fragment_tree/inline_cursor.h:649-650
// Keeping the span index makes C++'s iterator-distance conversion unnecessary.
#[derive(Clone, Copy)]
pub struct InlineCursorPosition {
    item_: *const FragmentItem,
    item_index_: usize,
}

impl Default for InlineCursorPosition {
    fn default() -> Self {
        Self {
            item_: std::ptr::null(),
            item_index_: 0,
        }
    }
}

#[allow(non_snake_case)]
impl InlineCursorPosition {
    // cpp: layoutng_fragment_tree/inline_cursor.h:54-60
    pub fn Item(&self) -> *const FragmentItem {
        self.item_
    }
    pub fn IsNotNull(&self) -> bool {
        !self.item_.is_null()
    }
    pub fn IsNull(&self) -> bool {
        self.item_.is_null()
    }

    fn item(&self) -> &FragmentItem {
        unsafe { &*self.item_ }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.h:66-111
    pub fn IsText(&self) -> bool {
        self.item().IsText()
    }
    pub fn IsGeneratedText(&self) -> bool {
        self.item().IsGeneratedText()
    }
    pub fn IsLayoutGeneratedText(&self) -> bool {
        self.item().Type() == FragmentItemType::kGeneratedText
    }
    pub fn IsLineBreak(&self) -> bool {
        self.IsText() && self.item().IsLineBreak()
    }
    pub fn IsEllipsis(&self) -> bool {
        self.item().IsEllipsis()
    }
    pub fn IsLineBox(&self) -> bool {
        self.item().Type() == FragmentItemType::kLine
    }
    pub fn IsEmptyLineBox(&self) -> bool {
        self.item().IsEmptyLineBox()
    }
    pub fn IsRubyAnnotationLine(&self) -> bool {
        self.item().IsRubyAnnotationLine()
    }
    pub fn IsInlineBox(&self) -> bool {
        self.item().IsInlineBox()
    }
    pub fn IsAtomicInline(&self) -> bool {
        self.item().IsAtomicInline()
    }
    pub fn IsListMarker(&self) -> bool {
        self.item().IsListMarker()
    }
    pub fn IsFloating(&self) -> bool {
        self.item().IsFloating()
    }
    pub fn IsHiddenForPaint(&self) -> bool {
        self.item().IsHiddenForPaint()
    }
    // cpp: layoutng_fragment_tree/inline_cursor.cc:170-174
    pub fn HasChildren(&self) -> bool {
        assert!(!self.item_.is_null());
        self.item().HasChildren()
    }
    // cpp: layoutng_fragment_tree/inline_cursor.cc:314-320
    pub fn CanHaveChildren(&self) -> bool {
        assert!(!self.item_.is_null());
        self.IsLineBox() || (self.item().Type() == FragmentItemType::kBox && !self.IsAtomicInline())
    }
    // cpp: layoutng_fragment_tree/inline_cursor.h:113-118
    pub fn GetStyleVariant(&self) -> StyleVariant {
        self.item().GetStyleVariant()
    }
    pub fn UsesFirstLineStyle(&self) -> bool {
        UsesFirstLineStyle(self.GetStyleVariant())
    }
    pub fn Style(&self) -> &ComputedStyle {
        self.item().Style()
    }
    // cpp: layoutng_fragment_tree/inline_cursor.h:120-135
    pub fn BoxFragment(&self) -> *const PhysicalBoxFragment {
        self.item().BoxFragment()
    }
    pub fn GetLayoutObject(&self) -> *const LayoutObject {
        self.item().GetLayoutObject()
    }
    pub fn GetMutableLayoutObject(&self) -> *mut LayoutObject {
        self.item().GetMutableLayoutObject()
    }
    pub fn GetDisplayItemClient(&self) -> *const DisplayItemClient {
        self.item().GetDisplayItemClient()
    }
    pub fn FragmentId(&self) -> u32 {
        self.item().FragmentId()
    }
    // cpp: layoutng_fragment_tree/inline_cursor.h:148-165
    pub fn RectInContainerFragment(&self) -> PhysicalRect {
        *self.item().RectInContainerFragment()
    }
    pub fn OffsetInContainerFragment(&self) -> PhysicalOffset {
        *self.item().OffsetInContainerFragment()
    }
    pub fn Size(&self) -> PhysicalSize {
        self.item().Size()
    }
    pub fn InkOverflowRect(&self) -> PhysicalRect {
        self.item().InkOverflowRect()
    }
    pub fn SelfInkOverflowRect(&self) -> PhysicalRect {
        self.item().SelfInkOverflowRect()
    }
    // cpp: layoutng_fragment_tree/inline_cursor.h:170-183
    pub fn TextOffset(&self) -> TextOffsetRange {
        self.item().TextOffset()
    }
    pub fn TextStartOffset(&self) -> u32 {
        self.TextOffset().start
    }
    pub fn TextEndOffset(&self) -> u32 {
        self.TextOffset().end
    }
    pub fn TextShapeResult(&self) -> *const ShapeResultView {
        self.item().TextShapeResult()
    }
    // cpp: layoutng_fragment_tree/inline_cursor.h:189-199
    pub fn ResolvedDirection(&self) -> TextDirection {
        self.item().ResolvedDirection()
    }

    // cpp: layoutng_fragment_tree/inline_cursor.h:143-146
    pub fn GetInlineBreakToken(&self) -> *const InlineBreakToken {
        self.item().GetInlineBreakToken()
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:217-234
    pub fn HasSoftWrapToNextLine(&self) -> bool {
        debug_assert!(self.IsLineBox());
        let token = self.GetInlineBreakToken();
        !token.is_null() && !unsafe { &*token }.IsForcedBreak()
    }

    pub fn IsInlineLeaf(&self) -> bool {
        if self.IsHiddenForPaint() {
            return false;
        }
        if self.IsText() {
            return !self.IsLayoutGeneratedText();
        }
        if self.IsAtomicInline() {
            return !self.IsListMarker();
        }
        false
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:236-269
    pub fn IsPartOfCulledInlineBox(&self, layout_inline: &LayoutInline) -> bool {
        debug_assert!(!layout_inline.ShouldCreateBoxFragment());
        let object = self.GetLayoutObject();
        if object.is_null() || unsafe { &*object }.IsAtomicInline() {
            return false;
        }
        if !unsafe { &*object }.IsInline() && !unsafe { &*object }.IsBlockInInline() {
            return false;
        }
        let mut parent = unsafe { &*object }.Parent();
        while !parent.is_null() {
            if std::ptr::eq(
                parent,
                layout_inline as *const LayoutInline as *const LayoutObject,
            ) {
                return true;
            }
            let parent_inline = DynamicTo::<LayoutInline>(parent);
            if parent_inline.is_null() || unsafe { &*parent_inline }.ShouldCreateBoxFragment() {
                return false;
            }
            parent = unsafe { &*parent }.Parent();
        }
        false
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:322-327
    pub fn BaseDirection(&self) -> TextDirection {
        assert!(!self.item_.is_null());
        debug_assert!(self.IsLineBox());
        self.item().BaseDirection()
    }

    pub fn ResolvedOrBaseDirection(&self) -> TextDirection {
        if self.IsLineBox() {
            self.BaseDirection()
        } else {
            self.ResolvedDirection()
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:329-393
    pub fn BidiLevel(&self) -> u8 {
        if self.IsText() {
            if self.IsLayoutGeneratedText() {
                return 0;
            }
            let layout_text = unsafe { &*To::<LayoutText>(self.GetLayoutObject()) };
            debug_assert!(!layout_text.NeedsLayout());
            let (items, check_layout_object) = self.InlineItemsFor(layout_text);
            if items.is_empty() {
                return 0;
            }
            let offset = self.TextOffset();
            let item = items
                .into_iter()
                .find(|item| {
                    let item = unsafe { &**item };
                    (!check_layout_object
                        || item.GetLayoutObject() as *const LayoutObject == self.GetLayoutObject())
                        && item.StartOffset() <= offset.start
                        && item.EndOffset() >= offset.end
                })
                .expect("text fragment must have an inline item");
            return unsafe { &*item }.BidiLevel();
        }
        if self.IsAtomicInline() {
            let object = unsafe { &*self.GetLayoutObject() };
            let block_flow = object.FragmentItemsContainer();
            debug_assert!(!block_flow.is_null());
            let node_data = unsafe { &*block_flow }.GetInlineNodeData();
            let items = &unsafe { &*node_data }
                .ItemsData(self.UsesFirstLineStyle())
                .items;
            let item = items
                .iter()
                .find(|item| {
                    unsafe { &*item.Get() }.GetLayoutObject() as *const LayoutObject
                        == self.GetLayoutObject()
                })
                .expect("atomic inline must have an inline item");
            return unsafe { &*item.Get() }.BidiLevel();
        }
        panic!("bidi level requires text or atomic inline");
    }

    fn InlineItemsFor(&self, layout_text: &LayoutText) -> (Vec<*const InlineItem>, bool) {
        let span = layout_text.GetInlineItems();
        if span.empty() {
            return (Vec::new(), false);
        }
        if self.UsesFirstLineStyle() && RuntimeEnabledFeatures::FirstLineTextTransformEnabled() {
            let block_flow = layout_text.FragmentItemsContainer();
            if !block_flow.is_null() {
                let node_data = unsafe { &*block_flow }.GetInlineNodeData();
                if !node_data.is_null() && unsafe { &*node_data }.HasFirstLineItems() {
                    let items = &unsafe { &*node_data }.ItemsData(true).items;
                    return (
                        items
                            .iter()
                            .map(|item| item.Get() as *const InlineItem)
                            .collect(),
                        true,
                    );
                }
            }
        }
        (
            span.Items()
                .iter()
                .map(|item| item.Get() as *const InlineItem)
                .collect(),
            false,
        )
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:396-415
    pub fn GetNode(&self) -> *const Node {
        let object = self.GetLayoutObject();
        if object.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*object }.GetNode()
        }
    }

    pub fn ObjectBoundingBox(&self, cursor: &InlineCursor) -> gfx::RectF {
        self.item().ObjectBoundingBox(cursor.Items())
    }

    pub fn Text(&self, cursor: &InlineCursor) -> StringView {
        debug_assert!(self.IsText());
        cursor.CheckValid(self);
        self.item().Text(cursor.Items())
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:452-464
    pub fn ConvertChildToLogical(&self, physical_rect: PhysicalRect) -> LogicalRect {
        WritingModeConverter::new(
            WritingDirectionMode::new(
                self.Style().GetWritingMode(),
                self.ResolvedOrBaseDirection(),
            ),
            self.Size(),
        )
        .ToLogicalRect(physical_rect)
    }

    pub fn ConvertChildToPhysical(&self, logical_rect: LogicalRect) -> PhysicalRect {
        WritingModeConverter::new(
            WritingDirectionMode::new(
                self.Style().GetWritingMode(),
                self.ResolvedOrBaseDirection(),
            ),
            self.Size(),
        )
        .ToPhysicalRect(logical_rect)
    }

    // cpp: layoutng_fragment_tree/inline_cursor.h:217-225
    fn Set(&mut self, item: *const FragmentItem, index: usize) {
        self.item_ = item;
        self.item_index_ = index;
    }
    fn Clear(&mut self) {
        self.item_ = std::ptr::null();
    }
}

impl PartialEq for InlineCursorPosition {
    fn eq(&self, other: &Self) -> bool {
        self.item_ == other.item_
    }
}

// cpp: layoutng_fragment_tree/inline_cursor.h:248-265
// cpp: layoutng_fragment_tree/inline_cursor.h:650-669
#[derive(Clone)]
pub struct InlineCursor {
    current_: InlineCursorPosition,
    items_data_: *const FragmentItem,
    items_len_: usize,
    fragment_items_: *const FragmentItems,
    root_box_fragment_: *const PhysicalBoxFragment,
    culled_inline_: CulledInlineTraversal,
    root_block_flow_: *const LayoutBlockFlow,
    previously_consumed_block_size_: LayoutUnit,
    fragment_index_: u32,
    max_fragment_index_: u32,
}

impl Default for InlineCursor {
    fn default() -> Self {
        Self {
            current_: InlineCursorPosition::default(),
            items_data_: std::ptr::null(),
            items_len_: 0,
            fragment_items_: std::ptr::null(),
            root_box_fragment_: std::ptr::null(),
            culled_inline_: CulledInlineTraversal::default(),
            root_block_flow_: std::ptr::null(),
            previously_consumed_block_size_: LayoutUnit::default(),
            fragment_index_: 0,
            max_fragment_index_: 0,
        }
    }
}

// cpp: layoutng_fragment_tree/inline_cursor.h:604-640
#[derive(Clone)]
struct CulledInlineTraversal {
    current_object_: *const LayoutObject,
    layout_inline_: *const layoutng::internal::layout_inline::LayoutInline,
    use_fragment_tree_: bool,
}

impl Default for CulledInlineTraversal {
    fn default() -> Self {
        Self {
            current_object_: std::ptr::null(),
            layout_inline_: std::ptr::null(),
            use_fragment_tree_: false,
        }
    }
}

#[allow(non_snake_case)]
impl CulledInlineTraversal {
    // cpp: layoutng_fragment_tree/inline_cursor.h:621-621
    fn GetLayoutInline(&self) -> *const LayoutInline {
        self.layout_inline_
    }

    // cpp: layoutng_fragment_tree/inline_cursor.h:623-623
    fn IsActive(&self) -> bool {
        !self.layout_inline_.is_null()
    }

    // cpp: layoutng_fragment_tree/inline_cursor.h:624-624
    fn Reset(&mut self) {
        self.layout_inline_ = std::ptr::null();
    }

    // cpp: layoutng_fragment_tree/inline_cursor.h:626-626
    fn UseFragmentTree(&self) -> bool {
        self.use_fragment_tree_
    }

    // cpp: layoutng_fragment_tree/inline_cursor.h:627-627
    // cpp: layoutng_fragment_tree/inline_cursor.cc:1147-1151
    fn SetUseFragmentTree(&mut self, inline: &LayoutInline) {
        self.layout_inline_ = inline;
        self.use_fragment_tree_ = true;
    }
    // cpp: layoutng_fragment_tree/inline_cursor.cc:1110-1164
    fn Find(&self, mut child: *const LayoutObject) -> *const LayoutObject {
        while !child.is_null() {
            let object = unsafe { &*child };
            if object.IsText() {
                return child;
            }
            if object.IsBox() {
                if !object.IsFloatingOrOutOfFlowPositioned()
                    && (!RuntimeEnabledFeatures::InlineCursorSkipNonIfcEnabled()
                        || object.IsInLayoutNGInlineFormattingContext())
                {
                    return child;
                }
                child =
                    object.NextInPreOrderAfterChildren(self.layout_inline_ as *const LayoutObject);
                continue;
            }
            let child_inline = DynamicTo::<LayoutInline>(child);
            if !child_inline.is_null() {
                if unsafe { &*child_inline }.ShouldCreateBoxFragment() {
                    return child;
                }
                let grandchild = unsafe { &*child_inline }.FirstChild();
                if !grandchild.is_null() {
                    child = grandchild;
                    continue;
                }
            }
            child = object.NextInPreOrderAfterChildren(self.layout_inline_ as *const LayoutObject);
        }
        std::ptr::null()
    }
    fn MoveToFirstFor(&mut self, inline: &LayoutInline) -> *const LayoutObject {
        self.layout_inline_ = inline;
        self.use_fragment_tree_ = false;
        self.current_object_ = self.Find(inline.FirstChild());
        self.current_object_
    }
    fn MoveToNext(&mut self) -> *const LayoutObject {
        if self.current_object_.is_null() {
            return std::ptr::null();
        }
        let child = unsafe { &*self.current_object_ }
            .NextInPreOrderAfterChildren(self.layout_inline_ as *const LayoutObject);
        self.current_object_ = self.Find(child);
        self.current_object_
    }
}

#[allow(non_snake_case)]
impl InlineCursor {
    fn items(&self) -> &[FragmentItem] {
        if self.items_len_ == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.items_data_, self.items_len_) }
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.h:577-577
    fn MakeNull(&mut self) {
        self.current_.Clear();
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:67-77
    fn SetRootSpan(
        &mut self,
        box_fragment: &PhysicalBoxFragment,
        fragment_items: &FragmentItems,
        items: &[FragmentItem],
    ) {
        debug_assert_eq!(box_fragment.Items(), fragment_items as *const _);
        debug_assert!(fragment_items.IsSubSpan(items));
        self.root_box_fragment_ = box_fragment;
        self.fragment_items_ = fragment_items;
        self.items_data_ = items.as_ptr();
        self.items_len_ = items.len();
        self.MoveToItem(0);
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:79-82
    fn SetRoot(&mut self, box_fragment: &PhysicalBoxFragment, items: &FragmentItems) {
        self.SetRootSpan(box_fragment, items, items.Items());
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:127-131
    pub fn from_container_and_items(
        box_fragment: &PhysicalBoxFragment,
        items: &FragmentItems,
    ) -> Self {
        let mut result = Self::default();
        result.SetRoot(box_fragment, items);
        result
    }

    pub fn new_with_items(box_fragment: &PhysicalBoxFragment, items: &FragmentItems) -> Self {
        Self::from_container_and_items(box_fragment, items)
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:138-142
    pub fn from_fragment(box_fragment: &PhysicalBoxFragment) -> Self {
        let mut result = Self::default();
        let items = box_fragment.Items();
        if !items.is_null() {
            result.SetRoot(box_fragment, unsafe { &*items });
        }
        result
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:144-147
    pub fn from_backward_cursor(backward: &InlineBackwardCursor<'_>) -> Self {
        let mut result = backward.cursor_.clone();
        result.MoveToPosition(backward.current_);
        result
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:123-125
    pub fn from_container(block_flow: &LayoutBlockFlow) -> Self {
        let mut result = Self::default();
        result.SetRootBlockFlow(block_flow);
        result
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:106-121
    fn SetRootBlockFlow(&mut self, block_flow: &LayoutBlockFlow) {
        debug_assert!(!self.HasRoot());
        let count = u32::try_from(block_flow.PhysicalFragmentCount())
            .expect("physical fragment count exceeds wtf_size_t");
        if count != 0 {
            self.root_block_flow_ = block_flow;
            self.max_fragment_index_ = count - 1;
            self.ResetFragmentIndex();
            self.TrySetRootFragmentItems();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:84-104
    fn TrySetRootFragmentItems(&mut self) -> bool {
        debug_assert!(!self.root_block_flow_.is_null());
        debug_assert!(self.fragment_items_.is_null() || self.Items().Equals(self.items()));
        let root = unsafe { &*self.root_block_flow_ };
        if !root.MayHaveFragmentItems() {
            self.fragment_index_ = self.max_fragment_index_ + 1;
            return false;
        }
        while self.fragment_index_ <= self.max_fragment_index_ {
            let fragment = root.GetPhysicalFragment(self.fragment_index_ as usize);
            debug_assert!(!fragment.is_null());
            let items = unsafe { &*fragment }.Items();
            if !items.is_null() {
                self.SetRoot(unsafe { &*fragment }, unsafe { &*items });
                return true;
            }
            self.IncrementFragmentIndex();
        }
        false
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:1223-1226
    fn ResetFragmentIndex(&mut self) {
        self.fragment_index_ = 0;
        self.previously_consumed_block_size_ = LayoutUnit::default();
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:1242-1251
    fn IncrementFragmentIndex(&mut self) {
        debug_assert!(self.fragment_index_ <= self.max_fragment_index_);
        self.fragment_index_ += 1;
        if self.root_box_fragment_.is_null() {
            return;
        }
        let token = unsafe { &*self.root_box_fragment_ }.GetBreakToken();
        if !token.is_null() {
            self.previously_consumed_block_size_ = unsafe { &*token }.ConsumedBlockSize();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.h:270-301
    pub fn HasRoot(&self) -> bool {
        !self.fragment_items_.is_null()
    }
    pub fn Items(&self) -> &FragmentItems {
        unsafe { &*self.fragment_items_ }
    }
    pub fn ContainerFragment(&self) -> &PhysicalBoxFragment {
        unsafe { &*self.root_box_fragment_ }
    }
    pub fn ContainerFragmentIndex(&self) -> u32 {
        self.fragment_index_
    }
    // cpp: layoutng_fragment_tree/inline_cursor.cc:159-168
    pub fn GetLayoutBlockFlow(&self) -> *const LayoutBlockFlow {
        debug_assert_eq!(self.HasRoot(), !self.root_box_fragment_.is_null());
        assert!(!self.root_box_fragment_.is_null());
        let object = self.ContainerFragment().GetSelfOrContainerLayoutObject();
        debug_assert!(!object.is_null());
        To::<LayoutBlockFlow>(object)
    }
    // cpp: layoutng_fragment_tree/inline_cursor.h:295-301
    pub fn Current(&self) -> &InlineCursorPosition {
        &self.current_
    }
    // cpp: layoutng_fragment_tree/inline_cursor.h:325-330
    pub fn CurrentItem(&self) -> *const FragmentItem {
        self.current_.Item()
    }
    pub fn CurrentText(&self) -> StringView {
        self.Current().Text(self)
    }
    pub fn IsNull(&self) -> bool {
        self.current_.IsNull()
    }
    pub fn IsNotNull(&self) -> bool {
        self.current_.IsNotNull()
    }
    // cpp: layoutng_fragment_tree/inline_cursor.h:495
    pub fn IsBlockFragmented(&self) -> bool {
        self.max_fragment_index_ > 0
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:271-285
    fn IsLastLineInInlineBlock(&self) -> bool {
        debug_assert!(self.Current().IsLineBox());
        if !unsafe { &*self.GetLayoutBlockFlow() }.IsInline() {
            return false;
        }
        let mut next = self.clone();
        loop {
            next.MoveToNextSkippingChildren();
            if next.IsNull() {
                return true;
            }
            if next.Current().IsLineBox() {
                return false;
            }
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:287-312
    pub fn IsBeforeSoftLineBreak(&self) -> bool {
        if self.Current().IsLineBreak() {
            return false;
        }
        let mut line = self.clone();
        line.MoveToContainingLine();
        if line.IsLastLineInInlineBlock() {
            return false;
        }
        let mut last_leaf = line.clone();
        last_leaf.MoveToLastLogicalLeaf();
        if last_leaf != *self {
            return false;
        }
        line.Current().BaseDirection() == self.Current().ResolvedDirection()
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:417-425
    pub fn CurrentLocalRect(&self, start_offset: u32, end_offset: u32) -> PhysicalRect {
        debug_assert!(self.Current().IsText());
        let item = unsafe { &*self.CurrentItem() };
        item.LocalTextRect(item.Text(self.Items()), start_offset, end_offset)
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:431-441
    pub fn CurrentRectInFirstContainerFragment(&self) -> PhysicalRect {
        let mut rect = self.Current().RectInContainerFragment();
        if self.ContainerFragment().IsFirstForNode() {
            return rect;
        }
        let first = unsafe { &*self.ContainerFragment().OwnerLayoutBox() }.GetPhysicalFragment(0);
        rect.offset += self
            .ContainerFragment()
            .OffsetFromRootFragmentationContext()
            - unsafe { &*first }.OffsetFromRootFragmentationContext();
        rect
    }

    // cpp: layoutng_fragment_tree/inline_cursor.h:341-346
    pub fn CurrentOffsetInFirstContainerFragment(&self) -> PhysicalOffset {
        self.CurrentRectInFirstContainerFragment().offset
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:443-450
    pub fn CaretInlinePositionForOffset(&self, offset: u32) -> LayoutUnit {
        debug_assert!(self.Current().IsText());
        let item = unsafe { &*self.CurrentItem() };
        item.CaretInlinePositionForOffset(item.Text(self.Items()), offset)
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:478-486
    fn GetTextOffsetForEndOfLine(&self, last_leaf: &InlineCursor) -> u32 {
        let offset = last_leaf.Current().TextOffset();
        if self.Current().BaseDirection() == last_leaf.Current().ResolvedDirection()
            && !last_leaf.Current().IsLineBreak()
        {
            offset.end
        } else {
            offset.start
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:488-496
    fn SpanBeginItemIndex(&self) -> usize {
        debug_assert!(self.HasRoot() && self.items_len_ != 0);
        debug_assert!(self.Items().IsSubSpan(self.items()));
        let start = self.Items().Items().as_ptr().addr();
        let bytes = self.items_data_.addr() - start;
        let index = bytes / std::mem::size_of::<FragmentItem>();
        debug_assert!(index < self.Items().Items().len());
        index
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:1376-1384
    pub fn CheckValid(&self, position: &InlineCursorPosition) {
        #[cfg(debug_assertions)]
        if position.IsNotNull() {
            debug_assert!(self.HasRoot());
            debug_assert!(position.item_index_ < self.items_len_);
            debug_assert_eq!(position.Item(), unsafe {
                self.items_data_.add(position.item_index_)
            });
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.h:557-566
    fn IsDescendantsCursor(&self) -> bool {
        self.HasRoot() && !self.Items().Equals(self.items())
    }

    fn CanMoveAcrossFragmentainer(&self) -> bool {
        !self.root_block_flow_.is_null() && self.HasRoot() && !self.IsDescendantsCursor()
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:1228-1241
    // cpp: layoutng_fragment_tree/inline_cursor.cc:1228-1240
    fn DecrementFragmentIndex(&mut self) {
        debug_assert!(self.fragment_index_ != 0);
        self.fragment_index_ -= 1;
        self.previously_consumed_block_size_ = LayoutUnit::default();
        if self.fragment_index_ == 0 {
            return;
        }
        let fragment = unsafe { &*self.root_block_flow_ }
            .GetPhysicalFragment((self.fragment_index_ - 1) as usize);
        let token = unsafe { &*fragment }.GetBreakToken();
        if !token.is_null() {
            self.previously_consumed_block_size_ = unsafe { &*token }.ConsumedBlockSize();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:892-902
    pub fn MoveToPreviousFragmentainer(&mut self) {
        debug_assert!(self.CanMoveAcrossFragmentainer());
        if self.fragment_index_ != 0 {
            self.DecrementFragmentIndex();
            if self.TrySetRootFragmentItems() {
                self.MoveToItem(self.items_len_ - 1);
                return;
            }
        }
        self.MakeNull();
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:904-908
    pub fn MoveToPreviousIncludingFragmentainer(&mut self) {
        self.MoveToPrevious();
        if self.IsNull() && self.max_fragment_index_ != 0 && self.CanMoveAcrossFragmentainer() {
            self.MoveToPreviousFragmentainer();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:910-919
    pub fn MoveToFirstIncludingFragmentainer(&mut self) {
        if self.fragment_index_ == 0 {
            self.MoveToFirst();
            return;
        }
        self.ResetFragmentIndex();
        if !self.TrySetRootFragmentItems() {
            self.MakeNull();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:921-929
    pub fn MoveToNextFragmentainer(&mut self) {
        debug_assert!(self.CanMoveAcrossFragmentainer());
        if self.fragment_index_ < self.max_fragment_index_ {
            self.IncrementFragmentIndex();
            if self.TrySetRootFragmentItems() {
                return;
            }
        }
        self.MakeNull();
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:931-935
    pub fn MoveToNextIncludingFragmentainer(&mut self) {
        self.MoveToNext();
        if self.IsNull() && self.max_fragment_index_ != 0 && self.CanMoveAcrossFragmentainer() {
            self.MoveToNextFragmentainer();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:176-189
    pub fn CursorForDescendants(&self) -> Self {
        if self.IsNull() {
            panic!("cursor has no current item");
        }
        let count = unsafe { &*self.CurrentItem() }.DescendantsCount() as usize;
        if count <= 1 {
            return Self::default();
        }
        let start = self.current_.item_index_ + 1;
        let items = &self.items()[start..start + count - 1];
        let mut result = Self::default();
        result.SetRootSpan(self.ContainerFragment(), self.Items(), items);
        result
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:191-215
    pub fn CursorForMovingAcrossFragmentainer(&self) -> Self {
        debug_assert!(self.IsNotNull());
        if self.IsBlockFragmented() {
            return self.clone();
        }
        let mut cursor = Self::from_container(unsafe { &*self.GetLayoutBlockFlow() });
        let item = unsafe { &*self.CurrentItem() };
        while cursor.IsNotNull() && !cursor.TryMoveToFragmentItem(item) {
            cursor.MoveToNextFragmentainer();
        }
        debug_assert!(cursor.IsNotNull());
        cursor
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:203-215
    pub fn ExpandRootToContainingBlock(&mut self) {
        assert!(self.HasRoot());
        let index_diff = self.SpanBeginItemIndex();
        let item_index = self.current_.item_index_;
        let all = unsafe { &*self.fragment_items_ }.Items();
        self.items_data_ = all.as_ptr();
        self.items_len_ = all.len();
        self.MoveToItem(index_diff + item_index);
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:539-562
    pub fn MoveToParent(&mut self) {
        if self.IsNull() {
            return;
        }
        let mut count = 0;
        loop {
            self.MoveToPrevious();
            if self.IsNull() {
                return;
            }
            count += 1;
            if unsafe { &*self.CurrentItem() }.DescendantsCount() > count {
                return;
            }
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:554-562
    pub fn MoveToContainingLine(&mut self) {
        debug_assert!(!self.Current().IsLineBox());
        assert!(self.IsNotNull());
        while self.IsNotNull() && !self.Current().IsLineBox() {
            self.MoveToPrevious();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:498-501
    pub fn MoveToPosition(&mut self, position: InlineCursorPosition) {
        self.CheckValid(&position);
        self.current_ = position;
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:513-528
    pub fn MoveToFragmentItem(&mut self, item: &FragmentItem) {
        assert!(self.TryMoveToFragmentItem(item));
    }

    fn TryMoveToFragmentItem(&mut self, item: &FragmentItem) -> bool {
        debug_assert!(self.HasRoot());
        if self.items_len_ == 0 {
            return false;
        }
        let address = (item as *const FragmentItem).addr();
        let start = self.items_data_.addr();
        let item_size = std::mem::size_of::<FragmentItem>();
        if address < start {
            return false;
        }
        let bytes = address - start;
        if bytes % item_size != 0 || bytes / item_size >= self.items_len_ {
            return false;
        }
        self.MoveToItem(bytes / item_size);
        true
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:530-537
    pub fn MoveToCursor(&mut self, cursor: &Self) {
        if cursor.IsNotNull() {
            if self.fragment_items_.is_null() {
                self.SetRoot(cursor.ContainerFragment(), cursor.Items());
            }
            self.MoveToFragmentItem(unsafe { &*cursor.CurrentItem() });
        } else {
            *self = cursor.clone();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:937-1107
    pub fn MoveToLayoutObject(&mut self, layout_object: &LayoutObject) {
        debug_assert!(layout_object.IsInLayoutNGInlineFormattingContext());
        if layout_object.IsOutOfFlowPositioned() {
            self.MakeNull();
            return;
        }
        let is_descendants_cursor = if !self.HasRoot() {
            let root = layout_object.FragmentItemsContainer();
            debug_assert!(!root.is_null());
            self.SetRootBlockFlow(unsafe { &*root });
            if !self.HasRoot() {
                self.MakeNull();
                return;
            }
            false
        } else {
            self.IsDescendantsCursor()
        };
        let mut index = u32::try_from(layout_object.FirstInlineFragmentItemIndex())
            .expect("inline fragment item index exceeds wtf_size_t");
        if index == 0 {
            self.MakeNull();
            return;
        }
        index -= 1;
        debug_assert_eq!(is_descendants_cursor, self.IsDescendantsCursor());
        if !self.root_block_flow_.is_null() {
            while index >= self.Items().EndItemIndex() {
                self.MoveToNextFragmentainer();
                if self.IsNull() {
                    return;
                }
            }
            debug_assert!(index >= self.Items().SizeOfEarlierFragments());
            index -= self.Items().SizeOfEarlierFragments();
        } else {
            if self.Items().HasItemIndex(index) {
                index -= self.Items().SizeOfEarlierFragments();
            } else {
                let mut cursor = InlineCursor::default();
                cursor.MoveToLayoutObject(layout_object);
                loop {
                    if cursor.IsNull()
                        || cursor.Items().SizeOfEarlierFragments()
                            > self.Items().SizeOfEarlierFragments()
                    {
                        self.MakeNull();
                        return;
                    }
                    if cursor.fragment_items_ == self.fragment_items_ {
                        let start = self.Items().Items().as_ptr().addr();
                        index = ((cursor.CurrentItem().addr() - start)
                            / std::mem::size_of::<FragmentItem>())
                            as u32;
                        break;
                    }
                    cursor.MoveToNextForSameLayoutObject();
                }
            }
            if is_descendants_cursor {
                let span_begin = self.SpanBeginItemIndex() as u32;
                while index < span_begin {
                    let delta =
                        self.Items().Items()[index as usize].DeltaToNextForSameLayoutObject();
                    if delta == 0 {
                        self.MakeNull();
                        return;
                    }
                    index += delta;
                }
                if index >= span_begin + self.items_len_ as u32 {
                    self.MakeNull();
                    return;
                }
                index -= span_begin;
            }
        }
        debug_assert!((index as usize) < self.items_len_);
        self.MoveToItem(index as usize);
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:937-940
    fn SlowMoveToForIfNeeded(&mut self, object: &LayoutObject) {
        while self.IsNotNull() && self.Current().GetLayoutObject() != object as *const _ {
            self.MoveToNextIncludingFragmentainer();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:942-945
    fn SlowMoveToFirstFor(&mut self, object: &LayoutObject) {
        self.MoveToFirstIncludingFragmentainer();
        self.SlowMoveToForIfNeeded(object);
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:947-951
    fn SlowMoveToNextForSameLayoutObject(&mut self, object: &LayoutObject) {
        self.MoveToNextIncludingFragmentainer();
        self.SlowMoveToForIfNeeded(object);
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:1065-1092
    fn MoveToNextForSameLayoutObjectExceptCulledInline(&mut self) {
        if self.IsNull() {
            return;
        }
        let mut delta = unsafe { &*self.CurrentItem() }.DeltaToNextForSameLayoutObject() as usize;
        if delta != 0 {
            loop {
                let delta_to_end = self.items_len_ - self.current_.item_index_;
                if delta < delta_to_end {
                    self.MoveToItem(self.current_.item_index_ + delta);
                    return;
                }
                if !self.CanMoveAcrossFragmentainer() {
                    break;
                }
                self.MoveToNextFragmentainer();
                assert!(self.IsNotNull());
                delta -= delta_to_end;
            }
        }
        self.MakeNull();
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:1270-1276
    pub fn MoveToNextForSameLayoutObject(&mut self) {
        if !self.culled_inline_.IsActive() {
            self.MoveToNextForSameLayoutObjectExceptCulledInline();
        } else {
            self.MoveToNextForCulledInline();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:1094-1103
    pub fn MoveToLastForSameLayoutObject(&mut self) {
        if self.IsNull() {
            return;
        }
        let mut last;
        loop {
            last = self.current_;
            self.MoveToNextForSameLayoutObject();
            if self.IsNull() {
                break;
            }
        }
        self.MoveToPosition(last);
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:1166-1221
    fn MoveToFirstForCulledInline(&mut self, inline: &LayoutInline) {
        if self.IsDescendantsCursor() {
            self.culled_inline_.SetUseFragmentTree(inline);
            debug_assert!(!self.CanMoveAcrossFragmentainer());
            self.MoveToFirst();
            while self.IsNotNull() && !self.Current().IsPartOfCulledInlineBox(inline) {
                self.MoveToNext();
            }
            return;
        }
        let object = self.culled_inline_.MoveToFirstFor(inline);
        if !object.is_null() {
            self.MoveToLayoutObject(unsafe { &*object });
            self.MoveToNextCulledInlineDescendantIfNeeded();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:1193-1209
    fn MoveToNextForCulledInline(&mut self) {
        debug_assert!(self.culled_inline_.IsActive());
        if self.culled_inline_.UseFragmentTree() {
            let inline = unsafe { &*self.culled_inline_.GetLayoutInline() };
            debug_assert!(!self.CanMoveAcrossFragmentainer());
            loop {
                self.MoveToNext();
                if self.IsNull() || self.Current().IsPartOfCulledInlineBox(inline) {
                    break;
                }
            }
            return;
        }
        self.MoveToNextForSameLayoutObjectExceptCulledInline();
        self.MoveToNextCulledInlineDescendantIfNeeded();
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:1211-1221
    fn MoveToNextCulledInlineDescendantIfNeeded(&mut self) {
        debug_assert!(self.culled_inline_.IsActive());
        if self.IsNotNull() {
            return;
        }
        loop {
            let object = self.culled_inline_.MoveToNext();
            if object.is_null() {
                break;
            }
            self.MoveToLayoutObject(unsafe { &*object });
            if self.IsNotNull() {
                break;
            }
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:1253-1330
    pub fn MoveToIncludingCulledInline(&mut self, object: &LayoutObject) {
        debug_assert!(object.IsInLayoutNGInlineFormattingContext());
        self.culled_inline_.Reset();
        self.MoveToLayoutObject(object);
        if self.IsNotNull() || !self.HasRoot() {
            return;
        }
        let inline = DynamicTo::<LayoutInline>(object as *const LayoutObject);
        if !inline.is_null() && !unsafe { &*inline }.ShouldCreateBoxFragment() {
            self.MoveToFirstForCulledInline(unsafe { &*inline });
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:1278-1283
    pub fn MoveToVisualLastForSameLayoutObject(&mut self) {
        if self.culled_inline_.IsActive() {
            self.MoveToVisualFirstOrLastForCulledInline(true);
        } else {
            self.MoveToLastForSameLayoutObject();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:1285-1288
    pub fn MoveToVisualFirstForSameLayoutObject(&mut self) {
        if self.culled_inline_.IsActive() {
            self.MoveToVisualFirstOrLastForCulledInline(false);
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:1290-1321
    fn MoveToVisualFirstOrLastForCulledInline(&mut self, last: bool) {
        let mut found = None;
        let mut found_index = None;
        let mut found_fragment_index = 0;
        while self.IsNotNull() {
            let start = self.Items().Items().as_ptr().addr();
            let index = (self.CurrentItem().addr() - start) / std::mem::size_of::<FragmentItem>();
            if found_index.map_or(true, |previous| {
                (last && index > previous) || (!last && index < previous)
            }) {
                found = Some(self.current_);
                found_index = Some(index);
                found_fragment_index = self.fragment_index_;
                if (last && index == self.Items().Items().len() - 1) || (!last && index == 0) {
                    break;
                }
            }
            self.MoveToNextForSameLayoutObject();
        }
        let position = found.expect("culled inline must have a fragment");
        let changed_fragmentainer = self.fragment_index_ > found_fragment_index;
        while self.fragment_index_ > found_fragment_index {
            self.DecrementFragmentIndex();
        }
        if changed_fragmentainer {
            assert!(self.TrySetRootFragmentItems());
        }
        self.MoveToPosition(position);
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:564-569
    pub fn IsAtFirst(&self) -> bool {
        self.IsNotNull() && self.current_.item_index_ == 0
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:571-577
    pub fn MoveToFirst(&mut self) {
        assert!(self.HasRoot());
        self.MoveToItem(0);
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:585-597
    pub fn MoveToFirstLine(&mut self) {
        assert!(self.HasRoot());
        let index = self
            .items()
            .iter()
            .position(|item| item.Type() == FragmentItemType::kLine);
        if let Some(index) = index {
            self.MoveToItem(index);
        } else {
            self.MakeNull();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:599-612
    pub fn MoveToFirstLogicalLeaf(&mut self) {
        debug_assert!(self.Current().IsLineBox());
        if self.Current().Style().Direction() == TextDirection::kLtr {
            while self.TryMoveToFirstChild() {}
        } else {
            while self.TryMoveToLastChild() {}
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:614-650
    pub fn MoveToFirstNonPseudoLeaf(&mut self) {
        let mut cursor = self.clone();
        while cursor.IsNotNull() {
            if cursor.Current().IsLineBox() {
                cursor.MoveToNext();
                continue;
            }
            let item = unsafe { &*cursor.CurrentItem() };
            if item.IsBlockInInline() {
                if !item.BlockInInline().NonPseudoNode().is_null() {
                    *self = cursor;
                    return;
                }
                cursor.MoveToNext();
                continue;
            }
            let object = cursor.Current().GetLayoutObject();
            if object.is_null() || unsafe { &*object }.NonPseudoNode().is_null() {
                cursor.MoveToNext();
                continue;
            }
            if cursor.Current().IsText() {
                if cursor.Current().IsLayoutGeneratedText() {
                    cursor.MoveToNext();
                    continue;
                }
                if cursor.Current().IsLineBreak() {
                    let mut next = cursor.clone();
                    next.MoveToNext();
                    if next.IsNotNull() {
                        cursor.MoveToNext();
                        continue;
                    }
                }
                *self = cursor;
                return;
            }
            if cursor.Current().IsInlineLeaf() {
                *self = cursor;
                return;
            }
            cursor.MoveToNext();
        }
        self.MakeNull();
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:652-656
    pub fn MoveToLastChild(&mut self) {
        debug_assert!(self.Current().CanHaveChildren());
        if !self.TryMoveToLastChild() {
            self.MakeNull();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:658-666
    pub fn MoveToLastLine(&mut self) {
        debug_assert!(self.HasRoot());
        let index = self
            .items()
            .iter()
            .rposition(|item| item.Type() == FragmentItemType::kLine);
        if let Some(index) = index {
            self.MoveToItem(index);
        } else {
            self.MakeNull();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:668-681
    pub fn MoveToLastLogicalLeaf(&mut self) {
        debug_assert!(self.Current().IsLineBox());
        if self.Current().Style().Direction() == TextDirection::kLtr {
            while self.TryMoveToLastChild() {}
        } else {
            while self.TryMoveToFirstChild() {}
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:683-733
    pub fn MoveToLastNonPseudoLeaf(&mut self) {
        let mut last_leaf = InlineCursor::default();
        let mut hidden_for_paint = false;
        let mut cursor = self.clone();
        while cursor.IsNotNull() {
            if cursor.Current().IsLineBox() {
                cursor.MoveToNext();
                continue;
            }
            let item = unsafe { &*cursor.CurrentItem() };
            if item.IsBlockInInline() {
                if !item.BlockInInline().NonPseudoNode().is_null() {
                    last_leaf = cursor.clone();
                }
                cursor.MoveToNext();
                continue;
            }
            let object = cursor.Current().GetLayoutObject();
            if object.is_null() || unsafe { &*object }.NonPseudoNode().is_null() {
                cursor.MoveToNext();
                continue;
            }
            if cursor.Current().IsLineBreak() && last_leaf.IsNotNull() {
                break;
            }
            if cursor.Current().IsText() {
                if cursor.Current().IsLayoutGeneratedText() {
                    break;
                }
                if hidden_for_paint && !cursor.Current().IsHiddenForPaint() {
                    break;
                }
                hidden_for_paint = cursor.Current().IsHiddenForPaint();
                if !IsBidiControl(cursor.Current().Text(&cursor)) {
                    last_leaf = cursor.clone();
                }
                cursor.MoveToNext();
                continue;
            }
            if cursor.Current().IsInlineLeaf() {
                last_leaf = cursor.clone();
            }
            cursor.MoveToNext();
        }
        *self = last_leaf;
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:735-740
    pub fn MoveToNextInlineLeaf(&mut self) {
        if self.IsNotNull() && self.Current().IsInlineLeaf() {
            self.MoveToNext();
        }
        while self.IsNotNull() && !self.Current().IsInlineLeaf() {
            self.MoveToNext();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:742-746
    pub fn MoveToNextInlineLeafIgnoringLineBreak(&mut self) {
        loop {
            self.MoveToNextInlineLeaf();
            if self.IsNull() || !self.Current().IsLineBreak() {
                break;
            }
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:748-772
    pub fn MoveToNextInlineLeafOnLine(&mut self) {
        self.MoveToLastForSameLayoutObject();
        if self.IsNull() {
            return;
        }
        let last_item = self.clone();
        self.MoveToContainingLine();
        let mut cursor = self.CursorForDescendants();
        cursor.MoveToCursor(&last_item);
        if cursor.Current().IsInlineLeaf() {
            cursor.MoveToNextInlineLeaf();
        } else {
            cursor.MoveToNextSkippingChildren();
            if cursor.IsNotNull() && !cursor.Current().IsInlineLeaf() {
                cursor.MoveToNextInlineLeaf();
            }
        }
        self.MoveToCursor(&cursor);
        debug_assert!(self.IsNull() || self.Current().IsInlineLeaf());
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:774-783
    pub fn MoveToNextLine(&mut self) {
        debug_assert!(self.Current().IsLineBox());
        loop {
            self.MoveToNextSkippingChildren();
            if self.IsNull() || self.Current().IsLineBox() {
                break;
            }
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:785-792
    pub fn MoveToNextLineIncludingFragmentainer(&mut self) {
        self.MoveToNextLine();
        if self.IsNull() && self.max_fragment_index_ != 0 && self.CanMoveAcrossFragmentainer() {
            self.MoveToNextFragmentainer();
            if self.IsNotNull() && !self.Current().IsLineBox() {
                self.MoveToFirstLine();
            }
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:794-799
    pub fn MoveToPreviousInlineLeaf(&mut self) {
        if self.IsNotNull() && self.Current().IsInlineLeaf() {
            self.MoveToPrevious();
        }
        while self.IsNotNull() && !self.Current().IsInlineLeaf() {
            self.MoveToPrevious();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:801-805
    pub fn MoveToPreviousInlineLeafIgnoringLineBreak(&mut self) {
        loop {
            self.MoveToPreviousInlineLeaf();
            if self.IsNull() || !self.Current().IsLineBreak() {
                break;
            }
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:807-816
    pub fn MoveToPreviousInlineLeafOnLine(&mut self) {
        if self.IsNull() {
            return;
        }
        let first_item = self.clone();
        self.MoveToContainingLine();
        let mut cursor = self.CursorForDescendants();
        cursor.MoveToCursor(&first_item);
        cursor.MoveToPreviousInlineLeaf();
        self.MoveToCursor(&cursor);
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:818-828
    pub fn MoveToPreviousLine(&mut self) {
        debug_assert!(self.Current().IsLineBox());
        loop {
            self.MoveToPrevious();
            if self.IsNull() || self.Current().IsLineBox() {
                break;
            }
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:837-853
    pub fn TryMoveToLastChild(&mut self) -> bool {
        if !self.Current().HasChildren() {
            return false;
        }
        let end =
            self.current_.item_index_ + unsafe { &*self.CurrentItem() }.DescendantsCount() as usize;
        self.MoveToNext();
        debug_assert!(self.IsNotNull());
        loop {
            let previous = self.current_.item_index_;
            debug_assert!(previous < end);
            self.MoveToNextSkippingChildren();
            if self.IsNull() || self.current_.item_index_ == end {
                self.MoveToItem(previous);
                break;
            }
        }
        true
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:830-835
    pub fn TryMoveToFirstChild(&mut self) -> bool {
        if !self.Current().HasChildren() {
            return false;
        }
        self.MoveToItem(self.current_.item_index_ + 1);
        true
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:579-583
    pub fn MoveToFirstChild(&mut self) {
        debug_assert!(self.Current().CanHaveChildren());
        if !self.TryMoveToFirstChild() {
            self.MakeNull();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:855-867
    pub fn MoveToNext(&mut self) {
        debug_assert!(self.HasRoot());
        if self.IsNull() {
            return;
        }
        // MoveToItem performs the C++ end-iterator branch and calls MakeNull.
        self.MoveToItem(self.current_.item_index_ + 1);
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:869-879
    pub fn MoveToNextSkippingChildren(&mut self) {
        debug_assert!(self.HasRoot());
        if self.IsNull() {
            return;
        }
        let descendants = unsafe { &*self.CurrentItem() }.DescendantsCount() as usize;
        if descendants != 0 {
            self.MoveToItem(self.current_.item_index_ + descendants);
        } else {
            self.MoveToNext();
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:881-890
    pub fn MoveToPrevious(&mut self) {
        debug_assert!(self.HasRoot());
        if self.IsNull() {
            return;
        }
        if self.current_.item_index_ == 0 {
            self.MakeNull();
        } else {
            self.MoveToItem(self.current_.item_index_ - 1);
        }
    }

    // cpp: layoutng_fragment_tree/inline_cursor.cc:503-511
    fn MoveToItem(&mut self, index: usize) {
        if index >= self.items_len_ {
            self.MakeNull();
        } else {
            self.current_
                .Set(unsafe { self.items_data_.add(index) }, index);
        }
    }
}

impl PartialEq for InlineCursor {
    // cpp: layoutng_fragment_tree/inline_cursor.h:267
    // cpp: layoutng_fragment_tree/inline_cursor.cc:149-157
    fn eq(&self, other: &Self) -> bool {
        if self.current_ != other.current_ {
            return false;
        }
        debug_assert_eq!(self.items_data_, other.items_data_);
        debug_assert_eq!(self.items_len_, other.items_len_);
        debug_assert_eq!(self.fragment_items_, other.fragment_items_);
        debug_assert_eq!(self.current_.item_index_, other.current_.item_index_);
        true
    }
}

// cpp: layoutng_fragment_tree/inline_cursor.h:676-706
// cpp: layoutng_fragment_tree/inline_cursor.cc:1326-1374
pub struct InlineBackwardCursor<'a> {
    current_: InlineCursorPosition,
    cursor_: &'a InlineCursor,
    sibling_item_indices_: Vec<usize>,
    current_index_: usize,
}

#[allow(non_snake_case)]
impl<'a> InlineBackwardCursor<'a> {
    pub fn new(cursor: &'a InlineCursor) -> Self {
        let mut sibling_item_indices = Vec::new();
        if cursor.HasRoot() {
            debug_assert!(cursor.IsNull() || cursor.current_.item_index_ == 0);
            let mut sibling = cursor.clone();
            while sibling.IsNotNull() {
                sibling_item_indices.push(sibling.current_.item_index_);
                sibling.MoveToNextSkippingChildren();
            }
        } else {
            debug_assert!(cursor.IsNull());
        }
        let current_index = sibling_item_indices.len().saturating_sub(1);
        let current = if sibling_item_indices.is_empty() {
            InlineCursorPosition::default()
        } else {
            let index = sibling_item_indices[current_index];
            InlineCursorPosition {
                item_: unsafe { cursor.items_data_.add(index) },
                item_index_: index,
            }
        };
        Self {
            current_: current,
            cursor_: cursor,
            sibling_item_indices_: sibling_item_indices,
            current_index_: current_index,
        }
    }

    pub fn Current(&self) -> &InlineCursorPosition {
        &self.current_
    }

    pub fn IsNotNull(&self) -> bool {
        self.current_.IsNotNull()
    }

    pub fn CursorForDescendants(&self) -> InlineCursor {
        assert!(self.IsNotNull());
        let mut cursor = self.cursor_.clone();
        cursor.MoveToItem(self.sibling_item_indices_[self.current_index_]);
        cursor.CursorForDescendants()
    }

    pub fn MoveToPreviousSibling(&mut self) {
        if self.current_index_ == 0 {
            self.current_.Clear();
            return;
        }
        assert!(self.IsNotNull());
        self.current_index_ -= 1;
        let index = self.sibling_item_indices_[self.current_index_];
        self.current_
            .Set(unsafe { self.cursor_.items_data_.add(index) }, index);
    }

    pub fn ContainerFragment(&self) -> &PhysicalBoxFragment {
        self.cursor_.ContainerFragment()
    }
}
