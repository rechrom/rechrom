//! Canvas context ownership translated from Blink's
//! `core/html/canvas/html_canvas_element.{h,cc}`.

#![allow(non_snake_case)]

use crate::{
    CanvasContextCreationAttributesCore, CanvasContextFactory, CanvasContextType,
    CanvasPaintResource, CanvasRenderingContext,
};
use std::sync::atomic::{AtomicU64, Ordering};

// Canvas snapshots and decoded image resources share Raster's image catalog.
// Keep the source-like stable identity in a disjoint allocation domain until
// Chromium's process-wide PaintImage id allocator is translated.
static NEXT_CANVAS_PAINT_RESOURCE_ID: AtomicU64 = AtomicU64::new(1 << 63);

/// HTML's non-negative-integer parser used by the canvas width/height content
/// attributes. It accepts leading ASCII whitespace/`+`, ignores trailing
/// non-digits, and accepts unsigned minus-zero exactly as Blink does.
pub fn ParseCanvasDimension(input: &str) -> Option<u32> {
    let input = input.trim_start_matches(|character: char| character.is_ascii_whitespace());
    let (negative, input) = if let Some(input) = input.strip_prefix('-') {
        (true, input)
    } else {
        (false, input.strip_prefix('+').unwrap_or(input))
    };
    let digit_count = input.bytes().take_while(u8::is_ascii_digit).count();
    if digit_count == 0 {
        return None;
    }
    let value = input[..digit_count].parse::<u32>().ok()?;
    (!negative || value == 0).then_some(value)
}

/// State owned by one `HTMLCanvasElement` DOM node.
///
/// Blink permits one rendering-context family for an element. Repeated calls
/// for the same family return the same object; calls for another family return
/// null. Resizing resets the drawing buffer through the existing context.
pub struct HTMLCanvasElementState {
    width: u32,
    height: u32,
    paint_resource_id: u64,
    context: Option<Box<dyn CanvasRenderingContext>>,
    /// Content revision at context creation or the latest size reset. Blink's
    /// HTMLCanvasElement::DidDraw clears `canvas_is_clear_`; a revision change
    /// is the package-boundary equivalent when the API handle shares the
    /// rendering context state with this element owner.
    clear_revision: Option<u64>,
    hit_testable_descendants: crate::CanvasDrawnElementOrder,
    origin_clean: bool,
    // HTMLCanvasElement.h:478; changed only by the owning context transition.
    needs_unbuffered_input: bool,
}

impl Default for HTMLCanvasElementState {
    fn default() -> Self {
        Self::new(300, 150)
    }
}

impl HTMLCanvasElementState {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            paint_resource_id: NEXT_CANVAS_PAINT_RESOURCE_ID.fetch_add(1, Ordering::Relaxed),
            context: None,
            clear_revision: None,
            hit_testable_descendants: Default::default(),
            origin_clean: true,
            needs_unbuffered_input: false,
        }
    }

    // Source html_canvas_element.h:342-346; cc:586-591.
    pub fn NeedsUnbufferedInputEvents(&self) -> bool {
        self.needs_unbuffered_input
    }
    pub fn SetNeedsUnbufferedInputEvents(&mut self, value: bool) {
        self.needs_unbuffered_input = value;
    }

    pub fn Width(&self) -> u32 {
        self.width
    }

    pub fn Height(&self) -> u32 {
        self.height
    }
    pub fn HitTestableDescendants(&self) -> &crate::CanvasDrawnElementOrder {
        &self.hit_testable_descendants
    }
    /// HTMLCanvasElement::UpdateDrawnElementGeometry, cc:758-772. DOM has
    /// already checked the element's live CanvasForDrawing identity.
    pub fn UpdateDrawnElementHitTestOrder(&mut self, element: crate::CanvasElementIdentity) {
        self.hit_testable_descendants.AppendOrMoveToLast(element);
    }

    pub fn OriginClean(&self) -> bool {
        self.origin_clean
    }
    pub fn SetOriginTainted(&mut self) {
        self.origin_clean = false;
    }
    pub fn SetSize(&mut self, width: u32, height: u32) {
        self.origin_clean = true;
        self.width = width;
        self.height = height;
        if let Some(context) = self.context.as_mut() {
            context.Reshape(width, height);
            self.clear_revision = Some(context.ContentRevision());
        }
    }

    pub fn ContextType(&self) -> Option<CanvasContextType> {
        self.context
            .as_ref()
            .map(|context| context.GetContextType())
    }

    /// CanvasRenderingContextHost::IsWebGPU, cc:180-182, and
    /// CanvasRenderingContext::IsWebGPU, h:135-137. Read the actual context's
    /// family; neither the canvas tag nor feature flags identify its API.
    pub fn IsWebGPU(&self) -> bool {
        self.ContextType() == Some(CanvasContextType::WebGPU)
    }
    /// Source html_canvas_element.cc:1147-1149. Context loss and buffer reset
    /// do not replace the creation attributes of the retained context.
    pub fn LowLatencyEnabled(&self) -> bool {
        self.context
            .as_ref()
            .is_some_and(|context| context.CreationAttributes().desynchronized)
    }
    pub fn PaintResourceId(&self) -> u64 {
        self.paint_resource_id
    }

    /// Publish the context's current front-buffer result as an immutable
    /// paint resource. Source: HTMLCanvasElement::Snapshot and
    /// CanvasRenderingContext::PaintRenderingResultsToCanvas.
    pub fn SnapshotForPaint(
        &mut self,
        dom_node_id: u64,
        origin_clean: bool,
    ) -> Option<CanvasPaintResource> {
        if self.width == 0 || self.height == 0 {
            return None;
        }
        let context = self.context.as_mut()?;
        let revision = context.ContentRevision();
        let is_clear = self.clear_revision == Some(revision);
        let snapshot = context.PaintRenderingResultsToCanvas()?;
        if snapshot.width != self.width || snapshot.height != self.height {
            return None;
        }
        let width = snapshot.width;
        let height = snapshot.height;
        let pixels = snapshot.IntoTightlyPackedPixels()?;
        Some(CanvasPaintResource {
            dom_node_id,
            resource_id: self.paint_resource_id,
            revision,
            width,
            height,
            origin_clean: origin_clean && self.origin_clean,
            is_clear,
            pixels,
        })
    }

    /// Equivalent to Blink's context-family arbitration in
    /// `HTMLCanvasElement::GetCanvasRenderingContext`.
    pub fn GetCanvasRenderingContext<'a>(
        &'a mut self,
        factory: &mut dyn CanvasContextFactory,
        context_id: &str,
        attributes: CanvasContextCreationAttributesCore,
    ) -> Option<&'a mut (dyn CanvasRenderingContext + 'static)> {
        let context_type = CanvasContextType::FromContextId(context_id)?;
        if let Some(existing_type) = self.ContextType() {
            if existing_type != context_type {
                return None;
            }
            return match self.context.as_mut() {
                Some(context) => Some(context.as_mut()),
                None => unreachable!(),
            };
        }

        self.context = factory.Create(context_type, attributes, self.width, self.height);
        // Source html_canvas_element.cc:586-589 reads the accepted context
        // attributes, not the requested dictionary supplied to the factory.
        if self.LowLatencyEnabled() {
            self.SetNeedsUnbufferedInputEvents(true);
        }
        self.clear_revision = self
            .context
            .as_ref()
            .map(|context| context.ContentRevision());
        match self.context.as_mut() {
            Some(context) => Some(context.as_mut()),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CanvasPixelSnapshot;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct TestContext {
        context_type: CanvasContextType,
        sizes: Rc<RefCell<Vec<(u32, u32)>>>,
        snapshot: Option<CanvasPixelSnapshot>,
    }

    impl CanvasRenderingContext for TestContext {
        fn GetContextType(&self) -> CanvasContextType {
            self.context_type
        }
        fn CreationAttributes(&self) -> CanvasContextCreationAttributesCore {
            Default::default()
        }
        fn Reshape(&mut self, width: u32, height: u32) {
            self.sizes.borrow_mut().push((width, height));
        }
        fn IsContextLost(&self) -> bool {
            false
        }
        fn PaintRenderingResultsToCanvas(&mut self) -> Option<CanvasPixelSnapshot> {
            self.snapshot.take()
        }
    }

    struct TestFactory {
        creations: usize,
        fail: bool,
        sizes: Rc<RefCell<Vec<(u32, u32)>>>,
    }

    impl CanvasContextFactory for TestFactory {
        fn Create(
            &mut self,
            context_type: CanvasContextType,
            _: CanvasContextCreationAttributesCore,
            width: u32,
            height: u32,
        ) -> Option<Box<dyn CanvasRenderingContext>> {
            self.creations += 1;
            if self.fail {
                return None;
            }
            self.sizes.borrow_mut().push((width, height));
            Some(Box::new(TestContext {
                context_type,
                sizes: self.sizes.clone(),
                snapshot: None,
            }))
        }
    }

    #[test]
    fn default_dimensions_and_context_ids_match_html_canvas() {
        let canvas = HTMLCanvasElementState::default();
        assert_eq!((canvas.Width(), canvas.Height()), (300, 150));
        assert_eq!(
            CanvasContextType::FromContextId("experimental-webgl"),
            Some(CanvasContextType::WebGL1)
        );
        assert_eq!(CanvasContextType::FromContextId("WebGL"), None);
        assert_eq!(ParseCanvasDimension("  +12px"), Some(12));
        assert_eq!(ParseCanvasDimension("-0px"), Some(0));
        assert_eq!(ParseCanvasDimension("-1"), None);
    }

    #[test]
    fn same_family_reuses_context_and_another_family_is_rejected() {
        let sizes = Rc::new(RefCell::new(Vec::new()));
        let mut factory = TestFactory {
            creations: 0,
            fail: false,
            sizes,
        };
        let mut canvas = HTMLCanvasElementState::default();
        assert!(canvas
            .GetCanvasRenderingContext(&mut factory, "webgl", Default::default())
            .is_some());
        assert!(canvas
            .GetCanvasRenderingContext(&mut factory, "experimental-webgl", Default::default())
            .is_some());
        assert!(canvas
            .GetCanvasRenderingContext(&mut factory, "2d", Default::default())
            .is_none());
        assert_eq!(factory.creations, 1);
    }

    #[test]
    fn failed_creation_does_not_claim_canvas_and_resize_reaches_context() {
        let sizes = Rc::new(RefCell::new(Vec::new()));
        let mut factory = TestFactory {
            creations: 0,
            fail: true,
            sizes: sizes.clone(),
        };
        let mut canvas = HTMLCanvasElementState::new(640, 480);
        assert!(canvas
            .GetCanvasRenderingContext(&mut factory, "webgl", Default::default())
            .is_none());
        factory.fail = false;
        assert!(canvas
            .GetCanvasRenderingContext(&mut factory, "2d", Default::default())
            .is_some());
        canvas.SetSize(800, 600);
        assert_eq!(&*sizes.borrow(), &[(640, 480), (800, 600)]);
        assert_eq!(factory.creations, 2);
    }

    #[test]
    fn paint_snapshot_packs_stride_and_keeps_stable_resource_identity() {
        struct SnapshotFactory;
        impl CanvasContextFactory for SnapshotFactory {
            fn Create(
                &mut self,
                context_type: CanvasContextType,
                _: CanvasContextCreationAttributesCore,
                _: u32,
                _: u32,
            ) -> Option<Box<dyn CanvasRenderingContext>> {
                Some(Box::new(TestContext {
                    context_type,
                    sizes: Rc::new(RefCell::new(Vec::new())),
                    snapshot: Some(CanvasPixelSnapshot {
                        width: 2,
                        height: 1,
                        row_bytes: 12,
                        pixels: vec![1, 2, 3, 4, 5, 6, 7, 8, 91, 92, 93, 94],
                    }),
                }))
            }
        }
        let mut canvas = HTMLCanvasElementState::new(2, 1);
        let id = canvas.PaintResourceId();
        assert!(canvas
            .GetCanvasRenderingContext(&mut SnapshotFactory, "2d", Default::default())
            .is_some());
        let resource = canvas.SnapshotForPaint(42, true).unwrap();
        assert_eq!(resource.resource_id, id);
        assert_eq!(&*resource.pixels, &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(resource.dom_node_id, 42);
        assert_eq!(resource.revision, 0);
        assert!(resource.is_clear);
    }

    #[test]
    fn content_revision_tracks_drawing_and_resize_clear_state() {
        use std::cell::Cell;

        struct RevisionContext {
            revision: Rc<Cell<u64>>,
            width: u32,
            height: u32,
        }
        impl CanvasRenderingContext for RevisionContext {
            fn GetContextType(&self) -> CanvasContextType {
                CanvasContextType::Canvas2D
            }
            fn CreationAttributes(&self) -> CanvasContextCreationAttributesCore {
                Default::default()
            }
            fn Reshape(&mut self, width: u32, height: u32) {
                self.width = width;
                self.height = height;
                self.revision.set(self.revision.get() + 1);
            }
            fn IsContextLost(&self) -> bool {
                false
            }
            fn ContentRevision(&self) -> u64 {
                self.revision.get()
            }
            fn PaintRenderingResultsToCanvas(&mut self) -> Option<CanvasPixelSnapshot> {
                Some(CanvasPixelSnapshot {
                    width: self.width,
                    height: self.height,
                    row_bytes: self.width as usize * 4,
                    pixels: vec![0; self.width as usize * self.height as usize * 4],
                })
            }
        }
        struct RevisionFactory(Rc<Cell<u64>>);
        impl CanvasContextFactory for RevisionFactory {
            fn Create(
                &mut self,
                _: CanvasContextType,
                _: CanvasContextCreationAttributesCore,
                width: u32,
                height: u32,
            ) -> Option<Box<dyn CanvasRenderingContext>> {
                Some(Box::new(RevisionContext {
                    revision: self.0.clone(),
                    width,
                    height,
                }))
            }
        }

        let revision = Rc::new(Cell::new(0));
        let mut canvas = HTMLCanvasElementState::new(1, 1);
        assert!(canvas
            .GetCanvasRenderingContext(
                &mut RevisionFactory(revision.clone()),
                "2d",
                Default::default(),
            )
            .is_some());
        assert!(canvas.SnapshotForPaint(1, true).unwrap().is_clear);

        revision.set(1);
        assert!(!canvas.SnapshotForPaint(1, true).unwrap().is_clear);

        canvas.SetSize(1, 1);
        assert!(canvas.SnapshotForPaint(1, true).unwrap().is_clear);
    }

    #[test]
    fn paint_resource_equality_uses_revision_identity_not_pixel_scans() {
        let resource = CanvasPaintResource {
            dom_node_id: 7,
            resource_id: 1 << 63,
            revision: 11,
            width: 1,
            height: 1,
            origin_clean: true,
            is_clear: false,
            pixels: vec![1, 2, 3, 4].into(),
        };
        let mut same_revision = resource.clone();
        same_revision.pixels = vec![9, 8, 7, 6].into();
        assert_eq!(resource, same_revision);

        same_revision.revision += 1;
        assert_ne!(resource, same_revision);
    }
}
