#![allow(non_snake_case)]

use layoutng_fragment_tree::fragment_tree::FragmentNode;
use layoutng_fragment_tree::layout_result::LayoutResult;

use crate::internal::constraint_space::ConstraintSpace as NativeConstraintSpace;
use crate::internal::layout_input::{ComputedStyle, ConstraintSpace};
use crate::internal::layout_object::LayoutObject;

// cpp: layoutng/layout_boundary_support.h:15-23
pub trait LayoutBoundaryEnvironment {
    fn ReusesPreparedFonts(&self) -> bool;
    fn Commit(&mut self, root: &mut LayoutObject);
}

// cpp: layoutng/layout_boundary_support.h:25-35
#[derive(Clone, Copy, Default)]
pub struct LayoutBoundarySupport {
    pub create_environment:
        Option<fn(&mut LayoutObject, &ConstraintSpace) -> Box<dyn LayoutBoundaryEnvironment>>,
    pub prepare_constraints:
        Option<fn(&ComputedStyle, &ConstraintSpace, bool) -> NativeConstraintSpace>,
    pub prepare_tree: Option<fn(&mut LayoutObject, &ConstraintSpace, &NativeConstraintSpace, bool)>,
    pub export_tree: Option<fn(&mut LayoutObject, &LayoutResult, &ConstraintSpace) -> FragmentNode>,
}
