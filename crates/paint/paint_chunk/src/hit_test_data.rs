use std::sync::Arc;

use paint_model::paint_property_tree::TransformPaintPropertyNode;
use paint_model::PaintRect;

/// Source: `hit_test_data.h:18-52`.
#[derive(Clone, Debug)]
pub struct PaintHitTestData {
    pub touch_action_rects: Vec<(PaintRect, u32)>,
    pub wheel_event_rects: Vec<PaintRect>,
    pub xr_regions: Vec<foundation::CompositorElementId>,
    pub scroll_hit_test_rect: PaintRect,
    pub scroll_translation: Option<Arc<TransformPaintPropertyNode>>,
    pub scrolling_contents_cull_rect: PaintRect,
}

// Source: hit_test_data.h:38-48. The pinned Chromium revision keeps
// ScrollingContentsCullRectOnScrollNode enabled, so the legacy resident cull
// rectangle is deliberately excluded. Member equality is pointer identity.
impl PartialEq for PaintHitTestData {
    fn eq(&self, other: &Self) -> bool {
        let same_scroll_translation = match (&self.scroll_translation, &other.scroll_translation) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        self.touch_action_rects == other.touch_action_rects
            && self.wheel_event_rects == other.wheel_event_rects
            && self.xr_regions == other.xr_regions
            && self.scroll_hit_test_rect == other.scroll_hit_test_rect
            && same_scroll_translation
    }
}

impl Default for PaintHitTestData {
    fn default() -> Self {
        let infinite = foundation::InfiniteIntRect();
        Self {
            touch_action_rects: Vec::new(),
            wheel_event_rects: Vec::new(),
            xr_regions: Vec::new(),
            scroll_hit_test_rect: PaintRect::default(),
            scroll_translation: None,
            scrolling_contents_cull_rect: PaintRect {
                x: infinite.x() as f64,
                y: infinite.y() as f64,
                width: infinite.width() as f64,
                height: infinite.height() as f64,
            },
        }
    }
}

impl PaintHitTestData {
    /// Source: `hit_test_data.cc:48-100`.
    pub fn ToString(&self) -> String {
        let mut fields = Vec::new();
        if !self.touch_action_rects.is_empty() {
            fields.push(format!("touch_action_rects: {:?}", self.touch_action_rects));
        }
        if !self.wheel_event_rects.is_empty() {
            fields.push(format!("wheel_event_rects: {:?}", self.wheel_event_rects));
        }
        if !self.xr_regions.is_empty() {
            fields.push(format!("xr_regions: {:?}", self.xr_regions));
        }
        if !self.scroll_hit_test_rect.is_empty() {
            fields.push(format!(
                "scroll_hit_test_rect: {:?}",
                self.scroll_hit_test_rect
            ));
        }
        if let Some(scroll_translation) = self.scroll_translation.as_deref() {
            fields.push(format!("scroll_translation: {:?}", scroll_translation));
        }
        format!("{{{}}}", fields.join(", "))
    }
}

// Source: `hit_test_data.cc:102-108`.
impl std::fmt::Display for PaintHitTestData {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.ToString())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_scroll_node_cull_rect_is_not_resident_hit_test_identity() {
        let mut left = PaintHitTestData::default();
        let mut right = left.clone();
        left.scrolling_contents_cull_rect = PaintRect {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        };
        right.scrolling_contents_cull_rect = PaintRect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        };
        assert_eq!(left, right);
        assert!(!left.ToString().contains("scrolling_contents_cull_rect"));
    }
}
