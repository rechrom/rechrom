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
    /// Restore the document-wide boundary indices after a targeted subtree
    /// replacement. Chromium keeps the same live LayoutObject graph here; the
    /// extracted immutable boundary mirrors that identity in this cache.
    pub fn RebuildBoundaryObjectIndex(
        &self,
        root: &mut crate::internal::layout_object::LayoutObject,
        cache: &mut crate::layout_boundary_support::LayoutBoundaryExportCache,
    ) {
        let mut trace =
            browser_tracing::span("layout", "LayoutAssembly.RebuildBoundaryObjectIndex");
        let _heap = foundation::LayoutHeapScope::new();
        let _pass = crate::internal::layout_pass_scope::LayoutPassScope::new(
            &self.algorithms,
            &self.objects,
        );
        crate::internal::boundary::layout_boundary::EnsureLayoutBoundaryObjectIndex(root, cache);
        trace.set("cached_objects", cache.objects.len() as f64);
    }

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
        // cppgc schedules collection from allocation pressure instead of
        // stopping a scroll update to collect at the Blink API boundary. The
        // document owner services the pending request in bounded foreground
        // collection tasks after latency-sensitive frame work.
        heap.DeferCollection();
    }

    /// Derive an owned snapshot for paint/diagnostics from the result already
    /// stored on the persistent tree. This does not perform layout.
    pub fn ExportFragments(
        &self,
        root: &mut crate::internal::layout_object::LayoutObject,
        space: &crate::internal::layout_input::ConstraintSpace,
        cache: &mut crate::layout_boundary_support::LayoutBoundaryExportCache,
    ) -> layoutng_fragment_tree::fragment_tree::FragmentNode {
        use crate::internal::{layout_box::LayoutBox, layout_pass_scope::LayoutPassScope};
        let mut trace = browser_tracing::span("layout", "LayoutAssembly.ExportFragments");
        trace.set("cached_objects", cache.objects.len() as f64);
        trace.set("cached_styles", cache.styles.len() as f64);
        let _heap = foundation::LayoutHeapScope::new();
        let _pass = LayoutPassScope::new(&self.algorithms, &self.objects);
        let root_box = foundation::DynamicTo::<LayoutBox>(root as *mut _);
        assert!(!root_box.is_null(), "snapshot root must be a LayoutBox");
        let result = unsafe { &*root_box }.GetLayoutResult(0);
        assert!(
            !result.is_null(),
            "layout must finish before exporting fragments"
        );
        let mut environment = {
            let _phase = browser_tracing::span("layout", "LayoutAssembly.CreateEnvironment");
            self.boundary
                .create_environment
                .expect("layout environment")(root, space)
        };
        let snapshot = {
            let _phase = browser_tracing::span("layout", "LayoutAssembly.ExportTree");
            self.boundary
                .export_tree
                .expect("fragment snapshot exporter")(
                root, unsafe { &*result }, space, cache
            )
        };
        {
            let _phase = browser_tracing::span("layout", "LayoutAssembly.CommitEnvironment");
            environment.Commit(root);
        }
        snapshot
    }

    /// Advance the expanded paint cull projection over an unchanged native
    /// LayoutResult. Chromium retains FragmentData and serializes no second
    /// tree here; this boundary moves overlapping owned nodes and exports only
    /// fragments which newly enter the cull window.
    pub fn RefreshFragmentProjection(
        &self,
        root: &mut crate::internal::layout_object::LayoutObject,
        space: &crate::internal::layout_input::ConstraintSpace,
        cache: &mut crate::layout_boundary_support::LayoutBoundaryExportCache,
        retained: layoutng_fragment_tree::fragment_tree::FragmentNode,
    ) -> layoutng_fragment_tree::fragment_tree::FragmentNode {
        use crate::internal::{layout_box::LayoutBox, layout_pass_scope::LayoutPassScope};
        let mut trace = browser_tracing::span("layout", "LayoutAssembly.RefreshFragmentProjection");
        trace.set("cached_objects", cache.objects.len() as f64);
        let mut heap = foundation::LayoutHeapScope::new();
        let _pass = LayoutPassScope::new(&self.algorithms, &self.objects);
        let root_box = foundation::DynamicTo::<LayoutBox>(root as *mut _);
        assert!(!root_box.is_null(), "snapshot root must be a LayoutBox");
        let result = unsafe { &*root_box }.GetLayoutResult(0);
        assert!(
            !result.is_null(),
            "layout must finish before refreshing fragment projection"
        );
        let mut environment = self
            .boundary
            .create_environment
            .expect("layout environment")(root, space);
        let snapshot = self
            .boundary
            .refresh_tree_projection
            .expect("fragment projection refresher")(
            root,
            unsafe { &*result },
            space,
            cache,
            retained,
        );
        environment.Commit(root);
        // Advancing the paint interest window is part of the active scroll
        // frame. Match cppgc's allocation-driven scheduling and leave any
        // requested collection to the document owner's bounded GC task.
        heap.DeferCollection();
        snapshot
    }

    /// Export one laid-out subtree while borrowing the document root's native
    /// layout environment. Chromium's font resolver and other per-frame
    /// services belong to LocalFrame/LayoutView, not to the dirty descendant
    /// chosen for fragment replacement.
    ///
    /// # Safety
    /// Both pointers must belong to the same live LayoutObject tree and the
    /// subtree must have a successful current LayoutResult. The caller holds
    /// exclusive access to LayoutEngine for the duration of this operation.
    pub unsafe fn ExportFragmentSubtree(
        &self,
        environment_root: *mut crate::internal::layout_object::LayoutObject,
        subtree_root: *mut crate::internal::layout_object::LayoutObject,
        space: &crate::internal::layout_input::ConstraintSpace,
    ) -> layoutng_fragment_tree::fragment_tree::FragmentNode {
        use crate::internal::{layout_box::LayoutBox, layout_pass_scope::LayoutPassScope};
        assert!(!environment_root.is_null() && !subtree_root.is_null());
        let _heap = foundation::LayoutHeapScope::new();
        let _pass = LayoutPassScope::new(&self.algorithms, &self.objects);
        let subtree_box = foundation::DynamicTo::<LayoutBox>(subtree_root);
        assert!(!subtree_box.is_null(), "snapshot root must be a LayoutBox");
        let result = unsafe { &*subtree_box }.GetLayoutResult(0);
        assert!(
            !result.is_null(),
            "layout must finish before exporting fragments"
        );
        let mut environment =
            self.boundary
                .create_environment
                .expect("layout environment")(unsafe { &mut *environment_root }, space);
        let mut cache = crate::layout_boundary_support::LayoutBoundaryExportCache::default();
        let snapshot = self
            .boundary
            .export_tree
            .expect("fragment snapshot exporter")(
            unsafe { &mut *subtree_root },
            unsafe { &*result },
            space,
            &mut cache,
        );
        environment.Commit(unsafe { &mut *environment_root });
        snapshot
    }
}
