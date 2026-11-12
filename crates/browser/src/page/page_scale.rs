//! Assembly for WebView's page scale owner; geometry and zoom policy remain in
//! Layout/Interaction, while cc retains animation, delta and BeginFrame owners.
use super::*;
use interaction::default::core::exported::web_view_zoom::{PageScaleSettings, ViewportFacts};
impl PageState {
    pub(super) fn PageScaleViewportFacts(&self) -> Option<ViewportFacts> {
        let layout = self.layout_engine.borrow();
        let native = layout.PageScaleGeometry()?;
        let constraints = self.constraints.borrow();
        let viewport = constraints.viewport.as_ref()?;
        let frame_view_size = foundation::gfx::Size::new(viewport.size.width, viewport.size.height);
        Some(ViewportFacts {
            size: frame_view_size,
            document_size: native.document_size,
            // Source: VisualViewport::ContentsSize() returns
            // LocalFrameView::Size(), not LayoutViewport's visible content
            // rect.  This Page currently has one host frame viewport, so the
            // same retained constraint supplies both values.  Feeding the
            // root scroll area's geometry here gives the inner viewport the
            // document's scroll range at scale 1 and lets cc move pixels
            // without advancing Window.scrollY.
            layout_viewport_size: frame_view_size,
            layout_scroll_offset: native.layout_scroll_offset,
            visual_scroll_offset: self.web_view_zoom.borrow().VisualOffset(),
            minimum_layout_scroll_offset: native.minimum_scroll_offset,
            maximum_layout_scroll_offset: native.maximum_scroll_offset,
        })
    }
    pub(super) fn SyncVisualViewportPaintState(&self) {
        let Some(facts) = self.PageScaleViewportFacts() else {
            return;
        };
        let zoom = self.web_view_zoom.borrow();
        self.paint_engine
            .borrow_mut()
            .SetVisualViewportState(paint::PaintVisualViewportState {
                unique_id: self.visual_viewport_identity,
                scale: zoom.Scale(),
                offset: facts.visual_scroll_offset,
                size: facts.size,
                contents_size: facts.layout_viewport_size,
                maximum_scroll_offset: zoom.MaximumScrollOffset(&facts),
            });
    }
}
impl Page {
    /// WebSettings + the resolved PageScaleConstraintsSet enter as scalar
    /// values. Desktop defaults retain fixed scale; mobile/WebView embeddings
    /// may enable source double-tap behavior after resolving their constraints.
    pub fn SetPageScaleSettings(
        &mut self,
        scale: f32,
        settings: PageScaleSettings,
    ) -> io::Result<()> {
        if !settings.IsValid()
            || !scale.is_finite()
            || scale < settings.minimum_scale
            || scale > settings.maximum_scale
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid page scale settings",
            ));
        }
        self.state
            .web_view_zoom
            .borrow_mut()
            .SetScaleAndSettings(scale, settings);
        self.state.SyncVisualViewportPaintState();
        self.state.dirty.set(true);
        self.RouteWheelCompositorInput(CompositorInput::SetPageScaleFactorAndLimits {
            scale,
            minimum: settings.minimum_scale,
            maximum: settings.maximum_scale,
        })?;
        self.RequestBeginFrameIfNeeded();
        Ok(())
    }
    pub(super) fn HandleDoubleTapZoom(
        &mut self,
        event: &interaction::input_event::WebGestureEvent,
    ) -> io::Result<()> {
        if !self.state.web_view_zoom.borrow().DoubleTapEnabled() {
            return Ok(());
        }
        let point = self
            .state
            .web_view_zoom
            .borrow()
            .ViewportToRootFrame(event.PositionInRootFrame());
        let point = foundation::gfx::Point::new(point.x().floor() as i32, point.y().floor() as i32);
        let scroll::WidgetPointResult::BlockBounds(bounds) = self.WidgetPointAt(
            Offset {
                x: point.x() as f64,
                y: point.y() as f64,
            },
            scroll::WidgetPointQuery::BlockBounds,
        )?
        else {
            unreachable!()
        };
        let facts = self
            .state
            .PageScaleViewportFacts()
            .ok_or_else(|| io::Error::other("double tap zoom requires viewport geometry"))?;
        let animation = self
            .state
            .web_view_zoom
            .borrow_mut()
            .AnimateDoubleTapZoom(point, bounds, facts);
        if let Some(animation) = animation {
            self.RouteWheelCompositorInput(CompositorInput::StartPageScaleAnimation {
                target: compositor::adapter::geometry::Point::new(
                    animation.target_position.x(),
                    animation.target_position.y(),
                ),
                use_anchor: animation.use_anchor,
                scale: animation.scale,
                duration: CompositorTimeDelta::Milliseconds(animation.duration_milliseconds),
            })?;
        }
        Ok(())
    }
}
