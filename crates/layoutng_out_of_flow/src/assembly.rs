#![allow(non_snake_case)]

use layoutng_assembly::box_fragment_builder::BoxFragmentBuilder;
use layoutng_assembly::layout_assembly::LayoutAssembly;

use crate::out_of_flow_layout_part::OutOfFlowLayoutPart;

// cpp: layoutng_out_of_flow/assembly.cc:9-11
fn RunOutOfFlowLayout(builder: &mut BoxFragmentBuilder) {
    OutOfFlowLayoutPart::new(builder).Run();
}

// cpp: layoutng_out_of_flow/assembly.h:3-5
// cpp: layoutng_out_of_flow/assembly.cc:15-17
pub fn InstallOutOfFlowAlgorithm(assembly: &mut LayoutAssembly) {
    assembly.algorithms.out_of_flow_support.run = Some(RunOutOfFlowLayout);
}
