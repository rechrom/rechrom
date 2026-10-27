#![allow(non_snake_case)]

use layout_fragment::layout_result::LayoutResult;

use super::layout_algorithm::LayoutAlgorithmParams;
use super::layout_algorithm_set::LayoutAlgorithmEntry;
use super::layout_input_node::MinMaxSizesFloatInput;
use super::min_max_sizes::MinMaxSizesResult;

// C++ templates construct a fresh algorithm for each entry call. Implementing
// this trait supplies the three operations that the templates require.
// cpp: layoutng/internal/algorithm_entry.h:5-7
pub trait NativeAlgorithm: Sized {
    /// Stable package-qualified trace names keep profiles attributable after
    /// monomorphization and make layout and intrinsic measurement separable.
    const LAYOUT_TRACE_NAME: &'static str;
    const MEASURE_TRACE_NAME: &'static str;
    const ROOT_TRACE_NAME: Option<&'static str> = None;

    fn new(params: &LayoutAlgorithmParams) -> Self;
    fn layout(&mut self) -> *const LayoutResult;
    fn compute_min_max_sizes(&mut self, input: &MinMaxSizesFloatInput) -> MinMaxSizesResult;
}

// cpp: layoutng/internal/algorithm_entry.h:8-20
#[inline(never)]
pub fn RunNativeAlgorithm<A: NativeAlgorithm>(
    params: &LayoutAlgorithmParams,
) -> *const LayoutResult {
    let _trace = browser_tracing::span("layout", A::LAYOUT_TRACE_NAME);
    A::new(params).layout()
}

#[inline(never)]
pub fn MeasureNativeAlgorithm<A: NativeAlgorithm>(
    params: &LayoutAlgorithmParams,
    input: &MinMaxSizesFloatInput,
) -> MinMaxSizesResult {
    let _trace = browser_tracing::span("layout", A::MEASURE_TRACE_NAME);
    A::new(params).compute_min_max_sizes(input)
}

#[inline(never)]
pub fn TraceNativeAlgorithmRoot<A: NativeAlgorithm>() -> browser_tracing::Span {
    browser_tracing::span(
        "layout",
        A::ROOT_TRACE_NAME.expect("native root trace was not registered"),
    )
}

// cpp: layoutng/internal/algorithm_entry.h:22-27
fn AssertPackageTraceName(package: &str, name: &str) {
    assert!(
        package.starts_with("layout_algo_")
            && name
                .strip_prefix(package)
                .is_some_and(|suffix| suffix.starts_with('.')),
        "native layout trace name {name:?} must be owned by package {package:?}"
    );
}

pub fn NativeAlgorithmEntry<A: NativeAlgorithm>(package: &'static str) -> LayoutAlgorithmEntry {
    AssertPackageTraceName(package, A::LAYOUT_TRACE_NAME);
    AssertPackageTraceName(package, A::MEASURE_TRACE_NAME);
    let root_trace = if let Some(name) = A::ROOT_TRACE_NAME {
        AssertPackageTraceName(package, name);
        Some(TraceNativeAlgorithmRoot::<A> as fn() -> browser_tracing::Span)
    } else {
        None
    };
    LayoutAlgorithmEntry {
        performance_owner: Some(package),
        layout: Some(RunNativeAlgorithm::<A>),
        measure: Some(MeasureNativeAlgorithm::<A>),
        root_trace,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RootedAlgorithm;

    impl NativeAlgorithm for RootedAlgorithm {
        const LAYOUT_TRACE_NAME: &'static str = "layout_algo_test.Layout";
        const MEASURE_TRACE_NAME: &'static str = "layout_algo_test.Measure";
        const ROOT_TRACE_NAME: Option<&'static str> = Some("layout_algo_test.RootLayout");

        fn new(_: &LayoutAlgorithmParams) -> Self {
            Self
        }

        fn layout(&mut self) -> *const LayoutResult {
            std::ptr::null()
        }

        fn compute_min_max_sizes(&mut self, _: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
            unreachable!()
        }
    }

    #[test]
    fn package_qualified_trace_name_is_accepted() {
        AssertPackageTraceName("layout_algo_test", "layout_algo_test.Layout");
        assert_eq!(
            NativeAlgorithmEntry::<RootedAlgorithm>("layout_algo_test").performance_owner,
            Some("layout_algo_test")
        );
    }

    #[test]
    fn optional_root_trace_is_registered_by_the_algorithm_package() {
        let entry = NativeAlgorithmEntry::<RootedAlgorithm>("layout_algo_test");
        let recording = browser_tracing::start(Default::default()).unwrap();
        drop(entry.root_trace.expect("root trace")());
        browser_tracing::stop().unwrap();
        let snapshot = browser_tracing::snapshot(Some(recording)).unwrap();
        assert!(snapshot
            .spans
            .iter()
            .any(|trace| trace.name == "layout_algo_test.RootLayout"));
    }

    #[test]
    #[should_panic(expected = "must be owned by package")]
    fn generic_trace_name_is_rejected() {
        AssertPackageTraceName("layout_algo_grid", "LayoutEngine.Layout");
    }

    #[test]
    #[should_panic(expected = "must be owned by package")]
    fn wrong_algorithm_package_is_rejected() {
        AssertPackageTraceName("layout_algo_test", "layout_algo_other_test.Layout");
    }
}
