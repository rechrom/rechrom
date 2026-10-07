// These are declarations in the layoutng package, not implementations of the
// algorithm-owned classes. Uninhabited Rust types retain distinct pointer
// identities while preventing this crate from constructing external objects.

// cpp: layoutng/internal/layout_algorithm_set.h:26-31,45-46
pub enum BlockLayoutAlgorithm {}
pub enum PreviousInflowPosition {}
pub enum GridLanesSizingSubtreeRequest {}
pub enum GridLanesSubgriddedItemsRequest {}

// cpp: layoutng/internal/inline_node.h:32-37
pub enum InlineChildLayoutContext {}

// cpp: layoutng_inline/inline_child_layout_context.h:64-66
// The context is owned by the excluded inline extension. A non-null context
// requires that extension's implementation at link time.
impl InlineChildLayoutContext {
    pub fn ParallelFlowBreakTokens(
        &self,
    ) -> &foundation::HeapVector<foundation::Member<crate::break_token::BreakToken>> {
        unsafe { InlineChildLayoutContextParallelFlowBreakTokens(self) }
    }
}

unsafe extern "Rust" {
    fn InlineChildLayoutContextParallelFlowBreakTokens(
        context: &InlineChildLayoutContext,
    ) -> &foundation::HeapVector<foundation::Member<crate::break_token::BreakToken>>;
}
pub use layoutng_inline::offset_mapping::OffsetMapping;
pub enum TextDiffRange {}

// cpp: layoutng/internal/inline_item_result.h:31
#[cfg(not(feature = "translation_in_progress"))]
pub enum InlineItemResultRubyColumn {}
#[cfg(feature = "translation_in_progress")]
pub use crate::inline_item_result_ruby_column::InlineItemResultRubyColumn;

// cpp: layoutng/internal/layout_box.h:50
pub enum CustomLayoutChild {}

// cpp: layoutng/internal/block_node.h:23
pub enum CSSLayoutDefinition {}
