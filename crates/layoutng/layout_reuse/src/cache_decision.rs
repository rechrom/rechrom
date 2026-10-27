use crate::LayoutCacheStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutCacheDecisionReason {
    IncompatibleSpace,
    IntrinsicSizeChanged,
    EqualSizes,
    StableSize,
    GeometryCheck,
}

/// Native geometry adapter for the size-based layout-cache policy. The core
/// assembly supplies the coupled calculations; this package owns their order
/// and all early-hit/early-miss decisions.
pub trait LayoutCacheDecisionBackend {
    fn may_skip_layout(&self) -> bool;
    fn inline_size_constraints_equal(&self) -> bool;
    fn block_size_constraints_equal(&self) -> bool;
    fn intrinsic_size_will_change(&mut self) -> bool;
    fn sizes_equal(&self) -> bool;
    fn size_may_change(&self) -> bool;
    fn calculate_with_geometry(&mut self) -> LayoutCacheStatus;
}

#[inline(never)]
pub fn calculate_size_based_layout_cache_status<B: LayoutCacheDecisionBackend>(
    backend: &mut B,
) -> LayoutCacheStatus {
    if !backend.may_skip_layout() {
        crate::trace::record_cache_decision(LayoutCacheDecisionReason::IncompatibleSpace);
        return LayoutCacheStatus::kNeedsLayout;
    }
    if backend.inline_size_constraints_equal() && backend.block_size_constraints_equal() {
        if backend.intrinsic_size_will_change() {
            crate::trace::record_cache_decision(LayoutCacheDecisionReason::IntrinsicSizeChanged);
            return LayoutCacheStatus::kNeedsLayout;
        }
        if backend.sizes_equal() {
            crate::trace::record_cache_decision(LayoutCacheDecisionReason::EqualSizes);
            return LayoutCacheStatus::kHit;
        }
        if !backend.size_may_change() {
            crate::trace::record_cache_decision(LayoutCacheDecisionReason::StableSize);
            return LayoutCacheStatus::kHit;
        }
    }
    crate::trace::record_cache_decision(LayoutCacheDecisionReason::GeometryCheck);
    backend.calculate_with_geometry()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Decision {
        may_skip: bool,
        inline_equal: bool,
        block_equal: bool,
        intrinsic_changes: bool,
        sizes_equal: bool,
        size_may_change: bool,
        geometry_status: LayoutCacheStatus,
        geometry_calls: usize,
    }

    impl Default for Decision {
        fn default() -> Self {
            Self {
                may_skip: true,
                inline_equal: true,
                block_equal: true,
                intrinsic_changes: false,
                sizes_equal: true,
                size_may_change: false,
                geometry_status: LayoutCacheStatus::kNeedsSimplifiedLayout,
                geometry_calls: 0,
            }
        }
    }

    impl LayoutCacheDecisionBackend for Decision {
        fn may_skip_layout(&self) -> bool {
            self.may_skip
        }

        fn inline_size_constraints_equal(&self) -> bool {
            self.inline_equal
        }

        fn block_size_constraints_equal(&self) -> bool {
            self.block_equal
        }

        fn intrinsic_size_will_change(&mut self) -> bool {
            self.intrinsic_changes
        }

        fn sizes_equal(&self) -> bool {
            self.sizes_equal
        }

        fn size_may_change(&self) -> bool {
            self.size_may_change
        }

        fn calculate_with_geometry(&mut self) -> LayoutCacheStatus {
            self.geometry_calls += 1;
            self.geometry_status
        }
    }

    #[test]
    fn incompatible_space_is_an_immediate_miss() {
        let mut decision = Decision {
            may_skip: false,
            ..Decision::default()
        };
        assert_eq!(
            calculate_size_based_layout_cache_status(&mut decision),
            LayoutCacheStatus::kNeedsLayout
        );
        assert_eq!(decision.geometry_calls, 0);
    }

    #[test]
    fn equal_constraints_and_sizes_are_an_immediate_hit() {
        let mut decision = Decision::default();
        assert_eq!(
            calculate_size_based_layout_cache_status(&mut decision),
            LayoutCacheStatus::kHit
        );
        assert_eq!(decision.geometry_calls, 0);
    }

    #[test]
    fn intrinsic_size_change_is_an_immediate_miss() {
        let mut decision = Decision {
            intrinsic_changes: true,
            ..Decision::default()
        };
        assert_eq!(
            calculate_size_based_layout_cache_status(&mut decision),
            LayoutCacheStatus::kNeedsLayout
        );
        assert_eq!(decision.geometry_calls, 0);
    }

    #[test]
    fn stable_size_hits_even_when_available_sizes_differ() {
        let mut decision = Decision {
            sizes_equal: false,
            size_may_change: false,
            ..Decision::default()
        };
        assert_eq!(
            calculate_size_based_layout_cache_status(&mut decision),
            LayoutCacheStatus::kHit
        );
        assert_eq!(decision.geometry_calls, 0);
    }

    #[test]
    fn uncertain_size_delegates_to_native_geometry() {
        let mut decision = Decision {
            sizes_equal: false,
            size_may_change: true,
            geometry_status: LayoutCacheStatus::kCanReuseLines,
            ..Decision::default()
        };
        assert_eq!(
            calculate_size_based_layout_cache_status(&mut decision),
            LayoutCacheStatus::kCanReuseLines
        );
        assert_eq!(decision.geometry_calls, 1);
    }
}
