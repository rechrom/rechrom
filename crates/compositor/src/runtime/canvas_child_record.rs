//! Immutable Canvas raster input; the finalized source list keeps its original
//! allocation. The prefix offset is calculated by ContentLayerClientImpl.
//! Nested CustomData must be substituted from actual canvas snapshots when a
//! Canvas2D receiver consumes this input; it is not a pre-flattened record.
use super::{paint::ContentDisplayItemList, skia::SkRegion};
use crate::adapter::geometry::Vector2dF;
use std::sync::Arc;
pub struct CanvasChildRecordReplay {
    pub list: Arc<ContentDisplayItemList<SkRegion>>,
    pub offset: Vector2dF,
}
impl canvas::CanvasRecordReadLease for CanvasChildRecordReplay {
    fn VisitCustomData(&self, visitor: &mut dyn FnMut(u32)) {
        self.list.paint_op_buffer().VisitCustomData(visitor);
    }
    fn BackendRecord(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }
}
