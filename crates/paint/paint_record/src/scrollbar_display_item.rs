//! Source-shaped `ScrollbarDisplayItem` behavior.
//!
//! The backend-neutral payload remains in `paint_record`; cc layer construction
//! consumes it through the existing compositor package boundary.

use std::sync::Arc;

use foundation::{
    CompositorElementId, CompositorElementIdNamespace, NamespaceFromCompositorElementId,
};
use layoutng_assembly::internal::layout_input_types::Color;
use paint_chunk::PaintHitTestOpaqueness;
use paint_model::{paint_property_tree::TransformPaintPropertyNode, PaintRect};

use crate::{RecordedDisplayItem, RecordedDisplayItemKind};

// Source: scrollbar_display_item.h:93-106. The cc::Scrollbar adapter is
// represented by its immutable paint inputs; lazy records and cc layers stay
// in compositor, preserving the paint_record -> compositor package direction.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollbarDisplayItemData {
    pub element_id: CompositorElementId,
    pub track_rect: PaintRect,
    pub thumb_rect: PaintRect,
    pub thumb_color: Color,
    pub horizontal: bool,
    pub overlay: bool,
    pub left_side_vertical: bool,
    pub minimum_thumb_length: i32,
    pub needs_update_display: bool,
    pub hit_test_opaqueness: PaintHitTestOpaqueness,
}

impl RecordedDisplayItem {
    fn ScrollbarData(&self) -> &ScrollbarDisplayItemData {
        assert!(!self.IsTombstone());
        assert_eq!(self.kind, RecordedDisplayItemKind::Scrollbar);
        self.scrollbar
            .as_ref()
            .expect("ScrollbarDisplayItem retains its source payload")
    }

    // Source: scrollbar_display_item.h:45-52.
    pub fn ScrollTranslation(&self) -> Option<&TransformPaintPropertyNode> {
        assert_eq!(self.kind, RecordedDisplayItemKind::Scrollbar);
        self.scroll_translation.as_deref()
    }

    pub fn ElementId(&self) -> CompositorElementId {
        self.ScrollbarData().element_id
    }

    // Source: scrollbar_display_item.cc:73-75.
    pub fn NeedsUpdateDisplay(&self) -> bool {
        self.ScrollbarData().needs_update_display
    }

    // Source: scrollbar_display_item.cc:110-114.
    pub fn IsOpaque(&self) -> bool {
        let data = self.ScrollbarData();
        !data.overlay && data.thumb_color.alpha >= 1.0
    }

    // Source: scrollbar_display_item.cc:116-125. The source compares retained
    // property-node pointer identity, not structural node values.
    pub fn EqualsScrollbarForUnderInvalidation(&self, other: &Self) -> bool {
        if self.kind != RecordedDisplayItemKind::Scrollbar
            || other.kind != RecordedDisplayItemKind::Scrollbar
            || self.IsTombstone()
            || other.IsTombstone()
        {
            return false;
        }
        let same_scroll_translation = match (&self.scroll_translation, &other.scroll_translation) {
            (Some(first), Some(second)) => Arc::ptr_eq(first, second),
            (None, None) => true,
            _ => false,
        };
        same_scroll_translation && self.ElementId() == other.ElementId()
    }

    // Source: scrollbar_display_item.h:79-84.
    pub fn IsScrollbarElementId(element_id: CompositorElementId) -> bool {
        matches!(
            NamespaceFromCompositorElementId(element_id),
            CompositorElementIdNamespace::kHorizontalScrollbar
                | CompositorElementIdNamespace::kVerticalScrollbar
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::CompositorElementIdFromUniqueObjectId;
    use paint_model::display_item_id::{DisplayItemId, DisplayItemIdType};
    use paint_model::paint_property_tree::{
        PaintBackfaceVisibility, PaintPropertyNodeLifecycle, TransformCompositorState,
    };

    fn transform() -> Arc<TransformPaintPropertyNode> {
        Arc::new(TransformPaintPropertyNode {
            compositor: TransformCompositorState::default(),
            geometry_mapper_cache: Default::default(),
            lifecycle: PaintPropertyNodeLifecycle::default(),
            id: 1,
            parent: None,
            matrix: Default::default(),
            origin: [0.0; 3],
            flattens_inherited_transform: false,
            rendering_context_id: 0,
            backface_visibility: PaintBackfaceVisibility::Inherited,
            scroll: None,
            scroll_parent_scroll_translation: None,
            sticky_constraint: None,
            direct_compositing_reasons: Default::default(),
        })
    }

    fn item(
        namespace: CompositorElementIdNamespace,
        scroll_translation: Option<Arc<TransformPaintPropertyNode>>,
    ) -> RecordedDisplayItem {
        let data = ScrollbarDisplayItemData {
            element_id: CompositorElementIdFromUniqueObjectId(9, namespace),
            track_rect: Default::default(),
            thumb_rect: Default::default(),
            thumb_color: Color {
                red: 0.1,
                green: 0.2,
                blue: 0.3,
                alpha: 1.0,
            },
            horizontal: namespace == CompositorElementIdNamespace::kHorizontalScrollbar,
            overlay: false,
            left_side_vertical: false,
            minimum_thumb_length: 4,
            needs_update_display: true,
            hit_test_opaqueness: PaintHitTestOpaqueness::Opaque,
        };
        RecordedDisplayItem {
            is_cacheable: true,
            is_tombstone: false,
            paint_invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            scrollbar: Some(data),
            foreign_layer: None,
            kind: RecordedDisplayItemKind::Scrollbar,
            id: DisplayItemId {
                client_id: 8,
                r#type: if namespace == CompositorElementIdNamespace::kHorizontalScrollbar {
                    DisplayItemIdType::kScrollbarHorizontal
                } else {
                    DisplayItemIdType::kScrollbarVertical
                },
                fragment: 0,
            },
            visual_rect: Default::default(),
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: crate::RasterEffectOutset::kNone,
            record_rect_known_to_be_opaque: Default::default(),
            record_background: Default::default(),
            record_has_text: false,
            record_begin: 0,
            record_end: 0,
            scroll_translation,
        }
    }

    #[test]
    fn accessors_and_namespace_follow_the_source_subtype() {
        let item = item(CompositorElementIdNamespace::kHorizontalScrollbar, None);
        assert!(RecordedDisplayItem::IsScrollbarElementId(item.ElementId()));
        assert!(item.NeedsUpdateDisplay());
        assert!(item.IsOpaque());
        assert!(item.ScrollTranslation().is_none());
        assert!(!RecordedDisplayItem::IsScrollbarElementId(
            CompositorElementIdFromUniqueObjectId(9, CompositorElementIdNamespace::kPrimary)
        ));
    }

    #[test]
    fn under_invalidation_comparison_uses_node_identity_and_element_id() {
        let translation = transform();
        let first = item(
            CompositorElementIdNamespace::kVerticalScrollbar,
            Some(Arc::clone(&translation)),
        );
        let same = item(
            CompositorElementIdNamespace::kVerticalScrollbar,
            Some(translation),
        );
        let different_node = item(
            CompositorElementIdNamespace::kVerticalScrollbar,
            Some(transform()),
        );
        assert!(first.EqualsScrollbarForUnderInvalidation(&same));
        assert!(!first.EqualsScrollbarForUnderInvalidation(&different_node));
    }
}
