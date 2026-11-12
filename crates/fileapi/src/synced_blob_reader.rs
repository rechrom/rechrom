//! SyncReadBlobDataHandle, data_object.cc:452-483. This File API contract has
//! already admitted complete immutable Blob bytes; native/filesystem files
//! follow their separate source transport branches and never enter this read.
use std::sync::Arc;
pub const MAX_SYNC_DRAG_BLOB_READ: usize = 256 * 1024 * 1024;
pub fn SyncReadBlobDataHandle(handle: Arc<[u8]>) -> Option<Arc<[u8]>> {
    (!handle.is_empty() && handle.len() <= MAX_SYNC_DRAG_BLOB_READ).then_some(handle)
}
