#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::fragment_tree::ScrollbarPaintAxis;
use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb, Size};
use layoutng_assembly::internal::layout_input_types::Color;

use crate::box_fragment_painter::{GetHitTestOpaqueness, IsVisible};
use crate::display_item_id::{DisplayItemId, DisplayItemIdType};
use crate::drawing_recorder::DrawingRecorder;
use crate::paint_auto_dark_mode::{ElementRole, InvertColorIfNeeded};
use crate::paint_chunker::DisplayItemClientInfo;
use crate::paint_context::{PaintContext, ScopedPaintChunkProperties};
use crate::paint_engine::{DisplayItem, DisplayItemType};
use crate::paint_info::PaintInfo;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::{PaintPhase, PaintRect};

// Source: core/paint/scrollable_area_painter.cc:233-250.
fn ShouldPaintOverflowControls(
    node: &PaintTreeNode<'_>,
    should_paint_as_overlay: bool,
    phase: PaintPhase,
) -> bool {
    if should_paint_as_overlay {
        // Source: scrollable_area_painter.cc:240-247. A self-painting layer or
        // a scroller explicitly reparented by PaintLayerStackingNode uses the
        // overlay-controls phase. Ordinary non-self-painting scrollers stay in
        // foreground.
        if node.is_self_painting_layer || node.needs_reorder_overlay_overflow_controls.get() {
            phase == PaintPhase::kOverlayOverflowControls
        } else {
            phase == PaintPhase::kForeground
        }
    } else {
        paint_model::paint_phase::ShouldPaintSelfBlockBackground(phase)
    }
}

// Source: core/paint/scrollable_area_painter.cc:105-108. Resizer geometry is
// pixel-snapped in Layout; only the fragment paint offset is rounded here, as
// Chromium's current TODO explicitly requires.
fn AbsoluteRoundedRect(node: &PaintTreeNode<'_>, offset: Offset, size: Size) -> PaintRect {
    PaintRect {
        x: node.paint_offset.x.round() + offset.x,
        y: node.paint_offset.y.round() + offset.y,
        width: size.width,
        height: size.height,
    }
}

// Source: core/paint/scrollable_area_painter.cc:380-391.
fn ScrollbarHitTestOpaqueness(
    fragment: &layoutng_assembly::fragment_tree::FragmentPaintData,
    theme_disallows_hit_test: bool,
) -> crate::paint_engine::PaintHitTestOpaqueness {
    if theme_disallows_hit_test {
        return crate::paint_engine::PaintHitTestOpaqueness::Transparent;
    }
    match GetHitTestOpaqueness(fragment) {
        // A scrollbar is opaque to hit testing whenever its host is visible
        // to hit testing; cc relies on this for non-solid-color layers.
        crate::paint_engine::PaintHitTestOpaqueness::Mixed => {
            crate::paint_engine::PaintHitTestOpaqueness::Opaque
        }
        value => value,
    }
}

fn ResizerPath(points: [(f64, f64); 4]) -> Vec<PaintPathCommand> {
    [
        (PaintPathVerb::kMoveTo, points[0]),
        (PaintPathVerb::kLineTo, points[1]),
        (PaintPathVerb::kMoveTo, points[2]),
        (PaintPathVerb::kLineTo, points[3]),
    ]
    .into_iter()
    .map(|(verb, (x, y))| PaintPathCommand {
        verb,
        point: Offset { x, y },
        ..PaintPathCommand::default()
    })
    .collect()
}

// Source: core/paint/scrollable_area_painter.cc:116-216.
fn PlatformResizerPaths(
    rect: PaintRect,
    paint_scale: f64,
    on_left: bool,
) -> (Vec<PaintPathCommand>, Vec<PaintPathCommand>, f64) {
    let mut points = [(0.0, 0.0); 4];
    if foundation::RuntimeEnabledFeatures::TextAreaResizerFixedSizeEnabled() {
        let default_spacing = (2.0 * paint_scale).round() as i32;
        let default_size = (7.0 * paint_scale).round() as i32;
        let width = rect.width as i32;
        let height = rect.height as i32;
        let spacing_x = if width > 0 {
            default_spacing.min(width / 4).max(1)
        } else {
            0
        };
        let spacing_y = if height > 0 {
            default_spacing.min(height / 4).max(1)
        } else {
            0
        };
        let size_x = default_size.min(width - spacing_x).max(1);
        let size_y = default_size.min(height - spacing_y).max(1);
        let spacing = f64::from(spacing_x.min(spacing_y));
        let size_large = f64::from(size_x.min(size_y));
        let size_small = (size_large / 2.0).floor().max(1.0);
        if on_left {
            points[0].0 = rect.x + spacing;
            points[1].0 = rect.x + spacing + size_large;
            points[2].0 = points[0].0;
            points[3].0 = rect.x + spacing + size_small;
        } else {
            points[0].0 = rect.x + rect.width - spacing;
            points[1].0 = rect.x + rect.width - (spacing + size_large);
            points[2].0 = points[0].0;
            points[3].0 = rect.x + rect.width - (spacing + size_small);
        }
        points[0].1 = rect.y + rect.height - (spacing + size_large);
        points[1].1 = rect.y + rect.height - spacing;
        points[2].1 = rect.y + rect.height - (spacing + size_small);
        points[3].1 = points[1].1;
    } else {
        let edge_offset = paint_scale.ceil();
        if on_left {
            points[0].0 = rect.x + edge_offset;
            points[1].0 = rect.x + rect.width - (rect.width as i32 / 2) as f64;
            points[2].0 = points[0].0;
            points[3].0 = rect.x + rect.width - (rect.width as i32 * 3 / 4) as f64;
        } else {
            points[0].0 = rect.x + rect.width - edge_offset;
            points[1].0 = rect.x + (rect.width as i32 / 2) as f64;
            points[2].0 = points[0].0;
            points[3].0 = rect.x + (rect.width as i32 * 3 / 4) as f64;
        }
        points[0].1 = rect.y + (rect.height as i32 / 2) as f64;
        points[1].1 = rect.y + rect.height - edge_offset;
        points[2].1 = rect.y + (rect.height as i32 * 3 / 4) as f64;
        points[3].1 = points[1].1;
    }
    let offset = paint_scale.ceil();
    let horizontal_offset = if on_left { -offset } else { offset };
    let light = [
        (points[0].0, points[0].1 + offset),
        (points[1].0 + horizontal_offset, points[1].1),
        (points[2].0, points[2].1 + offset),
        (points[3].0 + horizontal_offset, points[3].1),
    ];
    (ResizerPath(points), ResizerPath(light), offset)
}

// Source: core/paint/scrollable_area_painter.h:24-64.
pub struct ScrollableAreaPainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> ScrollableAreaPainter<'n, 'f, 'c, 'o> {
    // Source: core/paint/scrollable_area_painter.h:28-30.
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    // Source: core/paint/scrollable_area_painter.h:34-37.
    // Source: core/paint/scrollable_area_painter.cc:218-446.
    pub fn PaintOverflowControls(&self, info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let Some(data) = fragment.paint.scrollbars.as_ref() else {
            return;
        };
        if !ShouldPaintOverflowControls(self.node, data.should_paint_as_overlay, info.phase) {
            return;
        }
        let _border_properties = self
            .node
            .overflow_controls_properties
            .clone()
            .map(|state| ScopedPaintChunkProperties::state(self.context, state))
            .or_else(|| {
                Some(ScopedPaintChunkProperties::properties_only(
                    self.context,
                    self.node,
                ))
            });
        let paint_axis = |axis: &ScrollbarPaintAxis| {
            // Source: scrollable_area_painter.cc:312-324. Overlay scrollbars
            // are suppressed while printing, and every control is culled in
            // its rounded paint-offset geometry before recording.
            if data.uses_overlay_scrollbars && fragment.paint.IsPrinting() {
                return;
            }
            let visual_rect = AbsoluteRoundedRect(self.node, axis.track_offset, axis.track_size);
            if !IsVisible(self.node, &visual_rect) {
                return;
            }
            let scrollbar_properties = if axis.horizontal {
                self.node.horizontal_scrollbar_properties.as_ref()
            } else {
                self.node.vertical_scrollbar_properties.as_ref()
            };
            let _scrollbar_effect = scrollbar_properties
                .cloned()
                .map(|state| ScopedPaintChunkProperties::state(self.context, state));
            self.context.borrow_mut().BeginScrollbarDisplayItem(
                DisplayItemId {
                    client_id: axis.display_item_client_id,
                    r#type: if axis.horizontal {
                        DisplayItemIdType::kScrollbarHorizontal
                    } else {
                        DisplayItemIdType::kScrollbarVertical
                    },
                    fragment: fragment.paint.display_item_fragment,
                },
                DisplayItemClientInfo {
                    is_cacheable: axis.display_item_client_is_cacheable,
                    invalidation_reason: axis.display_item_client_invalidation_reason,
                    is_just_created: axis.display_item_client_is_just_created.get(),
                },
                visual_rect,
                Some(self.node.contents_properties.nodes.transform.clone()),
                crate::paint_engine::ScrollbarDisplayItemData {
                    element_id: axis.compositor_element_id,
                    track_rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: axis.track_size.width,
                        height: axis.track_size.height,
                    },
                    thumb_rect: PaintRect {
                        x: axis.thumb_offset.x - axis.track_offset.x,
                        y: axis.thumb_offset.y - axis.track_offset.y,
                        width: axis.thumb_size.width,
                        height: axis.thumb_size.height,
                    },
                    thumb_color: data.thumb_color,
                    painted_opacity: axis.painted_opacity,
                    painted_thumb_width: axis.painted_thumb_width,
                    is_solid_color: data.is_solid_color,
                    uses_macos_painted_scrollbars: data.uses_macos_painted_scrollbars,
                    horizontal: axis.horizontal,
                    overlay: data.uses_overlay_scrollbars,
                    left_side_vertical: axis.left_side_vertical,
                    minimum_thumb_length: data.minimum_thumb_length,
                    needs_update_display: axis.needs_update_display.unwrap_or(true),
                    hit_test_opaqueness: ScrollbarHitTestOpaqueness(
                        fragment,
                        data.theme_disallows_hit_test,
                    ),
                },
                fragment.paint.display_item_raster_effect_outset,
                self.node.paint_snap_offset,
            );
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawScrollbarTrack,
                    phase: info.phase,
                    node_id: fragment.node_id,
                    rect: visual_rect,
                    color: data.track_color,
                    horizontal: axis.horizontal,
                    ..Default::default()
                },
                self.node,
            );
            if axis.thumb_size.width > 0.0 && axis.thumb_size.height > 0.0 {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawScrollbarThumb,
                        phase: info.phase,
                        node_id: fragment.node_id,
                        rect: AbsoluteRoundedRect(self.node, axis.thumb_offset, axis.thumb_size),
                        color: data.thumb_color,
                        horizontal: axis.horizontal,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            if axis.has_buttons {
                let extent = if axis.horizontal {
                    axis.track_size.height
                } else {
                    axis.track_size.width
                };
                let button_size = if axis.horizontal {
                    Size {
                        width: extent,
                        height: axis.track_size.height,
                    }
                } else {
                    Size {
                        width: axis.track_size.width,
                        height: extent,
                    }
                };
                let mut end_offset = axis.track_offset;
                if axis.horizontal {
                    end_offset.x += axis.track_size.width - extent;
                } else {
                    end_offset.y += axis.track_size.height - extent;
                }
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawScrollbarButton,
                        phase: info.phase,
                        node_id: fragment.node_id,
                        rect: AbsoluteRoundedRect(self.node, axis.track_offset, button_size),
                        color: data.button_color,
                        horizontal: axis.horizontal,
                        at_start: true,
                        ..Default::default()
                    },
                    self.node,
                );
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawScrollbarButton,
                        phase: info.phase,
                        node_id: fragment.node_id,
                        rect: AbsoluteRoundedRect(self.node, end_offset, button_size),
                        color: data.button_color,
                        horizontal: axis.horizontal,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            self.context.borrow_mut().EndDrawingRecorder();
        };
        if let Some(horizontal) = data.horizontal.as_ref() {
            paint_axis(horizontal);
        }
        if let Some(vertical) = data.vertical.as_ref() {
            paint_axis(vertical);
        }
        let corner_rect = AbsoluteRoundedRect(self.node, data.corner_offset, data.corner_size);
        if data.corner_size.width > 0.0
            && data.corner_size.height > 0.0
            && IsVisible(self.node, &corner_rect)
            // Source: scrollable_area_painter.cc:429-432. The ordinary source
            // corner remains transparent behind overlay scrollbars.
            && !data.uses_overlay_scrollbars
        {
            let _corner_recorder = DrawingRecorder::with_client(
                self.context,
                self.node,
                DisplayItemId {
                    client_id: data.corner_client_id,
                    r#type: DisplayItemIdType::kScrollCorner,
                    fragment: fragment.paint.display_item_fragment,
                },
                DisplayItemClientInfo {
                    is_cacheable: data.corner_client_is_cacheable,
                    invalidation_reason: data.corner_client_invalidation_reason,
                    is_just_created: data.corner_client_is_just_created.get(),
                },
                corner_rect,
            );
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawScrollbarCorner,
                    phase: info.phase,
                    node_id: fragment.node_id,
                    rect: corner_rect,
                    color: data.corner_color,
                    ..Default::default()
                },
                self.node,
            );
        }
        self.PaintResizer(info);
    }

    // Source: core/paint/scrollable_area_painter.cc:43-95,116-216. Custom
    // ::-webkit-resizer pseudo painting remains outside this source payload;
    // the ordinary platform resizer follows the source recorder and path.
    fn PaintResizer(&self, info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let Some(data) = fragment.paint.scrollbars.as_ref() else {
            return;
        };
        if data.resizer_pointer_size.width <= 0.0 || data.resizer_pointer_size.height <= 0.0 {
            return;
        }
        let visual_rect = AbsoluteRoundedRect(
            self.node,
            data.resizer_pointer_offset,
            data.resizer_pointer_size,
        );
        if !IsVisible(self.node, &visual_rect) {
            return;
        }
        let id = DisplayItemId {
            client_id: data.corner_client_id,
            r#type: DisplayItemIdType::kResizer,
            fragment: fragment.paint.display_item_fragment,
        };
        let client = DisplayItemClientInfo {
            is_cacheable: data.corner_client_is_cacheable,
            invalidation_reason: data.corner_client_invalidation_reason,
            is_just_created: data.corner_client_is_just_created.get(),
        };
        if self
            .context
            .borrow_mut()
            .UseCachedDrawingWithClient(id, client, None)
        {
            return;
        }
        let _recorder =
            DrawingRecorder::with_client(self.context, self.node, id, client, visual_rect);
        let (dark_path, light_path, stroke_width) =
            PlatformResizerPaths(visual_rect, data.resizer_paint_scale, data.resizer_on_left);
        for (path, color) in [
            (
                dark_path,
                Color {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 153.0 / 255.0,
                },
            ),
            (
                light_path,
                Color {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 153.0 / 255.0,
                },
            ),
        ] {
            let color = InvertColorIfNeeded(&fragment.paint.style, color, ElementRole::kBackground);
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kStrokePath,
                    phase: info.phase,
                    node_id: fragment.node_id,
                    color,
                    line_style: layoutng_assembly::internal::layout_input::BorderLineStyle::kSolid,
                    stroke_width,
                    path,
                    ..Default::default()
                },
                self.node,
            );
        }
        if !foundation::RuntimeEnabledFeatures::TextAreaResizerFixedSizeEnabled()
            && data.needs_scroll_corner
        {
            for item in [
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase: info.phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kClipRect,
                    phase: info.phase,
                    node_id: fragment.node_id,
                    rect: visual_rect,
                    antialias: false,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kStrokeRect,
                    phase: info.phase,
                    node_id: fragment.node_id,
                    rect: PaintRect {
                        x: visual_rect.x + 0.5,
                        y: visual_rect.y + 0.5,
                        width: visual_rect.width,
                        height: visual_rect.height,
                    },
                    color: Color {
                        red: 217.0 / 255.0,
                        green: 217.0 / 255.0,
                        blue: 217.0 / 255.0,
                        alpha: 1.0,
                    },
                    stroke_width: 1.0,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: info.phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
            ] {
                self.context.borrow_mut().Append(item, self.node);
            }
        }
    }

    // Source: core/paint/scrollable_area_painter.h:42-45.
    // Source: core/paint/scrollable_area_painter.cc:97-114.
    pub fn RecordResizerScrollHitTestData(&self) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let Some(data) = fragment.paint.scrollbars.as_ref() else {
            return;
        };
        if data.resizer_touch_size.width <= 0.0 || data.resizer_touch_size.height <= 0.0 {
            return;
        }
        self.context.borrow_mut().RecordResizerScrollHitTestData(
            self.node,
            data.corner_client_id,
            DisplayItemClientInfo {
                is_cacheable: data.corner_client_is_cacheable,
                invalidation_reason: data.corner_client_invalidation_reason,
                is_just_created: data.corner_client_is_just_created.get(),
            },
            AbsoluteRoundedRect(
                self.node,
                data.resizer_touch_offset,
                data.resizer_touch_size,
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layoutng_assembly::fragment_tree::FragmentPaintData;

    fn node(is_paint_layer: bool) -> PaintTreeNode<'static> {
        let fragment = FragmentPaintData::default();
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.is_paint_layer = is_paint_layer;
        node
    }

    #[test]
    fn overflow_controls_follow_chromium_paint_phases() {
        let layer = node(true);
        assert!(ShouldPaintOverflowControls(
            &layer,
            false,
            PaintPhase::kSelfBlockBackgroundOnly,
        ));
        assert!(!ShouldPaintOverflowControls(
            &layer,
            false,
            PaintPhase::kOverlayOverflowControls,
        ));
        let mut self_painting_layer = node(true);
        self_painting_layer.is_self_painting_layer = true;
        assert!(ShouldPaintOverflowControls(
            &self_painting_layer,
            true,
            PaintPhase::kOverlayOverflowControls,
        ));

        let non_self_painting_layer = node(true);
        assert!(ShouldPaintOverflowControls(
            &non_self_painting_layer,
            true,
            PaintPhase::kForeground,
        ));
        assert!(!ShouldPaintOverflowControls(
            &non_self_painting_layer,
            true,
            PaintPhase::kOverlayOverflowControls,
        ));
        non_self_painting_layer
            .needs_reorder_overlay_overflow_controls
            .set(true);
        assert!(ShouldPaintOverflowControls(
            &non_self_painting_layer,
            true,
            PaintPhase::kOverlayOverflowControls,
        ));
        assert!(!ShouldPaintOverflowControls(
            &non_self_painting_layer,
            true,
            PaintPhase::kForeground,
        ));
    }

    #[test]
    fn resizer_touch_rect_rounds_only_the_paint_offset() {
        let mut node = node(true);
        node.paint_offset = Offset { x: 10.6, y: 20.4 };
        assert_eq!(
            AbsoluteRoundedRect(
                &node,
                Offset { x: 70.0, y: 30.0 },
                Size {
                    width: 30.0,
                    height: 30.0,
                },
            ),
            PaintRect {
                x: 81.0,
                y: 50.0,
                width: 30.0,
                height: 30.0,
            }
        );
    }

    #[test]
    fn scrollbar_hit_testing_follows_the_theme_and_host_box() {
        let mut fragment = FragmentPaintData::default();
        assert_eq!(
            ScrollbarHitTestOpaqueness(&fragment, false),
            crate::paint_engine::PaintHitTestOpaqueness::Opaque
        );
        assert_eq!(
            ScrollbarHitTestOpaqueness(&fragment, true),
            crate::paint_engine::PaintHitTestOpaqueness::Transparent
        );
        fragment.paint.style.SetPointerEventsNoneForTesting(true);
        assert_eq!(
            ScrollbarHitTestOpaqueness(&fragment, false),
            crate::paint_engine::PaintHitTestOpaqueness::Transparent
        );
    }

    #[test]
    fn platform_resizer_paths_match_fixed_size_chromium_geometry() {
        let rect = PaintRect {
            x: 85.0,
            y: 45.0,
            width: 15.0,
            height: 15.0,
        };
        let (dark, light, width) = PlatformResizerPaths(rect, 1.0, false);
        let points = |path: &[PaintPathCommand]| {
            path.iter()
                .map(|command| (command.point.x, command.point.y))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            points(&dark),
            vec![(98.0, 51.0), (91.0, 58.0), (98.0, 55.0), (95.0, 58.0)]
        );
        assert_eq!(
            points(&light),
            vec![(98.0, 52.0), (92.0, 58.0), (98.0, 56.0), (96.0, 58.0)]
        );
        assert_eq!(width, 1.0);

        let (scaled_dark, _, scaled_width) = PlatformResizerPaths(
            PaintRect {
                width: 30.0,
                height: 30.0,
                ..rect
            },
            2.0,
            true,
        );
        assert_eq!(
            points(&scaled_dark),
            vec![(89.0, 57.0), (103.0, 71.0), (89.0, 64.0), (96.0, 71.0)]
        );
        assert_eq!(scaled_width, 2.0);
    }
}
