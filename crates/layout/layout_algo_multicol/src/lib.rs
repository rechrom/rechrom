#![allow(non_snake_case)]
pub mod column_layout_algorithm;
mod column_spanner_invalidation;
pub mod registration {
    use crate::column_layout_algorithm::ColumnLayoutAlgorithm;
    use layout_base::{
        internal::algorithm_entry::LayoutAlgorithmEntryFor, layout_registry::LayoutRegistry,
    };
    pub fn Register(registry: &mut LayoutRegistry) {
        registry.algorithms.multicol =
            LayoutAlgorithmEntryFor::<ColumnLayoutAlgorithm>("layout_algo_multicol");
        registry.objects.invalidate_column_spanners =
            Some(crate::column_spanner_invalidation::MarkColumnSpannerCandidatesForLayoutIfNeeded);
    }
}
