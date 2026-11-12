//! Chromium 6c1d401f event_handler.cc153-165,2226-2382. Placement and the
//! generated MouseEvent belong to EventHandler; native owners only supply
//! existing caret, outline, viewport, focus and screen-coordinate values.
use super::{EventHandler, EventHandlerHitResult, NodeRef};
use foundation::gfx;
use interaction_dispatch::{
    core::events::pointer_event_factory::PointerEventFactory,
    public::platform::web_input_event_result::WebInputEventResult,
};
use interaction_input::{
    base::time::time::TimeTicks,
    public::common::input::{
        web_input_event::Type, web_mouse_event::WebMouseEvent, web_pointer_properties::Button,
    },
    ui::base::mojom::menu_source_type::MenuSourceType,
};
#[derive(Clone, Copy)]
pub struct NonLocatedSelectionFlags {
    pub is_range: bool,
    pub has_editable_root: bool,
    pub is_caret: bool,
}
#[derive(Clone, Copy)]
pub struct NonLocatedCaret {
    pub absolute_rect: gfx::Rect,
    pub horizontal: bool,
}
pub trait NonLocatedContextMenuSource<N: Clone = NodeRef> {
    fn HasDocument(&self) -> bool;
    fn FocusedElement(&self) -> Option<N>;
    fn SelectionFlags(&self) -> NonLocatedSelectionFlags;
    fn SelectionHasFocus(&self) -> bool;
    fn SelectionStartCaret(&self) -> Option<NonLocatedCaret>;
    fn SelectionEndCaret(&self) -> Option<NonLocatedCaret>;
    fn EnclosingAnchorAtSelectionStart(&self) -> Option<N>;
    fn ElementVisibleBoundsInLocalRoot(&self, element: &N) -> gfx::Rect;
    fn ElementOutlineRectsInWidget(&self, element: &N) -> Vec<gfx::Rect>;
    fn ConvertRectFromRootFrame(&self, rect: gfx::Rect) -> gfx::Rect;
    fn ConvertPointToRootFrame(&self, point: gfx::Point) -> gfx::Point;
    fn RootFrameToViewport(&self, rect: gfx::Rect) -> gfx::Rect;
    fn ViewportToRootFrame(&self, rect: gfx::Rect) -> gfx::Rect;
    fn VisualViewportSize(&self) -> gfx::Size;
    fn VisualViewportScrollOffset(&self) -> gfx::PointF;
    fn SetPointerCursor(&self);
    fn RootFramePointToScreenDIPs(&self, point: gfx::Point) -> gfx::Point;
    fn ShowContextMenuOnMouseUp(&self) -> bool;
    fn Now(&self) -> TimeTicks;
}
/// The source placement algorithm needs only these EventHandler leaves.
/// Native assembly can borrow its retained managers without recreating the
/// broad EventHandler frame interface or a parallel placement algorithm.
pub trait NonLocatedEventHandler<N: Clone> {
    fn HasView(&self) -> bool;
    fn IsOutermostMainFrame(&self) -> bool;
    fn UpdateStyleAndLayoutForContextMenu(&self);
    fn UpdateHoverActiveState(&self, focused: Option<N>);
    fn SendContextMenuEvent(
        &self,
        event: &WebMouseEvent,
        focused: Option<N>,
    ) -> WebInputEventResult;
}
fn MiddleSelectionCaret(caret: Option<NonLocatedCaret>) -> gfx::Point {
    let Some(caret) = caret else {
        return gfx::Point::default();
    };
    let rect = caret.absolute_rect;
    if caret.horizontal {
        gfx::Point::new(rect.x(), (rect.y() + rect.bottom()) / 2)
    } else {
        gfx::Point::new((rect.x() + rect.right()) / 2, rect.y())
    }
}
fn ContainsEvenAtEdge(rect: gfx::Rect, point: gfx::Point) -> bool {
    point.x() >= rect.x()
        && point.x() <= rect.right()
        && point.y() >= rect.y()
        && point.y() <= rect.bottom()
}
fn FocusedElementRect<N: Clone>(
    handler: &dyn NonLocatedEventHandler<N>,
    ctx: &dyn NonLocatedContextMenuSource<N>,
    element: &N,
) -> gfx::Rect {
    let mut visible = ctx.ElementVisibleBoundsInLocalRoot(element);
    if handler.IsOutermostMainFrame() {
        visible = ctx.RootFrameToViewport(visible);
        visible.Intersect(gfx::Rect::new(
            gfx::Point::default(),
            ctx.VisualViewportSize(),
        ));
    }
    let mut clipped = visible;
    let outlines = ctx.ElementOutlineRectsInWidget(element);
    if outlines.len() > 1 {
        clipped.Intersect(outlines[0]);
    }
    ctx.ViewportToRootFrame(clipped)
}
impl<'scope, R: EventHandlerHitResult> EventHandler<'scope, R> {
    pub fn ShowNonLocatedContextMenu(
        &self,
        override_target: Option<NodeRef>,
        source_type: MenuSourceType,
        ctx: &dyn NonLocatedContextMenuSource,
    ) -> WebInputEventResult {
        ShowNonLocatedContextMenu(self, override_target, source_type, ctx)
    }
}
impl<'scope, R: EventHandlerHitResult> NonLocatedEventHandler<NodeRef> for EventHandler<'scope, R> {
    fn HasView(&self) -> bool {
        self.frame_.HitTesting().HasView()
    }
    fn IsOutermostMainFrame(&self) -> bool {
        self.frame_.LocalFrameRoot().IsOutermostMainFrame()
    }
    fn UpdateStyleAndLayoutForContextMenu(&self) {
        self.frame_.UpdateStyleAndLayoutForContextMenu();
    }
    fn UpdateHoverActiveState(&self, focused: Option<NodeRef>) {
        self.frame_
            .HitTesting()
            .UpdateHoverActiveState(true, true, focused);
    }
    fn SendContextMenuEvent(
        &self,
        event: &WebMouseEvent,
        focused: Option<NodeRef>,
    ) -> WebInputEventResult {
        EventHandler::SendContextMenuEvent(self, event, focused)
    }
}
pub fn ShowNonLocatedContextMenu<N: Clone>(
    handler: &dyn NonLocatedEventHandler<N>,
    override_target: Option<N>,
    source_type: MenuSourceType,
    ctx: &dyn NonLocatedContextMenuSource<N>,
) -> WebInputEventResult {
    if !handler.HasView() || !ctx.HasDocument() {
        return WebInputEventResult::kNotHandled;
    }
    let mut focused = override_target.clone().or_else(|| ctx.FocusedElement());
    let should_show_at_selection = if override_target.is_none() {
        handler.UpdateStyleAndLayoutForContextMenu();
        let flags = ctx.SelectionFlags();
        (flags.is_range || flags.has_editable_root) && ctx.SelectionHasFocus()
    } else {
        false
    };
    let location = if should_show_at_selection {
        let flags = ctx.SelectionFlags();
        let start = MiddleSelectionCaret(ctx.SelectionStartCaret());
        let end = MiddleSelectionCaret(ctx.SelectionEndCaret());
        let (mut left, mut top, mut right, mut bottom) = (
            start.x().min(end.x()),
            start.y().min(end.y()),
            start.x().max(end.x()),
            start.y().max(end.y()),
        );
        if flags.is_caret {
            if let Some(anchor) = ctx.EnclosingAnchorAtSelectionStart() {
                focused = Some(anchor);
            }
        }
        if let Some(element) = focused.as_ref() {
            let clipped = ctx.ConvertRectFromRootFrame(FocusedElementRect(handler, ctx, element));
            left = left.max(clipped.x());
            top = top.max(clipped.y());
            right = right.min(clipped.right());
            bottom = bottom.min(clipped.bottom());
        }
        let rect = gfx::Rect::new(
            gfx::Point::new(left, top),
            gfx::Size::new(right - left, bottom - top),
        );
        let point = if ContainsEvenAtEdge(rect, start) {
            start
        } else if ContainsEvenAtEdge(rect, end) {
            end
        } else {
            rect.CenterPoint()
        };
        ctx.ConvertPointToRootFrame(point)
    } else if let Some(element) = focused.as_ref() {
        FocusedElementRect(handler, ctx, element).CenterPoint()
    } else {
        let offset = ctx.VisualViewportScrollOffset();
        gfx::Point::new((offset.x() + 1.0) as i32, (offset.y() + 1.0) as i32)
    };
    ctx.SetPointerCursor();
    let global = ctx.RootFramePointToScreenDIPs(location);
    // Source's transient result.InnerElement is exactly this focused
    // Element; Document fallback has no Element for hover/active state.
    handler.UpdateHoverActiveState(focused.clone());
    let kind = if ctx.ShowContextMenuOnMouseUp() {
        Type::kMouseUp
    } else {
        Type::kMouseDown
    };
    let mut event = WebMouseEvent::new(
        kind,
        gfx::PointF::new(location.x() as f32, location.y() as f32),
        gfx::PointF::new(global.x() as f32, global.y() as f32),
        Button::kNoButton,
        0,
        0,
        ctx.Now(),
        source_type,
        PointerEventFactory::kMouseId,
    );
    event.SetFrameScale(1.0);
    handler.SendContextMenuEvent(&event, focused)
}
