use std::cell::{Cell, UnsafeCell};
use std::mem::ManuallyDrop;
use foundation::graphics_types;

use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use font_engine::{AdjustMidCluster, Font, FontHeight, ShapeResultView, TextFragmentPaintInfo};
use foundation::{
    gfx, DynamicTo, IsHorizontalWritingMode, LayoutUnit, MakeGarbageCollected, Member,
    PhysicalOffset, PhysicalRect, PhysicalSize, String, StringBuilder, StringView, TextDirection,
    To, Visitor, WritingMode,
};
use graphics_types::graphics::paint::display_item_client::DisplayItemClient;
use layoutng::internal::ink_overflow::{InkOverflow, InkOverflowType};
use layoutng::internal::layout_node_metadata::Node;
use layoutng::internal::inline_item::InlineItem;
use layoutng::internal::layout_block_flow::LayoutBlockFlow;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_text::LayoutText;
use layoutng::internal::style_variant::StyleVariant;
use layoutng::internal::text_fit_scale::TextFitScale;
use layoutng::internal::text_item_type::TextItemType;
use layoutng::internal::text_offset_range::TextOffsetRange;
use layoutng::internal::used_font::UsedFont;
use layoutng_geometry::geometry::logical_size::ToPhysicalSize;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::inline_break_token::InlineBreakToken;
use crate::inline_cursor::InlineCursor;
use crate::logical_line_item::LogicalLineItem;
use crate::physical_box_fragment::PhysicalBoxFragment;
use crate::physical_line_box_fragment::{LineBoxType, PhysicalLineBoxFragment};

// cpp: layoutng_fragment_tree/fragment_item.h:35-52
pub struct TextFragmentRareData {
    pub rect: gfx::RectF,
    pub length_adjust_scale: f32,
    pub angle: f32,
    pub baseline_shift: f32,
    pub scaled_font: Member<Font>,
    pub in_text_path: bool,
    pub annotation_metrics: FontHeight,
    pub is_svg: bool,
}

impl Default for TextFragmentRareData {
    fn default() -> Self {
        Self {
            rect: gfx::RectF::default(),
            length_adjust_scale: 1.0,
            angle: 0.0,
            baseline_shift: 0.0,
            scaled_font: Member::default(),
            in_text_path: false,
            annotation_metrics: FontHeight::default(),
            is_svg: false,
        }
    }
}

#[allow(non_snake_case)]
impl TextFragmentRareData {
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.scaled_font);
    }
}

// cpp: layoutng_fragment_tree/fragment_item.h:64-76
#[derive(Clone)]
pub struct TextItem {
    pub shape_result: Member<ShapeResultView>,
    pub rare_data: Member<TextFragmentRareData>,
    pub text_offset: TextOffsetRange,
}

#[allow(non_snake_case)]
impl TextItem {
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.shape_result);
        visitor.Trace(&self.rare_data);
    }
}

// cpp: layoutng_fragment_tree/fragment_item.h:78-90
#[derive(Clone)]
pub struct GeneratedTextItem {
    pub shape_result: Member<ShapeResultView>,
    pub rare_data: Member<TextFragmentRareData>,
    pub text: String,
}

#[allow(non_snake_case)]
impl GeneratedTextItem {
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.shape_result);
        visitor.Trace(&self.rare_data);
    }
}

// cpp: layoutng_fragment_tree/fragment_item.h:92-101
#[derive(Clone)]
pub struct LineItem {
    pub line_box_fragment: Member<PhysicalLineBoxFragment>,
    pub descendants_count: u32,
    pub text_fit_scale: f32,
}

#[allow(non_snake_case)]
impl LineItem {
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.line_box_fragment);
    }
}

// cpp: layoutng_fragment_tree/fragment_item.h:103-119
pub struct BoxItem {
    pub box_fragment: UnsafeCell<Member<PhysicalBoxFragment>>,
    pub descendants_count: u32,
}

impl Clone for BoxItem {
    fn clone(&self) -> Self {
        Self {
            box_fragment: UnsafeCell::new(unsafe { &*self.box_fragment.get() }.clone()),
            descendants_count: self.descendants_count,
        }
    }
}

#[allow(non_snake_case)]
impl BoxItem {
    // cpp: layoutng_fragment_tree/fragment_item.cc:489-491
    pub fn new(box_fragment: *const PhysicalBoxFragment, descendants_count: u32) -> Self {
        Self {
            box_fragment: UnsafeCell::new(Member::from_ptr(box_fragment as *mut _)),
            descendants_count,
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:493-495
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(unsafe { &*self.box_fragment.get() });
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:497-501
    pub fn PostLayout(&self) -> *const PhysicalBoxFragment {
        let box_fragment = unsafe { &*self.box_fragment.get() }.Get();
        if box_fragment.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*box_fragment }.PostLayout()
        }
    }
}

// cpp: layoutng_fragment_tree/fragment_item.h:121-132
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemType {
    kInvalid = 0,
    kText = 1,
    kGeneratedText = 2,
    kLine = 3,
    kBox = 4,
}

impl ItemType {
    pub const kMaxValue: Self = Self::kBox;
}

pub type FragmentItemType = ItemType;

#[repr(C)]
union FragmentItemPayload {
    text_: ManuallyDrop<TextItem>,
    generated_text_: ManuallyDrop<GeneratedTextItem>,
    line_: ManuallyDrop<LineItem>,
    box_: ManuallyDrop<BoxItem>,
}

// The active union arm is selected by the low three bits of flags_. C++ uses
// one unsigned bitfield storage unit for these flags.
// cpp: layoutng_fragment_tree/fragment_item.h:645-682
#[repr(C)]
pub struct FragmentItem {
    payload_: FragmentItemPayload,
    rect_: PhysicalRect,
    ink_overflow_: UnsafeCell<InkOverflow>,
    layout_object_: UnsafeCell<Member<LayoutObject>>,
    fragment_id_: Cell<u32>,
    delta_to_next_for_same_layout_object_: Cell<u32>,
    flags_: Cell<u32>,
}

#[repr(C)]
struct SameSizeAsFragmentItem {
    payload: FragmentItemPayload,
    rect: PhysicalRect,
    ink_overflow: InkOverflow,
    member: Member<LayoutObject>,
    sizes: [u32; 2],
    flags: u32,
}

// The C++ painting/cloning façade narrows mutations through a const item.
// Only the box-fragment Member needs interior mutability for cloning.
// cpp: layoutng_fragment_tree/fragment_item.h:377-392
pub struct MutableForCloning<'a> {
    item_: &'a FragmentItem,
}

// cpp: layoutng_fragment_tree/fragment_item.h:353-371
pub struct MutableForPainting<'a> {
    item_: &'a FragmentItem,
}

#[allow(non_snake_case)]
impl MutableForPainting<'_> {
    pub fn InvalidateInkOverflow(&self) {
        self.item_.InvalidateInkOverflow();
    }

    // RecalcInkOverflow is declared by the supplied header but its C++ body
    // is absent from all supplied source packages; see translation status.
}

#[allow(non_snake_case)]
impl MutableForCloning<'_> {
    pub fn ReplaceBoxFragment(&self, new_fragment: &PhysicalBoxFragment) {
        debug_assert!(!self.item_.BoxFragment().is_null());
        let box_item = self.item_.box_item();
        unsafe {
            *box_item.box_fragment.get() =
                Member::from_ptr(new_fragment as *const PhysicalBoxFragment as *mut _);
        }
    }
}

// cpp: layoutng_fragment_tree/fragment_item.cc:21-39
const _: () =
    assert!(std::mem::size_of::<FragmentItem>() == std::mem::size_of::<SameSizeAsFragmentItem>());

#[allow(non_snake_case)]
impl FragmentItem {
    pub const kInvalid: ItemType = ItemType::kInvalid;
    pub const kText: ItemType = ItemType::kText;
    pub const kGeneratedText: ItemType = ItemType::kGeneratedText;
    pub const kLine: ItemType = ItemType::kLine;
    pub const kBox: ItemType = ItemType::kBox;
    pub const kInitialLineFragmentId: u32 = 0x8000_0000;

    const TYPE_BITS: u32 = 0;
    const SUB_TYPE_BITS: u32 = 3;
    const STYLE_VARIANT_BITS: u32 = 6;
    const HIDDEN_BIT: u32 = 8;
    const TEXT_DIRECTION_BIT: u32 = 9;
    const OVER_ANNOTATION_BIT: u32 = 10;
    const UNDER_ANNOTATION_BIT: u32 = 11;
    const INK_OVERFLOW_BITS: u32 = 12;
    const DIRTY_BIT: u32 = 15;
    const LAST_FOR_NODE_BIT: u32 = 16;

    fn bit(&self, bit: u32) -> bool {
        self.flags_.get() & (1 << bit) != 0
    }

    fn set_bit(&self, bit: u32, value: bool) {
        let old = self.flags_.get();
        self.flags_
            .set((old & !(1 << bit)) | ((value as u32) << bit));
    }

    fn bits(&self, shift: u32, width: u32) -> u32 {
        (self.flags_.get() >> shift) & ((1 << width) - 1)
    }

    fn set_bits(&self, shift: u32, width: u32, value: u32) {
        let mask = ((1 << width) - 1) << shift;
        self.flags_
            .set((self.flags_.get() & !mask) | ((value << shift) & mask));
    }

    fn initial_flags(
        item_type: ItemType,
        sub_type: u32,
        style_variant: StyleVariant,
        hidden: bool,
        direction: TextDirection,
    ) -> Cell<u32> {
        let value = (item_type as u32)
            | ((sub_type & 7) << Self::SUB_TYPE_BITS)
            | (((style_variant as u32) & 3) << Self::STYLE_VARIANT_BITS)
            | ((hidden as u32) << Self::HIDDEN_BIT)
            | ((direction as u32) << Self::TEXT_DIRECTION_BIT)
            | ((InkOverflowType::kNotSet as u32) << Self::INK_OVERFLOW_BITS)
            | (1 << Self::LAST_FOR_NODE_BIT);
        Cell::new(value)
    }

    fn text_item(&self) -> &TextItem {
        debug_assert_eq!(self.Type(), ItemType::kText);
        unsafe { &self.payload_.text_ }
    }

    fn text_item_mut(&mut self) -> &mut TextItem {
        debug_assert_eq!(self.Type(), ItemType::kText);
        unsafe { &mut self.payload_.text_ }
    }

    fn generated_text_item(&self) -> &GeneratedTextItem {
        debug_assert_eq!(self.Type(), ItemType::kGeneratedText);
        unsafe { &self.payload_.generated_text_ }
    }

    fn generated_text_item_mut(&mut self) -> &mut GeneratedTextItem {
        debug_assert_eq!(self.Type(), ItemType::kGeneratedText);
        unsafe { &mut self.payload_.generated_text_ }
    }

    fn line_item(&self) -> &LineItem {
        debug_assert_eq!(self.Type(), ItemType::kLine);
        unsafe { &self.payload_.line_ }
    }

    fn line_item_mut(&mut self) -> &mut LineItem {
        debug_assert_eq!(self.Type(), ItemType::kLine);
        unsafe { &mut self.payload_.line_ }
    }

    fn box_item(&self) -> &BoxItem {
        debug_assert_eq!(self.Type(), ItemType::kBox);
        unsafe { &self.payload_.box_ }
    }

    fn box_item_mut(&mut self) -> &mut BoxItem {
        debug_assert_eq!(self.Type(), ItemType::kBox);
        unsafe { &mut self.payload_.box_ }
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:591-596
    // cpp: layoutng_fragment_tree/fragment_item.cc:42-73
    fn from_text(
        inline_item: &InlineItem,
        shape_result: *const ShapeResultView,
        text_offset: TextOffsetRange,
        size: PhysicalSize,
        hidden: bool,
    ) -> Self {
        let result = Self {
            payload_: FragmentItemPayload {
                text_: ManuallyDrop::new(TextItem {
                    shape_result: Member::from_ptr(shape_result as *mut ShapeResultView),
                    rare_data: Member::default(),
                    text_offset,
                }),
            },
            rect_: PhysicalRect::new(PhysicalOffset::default(), size),
            ink_overflow_: UnsafeCell::new(InkOverflow::default()),
            layout_object_: UnsafeCell::new(Member::from_ptr(
                inline_item.GetLayoutObject() as *mut _
            )),
            fragment_id_: Cell::new(0),
            delta_to_next_for_same_layout_object_: Cell::new(0),
            flags_: Self::initial_flags(
                ItemType::kText,
                inline_item.TextType() as u32,
                inline_item.GetStyleVariant(),
                hidden,
                inline_item.Direction(),
            ),
        };
        if !shape_result.is_null() {
            debug_assert_eq!(unsafe { &*shape_result }.StartIndex(), result.StartOffset());
            debug_assert_eq!(unsafe { &*shape_result }.EndIndex(), result.EndOffset());
        }
        debug_assert_ne!(result.TextType(), TextItemType::kLayoutGenerated);
        debug_assert!(!result.IsFormattingContextRoot());
        result
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:603-610
    // cpp: layoutng_fragment_tree/fragment_item.cc:75-100
    fn from_generated_text(
        layout_object: &LayoutObject,
        text_type: TextItemType,
        style_variant: StyleVariant,
        direction: TextDirection,
        shape_result: *const ShapeResultView,
        text_content: String,
        size: PhysicalSize,
        hidden: bool,
    ) -> Self {
        let result = Self {
            payload_: FragmentItemPayload {
                generated_text_: ManuallyDrop::new(GeneratedTextItem {
                    shape_result: Member::from_ptr(shape_result as *mut ShapeResultView),
                    rare_data: Member::default(),
                    text: text_content,
                }),
            },
            rect_: PhysicalRect::new(PhysicalOffset::default(), size),
            ink_overflow_: UnsafeCell::new(InkOverflow::default()),
            layout_object_: UnsafeCell::new(Member::from_ptr(layout_object as *const _ as *mut _)),
            fragment_id_: Cell::new(0),
            delta_to_next_for_same_layout_object_: Cell::new(0),
            flags_: Self::initial_flags(
                ItemType::kGeneratedText,
                text_type as u32,
                style_variant,
                hidden,
                direction,
            ),
        };
        debug_assert!(!shape_result.is_null());
        debug_assert_eq!(unsafe { &*shape_result }.StartIndex(), result.StartOffset());
        debug_assert_eq!(unsafe { &*shape_result }.EndIndex(), result.EndOffset());
        debug_assert!(!result.IsFormattingContextRoot());
        result
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:597-602
    // cpp: layoutng_fragment_tree/fragment_item.cc:102-112
    fn from_generated_inline_text(
        inline_item: &InlineItem,
        shape_result: *const ShapeResultView,
        text_content: String,
        size: PhysicalSize,
        hidden: bool,
    ) -> Self {
        let layout_object = unsafe { &*inline_item.GetLayoutObject() };
        Self::from_generated_text(
            layout_object,
            inline_item.TextType(),
            inline_item.GetStyleVariant(),
            inline_item.Direction(),
            shape_result,
            text_content,
            size,
            hidden,
        )
    }

    // C++ placement-new selects one of the same constructors. Rust constructs
    // the selected variant directly and keeps its payload active until Drop.
    // cpp: layoutng_fragment_tree/fragment_item.h:135
    // cpp: layoutng_fragment_tree/fragment_item.cc:162-212
    pub fn from_logical_line_item(line_item: LogicalLineItem, writing_mode: WritingMode) -> Self {
        debug_assert!(line_item.CanCreateFragmentItem());
        let size = ToPhysicalSize(line_item.MarginSize(), writing_mode);
        let inline_item = line_item.inline_item.Get();
        if !inline_item.is_null() {
            let inline_item = unsafe { &*inline_item };
            let mut result = if !line_item.text_content.IsNull() {
                Self::from_generated_inline_text(
                    inline_item,
                    line_item.shape_result.Get(),
                    line_item.text_content,
                    size,
                    line_item.is_hidden_for_paint,
                )
            } else {
                Self::from_text(
                    inline_item,
                    line_item.shape_result.Get(),
                    line_item.text_offset,
                    size,
                    line_item.is_hidden_for_paint,
                )
            };
            result.set_bit(Self::OVER_ANNOTATION_BIT, line_item.has_over_annotation);
            result.set_bit(Self::UNDER_ANNOTATION_BIT, line_item.has_under_annotation);
            result.SetTextRareData(line_item.text_fit_scale.Get(), line_item.annotation_metrics);
            return result;
        }
        let layout_result = line_item.layout_result.Get();
        if !layout_result.is_null() {
            let physical = unsafe { &*layout_result }.GetPhysicalFragment();
            let box_fragment = unsafe { &*To::<PhysicalBoxFragment>(physical) };
            return Self::from_box_fragment(box_fragment, line_item.ResolvedDirection());
        }
        let layout_object = line_item.layout_object.Get();
        if !layout_object.is_null() {
            let shape = line_item.shape_result.Get();
            let direction = unsafe { &*shape }.Direction();
            let mut result = Self::from_generated_text(
                unsafe { &*layout_object },
                TextItemType::kLayoutGenerated,
                line_item.style_variant,
                direction,
                shape,
                line_item.text_content,
                size,
                line_item.is_hidden_for_paint,
            );
            result.SetTextRareData(line_item.text_fit_scale.Get(), FontHeight::default());
            return result;
        }
        unreachable!("CanCreateFragmentItem had no constructible variant")
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:140
    // cpp: layoutng_fragment_tree/fragment_item.cc:114-127
    pub fn from_line_fragment(line: &PhysicalLineBoxFragment) -> Self {
        let result = Self {
            payload_: FragmentItemPayload {
                line_: ManuallyDrop::new(LineItem {
                    line_box_fragment: Member::from_ptr(line as *const _ as *mut _),
                    descendants_count: 1,
                    text_fit_scale: 1.0,
                }),
            },
            rect_: PhysicalRect::new(PhysicalOffset::default(), line.Size()),
            ink_overflow_: UnsafeCell::new(InkOverflow::default()),
            layout_object_: UnsafeCell::new(Member::from_ptr(
                line.ContainerLayoutObject() as *mut LayoutObject
            )),
            fragment_id_: Cell::new(0),
            delta_to_next_for_same_layout_object_: Cell::new(0),
            flags_: Self::initial_flags(
                ItemType::kLine,
                line.GetLineBoxType() as u32,
                line.GetStyleVariant(),
                line.IsHiddenForPaint(),
                line.BaseDirection(),
            ),
        };
        debug_assert!(!result.IsFormattingContextRoot());
        result
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:142-143
    // cpp: layoutng_fragment_tree/fragment_item.cc:129-143
    pub fn from_annotation_line(size: PhysicalSize, base_line: &PhysicalLineBoxFragment) -> Self {
        let result = Self {
            payload_: FragmentItemPayload {
                line_: ManuallyDrop::new(LineItem {
                    line_box_fragment: Member::default(),
                    descendants_count: 1,
                    text_fit_scale: 1.0,
                }),
            },
            rect_: PhysicalRect::new(PhysicalOffset::default(), size),
            ink_overflow_: UnsafeCell::new(InkOverflow::default()),
            layout_object_: UnsafeCell::new(Member::from_ptr(
                base_line.ContainerLayoutObject() as *mut LayoutObject
            )),
            fragment_id_: Cell::new(0),
            delta_to_next_for_same_layout_object_: Cell::new(0),
            flags_: Self::initial_flags(
                ItemType::kLine,
                crate::physical_line_box_fragment::LineBoxType::kRubyLineBox as u32,
                base_line.GetStyleVariant(),
                false,
                base_line.BaseDirection(),
            ),
        };
        debug_assert!(!result.IsFormattingContextRoot());
        result
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:137-138
    // cpp: layoutng_fragment_tree/fragment_item.cc:145-160
    pub fn from_box_fragment(box_fragment: &PhysicalBoxFragment, direction: TextDirection) -> Self {
        let result = Self {
            payload_: FragmentItemPayload {
                box_: ManuallyDrop::new(BoxItem::new(box_fragment, 1)),
            },
            rect_: PhysicalRect::new(PhysicalOffset::default(), box_fragment.Size()),
            ink_overflow_: UnsafeCell::new(InkOverflow::default()),
            layout_object_: UnsafeCell::new(Member::from_ptr(
                box_fragment.GetLayoutObject() as *mut LayoutObject
            )),
            fragment_id_: Cell::new(0),
            delta_to_next_for_same_layout_object_: Cell::new(0),
            flags_: Self::initial_flags(
                ItemType::kBox,
                0,
                box_fragment.GetStyleVariant(),
                box_fragment.IsHiddenForPaint(),
                direction,
            ),
        };
        debug_assert_eq!(
            result.IsFormattingContextRoot(),
            box_fragment.IsFormattingContextRoot()
        );
        result
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:151
    pub fn Type(&self) -> ItemType {
        match self.bits(Self::TYPE_BITS, 3) {
            0 => ItemType::kInvalid,
            1 => ItemType::kText,
            2 => ItemType::kGeneratedText,
            3 => ItemType::kLine,
            4 => ItemType::kBox,
            _ => unreachable!("invalid FragmentItem type"),
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:153-168
    pub fn IsText(&self) -> bool {
        matches!(self.Type(), ItemType::kText | ItemType::kGeneratedText)
    }

    pub fn IsContainer(&self) -> bool {
        matches!(self.Type(), ItemType::kBox | ItemType::kLine)
    }

    pub fn IsHiddenForPaint(&self) -> bool {
        self.bit(Self::HIDDEN_BIT)
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:175-209
    pub fn FragmentId(&self) -> u32 {
        debug_assert!(
            self.Type() != ItemType::kLine
                || self.fragment_id_.get() >= Self::kInitialLineFragmentId
        );
        self.fragment_id_.get()
    }

    pub fn SetFragmentId(&self, id: u32) {
        debug_assert!(self.Type() != ItemType::kLine || id >= Self::kInitialLineFragmentId);
        self.fragment_id_.set(id);
    }

    pub fn IsFirstForNode(&self) -> bool {
        self.FragmentId() == 0
    }

    pub fn IsLastForNode(&self) -> bool {
        debug_assert_ne!(self.Type(), ItemType::kLine);
        self.bit(Self::LAST_FOR_NODE_BIT)
    }

    pub fn SetIsLastForNode(&self, is_last: bool) {
        self.set_bit(Self::LAST_FOR_NODE_BIT, is_last);
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:200-223
    pub fn GetStyleVariant(&self) -> StyleVariant {
        match self.bits(Self::STYLE_VARIANT_BITS, 2) {
            0 => StyleVariant::kStandard,
            1 => StyleVariant::kFirstLine,
            2 => StyleVariant::kStandardEllipsis,
            3 => StyleVariant::kFirstLineEllipsis,
            _ => unreachable!(),
        }
    }

    pub fn UsesFirstLineStyle(&self) -> bool {
        matches!(
            self.GetStyleVariant(),
            StyleVariant::kFirstLine | StyleVariant::kFirstLineEllipsis
        )
    }

    pub fn GetLayoutObject(&self) -> *const LayoutObject {
        unsafe { &*self.layout_object_.get() }.Get()
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:212-237
    pub fn Style(&self) -> &ComputedStyle {
        unsafe { &*self.GetLayoutObject() }.EffectiveStyle(self.GetStyleVariant())
    }

    pub fn GetNode(&self) -> *mut Node {
        unsafe { &*self.GetLayoutObject() }.GetNode()
    }

    pub fn NodeForHitTest(&self) -> *mut Node {
        unsafe { &*self.GetLayoutObject() }.NodeForHitTest()
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:225-228
    pub fn GetDisplayItemClient(&self) -> *const DisplayItemClient {
        self.GetLayoutObject() as *const DisplayItemClient
    }

    pub fn IsRelayoutBoundary(&self) -> bool {
        unsafe { &*self.GetLayoutObject() }.IsRelayoutBoundary()
    }

    pub fn GetMutableLayoutObject(&self) -> *mut LayoutObject {
        self.GetLayoutObject() as *mut LayoutObject
    }

    pub fn IsLayoutObjectDestroyedOrMoved(&self) -> bool {
        self.GetLayoutObject().is_null()
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:234-237
    pub fn DeltaToNextForSameLayoutObject(&self) -> u32 {
        self.delta_to_next_for_same_layout_object_.get()
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:947-950
    pub fn SetDeltaToNextForSameLayoutObject(&self, delta: u32) {
        debug_assert_ne!(self.Type(), ItemType::kLine);
        self.delta_to_next_for_same_layout_object_.set(delta);
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:239-268
    pub fn RectInContainerFragment(&self) -> &PhysicalRect {
        &self.rect_
    }
    pub fn OffsetInContainerFragment(&self) -> &PhysicalOffset {
        &self.rect_.offset
    }
    pub fn Size(&self) -> PhysicalSize {
        self.rect_.size
    }
    pub fn LocalRect(&self) -> PhysicalRect {
        PhysicalRect::new(PhysicalOffset::default(), self.rect_.size)
    }
    pub fn SetOffset(&mut self, offset: PhysicalOffset) {
        self.rect_.offset = offset;
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:270-304
    pub fn DescendantsCount(&self) -> u32 {
        match self.Type() {
            ItemType::kBox => self.box_item().descendants_count,
            ItemType::kLine => self.line_item().descendants_count,
            _ => 0,
        }
    }

    pub fn HasChildren(&self) -> bool {
        self.DescendantsCount() > 1
    }

    pub fn SetDescendantsCount(&mut self, count: u32) {
        match self.Type() {
            ItemType::kBox => self.box_item_mut().descendants_count = count,
            ItemType::kLine => self.line_item_mut().descendants_count = count,
            _ => unreachable!("non-container FragmentItem has no descendants"),
        }
    }

    pub fn BoxFragment(&self) -> *const PhysicalBoxFragment {
        if self.Type() == ItemType::kBox {
            unsafe { &*self.box_item().box_fragment.get() }.Get()
        } else {
            std::ptr::null()
        }
    }

    pub fn PostLayoutBoxFragment(&self) -> *const PhysicalBoxFragment {
        if self.Type() == ItemType::kBox {
            self.box_item().PostLayout()
        } else {
            std::ptr::null()
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:394-396
    pub fn GetMutableForCloning(&self) -> MutableForCloning<'_> {
        MutableForCloning { item_: self }
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:373-375
    pub fn GetMutableForPainting(&self) -> MutableForPainting<'_> {
        MutableForPainting { item_: self }
    }

    pub fn LineBoxFragment(&self) -> *const PhysicalLineBoxFragment {
        if self.Type() == ItemType::kLine {
            self.line_item().line_box_fragment.Get()
        } else {
            std::ptr::null()
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:398-404
    pub fn IsHorizontal(&self) -> bool {
        IsHorizontalWritingMode(self.GetWritingMode())
    }

    pub fn GetWritingMode(&self) -> WritingMode {
        self.Style().GetWritingMode()
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:334-342
    pub fn GetLineBoxType(&self) -> LineBoxType {
        debug_assert_eq!(self.Type(), ItemType::kLine);
        match self.bits(Self::SUB_TYPE_BITS, 3) {
            0 => LineBoxType::kNormalLineBox,
            1 => LineBoxType::kEmptyLineBox,
            2 => LineBoxType::kRubyLineBox,
            _ => unreachable!("invalid line box subtype"),
        }
    }

    pub fn IsRubyAnnotationLine(&self) -> bool {
        self.Type() == ItemType::kLine && self.GetLineBoxType() == LineBoxType::kRubyLineBox
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:407-429
    pub fn TextType(&self) -> TextItemType {
        match self.Type() {
            ItemType::kText => match self.bits(Self::SUB_TYPE_BITS, 3) {
                0 => TextItemType::kNormal,
                1 => TextItemType::kForcedLineBreak,
                2 => TextItemType::kFlowControl,
                3 => TextItemType::kSymbolMarker,
                4 => TextItemType::kLayoutGenerated,
                _ => unreachable!("invalid text subtype"),
            },
            ItemType::kGeneratedText => TextItemType::kLayoutGenerated,
            _ => unreachable!("non-text FragmentItem has no text subtype"),
        }
    }

    pub fn IsLineBreak(&self) -> bool {
        self.TextType() == TextItemType::kForcedLineBreak
    }
    pub fn IsFlowControl(&self) -> bool {
        self.IsLineBreak() || self.TextType() == TextItemType::kFlowControl
    }
    pub fn IsEllipsis(&self) -> bool {
        matches!(
            self.GetStyleVariant(),
            StyleVariant::kStandardEllipsis | StyleVariant::kFirstLineEllipsis
        )
    }
    // cpp: layoutng_fragment_tree/fragment_item.h:437-441
    pub fn IsLayoutGeneratedText(&self) -> bool {
        self.Type() == ItemType::kGeneratedText
    }
    pub fn IsSymbolMarker(&self) -> bool {
        self.TextType() == TextItemType::kSymbolMarker
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:443-451
    pub fn StartOffset(&self) -> u32 {
        self.TextOffset().start
    }
    pub fn EndOffset(&self) -> u32 {
        self.TextOffset().end
    }
    pub fn TextLength(&self) -> u32 {
        self.TextOffset().Length()
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:513-514
    pub fn HasOverAnnotation(&self) -> bool {
        self.bit(Self::OVER_ANNOTATION_BIT)
    }
    pub fn HasUnderAnnotation(&self) -> bool {
        self.bit(Self::UNDER_ANNOTATION_BIT)
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:530-532
    pub fn IsDirty(&self) -> bool {
        self.bit(Self::DIRTY_BIT)
    }
    pub fn SetDirty(&self) {
        self.set_bit(Self::DIRTY_BIT, true);
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:539-546
    pub fn GetSvgFragmentData(&self) -> *const TextFragmentRareData {
        if self.Type() != ItemType::kText {
            return std::ptr::null();
        }
        let data = self.text_item().rare_data.Get();
        if !data.is_null() && unsafe { &*data }.is_svg {
            data
        } else {
            std::ptr::null()
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:163-165
    pub fn IsSvgText(&self) -> bool {
        !self.GetSvgFragmentData().is_null()
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:577-578
    pub fn SetLineTextFitScale(&mut self, scale: f32) {
        debug_assert_eq!(self.Type(), ItemType::kLine);
        self.line_item_mut().text_fit_scale = scale;
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:311-374
    pub fn IsInlineBox(&self) -> bool {
        if self.Type() != ItemType::kBox {
            return false;
        }
        let box_fragment = self.BoxFragment();
        assert!(!box_fragment.is_null());
        unsafe { &*box_fragment }.IsInlineBox()
    }

    pub fn IsAtomicInline(&self) -> bool {
        let box_fragment = self.BoxFragment();
        !box_fragment.is_null() && unsafe { &*box_fragment }.IsAtomicInline()
    }

    pub fn IsBlockInInline(&self) -> bool {
        let box_fragment = self.BoxFragment();
        if !box_fragment.is_null() {
            return unsafe { &*box_fragment }.IsBlockInInline();
        }
        let line_fragment = self.LineBoxFragment();
        !line_fragment.is_null() && unsafe { &*line_fragment }.IsBlockInInline()
    }

    pub fn IsFloating(&self) -> bool {
        let box_fragment = self.BoxFragment();
        !box_fragment.is_null() && unsafe { &*box_fragment }.IsFloating()
    }

    pub fn IsEmptyLineBox(&self) -> bool {
        self.GetLineBoxType() == LineBoxType::kEmptyLineBox
    }

    pub fn IsStyleGeneratedText(&self) -> bool {
        self.Type() == ItemType::kText && unsafe { &*self.GetLayoutObject() }.IsStyleGenerated()
    }

    pub fn IsGeneratedText(&self) -> bool {
        self.IsLayoutGeneratedText() || self.IsStyleGeneratedText()
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:441
    pub fn IsFormattingContextRoot(&self) -> bool {
        let box_fragment = self.BoxFragment();
        !box_fragment.is_null() && unsafe { &*box_fragment }.IsFormattingContextRoot()
    }

    pub fn IsListMarker(&self) -> bool {
        let layout_object = self.GetLayoutObject();
        !layout_object.is_null() && unsafe { &*layout_object }.IsLayoutOutsideListMarker()
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:306-308
    // cpp: layoutng_fragment_tree/fragment_item.cc:376-381
    pub fn BlockInInline(&self) -> &LayoutObject {
        debug_assert!(self.IsBlockInInline());
        let block = unsafe { &*To::<LayoutBlockFlow>(self.GetLayoutObject()) }.FirstChild();
        assert!(!block.is_null());
        unsafe { &*block }
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:383-395
    // cpp: layoutng_fragment_tree/fragment_item.h:167-170
    pub fn SetSvgFragmentData(
        &mut self,
        data: *const TextFragmentRareData,
        unscaled_rect: PhysicalRect,
        hidden: bool,
    ) {
        debug_assert_eq!(self.Type(), ItemType::kText);
        self.text_item_mut().rare_data = Member::from_ptr(data as *mut _);
        self.rect_ = unscaled_rect;
        self.set_bit(Self::HIDDEN_BIT, hidden);
    }

    pub fn SetSvgLineLocalRect(&mut self, unscaled_rect: PhysicalRect) {
        debug_assert_eq!(self.Type(), ItemType::kLine);
        self.rect_ = unscaled_rect;
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:397-429
    pub fn ObjectBoundingBox(&self, items: &crate::fragment_items::FragmentItems) -> gfx::RectF {
        debug_assert!(self.IsSvgText());
        let font = self.ScaledFont();
        let mut ink_bounds = font.TextInkBounds(&self.TextPaintInfo(items));
        let font_data = font.PrimaryFont();
        if !font_data.is_null() {
            ink_bounds.Offset(0.0, unsafe { &*font_data }.GetFontMetrics().FloatAscent());
        }
        let svg_data = unsafe { &*self.GetSvgFragmentData() };
        ink_bounds.Scale(svg_data.length_adjust_scale, 1.0);
        let scaled_rect = svg_data.rect;
        match self.GetWritingMode() {
            WritingMode::kHorizontalTb => {}
            WritingMode::kVerticalLr | WritingMode::kVerticalRl | WritingMode::kSidewaysRl => {
                ink_bounds = gfx::RectF::new(
                    gfx::PointF::new(scaled_rect.width() - ink_bounds.bottom(), ink_bounds.x()),
                    gfx::SizeF::new(ink_bounds.height(), ink_bounds.width()),
                );
            }
            WritingMode::kSidewaysLr => {
                ink_bounds = gfx::RectF::new(
                    gfx::PointF::new(ink_bounds.y(), scaled_rect.height() - ink_bounds.right()),
                    gfx::SizeF::new(ink_bounds.height(), ink_bounds.width()),
                );
            }
        }
        let offset = scaled_rect.OffsetFromOrigin();
        ink_bounds.Offset(offset.x(), offset.y());
        ink_bounds.Union(scaled_rect);
        if self.HasSvgTransformForBoundingBox() {
            ink_bounds = self.BuildSvgTransformForBoundingBox().MapRect(ink_bounds);
        }
        ink_bounds.Scale(1.0 / self.SvgScalingFactor(), 1.0 / self.SvgScalingFactor());
        ink_bounds
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:431-438
    // cpp: layoutng_fragment_tree/fragment_item.h:561-563
    pub fn SvgUnscaledQuad(&self) -> gfx::QuadF {
        debug_assert!(self.IsSvgText());
        let rect = unsafe { &*self.GetSvgFragmentData() }.rect;
        let mut quad = self
            .BuildSvgTransformForBoundingBox()
            .MapQuad(gfx::QuadF::from(rect));
        let scale = self.SvgScalingFactor();
        quad.Scale(1.0 / scale, 1.0 / scale);
        quad
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:440-450
    pub fn MapPointInContainer(&self, point: PhysicalOffset) -> PhysicalOffset {
        if self.IsSvgText() && self.HasSvgTransformForBoundingBox() {
            let scale = self.SvgScalingFactor();
            let scaled = gfx::ScalePoint(gfx::PointF::from(point), scale, scale);
            let mapped = self
                .BuildSvgTransformForBoundingBox()
                .Inverse()
                .MapPoint(&scaled);
            let unscaled = gfx::ScalePoint(mapped, scale, scale);
            return PhysicalOffset::FromPointFRound(&unscaled);
        }
        point
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:452-462
    pub fn ScaleInlineOffset(&self, inline_offset: LayoutUnit) -> f32 {
        let svg_data = self.GetSvgFragmentData();
        if !svg_data.is_null() {
            return inline_offset.ToFloat() * self.SvgScalingFactor()
                / unsafe { &*svg_data }.length_adjust_scale;
        }
        let scale = self.GetTextFitScale();
        if scale != 1.0 {
            inline_offset.ToFloat() / scale
        } else {
            inline_offset.ToFloat()
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:464-473
    pub fn InclusiveContains(&self, position: gfx::PointF) -> bool {
        debug_assert!(self.IsSvgText());
        let scale = self.SvgScalingFactor();
        let scaled = gfx::ScalePoint(position, scale, scale);
        let rect = unsafe { &*self.GetSvgFragmentData() }.rect;
        if !self.HasSvgTransformForBoundingBox() {
            return rect.InclusiveContains(scaled);
        }
        self.BuildSvgTransformForBoundingBox()
            .MapQuad(gfx::QuadF::from(rect))
            .Contains(scaled)
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:475-487
    // cpp: layoutng_fragment_tree/fragment_item.h:310-311
    pub fn HasNonVisibleOverflow(&self) -> bool {
        let fragment = self.BoxFragment();
        !fragment.is_null() && unsafe { &*fragment }.HasNonVisibleOverflow()
    }

    pub fn HasSelfPaintingLayer(&self) -> bool {
        let fragment = self.BoxFragment();
        !fragment.is_null() && unsafe { &*fragment }.HasSelfPaintingLayer()
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:503-517
    pub fn LayoutObjectWillBeDestroyed(&self) {
        unsafe {
            *self.layout_object_.get() = Member::default();
        }
        let fragment = self.BoxFragment();
        if !fragment.is_null() {
            unsafe { &*fragment }.LayoutObjectWillBeDestroyed();
        }
    }

    pub fn LayoutObjectWillBeMoved(&self) {
        unsafe {
            *self.layout_object_.get() = Member::default();
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:519-525
    pub fn ContentOffsetInContainerFragment(&self) -> PhysicalOffset {
        let mut offset = *self.OffsetInContainerFragment();
        let box_fragment = self.BoxFragment();
        if !box_fragment.is_null() {
            offset += unsafe { &*box_fragment }.ContentOffset();
        }
        offset
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:622
    // cpp: layoutng_fragment_tree/fragment_item.cc:527-533
    fn MutableInkOverflowOwnerBox(&self) -> *mut LayoutBox {
        if self.Type() == ItemType::kBox {
            DynamicTo::<LayoutBox>(self.GetMutableLayoutObject())
        } else {
            std::ptr::null_mut()
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:535-553
    pub fn SelfInkOverflowRect(&self) -> PhysicalRect {
        let box_fragment = self.BoxFragment();
        if !box_fragment.is_null() {
            return unsafe { &*box_fragment }.SelfInkOverflowRect();
        }
        if !self.HasInkOverflow() {
            return self.LocalRect();
        }
        unsafe { &*self.ink_overflow_.get() }.SelfRect(self.InkOverflowType(), &self.Size())
    }

    pub fn InkOverflowRect(&self) -> PhysicalRect {
        let box_fragment = self.BoxFragment();
        if !box_fragment.is_null() {
            return unsafe { &*box_fragment }.InkOverflowRect();
        }
        if !self.HasInkOverflow() {
            return self.LocalRect();
        }
        if !self.IsContainer() || self.HasNonVisibleOverflow() {
            return unsafe { &*self.ink_overflow_.get() }
                .SelfRect(self.InkOverflowType(), &self.Size());
        }
        unsafe { &*self.ink_overflow_.get() }.SelfAndContents(self.InkOverflowType(), &self.Size())
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:555-569
    pub fn TextShapeResult(&self) -> *const ShapeResultView {
        match self.Type() {
            ItemType::kText => self.text_item().shape_result.Get(),
            ItemType::kGeneratedText => self.generated_text_item().shape_result.Get(),
            _ => unreachable!("not a text FragmentItem"),
        }
    }

    pub fn TextOffset(&self) -> TextOffsetRange {
        match self.Type() {
            ItemType::kText => self.text_item().text_offset,
            ItemType::kGeneratedText => TextOffsetRange {
                start: 0,
                end: self.generated_text_item().text.length() as u32,
            },
            _ => unreachable!("not a text FragmentItem"),
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:317-332
    pub fn GetInlineBreakToken(&self) -> *const InlineBreakToken {
        let line = self.LineBoxFragment();
        if line.is_null() {
            debug_assert_eq!(self.Type(), ItemType::kLine);
            return std::ptr::null();
        }
        let token = unsafe { &*line }.GetBreakToken();
        if token.is_null() {
            return std::ptr::null();
        }
        // C++ To<InlineBreakToken> checks the break-token discriminant before
        // downcasting the PhysicalFragment's base pointer.
        assert!(unsafe { &*token }.IsInlineType());
        token as *const InlineBreakToken
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:453-456
    // cpp: layoutng_fragment_tree/fragment_item.cc:571-590
    pub fn StartOffsetInContainer(&self, container: &InlineCursor) -> u32 {
        debug_assert_eq!(self.Type(), ItemType::kGeneratedText);
        debug_assert!(!self.IsEllipsis());
        debug_assert_eq!(container.Current().Item(), self as *const Self);
        let mut cursor = container.clone();
        cursor.MoveToPrevious();
        while cursor.IsNotNull() {
            let current = cursor.Current();
            if current.IsText() && !current.IsLayoutGeneratedText() {
                return unsafe { &*current.Item() }.EndOffset();
            }
            let item = unsafe { &*current.Item() };
            if item.Type() == ItemType::kBox && !item.IsInlineBox() {
                break;
            }
            cursor.MoveToPrevious();
        }
        0
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:458-461
    // cpp: layoutng_fragment_tree/fragment_item.cc:592-600
    pub fn Text(&self, items: &crate::fragment_items::FragmentItems) -> StringView {
        match self.Type() {
            ItemType::kText => {
                let range = self.text_item().text_offset;
                StringView::new(
                    items.Text(self.UsesFirstLineStyle()),
                    range.start,
                    range.Length(),
                )
            }
            ItemType::kGeneratedText => self.GeneratedText(),
            _ => unreachable!("not a text FragmentItem"),
        }
    }

    pub fn GeneratedText(&self) -> StringView {
        debug_assert_eq!(self.Type(), ItemType::kGeneratedText);
        StringView::from(&self.generated_text_item().text)
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:463
    // cpp: layoutng_fragment_tree/fragment_item.cc:602-614
    pub fn TextPaintInfo(
        &self,
        items: &crate::fragment_items::FragmentItems,
    ) -> TextFragmentPaintInfo {
        let scale = self.GetTextFitScale();
        match self.Type() {
            ItemType::kText => TextFragmentPaintInfo {
                text: StringView::from(items.Text(self.UsesFirstLineStyle())),
                from: self.text_item().text_offset.start,
                to: self.text_item().text_offset.end,
                shape_result: self.text_item().shape_result.Get(),
                text_fit_scaling_factor: scale,
            },
            ItemType::kGeneratedText => TextFragmentPaintInfo {
                text: StringView::from(&self.generated_text_item().text),
                from: 0,
                to: self.generated_text_item().text.length() as u32,
                shape_result: self.generated_text_item().shape_result.Get(),
                text_fit_scaling_factor: scale,
            },
            _ => unreachable!("not a text FragmentItem"),
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:616-624
    // cpp: layoutng_fragment_tree/fragment_item.h:486-490
    pub fn BaseDirection(&self) -> TextDirection {
        debug_assert_eq!(self.Type(), ItemType::kLine);
        if self.bit(Self::TEXT_DIRECTION_BIT) {
            TextDirection::kRtl
        } else {
            TextDirection::kLtr
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:492-494
    pub fn ResolvedDirection(&self) -> TextDirection {
        debug_assert!(self.IsText() || self.IsAtomicInline());
        if self.bit(Self::TEXT_DIRECTION_BIT) {
            TextDirection::kRtl
        } else {
            TextDirection::kLtr
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:837-858
    pub fn GetTextFitScale(&self) -> f32 {
        match self.Type() {
            ItemType::kText => {
                let data = self.text_item().rare_data.Get();
                if !data.is_null() && !unsafe { &*data }.is_svg {
                    return unsafe { &*data }.length_adjust_scale;
                }
            }
            ItemType::kGeneratedText => {
                let data = self.generated_text_item().rare_data.Get();
                if !data.is_null() {
                    debug_assert!(!unsafe { &*data }.is_svg);
                    return unsafe { &*data }.length_adjust_scale;
                }
            }
            ItemType::kLine => return self.line_item().text_fit_scale,
            _ => {}
        }
        1.0
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:860-866
    // cpp: layoutng_fragment_tree/fragment_item.h:515-528
    pub fn AnnotationMetrics(&self) -> FontHeight {
        if self.Type() != ItemType::kText {
            return FontHeight::default();
        }
        let data = self.text_item().rare_data.Get();
        if data.is_null() {
            FontHeight::default()
        } else {
            unsafe { &*data }.annotation_metrics
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:581
    // cpp: layoutng_fragment_tree/fragment_item.cc:868-914
    pub fn ToString(&self) -> String {
        let mut name = StringBuilder::default();
        name.Append("FragmentItem");
        if self.IsHiddenForPaint() {
            name.Append(" (hidden)");
        }
        let object = self.GetLayoutObject();
        match self.Type() {
            ItemType::kBox => {
                name.Append(" Box ");
                name.Append(&unsafe { &*object }.DebugName());
            }
            ItemType::kText => {
                name.Append(" Text ");
                let mut items: *const crate::fragment_items::FragmentItems = std::ptr::null();
                let block = unsafe { &*object }.FragmentItemsContainer();
                if !block.is_null() {
                    let block = unsafe { &*block };
                    for i in 0..block.PhysicalFragmentCount() {
                        let fragment = block.GetPhysicalFragment(i);
                        items = unsafe { &*fragment }.Items();
                        if !items.is_null() {
                            break;
                        }
                    }
                }
                if items.is_null() {
                    name.Append("\"(container not found)\"");
                } else {
                    name.Append(
                        &self
                            .Text(unsafe { &*items })
                            .ToString()
                            .EncodeForDebugging(),
                    );
                }
            }
            ItemType::kGeneratedText => {
                name.Append(" GeneratedText ");
                name.Append(&self.GeneratedText().EncodeForDebugging());
                name.Append(" ");
                if object.is_null() {
                    name.Append("null");
                } else {
                    name.Append(&unsafe { &*object }.DebugName());
                }
            }
            ItemType::kLine => name.Append(" Line"),
            ItemType::kInvalid => name.Append(" Invalid"),
        }
        name.ToString()
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:344
    // cpp: layoutng_fragment_tree/fragment_item.cc:916-934
    pub fn LocalVisualRectFor(layout_object: &LayoutObject) -> PhysicalRect {
        debug_assert!(layout_object.IsInLayoutNGInlineFormattingContext());
        let mut visual_rect = PhysicalRect::default();
        let mut cursor = InlineCursor::default();
        cursor.MoveToLayoutObject(layout_object);
        while cursor.IsNotNull() {
            let item = unsafe { &*cursor.Current().Item() };
            if !item.IsHiddenForPaint() {
                let mut child_rect = item.SelfInkOverflowRect();
                child_rect.offset += *item.OffsetInContainerFragment();
                visual_rect.Unite(&child_rect);
            }
            cursor.MoveToNextForSameLayoutObject();
        }
        visual_rect
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:626-638
    pub fn HasSvgTransformForPaint(&self) -> bool {
        let data = self.GetSvgFragmentData();
        !data.is_null()
            && (unsafe { &*data }.length_adjust_scale != 1.0 || unsafe { &*data }.angle != 0.0)
    }

    pub fn HasSvgTransformForBoundingBox(&self) -> bool {
        let data = self.GetSvgFragmentData();
        !data.is_null() && unsafe { &*data }.angle != 0.0
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:640-660
    // cpp: layoutng_fragment_tree/fragment_item.h:547-551
    pub fn BuildSvgTransformForPaint(&self) -> AffineTransform {
        debug_assert!(self.IsSvgText());
        let data = unsafe { &*self.GetSvgFragmentData() };
        if data.in_text_path {
            let adjust = self.BuildSvgTransformForLengthAdjust();
            if data.angle == 0.0 {
                return adjust;
            }
            return self.BuildSvgTransformForTextPath(&adjust);
        }
        let mut transform = self.BuildSvgTransformForBoundingBox();
        let adjust = self.BuildSvgTransformForLengthAdjust();
        if !adjust.IsIdentity() {
            transform.PostConcat(adjust);
        }
        transform
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:662-688
    // cpp: layoutng_fragment_tree/fragment_item.h:634-636
    fn BuildSvgTransformForLengthAdjust(&self) -> AffineTransform {
        debug_assert!(self.IsSvgText());
        let data = unsafe { &*self.GetSvgFragmentData() };
        let mut transform = AffineTransform::default();
        let scale = data.length_adjust_scale;
        if scale != 1.0 {
            let text_path_rotation = data.in_text_path && data.angle != 0.0;
            if self.IsHorizontal() {
                let x = data.rect.x();
                transform.SetMatrix(
                    scale as f64,
                    0.0,
                    0.0,
                    1.0,
                    if text_path_rotation {
                        0.0
                    } else {
                        (x - scale * x) as f64
                    },
                    0.0,
                );
            } else {
                let y = data.rect.y();
                transform.SetMatrix(
                    1.0,
                    0.0,
                    0.0,
                    scale as f64,
                    0.0,
                    if text_path_rotation {
                        0.0
                    } else {
                        (y - scale * y) as f64
                    },
                );
            }
        }
        transform
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:690-732
    fn BuildSvgTransformForTextPath(&self, length_adjust: &AffineTransform) -> AffineTransform {
        debug_assert!(self.IsSvgText());
        let data = unsafe { &*self.GetSvgFragmentData() };
        debug_assert!(data.in_text_path && data.angle != 0.0);
        let mut transform = AffineTransform::default();
        transform.Rotate(data.angle as f64);
        let font_data = unsafe { &*self.ScaledFont().PrimaryFont() };
        let baseline = self.Style().GetFontBaseline();
        let mut x = data.rect.x();
        let mut y = data.rect.y();
        match self.GetWritingMode() {
            WritingMode::kHorizontalTb => {
                y += font_data.GetFontMetrics().FixedAscent(baseline).ToFloat();
                transform.Translate((-data.rect.width() / 2.0) as f64, data.baseline_shift as f64);
            }
            WritingMode::kVerticalLr | WritingMode::kVerticalRl | WritingMode::kSidewaysRl => {
                x += font_data.GetFontMetrics().FixedDescent(baseline).ToFloat();
                transform.Translate(data.baseline_shift as f64, (-data.rect.height() / 2.0) as f64);
            }
            WritingMode::kSidewaysLr => {
                x += font_data.GetFontMetrics().FixedAscent(baseline).ToFloat();
                y = data.rect.bottom();
                transform.Translate((-data.baseline_shift) as f64, (data.rect.height() / 2.0) as f64);
            }
        }
        transform.PreConcat(*length_adjust);
        transform.SetE(transform.E() + x as f64);
        transform.SetF(transform.F() + y as f64);
        transform.Translate((-x) as f64, (-y) as f64);
        transform
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:734-763
    // cpp: layoutng_fragment_tree/fragment_item.h:552-559
    pub fn BuildSvgTransformForBoundingBox(&self) -> AffineTransform {
        debug_assert!(self.IsSvgText());
        let data = unsafe { &*self.GetSvgFragmentData() };
        let mut transform = AffineTransform::default();
        if data.angle == 0.0 {
            return transform;
        }
        if data.in_text_path {
            return self.BuildSvgTransformForTextPath(&AffineTransform::default());
        }
        transform.Rotate(data.angle as f64);
        let font_data = self.ScaledFont().PrimaryFont();
        let ascent = if font_data.is_null() {
            0.0
        } else {
            unsafe { &*font_data }
                .GetFontMetrics()
                .FixedAscent(font_engine::FontBaseline::kAlphabeticBaseline)
                .ToFloat()
        };
        let y = data.rect.y() + ascent;
        transform.SetE(transform.E() + data.rect.x() as f64);
        transform.SetF(transform.F() + y as f64);
        transform.Translate((-data.rect.x()) as f64, (-y) as f64);
        transform
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:765-772
    // cpp: layoutng_fragment_tree/fragment_item.h:565-567
    pub fn SvgScalingFactor(&self) -> f32 {
        let object = self.GetLayoutObject();
        if object.is_null() || !unsafe { &*object }.IsSVGInlineText() {
            return 1.0;
        }
        let factor = unsafe { &*To::<LayoutText>(object) }.SvgScalingFactor();
        debug_assert!(factor > 0.0);
        factor
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:774-785
    // cpp: layoutng_fragment_tree/fragment_item.h:569-572
    pub fn ScaledFont(&self) -> &Font {
        let object = self.GetLayoutObject();
        if !object.is_null() && unsafe { &*object }.IsSVGInlineText() {
            return unsafe { &*To::<LayoutText>(object) }.ScaledFont();
        }
        let data = match self.Type() {
            ItemType::kText => self.text_item().rare_data.Get(),
            ItemType::kGeneratedText => self.generated_text_item().rare_data.Get(),
            _ => std::ptr::null(),
        };
        if !data.is_null() {
            let font = unsafe { &*data }.scaled_font.Get();
            if !font.is_null() {
                return unsafe { &*font };
            }
        }
        unsafe { &*self.Style().GetFont() }
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:816-835
    // cpp: layoutng_fragment_tree/fragment_item.h:573-574
    pub fn GetUsedFont(&self) -> UsedFont {
        let object = self.GetLayoutObject();
        if !object.is_null() && unsafe { &*object }.IsSVGInlineText() {
            return UsedFont::new(unsafe { &*To::<LayoutText>(object) }.ScaledFont(), 1.0);
        }
        if self.Type() == ItemType::kLine {
            return UsedFont::new(
                unsafe { &*self.Style().GetFont() },
                self.line_item().text_fit_scale,
            );
        }
        let data = match self.Type() {
            ItemType::kText => self.text_item().rare_data.Get(),
            ItemType::kGeneratedText => self.generated_text_item().rare_data.Get(),
            _ => std::ptr::null(),
        };
        if !data.is_null() {
            let data = unsafe { &*data };
            debug_assert!(!data.is_svg);
            let font = data.scaled_font.Get();
            let font = if font.is_null() {
                self.Style().GetFont()
            } else {
                font
            };
            return UsedFont::new(unsafe { &*font }, data.length_adjust_scale);
        }
        UsedFont::new(unsafe { &*self.Style().GetFont() }, 1.0)
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:936-939
    // cpp: layoutng_fragment_tree/fragment_item.h:624
    pub fn InvalidateInkOverflow(&self) {
        let overflow_type = self.InkOverflowType();
        let new_type = unsafe { &mut *self.ink_overflow_.get() }.Invalidate(overflow_type);
        self.set_bits(Self::INK_OVERFLOW_BITS, 3, new_type as u32);
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:684-691
    // cpp: layoutng_fragment_tree/fragment_item.h:534-535
    pub fn CanReuse(&self) -> bool {
        debug_assert_ne!(self.Type(), ItemType::kLine);
        if self.IsDirty() {
            return false;
        }
        let layout_object = self.GetLayoutObject();
        !layout_object.is_null() && !unsafe { &*layout_object }.SelfNeedsFullLayout()
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:952-983
    // cpp: layoutng_fragment_tree/fragment_item.h:465-468
    pub fn CaretInlinePositionForOffset(&self, text: StringView, offset: u32) -> LayoutUnit {
        debug_assert!(offset >= self.StartOffset() && offset <= self.EndOffset());
        debug_assert_eq!(text.length(), self.TextLength());
        let offset = offset - self.StartOffset();
        let shape = self.TextShapeResult();
        if !shape.is_null() {
            let shape = unsafe { &*shape }.CreateShapeResult();
            return LayoutUnit::FromFloatRound(
                unsafe { &*shape }.CaretPositionForOffset(offset, &text, AdjustMidCluster::kToEnd)
                    * self.GetTextFitScale(),
            );
        }
        debug_assert!(self.IsFlowControl());
        debug_assert_eq!(text.length(), 1);
        if offset == 0 || self.Style().Direction() == TextDirection::kRtl {
            return LayoutUnit::default();
        }
        let data = self.GetSvgFragmentData();
        if !data.is_null() {
            let rect = unsafe { &*data }.rect;
            return LayoutUnit::from_f32(if self.IsHorizontal() {
                rect.width()
            } else {
                rect.height()
            });
        }
        if self.IsHorizontal() {
            self.Size().width
        } else {
            self.Size().height
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:985-1052
    // cpp: layoutng_fragment_tree/fragment_item.h:470-477
    pub fn LineLeftAndRightForOffsets(
        &self,
        text: StringView,
        start_offset: u32,
        end_offset: u32,
    ) -> (LayoutUnit, LayoutUnit) {
        debug_assert!(start_offset >= self.StartOffset() && start_offset <= self.EndOffset());
        debug_assert!(end_offset >= self.StartOffset() && end_offset <= self.EndOffset());
        debug_assert_eq!(text.length(), self.TextLength());
        let start = start_offset - self.StartOffset();
        let end = end_offset - self.StartOffset();
        let shape = self.TextShapeResult();
        let (start_position, end_position) = if !shape.is_null() {
            let scale = self.GetTextFitScale();
            let shape = unsafe { &*shape }.CreateShapeResult();
            let shape = unsafe { &*shape };
            LayoutUnit::FromFloatEncompassRound(
                shape.CaretPositionForOffset(start, &text, AdjustMidCluster::kToStart) * scale,
                shape.CaretPositionForOffset(end, &text, AdjustMidCluster::kToEnd) * scale,
            )
        } else {
            debug_assert!(self.IsFlowControl());
            debug_assert_eq!(text.length(), 1);
            let position = if self.Style().Direction() == TextDirection::kRtl {
                LayoutUnit::default()
            } else {
                let data = self.GetSvgFragmentData();
                if !data.is_null() {
                    let rect = unsafe { &*data }.rect;
                    LayoutUnit::from_f32(if self.IsHorizontal() {
                        rect.width()
                    } else {
                        rect.height()
                    })
                } else if self.IsHorizontal() {
                    self.Size().width
                } else {
                    self.Size().height
                }
            };
            (
                if start == 0 {
                    LayoutUnit::default()
                } else {
                    position
                },
                if end == 0 {
                    LayoutUnit::default()
                } else {
                    position
                },
            )
        };
        if start_position > end_position {
            (end_position, start_position)
        } else {
            (start_position, end_position)
        }
    }

    // Rust has no overloads; this is the text-range form of C++ LocalRect().
    // cpp: layoutng_fragment_tree/fragment_item.cc:1054-1088
    // cpp: layoutng_fragment_tree/fragment_item.h:479-484
    pub fn LocalTextRect(
        &self,
        text: StringView,
        start_offset: u32,
        end_offset: u32,
    ) -> PhysicalRect {
        let mut width = self.Size().width;
        let mut height = self.Size().height;
        let data = self.GetSvgFragmentData();
        if !data.is_null() {
            let data = unsafe { &*data };
            if self.IsHorizontal() {
                width = LayoutUnit::from_f32(data.rect.width() / data.length_adjust_scale);
                height = LayoutUnit::from_f32(data.rect.height());
            } else {
                width = LayoutUnit::from_f32(data.rect.width());
                height =
                    LayoutUnit::from_f32(data.rect.height() / data.length_adjust_scale);
            }
        }
        if start_offset == self.StartOffset() && end_offset == self.EndOffset() {
            return PhysicalRect::new(PhysicalOffset::default(), PhysicalSize::new(width, height));
        }
        let (start, end) = self.LineLeftAndRightForOffsets(text, start_offset, end_offset);
        let inline_size = end - start;
        match self.GetWritingMode() {
            WritingMode::kHorizontalTb => PhysicalRect::new(
                PhysicalOffset::new(start, LayoutUnit::default()),
                PhysicalSize::new(inline_size, height),
            ),
            WritingMode::kVerticalLr | WritingMode::kVerticalRl | WritingMode::kSidewaysRl => {
                PhysicalRect::new(
                    PhysicalOffset::new(LayoutUnit::default(), start),
                    PhysicalSize::new(width, inline_size),
                )
            }
            WritingMode::kSidewaysLr => PhysicalRect::new(
                PhysicalOffset::new(LayoutUnit::default(), height - end),
                PhysicalSize::new(width, inline_size),
            ),
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:1090-1109
    // cpp: layoutng_fragment_tree/fragment_item.h:496-500
    pub fn ComputeTextBoundsRectForHitTest(
        &self,
        inline_root_offset: PhysicalOffset,
        is_occlusion_test: bool,
    ) -> PhysicalRect {
        debug_assert!(self.IsText());
        let border_rect = PhysicalRect::new(
            inline_root_offset + *self.OffsetInContainerFragment(),
            self.Size(),
        );
        if is_occlusion_test {
            let mut overflow = self.SelfInkOverflowRect();
            overflow.Move(&border_rect.offset);
            return overflow;
        }
        if self.IsSvgText() {
            return border_rect;
        }
        PhysicalRect::from(foundation::ToPixelSnappedRect(border_rect))
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:1115-1147
    // cpp: layoutng_fragment_tree/fragment_item.h:508-509
    pub fn TextOffsetForPoint(
        &self,
        point: PhysicalOffset,
        items: &crate::fragment_items::FragmentItems,
    ) -> u32 {
        debug_assert_eq!(self.Type(), ItemType::kText);
        let converter = WritingModeConverter::new(
            foundation::WritingDirectionMode::new(self.GetWritingMode(), TextDirection::kLtr),
            self.Size(),
        );
        let line_offset = converter
            .ToLogicalOffset(point, PhysicalSize::default())
            .inline_offset;
        let shape = self.TextShapeResult();
        if !shape.is_null() {
            let scaled = self.ScaleInlineOffset(line_offset);
            let result = unsafe { &*shape }.CreateShapeResult();
            return unsafe { &*result }.CaretOffsetForHitTest(scaled, &self.Text(items))
                + self.StartOffset();
        }
        debug_assert!(self.IsFlowControl());
        let inline_size = converter.ToLogicalSize(self.Size()).inline_size;
        if inline_size == LayoutUnit::default() {
            return self.StartOffset();
        }
        let offset = if self.ResolvedDirection() == TextDirection::kLtr {
            line_offset
        } else {
            inline_size - line_offset
        };
        debug_assert_eq!(self.TextLength(), 1);
        if offset <= inline_size / 2 {
            self.StartOffset()
        } else {
            self.EndOffset()
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.h:612-620
    fn InkOverflowType(&self) -> InkOverflowType {
        match self.bits(Self::INK_OVERFLOW_BITS, 3) {
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

    fn HasInkOverflow(&self) -> bool {
        self.InkOverflowType() != InkOverflowType::kNone
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:787-814
    // cpp: layoutng_fragment_tree/fragment_item.h:638-639
    fn SetTextRareData(&mut self, scale: *const TextFitScale, annotation_metrics: FontHeight) {
        let scale_ref = if scale.is_null() {
            None
        } else {
            Some(unsafe { &*scale })
        };
        let is_text_fit =
            scale_ref.is_some_and(|scale| scale.scale != 1.0 || !scale.font.Get().is_null());
        if !is_text_fit
            && annotation_metrics.ascent == LayoutUnit::default()
            && annotation_metrics.descent == LayoutUnit::default()
        {
            return;
        }
        let mut data = TextFragmentRareData {
            annotation_metrics,
            ..TextFragmentRareData::default()
        };
        if is_text_fit {
            let scale_ref = scale_ref.unwrap();
            data.length_adjust_scale = scale_ref.scale;
            data.scaled_font = scale_ref.font.clone();
        }
        let data = Member::from_ptr(MakeGarbageCollected(data));
        match self.Type() {
            ItemType::kText => self.text_item_mut().rare_data = data,
            ItemType::kGeneratedText if is_text_fit => {
                self.generated_text_item_mut().rare_data = data
            }
            _ => unreachable!("rare data requires text item"),
        }
        if is_text_fit {
            debug_assert_eq!(scale_ref.unwrap().scale, self.GetTextFitScale());
        }
    }

    // cpp: layoutng_fragment_tree/fragment_item.cc:1149-1168
    // cpp: layoutng_fragment_tree/fragment_item.h:583
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(unsafe { &*self.layout_object_.get() });
        match self.Type() {
            ItemType::kInvalid => {}
            ItemType::kText => self.text_item().Trace(visitor),
            ItemType::kGeneratedText => self.generated_text_item().Trace(visitor),
            ItemType::kLine => self.line_item().Trace(visitor),
            ItemType::kBox => self.box_item().Trace(visitor),
        }
    }
}

// The source copy constructor resets the inline overflow type, then copies
// the overflow only when it has been computed.
// cpp: layoutng_fragment_tree/fragment_item.h:147-149
// cpp: layoutng_fragment_tree/fragment_item.cc:214-252
impl Clone for FragmentItem {
    fn clone(&self) -> Self {
        let payload = match self.Type() {
            ItemType::kText => FragmentItemPayload {
                text_: ManuallyDrop::new(self.text_item().clone()),
            },
            ItemType::kGeneratedText => FragmentItemPayload {
                generated_text_: ManuallyDrop::new(self.generated_text_item().clone()),
            },
            ItemType::kLine => FragmentItemPayload {
                line_: ManuallyDrop::new(self.line_item().clone()),
            },
            ItemType::kBox => FragmentItemPayload {
                box_: ManuallyDrop::new(self.box_item().clone()),
            },
            ItemType::kInvalid => unreachable!("cannot clone invalid FragmentItem"),
        };
        let computed = self.IsInkOverflowComputed();
        let mask = 7 << Self::INK_OVERFLOW_BITS;
        let flags = if computed {
            self.flags_.get()
        } else {
            (self.flags_.get() & !mask)
                | ((InkOverflowType::kNotSet as u32) << Self::INK_OVERFLOW_BITS)
        };
        Self {
            payload_: payload,
            rect_: self.rect_,
            ink_overflow_: UnsafeCell::new(if computed {
                InkOverflow::new(self.InkOverflowType(), unsafe {
                    &*self.ink_overflow_.get()
                })
            } else {
                InkOverflow::default()
            }),
            layout_object_: UnsafeCell::new(unsafe { &*self.layout_object_.get() }.clone()),
            fragment_id_: Cell::new(self.fragment_id_.get()),
            delta_to_next_for_same_layout_object_: Cell::new(
                self.delta_to_next_for_same_layout_object_.get(),
            ),
            flags_: Cell::new(flags),
        }
    }
}

// Rust moves the active payload and overflow with the whole struct; C++
// explicitly dispatches on the active union arm in its move constructor.
// The C++ VectorTraits permits bytewise relocation and zeroed unused slots;
// Rust Vec relocates initialized values and never exposes unused slots.
// cpp: layoutng_fragment_tree/fragment_item.h:696-702
// cpp: layoutng_fragment_tree/fragment_item.cc:254-288

// cpp: layoutng_fragment_tree/fragment_item.h:151
// cpp: layoutng_fragment_tree/fragment_item.cc:290-309
impl Drop for FragmentItem {
    fn drop(&mut self) {
        unsafe {
            match self.Type() {
                ItemType::kInvalid => return,
                ItemType::kText => ManuallyDrop::drop(&mut self.payload_.text_),
                ItemType::kGeneratedText => ManuallyDrop::drop(&mut self.payload_.generated_text_),
                ItemType::kLine => ManuallyDrop::drop(&mut self.payload_.line_),
                ItemType::kBox => ManuallyDrop::drop(&mut self.payload_.box_),
            }
        }
        let overflow_type = self.InkOverflowType();
        self.ink_overflow_.get_mut().Reset(overflow_type);
    }
}

// cpp: layoutng_fragment_tree/fragment_item.h:693-694
// cpp: layoutng_fragment_tree/fragment_item.cc:1170-1210
impl std::fmt::Display for FragmentItem {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(out, "{{")?;
        match self.Type() {
            ItemType::kInvalid => unreachable!("invalid FragmentItem"),
            ItemType::kText => write!(
                out,
                "Text {}-{} {}",
                self.StartOffset(),
                self.EndOffset(),
                if self.ResolvedDirection() == TextDirection::kLtr {
                    "LTR"
                } else {
                    "RTL"
                }
            )?,
            ItemType::kGeneratedText => write!(out, "GeneratedText \"{}\"", self.GeneratedText())?,
            ItemType::kLine => write!(
                out,
                "Line #descendants={} {}",
                self.DescendantsCount(),
                if self.BaseDirection() == TextDirection::kLtr {
                    "LTR"
                } else {
                    "RTL"
                }
            )?,
            ItemType::kBox => {
                write!(out, "Box #descendants={}", self.DescendantsCount())?;
                if self.IsAtomicInline() {
                    write!(
                        out,
                        " AtomicInline{}",
                        if self.ResolvedDirection() == TextDirection::kLtr {
                            "LTR"
                        } else {
                            "RTL"
                        }
                    )?;
                }
            }
        }
        let style = match self.GetStyleVariant() {
            StyleVariant::kStandard => "Standard",
            StyleVariant::kFirstLine => "FirstLine",
            StyleVariant::kStandardEllipsis => "StandardEllipsis",
            StyleVariant::kFirstLineEllipsis => "FirstLineEllipsis",
        };
        write!(out, " {style}}}")
    }
}

// cpp: layoutng_fragment_tree/fragment_item.cc:1212-1216
pub fn format_fragment_item_pointer(item: *const FragmentItem) -> String {
    if item.is_null() {
        return String::from("<null>");
    }
    String::from(format!("{}", unsafe { &*item }))
}
