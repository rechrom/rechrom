//! Touch-action regions used by compositor hit testing.

use crate::touch_action::TouchAction;
use compositor_tree::geometry::Rect;
use std::collections::BTreeMap;

pub trait TouchRegion: Clone + Default {
    fn Empty() -> &'static Self;
    fn UnionRect(&mut self, rect: &Rect);
    fn UnionRegion(&mut self, other: &Self);
    fn Intersects(&self, rect: &Rect) -> bool;
    fn Equals(&self, other: &Self) -> bool;
}

impl<B: compositor_tree::region_backend::SkRegionBackend> TouchRegion
    for compositor_tree::region::Region<B>
{
    fn Empty() -> &'static Self {
        compositor_tree::region::Region::Empty()
    }

    fn UnionRect(&mut self, rect: &Rect) {
        self.Union(rect);
    }

    fn UnionRegion(&mut self, other: &Self) {
        self.Union(other);
    }

    fn Intersects(&self, rect: &Rect) -> bool {
        self.Intersects(rect)
    }

    fn Equals(&self, other: &Self) -> bool {
        self.Equals(other)
    }
}

#[derive(Clone, Default)]
pub struct TouchActionRegion<R: TouchRegion> {
    map_: BTreeMap<TouchAction, R>,
}

impl<R: TouchRegion + 'static> TouchActionRegion<R> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn Union(&mut self, touch_action: TouchAction, rect: &Rect) {
        self.map_.entry(touch_action).or_default().UnionRect(rect);
    }

    pub fn GetAllRegions(&self) -> R {
        let mut all_regions = R::default();
        for region in self.map_.values() {
            all_regions.UnionRegion(region);
        }
        all_regions
    }

    pub fn GetRegionForTouchAction(&self, touch_action: TouchAction) -> &R {
        self.map_.get(&touch_action).unwrap_or_else(|| R::Empty())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&TouchAction, &R)> {
        self.map_.iter()
    }

    pub fn IsEmpty(&self) -> bool {
        self.map_.is_empty()
    }

    pub fn GetAllowedTouchAction(&self, touch_rect: &Rect) -> TouchAction {
        let mut allowed_touch_action = TouchAction::kAuto;
        let non_empty_touch_rect = Rect::new(
            touch_rect.x,
            touch_rect.y,
            touch_rect.width.max(1),
            touch_rect.height.max(1),
        );
        for (touch_action, region) in &self.map_ {
            if region.Intersects(&non_empty_touch_rect) {
                allowed_touch_action &= *touch_action;
            }
        }
        allowed_touch_action
    }
}

impl<R: TouchRegion + 'static> PartialEq for TouchActionRegion<R> {
    fn eq(&self, other: &Self) -> bool {
        self.map_.len() == other.map_.len()
            && self.map_.iter().all(|(action, region)| {
                other
                    .map_
                    .get(action)
                    .is_some_and(|other_region| region.Equals(other_region))
            })
    }
}

impl<R: TouchRegion + 'static> Eq for TouchActionRegion<R> {}
