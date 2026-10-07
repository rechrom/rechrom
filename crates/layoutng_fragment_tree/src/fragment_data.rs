use std::ops::{Deref, DerefMut};

use layoutng::internal::paint_layer::PaintLayer;

use foundation::{
    HeapVector, MakeGarbageCollected, Member, NewUniqueObjectId, PhysicalOffset, UniqueObjectId,
    Visitor,
};

// C++ nests RareData under FragmentData. Rust keeps it beside the owner so
// the list and GC tracing code can address it without inheritance.
// cpp: layoutng_fragment_tree/fragment_data.h:45-55
pub(crate) struct FragmentDataRareData {
    pub(crate) additional_fragments: HeapVector<Member<FragmentData>>,
    pub(crate) unique_id: UniqueObjectId,
    // cpp: core/paint/fragment_data.h:221
    pub(crate) layer: Member<PaintLayer>,
}

impl Default for FragmentDataRareData {
    // cpp: layoutng_fragment_tree/fragment_data.h:47-50
    // cpp: layoutng_fragment_tree/fragment_data.cc:8-9
    fn default() -> Self {
        Self {
            additional_fragments: HeapVector::default(),
            unique_id: 0,
            layer: Member::default(),
        }
    }
}

#[allow(non_snake_case)]
impl FragmentDataRareData {
    // cpp: layoutng_fragment_tree/fragment_data.h:51
    // cpp: layoutng_fragment_tree/fragment_data.cc:14-18
    pub(crate) fn EnsureId(&mut self) {
        if self.unique_id == 0 {
            self.unique_id = NewUniqueObjectId();
        }
    }

    // cpp: layoutng_fragment_tree/fragment_data.h:52
    // cpp: layoutng_fragment_tree/fragment_data.cc:10-12
    pub(crate) fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.layer);
        visitor.Trace(&self.additional_fragments);
    }
}

// cpp: layoutng_fragment_tree/fragment_data.h:21
// cpp: layoutng_fragment_tree/fragment_data.h:57-61
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
        if rare.is_null() { std::ptr::null_mut() } else { unsafe { &*rare }.layer.Get() }
    }

    // cpp: core/paint/fragment_data.cc:25-32,49-53
    pub fn SetLayer(&mut self, layer: *mut PaintLayer) {
        self.AssertIsFirst();
        if self.rare_data_.Get().is_null() && layer.is_null() { return; }
        let rare = self.EnsureRareData();
        let previous = rare.layer.Get();
        if !previous.is_null() && previous != layer {
            unsafe { &mut *previous }.Destroy();
        }
        rare.layer = Member::from_ptr(layer);
    }

    // cpp: layoutng_fragment_tree/fragment_data.h:23
    pub fn PaintOffset(&self) -> PhysicalOffset {
        self.paint_offset_
    }

    // cpp: layoutng_fragment_tree/fragment_data.h:24-26
    pub fn SetPaintOffset(&mut self, paint_offset: &PhysicalOffset) {
        self.paint_offset_ = *paint_offset;
    }

    // cpp: layoutng_fragment_tree/fragment_data.h:27-30
    pub fn UniqueId(&self) -> UniqueObjectId {
        let rare = self.rare_data_.Get();
        debug_assert!(!rare.is_null());
        unsafe { &*rare }.unique_id
    }

    // cpp: layoutng_fragment_tree/fragment_data.h:31
    pub fn EnsureId(&mut self) {
        self.EnsureRareData().EnsureId();
    }

    // cpp: layoutng_fragment_tree/fragment_data.h:32
    pub fn HasUniqueId(&self) -> bool {
        let rare = self.rare_data_.Get();
        !rare.is_null() && unsafe { &*rare }.unique_id != 0
    }

    // cpp: layoutng_fragment_tree/fragment_data.h:33-35
    #[cfg(debug_assertions)]
    pub fn SetIsFirst(&mut self) {
        self.is_first_ = true;
    }

    // C++ has a default destructor; Rust field ownership supplies it.
    // cpp: layoutng_fragment_tree/fragment_data.h:36

    // cpp: layoutng_fragment_tree/fragment_data.h:37
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.rare_data_);
    }

    // cpp: layoutng_fragment_tree/fragment_data.h:40-44
    pub(crate) fn AssertIsFirst(&self) {
        #[cfg(debug_assertions)]
        debug_assert!(self.is_first_);
    }

    // cpp: layoutng_fragment_tree/fragment_data.h:56
    // cpp: layoutng_fragment_tree/fragment_data.cc:20-24
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
// cpp: layoutng_fragment_tree/fragment_data.h:64-72
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
    // cpp: layoutng_fragment_tree/fragment_data.h:73
    // cpp: layoutng_fragment_tree/fragment_data.cc:26-31
    pub fn AppendNewFragment(&mut self) -> &mut FragmentData {
        self.AssertIsFirst();
        let fragment = MakeGarbageCollected(FragmentData::default());
        self.EnsureRareData()
            .additional_fragments
            .push(Member::from_ptr(fragment));
        unsafe { &mut *fragment }
    }

    // cpp: layoutng_fragment_tree/fragment_data.h:74
    // cpp: layoutng_fragment_tree/fragment_data.cc:33-39
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

    // cpp: layoutng_fragment_tree/fragment_data.h:76-79
    pub fn front_mut(&mut self) -> &mut FragmentData {
        self.AssertIsFirst();
        &mut self.base_
    }

    // cpp: layoutng_fragment_tree/fragment_data.h:80-83
    pub fn front(&self) -> &FragmentData {
        self.AssertIsFirst();
        &self.base_
    }

    // cpp: layoutng_fragment_tree/fragment_data.h:84
    // cpp: layoutng_fragment_tree/fragment_data.cc:41-47
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

    // cpp: layoutng_fragment_tree/fragment_data.h:85
    // cpp: layoutng_fragment_tree/fragment_data.cc:49-51
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

    // cpp: layoutng_fragment_tree/fragment_data.h:86
    // cpp: layoutng_fragment_tree/fragment_data.cc:53-60
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

    // cpp: layoutng_fragment_tree/fragment_data.h:87
    // cpp: layoutng_fragment_tree/fragment_data.cc:62-64
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

    // cpp: layoutng_fragment_tree/fragment_data.h:88
    // cpp: layoutng_fragment_tree/fragment_data.cc:66-68
    pub fn size(&self) -> u32 {
        let rare = self.rare_data_.Get();
        if rare.is_null() {
            1
        } else {
            unsafe { &*rare }.additional_fragments.len() as u32 + 1
        }
    }
}
