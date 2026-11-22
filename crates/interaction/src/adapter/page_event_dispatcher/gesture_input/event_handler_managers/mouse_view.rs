//! Live MouseEventManager FrameView/ChromeClient leaf projection.
//! Source: mouse_event_manager.cc:376-430,650-657,684-702,1256-1263;
//! svg_document_extensions.cc:103-122. The synthetic event reenters the same
//! transaction's source EventHandler through its existing frame registry.
use super::*;
use crate::adapter::page_event_handler_frame::PageEventHandlerFrameRegistry;

pub(super) struct PageEventHandlerMouseView<'scope, 'listener, 'frame> {
    pub(super) dispatcher: EventDispatcher<'listener, 'frame>,
    pub(super) registry: Rc<dyn PageEventHandlerFrameRegistry<'scope> + 'scope>,
    pub(super) identity: Rc<()>,
    pub(super) modifiers: i32,
    pub(super) page_active: bool,
    pub(super) cursor_visible: Option<bool>,
    pub(super) errors: Rc<InputErrors>,
}
impl PageEventHandlerMouseView<'_, '_, '_> {
    fn Coordinates(&self, point: PointF, integer: bool) -> PointF {
        let frame = self.dispatcher.mouse_frame_geometry.borrow().clone();
        let Some(frame) = frame else {
            self.errors.Fault(PointerInputError::MissingGeometry);
            return point;
        };
        let input = if integer {
            PointF::new(point.x().floor(), point.y().floor())
        } else {
            point
        };
        match frame.ConvertFromRootFrame(input) {
            Some(point) if integer => PointF::new(point.x().floor(), point.y().floor()),
            Some(point) => point,
            None => {
                self.errors.Fault(PointerInputError::SourceLeaf(
                    "LocalFrameView::ConvertFromRootFrame mouse",
                ));
                point
            }
        }
    }
}
impl MouseFrameViewPort for PageEventHandlerMouseView<'_, '_, '_> {
    fn HasView(&self) -> bool {
        self.dispatcher
            .mouse_frame_geometry
            .borrow()
            .as_ref()
            .is_some_and(|f| f.HasFrameView())
    }
    fn HasPage(&self) -> bool {
        self.dispatcher.focus_services.lifecycle.HasFrame()
    }
    fn IsLocalRoot(&self) -> bool {
        match self.registry.FrameForIdentity(&self.identity) {
            Some(frame) => frame.IsLocalRoot(),
            None => {
                self.errors.Fault(PointerInputError::LocalFrameRouting);
                false
            }
        }
    }
    fn IsPageActive(&self) -> bool {
        self.page_active
    }
    fn IsCursorVisible(&self) -> bool {
        match self.cursor_visible {
            Some(value) => value,
            None => {
                self.errors.Fault(PointerInputError::SourceLeaf(
                    "ChromeClient::IsCursorVisible",
                ));
                false
            }
        }
    }
    fn IsPointerLocked(&self) -> bool {
        let frame = self.dispatcher.mouse_frame_geometry.borrow().clone();
        match frame {
            Some(frame) => frame.PointerLockPosition().is_some(),
            None => {
                self.errors.Fault(PointerInputError::MissingGeometry);
                false
            }
        }
    }
    fn CurrentModifierState(&self) -> i32 {
        self.modifiers
    }
    fn ConvertFromRootFrame(&self, point: PointF) -> PointF {
        self.Coordinates(point, false)
    }
    fn ConvertIntPointFromRootFrame(&self, point: PointF) -> PointF {
        self.Coordinates(point, true)
    }
    fn ViewportToFrame(&self, point: PointF) -> PointF {
        // VisualViewport first maps viewport CSS coordinates into root-frame
        // space; LocalFrameView then maps that result into this frame.
        let frame = self.dispatcher.mouse_frame_geometry.borrow().clone();
        let Some(coordinates) = frame.and_then(|f| f.NonLocatedContextMenuCoordinates()) else {
            self.errors.Fault(PointerInputError::SourceLeaf(
                "VisualViewport::ViewportToRootFrame mouse",
            ));
            return point;
        };
        let root = PointF::new(
            point.x() / coordinates.visual_viewport_scale + coordinates.visual_viewport_offset.x(),
            point.y() / coordinates.visual_viewport_scale + coordinates.visual_viewport_offset.y(),
        );
        self.Coordinates(root, false)
    }
    fn HandleSyntheticMouseMove(&self, event: &WebMouseEvent) {
        if self.errors.failed.get() {
            return;
        }
        match self.registry.FrameForIdentity(&self.identity) {
            Some(frame) => {
                frame.Handler().HandleMouseMoveEvent(event, &[], &[]);
            }
            None => self.errors.Fault(PointerInputError::LocalFrameRouting),
        }
    }
    fn IsSvgDocumentWithZoomAndPan(&self) -> bool {
        let owner = self.dispatcher.document.borrow();
        let document = owner.GetDocument();
        document.DocumentType(document.Root()) == dom::persistent_document::DOMDocumentType::kSVG
            && document.SVGZoomAndPanEnabled()
    }
    fn StartSvgPan(&self, point: PointF) {
        if !self.errors.failed.get() {
            self.dispatcher
                .document
                .borrow_mut()
                .GetDocumentMut()
                .StartSVGPan([point.x(), point.y()]);
        }
    }
    fn UpdateSvgPan(&self, point: PointF) {
        if !self.errors.failed.get() {
            self.dispatcher
                .document
                .borrow_mut()
                .GetDocumentMut()
                .UpdateSVGPan([point.x(), point.y()]);
        }
    }
}

pub(super) struct PageEventHandlerMouseFeatures;
impl MouseFeatures for PageEventHandlerMouseFeatures {
    fn LightDismissFromClickEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::LightDismissFromClickEnabled()
    }
    fn SyntheticMouseHoverOverInactivePageEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::SyntheticMouseHoverOverInactivePageEnabled()
    }
    fn SuppressPointerStreamAfterDragEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::SuppressPointerStreamAfterDragEnabled()
    }
    fn BoundaryEventDispatchTracksNodeRemovalEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::BoundaryEventDispatchTracksNodeRemovalEnabled()
    }
    fn PseudoElementsFocusableEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::PseudoElementsFocusableEnabled()
    }
    fn BarrelButtonForDragEnabled(&self) -> bool {
        // Source Settings.json5:354-355. The embedding has no overriding setter.
        false
    }
}

impl<'scope, 'listener: 'scope, 'frame: 'scope>
    PageEventHandlerInputManagers<'scope, 'listener, 'frame>
{
    pub fn WithNativeView(
        dispatcher: &EventDispatcher<'listener, 'frame>,
        owners: &ResidentCursorOwners,
        nodes: Rc<PageEventHandlerNodes>,
        geometry: Option<Rc<dyn PagePointerGeometry>>,
        controls: Option<Rc<PagePointerScrollControlOwners>>,
        scroll: Rc<ScrollManager>,
        registry: Rc<dyn PageEventHandlerFrameRegistry<'scope> + 'scope>,
        identity: Rc<()>,
        modifiers: i32,
        page_active: bool,
        cursor_visible: Option<bool>,
    ) -> Result<Rc<Self>, PointerInputError> {
        let errors = Rc::new(InputErrors::default());
        let view = Rc::new(PageEventHandlerMouseView {
            dispatcher: dispatcher.clone(),
            registry,
            identity,
            modifiers,
            page_active,
            cursor_visible,
            errors: errors.clone(),
        });
        let mut managers = Self::new(
            dispatcher,
            owners,
            nodes,
            geometry,
            controls,
            scroll,
            view,
            Rc::new(PageEventHandlerMouseFeatures),
        )?;
        Rc::get_mut(&mut managers)
            .expect("binding has not escaped construction")
            .errors = errors;
        Ok(managers)
    }
}
