#![allow(non_snake_case)]

use layout_fragment::fragment_tree::LayoutResultHandle;
use layout_fragment::layout_result::LayoutResult;
use layoutng_style::style::computed_style::ComputedStyle as NativeComputedStyle;
use std::sync::Arc;

use foundation::{TrustedIdentityHashMap, TrustedIdentityHashSet};
pub use layout_commit::LayoutUpdateKind;

use crate::internal::constraint_space::ConstraintSpace as NativeConstraintSpace;
use crate::internal::layout_input::{ComputedStyle, ConstraintSpace, PaintStyle};
use crate::internal::layout_node_metadata::Node;
use crate::internal::layout_object::LayoutObject;

pub(crate) type LayoutBoundaryHashMap<K, V> = TrustedIdentityHashMap<K, V>;
pub(crate) type LayoutBoundaryHashSet<K> = TrustedIdentityHashSet<K>;

// cpp: layoutng/layout_boundary_support.h:15-23
pub trait LayoutBoundaryEnvironment {
    fn ReusesPreparedFonts(&self) -> bool;
    fn Commit(&mut self, root: &mut LayoutObject);
}

/// Immutable boundary inputs which remain valid while LayoutObject style
/// ownership is unchanged. Physical fragments are commonly rebuilt for a new
/// viewport constraint, but their native/source ComputedStyle pair is not.
#[derive(Clone, Copy)]
pub(crate) struct LayoutBoundarySourceEntry {
    pub(crate) node: *const Node,
    pub(crate) logical_tree_order: u64,
}

#[derive(Default)]
pub struct LayoutBoundaryCache {
    pub(crate) styles: LayoutBoundaryHashMap<(usize, usize), PaintStyle>,
    pub(crate) debug_names: LayoutBoundaryHashMap<*const LayoutObject, Arc<str>>,
    pub(crate) objects: Vec<*mut LayoutObject>,
    pub(crate) source_objects: Arc<Vec<*mut LayoutObject>>,
    pub(crate) sticky_objects: Arc<Vec<*mut LayoutObject>>,
    pub(crate) sources: LayoutBoundaryHashMap<*const LayoutObject, LayoutBoundarySourceEntry>,
    pub(crate) style_sources:
        LayoutBoundaryHashMap<*const NativeComputedStyle, *const ComputedStyle>,
    /// Layout objects whose paint can escape an ancestor line's bounds. These
    /// keep the corresponding inline subtree in Chromium's CullRect projection.
    pub(crate) uncullable_objects: Arc<foundation::TrustedIdentityHashSet<*const LayoutObject>>,
    // Native fragment/item identities remain stable while a LayoutResult is
    // retained. Chromium keeps the corresponding FragmentData identity across
    // cull-rect updates; preserve the same identity in fragment-local Paint
    // data so PrePaint does not mistake a retained native owner for a new one.
    pub(crate) fragment_identity_result: usize,
    pub(crate) fragment_instance_ids: LayoutBoundaryHashMap<(u8, usize), u64>,
    pub(crate) next_fragment_instance_id: u64,
}

impl LayoutBoundaryCache {
    pub fn Clear(&mut self) {
        self.styles.clear();
        self.debug_names.clear();
        self.objects.clear();
        self.source_objects = Arc::default();
        self.sticky_objects = Arc::default();
        self.sources.clear();
        self.style_sources.clear();
        self.uncullable_objects = Arc::default();
        self.fragment_identity_result = 0;
        self.fragment_instance_ids.clear();
        self.next_fragment_instance_id = 0;
    }
}

// cpp: layoutng/layout_boundary_support.h:25-35
#[derive(Clone, Copy, Default)]
pub struct LayoutBoundarySupport {
    pub create_environment:
        Option<fn(&mut LayoutObject, &ConstraintSpace) -> Box<dyn LayoutBoundaryEnvironment>>,
    pub prepare_constraints:
        Option<fn(&ComputedStyle, &ConstraintSpace, bool) -> NativeConstraintSpace>,
    pub prepare_tree: Option<
        fn(
            &mut LayoutObject,
            &ConstraintSpace,
            &NativeConstraintSpace,
            bool,
            bool,
            bool,
            bool,
            Option<&[*mut LayoutObject]>,
        ),
    >,
    pub publish_layout_result: Option<
        fn(
            &mut LayoutObject,
            &LayoutResult,
            &ConstraintSpace,
            &mut LayoutBoundaryCache,
        ) -> LayoutResultHandle,
    >,
}
