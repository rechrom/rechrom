use foundation::{gfx, HeapVector, Visitor};

use super::shadow_data::{ShadowData, ShadowStyle};

// cpp: layoutng_style/style/shadow_list.h:44
pub type ShadowDataVector = HeapVector<ShadowData, 1>;

// cpp: layoutng_style/style/shadow_list.h:48-68
pub struct ShadowList {
    shadows_: ShadowDataVector,
}

// cpp: shadow_list.h:55. Register native ShadowList with the production GC;
// ShadowData's trace retains unresolved StyleColor components.
impl foundation::Traceable for ShadowList {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        ShadowList::Trace(self, visitor);
    }
}

#[allow(non_snake_case)]
impl ShadowList {
    // cpp: layoutng_style/style/shadow_list.h:50-53
    pub fn new(shadows: ShadowDataVector) -> Self {
        debug_assert!(!shadows.empty());
        Self { shadows_: shadows }
    }

    // cpp: layoutng_style/style/shadow_list.h:55
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.shadows_);
    }

    // cpp: layoutng_style/style/shadow_list.h:57
    pub fn Shadows(&self) -> &ShadowDataVector {
        &self.shadows_
    }

    // Chromium core/style/shadow_list.cc:39-49.
    pub fn RectOutsetsIncludingOriginal(&self) -> gfx::OutsetsF {
        let mut outsets = gfx::OutsetsF::default();
        for shadow in self.Shadows() {
            if shadow.Style() == ShadowStyle::kInset {
                continue;
            }
            let current = shadow.RectOutsets();
            outsets.set_top(outsets.top().max(current.top()));
            outsets.set_right(outsets.right().max(current.right()));
            outsets.set_bottom(outsets.bottom().max(current.bottom()));
            outsets.set_left(outsets.left().max(current.left()));
        }
        outsets
    }

    // Chromium core/style/shadow_list.cc:51-53.
    pub fn AdjustRectForShadow(&self, rect: &mut gfx::RectF) {
        let outsets = self.RectOutsetsIncludingOriginal();
        *rect = gfx::RectF::new(
            gfx::PointF::new(rect.x() - outsets.left(), rect.y() - outsets.top()),
            gfx::SizeF::new(
                rect.width() + outsets.left() + outsets.right(),
                rect.height() + outsets.top() + outsets.bottom(),
            ),
        );
    }
}

// cpp: layoutng_style/style/shadow_list.h:58
impl PartialEq for ShadowList {
    fn eq(&self, other: &Self) -> bool {
        self.shadows_ == other.shadows_
    }
}
