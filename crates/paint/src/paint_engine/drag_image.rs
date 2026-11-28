//! Special image recording over the resident Layout/PrePaint owners.
//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15:
//! core/clipboard/data_transfer.cc:108-157; core/page/drag_controller.cc:1142-1173.
//! DOM dragged flags, lifecycle preparation, native stacking/bounds selection,
//! selection bounds and bitmap rasterization remain with their original owners.
use super::{
    EditingPaintState, PaintArtifact, PaintContext, PaintEngine, PaintLifecycleObserver,
    PaintLifecyclePhase, PaintRect,
};
use paint_input_types::layout_input::{Offset, Size};
use paint_model::paint_flags::{PaintFlag, PaintFlags};
use paint_model::paint_property_tree::PropertyTreeState;
use paint_record::PaintRecording;
use std::{cell::RefCell, sync::Arc};

/// An independently recorded PaintRecord and its source raster coordinate
/// basis. This never aliases, edits or crops Page's ordinary drawing artifact.
pub struct DragImageRecording {
    pub artifact: Arc<PaintArtifact>,
    pub layout_size: Size,
    pub paint_offset: Offset,
    pub border_box_properties: PropertyTreeState,
}
impl PaintEngine {
    /// The native geometry owner has already selected EnclosingLayer's nearest
    /// stacking context, mapped the source bounds into its space and applied
    /// source's 8192 dimension cap. PrePaint supplies the same first fragment's
    /// PaintOffset and LocalBorderBoxProperties, which Layout does not own.
    pub fn RecordNodeDragImage(
        &mut self,
        layer_client_id: u64,
        bounding_box: PaintRect,
        editing: EditingPaintState,
        lifecycle: &dyn PaintLifecycleObserver,
    ) -> Option<DragImageRecording> {
        assert_ne!(layer_client_id, 0, "source PaintLayer identity is required");
        self.RecordDragImage(
            layer_client_id,
            bounding_box,
            editing,
            PaintFlag::kOmitCompositingInfo,
            lifecycle,
        )
    }
    /// Caller supplies FrameSelection::AbsoluteUnclippedBounds clipped by the
    /// visual viewport. No selection geometry is reconstructed by Paint.
    pub fn RecordSelectionDragImage(
        &mut self,
        painting_rect: PaintRect,
        editing: EditingPaintState,
        lifecycle: &dyn PaintLifecycleObserver,
    ) -> Option<DragImageRecording> {
        self.RecordDragImage(
            0,
            painting_rect,
            editing,
            PaintFlag::kSelectionDragImageOnly | PaintFlag::kOmitCompositingInfo,
            lifecycle,
        )
    }
    fn RecordDragImage(
        &mut self,
        layer_client_id: u64,
        rect: PaintRect,
        editing: EditingPaintState,
        flags: PaintFlags,
        lifecycle: &dyn PaintLifecycleObserver,
    ) -> Option<DragImageRecording> {
        let foreign_layers = &self.foreign_layers;
        let next_property_id = self.pre_paint.next_property_id;
        self.pre_paint.WithSpecialPaint(
            layer_client_id,
            rect,
            layer_client_id != 0,
            |layer, root, walk| {
                let _trace = browser_tracing::span("paint", "Paint.RecordDragImage");
                // LocalFrameView::PaintOutsideOfLifecycle, source.cc:4521-4530;
                // pinned kPaintTimingIngnoreOutOfLifecyclePaints is enabled.
                let _ignore_timing = (layer_client_id == 0).then(|| {
                    let scope = paint_record::ignore_paint_timing_scope::IgnorePaintTimingScope::new();
                    paint_record::ignore_paint_timing_scope::IgnorePaintTimingScope::IncrementIgnoreDepth();
                    scope
                });
                // The special paint scope hands out the overridden layer and
                // root occurrences' fragment cursors.
                let source = root;
                let layer_cursor = layer;
                let mut output = PaintRecording::default();
                output.resources = source.DocumentResources().cloned();
                let mut paint_context = PaintContext::WithEditing(&mut output, editing);
                paint_context.BindForeignLayers(foreign_layers);
                paint_context.SetDocumentElementIsInvisible(source.paint.style.opacity() == 0.0);
                paint_context.SetNextPropertyId(next_property_id);
                let context = RefCell::new(paint_context);
                lifecycle.AdvanceTo(PaintLifecyclePhase::InPaint);
                let frames: &dyn paint_walk::paint_layer_painter::LocalFrameRoots = walk;
                paint_walk::paint_layer_painter::PaintLayerPainter::new(
                    layer_cursor,
                    &context,
                    Some(frames),
                )
                .WithFlags(flags)
                .Paint();
                context.borrow_mut().Finish();
                drop(context);
                lifecycle.AdvanceTo(PaintLifecyclePhase::PaintClean);
                paint_commit::FinalizeArtifact(&mut output);
                // Node-image source adds first-fragment PaintOffset once. Selection
                // image paints LocalFrameView and uses painting_rect's own origin.
                let paint_offset = if layer_client_id == 0 {
                    Offset {
                        x: rect.x,
                        y: rect.y,
                    }
                } else {
                    // data_transfer.cc:131-134 adds FirstFragment().PaintOffset().
                    // The recorded ops are in the flat replay space, so read
                    // the FragmentData offset in that convention.
                    let layer_paint_offset = paint_record::paint_object::PaintObject::FromCursor(
                        layer_cursor,
                        paint_record::paint_object::PaintWalkContext::default(),
                    )
                    .FlatPaintOffset();
                    Offset {
                        x: rect.x + layer_paint_offset.x,
                        y: rect.y + layer_paint_offset.y,
                    }
                };
                // data_transfer.cc:136-137 FirstFragment().LocalBorderBoxProperties().
                let border_box_properties = layer_cursor
                    .PrePaintData::<prepaint::fragment_data::PrePaintFragmentData>()
                    .map_or_else(PropertyTreeState::default, |data| {
                        data.LocalBorderBoxProperties()
                    });
                DragImageRecording {
                    artifact: paint_commit::CommitArtifact(output),
                    layout_size: Size {
                        width: rect.width,
                        height: rect.height,
                    },
                    paint_offset,
                    border_box_properties,
                }
            },
        )
    }
}
