#![allow(non_snake_case)]

use layoutng_fragment_tree::layout_result::LayoutResult;

use super::layout_algorithm::LayoutAlgorithmParams;
use super::layout_algorithm_set::LayoutAlgorithmEntry;
use super::layout_input_node::MinMaxSizesFloatInput;
use super::min_max_sizes::MinMaxSizesResult;

// C++ templates construct a fresh algorithm for each entry call. Implementing
// this trait supplies the three operations that the templates require.
// cpp: layoutng/internal/algorithm_entry.h:5-7
pub trait NativeAlgorithm: Sized {
    fn new(params: &LayoutAlgorithmParams) -> Self;
    fn layout(&mut self) -> *const LayoutResult;
    fn compute_min_max_sizes(&mut self, input: &MinMaxSizesFloatInput) -> MinMaxSizesResult;
}

// cpp: layoutng/internal/algorithm_entry.h:8-20
#[inline(never)]
pub fn RunNativeAlgorithm<A: NativeAlgorithm>(
    params: &LayoutAlgorithmParams,
) -> *const LayoutResult {
    A::new(params).layout()
}

#[inline(never)]
pub fn MeasureNativeAlgorithm<A: NativeAlgorithm>(
    params: &LayoutAlgorithmParams,
    input: &MinMaxSizesFloatInput,
) -> MinMaxSizesResult {
    A::new(params).compute_min_max_sizes(input)
}

// cpp: layoutng/internal/algorithm_entry.h:22-27
pub fn NativeAlgorithmEntry<A: NativeAlgorithm>() -> LayoutAlgorithmEntry {
    LayoutAlgorithmEntry {
        layout: Some(RunNativeAlgorithm::<A>),
        measure: Some(MeasureNativeAlgorithm::<A>),
    }
}
