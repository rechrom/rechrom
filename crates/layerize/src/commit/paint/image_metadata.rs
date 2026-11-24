//! cc/paint image identity and animation metadata values (PaintImage::Id,
//! FrameMetadata, AnimatedImageMetadata), shared by recording on the main
//! thread and image decode/animation on the impl thread.
use crate::commit::input_types::ElementId;
use scroll_animation::TimeDelta;
use std::collections::{BTreeMap, BTreeSet};
// Source dependency: cc/paint/paint_image.h:177-186; paint_image.cc:45-46.
pub use paint::paint_engine::PaintImageResourceId as PaintImageId;
pub const kInvalidId: PaintImageId = PaintImageId(-2);
pub const kDefaultFrameIndex: usize = 0;
// Source dependency: cc/paint/paint_image.h:218-219.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionState {
    kDone,
    kPartiallyDone,
}
// Source dependency: cc/paint/frame_metadata.h:14-27.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameMetadata {
    pub complete: bool,
    pub duration: TimeDelta,
}
impl Default for FrameMetadata {
    fn default() -> Self {
        Self {
            complete: true,
            duration: TimeDelta::default(),
        }
    }
}
// Source dependency: cc/paint/discardable_image_map.h:44-61.
#[derive(Clone, Debug)]
pub struct AnimatedImageMetadata {
    pub paint_image_id: PaintImageId,
    pub sync_animation_target_id: PaintImageId,
    pub sync_animation_sequence_id: u32,
    pub completion_state: CompletionState,
    pub frames: Vec<FrameMetadata>,
    pub repetition_count: i32,
    pub reset_animation_sequence_id: u32,
}
// Source dependency: cc/paint/image_id.h:20-28.
pub type PaintImageIdFlatSet = BTreeSet<PaintImageId>;
pub type AnimatedImageDriverState = (bool, Vec<ElementId>);
pub type AnimatedImageDriverMap = BTreeMap<PaintImageId, AnimatedImageDriverState>;
// Source dependency: cc/paint/paint_image.h:520-532 (scoped_refptr value owner).
pub type AnimatedImageFrameIndexMap = BTreeMap<PaintImageId, usize>;
// Source dependency: cc/paint/image_animation_count.h:28-31.
pub const kAnimationLoopOnce: i32 = 0;
pub const kAnimationLoopInfinite: i32 = -1;
pub const kAnimationNone: i32 = -2;
pub const kAnimationPaused: i32 = -3;
