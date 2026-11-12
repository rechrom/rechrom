//! Committed blocking-wheel hit-test metadata at the Page boundary.
//!
//! This is a Rechrom adapter, not an alternate PrePaint or hit-test walk.
//! Chromium's EventHandlerRegistry invalidates the retained LayoutObject;
//! PrePaintTreeWalk::UpdateBlockingWheelEventHandler propagates the state on
//! that owner, and ObjectPainter::RecordHitTestData records the rectangles.
//! The source port below projects that committed data without materializing
//! another DOM, fragment tree, descendant set, or node-to-rect index.

use std::sync::Arc;

use interaction_scroll::core::frame::event_handler_registry::{
    EventHandlerClass, EventHandlerRegistry, EventListenerProperties, GetEventListenerProperties,
};

/// CSS client-space value; Page converts this value to its paint rectangle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WheelEventRegionRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Attachment of the committed rectangle, after native property-tree mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WheelEventRegionAttachment {
    /// Moves with the root scroll node. The compositor may sample this rect
    /// using its pending root-scroll displacement relative to this commit.
    RootScrolling,
    /// Does not move with the root scroll node. Fixed descendants retain this
    /// attachment; do not infer it from the listener target's own positioning.
    FixedToViewport,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WheelEventRegion {
    pub rect: WheelEventRegionRect,
    pub attachment: WheelEventRegionAttachment,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockingWheelEventRegions {
    /// Routes every viewport point to the blocking main-thread wheel path.
    /// Rectangle arrays are empty when this is true. This also preserves the
    /// conservative fallback when blocking listeners exist before a hit-test
    /// geometry commit is available.
    pub covers_viewport: bool,
    /// CSS client-space rectangles moving with the root scroll node.
    pub scrolling_rects: Arc<[WheelEventRegionRect]>,
    /// CSS client-space rectangles unaffected by pending root-scroll deltas.
    pub fixed_rects: Arc<[WheelEventRegionRect]>,
}

/// A borrowed projection of the canonical committed Paint/HitTest owner.
///
/// An empty iterator is valid committed geometry (for example, all listener
/// targets lack visible layout); it is different from unavailable geometry.
pub struct CommittedBlockingWheelEventRegions<'a> {
    /// The source owner resolves Window/document-wide coverage. No node-id
    /// sentinel or DOM descendant expansion is performed by this adapter.
    pub covers_viewport: bool,
    pub regions: Box<dyn Iterator<Item = WheelEventRegion> + 'a>,
}

/// Narrow, read-only projection supplied by the resident Paint/HitTest owner.
///
/// The embedding must connect registry target membership and
/// MarkBlockingWheelEventHandlerChanged to the owning DOM/Layout/PrePaint
/// lifecycle before committing this view. Ancestor propagation, hidden SVG
/// exclusion, visibility, transforms, clipping and fixed attachment belong to
/// those existing owners. Implementations borrow their committed hit-test
/// records; they must not run a replacement whole-DOM or whole-fragment walk.
pub trait BlockingWheelEventRegionSource {
    /// `None` means no coherent geometry commit is available. A pending
    /// listener invalidation must not expose an older commit as current.
    fn CommittedBlockingWheelEventRegions(&self) -> Option<CommittedBlockingWheelEventRegions<'_>>;
}

#[derive(Default)]
pub struct WheelEventRegionResolver {
    cached: Option<((u64, u64), BlockingWheelEventRegions)>,
}

impl WheelEventRegionResolver {
    /// Resolve once per geometry/listener revision pair.
    ///
    /// `frame_revision` identifies the exact client-space geometry commit,
    /// including viewport, transform, clipping and relevant scroll changes.
    /// `listener_revision` changes whenever blocking target membership changes,
    /// even if aggregate listener properties stay `kBlocking`. Listener identity
    /// and registration counts remain in EventHandlerRegistry, never here.
    pub fn ResolvePlatform(
        &mut self,
        frame_revision: u64,
        listener_revision: u64,
        registry: &EventHandlerRegistry,
        source: &impl BlockingWheelEventRegionSource,
    ) -> BlockingWheelEventRegions {
        let properties = GetEventListenerProperties(
            registry
                .EventHandlerTargets(EventHandlerClass::kWheelEventBlocking)
                .size()
                != 0,
            registry.HasEventHandlers(EventHandlerClass::kWheelEventPassive),
        );
        self.ResolveSelectedBlockingTargets(
            frame_revision,
            listener_revision,
            matches!(
                properties,
                EventListenerProperties::kBlocking | EventListenerProperties::kBlockingAndPassive
            ),
            source,
        )
    }

    /// Project already-selected blocking membership while the facade's native
    /// dispatch target adapters are being connected. The caller supplies the
    /// same distinct-target revision as EventHandlerRegistry's counted set;
    /// this entry never constructs replacement EventTarget/registry owners.
    pub fn ResolveSelectedBlockingTargets(
        &mut self,
        frame_revision: u64,
        membership_revision: u64,
        has_blocking_targets: bool,
        source: &impl BlockingWheelEventRegionSource,
    ) -> BlockingWheelEventRegions {
        let revision = (frame_revision, membership_revision);
        if let Some((cached_revision, regions)) = &self.cached {
            if *cached_revision == revision {
                return regions.clone();
            }
        }
        let regions = if has_blocking_targets {
            Self::ResolveCommitted(source)
        } else {
            BlockingWheelEventRegions::default()
        };
        self.cached = Some((revision, regions.clone()));
        regions
    }

    /// Drop the final-value cache on navigation, owner replacement or revision
    /// reset. This does not invalidate DOM/Layout/Paint: the registry effects
    /// must still reach their canonical owners synchronously.
    pub fn Invalidate(&mut self) {
        self.cached = None;
    }

    fn ResolveCommitted(source: &impl BlockingWheelEventRegionSource) -> BlockingWheelEventRegions {
        let Some(committed) = source.CommittedBlockingWheelEventRegions() else {
            return Self::ViewportCoverage();
        };
        if committed.covers_viewport {
            return Self::ViewportCoverage();
        }
        let mut scrolling_rects = Vec::new();
        let mut fixed_rects = Vec::new();
        for region in committed.regions {
            let rect = region.rect;
            if rect.width <= 0.0 || rect.height <= 0.0 {
                continue;
            }
            // A corrupt projection cannot silently remove a blocking region.
            if ![rect.x, rect.y, rect.width, rect.height]
                .iter()
                .all(|value| value.is_finite())
            {
                return Self::ViewportCoverage();
            }
            match region.attachment {
                WheelEventRegionAttachment::RootScrolling => scrolling_rects.push(rect),
                WheelEventRegionAttachment::FixedToViewport => fixed_rects.push(rect),
            }
        }
        BlockingWheelEventRegions {
            covers_viewport: false,
            scrolling_rects: scrolling_rects.into(),
            fixed_rects: fixed_rects.into(),
        }
    }

    fn ViewportCoverage() -> BlockingWheelEventRegions {
        BlockingWheelEventRegions {
            covers_viewport: true,
            ..BlockingWheelEventRegions::default()
        }
    }
}
