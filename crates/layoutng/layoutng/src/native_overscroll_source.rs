//! Native OverscrollAreaTracker geometry and its same scrollable-area snap
//! request. Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! core/overscroll/overscroll_area_tracker.cc:19-51,94-171.
use super::*;
use foundation::overscroll::*;
impl NativeScrollSource<'_> {
    fn OverscrollBox(&self) -> Option<&LayoutBox> {
        let object = unsafe { &*self.object };
        object
            .IsBox()
            .then(|| unsafe { &*(self.object as *const LayoutBox) })
    }
    pub fn OverscrollAreaGeometry(&self, action: OverscrollAction) -> Option<OverscrollGeometry> {
        let box_ = self.OverscrollBox()?;
        let area = unsafe { box_.GetScrollableArea().as_ref()? };
        if action == OverscrollAction::Close {
            return Some(OverscrollGeometry {
                snap: None,
                open_box: None,
            });
        }
        let snap = area.GetSnapContainerData().map(|data| {
            let enough = data.size() >= 2;
            OverscrollSnapData {
                area_count: data.size(),
                toggle_targets: (action == OverscrollAction::Toggle && enough).then(|| {
                    let targets = data.GetTargetSnapAreaElementIds();
                    OverscrollToggleTargets {
                        first_area_id: data.at(0).element_id.GetInternalValue(),
                        target_x_id: targets.x.GetInternalValue(),
                        target_y_id: targets.y.GetInternalValue(),
                    }
                }),
                second_area_rect: (action == OverscrollAction::Open && enough).then(|| {
                    let rect = data.at(1).rect;
                    OverscrollRect {
                        x: rect.origin().x(),
                        y: rect.origin().y(),
                        width: rect.size().width(),
                        height: rect.size().height(),
                    }
                }),
            }
        });
        let open_box = (action == OverscrollAction::Open
            && snap.is_some_and(|data| data.area_count >= 2))
        .then(|| {
            let origin = area.ScrollOrigin();
            let size = box_.PhysicalContentBoxRect().size;
            OverscrollOpenBox {
                scroll_origin: OverscrollOffset {
                    x: origin.x() as f32,
                    y: origin.y() as f32,
                },
                content_box_width: size.width.ToFloat(),
                content_box_height: size.height.ToFloat(),
            }
        });
        Some(OverscrollGeometry { snap, open_box })
    }
    pub fn OverscrollScrollToAndSnapBothAxes(
        &mut self,
        offset: OverscrollOffset,
    ) -> Option<NativePageScrollRequest> {
        let box_ = self.OverscrollBox()?;
        let area = unsafe { box_.GetScrollableArea().as_mut()? };
        Some(area.OverscrollScrollToAndSnapBothAxes(
            crate::internal::scroll_types::ScrollOffset::new(offset.x, offset.y),
            foundation::RuntimeEnabledFeatures::FractionalScrollOffsetsEnabled(),
        ))
    }
}
