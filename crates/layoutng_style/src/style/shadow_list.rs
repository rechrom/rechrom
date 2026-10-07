use foundation::{gfx, HeapVector, Visitor};

use super::shadow_data::ShadowData;

// cpp: layoutng_style/style/shadow_list.h:44
pub type ShadowDataVector = HeapVector<ShadowData, 1>;

// cpp: layoutng_style/style/shadow_list.h:48-68
pub struct ShadowList {
    shadows_: ShadowDataVector,
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

    // cpp: layoutng_style/style/shadow_list.h:60-62
    // No definition is supplied in this package.
    pub fn RectOutsetsIncludingOriginal(&self) -> gfx::OutsetsF {
        unsafe { ShadowListRectOutsetsIncludingOriginal(self) }
    }

    // cpp: layoutng_style/style/shadow_list.h:64
    // No definition is supplied in this package.
    pub fn AdjustRectForShadow(&self, rect: &mut gfx::RectF) {
        unsafe { ShadowListAdjustRectForShadow(self, rect) }
    }
}

// cpp: layoutng_style/style/shadow_list.h:58
impl PartialEq for ShadowList {
    fn eq(&self, other: &Self) -> bool {
        self.shadows_ == other.shadows_
    }
}

unsafe extern "Rust" {
    fn ShadowListRectOutsetsIncludingOriginal(value: &ShadowList) -> gfx::OutsetsF;
    fn ShadowListAdjustRectForShadow(value: &ShadowList, rect: &mut gfx::RectF);
}
