//! ScrollbarTheme geometry shared by input and paint. No Interaction dependency.
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! scrollbar_theme.cc:54-101,195-278; scrollbar_theme_aura.cc:158-190,467-482;
//! scrollbar_theme_mac.cc:346-357; custom_scrollbar_theme.cc:45-56,59-139.
use super::super::super::scrollbar_part::ScrollbarPart as P;
use super::*;
#[derive(Clone, Copy)]
pub struct NativeScrollbarGeometry {
    pub frame: gfx::Rect,
    pub track_rect: gfx::Rect,
    pub thumb_rect: gfx::Rect,
    pub track_position: i32,
    pub track_length: i32,
    pub thumb_position: i32,
    pub thumb_length: i32,
    pub has_buttons: bool,
    before: gfx::Rect,
    after: gfx::Rect,
    back_button: gfx::Rect,
    forward_button: gfx::Rect,
}
#[derive(Clone, Copy)]
pub struct NativeScrollbarScrollState {
    pub area_identity: usize,
    pub scroll_element_id: foundation::CompositorElementId,
    pub horizontal: bool,
    pub is_layout_view: bool,
    pub minimum_offset: gfx::Vector2dF,
    pub maximum_offset: gfx::Vector2dF,
    pub current_offset: gfx::Vector2dF,
}
impl Scrollbar {
    pub fn ThemeGeometryAt(&self, current: f32) -> NativeScrollbarGeometry {
        let frame = self.FrameRect();
        let horizontal = self.Orientation() == ScrollbarOrientation::kHorizontalScrollbar;
        let axis = |r: gfx::Rect| {
            if horizontal {
                (r.x(), r.width())
            } else {
                (r.y(), r.height())
            }
        };
        let (track, constrained, before, thumb, after, back_button, forward_button, has_buttons) =
            if let Some(custom) = unsafe { self.custom.get().Get().as_ref() } {
                let track = custom.ThemeTrackRect();
                let constrained = custom.ConstrainTrackRect(&track);
                let (_, before, thumb, after) = custom.SplitTrack(
                    self.Enabled(),
                    self.VisibleSize(),
                    self.TotalSize(),
                    current,
                );
                let back = custom.ButtonRect(P::kBackButtonStartPart);
                let forward = custom.ButtonRect(P::kForwardButtonEndPart);
                let buttons = [
                    P::kBackButtonStartPart,
                    P::kForwardButtonStartPart,
                    P::kBackButtonEndPart,
                    P::kForwardButtonEndPart,
                ]
                .into_iter()
                .map(|p| axis(custom.ButtonRect(p)).1)
                .sum::<i32>();
                (
                    track,
                    constrained,
                    before,
                    thumb,
                    after,
                    back,
                    forward,
                    buttons <= axis(frame).1,
                )
            } else {
                let (start, length) = axis(frame);
                let cross = if horizontal {
                    frame.height()
                } else {
                    frame.width()
                };
                let button = if self.ThemeMetrics().has_buttons {
                    cross.min(length / 2)
                } else {
                    0
                };
                let back = if button == 0 {
                    gfx::Rect::default()
                } else if horizontal {
                    gfx::Rect::new(frame.origin(), gfx::Size::new(button, cross))
                } else {
                    gfx::Rect::new(frame.origin(), gfx::Size::new(cross, button))
                };
                let forward = if button == 0 {
                    gfx::Rect::default()
                } else if horizontal {
                    gfx::Rect::new(
                        gfx::Point::new(frame.right() - button, frame.y()),
                        back.size(),
                    )
                } else {
                    gfx::Rect::new(
                        gfx::Point::new(frame.x(), frame.bottom() - button),
                        back.size(),
                    )
                };
                let track = if length <= 2 * button {
                    gfx::Rect::default()
                } else if horizontal {
                    gfx::Rect::new(
                        gfx::Point::new(start + button, frame.y()),
                        gfx::Size::new(length - 2 * button, cross),
                    )
                } else {
                    gfx::Rect::new(
                        gfx::Point::new(frame.x(), start + button),
                        gfx::Size::new(cross, length - 2 * button),
                    )
                };
                let track_length = axis(track).1;
                let proportion = if self.TotalSize() > 0 {
                    self.VisibleSize() as f32 / self.TotalSize() as f32
                } else {
                    0.0
                };
                let thumb_length = if self.Enabled() {
                    ((proportion * track_length as f32).round() as i32)
                        .max(self.ThemeMetrics().minimum_thumb_length)
                        .min(track_length)
                } else {
                    0
                };
                let range = (self.TotalSize() - self.VisibleSize()) as f32;
                let thumb_position = if self.Enabled() && range != 0.0 {
                    let pos = current.max(0.0) * (track_length - thumb_length) as f32 / range;
                    if pos > 0.0 && pos < 1.0 {
                        1
                    } else {
                        pos as i32
                    }
                } else {
                    0
                };
                let (before, thumb, after) = if horizontal {
                    let thumb = gfx::Rect::new(
                        gfx::Point::new(track.x() + thumb_position, track.y()),
                        gfx::Size::new(thumb_length, frame.height()),
                    );
                    let before = gfx::Rect::new(
                        track.origin(),
                        gfx::Size::new(thumb_position + thumb_length / 2, track.height()),
                    );
                    let after = gfx::Rect::new(
                        gfx::Point::new(before.right(), track.y()),
                        gfx::Size::new(track.right() - before.right(), track.height()),
                    );
                    (before, thumb, after)
                } else {
                    let thumb = gfx::Rect::new(
                        gfx::Point::new(track.x(), track.y() + thumb_position),
                        gfx::Size::new(frame.width(), thumb_length),
                    );
                    let before = gfx::Rect::new(
                        track.origin(),
                        gfx::Size::new(track.width(), thumb_position + thumb_length / 2),
                    );
                    let after = gfx::Rect::new(
                        gfx::Point::new(track.x(), before.bottom()),
                        gfx::Size::new(track.width(), track.bottom() - before.bottom()),
                    );
                    (before, thumb, after)
                };
                (
                    track,
                    track,
                    before,
                    thumb,
                    after,
                    back,
                    forward,
                    self.ThemeMetrics().has_buttons,
                )
            };
        NativeScrollbarGeometry {
            frame,
            track_rect: track,
            thumb_rect: thumb,
            track_position: axis(constrained).0 - axis(frame).0,
            track_length: axis(constrained).1,
            thumb_position: axis(thumb).0 - axis(constrained).0,
            thumb_length: axis(thumb).1,
            has_buttons,
            before,
            after,
            back_button,
            forward_button,
        }
    }
    /// Input is already in the containing frame, after the real FrameView /
    /// VisualViewport conversion. Source: paint_layer_scrollable_area.cc:388-398.
    pub fn ConvertContainingFramePointToScrollbar(&self, point: gfx::Point) -> gfx::Point {
        let Some(area) = (unsafe { self.scrollable_area.Get().as_ref() }) else {
            return point;
        };
        let box_ = unsafe { &*area.GetLayoutBox() };

        let local = box_.AbsoluteToLocalPointPhysical(
            &foundation::PhysicalOffset::new(
                foundation::LayoutUnit::from_signed(point.x()),
                foundation::LayoutUnit::from_signed(point.y()),
            ),
            Default::default(),
        );
        let offset = area.ScrollbarOffset(self);
        gfx::Point::new(
            local.left.Round() - offset.x(),
            local.top.Round() - offset.y(),
        )
    }
    pub fn InputScrollState(&self) -> Option<NativeScrollbarScrollState> {
        let area = unsafe { self.scrollable_area.Get().as_ref() }?;
        Some(NativeScrollbarScrollState {
            area_identity: area as *const _ as usize,
            scroll_element_id: area.GetScrollElementId(),
            horizontal: self.Orientation() == ScrollbarOrientation::kHorizontalScrollbar,
            is_layout_view: unsafe { &*area.GetLayoutBox() }.IsLayoutView(),
            minimum_offset: area.MinimumScrollOffset(),
            maximum_offset: area.MaximumScrollOffset(),
            current_offset: area.GetScrollOffset(),
        })
    }
    pub fn HitTestScrollbarPoint(&self, local: gfx::Point) -> P {
        if !self.ThemeMetrics().allows_hit_test || !self.Enabled() {
            return P::kNoPart;
        }
        let geometry = self.ThemeGeometryAt(self.CurrentPos());
        let point = gfx::Point::new(
            local.x() + geometry.frame.x(),
            local.y() + geometry.frame.y(),
        );
        if !geometry.frame.ContainsPoint(point) {
            return P::kNoPart;
        }
        if geometry.track_rect.ContainsPoint(point) {
            if geometry.thumb_rect.ContainsPoint(point) {
                return P::kThumbPart;
            }
            if geometry.before.ContainsPoint(point) {
                return P::kBackTrackPart;
            }
            if geometry.after.ContainsPoint(point) {
                return P::kForwardTrackPart;
            }
            return P::kTrackBGPart;
        }
        if geometry.back_button.ContainsPoint(point) {
            return P::kBackButtonStartPart;
        }
        if geometry.forward_button.ContainsPoint(point) {
            return P::kForwardButtonEndPart;
        }
        if let Some(custom) = unsafe { self.custom.get().Get().as_ref() } {
            if custom
                .ButtonRect(P::kBackButtonEndPart)
                .ContainsPoint(point)
            {
                return P::kBackButtonEndPart;
            }
            if custom
                .ButtonRect(P::kForwardButtonStartPart)
                .ContainsPoint(point)
            {
                return P::kForwardButtonStartPart;
            }
        }
        P::kScrollbarBGPart
    }
}
