//! Owning Canvas2D recording/context slice, Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! BaseRenderingContext2D::DrawElementInternal cc:1483-1568 and context reset.
//! This implements the default image-paint state and its transform/global-alpha
//! stack. Untranslated filter/shadow/composite setters are not exposed as APIs.
use crate::{draw_element_image::*, *};
use foundation::gfx;
use std::{cell::RefCell, rc::Rc};

pub trait Canvas2DHost: DrawElementImageGeometryHost {
    fn HasDocumentView(&self) -> bool;
    fn CanvasDrawElementEnabled(&self) -> bool;
    fn ElementCanvasTransformEnabled(&self) -> bool;
    /// Source GetChildPaintRecord includes the call-time nested canvas snapshots.
    fn GetChildPaintRecord(
        &self,
        element: CanvasElementIdentity,
    ) -> Option<PublishedCanvasChildRecord>;
    fn DidDraw(&self);
}
/// The recorder passes commands and immutable leases; the receiving package
/// owns its raster allocation and source PaintOp execution, never a DOM owner.
pub trait Canvas2DBackend {
    /// Capability admission for the effective desynchronized attribute. CPU
    /// snapshot backends do not provide independently presented surfaces.
    fn SupportsLowLatencyPresentation(&self) -> bool {
        false
    }
    fn Reshape(&mut self, width: u32, height: u32) -> std::io::Result<()>;
    fn Playback(&mut self, commands: &[Canvas2DCommand]) -> std::io::Result<()>;
    fn Snapshot(&self) -> Option<CanvasPixelSnapshot>;
}
pub trait Canvas2DBackendFactory {
    fn Create(&self, width: u32, height: u32, alpha: bool) -> Option<Box<dyn Canvas2DBackend>>;
}
pub struct Canvas2DCommand {
    pub transform: gfx::Transform,
    pub global_alpha: f32,
    pub draw: ElementImageDraw<CanvasRecord>,
}
#[derive(Clone)]
struct DrawingState {
    transform: gfx::Transform,
    global_alpha: f32,
    resolved_filter: ResolvedElementImageFilter,
}
impl Default for DrawingState {
    fn default() -> Self {
        Self {
            transform: gfx::Transform::default(),
            global_alpha: 1.,
            resolved_filter: ResolvedElementImageFilter::None,
        }
    }
}
struct State {
    width: u32,
    height: u32,
    revision: u64,
    lost: bool,
    backend: Box<dyn Canvas2DBackend>,
    states: Vec<DrawingState>,
    commands: Vec<Canvas2DCommand>,
}
/// The API handle and HTMLCanvasElement context retain the same owner, as with
/// CanvasRenderingContext2D's original recorder/resource provider.
#[derive(Clone)]
pub struct CanvasRenderingContext2D {
    state: Rc<RefCell<State>>,
    host: Rc<dyn Canvas2DHost>,
    creation_attributes: CanvasContextCreationAttributesCore,
}
impl CanvasRenderingContext2D {
    pub fn Save(&self) {
        let mut s = self.state.borrow_mut();
        let state = s.states.last().expect("initial canvas state").clone();
        s.states.push(state);
    }
    pub fn Restore(&self) {
        let mut s = self.state.borrow_mut();
        if s.states.len() > 1 {
            s.states.pop();
        }
    }
    pub fn SetTransform(&self, transform: gfx::Transform) -> bool {
        if !transform.Is2dTransform()
            || !(0..4).all(|r| (0..4).all(|c| transform.rc(r, c).is_finite()))
        {
            return false;
        }
        self.state
            .borrow_mut()
            .states
            .last_mut()
            .expect("initial canvas state")
            .transform = transform;
        true
    }
    pub fn SetGlobalAlpha(&self, alpha: f64) {
        if (0. ..=1.).contains(&alpha) {
            self.state
                .borrow_mut()
                .states
                .last_mut()
                .expect("initial canvas state")
                .global_alpha = alpha as f32;
        }
    }
    pub fn DrawElementImage(
        &self,
        source: DrawElementImageSource<'_, CanvasRecord>,
        arguments: DrawElementImageArguments,
    ) -> Result<Option<gfx::Transform>, DrawElementImageError> {
        DrawElementImage(self, source, arguments)
    }
    pub fn Flush(&self) -> std::io::Result<()> {
        let mut s = self.state.borrow_mut();
        let commands = std::mem::take(&mut s.commands);
        if let Err(error) = s.backend.Playback(&commands) {
            s.lost = true;
            return Err(error);
        }
        Ok(())
    }
}
impl DrawElementImageContext for CanvasRenderingContext2D {
    type Record = CanvasRecord;
    fn CanvasDrawElementEnabled(&self) -> bool {
        self.host.CanvasDrawElementEnabled()
    }
    fn ElementCanvasTransformEnabled(&self) -> bool {
        self.host.ElementCanvasTransformEnabled()
    }
    fn GetOrCreatePaintCanvas(&self) -> bool {
        !self.state.borrow().lost
    }
    fn HasDocumentView(&self) -> bool {
        self.host.HasDocumentView()
    }
    fn GeometryHost(&self) -> Option<&dyn DrawElementImageGeometryHost> {
        Some(self.host.as_ref())
    }
    fn GetChildPaintRecord(
        &self,
        element: CanvasElementIdentity,
    ) -> Option<PublishedCanvasChildRecord> {
        self.host.GetChildPaintRecord(element)
    }
    fn CanvasSize(&self) -> [i32; 2] {
        let s = self.state.borrow();
        [s.width as i32, s.height as i32]
    }
    // The only admitted state here is the original default filter `none`.
    // No parser/setter is exposed that could leave an unresolved filter behind.
    fn ResolveFilter(&self) -> ResolvedElementImageFilter {
        // Source canvas_rendering_context_2d_state.cc:565-568 returns the
        // state's resolved filter immediately for the admitted kNone state.
        self.state
            .borrow()
            .states
            .last()
            .expect("initial canvas state")
            .resolved_filter
    }
    fn CurrentTransform(&self) -> gfx::Transform {
        self.state
            .borrow()
            .states
            .last()
            .expect("initial canvas state")
            .transform
            .clone()
    }
    fn DrawElementRecord(&self, draw: ElementImageDraw<CanvasRecord>) {
        let mut s = self.state.borrow_mut();
        let current = s.states.last().expect("initial canvas state").clone();
        // Canvas2DRecorderContext::Draw, h:868-878. An empty device clip or
        // singular state rejects the draw, but DrawElementInternal still
        // computes/returns its geometry transform after this call.
        if !current.transform.IsInvertible() || s.width == 0 || s.height == 0 {
            return;
        }
        s.commands.push(Canvas2DCommand {
            transform: current.transform,
            global_alpha: current.global_alpha,
            draw,
        });
        s.revision = s
            .revision
            .checked_add(1)
            .expect("canvas content revision overflow");
        drop(s);
        self.host.DidDraw();
    }
}
struct Canvas2DContextOwner(CanvasRenderingContext2D);
impl CanvasRenderingContext for Canvas2DContextOwner {
    fn GetContextType(&self) -> CanvasContextType {
        CanvasContextType::Canvas2D
    }
    fn CreationAttributes(&self) -> CanvasContextCreationAttributesCore {
        self.0.creation_attributes
    }
    fn Reshape(&mut self, width: u32, height: u32) {
        let mut s = self.0.state.borrow_mut();
        s.width = width;
        s.height = height;
        s.commands.clear();
        s.states = vec![DrawingState::default()];
        s.lost = s.backend.Reshape(width, height).is_err();
        s.revision = s
            .revision
            .checked_add(1)
            .expect("canvas content revision overflow");
    }
    fn IsContextLost(&self) -> bool {
        self.0.state.borrow().lost
    }
    fn ContentRevision(&self) -> u64 {
        self.0.state.borrow().revision
    }
    fn PaintRenderingResultsToCanvas(&mut self) -> Option<CanvasPixelSnapshot> {
        self.0.Flush().ok()?;
        self.0.state.borrow().backend.Snapshot()
    }
}
pub struct Canvas2DContextFactory {
    host: Rc<dyn Canvas2DHost>,
    backend: Rc<dyn Canvas2DBackendFactory>,
    created: Option<CanvasRenderingContext2D>,
}
impl Canvas2DContextFactory {
    pub fn new(host: Rc<dyn Canvas2DHost>, backend: Rc<dyn Canvas2DBackendFactory>) -> Self {
        Self {
            host,
            backend,
            created: None,
        }
    }
    pub fn TakeCreated(&mut self) -> Option<CanvasRenderingContext2D> {
        self.created.take()
    }
}
impl CanvasContextFactory for Canvas2DContextFactory {
    fn Create(
        &mut self,
        kind: CanvasContextType,
        mut attributes: CanvasContextCreationAttributesCore,
        width: u32,
        height: u32,
    ) -> Option<Box<dyn CanvasRenderingContext>> {
        if kind != CanvasContextType::Canvas2D
            || width > i32::MAX as u32
            || height > i32::MAX as u32
        {
            return None;
        }
        let backend = self.backend.Create(width, height, attributes.alpha)?;
        // This recorder publishes immutable CPU snapshots. There is no
        // low-latency surface/layer capability on Canvas2DBackend yet; keep
        // the requested value but reject the effective desynchronized mode.
        // Source canvas_context_creation_attributes_core.h:34-39 explicitly
        // permits platform admission to disable the effective attribute.
        attributes.desynchronized &= backend.SupportsLowLatencyPresentation();
        let context = CanvasRenderingContext2D {
            state: Rc::new(RefCell::new(State {
                width,
                height,
                revision: 0,
                lost: false,
                backend,
                states: vec![DrawingState::default()],
                commands: Vec::new(),
            })),
            host: self.host.clone(),
            creation_attributes: attributes,
        };
        self.created = Some(context.clone());
        Some(Box::new(Canvas2DContextOwner(context)))
    }
}
