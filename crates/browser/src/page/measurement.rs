//! A measurement belongs to one immutable fragment snapshot. Its first geometry
//! demand indexes all node rectangles once, after the ordinary style/layout
//! flush; subsequent demands preserve fragmented-rect ordering by cloning the
//! stored value. No hit-test, paint clip or composited result is cached here.
use layoutng_assembly::fragment_tree::FragmentNode;
use paint::paint_engine::PaintRect;
use std::{cell::OnceCell, collections::HashMap, ops::Deref, rc::Rc};

pub(super) struct MeasurementSnapshot {
    fragments: Rc<FragmentNode>,
    geometry_revision: u64,
    paint_current: bool,
    rects: OnceCell<HashMap<u64, Vec<PaintRect>>>,
    #[cfg(test)]
    index_builds: std::cell::Cell<usize>,
}

/// Geometry-only state moved aside while Paint updates native client flags.
/// Holding it retains no fragment Rc, so it cannot force a whole-tree COW.
pub(super) struct MeasurementGeometryCache {
    geometry_revision: u64,
    rects: OnceCell<HashMap<u64, Vec<PaintRect>>>,
    #[cfg(test)]
    index_builds: std::cell::Cell<usize>,
}
impl MeasurementSnapshot {
    #[cfg(test)]
    pub(super) fn new(fragments: Rc<FragmentNode>) -> Self {
        Self::with_revision(fragments, 0)
    }
    pub(super) fn with_revision(fragments: Rc<FragmentNode>, geometry_revision: u64) -> Self {
        Self {
            fragments,
            geometry_revision,
            paint_current: true,
            rects: OnceCell::new(),
            #[cfg(test)]
            index_builds: Default::default(),
        }
    }
    pub(super) fn GeometryRevision(&self) -> u64 {
        self.geometry_revision
    }
    pub(super) fn PaintCurrent(&self) -> bool {
        self.paint_current
    }
    pub(super) fn MarkPaintStale(&mut self) {
        self.paint_current = false;
    }
    pub(super) fn Fragments(&self) -> Rc<FragmentNode> {
        self.fragments.clone()
    }
    /// The caller admits only a paint-current snapshot. CommitPaint changes
    /// client lifecycle bits, so these exact geometry results remain valid.
    pub(super) fn IntoPaintParts(self) -> (Rc<FragmentNode>, MeasurementGeometryCache) {
        debug_assert!(self.paint_current);
        (
            self.fragments,
            MeasurementGeometryCache {
                geometry_revision: self.geometry_revision,
                rects: self.rects,
                #[cfg(test)]
                index_builds: self.index_builds,
            },
        )
    }
    pub(super) fn FromPaintParts(
        fragments: Rc<FragmentNode>,
        cache: MeasurementGeometryCache,
    ) -> Self {
        Self {
            fragments,
            geometry_revision: cache.geometry_revision,
            paint_current: true,
            rects: cache.rects,
            #[cfg(test)]
            index_builds: cache.index_builds,
        }
    }
    pub(super) fn ClientRects(&self, node_id: u64) -> Vec<PaintRect> {
        self.rects
            .get_or_init(|| {
                #[cfg(test)]
                self.index_builds.set(self.index_builds.get() + 1);
                paint::paint_engine::FragmentClientRectsByNode(&self.fragments)
            })
            .get(&node_id)
            .cloned()
            .unwrap_or_default()
    }
}
// Existing metric reads consume the exact same fragment snapshot and retain
// their CSS zoom/rounding behavior; geometry indexing is lazy and independent.
impl Deref for MeasurementSnapshot {
    type Target = FragmentNode;
    fn deref(&self) -> &FragmentNode {
        &self.fragments
    }
}

#[cfg(test)]
#[test]
fn paint_commit_transfers_measurement_without_cloning_and_preserves_external_snapshots() {
    let mut fragment = FragmentNode {
        node_id: 1,
        size: layoutng_assembly::internal::layout_input::Size {
            width: 32.0,
            height: 20.0,
        },
        ..Default::default()
    };
    fragment.paint.display_item_client_is_just_created = true;
    let snapshot = MeasurementSnapshot::new(Rc::new(fragment));
    let expected = snapshot.ClientRects(1);
    let original = Rc::as_ptr(&snapshot.fragments);
    let (mut fragments, cache) = snapshot.IntoPaintParts();
    assert_eq!(Rc::strong_count(&fragments), 1);
    Rc::make_mut(&mut fragments)
        .paint
        .display_item_client_is_just_created = false;
    assert_eq!(
        Rc::as_ptr(&fragments),
        original,
        "the internal measurement holder cannot cause COW"
    );
    let snapshot = MeasurementSnapshot::FromPaintParts(fragments, cache);
    assert_eq!(snapshot.ClientRects(1), expected);
    assert_eq!(
        snapshot.index_builds.get(),
        1,
        "client validation preserves the existing geometry index"
    );
    let external = snapshot.Fragments();
    let (mut fragments, cache) = snapshot.IntoPaintParts();
    Rc::make_mut(&mut fragments)
        .paint
        .display_item_client_is_just_created = true;
    assert!(!Rc::ptr_eq(&external, &fragments));
    assert!(
        !external.paint.display_item_client_is_just_created,
        "a genuinely held old snapshot stays immutable"
    );
    let snapshot = MeasurementSnapshot::FromPaintParts(fragments, cache);
    assert_eq!(snapshot.ClientRects(1), expected);
    assert_eq!(snapshot.index_builds.get(), 1);
}

#[cfg(test)]
#[path = "measurement_tests.rs"]
mod tests;
