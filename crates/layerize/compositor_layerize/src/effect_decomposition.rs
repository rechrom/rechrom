#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectDecompositionRejection {
    LayerEffectMismatch,
    LayerRequiresOwnBacking,
    DirectCompositingReason,
    CannotUpcast,
    TooManyPreviousSiblings,
    FirstSiblingDrawsContent,
    PreviousSiblingCannotMerge,
}

pub fn CheckEffectDecompositionBase(
    layer_matches_effect: bool,
    layer_requires_own_backing: bool,
    effect_has_direct_compositing_reason: bool,
) -> Result<(), EffectDecompositionRejection> {
    if !layer_matches_effect {
        return Err(EffectDecompositionRejection::LayerEffectMismatch);
    }
    if layer_requires_own_backing {
        return Err(EffectDecompositionRejection::LayerRequiresOwnBacking);
    }
    if effect_has_direct_compositing_reason {
        return Err(EffectDecompositionRejection::DirectCompositingReason);
    }
    Ok(())
}

pub fn CheckBlendModeDecomposition(
    has_non_source_over_blend: bool,
    previous_sibling_count: usize,
    first_sibling_draws_content: bool,
    previous_sibling_draws_content: bool,
    previous_sibling_can_merge: bool,
) -> Result<(), EffectDecompositionRejection> {
    if !has_non_source_over_blend || previous_sibling_count == 0 {
        return Ok(());
    }
    if previous_sibling_count > 2 {
        return Err(EffectDecompositionRejection::TooManyPreviousSiblings);
    }
    if previous_sibling_count == 2 && first_sibling_draws_content {
        return Err(EffectDecompositionRejection::FirstSiblingDrawsContent);
    }
    if previous_sibling_draws_content && !previous_sibling_can_merge {
        return Err(EffectDecompositionRejection::PreviousSiblingCannotMerge);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blend_mode_limits_previous_siblings() {
        assert_eq!(
            CheckBlendModeDecomposition(true, 3, false, false, true),
            Err(EffectDecompositionRejection::TooManyPreviousSiblings)
        );
        assert_eq!(
            CheckBlendModeDecomposition(true, 1, false, true, false),
            Err(EffectDecompositionRejection::PreviousSiblingCannotMerge)
        );
        assert!(CheckBlendModeDecomposition(false, 8, true, true, false).is_ok());
    }
}
