use std::ops::{Deref, DerefMut};

use layoutng::internal::paint_layer::PaintLayer;

use foundation::{
    HeapVector, MakeGarbageCollected, Member, NewUniqueObjectId, PhysicalOffset, UniqueObjectId,
    Visitor,
};
use layoutng_geometry::geometry::axis::{
    kPhysicalAxesHorizontal, kPhysicalAxesVertical, PhysicalAxes,
};

use layoutng::internal::sticky_position_scrolling_constraints::{
    PerAxisData, StickyConstraintsData, StickyPositionScrollingConstraints,
};

// C++ nests RareData under FragmentData. Rust keeps it beside the owner so
// the list and GC tracing code can address it without inheritance.
// cpp: core/paint/fragment_data.h:205-235
pub(crate) struct FragmentDataRareData {
    pub(crate) additional_fragments: HeapVector<Member<FragmentData>>,
    pub(crate) unique_id: UniqueObjectId,
    // These object-wide values are stored only on the first fragment.
    // cpp: core/paint/fragment_data.h:218-226
    pub(crate) layer: Member<PaintLayer>,
    pub(crate) x_sticky_constraints: Member<PerAxisData>,
    pub(crate) y_sticky_constraints: Member<PerAxisData>,
}

impl Default for FragmentDataRareData {
    // cpp: core/paint/fragment_data.cc:16-17
    fn default() -> Self {
        Self {
            additional_fragments: HeapVector::default(),
            unique_id: 0,
            layer: Member::default(),
            x_sticky_constraints: Member::default(),
            y_sticky_constraints: Member::default(),
        }
    }
}

#[allow(non_snake_case)]
impl FragmentDataRareData {
    // cpp: core/paint/fragment_data.h:213
    // cpp: core/paint/fragment_data.cc:19-23
    pub(crate) fn EnsureId(&mut self) {
        if self.unique_id == 0 {
            self.unique_id = NewUniqueObjectId();
        }
    }

    // cpp: core/paint/fragment_data.h:216
    // cpp: core/paint/fragment_data.cc:34-41
    pub(crate) fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.layer);
        visitor.Trace(&self.x_sticky_constraints);
        visitor.Trace(&self.y_sticky_constraints);
        visitor.Trace(&self.additional_fragments);
    }
}

// cpp: core/paint/fragment_data.h:28-30,239-244
pub struct FragmentData {
    pub(crate) paint_offset_: PhysicalOffset,
    pub(crate) rare_data_: Member<FragmentDataRareData>,
    #[cfg(debug_assertions)]
    is_first_: bool,
}

impl Default for FragmentData {
    fn default() -> Self {
        Self {
            paint_offset_: PhysicalOffset::default(),
            rare_data_: Member::default(),
            #[cfg(debug_assertions)]
            is_first_: false,
        }
    }
}

#[allow(non_snake_case)]
impl FragmentData {
    // cpp: core/paint/fragment_data.h:48-52
    pub fn Layer(&self) -> *mut PaintLayer {
        self.AssertIsFirst();
        let rare = self.rare_data_.Get();
        if rare.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*rare }.layer.Get()
        }
    }

    // cpp: core/paint/fragment_data.cc:25-32,49-53
    pub fn SetLayer(&mut self, layer: *mut PaintLayer) {
        self.AssertIsFirst();
        if self.rare_data_.Get().is_null() && layer.is_null() {
            return;
        }
        let rare = self.EnsureRareData();
        let previous = rare.layer.Get();
        if !previous.is_null() && previous != layer {
            unsafe { &mut *previous }.Destroy();
            rare.x_sticky_constraints = Member::default();
            rare.y_sticky_constraints = Member::default();
        }
        rare.layer = Member::from_ptr(layer);
    }

    // cpp: core/paint/fragment_data.h:57-64
    pub fn StickyConstraints(&self) -> StickyPositionScrollingConstraints {
        self.AssertIsFirst();
        let rare = self.rare_data_.Get();
        if rare.is_null() {
            return StickyPositionScrollingConstraints::default();
        }
        let rare = unsafe { &*rare };
        StickyPositionScrollingConstraints::new(
            rare.x_sticky_constraints.Get(),
            rare.y_sticky_constraints.Get(),
        )
    }

    // cpp: core/paint/fragment_data.h:66-70
    pub fn HasStickyConstraints(&self) -> bool {
        self.AssertIsFirst();
        let rare = self.rare_data_.Get();
        !rare.is_null()
            && (!unsafe { &*rare }.x_sticky_constraints.Get().is_null()
                || !unsafe { &*rare }.y_sticky_constraints.Get().is_null())
    }

    // cpp: core/paint/fragment_data.h:72-88
    pub fn SetStickyConstraints(&mut self, constraints: StickyConstraintsData) -> bool {
        self.AssertIsFirst();
        debug_assert!(!constraints.x_data.is_null() || !constraints.y_data.is_null());
        let rare = self.EnsureRareData();
        let mut changed = false;
        if !constraints.x_data.is_null() {
            changed = true;
            rare.x_sticky_constraints = Member::from_ptr(constraints.x_data);
        }
        if !constraints.y_data.is_null() {
            changed = true;
            rare.y_sticky_constraints = Member::from_ptr(constraints.y_data);
        }
        changed
    }

    // cpp: core/paint/fragment_data.h:90-108
    pub fn ClearStickyConstraints(&mut self, axes_to_clear: PhysicalAxes) -> bool {
        self.AssertIsFirst();
        let rare = self.rare_data_.Get();
        if rare.is_null() {
            return false;
        }
        let rare = unsafe { &mut *rare };
        let mut changed = false;
        if (axes_to_clear & kPhysicalAxesHorizontal).is_nonzero()
            && !rare.x_sticky_constraints.Get().is_null()
        {
            rare.x_sticky_constraints = Member::default();
            changed = true;
        }
        if (axes_to_clear & kPhysicalAxesVertical).is_nonzero()
            && !rare.y_sticky_constraints.Get().is_null()
        {
            rare.y_sticky_constraints = Member::default();
            changed = true;
        }
        changed
    }

    // cpp: core/paint/fragment_data.h:32-35
    pub fn PaintOffset(&self) -> PhysicalOffset {
        self.paint_offset_
    }

    // cpp: core/paint/fragment_data.h:36-38
    pub fn SetPaintOffset(&mut self, paint_offset: &PhysicalOffset) {
        self.paint_offset_ = *paint_offset;
    }

    // cpp: core/paint/fragment_data.h:40-44
    pub fn UniqueId(&self) -> UniqueObjectId {
        let rare = self.rare_data_.Get();
        debug_assert!(!rare.is_null());
        unsafe { &*rare }.unique_id
    }

    // cpp: core/paint/fragment_data.h:123
    pub fn EnsureId(&mut self) {
        self.EnsureRareData().EnsureId();
    }

    // cpp: core/paint/fragment_data.h:124
    pub fn HasUniqueId(&self) -> bool {
        let rare = self.rare_data_.Get();
        !rare.is_null() && unsafe { &*rare }.unique_id != 0
    }

    // cpp: core/paint/fragment_data.h:187-189
    #[cfg(debug_assertions)]
    pub fn SetIsFirst(&mut self) {
        self.is_first_ = true;
    }

    // C++ has a default destructor; Rust field ownership supplies it.
    // cpp: core/paint/fragment_data.h:191

    // cpp: core/paint/fragment_data.h:192
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.rare_data_);
    }

    // cpp: core/paint/fragment_data.h:199-203
    pub(crate) fn AssertIsFirst(&self) {
        #[cfg(debug_assertions)]
        debug_assert!(self.is_first_);
    }

    // cpp: core/paint/fragment_data.h:237
    // cpp: core/paint/fragment_data.cc:43-47
    pub(crate) fn EnsureRareData(&mut self) -> &mut FragmentDataRareData {
        if self.rare_data_.Get().is_null() {
            self.rare_data_ =
                Member::from_ptr(MakeGarbageCollected(FragmentDataRareData::default()));
        }
        unsafe { &mut *self.rare_data_.Get() }
    }
}

// FragmentDataList stores its first item inline, as the C++ derived object
// does, while additional entries remain GC-owned through RareData.
// cpp: core/paint/fragment_data.h:247-254
#[repr(C)]
pub struct FragmentDataList {
    base_: FragmentData,
}

impl Default for FragmentDataList {
    fn default() -> Self {
        Self {
            base_: FragmentData::default(),
        }
    }
}

impl Deref for FragmentDataList {
    type Target = FragmentData;

    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

impl DerefMut for FragmentDataList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}

#[allow(non_snake_case)]
impl FragmentDataList {
    // cpp: core/paint/fragment_data.h:256
    // cpp: core/paint/fragment_data.cc:164-169
    pub fn AppendNewFragment(&mut self) -> &mut FragmentData {
        self.AssertIsFirst();
        let fragment = MakeGarbageCollected(FragmentData::default());
        self.EnsureRareData()
            .additional_fragments
            .push(Member::from_ptr(fragment));
        unsafe { &mut *fragment }
    }

    // cpp: core/paint/fragment_data.h:257
    // cpp: core/paint/fragment_data.cc:171-177
    pub fn Shrink(&mut self, new_size: u32) {
        if new_size < 1 || new_size > self.size() {
            std::process::abort();
        }
        let rare = self.rare_data_.Get();
        if !rare.is_null() {
            unsafe { &mut *rare }
                .additional_fragments
                .truncate((new_size - 1) as usize);
        }
    }

    // cpp: core/paint/fragment_data.h:259-262
    pub fn front_mut(&mut self) -> &mut FragmentData {
        self.AssertIsFirst();
        &mut self.base_
    }

    // cpp: core/paint/fragment_data.h:263-266
    pub fn front(&self) -> &FragmentData {
        self.AssertIsFirst();
        &self.base_
    }

    // cpp: core/paint/fragment_data.h:267
    // cpp: core/paint/fragment_data.cc:179-185
    pub fn back_mut(&mut self) -> &mut FragmentData {
        self.AssertIsFirst();
        let rare = self.rare_data_.Get();
        if !rare.is_null() {
            if let Some(last) = unsafe { &*rare }.additional_fragments.last() {
                return unsafe { &mut *last.Get() };
            }
        }
        &mut self.base_
    }

    // cpp: core/paint/fragment_data.h:268
    // cpp: core/paint/fragment_data.cc:187-189
    pub fn back(&self) -> &FragmentData {
        self.AssertIsFirst();
        let rare = self.rare_data_.Get();
        if !rare.is_null() {
            if let Some(last) = unsafe { &*rare }.additional_fragments.last() {
                return unsafe { &*last.Get() };
            }
        }
        &self.base_
    }

    // cpp: core/paint/fragment_data.h:269
    // cpp: core/paint/fragment_data.cc:191-198
    pub fn at_mut(&mut self, idx: u32) -> &mut FragmentData {
        self.AssertIsFirst();
        if idx == 0 {
            return &mut self.base_;
        }
        let rare = self.rare_data_.Get();
        if rare.is_null() {
            std::process::abort();
        }
        let member = unsafe { &*rare }
            .additional_fragments
            .get((idx - 1) as usize)
            .unwrap_or_else(|| std::process::abort());
        unsafe { &mut *member.Get() }
    }

    // cpp: core/paint/fragment_data.h:270
    // cpp: core/paint/fragment_data.cc:200-202
    pub fn at(&self, idx: u32) -> &FragmentData {
        self.AssertIsFirst();
        if idx == 0 {
            return &self.base_;
        }
        let rare = self.rare_data_.Get();
        if rare.is_null() {
            std::process::abort();
        }
        let member = unsafe { &*rare }
            .additional_fragments
            .get((idx - 1) as usize)
            .unwrap_or_else(|| std::process::abort());
        unsafe { &*member.Get() }
    }

    // cpp: core/paint/fragment_data.h:271
    // cpp: core/paint/fragment_data.cc:204-206
    pub fn size(&self) -> u32 {
        let rare = self.rare_data_.Get();
        if rare.is_null() {
            1
        } else {
            unsafe { &*rare }.additional_fragments.len() as u32 + 1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{LayoutUnit, PhysicalRect};
    use layoutng_geometry::geometry::axis::{
        kPhysicalAxesHorizontal, kPhysicalAxesVertical, PhysicalAxis,
    };

    fn axis_data(axis: PhysicalAxis) -> *mut PerAxisData {
        MakeGarbageCollected(PerAxisData::new(
            axis,
            &PhysicalRect::default(),
            &PhysicalRect::default(),
            &PhysicalRect::default(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            false,
            Some(LayoutUnit::default()),
            None,
            None,
            None,
        ))
    }

    #[test]
    fn sticky_constraints_live_on_the_first_fragment_and_clear_per_axis() {
        let mut list = FragmentDataList::default();
        #[cfg(debug_assertions)]
        list.SetIsFirst();
        let x_data = axis_data(PhysicalAxis::kHorizontal);
        let y_data = axis_data(PhysicalAxis::kVertical);

        assert!(list.SetStickyConstraints(StickyConstraintsData { x_data, y_data }));
        assert!(list.HasStickyConstraints());
        assert_eq!(
            list.StickyConstraints().AxisData(PhysicalAxis::kHorizontal),
            x_data
        );
        assert_eq!(
            list.StickyConstraints().AxisData(PhysicalAxis::kVertical),
            y_data
        );

        assert!(list.ClearStickyConstraints(kPhysicalAxesHorizontal));
        assert!(list
            .StickyConstraints()
            .AxisData(PhysicalAxis::kHorizontal)
            .is_null());
        assert!(!list
            .StickyConstraints()
            .AxisData(PhysicalAxis::kVertical)
            .is_null());
        assert!(list.ClearStickyConstraints(kPhysicalAxesVertical));
        assert!(!list.HasStickyConstraints());
    }
}
