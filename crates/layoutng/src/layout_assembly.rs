#![allow(non_snake_case)]

use crate::internal::layout_algorithm_set::LayoutAlgorithmSet;
use crate::internal::layout_object_factory_set::LayoutObjectFactorySet;
use crate::layout_boundary_support::LayoutBoundarySupport;

// cpp: layoutng/layout_assembly.h:9-15
#[derive(Clone, Copy, Default)]
pub struct LayoutAssembly {
    pub algorithms: LayoutAlgorithmSet,
    pub objects: LayoutObjectFactorySet,
    pub boundary: LayoutBoundarySupport,
}

// cpp: layoutng/layout_assembly.h:17-17
// Implemented by //src/main/full_layout_algorithms.cc, outside the selected
// package set. The external symbol remains a pending assembly dependency.
unsafe extern "Rust" {
    pub fn FullLayoutAssembly() -> &'static LayoutAssembly;
}

impl LayoutAssembly {
    /// Refresh scroll-dependent paint properties after Layout retained its result.
    /// The caller must fall back to ExportFragments if geometry or styles changed.
    /// Reusing sticky constraints additionally requires the offset-only Page proof.
    pub fn RefreshScrollPaintProperties(
        &self,
        root: &mut crate::internal::layout_object::LayoutObject,
        fragments: &mut layoutng_fragment_tree::fragment_tree::FragmentNode,
        reuse_sticky_constraints: bool,
    ) {
        let mut heap = foundation::LayoutHeapScope::new();
        let _pass = crate::internal::layout_pass_scope::LayoutPassScope::new(
            &self.algorithms,
            &self.objects,
        );
        crate::internal::boundary::layout_boundary::RefreshScrollPaintProperties(
            root,
            fragments,
            reuse_sticky_constraints,
        );
        // Ordinary scroll updates allocate no managed objects. Sticky constraint
        // allocation still forces collection through the heap activity check.
        heap.AllowUnchangedReuse();
    }

    /// Derive an owned snapshot for paint/diagnostics from the result already
    /// stored on the persistent tree. This does not perform layout.
    pub fn ExportFragments(
        &self,
        root: &mut crate::internal::layout_object::LayoutObject,
        space: &crate::internal::layout_input::ConstraintSpace,
    ) -> layoutng_fragment_tree::fragment_tree::FragmentNode {
        use crate::internal::{layout_box::LayoutBox, layout_pass_scope::LayoutPassScope};
        let _heap = foundation::LayoutHeapScope::new();
        let _pass = LayoutPassScope::new(&self.algorithms, &self.objects);
        let root_box = foundation::DynamicTo::<LayoutBox>(root as *mut _);
        assert!(!root_box.is_null(), "snapshot root must be a LayoutBox");
        let result = unsafe { &*root_box }.GetLayoutResult(0);
        assert!(
            !result.is_null(),
            "layout must finish before exporting fragments"
        );
        let mut environment = self
            .boundary
            .create_environment
            .expect("layout environment")(root, space);
        let snapshot =
            self.boundary
                .export_tree
                .expect("fragment snapshot exporter")(root, unsafe { &*result }, space);
        environment.Commit(root);
        snapshot
    }
}
