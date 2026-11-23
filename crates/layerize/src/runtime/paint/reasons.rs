//! Adapter for Chromium's native `CompositingReasons` value domain.

use crate::adapter::paint_artifact_compositor::{CompositingReason, CompositingReasons};

#[derive(Clone, Default, Debug)]
pub struct CpuCompositingReasons {
    source: foundation::CompositingReasons,
}

impl CpuCompositingReasons {
    pub fn from_source(reasons: &foundation::CompositingReasons) -> Self {
        Self { source: *reasons }
    }

    pub fn descriptions(&self) -> Vec<&'static str> {
        foundation::Descriptions(self.source)
    }

    pub fn short_names(&self) -> Vec<&'static str> {
        foundation::ShortNames(self.source)
    }
}

impl CompositingReasons for CpuCompositingReasons {
    fn empty(&self) -> bool {
        self.source.IsEmpty()
    }

    fn Put(&mut self, reason: CompositingReason) {
        let reason = match reason {
            CompositingReason::OverflowScrolling => {
                foundation::CompositingReason::kOverflowScrolling
            }
            CompositingReason::FixedAttachmentBackground => {
                foundation::CompositingReason::kFixedAttachmentBackground
            }
            CompositingReason::Caret => foundation::CompositingReason::kCaret,
            CompositingReason::Scrollbar => foundation::CompositingReason::kScrollbar,
            CompositingReason::Canvas => foundation::CompositingReason::kCanvas,
            CompositingReason::DevToolsOverlay => foundation::CompositingReason::kDevToolsOverlay,
            CompositingReason::Plugin => foundation::CompositingReason::kPlugin,
            CompositingReason::Video => foundation::CompositingReason::kVideo,
            CompositingReason::IFrame => foundation::CompositingReason::kIFrame,
            CompositingReason::LinkHighlight => foundation::CompositingReason::kLinkHighlight,
            CompositingReason::Viewport => foundation::CompositingReason::kViewport,
            CompositingReason::ViewTransitionContent => {
                foundation::CompositingReason::kViewTransitionContent
            }
            CompositingReason::BackfaceVisibilityHidden => {
                foundation::CompositingReason::kBackfaceVisibilityHidden
            }
            CompositingReason::Overlap => foundation::CompositingReason::kOverlap,
        };
        self.source.Put(reason);
    }

    fn PutAll(&mut self, other: Self) {
        self.source.PutAll(other.source);
    }
}
