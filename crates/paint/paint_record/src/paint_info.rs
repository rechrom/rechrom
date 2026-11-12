#![allow(non_snake_case)]

use std::cell::RefCell;

use crate::display_item_id::DisplayItemIdType;
use crate::paint_context::PaintContext;
use crate::paint_engine::PaintPhase;
use layoutng_assembly::internal::layout_input::Offset;
use paint_model::paint_flags::{PaintFlag, PaintFlags};
use paint_model::paint_property_tree::TransformPaintPropertyNode;
use paint_model::{CullRect, PaintRect};

// Source: core/paint/paint_info.h:75-90, 170-183, 201-206.
#[derive(Clone, Copy)]
pub struct PaintInfo<'c, 'o> {
    pub context: &'c RefCell<PaintContext<'o>>,
    pub phase: PaintPhase,
    pub descendant_painting_blocked: bool,
    cull_rect: CullRect,
    paint_flags: PaintFlags,
    is_painting_background_in_contents_space: bool,
    skips_background: bool,
    skips_gap_decorations: bool,
    svg_context_paints: Option<&'c crate::svg_context_paints::SvgContextPaints<'c>>,
    svg_resource_root: u64,
    pub svg_resource_painter: Option<
        for<'n, 'f, 'p, 'r> fn(
            &'n prepaint::pre_paint_tree_walk::PaintTreeNode<'f>,
            &PaintInfo<'p, 'r>,
        ),
    >,
}

impl<'c, 'o> PaintInfo<'c, 'o> {
    pub fn new(context: &'c RefCell<PaintContext<'o>>, phase: PaintPhase) -> Self {
        Self::new_with_cull_and_flags(
            context,
            CullRect::Infinite(),
            phase,
            false,
            PaintFlag::kNoFlag,
        )
    }

    pub fn new_with_cull(
        context: &'c RefCell<PaintContext<'o>>,
        cull_rect: CullRect,
        phase: PaintPhase,
    ) -> Self {
        Self::new_with_cull_and_flags(context, cull_rect, phase, false, PaintFlag::kNoFlag)
    }

    pub fn new_with_flags(
        context: &'c RefCell<PaintContext<'o>>,
        phase: PaintPhase,
        descendant_painting_blocked: bool,
        paint_flags: PaintFlags,
    ) -> Self {
        Self::new_with_cull_and_flags(
            context,
            CullRect::Infinite(),
            phase,
            descendant_painting_blocked,
            paint_flags,
        )
    }

    pub fn new_with_cull_and_flags(
        context: &'c RefCell<PaintContext<'o>>,
        cull_rect: CullRect,
        phase: PaintPhase,
        descendant_painting_blocked: bool,
        paint_flags: PaintFlags,
    ) -> Self {
        Self {
            context,
            phase,
            descendant_painting_blocked,
            cull_rect,
            paint_flags,
            is_painting_background_in_contents_space: false,
            skips_background: false,
            skips_gap_decorations: false,
            svg_context_paints: None,
            svg_resource_root: 0,
            svg_resource_painter: None,
        }
    }

    pub fn GetSvgContextPaints(
        &self,
    ) -> Option<&'c crate::svg_context_paints::SvgContextPaints<'c>> {
        self.svg_context_paints
    }
    pub fn WithSvgContextPaints<'s>(
        &self,
        paints: &'s crate::svg_context_paints::SvgContextPaints<'s>,
    ) -> PaintInfo<'s, 'o>
    where
        'c: 's,
    {
        let mut info: PaintInfo<'s, 'o> = *self;
        info.svg_context_paints = Some(paints);
        info
    }
    pub fn SetSvgResourceRoot(&mut self, client: u64) {
        self.svg_resource_root = client;
        self.paint_flags |= PaintFlag::kOmitCompositingInfo | PaintFlag::kPaintingResourceSubtree;
        self.cull_rect = CullRect::Infinite();
    }
    pub fn IsSvgResourceRoot(&self, client: u64) -> bool {
        self.svg_resource_root == client && client != 0
    }

    // Source: core/paint/paint_info.h:92-108.
    pub fn ForDescendants(&self) -> Self {
        let mut result = *self;
        debug_assert!(!result.is_painting_background_in_contents_space);
        if result.phase == PaintPhase::kDescendantOutlinesOnly {
            result.phase = PaintPhase::kOutline;
        } else if result.phase == PaintPhase::kDescendantBlockBackgroundsOnly {
            result.phase = PaintPhase::kBlockBackground;
        }
        result
    }

    pub fn ShouldOmitCompositingInfo(&self) -> bool {
        self.paint_flags & PaintFlag::kOmitCompositingInfo != 0
    }

    pub fn IsRenderingClipPathAsMaskImage(&self) -> bool {
        self.paint_flags & PaintFlag::kPaintingClipPathAsMask != 0
    }

    pub fn IsRenderingResourceSubtree(&self) -> bool {
        self.paint_flags & PaintFlag::kPaintingResourceSubtree != 0
    }

    pub fn IsPrivacyPreserving(&self) -> bool {
        self.paint_flags & PaintFlag::kPrivacyPreserving != 0
    }

    pub fn ShouldSkipBackground(&self) -> bool {
        self.skips_background
    }

    pub fn SetSkipsBackground(&mut self, value: bool) {
        self.skips_background = value;
    }

    pub fn ShouldSkipGapDecorations(&self) -> bool {
        self.skips_gap_decorations
    }

    pub fn SetSkipsGapDecorations(&mut self, value: bool) {
        self.skips_gap_decorations = value;
    }

    pub fn ShouldAddUrlMetadata(&self) -> bool {
        self.paint_flags & PaintFlag::kAddUrlMetadata != 0
    }

    pub fn DisplayItemTypeForClipping(&self) -> DisplayItemIdType {
        DisplayItemIdType::PaintPhaseToClipType(self.phase)
    }

    pub fn GetPaintFlags(&self) -> PaintFlags {
        self.paint_flags
    }

    pub fn GetCullRect(&self) -> CullRect {
        self.cull_rect
    }

    pub fn SetCullRect(&mut self, cull_rect: CullRect) {
        self.cull_rect = cull_rect;
    }

    pub fn IntersectsCullRect(&self, rect: PaintRect, offset: Offset) -> bool {
        self.cull_rect.Intersects(PaintRect {
            x: rect.x + offset.x,
            y: rect.y + offset.y,
            ..rect
        })
    }

    pub fn ApplyInfiniteCullRect(&mut self) {
        self.cull_rect = CullRect::Infinite();
    }

    pub fn TransformCullRect(&mut self, transform: &TransformPaintPropertyNode) {
        self.cull_rect.ApplyTransform(&transform.matrix);
    }

    pub fn IsPaintingBackgroundInContentsSpace(&self) -> bool {
        self.is_painting_background_in_contents_space
    }

    pub fn SetIsPaintingBackgroundInContentsSpace(&mut self, value: bool) {
        self.is_painting_background_in_contents_space = value;
    }

    pub fn DescendantPaintingBlocked(&self) -> bool {
        self.descendant_painting_blocked
    }

    pub fn SetDescendantPaintingBlocked(&mut self) {
        self.descendant_painting_blocked = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint_context::PaintContext;
    use crate::PaintRecording;

    #[test]
    fn cull_rect_mutation_and_intersection_follow_paint_info_contract() {
        let mut output = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        let mut info = PaintInfo::new_with_cull(
            &context,
            CullRect::FromRect(PaintRect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            }),
            PaintPhase::kForeground,
        );
        assert!(info.IntersectsCullRect(
            PaintRect {
                width: 2.0,
                height: 2.0,
                ..Default::default()
            },
            Offset { x: 10.0, y: 20.0 }
        ));
        assert!(!info.IntersectsCullRect(
            PaintRect {
                x: 100.0,
                y: 100.0,
                width: 2.0,
                height: 2.0,
            },
            Offset::default()
        ));
        info.ApplyInfiniteCullRect();
        assert!(info.IntersectsCullRect(
            PaintRect {
                x: 100.0,
                y: 100.0,
                width: 2.0,
                height: 2.0,
            },
            Offset::default()
        ));
    }
}
