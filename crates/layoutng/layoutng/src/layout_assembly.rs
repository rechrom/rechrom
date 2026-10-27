#![allow(non_snake_case)]

use crate::internal::layout_algorithm_set::LayoutAlgorithmSet;
use crate::internal::layout_object_factory_set::LayoutObjectFactorySet;
use crate::layout_boundary_support::LayoutBoundarySupport;

struct NativeLayoutPublication<'a> {
    boundary: LayoutBoundarySupport,
    root: &'a mut crate::internal::layout_object::LayoutObject,
    result: &'a layout_fragment::layout_result::LayoutResult,
    space: &'a crate::internal::layout_input::ConstraintSpace,
    cache: &'a mut crate::layout_boundary_support::LayoutBoundaryCache,
}

impl layout_commit::LayoutPublicationBackend for NativeLayoutPublication<'_> {
    type Environment = Box<dyn crate::layout_boundary_support::LayoutBoundaryEnvironment>;
    type Output = layout_fragment::fragment_tree::LayoutResultHandle;

    fn CachedObjects(&self) -> usize {
        self.cache.objects.len()
    }

    fn CachedStyles(&self) -> usize {
        self.cache.styles.len()
    }

    fn CreateEnvironment(&mut self) -> Self::Environment {
        self.boundary
            .create_environment
            .expect("layout environment")(self.root, self.space)
    }

    fn ExportFragments(&mut self) -> Self::Output {
        self.boundary
            .publish_layout_result
            .expect("layout-result publisher")(
            self.root, self.result, self.space, self.cache
        )
    }

    fn CommitEnvironment(&mut self, mut environment: Self::Environment) {
        environment.Commit(self.root);
    }
}

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
        cache: &mut crate::layout_boundary_support::LayoutBoundaryCache,
    ) {
        let mut trace = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::RebuildBoundaryObjectIndex,
        );
        let _heap = foundation::LayoutHeapScope::new();
        let _pass = crate::internal::layout_pass_scope::LayoutPassScope::new(
            &self.algorithms,
            &self.objects,
        );
        crate::internal::boundary::layout_boundary::EnsureLayoutBoundaryObjectIndex(root, cache);
        trace.set("cached_objects", cache.objects.len() as f64);
    }

    /// Refresh scroll-dependent paint properties after Layout retained its result.
    /// The caller must fall back to PublishLayoutResult if geometry or styles changed.
    /// Reusing sticky constraints additionally requires the offset-only Page proof.
    pub fn RefreshScrollPaintProperties(
        &self,
        root: &mut crate::internal::layout_object::LayoutObject,
        fragments: &layout_fragment::fragment_tree::LayoutResultHandle,
        reuse_sticky_constraints: bool,
    ) {
        let mut heap = foundation::LayoutHeapScope::new();
        let _pass = crate::internal::layout_pass_scope::LayoutPassScope::new(
            &self.algorithms,
            &self.objects,
        );
        crate::internal::boundary::layout_boundary::RefreshScrollPaintProperties(
            root,
            crate::layout_engine::NativePaintFragmentCursor::FromHandle(fragments),
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
    pub fn PublishLayoutResult(
        &self,
        root: &mut crate::internal::layout_object::LayoutObject,
        space: &crate::internal::layout_input::ConstraintSpace,
        cache: &mut crate::layout_boundary_support::LayoutBoundaryCache,
    ) -> layout_fragment::fragment_tree::LayoutResultHandle {
        use crate::internal::{layout_box::LayoutBox, layout_pass_scope::LayoutPassScope};
        let _heap = foundation::LayoutHeapScope::new();
        let _pass = LayoutPassScope::new(&self.algorithms, &self.objects);
        let root_box = foundation::DynamicTo::<LayoutBox>(root as *mut _);
        assert!(!root_box.is_null(), "snapshot root must be a LayoutBox");
        let result = unsafe { &*root_box }.GetLayoutResult(0);
        assert!(
            !result.is_null(),
            "layout must finish before publishing its native result"
        );
        layout_commit::PublishLayoutResult(&mut NativeLayoutPublication {
            boundary: self.boundary,
            root,
            result: unsafe { &*result },
            space,
            cache,
        })
    }
}
