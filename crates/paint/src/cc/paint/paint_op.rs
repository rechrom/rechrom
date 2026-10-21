//! Selected-source PaintOp metadata methods. This module does not claim
//! serialization, command storage or complete paint_op.{h,cc} coverage.
#![allow(non_snake_case)]
use super::{
    paint_op_buffer::{MetadataOp, PaintOpBufferMetadata},
    paint_op_buffer_iterator::PaintOpType,
};
/// Borrowed actual flags. All methods project native resident values, not
/// summary booleans cached beside a Paint operation.
pub trait MetadataFlags<C> {
    fn IsValid(&self) -> bool;
    fn isAntiAlias(&self) -> bool;
    fn HasPathEffect(&self) -> bool;
    fn DashIntervalCount(&self) -> i32;
    fn StrokeCapIsRound(&self) -> bool;
    fn IsStroke(&self) -> bool;
    fn StrokeWidth(&self) -> f32;
    fn HasDiscardableImages(&self, usage: &mut C) -> bool;
}
pub trait PaintOpMetadataFields<C: Copy> {
    type Flags<'a>: MetadataFlags<C>
    where
        Self: 'a;
    fn GetType(&self) -> PaintOpType;
    fn Flags(&self) -> Option<Self::Flags<'_>>;
    fn IsValidGeometry(&self) -> bool;
    fn ClipAntialias(&self) -> bool;
    fn NestedBytesUsed(&self) -> usize;
    fn PathIsConvex(&self) -> bool;
    fn PathBounds(&self) -> [f32; 4];
    fn PathIsVolatile(&self) -> bool;
    fn NestedMetadata(&self) -> &PaintOpBufferMetadata<C>;
    fn IsDrawOp(&self) -> bool;
    /// The actual image's IsDiscardableImage projection (paint_op.cc:377-386).
    /// It updates usage only for an existing non-texture-backed image.
    fn IsDiscardableImage(&self, usage: &mut C) -> bool;
}
pub struct PaintOpMetadata<'a, V>(pub &'a V);

/// Native geometry/SkPaint primitives for the source paint-rect algorithm.
/// Bounds and flags are projected from the actual selected command. Returning
/// None from ComputeFastBounds means the native paint cannot compute them,
/// never permission to skip an unavailable image/path/filter capability.
pub trait PaintRectBackend {
    type Rect;
    type FloatRect: Copy;
    fn GetBounds(&self) -> Option<Self::FloatRect>;
    fn Clip(&self) -> Self::FloatRect;
    fn MapRect(&self, rect: Self::FloatRect) -> Self::FloatRect;
    fn HasFlags(&self) -> bool;
    fn IsFinite(&self, rect: Self::FloatRect) -> bool;
    fn ComputeFastBounds(&self, rect: Self::FloatRect) -> Option<Self::FloatRect>;
    fn Intersect(&self, a: &mut Self::FloatRect, b: Self::FloatRect) -> bool;
    fn ToEnclosingRect(&self, rect: Self::FloatRect) -> Self::Rect;
    fn Empty(&self) -> Self::Rect;
    fn OutsetOne(&self, rect: &mut Self::Rect);
}
// Source: cc/paint/paint_op.cc:2327-2368. The final one-pixel outset is part
// of native antialias-aware image queries, including the complete-clip case.
pub fn ComputePaintRect<B: PaintRectBackend>(backend: &B) -> B::Rect {
    let clip = backend.Clip();
    let mut transformed = if let Some(bounds) = backend.GetBounds() {
        let mut paint = backend.MapRect(bounds);
        if backend.HasFlags() {
            paint = if backend.IsFinite(paint) {
                backend.ComputeFastBounds(paint).unwrap_or(clip)
            } else {
                clip
            };
        }
        if !backend.Intersect(&mut paint, clip) {
            return backend.Empty();
        }
        backend.ToEnclosingRect(paint)
    } else {
        backend.ToEnclosingRect(clip)
    };
    backend.OutsetOne(&mut transformed);
    transformed
}
fn nested(kind: PaintOpType) -> bool {
    matches!(
        kind,
        PaintOpType::kDrawRecord | PaintOpType::kDrawScrollingContents
    )
}
impl<C: Copy + Ord, V: PaintOpMetadataFields<C>> MetadataOp<C> for PaintOpMetadata<'_, V> {
    fn IsValid(&self) -> bool {
        self.0.IsValidGeometry() && self.0.Flags().is_none_or(|flags| flags.IsValid())
    }
    // Source: cc/paint/paint_op.h:230-231,285-286.
    fn CountSlowPathsFromFlags(&self) -> i32 {
        self.0.Flags().is_some_and(|f| f.HasPathEffect()) as i32
    }
    // Source: cc/paint/paint_op.cc:2431-2468,2502-2504.
    fn CountSlowPaths(&self) -> i32 {
        use PaintOpType::*;
        match self.0.GetType() {
            kClipPath => (self.0.ClipAntialias() && !self.0.PathIsConvex()) as i32,
            kDrawLine => {
                if let Some(f) = self.0.Flags() {
                    if f.HasPathEffect() && !f.StrokeCapIsRound() && f.DashIntervalCount() == 2 {
                        return -1;
                    }
                }
                0
            }
            kDrawPath => {
                let f = self.0.Flags().expect("DrawPathOp source flags");
                if !f.isAntiAlias() || self.0.PathIsConvex() {
                    return 0;
                }
                let [left, top, right, bottom] = self.0.PathBounds();
                if f.IsStroke() && f.StrokeWidth() == 0. {
                    0
                } else if !f.IsStroke()
                    && right - left < 64.
                    && bottom - top < 64.
                    && !self.0.PathIsVolatile()
                {
                    0
                } else {
                    1
                }
            }
            kDrawRecord | kDrawScrollingContents => {
                self.0.NestedMetadata().num_slow_paths_up_to_min_for_MSAA()
            }
            _ => 0,
        }
    }
    // Source: cc/paint/paint_op.h:233,308,700,757,1139;
    // cc/paint/paint_op.cc:2472-2474,2506-2508.
    fn HasNonAAPaint(&self) -> bool {
        use PaintOpType::*;
        let kind = self.0.GetType();
        if nested(kind) {
            return self.0.NestedMetadata().has_non_aa_paint();
        }
        if matches!(kind, kDrawImage | kDrawImageRect | kSaveLayer) {
            return false;
        }
        self.0.Flags().is_some_and(|f| !f.isAntiAlias())
    }
    // Source: cc/paint/paint_op.h:255; cc/paint/paint_op.cc:2623-2625,2640-2642.
    fn AdditionalBytesUsed(&self) -> usize {
        if nested(self.0.GetType()) {
            self.0.NestedBytesUsed()
        } else {
            0
        }
    }
    // Source: cc/paint/paint_op.h:258; cc/paint/paint_op.cc:2627-2629,2644-2646.
    fn AdditionalOpCount(&self) -> usize {
        if nested(self.0.GetType()) {
            self.0.NestedMetadata().total_op_count()
        } else {
            0
        }
    }
    fn IsDrawOp(&self) -> bool {
        self.0.IsDrawOp()
    }
    // Source: cc/paint/paint_op.h:234,1024,1055; cc:2476-2478,2510-2512.
    fn HasDrawTextOps(&self) -> bool {
        let kind = self.0.GetType();
        if nested(kind) {
            self.0.NestedMetadata().has_draw_text_ops()
        } else {
            matches!(kind, PaintOpType::kDrawTextBlob | PaintOpType::kDrawSlug)
        }
    }
    // Source: cc/paint/paint_op.h:235,1143,1168,1201; cc:2480-2482,2514-2516.
    fn HasSaveLayerOps(&self) -> bool {
        let kind = self.0.GetType();
        if nested(kind) {
            self.0.NestedMetadata().has_save_layer_ops()
        } else {
            matches!(
                kind,
                PaintOpType::kSaveLayer
                    | PaintOpType::kSaveLayerAlpha
                    | PaintOpType::kSaveLayerFilters
            )
        }
    }
    // Source: cc/paint/paint_op.h:236,1169; cc:2484-2486,2518-2520.
    fn HasSaveLayerAlphaOps(&self) -> bool {
        let kind = self.0.GetType();
        if nested(kind) {
            self.0.NestedMetadata().has_save_layer_alpha_ops()
        } else {
            kind == PaintOpType::kSaveLayerAlpha
        }
    }
    // Source: cc/paint/paint_op.h:237-240,1140-1142; cc:2488-2490,2522-2525.
    fn HasEffectsPreventingLCDTextForSaveLayerAlpha(&self) -> bool {
        let kind = self.0.GetType();
        if nested(kind) {
            self.0
                .NestedMetadata()
                .has_effects_preventing_lcd_text_for_save_layer_alpha()
        } else {
            kind == PaintOpType::kSaveLayer
        }
    }
    // Source: cc/paint/paint_op.h:245-247; cc:2492-2500,2527-2536,2569-2572,2605-2608.
    fn HasDiscardableImages(&self, usage: &mut C) -> bool {
        let kind = self.0.GetType();
        if nested(kind) {
            let metadata = self.0.NestedMetadata();
            let has = metadata.has_discardable_images();
            if has {
                *usage = (*usage).max(metadata.content_color_usage());
            }
            has
        } else if matches!(kind, PaintOpType::kDrawImage | PaintOpType::kDrawImageRect) {
            self.0.IsDiscardableImage(usage)
        } else {
            false
        }
    }
    // Source: cc/paint/paint_op.h:248-251; cc/paint/paint_op.cc:2420-2423.
    fn HasDiscardableImagesFromFlags(&self, usage: &mut C) -> bool {
        self.0
            .Flags()
            .is_some_and(|f| f.HasDiscardableImages(usage))
    }
}
