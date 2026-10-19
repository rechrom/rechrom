//! Native immutable Paint worklet input/value ownership. The external input
//! supplies actual property keys and repaint semantics; cc owns the record map.
use crate::adapter::{input_types::ElementId, layer_tree_impl::Color4f};
use std::sync::Arc;
// Source: cc/paint/paint_worklet_input.h:24-28,43-63;
// cc/paint/paint_worklet_input.cc:40-61. Rust's sum type enforces exactly one
// native/custom discriminator; Custom sorts before Native, then by name/id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NativePropertyType {
    kBackgroundColor,
    kClipPath,
    kInvalid,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkletPropertyKey {
    Custom(String, ElementId),
    Native(NativePropertyType, ElementId),
}
// Source: cc/paint/paint_worklet_input.h:65-80; .cc:64-85.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorkletPropertyValue {
    pub float_value: Option<f32>,
    pub color_value: Option<Color4f>,
}
impl crate::adapter::paint_worklet::PropertyValue for WorkletPropertyValue {
    fn has_value(&self) -> bool {
        self.float_value.is_some() || self.color_value.is_some()
    }
    fn reset(&mut self) {
        self.float_value = None;
        self.color_value = None;
    }
}
/// The actual external PaintWorkletInput allocation. No Page, tree or callback
/// to a compositor owner can be retained by this immutable value capability.
pub trait CpuPaintWorkletInput: Send + Sync {
    fn WorkletId(&self) -> u64;
    fn GetPropertyKeys(&self) -> &[WorkletPropertyKey];
    fn ValueChangeShouldCauseRepaint(
        &self,
        previous: &WorkletPropertyValue,
        next: &WorkletPropertyValue,
    ) -> bool;
}
#[derive(Clone)]
pub struct CpuWorkletInput(pub Arc<dyn CpuPaintWorkletInput>);
impl crate::adapter::paint_worklet::PaintWorkletInput<WorkletPropertyKey> for CpuWorkletInput {
    fn WorkletId(&self) -> u64 {
        self.0.WorkletId()
    }
    fn GetPropertyKeys(&self) -> &[WorkletPropertyKey] {
        self.0.GetPropertyKeys()
    }
}
impl
    crate::adapter::picture_layer_impl::PicturePaintWorkletInput<
        WorkletPropertyKey,
        WorkletPropertyValue,
    > for CpuWorkletInput
{
    fn ValueChangeShouldCauseRepaint(
        &self,
        previous: &WorkletPropertyValue,
        next: &WorkletPropertyValue,
    ) -> bool {
        self.0.ValueChangeShouldCauseRepaint(previous, next)
    }
}
pub type CpuPaintWorkletInputs = Vec<(
    Arc<CpuWorkletInput>,
    crate::adapter::image_animation::PaintImageId,
)>;
/// Source trace/object lifetime values are actual externally created RAII
/// allocations. Their destruction is the native event endpoint, not a stored
/// callback into a compositor owner.
pub trait CpuPaintObjectTracker: Send + Sync {}
pub trait CpuPaintTraceScope {}
pub type ObjectTracker = Box<dyn CpuPaintObjectTracker>;
pub type PaintTraceScope = Box<dyn CpuPaintTraceScope>;
