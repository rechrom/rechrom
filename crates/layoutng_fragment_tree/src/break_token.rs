use foundation::{HeapVector, Member, String, StringBuilder, Visitor};
use layoutng::internal::layout_input_node::LayoutInputNode;

use crate::block_break_token::BlockBreakToken;
use crate::inline_break_token::InlineBreakToken;

// LayoutInputNode::kBlock and kInline are 0 and 1 in the source enum.
// cpp: layoutng_fragment_tree/break_token.h:36-39
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakTokenType {
    kBlockBreakToken = 0,
    kInlineBreakToken = 1,
}

// Source bitfields occupy one unsigned word. Keep their exact bit positions
// and let derived tokens embed this base at offset zero for GC dispatch.
// cpp: layoutng_fragment_tree/break_token.h:34-35
// cpp: layoutng_fragment_tree/break_token.h:71-109
#[repr(C)]
pub struct BreakToken {
    pub(crate) bits_: u32,
}

// cpp: layoutng_fragment_tree/break_token.h:111
pub type BreakTokenVector = HeapVector<Member<BreakToken>>;

#[allow(non_snake_case)]
impl BreakToken {
    pub(crate) const TYPE_BIT: u32 = 1 << 0;
    pub(crate) const REPEATED_ACTUAL_BREAK_BIT: u32 = 1 << 1;
    pub(crate) const FLAGS_SHIFT: u32 = 2;
    pub(crate) const FLAGS_MASK: u32 = 0x7f << Self::FLAGS_SHIFT;
    pub(crate) const BREAK_BEFORE_BIT: u32 = 1 << 9;
    pub(crate) const FORCED_BREAK_BIT: u32 = 1 << 10;
    pub(crate) const REPEATED_BIT: u32 = 1 << 11;
    pub(crate) const COLUMN_SPANNER_BIT: u32 = 1 << 12;
    pub(crate) const AT_BLOCK_END_BIT: u32 = 1 << 13;
    pub(crate) const SEEN_ALL_CHILDREN_BIT: u32 = 1 << 14;
    pub(crate) const UNPOSITIONED_LIST_MARKER_BIT: u32 = 1 << 15;

    // C++ leaves the final list-marker bit uninitialized in this constructor;
    // Rust zero-initializes it until a derived block token assigns its value.
    // cpp: layoutng_fragment_tree/break_token.h:58-69
    pub(crate) fn new(token_type: BreakTokenType, node: LayoutInputNode, flags: u32) -> Self {
        debug_assert_eq!(token_type as u8, node.Type() as u8);
        debug_assert_eq!(flags & !0x7f, 0);
        Self {
            bits_: (token_type as u32) | ((flags & 0x7f) << Self::FLAGS_SHIFT),
        }
    }

    // cpp: layoutng_fragment_tree/break_token.h:40
    pub fn Type(&self) -> BreakTokenType {
        if self.bits_ & Self::TYPE_BIT == 0 {
            BreakTokenType::kBlockBreakToken
        } else {
            BreakTokenType::kInlineBreakToken
        }
    }

    // cpp: layoutng_fragment_tree/break_token.h:42-43
    pub fn IsBlockType(&self) -> bool {
        self.Type() == BreakTokenType::kBlockBreakToken
    }

    pub fn IsInlineType(&self) -> bool {
        self.Type() == BreakTokenType::kInlineBreakToken
    }

    pub(crate) fn flags(&self) -> u32 {
        (self.bits_ & Self::FLAGS_MASK) >> Self::FLAGS_SHIFT
    }

    pub(crate) fn bit(&self, bit: u32) -> bool {
        self.bits_ & bit != 0
    }

    pub(crate) fn set_bit(&mut self, bit: u32, value: bool) {
        if value {
            self.bits_ |= bit;
        } else {
            self.bits_ &= !bit;
        }
    }

    // Type is an immutable one-bit discriminant. The checked cast follows the
    // C++ DynamicTo tests; derived structs must have BreakToken first.
    pub(crate) fn as_block(&self) -> Option<&BlockBreakToken> {
        if self.IsBlockType() {
            Some(unsafe { &*(self as *const Self as *const BlockBreakToken) })
        } else {
            None
        }
    }

    pub(crate) fn as_inline(&self) -> Option<&InlineBreakToken> {
        if self.IsInlineType() {
            Some(unsafe { &*(self as *const Self as *const InlineBreakToken) })
        } else {
            None
        }
    }

    // cpp: layoutng_fragment_tree/break_token.h:47
    // cpp: layoutng_fragment_tree/break_token.cc:24-32
    pub fn IsInParallelFlow(&self) -> bool {
        if let Some(block) = self.as_block() {
            return block.IsAtBlockEnd();
        }
        if let Some(inline) = self.as_inline() {
            return inline.IsInParallelBlockFlow();
        }
        false
    }

    // cpp: layoutng_fragment_tree/break_token.h:50
    // cpp: layoutng_fragment_tree/break_token.cc:64-72
    #[cfg(debug_assertions)]
    pub fn ToString(&self) -> String {
        match self.Type() {
            BreakTokenType::kBlockBreakToken => self.as_block().unwrap().ToString(),
            BreakTokenType::kInlineBreakToken => self.as_inline().unwrap().ToString(),
        }
    }

    // cpp: layoutng_fragment_tree/break_token.h:51
    // cpp: layoutng_fragment_tree/break_token.cc:74-79
    #[cfg(debug_assertions)]
    pub fn ShowBreakTokenTree(&self) {
        let mut builder = StringBuilder::default();
        builder.Append(".:: LayoutNG Break Token Tree ::.\n");
        append_break_token_to_string(self, &mut builder, 2);
        eprintln!("{}", builder.ToString());
    }

    // cpp: layoutng_fragment_tree/break_token.h:54
    // cpp: layoutng_fragment_tree/break_token.cc:82-92
    pub fn Trace(&self, visitor: &mut Visitor) {
        match self.Type() {
            BreakTokenType::kBlockBreakToken => {
                self.as_block().unwrap().TraceAfterDispatch(visitor)
            }
            BreakTokenType::kInlineBreakToken => {
                self.as_inline().unwrap().TraceAfterDispatch(visitor)
            }
        }
    }

    // cpp: layoutng_fragment_tree/break_token.h:55
    // cpp: layoutng_fragment_tree/break_token.cc:94
    pub fn TraceAfterDispatch(&self, _visitor: &mut Visitor) {}
}

// cpp: layoutng_fragment_tree/break_token.cc:16-22
// C++ ASSERT_SIZE compares BreakToken with a single unsigned word. This
// structural check covers the Rust base, pending the GC wrapper's layout.
const _: [(); std::mem::size_of::<u32>()] = [(); std::mem::size_of::<BreakToken>()];

// cpp: layoutng_fragment_tree/break_token.cc:38-61
#[cfg(debug_assertions)]
fn append_break_token_to_string(token: &BreakToken, builder: &mut StringBuilder, indent: u32) {
    for _ in 0..indent {
        builder.Append(" ");
    }
    builder.Append(&token.ToString());
    builder.Append("\n");

    if let Some(block) = token.as_block() {
        for child in block.ChildBreakTokens() {
            let child = child.Get();
            if !child.is_null() {
                append_break_token_to_string(unsafe { &*child }, builder, indent + 2);
            }
        }
    } else if let Some(inline) = token.as_inline() {
        let child = inline.GetBlockBreakToken();
        if !child.is_null() {
            append_break_token_to_string(
                unsafe { &*(child as *const BreakToken) },
                builder,
                indent + 2,
            );
        }
    }
}
