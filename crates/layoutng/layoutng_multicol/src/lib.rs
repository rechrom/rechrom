#![allow(non_snake_case)]
pub mod column_layout_algorithm;
mod column_spanner_invalidation;
pub mod assembly {
    use crate::column_layout_algorithm::ColumnLayoutAlgorithm;
    use layoutng_assembly::{
        internal::algorithm_entry::NativeAlgorithmEntry, layout_assembly::LayoutAssembly,
    };
    pub fn InstallMulticolAlgorithm(assembly: &mut LayoutAssembly) {
        assembly.algorithms.multicol = NativeAlgorithmEntry::<ColumnLayoutAlgorithm>();
        assembly.objects.invalidate_column_spanners =
            Some(crate::column_spanner_invalidation::MarkColumnSpannerCandidatesForLayoutIfNeeded);
    }
}
