//! Reusable layout values and caches with no dependency on the LayoutObject
//! tree or a concrete layout algorithm.

pub mod block_formatting_context;
pub mod cache_decision;
pub mod layout_cache_status;
pub mod layout_tree_update_stats;
pub mod min_max_sizes;
pub mod min_max_sizes_cache;
pub mod recent_entries;
pub mod trace;

pub use block_formatting_context::{
    calculate_block_formatting_context_reuse, BlockFormattingContextReuseAdjustment,
    BlockFormattingContextReuseInput, BlockFormattingContextReuseReason,
};
pub use cache_decision::{
    calculate_size_based_layout_cache_status, LayoutCacheDecisionBackend, LayoutCacheDecisionReason,
};
pub use layout_cache_status::LayoutCacheStatus;
pub use layout_tree_update_stats::LayoutTreeUpdateStats;
pub use min_max_sizes::{MinMaxSizes, MinMaxSizesResult};
pub use min_max_sizes_cache::MinMaxSizesCache;
pub use recent_entries::RecentEntries;
pub use trace::{
    retain_tree_span, LayoutReuseLookupTrace, LayoutReuseTraceSession, LayoutReuseTraceStats,
};
