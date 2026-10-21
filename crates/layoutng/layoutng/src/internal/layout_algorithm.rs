#![allow(non_snake_case, non_camel_case_types)]

use std::marker::PhantomData;

use foundation::{HeapVector, LayoutUnit, Member, TextDirection, WritingDirectionMode};
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::box_fragment_builder::BoxFragmentBuilder;
use layoutng_fragment_tree::fragment_builder::FragmentBuilder;
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_geometry::geometry::bfc_offset::BfcOffset;
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::fragment_geometry::FragmentGeometry;
use layoutng_geometry::geometry::logical_size::LogicalSize;
use layoutng_style::style::computed_style::ComputedStyle;

use super::algorithm_entry::NativeAlgorithm;
use super::block_node::BlockNode;
use super::break_appeal::BreakAppeal;
use super::column_spanner_path::ColumnSpannerPath;
use super::constraint_space::ConstraintSpace;
use super::early_break::EarlyBreak;
use super::exclusions::exclusion_space::ExclusionSpace;
use super::fragmentation_utils::{self, BreakStatus};
use super::gap::gap_geometry::GapGeometry;
use super::layout_input_node::LayoutInputNode;

// C++ stack references retain their caller lifetime; optional source pointers
// stay raw so they do not imply ownership of break or previous-result state.
// cpp: layoutng/internal/layout_algorithm.h:23-41
#[repr(C)]
pub struct LayoutAlgorithmParams<'a> {
    pub node: BlockNode,
    // C++ stores a const reference while the caller may recompute the same
    // geometry between algorithm runs. A raw pointer keeps that aliasing
    // contract explicit without a lasting Rust shared borrow.
    pub fragment_geometry: *const FragmentGeometry,
    pub space: &'a ConstraintSpace,
    pub break_token: *const BlockBreakToken,
    pub early_break: *const EarlyBreak,
    pub column_spanner_path: *const ColumnSpannerPath,
    pub previous_result: *const LayoutResult,
    pub additional_early_breaks: *const HeapVector<Member<EarlyBreak>>,
}

impl<'a> LayoutAlgorithmParams<'a> {
    // cpp: layoutng/internal/layout_algorithm.h:27-31
    pub fn new(
        node: BlockNode,
        fragment_geometry: &FragmentGeometry,
        space: &'a ConstraintSpace,
    ) -> Self {
        Self {
            node,
            fragment_geometry: fragment_geometry as *const FragmentGeometry,
            space,
            break_token: std::ptr::null(),
            early_break: std::ptr::null(),
            column_spanner_path: std::ptr::null(),
            previous_result: std::ptr::null(),
            additional_early_breaks: std::ptr::null(),
        }
    }
}

// The template's input and builder APIs are expressed as Rust traits. Concrete
// builder adapters preserve their owning crate and do not copy its behavior.
// cpp: layoutng/internal/layout_algorithm.h:58-71
pub trait LayoutAlgorithmInputNode: Clone {
    fn Style(&self) -> &ComputedStyle;
}

pub trait LayoutAlgorithmFromBlockNode: Sized {
    fn FromBlockNode(node: BlockNode) -> Self;
    fn ToBlockNode(&self) -> BlockNode;
}

impl LayoutAlgorithmInputNode for BlockNode {
    fn Style(&self) -> &ComputedStyle {
        self.base.Style()
    }
}

impl LayoutAlgorithmFromBlockNode for BlockNode {
    fn FromBlockNode(node: BlockNode) -> Self {
        node
    }
    fn ToBlockNode(&self) -> BlockNode {
        self.clone()
    }
}

// TableNode is the source table specialization's zero-data BlockNode wrapper.
impl LayoutAlgorithmInputNode for super::table_node::TableNode {
    fn Style(&self) -> &ComputedStyle {
        self.base.Style()
    }
}
impl LayoutAlgorithmFromBlockNode for super::table_node::TableNode {
    fn FromBlockNode(node: BlockNode) -> Self {
        Self { base: node }
    }
    fn ToBlockNode(&self) -> BlockNode {
        self.base.clone()
    }
}

pub trait LayoutAlgorithmBuilder<Node, Token>: Sized {
    fn New(
        node: Node,
        style: *const ComputedStyle,
        space: &ConstraintSpace,
        direction: WritingDirectionMode,
        break_token: *const Token,
    ) -> Self;
    fn GetGapGeometry(&self) -> *const GapGeometry;
    fn GetConstraintSpace(&self) -> &ConstraintSpace;
    fn BfcLineOffset(&self) -> LayoutUnit;
    fn BfcBlockOffset(&self) -> Option<LayoutUnit>;
    fn PreviousBreakToken(&self) -> *const Token;
    fn Borders(&self) -> &BoxStrut;
    fn Scrollbar(&self) -> &BoxStrut;
    fn Padding(&self) -> &BoxStrut;
    fn BorderPadding(&self) -> &BoxStrut;
    fn BorderScrollbarPadding(&self) -> &BoxStrut;
    fn OriginalBorderScrollbarPaddingBlockStart(&self) -> LayoutUnit;
    fn ChildAvailableSize(&self) -> &LogicalSize;
    fn GetExclusionSpace(&mut self) -> &mut ExclusionSpace;
}

// These two inherited builder operations are also used by line builders,
// which do not expose the box-only border and padding members above.
pub trait LayoutAlgorithmCoreBuilder<Token> {
    fn PreviousBreakToken(&self) -> *const Token;
    fn GetExclusionSpace(&mut self) -> &mut ExclusionSpace;
}

impl LayoutAlgorithmCoreBuilder<BlockBreakToken> for BoxFragmentBuilder {
    fn PreviousBreakToken(&self) -> *const BlockBreakToken {
        BoxFragmentBuilder::PreviousBreakToken(self)
    }
    fn GetExclusionSpace(&mut self) -> &mut ExclusionSpace {
        FragmentBuilder::GetExclusionSpace(std::ops::DerefMut::deref_mut(self))
    }
}

impl LayoutAlgorithmBuilder<BlockNode, BlockBreakToken> for BoxFragmentBuilder {
    fn New(
        node: BlockNode,
        style: *const ComputedStyle,
        space: &ConstraintSpace,
        direction: WritingDirectionMode,
        break_token: *const BlockBreakToken,
    ) -> Self {
        BoxFragmentBuilder::new(node.base, style, space, direction, break_token)
    }
    fn GetGapGeometry(&self) -> *const GapGeometry {
        BoxFragmentBuilder::GetGapGeometry(self)
    }
    fn GetConstraintSpace(&self) -> &ConstraintSpace {
        FragmentBuilder::GetConstraintSpace(std::ops::Deref::deref(self))
    }
    fn BfcLineOffset(&self) -> LayoutUnit {
        FragmentBuilder::BfcLineOffset(std::ops::Deref::deref(self))
    }
    fn BfcBlockOffset(&self) -> Option<LayoutUnit> {
        *FragmentBuilder::BfcBlockOffset(std::ops::Deref::deref(self))
    }
    fn PreviousBreakToken(&self) -> *const BlockBreakToken {
        BoxFragmentBuilder::PreviousBreakToken(self)
    }
    fn Borders(&self) -> &BoxStrut {
        BoxFragmentBuilder::Borders(self)
    }
    fn Scrollbar(&self) -> &BoxStrut {
        BoxFragmentBuilder::Scrollbar(self)
    }
    fn Padding(&self) -> &BoxStrut {
        BoxFragmentBuilder::Padding(self)
    }
    fn BorderPadding(&self) -> &BoxStrut {
        BoxFragmentBuilder::BorderPadding(self)
    }
    fn BorderScrollbarPadding(&self) -> &BoxStrut {
        BoxFragmentBuilder::BorderScrollbarPadding(self)
    }
    fn OriginalBorderScrollbarPaddingBlockStart(&self) -> LayoutUnit {
        BoxFragmentBuilder::OriginalBorderScrollbarPaddingBlockStart(self)
    }
    fn ChildAvailableSize(&self) -> &LogicalSize {
        BoxFragmentBuilder::ChildAvailableSize(self)
    }
    fn GetExclusionSpace(&mut self) -> &mut ExclusionSpace {
        FragmentBuilder::GetExclusionSpace(std::ops::DerefMut::deref_mut(self))
    }
}

impl LayoutAlgorithmBuilder<super::table_node::TableNode, BlockBreakToken> for BoxFragmentBuilder {
    fn New(
        node: super::table_node::TableNode,
        style: *const ComputedStyle,
        space: &ConstraintSpace,
        direction: WritingDirectionMode,
        break_token: *const BlockBreakToken,
    ) -> Self {
        BoxFragmentBuilder::new(node.base.base, style, space, direction, break_token)
    }
    fn GetGapGeometry(&self) -> *const GapGeometry {
        BoxFragmentBuilder::GetGapGeometry(self)
    }
    fn GetConstraintSpace(&self) -> &ConstraintSpace {
        FragmentBuilder::GetConstraintSpace(std::ops::Deref::deref(self))
    }
    fn BfcLineOffset(&self) -> LayoutUnit {
        FragmentBuilder::BfcLineOffset(std::ops::Deref::deref(self))
    }
    fn BfcBlockOffset(&self) -> Option<LayoutUnit> {
        *FragmentBuilder::BfcBlockOffset(std::ops::Deref::deref(self))
    }
    fn PreviousBreakToken(&self) -> *const BlockBreakToken {
        BoxFragmentBuilder::PreviousBreakToken(self)
    }
    fn Borders(&self) -> &BoxStrut {
        BoxFragmentBuilder::Borders(self)
    }
    fn Scrollbar(&self) -> &BoxStrut {
        BoxFragmentBuilder::Scrollbar(self)
    }
    fn Padding(&self) -> &BoxStrut {
        BoxFragmentBuilder::Padding(self)
    }
    fn BorderPadding(&self) -> &BoxStrut {
        BoxFragmentBuilder::BorderPadding(self)
    }
    fn BorderScrollbarPadding(&self) -> &BoxStrut {
        BoxFragmentBuilder::BorderScrollbarPadding(self)
    }
    fn OriginalBorderScrollbarPaddingBlockStart(&self) -> LayoutUnit {
        BoxFragmentBuilder::OriginalBorderScrollbarPaddingBlockStart(self)
    }
    fn ChildAvailableSize(&self) -> &LogicalSize {
        BoxFragmentBuilder::ChildAvailableSize(self)
    }
    fn GetExclusionSpace(&mut self) -> &mut ExclusionSpace {
        FragmentBuilder::GetExclusionSpace(std::ops::DerefMut::deref_mut(self))
    }
}

// cpp: layoutng/internal/layout_algorithm.h:107-120
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelayoutType {
    kNoRelayout = 0,
    kRelayoutForEarlyBreak = 1,
    kRelayoutIgnoringLineClamp = 2,
    kRelayoutClampingByLines = 4,
    kRelayoutForTextBoxTrim = 8,
    kRelayoutWithoutFragmentation = 16,
    kRelayoutIgnoringChildScrollbarChanges = 32,
    kRelayoutAsLastTableBox = 64,
    kRelayoutClampingAfterLayoutObject = 128,
    kRelayoutForMarginTrim = 256,
}

pub type RelayoutMode = i32;

// C++ subclasses embed this base first; the future block algorithm owner can
// use the same field order for its inherited methods and relayout callback.
// cpp: layoutng/internal/layout_algorithm.h:58-69
// cpp: layoutng/internal/layout_algorithm.h:293-309
#[repr(C)]
pub struct LayoutAlgorithm<Node, Builder, Token> {
    pub node_: Node,
    pub early_break_: *const EarlyBreak,
    pub container_builder_: Builder,
    pub additional_early_breaks_: *const HeapVector<Member<EarlyBreak>>,
    pub relayout_mode_: RelayoutMode,
    token_: PhantomData<*const Token>,
}

impl<Node, Builder, Token> LayoutAlgorithm<Node, Builder, Token> {
    // The C++ template constructor receives a builder selected by Node and
    // Builder. This entry keeps the same base-field initialization when a
    // source-owned algorithm supplies its own derived fragment builder.
    // cpp: layoutng/internal/layout_algorithm.h:64-74
    pub fn new_with_builder(node: Node, container_builder: Builder) -> Self {
        Self {
            node_: node,
            early_break_: std::ptr::null(),
            container_builder_: container_builder,
            additional_early_breaks_: std::ptr::null(),
            relayout_mode_: 0,
            token_: PhantomData,
        }
    }
}

impl<Node, Builder, Token> LayoutAlgorithm<Node, Builder, Token>
where
    Node: LayoutAlgorithmInputNode,
    Builder: LayoutAlgorithmBuilder<Node, Token>,
{
    // cpp: layoutng/internal/layout_algorithm.h:64-74
    pub fn new(
        node: Node,
        style: *const ComputedStyle,
        space: &ConstraintSpace,
        direction: TextDirection,
        break_token: *const Token,
    ) -> Self {
        let writing_direction = WritingDirectionMode::new(space.GetWritingMode(), direction);
        let container_builder_ =
            Builder::New(node.clone(), style, space, writing_direction, break_token);
        Self {
            node_: node,
            early_break_: std::ptr::null(),
            container_builder_,
            additional_early_breaks_: std::ptr::null(),
            relayout_mode_: 0,
            token_: PhantomData,
        }
    }

    // cpp: layoutng/internal/layout_algorithm.h:98-100
    pub fn GetGapGeometry(&self) -> *const GapGeometry {
        self.container_builder_.GetGapGeometry()
    }

    // cpp: layoutng/internal/layout_algorithm.h:122-158
    pub fn GetConstraintSpace(&self) -> &ConstraintSpace {
        self.container_builder_.GetConstraintSpace()
    }
    pub fn Style(&self) -> &ComputedStyle {
        self.node_.Style()
    }
    pub fn ContainerBfcOffset(&self) -> BfcOffset {
        let block_offset = self.container_builder_.BfcBlockOffset();
        debug_assert!(block_offset.is_some());
        BfcOffset::new(
            self.container_builder_.BfcLineOffset(),
            block_offset.expect("BFC block offset"),
        )
    }
    pub fn Node(&self) -> &Node {
        &self.node_
    }
    pub fn Borders(&self) -> &BoxStrut {
        self.container_builder_.Borders()
    }
    pub fn Scrollbar(&self) -> &BoxStrut {
        self.container_builder_.Scrollbar()
    }
    pub fn Padding(&self) -> &BoxStrut {
        self.container_builder_.Padding()
    }
    pub fn BorderPadding(&self) -> &BoxStrut {
        self.container_builder_.BorderPadding()
    }
    pub fn BorderScrollbarPadding(&self) -> &BoxStrut {
        self.container_builder_.BorderScrollbarPadding()
    }
    pub fn OriginalBorderScrollbarPaddingBlockStart(&self) -> LayoutUnit {
        self.container_builder_
            .OriginalBorderScrollbarPaddingBlockStart()
    }
    pub fn ChildAvailableSize(&self) -> &LogicalSize {
        self.container_builder_.ChildAvailableSize()
    }
}

impl<Node, Builder, Token> LayoutAlgorithm<Node, Builder, Token>
where
    Builder: LayoutAlgorithmCoreBuilder<Token>,
{
    pub fn GetBreakToken(&self) -> *const Token {
        self.container_builder_.PreviousBreakToken()
    }

    pub fn GetExclusionSpace(&mut self) -> &mut ExclusionSpace {
        self.container_builder_.GetExclusionSpace()
    }
}

// This constructor converts BlockNode to the specialization's input-node type.
// The current package supplies BlockNode; other algorithm packages can provide
// their own conversion without copying the constructor's body.
impl<Node: LayoutAlgorithmFromBlockNode>
    LayoutAlgorithm<Node, BoxFragmentBuilder, BlockBreakToken>
{
    // cpp: layoutng/internal/layout_algorithm.h:78-96
    pub fn from_params(params: &LayoutAlgorithmParams) -> Self {
        let node = Node::FromBlockNode(params.node.clone());
        let mut container_builder_ = BoxFragmentBuilder::new(
            params.node.base.clone(),
            params.node.Style(),
            params.space,
            WritingDirectionMode::new(params.space.GetWritingMode(), params.space.Direction()),
            params.break_token,
        );
        container_builder_.SetIsNewFormattingContext(params.space.IsNewFormattingContext());
        container_builder_.SetInitialFragmentGeometry(unsafe { &*params.fragment_geometry });
        if params.space.HasBlockFragmentation()
            || fragmentation_utils::IsBreakInside(params.break_token)
        {
            fragmentation_utils::SetupFragmentBuilderForFragmentation(
                params.space,
                params.node.clone(),
                params.break_token,
                &mut container_builder_,
            );
        }
        Self {
            node_: node,
            early_break_: params.early_break,
            container_builder_,
            additional_early_breaks_: params.additional_early_breaks,
            relayout_mode_: RelayoutType::kNoRelayout as i32,
            token_: PhantomData,
        }
    }
}

// Fragmentation and relayout require BoxFragmentBuilder and BlockBreakToken.
// Future node specializations supply their own node and builder adapters.
impl<Node> LayoutAlgorithm<Node, BoxFragmentBuilder, BlockBreakToken>
where
    Node: LayoutAlgorithmInputNode + LayoutAlgorithmFromBlockNode,
    BoxFragmentBuilder: LayoutAlgorithmBuilder<Node, BlockBreakToken>,
{
    // cpp: layoutng/internal/layout_algorithm.h:160-197
    pub fn FragmentainerCapacityForChildren(&self) -> LayoutUnit {
        fragmentation_utils::FragmentainerCapacity(&self.container_builder_, true)
    }
    pub fn FragmentainerOffsetForChildren(&self) -> LayoutUnit {
        fragmentation_utils::FragmentainerOffset(&self.container_builder_, true)
    }
    pub fn FragmentainerSpaceLeftForChildren(&self) -> LayoutUnit {
        fragmentation_utils::FragmentainerSpaceLeft(&self.container_builder_, true)
    }
    pub fn BreakBeforeChildIfNeeded(
        &mut self,
        child: LayoutInputNode,
        result: &LayoutResult,
        block_offset: LayoutUnit,
        has_container_separation: bool,
    ) -> BreakStatus {
        let capacity = self.FragmentainerCapacityForChildren();
        fragmentation_utils::BreakBeforeChildIfNeeded(
            child,
            result,
            block_offset,
            capacity,
            has_container_separation,
            &mut self.container_builder_,
        )
    }
    pub fn MovePastBreakpoint(
        &mut self,
        child: LayoutInputNode,
        result: &LayoutResult,
        block_offset: LayoutUnit,
        appeal: BreakAppeal,
    ) -> bool {
        let capacity = self.FragmentainerCapacityForChildren();
        let space = self.GetConstraintSpace() as *const ConstraintSpace;
        fragmentation_utils::MovePastBreakpoint(
            unsafe { &*space },
            child,
            result,
            block_offset,
            capacity,
            appeal,
            &mut self.container_builder_,
        )
    }
    pub fn MovePastBreakpointWithoutChild(
        &mut self,
        result: &LayoutResult,
        block_offset: LayoutUnit,
        appeal: BreakAppeal,
    ) -> bool {
        let capacity = self.FragmentainerCapacityForChildren();
        let space = self.GetConstraintSpace() as *const ConstraintSpace;
        fragmentation_utils::MovePastBreakpointWithoutChild(
            unsafe { &*space },
            result,
            block_offset,
            capacity,
            appeal,
            &mut self.container_builder_,
        )
    }

    // cpp: layoutng/internal/layout_algorithm.h:203-224
    pub fn SetupRelayoutData(&mut self, previous: &Self, relayout_type: RelayoutType) {
        if self.relayout_mode_ & RelayoutType::kRelayoutWithoutFragmentation as i32 != 0 {
            self.container_builder_
                .SetPageNameIfNeeded(previous.container_builder_.PageName().clone());
        }
        if self.relayout_mode_ & RelayoutType::kRelayoutForEarlyBreak as i32 != 0 {
            self.container_builder_
                .PropagateSpaceShortage(previous.container_builder_.MinimalSpaceShortage());
        }
        if relayout_type != RelayoutType::kRelayoutForEarlyBreak {
            self.early_break_ = previous.early_break_;
            self.additional_early_breaks_ = previous.additional_early_breaks_;
        }
        self.container_builder_
            .SetBoxType(previous.container_builder_.GetBoxType());
    }

    // cpp: layoutng/internal/layout_algorithm.h:227-265
    pub fn Relayout<A: RelayoutAlgorithm<Node>>(
        &self,
        relayout_type: RelayoutType,
        breakpoint: *const EarlyBreak,
        additional_early_breaks: *const HeapVector<Member<EarlyBreak>>,
    ) -> *const LayoutResult {
        debug_assert_eq!(self.relayout_mode_ & relayout_type as i32, 0);
        debug_assert!(
            breakpoint.is_null() || relayout_type == RelayoutType::kRelayoutForEarlyBreak
        );
        debug_assert!(
            additional_early_breaks.is_null()
                || relayout_type == RelayoutType::kRelayoutForEarlyBreak
        );
        let new_relayout_mode = self.relayout_mode_ | relayout_type as i32;
        let new_space =
            if new_relayout_mode & RelayoutType::kRelayoutWithoutFragmentation as i32 != 0 {
                Some(self.GetConstraintSpace().CloneWithoutFragmentation())
            } else {
                None
            };
        let space = new_space.as_ref().unwrap_or(self.GetConstraintSpace());
        let mut params = LayoutAlgorithmParams::new(
            self.Node().ToBlockNode(),
            self.container_builder_.InitialFragmentGeometry(),
            space,
        );
        params.break_token = self.GetBreakToken();
        params.early_break = breakpoint;
        params.additional_early_breaks = additional_early_breaks;
        let mut relayout_algorithm = A::new(&params);
        relayout_algorithm.base_mut().relayout_mode_ = new_relayout_mode;
        let previous = unsafe { A::from_base(self) };
        relayout_algorithm.setup_relayout_data(previous, relayout_type);
        relayout_algorithm.layout()
    }

    pub fn RelayoutDefault<A: RelayoutAlgorithm<Node>>(
        &self,
        relayout_type: RelayoutType,
    ) -> *const LayoutResult {
        self.Relayout::<A>(relayout_type, std::ptr::null(), std::ptr::null())
    }

    // cpp: layoutng/internal/layout_algorithm.h:271-281
    pub fn RelayoutAndBreakEarlier<A: RelayoutAlgorithm<Node>>(
        &mut self,
        breakpoint: &EarlyBreak,
        additional_early_breaks: *const HeapVector<Member<EarlyBreak>>,
    ) -> *const LayoutResult {
        debug_assert!(self.early_break_.is_null());
        debug_assert!(
            self.additional_early_breaks_.is_null()
                || unsafe { &*self.additional_early_breaks_ }.is_empty()
        );
        self.Relayout::<A>(
            RelayoutType::kRelayoutForEarlyBreak,
            breakpoint,
            additional_early_breaks,
        )
    }

    pub fn RelayoutAndBreakEarlierDefault<A: RelayoutAlgorithm<Node>>(
        &mut self,
        breakpoint: &EarlyBreak,
    ) -> *const LayoutResult {
        self.RelayoutAndBreakEarlier::<A>(breakpoint, std::ptr::null())
    }

    // cpp: layoutng/internal/layout_algorithm.h:287-291
    pub fn RelayoutWithoutFragmentation<A: RelayoutAlgorithm<Node>>(
        &mut self,
    ) -> *const LayoutResult {
        debug_assert!(self.GetConstraintSpace().HasBlockFragmentation());
        self.Relayout::<A>(
            RelayoutType::kRelayoutWithoutFragmentation,
            std::ptr::null(),
            std::ptr::null(),
        )
    }
}

// The C++ static_cast from a base subobject to its concrete algorithm is an
// explicit unsafe contract; a derived owner must prove its base-first layout.
pub trait RelayoutAlgorithm<Node>: NativeAlgorithm
where
    Node: LayoutAlgorithmInputNode + LayoutAlgorithmFromBlockNode,
    BoxFragmentBuilder: LayoutAlgorithmBuilder<Node, BlockBreakToken>,
{
    fn base(&self) -> &LayoutAlgorithm<Node, BoxFragmentBuilder, BlockBreakToken>;
    fn base_mut(&mut self) -> &mut LayoutAlgorithm<Node, BoxFragmentBuilder, BlockBreakToken>;
    unsafe fn from_base<'a>(
        base: &'a LayoutAlgorithm<Node, BoxFragmentBuilder, BlockBreakToken>,
    ) -> &'a Self;
    fn setup_relayout_data(&mut self, previous: &Self, relayout_type: RelayoutType) {
        self.base_mut()
            .SetupRelayoutData(previous.base(), relayout_type);
    }
}
