/// Concrete backend-neutral operation emitted by Paint recording.
// cpp: paint/paint_engine.h:33-68
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PaintOpType {
    kSave,
    kRestore,
    kConcat,
    kClipRect,
    kClipRoundedRect,
    kClipOutRoundedRect,
    kClipPath,
    kClipOutRect,
    kSaveLayer,
    kSaveLayerAlpha,
    kSaveLayerBlend,
    kSaveLayerFilter,
    /// Internal Mask effect: isolated source restored with SkBlendMode::kDstIn.
    kSaveLayerDstIn,
    kBeginMask,
    kEndMask,
    kDrawRect,
    /// Mask-image layers draw the mask source plane, not the masked contents.
    kDrawMask,
    kDrawRoundedRect,
    kDrawDoubleRoundedRect,
    kDrawEllipse,
    kStrokeEllipse,
    kDrawPath,
    kStrokePath,
    kDrawGradientRect,
    kDrawTiledGradient,
    kStrokeRect,
    kStrokeLine,
    kStrokeWavyLine,
    kDrawGlyphRun,
    kDrawImageRect,
    kDrawTiledImage,
    kDrawBoxShadow,
    kDrawScrollbarTrack,
    kDrawScrollbarThumb,
    kDrawScrollbarButton,
    kDrawScrollbarCorner,
}
