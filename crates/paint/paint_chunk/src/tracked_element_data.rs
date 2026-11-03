use paint_model::PaintRect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintFrameToken {
    Local([u64; 2]),
    Remote([u64; 2]),
}

/// Source: `tracked_element_data.h:29-46`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintTrackedElementSubRectType {
    IntersectWithElementRect,
    NoIntersection,
}

/// Source: `tracked_element_data.h:29-46`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintTrackedElementRelativeRect {
    pub rect: PaintRect,
    pub r#type: PaintTrackedElementSubRectType,
}

/// Source: `tracked_element_data.h:28-103`.
#[derive(Clone, Debug, PartialEq)]
pub struct PaintTrackedElementSubRect {
    pub feature: i32,
    pub id: [u64; 2],
    pub should_add_to_compositor_frame_metadata: bool,
    pub should_exclude_fixed_and_sticky_occlusions: bool,
    pub sub_rect: Option<PaintTrackedElementRelativeRect>,
    pub frame_token: Option<PaintFrameToken>,
    pub parent_frame_token: Option<PaintFrameToken>,
}

impl PaintTrackedElementSubRect {
    /// Source: `tracked_element_data.cc:57-70`.
    pub fn GetEffectiveBounds(&self, element_paint_rect: PaintRect) -> PaintRect {
        let Some(sub_rect) = self.sub_rect else {
            return element_paint_rect;
        };
        let rect = PaintRect {
            x: element_paint_rect.x + sub_rect.rect.x,
            y: element_paint_rect.y + sub_rect.rect.y,
            width: sub_rect.rect.width,
            height: sub_rect.rect.height,
        };
        match sub_rect.r#type {
            PaintTrackedElementSubRectType::IntersectWithElementRect => {
                rect.intersection(element_paint_rect)
            }
            PaintTrackedElementSubRectType::NoIntersection => rect,
        }
    }
}

/// Source: `tracked_element_data.h:105-147`.
#[derive(Clone, Debug, PartialEq)]
pub struct PaintTrackedElementRect {
    pub feature: i32,
    pub rect: PaintRect,
    pub id: [u64; 2],
    pub should_add_to_compositor_frame_metadata: bool,
    pub should_exclude_fixed_and_sticky_occlusions: bool,
    pub frame_token: Option<PaintFrameToken>,
    pub parent_frame_token: Option<PaintFrameToken>,
}

impl PaintTrackedElementRect {
    /// Source: `tracked_element_data.cc:12-30`.
    pub fn ToString(&self) -> String {
        format!(
            "{{ id: {:?}, bounds: {:?}, should_exclude_fixed_and_sticky_occlusions: {}, frame_token: {:?}, parent_frame_token: {:?} }}",
            self.id,
            self.rect,
            self.should_exclude_fixed_and_sticky_occlusions,
            self.frame_token,
            self.parent_frame_token
        )
    }
}

pub type PaintTrackedElementRects = Vec<PaintTrackedElementRect>;

/// Source: `tracked_element_data.cc:32-55`.
pub fn TrackedElementRectsToString(data: &PaintTrackedElementRects) -> String {
    let rects = data
        .iter()
        .map(PaintTrackedElementRect::ToString)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{rects}}}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_bounds_follow_the_source_sub_rect_modes() {
        let element = PaintRect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        };
        let mut tracked = PaintTrackedElementSubRect {
            feature: 0,
            id: [1, 2],
            should_add_to_compositor_frame_metadata: false,
            should_exclude_fixed_and_sticky_occlusions: false,
            sub_rect: Some(PaintTrackedElementRelativeRect {
                rect: PaintRect {
                    x: 25.0,
                    y: 30.0,
                    width: 20.0,
                    height: 20.0,
                },
                r#type: PaintTrackedElementSubRectType::IntersectWithElementRect,
            }),
            frame_token: None,
            parent_frame_token: None,
        };
        assert_eq!(
            tracked.GetEffectiveBounds(element),
            PaintRect {
                x: 35.0,
                y: 50.0,
                width: 5.0,
                height: 10.0,
            }
        );
        tracked.sub_rect.as_mut().unwrap().r#type = PaintTrackedElementSubRectType::NoIntersection;
        assert_eq!(
            tracked.GetEffectiveBounds(element),
            PaintRect {
                x: 35.0,
                y: 50.0,
                width: 20.0,
                height: 20.0,
            }
        );
    }
}
