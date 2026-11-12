//! Page values projected from selected blocking targets and committed Paint regions.
//!
//! The caller selects blocking target membership. PrePaint propagates blocking
//! handler state on the resident LayoutObjects, and ObjectPainter records the
//! geometry. This boundary only maps their completed values for the host.

use crate::adapter::wheel_event_regions::{
    self as source, BlockingWheelEventRegionSource, CommittedBlockingWheelEventRegions,
    WheelEventRegion, WheelEventRegionAttachment, WheelEventRegionRect,
};
use dom::{persistent_document::DOMNodeType, Document};
use paint::paint_engine::{PaintArtifact, PaintRect};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockingWheelEventRegions {
    pub covers_viewport: bool,
    pub scrolling_rects: Vec<PaintRect>,
    pub fixed_rects: Vec<PaintRect>,
}

#[derive(Default)]
pub struct WheelEventRegionResolver {
    source: source::WheelEventRegionResolver,
    blocking_targets: Vec<u64>,
    local_frame_targets: Vec<(u64, Vec<u64>)>,
    membership_revision: u64,
}

/// The source Window/document target classification and canonical Paint commit
/// are supplied together; no fragment or descendant index is reconstructed.
struct CommittedRegionSource<'a> {
    covers_viewport: bool,
    artifact: Option<&'a PaintArtifact>,
}

impl BlockingWheelEventRegionSource for CommittedRegionSource<'_> {
    fn CommittedBlockingWheelEventRegions(&self) -> Option<CommittedBlockingWheelEventRegions<'_>> {
        Some(CommittedBlockingWheelEventRegions {
            covers_viewport: self.covers_viewport,
            regions: Box::new(self.artifact?.CommittedBlockingWheelEventRegions()?.map(
                |(rect, root_scrolling)| WheelEventRegion {
                    rect: WheelEventRegionRect {
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: rect.height,
                    },
                    attachment: if root_scrolling {
                        WheelEventRegionAttachment::RootScrolling
                    } else {
                        WheelEventRegionAttachment::FixedToViewport
                    },
                },
            )),
        })
    }
}

impl WheelEventRegionResolver {
    pub fn ResolvePlatform(
        &mut self,
        frame_sequence: u64,
        listener_targets: Vec<u64>,
        document: &Document,
        artifact: Option<&PaintArtifact>,
    ) -> BlockingWheelEventRegions {
        self.ResolvePlatformWithLocalFrameTargets(
            frame_sequence,
            listener_targets,
            Vec::new(),
            document,
            artifact,
        )
    }
    pub fn ResolvePlatformWithLocalFrameTargets(
        &mut self,
        frame_sequence: u64,
        mut listener_targets: Vec<u64>,
        mut local_frame_targets: Vec<(u64, Vec<u64>)>,
        document: &Document,
        artifact: Option<&PaintArtifact>,
    ) -> BlockingWheelEventRegions {
        // The caller has already selected blocking registrations. Counted-set
        // multiplicity cannot change geometry (EventHandlerRegistry.cc:109-141).
        listener_targets.sort_unstable();
        listener_targets.dedup();
        for (_, targets) in &mut local_frame_targets {
            targets.sort_unstable();
            targets.dedup();
        }
        local_frame_targets.retain(|(_, targets)| !targets.is_empty());
        local_frame_targets.sort_unstable_by_key(|(frame, _)| *frame);
        if self.blocking_targets != listener_targets
            || self.local_frame_targets != local_frame_targets
        {
            self.blocking_targets = listener_targets;
            self.local_frame_targets = local_frame_targets;
            self.membership_revision = self
                .membership_revision
                .checked_add(1)
                .expect("blocking wheel membership revision exhausted");
        }
        // Window is the existing host target sentinel. Document targets have
        // global coverage; all element geometry is already in the Paint record.
        let covers_viewport = self.blocking_targets.iter().any(|&id| {
            id == 0
                || document
                    .FindNodeById(id)
                    .is_some_and(|index| document.Node(index).Type() == DOMNodeType::kDocument)
        });
        let regions = self.source.ResolveSelectedBlockingTargets(
            frame_sequence,
            self.membership_revision,
            !self.blocking_targets.is_empty() || !self.local_frame_targets.is_empty(),
            &CommittedRegionSource {
                covers_viewport,
                artifact,
            },
        );
        fn rect(rect: &WheelEventRegionRect) -> PaintRect {
            PaintRect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            }
        }
        BlockingWheelEventRegions {
            covers_viewport: regions.covers_viewport,
            scrolling_rects: regions.scrolling_rects.iter().map(rect).collect(),
            fixed_rects: regions.fixed_rects.iter().map(rect).collect(),
        }
    }

    pub fn Invalidate(&mut self) {
        self.source.Invalidate();
    }
}
