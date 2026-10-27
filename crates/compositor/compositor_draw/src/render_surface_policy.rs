use compositor_tree::geometry::Rect;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderSurfacePropertyChangedFlags {
    pub self_changed: bool,
    pub ancestor_changed: bool,
}

impl RenderSurfacePropertyChangedFlags {
    pub const fn new(self_changed: bool, ancestor_changed: bool) -> Self {
        Self {
            self_changed,
            ancestor_changed,
        }
    }

    pub const fn self_changed(&self) -> bool {
        self.self_changed
    }

    pub const fn ancestor_changed(&self) -> bool {
        self.ancestor_changed
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContributingLayerDisposition {
    Ignore,
    Defer,
    Accumulate,
}

pub const fn ClassifyContributingLayer(
    target_is_self: bool,
    has_view_transition_resource: bool,
    resource_matches_token: bool,
) -> ContributingLayerDisposition {
    if target_is_self {
        return ContributingLayerDisposition::Ignore;
    }
    if has_view_transition_resource {
        if resource_matches_token {
            ContributingLayerDisposition::Ignore
        } else {
            ContributingLayerDisposition::Defer
        }
    } else {
        ContributingLayerDisposition::Accumulate
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContributingSurfaceDisposition {
    Ignore,
    AccumulateNormalOnly,
    AccumulateNormalAndCapture,
}

pub const fn ClassifyContributingSurface(
    target_is_self: bool,
    is_view_transition_element: bool,
    resource_matches_token: bool,
) -> ContributingSurfaceDisposition {
    if target_is_self || (is_view_transition_element && !resource_matches_token) {
        ContributingSurfaceDisposition::Ignore
    } else if is_view_transition_element {
        ContributingSurfaceDisposition::AccumulateNormalOnly
    } else {
        ContributingSurfaceDisposition::AccumulateNormalAndCapture
    }
}

pub fn ClampSurfaceContentRects(
    mut content: Rect,
    mut capture: Rect,
    max_texture_size: i32,
) -> (Rect, Rect, bool) {
    let exceeded = content.width > max_texture_size || content.height > max_texture_size;
    content.set_width(content.width.min(max_texture_size));
    content.set_height(content.height.min(max_texture_size));
    capture.set_width(capture.width.min(max_texture_size));
    capture.set_height(capture.height.min(max_texture_size));
    (content, capture, exceeded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_layers_are_deferred_until_their_token_matches() {
        assert_eq!(
            ClassifyContributingLayer(false, true, false),
            ContributingLayerDisposition::Defer
        );
        assert_eq!(
            ClassifyContributingLayer(false, false, false),
            ContributingLayerDisposition::Accumulate
        );
    }

    #[test]
    fn clamps_normal_and_capture_rects() {
        let (content, capture, exceeded) =
            ClampSurfaceContentRects(Rect::new(1, 2, 200, 50), Rect::new(3, 4, 30, 300), 100);
        assert!(exceeded);
        assert_eq!(content.width, 100);
        assert_eq!(content.height, 50);
        assert_eq!(capture.width, 30);
        assert_eq!(capture.height, 100);
    }
}
