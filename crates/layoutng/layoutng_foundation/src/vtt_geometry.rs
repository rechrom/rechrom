//! Small borrowed-layout query values consumed by WebVTT cue positioning.
use crate::{LayoutUnit, PhysicalRect, PhysicalSize};
/// Valid only for one caller-held resident LayoutOutput lease.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VTTLayoutObjectId(pub usize);
#[derive(Clone, Copy)]
pub struct VTTCueLayoutFacts {
    pub horizontal: bool,
    pub flipped_blocks: bool,
    pub stitched_width: LayoutUnit,
    pub logical_height: LayoutUnit,
    pub has_parent_and_grandparent: bool,
    pub container_size: PhysicalSize,
    pub container_border_box: PhysicalRect,
    pub border_box_in_containing_block: PhysicalRect,
    pub parent_node_id: Option<u64>,
}
