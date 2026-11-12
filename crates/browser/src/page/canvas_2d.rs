//! Source Canvas2D factory assembly. The host lends the same DOM canvas owner
//! and current Layout attachment; recorder and PaintOp decisions stay in their
//! owning Canvas and Raster packages.
use super::PageState;
use canvas::{
    canvas_2d::{Canvas2DContextFactory, Canvas2DHost},
    child_paint_record::{CanvasRecordWithSnapshots, CanvasSnapshotReplacement},
    draw_element_image::{DrawElementImageError, DrawElementImageGeometryHost},
    CanvasElementIdentity, PublishedCanvasChildRecord,
};
use std::{
    collections::BTreeMap,
    rc::{Rc, Weak},
    sync::Arc,
};

struct PageCanvas2DHost {
    state: Weak<PageState>,
    canvas: canvas::CanvasElementIdentity,
    geometry: Rc<dom::persistent_document::DOMCanvasDrawElementGeometry>,
}
impl DrawElementImageGeometryHost for PageCanvas2DHost {
    fn CanvasIdentity(&self) -> CanvasElementIdentity {
        self.canvas
    }
    fn VerifyElementEligibility(
        &self,
        element: CanvasElementIdentity,
    ) -> Result<(), DrawElementImageError> {
        self.geometry.VerifyElementEligibility(element)
    }
    fn UpdateDrawnElementGeometry(
        &self,
        element: CanvasElementIdentity,
        transform: foundation::gfx::Transform,
        hit_order: bool,
    ) {
        self.geometry
            .UpdateDrawnElementGeometry(element, transform, hit_order);
    }
}
impl Canvas2DHost for PageCanvas2DHost {
    fn HasDocumentView(&self) -> bool {
        self.state
            .upgrade()
            .is_some_and(|state| state.document.borrow().GetDocument().IsActive())
    }
    fn CanvasDrawElementEnabled(&self) -> bool {
        self.state.upgrade().is_some_and(|state| {
            state
                .document
                .borrow()
                .GetDocument()
                .CanvasRuntimeFeatures()
                .0
        })
    }
    fn ElementCanvasTransformEnabled(&self) -> bool {
        self.state.upgrade().is_some_and(|state| {
            state
                .document
                .borrow()
                .GetDocument()
                .CanvasRuntimeFeatures()
                .1
        })
    }
    fn GetChildPaintRecord(
        &self,
        element: CanvasElementIdentity,
    ) -> Option<PublishedCanvasChildRecord> {
        let state = self.state.upgrade()?;
        let mut record = state
            .document
            .borrow()
            .GetDocument()
            .CanvasChildPaintRecord(element)?;
        let replacements =
            state.SnapshotCanvasCustomData(|visitor| record.record.VisitCustomData(visitor));
        record.record = Arc::new(CanvasRecordWithSnapshots {
            source: record.record,
            replacements,
        });
        Some(record)
    }
    fn DidDraw(&self) {
        if let Some(state) = self.state.upgrade() {
            state.paint_resources_dirty.set(true);
            state.dirty.set(true);
        }
    }
}
impl PageState {
    /// Visit only source-referenced CustomData, retaining actual canvas
    /// snapshots at this call. No document collection or display-list copy.
    pub(super) fn SnapshotCanvasCustomData(
        &self,
        visit: impl FnOnce(&mut dyn FnMut(u32)),
    ) -> BTreeMap<u32, CanvasSnapshotReplacement> {
        let mut replacements = BTreeMap::new();
        visit(&mut |id| {
            if replacements.contains_key(&id) {
                return;
            }
            let attached = self
                .layout_engine
                .borrow()
                .GetLayoutTree()
                .and_then(|tree| tree.GetLayoutObjectForNode(u64::from(id)))
                .is_some();
            let replacement = self
                .document
                .borrow()
                .GetDocument()
                .CanvasSnapshotForCustomData(id, attached);
            replacements.insert(id, replacement);
        });
        replacements
    }

    /// CanvasContextFactory::Create remains the actual family arbitration entry
    /// on HTMLCanvasElementState; its returned API handle aliases that context.
    /// This source slice does not advertise the untranslated full JS 2D API.
    pub(super) fn CreateCanvas2DContextFactory(
        self: &Rc<Self>,
        canvas_node: u64,
    ) -> Option<Canvas2DContextFactory> {
        let document = self.document.borrow();
        let document = document.GetDocument();
        let node = document.FindNodeById(canvas_node)?;
        if !document.Node(node).IsHTMLElement("canvas") {
            return None;
        }
        let canvas = document.CanvasElementIdentity(node);
        let geometry = dom::persistent_document::DOMCanvasDrawElementGeometry::new(
            &self.document.Handle(),
            node,
        );
        Some(Canvas2DContextFactory::new(
            Rc::new(PageCanvas2DHost {
                state: Rc::downgrade(self),
                canvas,
                geometry,
            }),
            Rc::new(raster::Canvas2DRasterBackendFactory),
        ))
    }
}
