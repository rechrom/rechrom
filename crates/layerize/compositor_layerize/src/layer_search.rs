//! Traversal policies used while turning Paint effect groups into pending
//! compositor layers.

pub struct MergeCandidateIndices {
    next_exclusive: usize,
    first_in_group: usize,
    remaining: usize,
}

impl MergeCandidateIndices {
    pub fn new(new_layer_index: usize, first_in_group: usize, distance_limit: usize) -> Self {
        Self {
            next_exclusive: new_layer_index,
            first_in_group,
            remaining: distance_limit,
        }
    }
}

impl Iterator for MergeCandidateIndices {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 || self.next_exclusive <= self.first_in_group {
            return None;
        }
        self.next_exclusive -= 1;
        self.remaining -= 1;
        Some(self.next_exclusive)
    }
}

pub trait MergeCandidateContext {
    fn TryMergeCandidate(&mut self, candidate_index: usize) -> bool;
    fn MightOverlapCandidate(&self, candidate_index: usize) -> bool;
    fn MarkNewLayerAsOverlap(&mut self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MergeSearchStats {
    pub attempts: usize,
    pub merged: bool,
    pub overlap_barrier: bool,
    pub max_search_depth: usize,
}

/// Searches nearest-first for a merge target. A failed merge that overlaps is
/// a hard barrier: layers behind it cannot be considered without reordering.
pub fn SearchMergeCandidate<C: MergeCandidateContext>(
    context: &mut C,
    new_layer_index: usize,
    first_in_group: usize,
    distance_limit: usize,
) -> MergeSearchStats {
    let mut trace = browser_tracing::span(
        "compositor.layerize",
        "Compositor.Layerize.SearchMergeCandidate",
    );
    let mut stats = MergeSearchStats::default();
    for candidate_index in
        MergeCandidateIndices::new(new_layer_index, first_in_group, distance_limit)
    {
        stats.attempts += 1;
        stats.max_search_depth = stats
            .max_search_depth
            .max(new_layer_index - candidate_index);
        if context.TryMergeCandidate(candidate_index) {
            stats.merged = true;
            break;
        }
        if context.MightOverlapCandidate(candidate_index) {
            context.MarkNewLayerAsOverlap();
            stats.overlap_barrier = true;
            break;
        }
    }
    trace.set("attempts", stats.attempts as f64);
    trace.set("merged", usize::from(stats.merged) as f64);
    trace.set("overlap_barrier", usize::from(stats.overlap_barrier) as f64);
    trace.set("max_search_depth", stats.max_search_depth as f64);
    stats
}

pub fn StrictChildAlongPath<N: Clone + PartialEq>(
    ancestor: &N,
    node: &N,
    mut parent: impl FnMut(&N) -> Option<N>,
) -> Option<N> {
    let mut current = Some(node.clone());
    while let Some(value) = current {
        let next = parent(&value);
        if next.as_ref() == Some(ancestor) {
            return Some(value);
        }
        current = next;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_are_nearest_first_and_bounded() {
        assert_eq!(
            MergeCandidateIndices::new(8, 2, 3).collect::<Vec<_>>(),
            vec![7, 6, 5]
        );
        assert_eq!(
            MergeCandidateIndices::new(4, 2, 9).collect::<Vec<_>>(),
            vec![3, 2]
        );
    }

    struct SearchContext {
        merge_at: Option<usize>,
        overlap_at: Option<usize>,
        visited: Vec<usize>,
        marked_overlap: bool,
    }

    impl MergeCandidateContext for SearchContext {
        fn TryMergeCandidate(&mut self, candidate_index: usize) -> bool {
            self.visited.push(candidate_index);
            self.merge_at == Some(candidate_index)
        }
        fn MightOverlapCandidate(&self, candidate_index: usize) -> bool {
            self.overlap_at == Some(candidate_index)
        }
        fn MarkNewLayerAsOverlap(&mut self) {
            self.marked_overlap = true;
        }
    }

    #[test]
    fn overlap_stops_search_before_a_more_distant_merge() {
        let mut context = SearchContext {
            merge_at: Some(3),
            overlap_at: Some(5),
            visited: Vec::new(),
            marked_overlap: false,
        };
        let stats = SearchMergeCandidate(&mut context, 6, 1, 8);
        assert_eq!(context.visited, vec![5]);
        assert!(context.marked_overlap);
        assert!(!stats.merged);
        assert!(stats.overlap_barrier);
        assert_eq!(stats.attempts, 1);
    }

    #[test]
    fn nearest_success_owns_depth_and_attempt_counts() {
        let mut context = SearchContext {
            merge_at: Some(3),
            overlap_at: None,
            visited: Vec::new(),
            marked_overlap: false,
        };
        let stats = SearchMergeCandidate(&mut context, 6, 1, 8);
        assert_eq!(context.visited, vec![5, 4, 3]);
        assert!(stats.merged);
        assert_eq!(stats.attempts, 3);
        assert_eq!(stats.max_search_depth, 3);
    }
}
