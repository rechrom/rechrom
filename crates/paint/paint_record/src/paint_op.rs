/// Concrete backend-neutral operation emitted by Paint recording.
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
    /// PaintCanvas custom-data placeholder. PaintArtifactCompositor replaces
    /// the DOM-node id with the nested canvas snapshot before raster.
    kCustomData,
}
