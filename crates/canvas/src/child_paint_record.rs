//! Immutable child-record publication. The lease owns the original cc record;
//! consumers receive no compositor, layer, tile or property-tree mutation API.
use crate::{draw_element_image::CanvasChildPaintRecord, CanvasElementIdentity};
use std::{any::Any, collections::HashMap, fmt, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CanvasDocumentIdentity(pub u64);
impl CanvasDocumentIdentity {
    pub fn Element(self, node: u64) -> CanvasElementIdentity {
        CanvasElementIdentity((u128::from(self.0) << 64) | u128::from(node))
    }
}
pub trait CanvasRecordReadLease: Send + Sync {
    fn BackendRecord(&self) -> &(dyn Any + Send + Sync);
    /// Source FindCustomDataPlaceholders visits nested DrawRecord operations.
    fn VisitCustomData(&self, visitor: &mut dyn FnMut(u32));
}
pub type CanvasRecord = Arc<dyn CanvasRecordReadLease>;
pub type PublishedCanvasChildRecord = CanvasChildPaintRecord<CanvasRecord>;
/// One actual main-frame PAC update, including a genuinely empty map when that
/// update removed every canvas child. No missing child becomes an empty record.
pub struct CanvasChildRecordPublication {
    pub document: CanvasDocumentIdentity,
    pub source_frame: u64,
    records: HashMap<CanvasElementIdentity, PublishedCanvasChildRecord>,
}
impl CanvasChildRecordPublication {
    pub fn new(
        document: CanvasDocumentIdentity,
        source_frame: u64,
        records: impl IntoIterator<Item = PublishedCanvasChildRecord>,
    ) -> Self {
        Self {
            document,
            source_frame,
            records: records
                .into_iter()
                .map(|record| (record.element, record))
                .collect(),
        }
    }
    pub fn Get(&self, element: CanvasElementIdentity) -> Option<&PublishedCanvasChildRecord> {
        self.records.get(&element)
    }
}
impl fmt::Debug for CanvasChildRecordPublication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CanvasChildRecordPublication")
            .field("document", &self.document)
            .field("source_frame", &self.source_frame)
            .field("records", &self.records.len())
            .finish()
    }
}

/// The source callback distinguishes a non-canvas id from an intentionally
/// empty canvas record (tainted, missing layout or unavailable front buffer).
#[derive(Clone, Debug)]
pub enum CanvasSnapshotReplacement {
    NotCanvas,
    EmptyCanvas,
    Bitmap(crate::CanvasPaintResource),
}
/// Captured at GetChildPaintRecord, never at compositor commit or later replay.
/// This preserves ElementImage's snapshot lifetime without copying PaintOps.
pub struct CanvasRecordWithSnapshots {
    pub source: CanvasRecord,
    pub replacements: std::collections::BTreeMap<u32, CanvasSnapshotReplacement>,
}
impl CanvasRecordReadLease for CanvasRecordWithSnapshots {
    fn BackendRecord(&self) -> &(dyn Any + Send + Sync) {
        self
    }
    fn VisitCustomData(&self, _: &mut dyn FnMut(u32)) {
        // Every placeholder was resolved by the original getter. A retained
        // ElementImage must not refresh its nested canvas images on replay.
    }
}
