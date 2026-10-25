#![allow(non_snake_case)]

use layoutng_fragment_tree::fragment_tree::FragmentNode;
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_style::style::computed_style::ComputedStyle as NativeComputedStyle;
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::Arc;

use crate::internal::constraint_space::ConstraintSpace as NativeConstraintSpace;
use crate::internal::layout_input::{ComputedStyle, ConstraintSpace, PaintStyle};
use crate::internal::layout_node_metadata::Node;
use crate::internal::layout_object::LayoutObject;

/// These maps contain only trusted resident pointers and small numeric
/// identity tuples. A compact hasher avoids SipHash work in every exported
/// fragment while HashMap still performs the full key equality check.
#[derive(Default)]
pub(crate) struct LayoutBoundaryIdentityHasher(u64);

impl Hasher for LayoutBoundaryIdentityHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = (self.0.rotate_left(5) ^ u64::from(byte)).wrapping_mul(0x9e37_79b1_85eb_ca87);
        }
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = (self.0.rotate_left(17) ^ value).wrapping_mul(0x9e37_79b1_85eb_ca87);
    }

    fn write_usize(&mut self, value: usize) {
        self.write_u64(value as u64);
    }
}

pub(crate) type LayoutBoundaryHashMap<K, V> =
    HashMap<K, V, BuildHasherDefault<LayoutBoundaryIdentityHasher>>;
pub(crate) type LayoutBoundaryHashSet<K> =
    HashSet<K, BuildHasherDefault<LayoutBoundaryIdentityHasher>>;

// cpp: layoutng/layout_boundary_support.h:15-23
pub trait LayoutBoundaryEnvironment {
    fn ReusesPreparedFonts(&self) -> bool;
    fn Commit(&mut self, root: &mut LayoutObject);
}

/// Layout updates retain native fragments for borrowed queries. Paint also
/// materializes the current paint boundary output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutUpdateKind {
    Layout,
    Paint,
}

/// Immutable boundary projections which remain valid while LayoutObject style
/// ownership is unchanged. Physical fragments are commonly rebuilt for a new
/// viewport constraint, but their native/projected ComputedStyle pair is not.
#[derive(Clone, Copy)]
pub(crate) struct LayoutBoundarySourceEntry {
    pub(crate) node: *const Node,
    pub(crate) logical_tree_order: u64,
}

#[derive(Default)]
pub struct LayoutBoundaryExportCache {
    pub(crate) styles: LayoutBoundaryHashMap<(usize, usize), PaintStyle>,
    pub(crate) caret_positions:
        LayoutBoundaryHashMap<(usize, u32, u32, u32, u32, u64), Arc<Vec<(u32, f64)>>>,
    pub(crate) debug_names: LayoutBoundaryHashMap<*const LayoutObject, Arc<str>>,
    pub(crate) objects: Vec<*mut LayoutObject>,
    pub(crate) source_objects: Arc<Vec<*mut LayoutObject>>,
    pub(crate) sticky_objects: Arc<Vec<*mut LayoutObject>>,
    pub(crate) sources: LayoutBoundaryHashMap<*const LayoutObject, LayoutBoundarySourceEntry>,
    pub(crate) style_sources:
        LayoutBoundaryHashMap<*const NativeComputedStyle, *const ComputedStyle>,
    pub(crate) uncullable_objects: Arc<LayoutBoundaryHashSet<*const LayoutObject>>,
    // Native fragment/item identities remain stable while a LayoutResult is
    // retained. Chromium keeps the corresponding FragmentData identity across
    // cull-rect updates; preserve the same identity in the owned projection so
    // PrePaint does not mistake a shifted culled child index for a new object.
    pub(crate) fragment_identity_result: usize,
    pub(crate) fragment_instance_ids: LayoutBoundaryHashMap<(u8, usize), u64>,
    pub(crate) next_fragment_instance_id: u64,
}

impl LayoutBoundaryExportCache {
    pub fn Clear(&mut self) {
        self.styles.clear();
        self.caret_positions.clear();
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
            Option<&[*mut LayoutObject]>,
        ),
    >,
    pub export_tree: Option<
        fn(
            &mut LayoutObject,
            &LayoutResult,
            &ConstraintSpace,
            &mut LayoutBoundaryExportCache,
        ) -> FragmentNode,
    >,
    pub refresh_tree_projection: Option<
        fn(
            &mut LayoutObject,
            &LayoutResult,
            &ConstraintSpace,
            &mut LayoutBoundaryExportCache,
            FragmentNode,
        ) -> FragmentNode,
    >,
}
