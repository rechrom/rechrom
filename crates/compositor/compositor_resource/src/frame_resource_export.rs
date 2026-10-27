//! Resource-reference collection before compositor-frame export.

pub trait FrameResourcePasses {
    type ResourceId: Copy + Default + Eq;

    fn pass_count(&self) -> usize;
    fn quad_count(&self, pass_index: usize) -> usize;
    fn quad_resource_id(&self, pass_index: usize, quad_index: usize) -> Self::ResourceId;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameResourceScanStats {
    pub passes: usize,
    pub quads: usize,
    pub resource_references: usize,
}

/// Collects resource references in pass/quad order. Duplicates are preserved:
/// the provider owns export deduplication and synchronization semantics.
pub fn CollectFrameResourceIds<P: FrameResourcePasses>(
    passes: &P,
) -> (Vec<P::ResourceId>, FrameResourceScanStats) {
    let mut trace = browser_tracing::span(
        "compositor.resource",
        "Compositor.Resource.CollectFrameResources",
    );
    let mut resources = Vec::new();
    let mut stats = FrameResourceScanStats {
        passes: passes.pass_count(),
        ..Default::default()
    };
    for pass_index in 0..passes.pass_count() {
        let quad_count = passes.quad_count(pass_index);
        stats.quads += quad_count;
        for quad_index in 0..quad_count {
            let resource = passes.quad_resource_id(pass_index, quad_index);
            if resource != P::ResourceId::default() {
                resources.push(resource);
            }
        }
    }
    stats.resource_references = resources.len();
    trace.set("passes", stats.passes as f64);
    trace.set("quads", stats.quads as f64);
    trace.set("resource_references", stats.resource_references as f64);
    (resources, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Passes(Vec<Vec<u32>>);

    impl FrameResourcePasses for Passes {
        type ResourceId = u32;
        fn pass_count(&self) -> usize {
            self.0.len()
        }
        fn quad_count(&self, pass_index: usize) -> usize {
            self.0[pass_index].len()
        }
        fn quad_resource_id(&self, pass_index: usize, quad_index: usize) -> Self::ResourceId {
            self.0[pass_index][quad_index]
        }
    }

    #[test]
    fn preserves_reference_order_and_duplicates_but_omits_invalid_ids() {
        let (resources, stats) = CollectFrameResourceIds(&Passes(vec![vec![0, 7, 7], vec![3, 0]]));
        assert_eq!(resources, vec![7, 7, 3]);
        assert_eq!(stats.passes, 2);
        assert_eq!(stats.quads, 5);
        assert_eq!(stats.resource_references, 3);
    }
}
