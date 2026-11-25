//! viz::CopyOutputRequest (components/viz/common/frame_sinks/
//! copy_output_request.h). Native request ownership and abort-on-drop
//! behavior are implemented by the external Viz request owner; this value
//! forwards only the request's native fields. The cc facets
//! (`PropertyTreeCopyOutputRequest`, `MainLayerCopyOutputRequest`) are
//! implemented by `layerize::commit`, which owns those traits.
use compositor_tree::geometry::{Rect, Vector2d};
use compositor_values::unguessable_token::UnguessableToken;
use std::sync::Arc;

/// Strong native sequenced-task-runner capability, supplied by the embedding.
/// No worker/thread/clock implementation is created by the compositor family.
// Source: components/viz/common/frame_sinks/copy_output_request.h
//         (result_task_runner_).
pub trait ResultTaskRunner: Send + Sync {
    fn PostTask(&self, task: Box<dyn FnOnce() + Send>);
}
pub type TaskRunner = Arc<dyn ResultTaskRunner>;

/// The request owner's native fields (copy_output_request.h accessors).
pub trait CopyOutputRequestBackend: Send + Sync {
    fn has_area(&self) -> bool;
    fn area(&self) -> &Rect;
    fn set_area(&mut self, area: Rect);
    fn is_scaled(&self) -> bool;
    fn has_result_selection(&self) -> bool;
    fn scale_from(&self) -> Vector2d;
    fn scale_to(&self) -> Vector2d;
    fn SetScaleRatio(&mut self, from: Vector2d, to: Vector2d);
    fn has_result_task_runner(&self) -> bool;
    fn set_result_task_runner(&mut self, runner: TaskRunner);
    fn has_source(&self) -> bool;
    fn source(&self) -> UnguessableToken;
}
pub struct CopyOutputRequest(pub Box<dyn CopyOutputRequestBackend>);
impl CopyOutputRequest {
    pub fn has_area(&self) -> bool {
        self.0.has_area()
    }
    pub fn area(&self) -> &Rect {
        self.0.area()
    }
    pub fn set_area(&mut self, area: Rect) {
        self.0.set_area(area);
    }
    pub fn is_scaled(&self) -> bool {
        self.0.is_scaled()
    }
    pub fn has_result_selection(&self) -> bool {
        self.0.has_result_selection()
    }
    pub fn scale_from(&self) -> Vector2d {
        self.0.scale_from()
    }
    pub fn scale_to(&self) -> Vector2d {
        self.0.scale_to()
    }
    pub fn SetScaleRatio(&mut self, from: Vector2d, to: Vector2d) {
        self.0.SetScaleRatio(from, to);
    }
    pub fn has_result_task_runner(&self) -> bool {
        self.0.has_result_task_runner()
    }
    pub fn set_result_task_runner(&mut self, runner: TaskRunner) {
        self.0.set_result_task_runner(runner);
    }
    pub fn has_source(&self) -> bool {
        self.0.has_source()
    }
    pub fn source(&self) -> UnguessableToken {
        self.0.source()
    }
}
