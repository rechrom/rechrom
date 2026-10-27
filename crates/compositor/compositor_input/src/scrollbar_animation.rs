//! Live scrollbar layer and weak, cancellable task boundaries. No layer state
//! is copied here, and task delivery never owns the animation controller.
use compositor_tree::geometry::{self, PointF};
use std::sync::{Arc, Weak};

/// Host queues this weak task, then routes it to RunDelayedAnimationTask only
/// while IsAlive() is true. Generation and liveness identity reject stale tasks.
#[derive(Clone, Debug)]
pub struct ScrollbarAnimationTask {
    pub owner_id: u64,
    pub generation: u64,
    alive: Weak<()>,
}
impl ScrollbarAnimationTask {
    pub(crate) fn new(owner_id: u64, generation: u64, alive: &Arc<()>) -> Self {
        Self {
            owner_id,
            generation,
            alive: Arc::downgrade(alive),
        }
    }
    pub fn IsAlive(&self) -> bool {
        self.alive.upgrade().is_some()
    }
    pub(crate) fn matches(&self, other: &Self) -> bool {
        self.owner_id == other.owner_id
            && self.generation == other.generation
            && self.alive.ptr_eq(&other.alive)
    }
}

// Source dependency: ui/gfx/geometry/rect_f.cc:243-250.
pub fn ManhattanDistanceToPoint(rect: geometry::RectF, point: PointF) -> f32 {
    let x_distance = geometry::cpp_max(
        0.,
        geometry::cpp_max(rect.x() - point.x(), point.x() - rect.right()),
    );
    let y_distance = geometry::cpp_max(
        0.,
        geometry::cpp_max(rect.y() - point.y(), point.y() - rect.bottom()),
    );
    x_distance + y_distance
}
