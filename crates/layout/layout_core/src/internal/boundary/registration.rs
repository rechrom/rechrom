#![allow(non_snake_case)]

use crate::internal::layout_object::LayoutObject;
use crate::layout_registry::LayoutRegistry;
use crate::layout_boundary_support::LayoutBoundaryEnvironment;

use super::input::PrepareConstraints;
use super::layout_boundary::{LayoutEnvironment, PrepareLayoutObjectTree, PublishLayoutResult};

// cpp: layoutng/internal/boundary/assembly.h:3-6
// cpp: layoutng/internal/boundary/assembly.cc:11-19
pub fn Register(registry: &mut LayoutRegistry) {
    registry.boundary.create_environment = Some(
        |root: &mut LayoutObject, space| -> Box<dyn LayoutBoundaryEnvironment> {
            Box::new(LayoutEnvironment::new(root, space))
        },
    );
    registry.boundary.prepare_constraints = Some(PrepareConstraints);
    registry.boundary.prepare_tree = Some(PrepareLayoutObjectTree);
    registry.boundary.publish_layout_result = Some(PublishLayoutResult);
}
