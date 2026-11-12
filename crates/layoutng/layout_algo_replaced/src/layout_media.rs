//! Source LayoutMedia panel geometry. Chromium 6c1d401fcca5,
//! core/layout/layout_media.cc:100-187. Coordinates stay on the resident object.
#![allow(non_snake_case)]
use foundation::{gfx, DynamicTo, LayoutUnit, PhysicalRect};
use layoutng_assembly::internal::{
    layout_input::MediaPanelMainFrame,
    layout_node_metadata::Element,
    layout_replaced::LayoutMedia,
    map_coordinates_flags::{MapCoordinatesFlags, MapCoordinatesMode},
    scrollbar_mode::mojom::blink::ScrollbarMode,
};

#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutMediaComputePanelWidth(
    this: &LayoutMedia,
    media_rect: &PhysicalRect,
) -> LayoutUnit {
    this.CheckIsNotDestroyed();
    let node = unsafe { &*this.GetNode() };
    let page = node.InputMediaPanelPageFacts().expect(
        "media panel layout requires main-frame topology and the canonical VisualViewport width",
    );
    if page.main_frame == MediaPanelMainFrame::Remote {
        return media_rect.size.width;
    }
    let element = DynamicTo::<Element>(this.GetNode());
    if !element.is_null()
        && unsafe { &*element }
            .InputElementData()
            .as_ref()
            .is_some_and(|data| data.media_is_fullscreen)
    {
        return media_rect.size.width;
    }
    let view = unsafe { this.View().as_ref() };
    let horizontal_mode = |view: &layoutng_assembly::internal::layout_view::LayoutView| {
        let mut horizontal = ScrollbarMode::kAuto;
        let mut vertical = ScrollbarMode::kAuto;
        view.CalculateScrollbarModes(&mut horizontal, &mut vertical, None, None);
        horizontal
    };
    let main_mode = match page.main_frame {
        MediaPanelMainFrame::Remote => unreachable!("remote main frame returned above"),
        MediaPanelMainFrame::NoLocalView => return media_rect.size.width,
        MediaPanelMainFrame::CurrentLocalFrame => {
            let Some(view) = view else {
                return media_rect.size.width;
            };
            horizontal_mode(view)
        }
        MediaPanelMainFrame::OtherLocalFrame {
            horizontal_scrollbar_mode,
        } => horizontal_scrollbar_mode,
    };
    if main_mode != ScrollbarMode::kAlwaysOff {
        return media_rect.size.width;
    }
    if view.is_some_and(|view| horizontal_mode(view) != ScrollbarMode::kAlwaysOff) {
        return media_rect.size.width;
    }
    let visible_width = page.visual_viewport_visible_width.expect(
        "first-layout media clipping requires the original VisualViewport to be initialized before Paint",
    );
    let flags = MapCoordinatesFlags::from_mode(MapCoordinatesMode::kTraverseDocumentBoundaries);
    let bottom_left = this.LocalToAbsolutePointF(
        &gfx::PointF::new(
            media_rect.offset.left.ToFloat(),
            media_rect.Bottom().ToFloat(),
        ),
        flags,
    );
    let bottom_right = this.LocalToAbsolutePointF(
        &gfx::PointF::new(media_rect.Right().ToFloat(), media_rect.Bottom().ToFloat()),
        flags,
    );
    let left_visible = bottom_left.x() < visible_width.ToFloat();
    let right_visible = bottom_right.x() < visible_width.ToFloat();
    if (left_visible && right_visible) || (!left_visible && !right_visible) || right_visible {
        return media_rect.size.width;
    }
    let slope = (bottom_right.y() - bottom_left.y()) / (bottom_right.x() - bottom_left.x());
    let edge_y = bottom_left.y() + (visible_width.ToFloat() - bottom_left.x()) * slope;
    let dx = visible_width.ToFloat() - bottom_left.x();
    let dy = edge_y - bottom_left.y();
    LayoutUnit::from_f64((dx * dx + dy * dy).sqrt() as f64)
}
