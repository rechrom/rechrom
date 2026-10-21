#![allow(non_snake_case, non_camel_case_types)]

use super::algorithm_forward::CustomLayoutChild;
use foundation::{
    kIndefiniteSize, DynamicTo, LayoutUnit, Member, PhysicalSize, String as BlinkString,
    StringBuilder, UnsupportedLayout,
};
use layoutng_geometry::geometry::axis::{
    kLogicalAxesBlock, kLogicalAxesInline, kLogicalAxesNone, LogicalAxes,
};

use super::block_node::BlockNode;
use super::form_node_metadata::TextControlElement;
use super::inline_node::InlineNode;
use super::layout_box::LayoutBox;
use super::layout_node_metadata::Element;
use super::layout_pass_scope::LayoutPassScope;
use super::layout_view::LayoutView;

// cpp: layoutng/internal/layout_input_node.h:27-36
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MinMaxSizesFloatInput {
    pub float_left_inline_size: LayoutUnit,
    pub float_right_inline_size: LayoutUnit,
    pub constrained_inline_size: LayoutUnit,
}

impl Default for MinMaxSizesFloatInput {
    fn default() -> Self {
        Self {
            float_left_inline_size: LayoutUnit::default(),
            float_right_inline_size: LayoutUnit::default(),
            constrained_inline_size: LayoutUnit::Max(),
        }
    }
}

// cpp: layoutng/internal/layout_input_node.h:43-49
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutInputNodeType {
    kBlock,
    kInline,
}

// cpp: layoutng/internal/layout_input_node.h:38-40,254-256
// DISALLOW_NEW is represented by value construction at the call sites. The
// one-bit C++ discriminator occupies one Rust byte and accepts only two values.
#[repr(C)]
pub struct LayoutInputNode {
    pub(crate) box_: Member<LayoutBox>,
    type_: LayoutInputNodeType,
}

impl Clone for LayoutInputNode {
    fn clone(&self) -> Self {
        Self::Create(self.GetLayoutBox(), self.Type())
    }
}

impl PartialEq for LayoutInputNode {
    // cpp: layoutng/internal/layout_input_node.h:229-231
    fn eq(&self, other: &Self) -> bool {
        self.GetLayoutBox() == other.GetLayoutBox() && self.type_ == other.type_
    }
}

impl Eq for LayoutInputNode {}

impl LayoutInputNode {
    // cpp: layoutng/internal/layout_input_node.h:55-59,247-248
    pub fn Create(box_: *mut LayoutBox, type_: LayoutInputNodeType) -> Self {
        Self {
            box_: Member::from_ptr(box_),
            type_,
        }
    }

    // cpp: layoutng/internal/layout_input_node.h:61
    pub fn null() -> Self {
        Self::Create(std::ptr::null_mut(), LayoutInputNodeType::kBlock)
    }

    // cpp: layoutng/internal/layout_input_node.h:63-68
    pub fn Type(&self) -> LayoutInputNodeType {
        self.type_
    }
    pub fn IsInline(&self) -> bool {
        self.type_ == LayoutInputNodeType::kInline
    }
    pub fn IsBlock(&self) -> bool {
        self.type_ == LayoutInputNodeType::kBlock
    }

    // cpp: layoutng/internal/layout_input_node.h:227
    // Rust call sites spell the negation of C++'s explicit operator bool.
    pub fn IsNull(&self) -> bool {
        self.GetLayoutBox().is_null()
    }

    // cpp: layoutng/internal/layout_input_node.h:76-78
    pub fn IsFloatingOrOutOfFlowPositioned(&self) -> bool {
        self.IsFloating() || self.IsOutOfFlowPositioned()
    }

    // cpp: layoutng/internal/layout_input_node.h:182
    pub fn GetLayoutBox(&self) -> *mut LayoutBox {
        self.box_.Get()
    }

    // cpp: layoutng/internal/layout_input_node.h:198-205
    pub fn ContainedAxes(&self) -> LogicalAxes {
        let mut axes = kLogicalAxesNone;
        if self.ShouldApplyInlineSizeContainment() {
            axes |= kLogicalAxesInline;
        }
        if self.ShouldApplyBlockSizeContainment() {
            axes |= kLogicalAxesBlock;
        }
        axes
    }

    // cpp: layoutng/internal/layout_input_node.h:117
    // The body is owned by //src/layoutng_table/layout_table_section.cc.
    pub fn IsEmptyTableSection(&self) -> bool {
        unsafe { LayoutInputNodeIsEmptyTableSectionFromTable(self) }
    }

    // cpp: layoutng/internal/layout_input_node.h:123-127
    // These bodies are owned by //src/layoutng_table.
    pub fn TableColumnSpan(&self) -> u32 {
        unsafe { LayoutInputNodeTableColumnSpanFromTable(self) }
    }

    pub fn TableCellColspan(&self) -> u32 {
        unsafe { LayoutInputNodeTableCellColspanFromTable(self) }
    }

    pub fn TableCellRowspan(&self) -> u32 {
        unsafe { LayoutInputNodeTableCellRowspanFromTable(self) }
    }

    // cpp: layoutng/internal/layout_input_node.h:218
    // The body is owned by //src/layoutng_custom/custom_layout_child.cc.
    pub fn GetCustomLayoutChild(&self) -> *mut CustomLayoutChild {
        unsafe { LayoutInputNodeGetCustomLayoutChildFromCustom(self) }
    }

    // cpp: layoutng/internal/layout_input_node.h:227
    pub fn is_non_null(&self) -> bool {
        !self.GetLayoutBox().is_null()
    }

    // cpp: layoutng/internal/layout_input_node.cc:102-104
    pub fn IsSvgText(&self) -> bool {
        self.is_non_null() && unsafe { &*self.GetLayoutBox() }.IsSVGText()
    }

    // cpp: layoutng/internal/layout_input_node.cc:106-110
    pub fn IsTextControlPlaceholder(&self) -> bool {
        let element = DynamicTo::<Element>(self.GetDOMNode());
        self.IsBlock()
            && !element.is_null()
            && unsafe { &*element }.InputIsTextControlPlaceholder()
            && !DynamicTo::<TextControlElement>(unsafe { &*element }.OwnerShadowHost()).is_null()
    }

    // cpp: layoutng/internal/layout_input_node.cc:112-117
    pub fn IsPaginatedRoot(&self) -> bool {
        if !self.IsBlock() {
            return false;
        }
        let view = DynamicTo::<LayoutView>(self.GetLayoutBox());
        !view.is_null() && unsafe { &*view }.IsFragmentationContextRoot()
    }

    // cpp: layoutng/internal/layout_input_node.cc:119-126
    pub fn ListMarkerBlockNodeIfListItem(&self) -> BlockNode {
        let box_ = unsafe { &*self.GetLayoutBox() };
        if box_.IsLayoutListItem() {
            let item = DynamicTo::<Element>(box_.GetNode());
            let marker = if item.is_null() {
                std::ptr::null_mut()
            } else {
                unsafe { &*item }.InputListMarkerLayoutObject()
            };
            return BlockNode::new(DynamicTo::<LayoutBox>(marker));
        }
        BlockNode::new(std::ptr::null_mut())
    }

    // cpp: layoutng/internal/layout_input_node.cc:128-162
    pub fn IntrinsicSize(
        &self,
        computed_inline_size: &mut Option<LayoutUnit>,
        computed_block_size: &mut Option<LayoutUnit>,
    ) {
        debug_assert!(self.IsReplaced());
        self.GetOverrideIntrinsicSize(computed_inline_size, computed_block_size);
        if computed_inline_size.is_some() && computed_block_size.is_some() {
            return;
        }
        let algorithms = LayoutPassScope::Algorithms();
        let callback = if algorithms.is_null() {
            None
        } else {
            unsafe { &*algorithms }.replaced_sizing.natural_sizing_info
        };
        let callback = callback.unwrap_or_else(|| {
            std::panic::panic_any(UnsupportedLayout::new(
                "replaced layout module is not installed",
            ))
        });
        let legacy_sizing_info = callback(self);
        let mut intrinsic_inline_size = legacy_sizing_info
            .has_width
            .then_some(legacy_sizing_info.size.width);
        let mut intrinsic_block_size = legacy_sizing_info
            .has_height
            .then_some(legacy_sizing_info.size.height);
        if !self.IsHorizontalWritingMode() {
            std::mem::swap(&mut intrinsic_inline_size, &mut intrinsic_block_size);
        }
        if computed_inline_size.is_none() {
            *computed_inline_size = intrinsic_inline_size;
        }
        if computed_block_size.is_none() {
            *computed_block_size = intrinsic_block_size;
        }
    }

    // cpp: layoutng/internal/layout_input_node.cc:164-167
    pub fn NextSibling(&self) -> LayoutInputNode {
        if self.IsInline() {
            LayoutInputNode::null()
        } else {
            self.as_block_node().NextSibling()
        }
    }

    // cpp: layoutng/internal/layout_input_node.cc:169-174
    pub fn InitialContainingBlockSize(&self) -> PhysicalSize {
        let owner = unsafe { &*self.GetLayoutBox() }.InputOwnerForLayout();
        let viewport = owner
            .InputViewport()
            .as_ref()
            .expect("initial containing block requires viewport input");
        PhysicalSize::new(
            LayoutUnit::from_signed(viewport.size.width),
            LayoutUnit::from_signed(viewport.size.height),
        )
    }

    // cpp: layoutng/internal/layout_input_node.cc:176-178
    pub fn FirstLineStyle(&self) -> &layoutng_style::style::computed_style::ComputedStyle {
        unsafe { &*self.GetLayoutBox() }.FirstLineStyleRef()
    }

    // cpp: layoutng/internal/layout_input_node.cc:180-184
    pub fn ToString(&self) -> BlinkString {
        if self.IsInline() {
            self.as_inline_node().ToString()
        } else {
            self.as_block_node().ToString()
        }
    }

    // cpp: layoutng/internal/layout_input_node.cc:209-227
    pub fn GetOverrideIntrinsicSize(
        &self,
        computed_inline_size: &mut Option<LayoutUnit>,
        computed_block_size: &mut Option<LayoutUnit>,
    ) {
        debug_assert!(self.IsReplaced());
        let override_inline_size = self.OverrideIntrinsicContentInlineSize();
        if override_inline_size != kIndefiniteSize {
            *computed_inline_size = Some(override_inline_size);
        } else if self.ShouldApplyInlineSizeContainment() {
            *computed_inline_size = Some(LayoutUnit::default());
        }
        let override_block_size = self.OverrideIntrinsicContentBlockSize();
        if override_block_size != kIndefiniteSize {
            *computed_block_size = Some(override_block_size);
        } else if self.ShouldApplyBlockSizeContainment() {
            *computed_block_size = Some(LayoutUnit::default());
        }
    }

    // The C++ downcasts are valid because BlockNode and InlineNode add no
    // fields; their Rust owners must embed LayoutInputNode at offset zero.
    fn as_block_node(&self) -> &BlockNode {
        debug_assert!(self.IsBlock());
        unsafe { &*(self as *const LayoutInputNode).cast::<BlockNode>() }
    }
    fn as_inline_node(&self) -> &InlineNode {
        debug_assert!(self.IsInline());
        unsafe { &*(self as *const LayoutInputNode).cast::<InlineNode>() }
    }

    // cpp: layoutng/internal/layout_input_node.cc:187-192
    #[cfg(debug_assertions)]
    pub fn DumpNodeTree(&self, target: *const LayoutInputNode) -> BlinkString {
        let mut string_builder = StringBuilder::default();
        string_builder.Append(".:: Layout input node tree ::.\n");
        AppendNodeToString(self, target, &mut string_builder, 2);
        string_builder.ToString()
    }

    // cpp: layoutng/internal/layout_input_node.cc:194-196
    #[cfg(debug_assertions)]
    pub fn DumpNodeTreeFromRoot(&self) -> BlinkString {
        let view = unsafe { &*self.GetLayoutBox() }.View();
        let root = BlockNode::new(view.cast::<LayoutBox>());
        unsafe { &*(&root as *const BlockNode).cast::<LayoutInputNode>() }.DumpNodeTree(self)
    }

    // cpp: layoutng/internal/layout_input_node.cc:198-201
    #[cfg(debug_assertions)]
    pub fn ShowNodeTree(&self, target: *const LayoutInputNode) {
        let utf8 = self.DumpNodeTree(target).Utf8();
        eprintln!("\n{}\n", utf8);
    }

    // cpp: layoutng/internal/layout_input_node.cc:203-206
    #[cfg(debug_assertions)]
    pub fn ShowNodeTreeFromRoot(&self) {
        let view = if self.is_non_null() {
            unsafe { &*self.GetLayoutBox() }.View()
        } else {
            std::ptr::null_mut()
        };
        let root = BlockNode::new(view.cast::<LayoutBox>());
        unsafe { &*(&root as *const BlockNode).cast::<LayoutInputNode>() }.ShowNodeTree(self);
    }
}

// Cross-package method definitions have to be linked through typed functions:
// Rust cannot add inherent methods to LayoutInputNode from another crate.
unsafe extern "Rust" {
    fn LayoutInputNodeIsEmptyTableSectionFromTable(node: &LayoutInputNode) -> bool;
    fn LayoutInputNodeTableColumnSpanFromTable(node: &LayoutInputNode) -> u32;
    fn LayoutInputNodeTableCellColspanFromTable(node: &LayoutInputNode) -> u32;
    fn LayoutInputNodeTableCellRowspanFromTable(node: &LayoutInputNode) -> u32;
    fn LayoutInputNodeGetCustomLayoutChildFromCustom(
        node: &LayoutInputNode,
    ) -> *mut CustomLayoutChild;
}

// cpp: layoutng/internal/layout_input_node.cc:39-51
#[cfg(debug_assertions)]
fn IndentForDump(
    node: &LayoutInputNode,
    target: *const LayoutInputNode,
    string_builder: &mut StringBuilder,
    indent: u32,
) {
    let mut start_col = 0;
    if node.is_non_null() && !target.is_null() && node == unsafe { &*target } {
        string_builder.Append("*");
        start_col = 1;
    }
    for _ in start_col..indent {
        string_builder.Append(" ");
    }
}

// cpp: layoutng/internal/layout_input_node.cc:53-86
#[cfg(debug_assertions)]
fn AppendNodeToString(
    node: &LayoutInputNode,
    target: *const LayoutInputNode,
    string_builder: &mut StringBuilder,
    indent: u32,
) {
    if !node.is_non_null() {
        return;
    }
    IndentForDump(node, target, string_builder, indent);
    string_builder.Append(node.ToString());
    string_builder.Append("\n");
    if node.IsBlock() {
        AppendSubtreeToString(node.as_block_node(), target, string_builder, indent + 2);
    } else {
        let inline_node = node.as_inline_node();
        let items = &inline_node.ItemsData(false).items;
        let indent = indent + 2;
        for inline_item_ptr in items.iter() {
            let inline_item = unsafe { &*inline_item_ptr.Get() };
            let box_ = DynamicTo::<LayoutBox>(inline_item.GetLayoutObject());
            let child_node = BlockNode::new(box_);
            let child_input =
                unsafe { &*(&child_node as *const BlockNode).cast::<LayoutInputNode>() };
            IndentForDump(child_input, target, string_builder, indent);
            string_builder.Append(inline_item.ToString());
            string_builder.Append("\n");
            if child_input.is_non_null() {
                AppendSubtreeToString(&child_node, target, string_builder, indent + 2);
            }
        }
        debug_assert!(!node.NextSibling().is_non_null());
    }
}

// cpp: layoutng/internal/layout_input_node.cc:88-97
#[cfg(debug_assertions)]
fn AppendSubtreeToString(
    node: &BlockNode,
    target: *const LayoutInputNode,
    string_builder: &mut StringBuilder,
    indent: u32,
) {
    let mut node_runner = node.FirstChild();
    while node_runner.is_non_null() {
        AppendNodeToString(&node_runner, target, string_builder, indent);
        node_runner = node_runner.NextSibling();
    }
}

// cpp: layoutng/internal/layout_input_node.cc:233-235
#[cfg(debug_assertions)]
pub fn ShowLayoutInputNodeTree(node: &LayoutInputNode) {
    super::layout_object::ShowLayoutTree(node.GetLayoutBox().cast());
}
