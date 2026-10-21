#![allow(non_snake_case)]

use crate::internal::layout_object::LayoutObject;
use crate::layout_assembly::LayoutAssembly;
use crate::layout_boundary_support::LayoutBoundaryEnvironment;

use super::layout_boundary::{
    ExportLayoutObjectTree, NativeLayoutEnvironment, PrepareLayoutObjectTree,
};
use super::native_input::PrepareNativeConstraints;

// cpp: layoutng/internal/boundary/assembly.h:3-6
// cpp: layoutng/internal/boundary/assembly.cc:11-19
pub fn InstallLayoutBoundary(assembly: &mut LayoutAssembly) {
    assembly.boundary.create_environment = Some(
        |root: &mut LayoutObject, space| -> Box<dyn LayoutBoundaryEnvironment> {
            Box::new(NativeLayoutEnvironment::new(root, space))
        },
    );
    assembly.boundary.prepare_constraints = Some(PrepareNativeConstraints);
    assembly.boundary.prepare_tree = Some(PrepareLayoutObjectTree);
    assembly.boundary.export_tree = Some(ExportLayoutObjectTree);
}
