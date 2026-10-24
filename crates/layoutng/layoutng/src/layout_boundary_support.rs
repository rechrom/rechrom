#![allow(non_snake_case)]

use layoutng_fragment_tree::fragment_tree::FragmentNode;
use layoutng_fragment_tree::layout_result::LayoutResult;
use std::collections::HashMap;
use std::sync::Arc;

use crate::internal::constraint_space::ConstraintSpace as NativeConstraintSpace;
use crate::internal::layout_input::{ComputedStyle, ConstraintSpace, PaintStyle};
use crate::internal::layout_object::LayoutObject;

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
#[derive(Default)]
pub struct LayoutBoundaryExportCache {
    pub(crate) styles: HashMap<(usize, usize), PaintStyle>,
    pub(crate) caret_positions: HashMap<(usize, u32, u32, u32, u32, u64), Arc<Vec<(u32, f64)>>>,
}

impl LayoutBoundaryExportCache {
    pub fn Clear(&mut self) {
        self.styles.clear();
        self.caret_positions.clear();
    }
}

// cpp: layoutng/layout_boundary_support.h:25-35
#[derive(Clone, Copy, Default)]
pub struct LayoutBoundarySupport {
    pub create_environment:
        Option<fn(&mut LayoutObject, &ConstraintSpace) -> Box<dyn LayoutBoundaryEnvironment>>,
    pub prepare_constraints:
        Option<fn(&ComputedStyle, &ConstraintSpace, bool) -> NativeConstraintSpace>,
    pub prepare_tree: Option<fn(&mut LayoutObject, &ConstraintSpace, &NativeConstraintSpace, bool)>,
    pub export_tree: Option<
        fn(
            &mut LayoutObject,
            &LayoutResult,
            &ConstraintSpace,
            &mut LayoutBoundaryExportCache,
        ) -> FragmentNode,
    >,
}
