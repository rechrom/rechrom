//! BaseRenderingContext2D::drawElementImage / DrawElementInternal source body.
//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15:
//! modules/canvas/canvas2d/base_rendering_context_2d.cc:1348-1619.
//! The Canvas2D recorder owns Draw's filter/blend/shadow execution. This source
//! passes its actual immutable child record; no bitmap or DOM rendering is made
//! by the geometry host. A Canvas2D recorder is still required to expose this API.
use crate::CanvasElementIdentity;
use foundation::{canvas_child_paint_state::CanvasChildPaintState, gfx};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawElementImageError {
    OffscreenCanvas,
    RequiresLayoutSubtree,
    NearestAncestorCanvas,
    DrawableRequired,
    NoCachedPaintRecord,
    ClosedElementImage,
    DifferentCanvas,
}
#[derive(Clone)]
pub struct CanvasChildPaintRecord<R: Clone> {
    pub paint_state: CanvasChildPaintState,
    pub record: R,
    pub canvas: CanvasElementIdentity,
    pub element: CanvasElementIdentity,
}
pub struct ElementImage<R: Clone> {
    record: Option<CanvasChildPaintRecord<R>>,
}
impl<R: Clone> ElementImage<R> {
    pub fn new(record: CanvasChildPaintRecord<R>) -> Self {
        Self {
            record: Some(record),
        }
    }
    pub fn close(&mut self) {
        self.record.take();
    }
    pub fn PaintRecord(&self) -> Option<&CanvasChildPaintRecord<R>> {
        self.record.as_ref()
    }
    pub fn TransferPaintRecord(&mut self) -> Option<CanvasChildPaintRecord<R>> {
        self.record.take()
    }
}
pub enum DrawElementImageSource<'a, R: Clone> {
    Element(CanvasElementIdentity),
    ElementImage(&'a ElementImage<R>),
}
#[derive(Clone, Copy)]
pub struct DrawElementImageArguments {
    pub crop: Option<[f64; 4]>,
    pub position: [f64; 2],
    pub size: Option<[f64; 2]>,
    /// DrawElementOptions::updateGeometry defaults to true.
    pub update_geometry: bool,
}
#[derive(Clone, Copy)]
pub struct ElementImageRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl ElementImageRect {
    fn Size(width: f32, height: f32) -> [f32; 2] {
        // gfx::SizeF's original trivial-dimension/NaN clamp, size_f.h:139.
        let clamp = |value| {
            if value > 8.0 * f32::EPSILON {
                value
            } else {
                0.0
            }
        };
        [clamp(width), clamp(height)]
    }
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        let [width, height] = Self::Size(width, height);
        Self {
            x,
            y,
            width,
            height,
        }
    }
    fn IsEmpty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }
}
/// The original Draw<kNone> command inputs. The owning 2D recorder executes its
/// image-paint pipeline, including the source nonopaque/filter/blend handling.
#[derive(Clone, Copy)]
pub enum ResolvedElementImageFilter {
    /// CanvasRenderingContext2DState::FilterState::kNone's resolved null filter.
    None,
}
pub struct ElementImageDraw<R> {
    pub record: R,
    pub filter: ResolvedElementImageFilter,
    pub source: ElementImageRect,
    pub destination: ElementImageRect,
    pub animated_image_frame_index_map: Option<Arc<BTreeMap<i32, usize>>>,
}
pub trait DrawElementImageGeometryHost {
    fn CanvasIdentity(&self) -> CanvasElementIdentity;
    fn VerifyElementEligibility(
        &self,
        element: CanvasElementIdentity,
    ) -> Result<(), DrawElementImageError>;
    fn UpdateDrawnElementGeometry(
        &self,
        element: CanvasElementIdentity,
        transform: gfx::Transform,
        update_hit_test_order: bool,
    );
}
/// Specific source Canvas2D/recorder inputs. This interface is implemented by
/// the owning context, rather than an embedding-provided drawing algorithm.
pub trait DrawElementImageContext {
    type Record: Clone;
    fn CanvasDrawElementEnabled(&self) -> bool;
    fn ElementCanvasTransformEnabled(&self) -> bool;
    fn GetOrCreatePaintCanvas(&self) -> bool;
    fn HasDocumentView(&self) -> bool;
    fn GeometryHost(&self) -> Option<&dyn DrawElementImageGeometryHost>;
    fn GetChildPaintRecord(
        &self,
        element: CanvasElementIdentity,
    ) -> Option<CanvasChildPaintRecord<Self::Record>>;
    fn CanvasSize(&self) -> [i32; 2];
    fn ResolveFilter(&self) -> ResolvedElementImageFilter;
    fn CurrentTransform(&self) -> gfx::Transform;
    fn DrawElementRecord(&self, draw: ElementImageDraw<Self::Record>);
}

pub fn DrawElementImage<C: DrawElementImageContext>(
    context: &C,
    source: DrawElementImageSource<'_, C::Record>,
    args: DrawElementImageArguments,
) -> Result<Option<gfx::Transform>, DrawElementImageError> {
    assert!(
        context.CanvasDrawElementEnabled(),
        "drawElementImage requires its execution-context feature"
    );
    if !context.GetOrCreatePaintCanvas() {
        return Ok(Some(gfx::Transform::default()));
    }
    let (record, element) = match source {
        DrawElementImageSource::Element(element) => {
            let host = context
                .GeometryHost()
                .ok_or(DrawElementImageError::OffscreenCanvas)?;
            if !context.HasDocumentView() {
                return Ok(None);
            }
            host.VerifyElementEligibility(element)?;
            (
                context
                    .GetChildPaintRecord(element)
                    .ok_or(DrawElementImageError::NoCachedPaintRecord)?,
                element,
            )
        }
        DrawElementImageSource::ElementImage(image) => {
            let record = image
                .PaintRecord()
                .ok_or(DrawElementImageError::ClosedElementImage)?;
            if context.GeometryHost().map(|host| host.CanvasIdentity()) != Some(record.canvas) {
                return Err(DrawElementImageError::DifferentCanvas);
            }
            (record.clone(), record.element)
        }
    };
    let state = &record.paint_state;
    let zoom = f64::from(state.effective_zoom);
    let src = args.crop.map_or(
        ElementImageRect::new(0.0, 0.0, state.box_size[0], state.box_size[1]),
        |[x, y, width, height]| {
            ElementImageRect::new(
                (x * zoom) as f32,
                (y * zoom) as f32,
                (width * zoom) as f32,
                (height * zoom) as f32,
            )
        },
    );
    if src.IsEmpty() {
        return Ok(Some(gfx::Transform::default()));
    }
    let filter = context.ResolveFilter();
    let grid_scale = state.GetCanvasGridScaleFactor(context.CanvasSize());
    let ideal = ElementImageRect::Size(src.width * grid_scale[0], src.height * grid_scale[1]);
    let size = args
        .size
        .map_or(ideal, |[width, height]| [width as f32, height as f32]);
    let dst = ElementImageRect::new(
        args.position[0] as f32,
        args.position[1] as f32,
        size[0],
        size[1],
    );
    if dst.IsEmpty() {
        return Ok(Some(gfx::Transform::default()));
    }
    context.DrawElementRecord(ElementImageDraw {
        record: record.record,
        filter,
        source: src,
        destination: dst,
        animated_image_frame_index_map: state.animated_image_frame_index_map.clone(),
    });
    let mut draw_transform = context.CurrentTransform();
    draw_transform.Translate(dst.x, dst.y);
    draw_transform.Scale(dst.width / ideal[0], dst.height / ideal[1]);
    if args.crop.is_some() {
        draw_transform.Translate(-src.x * grid_scale[0], -src.y * grid_scale[1]);
    }
    let canvas_transform_enabled = context.ElementCanvasTransformEnabled();
    let result = state.GetElementTransform(
        context.CanvasSize(),
        &draw_transform,
        canvas_transform_enabled,
    );
    if args.update_geometry && canvas_transform_enabled {
        context
            .GeometryHost()
            .expect("eligible source retains its canvas host")
            .UpdateDrawnElementGeometry(element, result.clone(), true);
    }
    Ok(Some(result))
}
