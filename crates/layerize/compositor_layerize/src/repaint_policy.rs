#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepaintLayerizationDecision {
    ReuseLayerization,
    ChunkIdentityChanged,
    BoundsChanged,
    ForeignLayerChanged,
    OpaqueRectChanged,
    OpaqueTextStateChanged,
    BackgroundTransparencyChanged,
    TextPresenceChanged,
    DrawsContentChanged,
    SolidColorStateChanged,
    HitTestOpaquenessChanged,
    RawPropertiesChanged,
}

impl RepaintLayerizationDecision {
    pub const fn NeedsFullUpdate(self) -> bool {
        !matches!(self, Self::ReuseLayerization)
    }

    pub const fn RawPropertiesDiffer(self) -> bool {
        matches!(self, Self::RawPropertiesChanged)
    }
}

pub trait RepaintChunkComparisonSource {
    fn ChunksMatch(&self) -> bool;
    fn MovedFromCachedSubsequence(&self) -> bool;
    fn BoundsEqual(&self) -> bool;
    fn ForeignLayerEqual(&self) -> bool;
    fn OpaqueRectEqual(&self) -> bool;
    fn OpaqueTextStateEqual(&self) -> bool;
    fn BackgroundTransparencyEqual(&self) -> bool;
    fn TextPresenceEqual(&self) -> bool;
    fn DrawsContentEqual(&self) -> bool;
    fn SolidColorStateEqual(&self) -> bool;
    fn HitTestOpaquenessEqual(&self) -> bool;
    fn RawPropertiesEqual(&self) -> bool;
}

pub fn ClassifyRepaintLayerization(
    source: &dyn RepaintChunkComparisonSource,
) -> RepaintLayerizationDecision {
    use RepaintLayerizationDecision::*;
    if !source.ChunksMatch() {
        return ChunkIdentityChanged;
    }
    if source.MovedFromCachedSubsequence() {
        return if source.RawPropertiesEqual() {
            ReuseLayerization
        } else {
            RawPropertiesChanged
        };
    }
    if !source.BoundsEqual() {
        return BoundsChanged;
    }
    if !source.ForeignLayerEqual() {
        return ForeignLayerChanged;
    }
    if !source.OpaqueRectEqual() {
        return OpaqueRectChanged;
    }
    if !source.OpaqueTextStateEqual() {
        return OpaqueTextStateChanged;
    }
    if !source.BackgroundTransparencyEqual() {
        return BackgroundTransparencyChanged;
    }
    if !source.TextPresenceEqual() {
        return TextPresenceChanged;
    }
    if !source.DrawsContentEqual() {
        return DrawsContentChanged;
    }
    if !source.SolidColorStateEqual() {
        return SolidColorStateChanged;
    }
    if !source.HitTestOpaquenessEqual() {
        return HitTestOpaquenessChanged;
    }
    if !source.RawPropertiesEqual() {
        return RawPropertiesChanged;
    }
    ReuseLayerization
}
