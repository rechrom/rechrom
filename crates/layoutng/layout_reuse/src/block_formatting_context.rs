use foundation::{LayoutUnit, MarginStrut};

/// Algorithm-neutral facts needed to decide whether a cached result can be
/// repositioned inside the same block formatting context.
///
/// The native adapter extracts these facts from ConstraintSpace and the
/// physical fragment. Keeping the decision tree here makes both sampling
/// profiles and rejection telemetry resolve to the reuse package.
#[derive(Clone, Copy, Debug)]
pub struct BlockFormattingContextReuseInput {
    pub cached_result_bfc_block_offset: Option<LayoutUnit>,
    pub current_bfc_block_offset: Option<LayoutUnit>,
    pub is_pushed_by_floats: bool,
    pub subtree_modified_margin_strut: bool,
    pub may_have_descendant_above_block_start: bool,
    pub is_self_collapsing: bool,
    pub has_adjoining_object_descendants: bool,
    pub old_has_floats: bool,
    pub new_has_floats: bool,
    pub old_margin_strut: MarginStrut,
    pub new_margin_strut: MarginStrut,
    pub old_clearance_offset: LayoutUnit,
    pub new_clearance_offset: LayoutUnit,
    pub old_bfc_block_offset: LayoutUnit,
    pub new_bfc_block_offset: LayoutUnit,
    pub old_expected_bfc_block_offset: LayoutUnit,
    pub new_expected_bfc_block_offset: LayoutUnit,
    pub old_forced_bfc_block_offset: Option<LayoutUnit>,
    pub new_forced_bfc_block_offset: Option<LayoutUnit>,
    pub ancestor_has_clearance_past_adjoining_floats: bool,
    pub old_exclusion_clearance: LayoutUnit,
    pub new_exclusion_clearance: LayoutUnit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockFormattingContextReuseAdjustment {
    pub bfc_block_offset: Option<LayoutUnit>,
    pub block_offset_delta: LayoutUnit,
    /// `None` means that the caller must preserve the cached end margin strut.
    pub end_margin_strut: Option<MarginStrut>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockFormattingContextReuseReason {
    PushedFloatMarginChanged,
    PushedFloatOffsetChanged,
    PushedFloatClearanceRegressed,
    ModifiedMarginStrutChanged,
    DescendantAboveBlockStartWithFloats,
    SelfCollapsingPushedByFloats,
    SelfCollapsingClearance,
    ForcedOffsetChanged,
    OldExclusionClearance,
    NewExclusionClearance,
    Reusable,
}

fn reject(
    reason: BlockFormattingContextReuseReason,
) -> Option<BlockFormattingContextReuseAdjustment> {
    crate::trace::record_bfc_reuse_decision(reason);
    None
}

/// Owns the complete ordering of BFC cache-reuse checks and offset updates.
/// This is deliberately out of line so sampling profilers attribute the hot
/// decision path to `layout_reuse` rather than its native LayoutNG adapter.
#[inline(never)]
pub fn calculate_block_formatting_context_reuse(
    input: &BlockFormattingContextReuseInput,
) -> Option<BlockFormattingContextReuseAdjustment> {
    let is_margin_strut_equal = input.old_margin_strut == input.new_margin_strut;

    if input.is_pushed_by_floats {
        debug_assert!(input.old_has_floats);
        if !is_margin_strut_equal {
            return reject(BlockFormattingContextReuseReason::PushedFloatMarginChanged);
        }
        if input.cached_result_bfc_block_offset != Some(input.old_clearance_offset) {
            return reject(BlockFormattingContextReuseReason::PushedFloatOffsetChanged);
        }
        if input.old_clearance_offset - input.old_bfc_block_offset
            > input.new_clearance_offset - input.new_bfc_block_offset
        {
            return reject(BlockFormattingContextReuseReason::PushedFloatClearanceRegressed);
        }
    }

    if input.subtree_modified_margin_strut && !is_margin_strut_equal {
        return reject(BlockFormattingContextReuseReason::ModifiedMarginStrutChanged);
    }
    if input.may_have_descendant_above_block_start && (input.old_has_floats || input.new_has_floats)
    {
        return reject(BlockFormattingContextReuseReason::DescendantAboveBlockStartWithFloats);
    }

    if input.is_self_collapsing {
        if input.is_pushed_by_floats {
            return reject(BlockFormattingContextReuseReason::SelfCollapsingPushedByFloats);
        }
        if input.has_adjoining_object_descendants
            && (input.old_expected_bfc_block_offset < input.old_exclusion_clearance
                || input.new_expected_bfc_block_offset < input.new_exclusion_clearance)
        {
            return reject(BlockFormattingContextReuseReason::SelfCollapsingClearance);
        }
        let adjustment = BlockFormattingContextReuseAdjustment {
            bfc_block_offset: input.new_forced_bfc_block_offset,
            block_offset_delta: input.new_expected_bfc_block_offset
                - input.old_expected_bfc_block_offset,
            end_margin_strut: (!input.subtree_modified_margin_strut)
                .then_some(input.new_margin_strut),
        };
        crate::trace::record_bfc_reuse_decision(BlockFormattingContextReuseReason::Reusable);
        return Some(adjustment);
    }

    let mut bfc_block_offset = input
        .current_bfc_block_offset
        .expect("regular block has BFC offset");
    if input.ancestor_has_clearance_past_adjoining_floats {
        debug_assert!(input.old_forced_bfc_block_offset.is_some());
        debug_assert!(input.new_forced_bfc_block_offset.is_some());
        debug_assert_eq!(
            input.old_forced_bfc_block_offset,
            Some(input.old_clearance_offset)
        );
        debug_assert_eq!(
            input.new_forced_bfc_block_offset,
            Some(input.new_clearance_offset)
        );
    } else if input.old_forced_bfc_block_offset != input.new_forced_bfc_block_offset {
        return reject(BlockFormattingContextReuseReason::ForcedOffsetChanged);
    }
    if bfc_block_offset < input.old_exclusion_clearance {
        return reject(BlockFormattingContextReuseReason::OldExclusionClearance);
    }

    let block_offset_delta =
        if input.is_pushed_by_floats || input.ancestor_has_clearance_past_adjoining_floats {
            debug_assert_eq!(bfc_block_offset, input.old_clearance_offset);
            bfc_block_offset = input.new_clearance_offset;
            input.new_clearance_offset - input.old_clearance_offset
        } else if is_margin_strut_equal {
            let delta = input.new_bfc_block_offset - input.old_bfc_block_offset;
            bfc_block_offset += delta;
            delta
        } else {
            #[cfg(debug_assertions)]
            {
                debug_assert!(!input.subtree_modified_margin_strut);
                debug_assert_eq!(
                    bfc_block_offset,
                    input.old_bfc_block_offset + input.old_margin_strut.Sum()
                );
            }
            let new_offset = input.new_bfc_block_offset + input.new_margin_strut.Sum();
            let delta = new_offset - bfc_block_offset;
            bfc_block_offset += delta;
            delta
        };

    if bfc_block_offset < input.new_exclusion_clearance {
        return reject(BlockFormattingContextReuseReason::NewExclusionClearance);
    }

    crate::trace::record_bfc_reuse_decision(BlockFormattingContextReuseReason::Reusable);
    Some(BlockFormattingContextReuseAdjustment {
        bfc_block_offset: Some(bfc_block_offset),
        block_offset_delta,
        end_margin_strut: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(value: i32) -> LayoutUnit {
        LayoutUnit::from_signed(value)
    }

    fn input() -> BlockFormattingContextReuseInput {
        BlockFormattingContextReuseInput {
            cached_result_bfc_block_offset: Some(unit(10)),
            current_bfc_block_offset: Some(unit(10)),
            is_pushed_by_floats: false,
            subtree_modified_margin_strut: false,
            may_have_descendant_above_block_start: false,
            is_self_collapsing: false,
            has_adjoining_object_descendants: false,
            old_has_floats: false,
            new_has_floats: false,
            old_margin_strut: MarginStrut::default(),
            new_margin_strut: MarginStrut::default(),
            old_clearance_offset: unit(10),
            new_clearance_offset: unit(14),
            old_bfc_block_offset: unit(10),
            new_bfc_block_offset: unit(14),
            old_expected_bfc_block_offset: unit(10),
            new_expected_bfc_block_offset: unit(14),
            old_forced_bfc_block_offset: None,
            new_forced_bfc_block_offset: None,
            ancestor_has_clearance_past_adjoining_floats: false,
            old_exclusion_clearance: LayoutUnit::new(),
            new_exclusion_clearance: LayoutUnit::new(),
        }
    }

    #[test]
    fn regular_block_repositions_for_new_bfc_offset() {
        let adjustment = calculate_block_formatting_context_reuse(&input()).unwrap();
        assert_eq!(adjustment.bfc_block_offset, Some(unit(14)));
        assert_eq!(adjustment.block_offset_delta, unit(4));
        assert_eq!(adjustment.end_margin_strut, None);
    }

    #[test]
    fn changed_modified_margin_strut_rejects_reuse() {
        let mut facts = input();
        facts.subtree_modified_margin_strut = true;
        facts.new_margin_strut.positive_margin = unit(1);
        assert_eq!(calculate_block_formatting_context_reuse(&facts), None);
    }

    #[test]
    fn self_collapsing_block_updates_forced_offset_and_end_margin() {
        let mut facts = input();
        facts.is_self_collapsing = true;
        facts.new_forced_bfc_block_offset = Some(unit(20));
        facts.new_margin_strut.positive_margin = unit(3);
        let adjustment = calculate_block_formatting_context_reuse(&facts).unwrap();
        assert_eq!(adjustment.bfc_block_offset, Some(unit(20)));
        assert_eq!(adjustment.block_offset_delta, unit(4));
        assert_eq!(adjustment.end_margin_strut, Some(facts.new_margin_strut));
    }

    #[test]
    fn new_float_clearance_rejects_adjusted_block() {
        let mut facts = input();
        facts.new_exclusion_clearance = unit(15);
        assert_eq!(calculate_block_formatting_context_reuse(&facts), None);
    }

    #[test]
    fn rejection_families_are_counted_in_the_pass_summary() {
        let session = crate::LayoutReuseTraceSession::begin(true);

        let mut float_context = input();
        float_context.is_pushed_by_floats = true;
        float_context.old_has_floats = true;
        float_context.new_margin_strut.positive_margin = unit(1);
        assert_eq!(
            calculate_block_formatting_context_reuse(&float_context),
            None
        );

        let mut margin = input();
        margin.subtree_modified_margin_strut = true;
        margin.new_margin_strut.positive_margin = unit(1);
        assert_eq!(calculate_block_formatting_context_reuse(&margin), None);

        let mut descendant = input();
        descendant.may_have_descendant_above_block_start = true;
        descendant.new_has_floats = true;
        assert_eq!(calculate_block_formatting_context_reuse(&descendant), None);

        let mut self_collapse = input();
        self_collapse.is_self_collapsing = true;
        self_collapse.has_adjoining_object_descendants = true;
        self_collapse.new_exclusion_clearance = unit(15);
        assert_eq!(
            calculate_block_formatting_context_reuse(&self_collapse),
            None
        );

        let mut forced_offset = input();
        forced_offset.new_forced_bfc_block_offset = Some(unit(14));
        assert_eq!(
            calculate_block_formatting_context_reuse(&forced_offset),
            None
        );

        let mut exclusion = input();
        exclusion.new_exclusion_clearance = unit(15);
        assert_eq!(calculate_block_formatting_context_reuse(&exclusion), None);

        let stats = session.finish();
        assert_eq!(stats.bfc_float_context_rejections, 1);
        assert_eq!(stats.bfc_margin_rejections, 1);
        assert_eq!(stats.bfc_descendant_rejections, 1);
        assert_eq!(stats.bfc_self_collapsing_clearance_rejections, 1);
        assert_eq!(stats.bfc_forced_offset_rejections, 1);
        assert_eq!(stats.bfc_exclusion_clearance_rejections, 1);
        assert_eq!(stats.bfc_hits, 0);
    }
}
