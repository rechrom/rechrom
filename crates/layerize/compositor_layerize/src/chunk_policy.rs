use crate::pending_layer_policy::CompositingType;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChunkCompositingInput {
    pub scroll_hit_test_requires_compositing: bool,
    pub has_single_display_item: bool,
    pub single_item_is_foreign_layer: bool,
    pub single_item_is_scrollbar: bool,
    pub scrollbar_has_scroll_translation: bool,
    pub raster_inducing_scroll_enabled: bool,
    pub scrollbar_scroll_requires_compositing: bool,
}

pub fn ChooseChunkCompositingType(input: ChunkCompositingInput) -> CompositingType {
    if input.scroll_hit_test_requires_compositing {
        return CompositingType::kScrollHitTestLayer;
    }
    if !input.has_single_display_item {
        return CompositingType::kOther;
    }
    if input.single_item_is_foreign_layer {
        return CompositingType::kForeignLayer;
    }
    if input.single_item_is_scrollbar
        && input.scrollbar_has_scroll_translation
        && (input.raster_inducing_scroll_enabled || input.scrollbar_scroll_requires_compositing)
    {
        return CompositingType::kScrollbarLayer;
    }
    CompositingType::kOther
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_hit_test_has_precedence() {
        let input = ChunkCompositingInput {
            scroll_hit_test_requires_compositing: true,
            has_single_display_item: true,
            single_item_is_foreign_layer: true,
            ..Default::default()
        };
        assert_eq!(
            ChooseChunkCompositingType(input),
            CompositingType::kScrollHitTestLayer
        );
    }

    #[test]
    fn scrollbar_needs_a_scroll_reason() {
        let base = ChunkCompositingInput {
            has_single_display_item: true,
            single_item_is_scrollbar: true,
            scrollbar_has_scroll_translation: true,
            ..Default::default()
        };
        assert_eq!(ChooseChunkCompositingType(base), CompositingType::kOther);
        assert_eq!(
            ChooseChunkCompositingType(ChunkCompositingInput {
                raster_inducing_scroll_enabled: true,
                ..base
            }),
            CompositingType::kScrollbarLayer
        );
    }
}
