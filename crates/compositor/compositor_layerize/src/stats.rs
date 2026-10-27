#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayerizeStats {
    pub input_chunks: usize,
    pub output_layers: usize,
    pub own_layer_chunks: usize,
    pub decomposited_effects: usize,
    pub effect_decomposition_rejections: usize,
    pub scroll_range_dependent_upcasts: usize,
    pub merge_attempts: usize,
    pub merged_layers: usize,
    pub overlap_barriers: usize,
    pub max_merge_search_depth: usize,
}
