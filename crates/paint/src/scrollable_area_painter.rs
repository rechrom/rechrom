#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::fragment_tree::ScrollbarPaintAxis;
use layoutng_assembly::internal::layout_input::{Offset, Size};

use crate::display_item_id::{DisplayItemId, DisplayItemIdType};
use crate::drawing_recorder::DrawingRecorder;
use crate::paint_chunker::DisplayItemClientInfo;
use crate::paint_context::{PaintContext, ScopedPaintChunkProperties};
use crate::paint_engine::{DisplayItem, DisplayItemType};
use crate::paint_info::PaintInfo;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;

// cpp: paint/scrollable_area_painter.cc:6-11
fn AbsoluteRect(node: &PaintTreeNode<'_>, offset: Offset, size: Size) -> PaintRect {
    PaintRect {
        x: node.paint_offset.x + offset.x,
        y: node.paint_offset.y + offset.y,
        width: size.width,
        height: size.height,
    }
}

// cpp: paint/scrollable_area_painter.h:7-17
pub struct ScrollableAreaPainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> ScrollableAreaPainter<'n, 'f, 'c, 'o> {
    // cpp: paint/scrollable_area_painter.h:9-10
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    // cpp: paint/scrollable_area_painter.h:12
    // cpp: paint/scrollable_area_painter.cc:15-82
    pub fn PaintOverflowControls(&self, info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let Some(data) = fragment.paint.scrollbars.as_ref() else {
            return;
        };
        let _border_properties =
            ScopedPaintChunkProperties::properties_only(self.context, self.node);
        let paint_axis = |axis: &ScrollbarPaintAxis| {
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
                    is_just_created: axis.display_item_client_is_just_created,
                },
                AbsoluteRect(self.node, axis.track_offset, axis.track_size),
                self.node.contents_properties.nodes.transform.clone(),
                self.node.paint_snap_offset,
            );
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawScrollbarTrack,
                    phase: info.phase,
                    node_id: fragment.node_id,
                    rect: AbsoluteRect(self.node, axis.track_offset, axis.track_size),
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
                        rect: AbsoluteRect(self.node, axis.thumb_offset, axis.thumb_size),
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
                        rect: AbsoluteRect(self.node, axis.track_offset, button_size),
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
                        rect: AbsoluteRect(self.node, end_offset, button_size),
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
        if data.corner_size.width > 0.0 && data.corner_size.height > 0.0 {
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
                    is_just_created: data.corner_client_is_just_created,
                },
                AbsoluteRect(self.node, data.corner_offset, data.corner_size),
            );
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawScrollbarCorner,
                    phase: info.phase,
                    node_id: fragment.node_id,
                    rect: AbsoluteRect(self.node, data.corner_offset, data.corner_size),
                    color: data.corner_color,
                    ..Default::default()
                },
                self.node,
            );
        }
    }
}
