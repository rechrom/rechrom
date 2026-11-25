//! The DisplayItemList value PictureLayer records and PictureDrawQuad
//! carries (cc/paint/display_item_list.h). The recording source, raster
//! source and picture recorder bindings stay in layerize
//! (commit/paint/picture.rs).
use super::{display_item_list::DisplayItemList, CpuPaintTypes};

pub type ContentDisplayItemList<R> = DisplayItemList<CpuPaintTypes<R>>;
pub use super::CpuDrawImage as DrawImage;
pub use super::CpuFilterQuality as FilterQuality;
pub use super::node_info::NodeInfo;
// Source: cc/paint/paint_flags.h:95-133. The native high limit is (0,0).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DynamicRangeLimit {
    pub standard_mix: f32,
    pub constrained_high_mix: f32,
}
