//! Chromium cc/animation/scroll_offset_animations.{h,cc}:16-77.
use compositor_tree::{geometry::Vector2dF, ElementId};
use std::collections::BTreeMap;
#[derive(Clone, Copy)]
pub struct ScrollOffsetAnimationUpdate {
    pub element: ElementId,
    pub adjustment: Vector2dF,
    pub takeover: bool,
}
#[derive(Default)]
pub struct ScrollOffsetAnimations {
    updates: BTreeMap<ElementId, ScrollOffsetAnimationUpdate>,
}
impl ScrollOffsetAnimations {
    fn Update(&mut self, element: ElementId) -> &mut ScrollOffsetAnimationUpdate {
        assert_ne!(element, ElementId::default());
        self.updates
            .entry(element)
            .or_insert(ScrollOffsetAnimationUpdate {
                element,
                adjustment: Vector2dF::default(),
                takeover: false,
            })
    }
    // The AnimationHost caller immediately sets needs commit and push props.
    pub fn AddAdjustmentUpdate(&mut self, element: ElementId, adjustment: Vector2dF) {
        self.Update(element).adjustment += adjustment;
    }
    pub fn AddTakeoverUpdate(&mut self, element: ElementId) {
        self.Update(element).takeover = true;
    }
    pub fn TakeUpdates(&mut self) -> BTreeMap<ElementId, ScrollOffsetAnimationUpdate> {
        std::mem::take(&mut self.updates)
    }
}
