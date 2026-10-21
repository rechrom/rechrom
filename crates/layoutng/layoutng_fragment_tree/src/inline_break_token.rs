use std::ops::Deref;

use foundation::{
    MakeGarbageCollectedWithAdditionalBytes, Member, String, StringBuilder, Traceable, Visitor,
};
use layoutng::internal::inline_item_text_index::InlineItemTextIndex;
use layoutng::internal::inline_node::InlineNode;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::block_break_token::BlockBreakToken;
use crate::break_token::{BreakToken, BreakTokenType};

// cpp: layoutng_fragment_tree/inline_break_token.h:21-28
#[derive(Clone)]
pub struct AnnotationBreakTokenData {
    pub start: InlineItemTextIndex,
    pub start_item_index: u32,
    pub end_item_index: u32,
}

// cpp: layoutng_fragment_tree/inline_break_token.h:30-45
// Rust Vec preserves ordering and ownership; the C++ Vector's inline capacity
// of one changes only the allocation strategy.
pub struct RubyBreakTokenData {
    pub open_column_item_index: u32,
    pub ruby_base_end_item_index: u32,
    pub annotation_data: Vec<AnnotationBreakTokenData>,
}

#[allow(non_snake_case)]
impl RubyBreakTokenData {
    // cpp: layoutng_fragment_tree/inline_break_token.h:38-43
    pub fn new(
        open_column_index: u32,
        base_end_index: u32,
        annotations: &Vec<AnnotationBreakTokenData>,
    ) -> Self {
        Self {
            open_column_item_index: open_column_index,
            ruby_base_end_item_index: base_end_index,
            annotation_data: annotations.clone(),
        }
    }

    // The source trace is empty because the annotation vector has no GC
    // references. The vector remains owned by this garbage-collected object.
    // cpp: layoutng_fragment_tree/inline_break_token.h:44
    pub fn Trace(&self, _visitor: &mut Visitor) {}
}

// These are bit values, not exclusive states; flags combine in a u32.
// cpp: layoutng_fragment_tree/inline_break_token.h:50-60
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlineBreakTokenFlag {
    kDefault = 0,
    kIsForcedBreak = 1 << 0,
    kHasRareData = 1 << 1,
    kUseFirstLineStyle = 1 << 2,
    kHasClonedBoxDecorations = 1 << 3,
    kIsInParallelBlockFlow = 1 << 4,
    kIsPastFirstFormattedLine = 1 << 5,
    kIsLineClampDisplacedLine = 1 << 6,
}

// C++ stores this only when kHasRareData is set. It has no non-trivial
// destructor, so it can be initialized inside a GC object's trailing bytes.
// cpp: layoutng_fragment_tree/inline_break_token.h:147-155
#[repr(C)]
struct InlineBreakTokenRareData {
    sub_break_token: Member<BlockBreakToken>,
    ruby_data: Member<RubyBreakTokenData>,
}

#[allow(non_snake_case)]
impl InlineBreakTokenRareData {
    // cpp: layoutng_fragment_tree/inline_break_token.cc:117-120
    fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.sub_break_token);
        visitor.Trace(&self.ruby_data);
    }
}

impl Traceable for InlineBreakTokenRareData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        InlineBreakTokenRareData::Trace(self, visitor);
    }
}

// The zero-length final field preserves the C++ flexible-array header. The
// GC allocation contract must construct this header and its optional tail
// before publishing the object to the collector.
// cpp: layoutng_fragment_tree/inline_break_token.h:47-49
// cpp: layoutng_fragment_tree/inline_break_token.h:157-162
#[repr(C)]
pub struct InlineBreakToken {
    pub(crate) base_: BreakToken,
    style_: Member<ComputedStyle>,
    start_: InlineItemTextIndex,
    rare_data_: [InlineBreakTokenRareData; 0],
}

// cpp: layoutng_fragment_tree/inline_break_token.h:172-179
impl foundation::DowncastFrom<BreakToken> for InlineBreakToken {
    fn AllowFrom(token: &BreakToken) -> bool {
        token.IsInlineType()
    }
}

const _: () = assert!(std::mem::offset_of!(InlineBreakToken, base_) == 0);

impl Deref for InlineBreakToken {
    type Target = BreakToken;

    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

#[allow(non_snake_case)]
impl InlineBreakToken {
    fn has_flag(&self, flag: InlineBreakTokenFlag) -> bool {
        self.base_.flags() & flag as u32 != 0
    }

    // Only call after the immutable has-rare-data flag has been checked.
    fn rare_data(&self) -> &InlineBreakTokenRareData {
        unsafe { &*self.rare_data_.as_ptr() }
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:62-71
    // cpp: layoutng_fragment_tree/inline_break_token.cc:40-60
    pub fn CreateWithoutRareData(
        node: InlineNode,
        style: *const ComputedStyle,
        start: &InlineItemTextIndex,
        flags: u32,
    ) -> *mut Self {
        Self::Create(
            node,
            style,
            start,
            flags,
            std::ptr::null(),
            std::ptr::null(),
        )
    }

    pub fn CreateWithBlockBreakToken(
        node: InlineNode,
        style: *const ComputedStyle,
        start: &InlineItemTextIndex,
        flags: u32,
        sub_break_token: *const BlockBreakToken,
    ) -> *mut Self {
        Self::Create(node, style, start, flags, sub_break_token, std::ptr::null())
    }

    pub fn Create(
        node: InlineNode,
        style: *const ComputedStyle,
        start: &InlineItemTextIndex,
        mut flags: u32,
        sub_break_token: *const BlockBreakToken,
        ruby_data: *const RubyBreakTokenData,
    ) -> *mut Self {
        let mut size = std::mem::size_of::<Self>();
        let has_rare_data = !sub_break_token.is_null() || !ruby_data.is_null();
        if has_rare_data {
            size += std::mem::size_of::<InlineBreakTokenRareData>();
            flags |= InlineBreakTokenFlag::kHasRareData as u32;
        }

        let base_node = node.into();
        let value = Self {
            base_: BreakToken::new(BreakTokenType::kInlineBreakToken, base_node, flags),
            style_: Member::from_ptr(style as *mut ComputedStyle),
            start_: start.clone(),
            rare_data_: [],
        };
        // Mirrors AdditionalBytes(size), including the source's size-based
        // over-allocation. Foundation must run this initializer before GC
        // registration so Trace cannot observe an uninitialized rare slot.
        unsafe {
            MakeGarbageCollectedWithAdditionalBytes(size, |ptr: *mut Self| {
                ptr.write(value);
                if has_rare_data {
                    let rare_ptr =
                        std::ptr::addr_of_mut!((*ptr).rare_data_) as *mut InlineBreakTokenRareData;
                    rare_ptr.write(InlineBreakTokenRareData {
                        sub_break_token: Member::from_ptr(sub_break_token as *mut BlockBreakToken),
                        ruby_data: Member::from_ptr(ruby_data as *mut RubyBreakTokenData),
                    });
                }
            })
        }
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:73-80
    // cpp: layoutng_fragment_tree/inline_break_token.cc:62-69
    pub fn CreateForParallelBlockFlow(
        node: InlineNode,
        start: &InlineItemTextIndex,
        child_break_token: &BlockBreakToken,
    ) -> *mut Self {
        let style = node.Style() as *const ComputedStyle;
        Self::Create(
            node,
            style,
            start,
            InlineBreakTokenFlag::kIsInParallelBlockFlow as u32,
            child_break_token,
            std::ptr::null(),
        )
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:82-84
    pub fn Style(&self) -> *const ComputedStyle {
        self.style_.Get()
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:86-91
    pub fn Start(&self) -> &InlineItemTextIndex {
        &self.start_
    }

    pub fn StartItemIndex(&self) -> u32 {
        self.start_.item_index
    }

    pub fn StartTextOffset(&self) -> u32 {
        self.start_.text_offset
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:164-170
    pub fn IsStartEqual(lhs: *const Self, rhs: *const Self) -> bool {
        if lhs.is_null() {
            return rhs.is_null();
        }
        !rhs.is_null() && unsafe { &*lhs }.Start() == unsafe { &*rhs }.Start()
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:93-95
    pub fn UseFirstLineStyle(&self) -> bool {
        self.has_flag(InlineBreakTokenFlag::kUseFirstLineStyle)
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:97-101
    pub fn IsLineClampDisplacedLine(&self) -> bool {
        self.has_flag(InlineBreakTokenFlag::kIsLineClampDisplacedLine)
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:103-105
    pub fn IsForcedBreak(&self) -> bool {
        self.has_flag(InlineBreakTokenFlag::kIsForcedBreak)
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:107-108
    // cpp: layoutng_fragment_tree/inline_break_token.cc:26-31
    pub fn GetBlockBreakToken(&self) -> *const BlockBreakToken {
        if !self.has_flag(InlineBreakTokenFlag::kHasRareData) {
            return std::ptr::null();
        }
        self.rare_data().sub_break_token.Get()
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:110-111
    // cpp: layoutng_fragment_tree/inline_break_token.cc:33-38
    pub fn RubyData(&self) -> *const RubyBreakTokenData {
        if !self.has_flag(InlineBreakTokenFlag::kHasRareData) {
            return std::ptr::null();
        }
        self.rare_data().ruby_data.Get()
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:113-117
    pub fn HasClonedBoxDecorations(&self) -> bool {
        self.has_flag(InlineBreakTokenFlag::kHasClonedBoxDecorations)
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:119-121
    pub fn IsInParallelBlockFlow(&self) -> bool {
        self.has_flag(InlineBreakTokenFlag::kIsInParallelBlockFlow)
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:123-128
    pub fn IsPastFirstFormattedLine(&self) -> bool {
        self.has_flag(InlineBreakTokenFlag::kIsPastFirstFormattedLine)
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:130-137
    // cpp: layoutng_fragment_tree/inline_break_token.cc:71-83
    // The PassKey constructor is folded into Create's GC initializer above.
    // cpp: layoutng_fragment_tree/inline_break_token.h:139
    // This separate LayoutInputNode constructor has no supplied definition.

    // cpp: layoutng_fragment_tree/inline_break_token.h:141-143
    // cpp: layoutng_fragment_tree/inline_break_token.cc:87-103
    #[cfg(debug_assertions)]
    pub fn ToString(&self) -> String {
        let mut builder = StringBuilder::default();
        builder.Append("InlineBreakToken index:");
        builder.AppendNumber(self.StartItemIndex());
        builder.Append(" offset:");
        builder.AppendNumber(self.StartTextOffset());
        if self.UseFirstLineStyle() {
            builder.Append(" first-line");
        }
        if self.IsForcedBreak() {
            builder.Append(" forced");
        }
        if self.HasClonedBoxDecorations() {
            builder.Append(" cloned-box-decorations");
        }
        if self.IsInParallelBlockFlow() {
            builder.Append(" parallel-flow");
        }
        builder.ReleaseString()
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:145
    // cpp: layoutng_fragment_tree/inline_break_token.cc:107-115
    pub fn TraceAfterDispatch(&self, visitor: &mut Visitor) {
        if self.has_flag(InlineBreakTokenFlag::kHasRareData) {
            self.rare_data().Trace(visitor);
        }
        visitor.Trace(&self.style_);
        self.base_.TraceAfterDispatch(visitor);
    }

    // cpp: layoutng_fragment_tree/inline_break_token.h:172-177
    pub fn AllowFrom(token: &BreakToken) -> bool {
        token.IsInlineType()
    }
}

// cpp: layoutng_fragment_tree/inline_break_token.cc:17-22
#[repr(C)]
struct SameSizeAsInlineBreakToken {
    base: BreakToken,
    style: Member<ComputedStyle>,
    numbers: [u32; 2],
}

const _: [(); std::mem::size_of::<SameSizeAsInlineBreakToken>()] =
    [(); std::mem::size_of::<InlineBreakToken>()];
