//! Canonical HTMLCanvasElement/Element state; source cc:758-784,961-996;
//! Element::CanvasForDrawing/SetCanvasTransformInternal cc:10572-10602,4552-4575.
use super::*;

/// The rendering context keeps this weak alias of its original canvas element.
/// It resolves the live DOM identities used by the source draw call.
pub struct DOMCanvasDrawElementGeometry {
    owner: std::rc::Weak<RefCell<DOM>>,
    canvas: DOMOwnerHandle,
    enabled: bool,
}
impl DOMCanvasDrawElementGeometry {
    pub fn new(owner: &Rc<RefCell<DOM>>, canvas: usize) -> Rc<Self> {
        let (identity, enabled) = {
            let owner = owner.borrow();
            let document = owner.GetDocument();
            assert!(document.Node(canvas).IsHTMLElement("canvas"));
            (
                document.NodeHandle(canvas),
                document.canvas_runtime_features.get().0,
            )
        };
        Rc::new(Self {
            owner: Rc::downgrade(owner),
            canvas: identity,
            enabled,
        })
    }
}
impl canvas::draw_element_image::DrawElementImageGeometryHost for DOMCanvasDrawElementGeometry {
    fn CanvasIdentity(&self) -> canvas::CanvasElementIdentity {
        canvas::CanvasElementIdentity(self.canvas.Identity())
    }
    fn VerifyElementEligibility(
        &self,
        element: canvas::CanvasElementIdentity,
    ) -> Result<(), canvas::draw_element_image::DrawElementImageError> {
        use canvas::draw_element_image::DrawElementImageError as Error;
        let owner = self.owner.upgrade().ok_or(Error::NearestAncestorCanvas)?;
        let owner = owner.borrow();
        let document = owner.GetDocument();
        let canvas = document
            .FindNodeById(self.canvas.Id())
            .filter(|node| document.NodeHandle(*node) == self.canvas)
            .ok_or(Error::NearestAncestorCanvas)?;
        let element = document
            .FindNodeById(element.0 as u64)
            .filter(|node| {
                document.NodeHandle(*node).Identity() == element.0
                    && document.Node(*node).Type() == DOMNodeType::kElement
            })
            .ok_or(Error::NearestAncestorCanvas)?;
        document.VerifyCanvasDrawElementImageEligibility(canvas, element, self.enabled)
    }
    fn UpdateDrawnElementGeometry(
        &self,
        element: canvas::CanvasElementIdentity,
        transform: foundation::gfx::Transform,
        update_hit_test_order: bool,
    ) {
        let Some(owner) = self.owner.upgrade() else {
            return;
        };
        let mut owner = owner.borrow_mut();
        let document = owner.GetDocumentMut();
        let Some(canvas) = document
            .FindNodeById(self.canvas.Id())
            .filter(|node| document.NodeHandle(*node) == self.canvas)
        else {
            return;
        };
        let Some(element) = document
            .FindNodeById(element.0 as u64)
            .filter(|node| document.NodeHandle(*node).Identity() == element.0)
        else {
            return;
        };
        document.UpdateDrawnElementGeometry(
            canvas,
            element,
            Some(transform),
            update_hit_test_order,
            self.enabled,
        );
    }
}

impl PersistentDocument {
    /// LocalFrameView::GetCanvasSnapshot's CustomData node-class branch.
    /// The caller routes only the current Layout attachment scalar.
    pub fn CanvasSnapshotForCustomData(
        &self,
        id: u32,
        attached: bool,
    ) -> canvas::child_paint_record::CanvasSnapshotReplacement {
        use canvas::child_paint_record::CanvasSnapshotReplacement as R;
        let Some(node) = self.FindNodeById(u64::from(id)) else {
            return R::NotCanvas;
        };
        if !matches!(self.arena[node].class, DOMNodeClass::HTMLCanvasElement) {
            return R::NotCanvas;
        }
        let owner = self.CanvasElementState(node);
        let mut owner = owner.borrow_mut();
        if !attached || !owner.OriginClean() {
            return R::EmptyCanvas;
        }
        owner
            .SnapshotForPaint(u64::from(id), true)
            .map_or(R::EmptyCanvas, R::Bitmap)
    }

    pub fn CanvasDocumentIdentity(&self) -> canvas::CanvasDocumentIdentity {
        canvas::CanvasDocumentIdentity(self.arena_id)
    }
    pub fn PublishCanvasChildRecords(
        &self,
        publication: std::sync::Arc<canvas::CanvasChildRecordPublication>,
    ) {
        if publication.document != self.CanvasDocumentIdentity() {
            return;
        }
        let mut current = self.canvas_child_records.borrow_mut();
        if current
            .as_ref()
            .is_some_and(|current| current.source_frame > publication.source_frame)
        {
            return;
        }
        *current = Some(publication);
    }
    pub fn CanvasChildPaintRecord(
        &self,
        element: canvas::CanvasElementIdentity,
    ) -> Option<canvas::PublishedCanvasChildRecord> {
        self.canvas_child_records
            .borrow()
            .as_ref()?
            .Get(element)
            .cloned()
    }
    /// Rendering-opportunity front buffers come from the canonical canvas
    /// contexts, including typed non-script callers. This visits canvas owners,
    /// never the DOM tree or a WebAPI-only context registry.
    pub fn SnapshotCanvasPaintResources(&self) -> Vec<canvas::CanvasPaintResource> {
        self.canvas_elements
            .borrow()
            .iter()
            .filter_map(|(&id, owner)| {
                let mut owner = owner.borrow_mut();
                let clean = owner.OriginClean();
                owner.SnapshotForPaint(id, clean)
            })
            .collect()
    }
    pub fn CanvasRuntimeFeatures(&self) -> (bool, bool) {
        self.canvas_runtime_features.get()
    }
    pub fn InitializeCanvasRuntimeFeatures(&self, draw_element: bool, element_transform: bool) {
        assert!(
            !self.IsActive(),
            "execution-context Canvas features initialize before Document attachment"
        );
        self.canvas_runtime_features
            .set((draw_element, element_transform));
    }
    pub fn UsedCanvasTransformForLayout(&self, node: usize) -> Option<&foundation::gfx::Transform> {
        let (draw_element, element_transform) = self.canvas_runtime_features.get();
        (element_transform && self.CanvasForDrawing(node, draw_element).is_some())
            .then(|| self.canvas_transforms.get(&self.Node(node).Id()))
            .flatten()
    }
    pub fn CanvasElementIdentity(&self, node: usize) -> canvas::CanvasElementIdentity {
        assert_eq!(self.Node(node).Type(), DOMNodeType::kElement);
        canvas::CanvasElementIdentity(self.NodeHandle(node).Identity())
    }
    pub fn CanvasElementState(&self, node: usize) -> Rc<RefCell<canvas::HTMLCanvasElementState>> {
        assert!(self.Node(node).IsHTMLElement("canvas"));
        let dimensions = self.CanvasDimensions(node);
        self.canvas_elements
            .borrow_mut()
            .entry(self.Node(node).Id())
            .or_insert_with(|| {
                Rc::new(RefCell::new(canvas::HTMLCanvasElementState::new(
                    dimensions.0,
                    dimensions.1,
                )))
            })
            .clone()
    }
    pub fn ExistingCanvasElementState(
        &self,
        node: usize,
    ) -> Option<Rc<RefCell<canvas::HTMLCanvasElementState>>> {
        self.canvas_elements
            .borrow()
            .get(&self.Node(node).Id())
            .cloned()
    }
    /// PointerEventManager::DispatchPointerEvent, pointer_event_manager.cc:
    /// 211-216, reads HTMLCanvasElement's retained context transition flag.
    /// A query does not instantiate a Canvas owner or infer admission from the
    /// element tag. Resizing/loss retain the same context and flag.
    pub fn CanvasNeedsUnbufferedInputEvents(&self, node: usize) -> bool {
        self.ExistingCanvasElementState(node)
            .is_some_and(|owner| owner.borrow().NeedsUnbufferedInputEvents())
    }
    fn CanvasDimensions(&self, node: usize) -> (u32, u32) {
        let dimension = |name, fallback| {
            self.Node(node)
                .FindAttribute(name)
                .and_then(|attr| canvas::ParseCanvasDimension(&attr.value))
                .filter(|value| *value <= i32::MAX as u32)
                .unwrap_or(fallback)
        };
        (dimension("width", 300), dimension("height", 150))
    }
    pub(super) fn CanvasDimensionAttributeChanged(&self, node: usize, name: &str, namespace: &str) {
        if !namespace.is_empty()
            || !matches!(name, "width" | "height")
            || !self.Node(node).IsHTMLElement("canvas")
        {
            return;
        }
        if let Some(owner) = self.ExistingCanvasElementState(node) {
            let (width, height) = self.CanvasDimensions(node);
            owner.borrow_mut().SetSize(width, height);
        }
    }
    fn CanvasFlatParentSkippingSlots(&self, node: usize) -> Option<usize> {
        let mut parent = self.FlatTreeParentElement(node)?;
        while self.Node(parent).IsHTMLElement("slot") {
            parent = self.FlatTreeParentElement(parent)?;
        }
        Some(parent)
    }
    fn VerifyCanvasDrawElementImageEligibility(
        &self,
        canvas: usize,
        element: usize,
        enabled: bool,
    ) -> Result<(), canvas::draw_element_image::DrawElementImageError> {
        use canvas::draw_element_image::DrawElementImageError as Error;
        if self.Node(canvas).FindAttribute("layoutsubtree").is_none() {
            return Err(Error::RequiresLayoutSubtree);
        }
        let direct_child = self.CanvasFlatParentSkippingSlots(element) == Some(canvas);
        if !direct_child && self.CanvasForDrawing(element, enabled) != Some(canvas) {
            return Err(Error::NearestAncestorCanvas);
        }
        if !direct_child && self.Node(element).FindAttribute("drawable").is_none() {
            return Err(Error::DrawableRequired);
        }
        Ok(())
    }
    /// Original flat-tree ancestry decision. Source weak drawing order never
    /// keeps a removed or adopted element eligible merely because it was drawn.
    pub fn CanvasForDrawing(&self, node: usize, enabled: bool) -> Option<usize> {
        if !enabled
            || !self.IsConnectedToDocument(node)
            || self.Node(node).Type() != DOMNodeType::kElement
        {
            return None;
        }
        let parent = self.CanvasFlatParentSkippingSlots(node)?;
        let is_canvas = |node| self.Node(node).IsHTMLElement("canvas");
        let layout_subtree = |node| self.Node(node).FindAttribute("layoutsubtree").is_some();
        if is_canvas(parent) {
            return layout_subtree(parent).then_some(parent);
        }
        if self.Node(node).FindAttribute("drawable").is_none() {
            return None;
        }
        let mut ancestor = Some(parent);
        while let Some(node) = ancestor {
            ancestor = self.CanvasFlatParentSkippingSlots(node);
            if let Some(canvas) = ancestor.filter(|node| is_canvas(*node)) {
                return layout_subtree(canvas).then_some(canvas);
            }
        }
        None
    }
    pub fn UpdateDrawnElementGeometry(
        &mut self,
        canvas: usize,
        element: usize,
        transform: Option<foundation::gfx::Transform>,
        update_hit_test_order: bool,
        enabled: bool,
    ) {
        if self.CanvasForDrawing(element, enabled) != Some(canvas) {
            return;
        }
        if update_hit_test_order {
            self.CanvasElementState(canvas)
                .borrow_mut()
                .UpdateDrawnElementHitTestOrder(canvas::CanvasElementIdentity(
                    self.NodeHandle(element).Identity(),
                ));
        }
        if let Some(transform) = transform {
            let id = self.Node(element).Id();
            if self.canvas_transforms.get(&id) != Some(&transform) {
                self.canvas_transforms.insert(id, transform);
                self.RecordLayoutGeometryChange(element);
                self.InvalidateNodeStyle(element);
            }
        }
    }
    /// One step of the source weak linked-set reverse cursor. This yields only
    /// its original identity; the resident Layout lease resolves the layer.
    pub fn CanvasDrawnElementNode(
        &self,
        canvas: usize,
        element: canvas::CanvasElementIdentity,
        enabled: bool,
    ) -> Option<u64> {
        let id = element.0 as u64;
        let node = self.FindNodeById(id)?;
        (self.NodeHandle(node).Identity() == element.0
            && self.CanvasForDrawing(node, enabled) == Some(canvas))
        .then_some(id)
    }
}
