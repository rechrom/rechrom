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
// The context is owned by `layout_algo_inline`, which installs the accessor
// into `InlineLayoutSupport::parallel_flow_break_tokens`. A non-null context
// exists only while an inline formatting context is laid out, so the slot is
// read from the active pass.
impl InlineChildLayoutContext {
    pub fn ParallelFlowBreakTokens(
        &self,
    ) -> &foundation::HeapVector<foundation::Member<crate::break_token::BreakToken>> {
        let algorithms = super::layout_pass_scope::LayoutPassScope::Algorithms();
        let callback = if algorithms.is_null() {
            None
        } else {
            unsafe { &*algorithms }.inline_support.parallel_flow_break_tokens
        };
        let callback = callback.unwrap_or_else(|| {
            std::panic::panic_any(foundation::UnsupportedLayout::new(
                "inline layout module is not installed",
            ))
        });
        callback(self)
    }
}

pub use crate::offset_mapping::OffsetMapping;
pub enum TextDiffRange {}

// cpp: layoutng/internal/inline_item_result.h:31
// Forward declaration of the inline-owned column (`layout_algo_inline`'s
// `inline_item_result_ruby_column`). Inline casts `InlineItemResult::ruby_column`
// to its concrete type; base only tests the member for null.
pub enum InlineItemResultRubyColumn {}

// cpp: layoutng/internal/layout_box.h:50
pub enum CustomLayoutChild {}

// cpp: layoutng/internal/block_node.h:23
pub enum CSSLayoutDefinition {}
