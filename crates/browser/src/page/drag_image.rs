//! Assembly of DataTransfer/DragController's existing native Layout, Paint,
//! FontEngine, compositor conversion and Raster leaves. No page-artifact crop.
use super::*;
use interaction::adapter::page_pointer_hit_test::PagePointerGeometry;
use interaction::default::core::page::{
    drag_controller::DragOverlay,
    drag_image::{self as source, DragLabelFontContext},
};
use interaction::dispatch::core::clipboard::data_transfer::{
    DataTransfer, DragImageBitmap, DragImageResource, DragVisualViewportClip,
};

/// DataTransfer.cc:422-425,509-515. Retain CachedImage itself so completion and
/// the current animation frame are read when DragController creates the image.
struct CachedDragImageResource(Rc<dom::persistent_document::DOMImageResourceContent>);
pub(super) fn RetainImageResource(
    content: Rc<dom::persistent_document::DOMImageResourceContent>,
) -> Rc<dyn DragImageResource> {
    Rc::new(CachedDragImageResource(content))
}
impl CachedDragImageResource {
    fn CurrentBitmap(&self) -> Option<image_resource::ImmutableImageBitmap> {
        let image = self.0.ImageSnapshot()?;
        if image.IsDocumentImage() {
            let size = self
                .0
                .ConcreteImageSize(1., (image.width as f32, image.height as f32))
                .ok()??;
            return self
                .0
                .BitmapForCurrentFrame(Some(image_resource::DocumentImageBitmapRequest {
                    width: size.0,
                    height: size.1,
                    zoom: 1.,
                    preferred_color_scheme: image
                        .document_container_key
                        .as_ref()
                        .map_or(0, |key| key.preferred_color_scheme),
                }))
                .ok();
        }
        // DragImage::Create's default respects bitmap orientation and keeps
        // full opacity. Its later Scale is a separate native raster step.
        raster::ResizeAndOrientDragImage(&image, true, (1., 1.), 1., false)
            .ok()
            .flatten()
    }
}
impl DragImageResource for CachedDragImageResource {
    fn Bitmap(&self) -> Option<Arc<DragImageBitmap>> {
        self.CurrentBitmap().map(Packet)
    }
    fn BitmapScaled(&self, scale: f32) -> Option<Arc<DragImageBitmap>> {
        let bitmap = self.CurrentBitmap()?;
        if scale == 1. {
            return Some(Packet(bitmap));
        }
        raster::ResizeDragImageBitmap(&bitmap, (scale, scale), 1., false)
            .ok()
            .flatten()
            .map(Packet)
    }
}

fn Packet(bitmap: image_resource::ImmutableImageBitmap) -> Arc<DragImageBitmap> {
    Arc::new(DragImageBitmap {
        width: bitmap.width,
        height: bitmap.height,
        row_bytes: bitmap.width as usize * 4,
        rgba8: bitmap.rgba8,
    })
}
struct DraggedNode {
    state: Rc<PageState>,
    node: u64,
}
impl DraggedNode {
    fn Set(&self, dragged: bool) {
        let mut owner = self.state.document.borrow_mut();
        let tree = owner.GetDocumentMut();
        let Some(root) = tree.FindNodeById(self.node) else {
            return;
        };
        // NodeTraversal::InclusiveDescendantsOf, data_transfer.cc:94-104.
        // This is a live DOM traversal and stores no duplicate document tree.
        let mut current = Some(root);
        let layout = self.state.layout_engine.borrow();
        while let Some(node) = current {
            tree.SetNodeDragged(
                node,
                dragged,
                layout.MouseCoordinateLayout(tree.Node(node).Id()).is_some(),
            );
            current = if let Some(child) = tree.Node(node).Children().first() {
                Some(*child)
            } else {
                let mut next = node;
                loop {
                    if next == root {
                        break None;
                    }
                    let Some(parent) = tree.Node(next).Parent() else {
                        break None;
                    };
                    let siblings = tree.Node(parent).Children();
                    let offset = siblings
                        .iter()
                        .position(|n| *n == next)
                        .expect("live child identity");
                    if let Some(sibling) = siblings.get(offset + 1) {
                        break Some(*sibling);
                    }
                    next = parent;
                }
            };
        }
        drop(layout);
        drop(owner);
        self.state.InvalidateStylePreservingGeometry();
    }
}
impl Drop for DraggedNode {
    fn drop(&mut self) {
        self.Set(false);
    }
}
struct Viewport<'a>(&'a PageState);
impl DragVisualViewportClip for Viewport<'_> {
    fn VisibleRect(&self) -> foundation::gfx::RectF {
        let c = self.0.constraints.borrow();
        foundation::gfx::RectF::new(
            foundation::gfx::PointF::default(),
            foundation::gfx::SizeF::new(
                c.viewport
                    .map_or(c.available_size.width as f32, |v| v.size.width as f32),
                c.viewport
                    .map_or(c.available_size.height as f32, |v| v.size.height as f32),
            ),
        )
    }
    fn ConvertRectFromRootFrame(&self, rect: foundation::gfx::Rect) -> foundation::gfx::Rect {
        let scroll = self
            .0
            .layout_engine
            .borrow()
            .MouseLayoutViewportScrollOffsetInt()
            .unwrap_or_default();
        foundation::gfx::Rect::new(
            foundation::gfx::Point::new(rect.x() + scroll.0, rect.y() + scroll.1),
            rect.size(),
        )
    }
}
impl PageState {
    fn RasterDragRecording(
        &self,
        recording: paint::drag_image::DragImageRecording,
        opacity: f32,
    ) -> io::Result<Option<Arc<DragImageBitmap>>> {
        let (record, raster) = {
            let pipeline = self.rendering.borrow();
            let compositor = pipeline.compositor.as_ref().ok_or_else(|| {
                io::Error::other("drag image requires the Page compositor's Paint services")
            })?;
            (
                compositor.ConvertPaintArtifactSubsetToRecord(
                    recording.artifact,
                    &recording.border_box_properties,
                ),
                pipeline.raster_engine.clone(),
            )
        };
        let snapshots = self.SnapshotCanvasCustomData(|visitor| record.VisitCustomData(visitor));
        // Page's VisualViewport scale is currently 1; device scale is already
        // represented by its source Layout/Paint coordinate contract.
        raster
            .RasterizeDragImageRecord(
                &record,
                (recording.layout_size.width, recording.layout_size.height),
                (recording.paint_offset.x, recording.paint_offset.y),
                1.,
                opacity,
                &snapshots,
            )
            .map(|image| image.map(Packet))
    }
}
impl PageMouseFrameGeometry {
    pub(super) fn NodeDragImage(&self, node: u64) -> io::Result<Option<Arc<DragImageBitmap>>> {
        let state = self
            .0
            .upgrade()
            .ok_or_else(|| io::Error::other("detached drag document"))?;
        let guard = DraggedNode {
            state: state.clone(),
            node,
        };
        guard.Set(true);
        pointer_geometry::PagePointerGeometryProvider(self.0.clone())
            .PrepareLifecycleExceptPaint()
            .map_err(|error| io::Error::other(format!("{error:?}")))?;
        let geometry = {
            let layout = state.layout_engine.borrow();
            layout
                .MouseCoordinateLayout(node)
                .and_then(|source| source.DragNodeImageGeometry(&layout))
        };
        let Some(geometry) = geometry else {
            return Ok(None);
        };
        let b = geometry.bounding_box;
        let recording = state.paint_engine.borrow_mut().RecordNodeDragImage(
            geometry.layer_client_id,
            paint::paint_engine::PaintRect {
                x: f64::from(b.x()),
                y: f64::from(b.y()),
                width: f64::from(b.width()),
                height: f64::from(b.height()),
            },
            state.layout_editing.PaintState(),
            &PagePaintLifecycle(state.DocumentLifecycle()),
        );
        let result = recording
            .map(|record| state.RasterDragRecording(record, 1.))
            .transpose()
            .map(Option::flatten);
        drop(guard);
        result
    }
    pub(super) fn SelectionDragOverlay(&self) -> io::Result<DragOverlay> {
        let state = self
            .0
            .upgrade()
            .ok_or_else(|| io::Error::other("detached drag document"))?;
        let Some(selection) = Self::CurrentSelection(&state) else {
            return Ok(DragOverlay::default());
        };
        if selection.anchor == selection.focus {
            return Ok(DragOverlay::default());
        }
        pointer_geometry::PagePointerGeometryProvider(self.0.clone())
            .PrepareLifecycleExceptPaint()
            .map_err(|error| io::Error::other(format!("{error:?}")))?;
        let selection_host =
            super::frame_selection_host::ResidentFrameSelectionHost::Root(self.0.clone());
        let bounds = interaction::adapter::page_frame_selection::PageFrameSelectionHost::AbsoluteSelectionBounds(
            selection_host.as_ref(),
        );
        let rect = DataTransfer::ClipByVisualViewport(
            foundation::gfx::RectF::from(bounds),
            &Viewport(&state),
        );
        let recording = state.paint_engine.borrow_mut().RecordSelectionDragImage(
            paint::paint_engine::PaintRect {
                x: f64::from(rect.x()),
                y: f64::from(rect.y()),
                width: f64::from(rect.width()),
                height: f64::from(rect.height()),
            },
            state.layout_editing.PaintState(),
            &PagePaintLifecycle(state.DocumentLifecycle()),
        );
        let image = recording
            .map(|record| state.RasterDragRecording(record, 0.75))
            .transpose()?
            .flatten();
        Ok(DragOverlay {
            image,
            rect: foundation::gfx::ToEnclosingRect(rect),
        })
    }
    pub(super) fn ResourceDragImage(
        &self,
        node: u64,
        size: (i32, i32),
    ) -> io::Result<Option<Arc<DragImageBitmap>>> {
        let state = self
            .0
            .upgrade()
            .ok_or_else(|| io::Error::other("detached drag document"))?;
        let Some(facts) = state
            .layout_engine
            .borrow()
            .MouseCoordinateLayout(node)
            .and_then(|source| source.DragResourceImageFacts())
        else {
            return Ok(None);
        };
        let Some(content) = facts.resource_id.and_then(|id| {
            state
                .document
                .borrow()
                .GetDocument()
                .ImageResourceContentById(id)
        }) else {
            return Ok(None);
        };
        let Some(image) = content.ImageSnapshot() else {
            return Ok(None);
        };
        let oriented = image.ForceOrientationIfNecessary(facts.respect_orientation)
            && image.orientation.uses_width_as_height();
        let natural = if oriented {
            (image.height, image.width)
        } else {
            (image.width, image.height)
        };
        // Pinned DragImageForLargeImages is stable: clamp before allocating.
        let scale = source::ClampedImageScale(
            natural,
            size,
            state.constraints.borrow().device_pixel_ratio as f32,
        );
        let nearest = matches!(
            facts.image_rendering,
            foundation::EImageRendering::kPixelated | foundation::EImageRendering::kCrispEdges
        );
        let bitmap = if let image_resource::PaintImageContent::Document(document) = &image.content {
            let key = image
                .document_container_key
                .as_ref()
                .ok_or_else(|| io::Error::other("SVG drag image has no source container"))?;
            let record = document
                .record_for(key)
                .and_then(
                    image_resource::downcast_document_paint_record::<
                        paint::paint_engine::DocumentPaintArtifactRecord,
                    >,
                )
                .ok_or_else(|| io::Error::other("SVG drag image has no exact container record"))?;
            raster::RasterizeDocumentDragImage(record, natural, scale, 0.75)?
        } else {
            raster::ResizeAndOrientDragImage(
                &image,
                facts.respect_orientation,
                scale,
                0.75,
                nearest,
            )?
        };
        Ok(bitmap.map(Packet))
    }
    pub(super) fn LinkDragImage(
        &self,
        url: &str,
        label: &[u16],
    ) -> io::Result<Option<Arc<DragImageBitmap>>> {
        let state = self
            .0
            .upgrade()
            .ok_or_else(|| io::Error::other("detached drag document"))?;
        let scale = state.constraints.borrow().device_pixel_ratio as f32;
        let image = source::CreateLinkDragImage(&DragFontContext(&state), url, label, scale)?;
        let result = state
            .rendering
            .borrow()
            .raster_engine
            .RasterizeLinkDragImage(image.size, scale, &image.text)
            .map(|image| image.map(Packet));
        result
    }
}
struct DragFontContext<'a>(&'a PageState);
impl DragFontContext<'_> {
    fn Compute<T>(
        &self,
        mut compute: impl FnMut(&mut font_engine::FontEngine) -> font_engine::FontCompute<T>,
    ) -> io::Result<T> {
        loop {
            let result = self.0.document_font_inputs.engine.with_mut(&mut compute);
            match result {
                font_engine::FontCompute::Ready(value) => return Ok(value),
                font_engine::FontCompute::Error(error) => {
                    return Err(io::Error::other(format!("drag label font: {error:?}")));
                }
                font_engine::FontCompute::Pending(pending) => self
                    .0
                    .system_fonts
                    .borrow_mut()
                    .CompletePending(pending, &mut self.0.constraints.borrow_mut())
                    .map_err(|error| io::Error::other(format!("drag label font: {error:?}")))?,
            }
        }
    }
}
impl DragLabelFontContext for DragFontContext<'_> {
    type Error = io::Error;
    fn Metrics(
        &self,
        description: &font_engine::FontDescription,
    ) -> io::Result<font_engine::FontMetrics> {
        self.Compute(|engine| engine.GetMetrics(description, engine.ContextVersion()))
    }
    fn Shape(
        &self,
        text: &[u16],
        description: &font_engine::FontDescription,
        direction: foundation::TextDirection,
        bidi: bool,
    ) -> io::Result<font_engine::plain_text_painter::PlainTextPaint> {
        self.Compute(|engine| {
            font_engine::plain_text_painter::PlainTextPainter::Shape(
                engine,
                text,
                description,
                direction,
                bidi,
            )
        })
    }
}
