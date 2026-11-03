#![allow(non_snake_case)]

use std::cell::{Cell, OnceCell};
use std::marker::PhantomData;
use std::rc::Rc;

use foundation::{DisallowNewWrapper, HeapVector, MakeGarbageCollected, Member, Persistent};

use super::super::layout_box::LayoutBox;

// The delayed queue roots the real source owner. No external placeholder
// dispatch is needed now that scrollable areas live in this same package.
pub type ScrollOffsetClampTarget = super::super::layout_scrollable_area::PaintLayerScrollableArea;

// Source: core/paint/paint_layer_scrollable_area.h:217-228.
// Source: core/paint/paint_layer_scrollable_area.cc:3039.
thread_local! {
    static FREEZE_COUNT: Cell<i32> = const { Cell::new(0) };
}

// Source: core/paint/paint_layer_scrollable_area.h:217-228.
pub struct FreezeScrollbarsScope {
    _thread_bound: PhantomData<Rc<()>>,
}

impl FreezeScrollbarsScope {
    // Source: core/paint/paint_layer_scrollable_area.h:221.
    pub fn new() -> Self {
        FREEZE_COUNT.with(|count| count.set(count.get() + 1));
        Self {
            _thread_bound: PhantomData,
        }
    }

    // Source: core/paint/paint_layer_scrollable_area.h:224.
    pub fn ScrollbarsAreFrozen() -> bool {
        FREEZE_COUNT.with(|count| count.get() != 0)
    }
}

impl Default for FreezeScrollbarsScope {
    fn default() -> Self {
        Self::new()
    }
}

// Source: core/paint/paint_layer_scrollable_area.h:222.
impl Drop for FreezeScrollbarsScope {
    fn drop(&mut self) {
        FREEZE_COUNT.with(|count| {
            debug_assert!(count.get() > 0);
            count.set(count.get() - 1);
        });
    }
}

// Source: core/paint/paint_layer_scrollable_area.h:641-652,861-863.
#[derive(Default)]
pub struct ScrollbarFreezeState {
    is_scrollbar_freeze_root_: bool,
    is_horizontal_scrollbar_frozen_: bool,
    is_vertical_scrollbar_frozen_: bool,
}

impl ScrollbarFreezeState {
    // Source: core/paint/paint_layer_scrollable_area.cc:3183-3189.
    pub fn EstablishScrollbarRoot(&mut self, freeze_horizontal: bool, freeze_vertical: bool) {
        debug_assert!(!FreezeScrollbarsScope::ScrollbarsAreFrozen());
        self.is_scrollbar_freeze_root_ = true;
        self.is_horizontal_scrollbar_frozen_ = freeze_horizontal;
        self.is_vertical_scrollbar_frozen_ = freeze_vertical;
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:3191-3195.
    pub fn ClearScrollbarRoot(&mut self) {
        self.is_scrollbar_freeze_root_ = false;
        self.is_horizontal_scrollbar_frozen_ = false;
        self.is_vertical_scrollbar_frozen_ = false;
    }

    // Source: core/paint/paint_layer_scrollable_area.h:643-647.
    pub fn IsHorizontalScrollbarFrozen(&self) -> bool {
        if self.is_scrollbar_freeze_root_ {
            self.is_horizontal_scrollbar_frozen_
        } else {
            FreezeScrollbarsScope::ScrollbarsAreFrozen()
        }
    }

    // Source: core/paint/paint_layer_scrollable_area.h:648-652.
    pub fn IsVerticalScrollbarFrozen(&self) -> bool {
        if self.is_scrollbar_freeze_root_ {
            self.is_vertical_scrollbar_frozen_
        } else {
            FreezeScrollbarsScope::ScrollbarsAreFrozen()
        }
    }
}

// Source: core/paint/paint_layer_scrollable_area.h:233-251.
pub struct FreezeScrollbarsRootScope {
    scrollable_area_: *mut ScrollbarFreezeState,
    freezer_: Option<FreezeScrollbarsScope>,
}

impl FreezeScrollbarsRootScope {
    // Source: core/paint/paint_layer_scrollable_area.h:243-246.
    pub fn from_box(box_: &LayoutBox, freeze_horizontal: bool, freeze_vertical: bool) -> Self {
        Self::new(
            box_.GetScrollbarFreezeState(),
            freeze_horizontal,
            freeze_vertical,
        )
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:3041-3052.
    pub fn new(
        state: *mut ScrollbarFreezeState,
        freeze_horizontal: bool,
        freeze_vertical: bool,
    ) -> Self {
        let mut scope = Self {
            scrollable_area_: state,
            freezer_: None,
        };
        if !state.is_null()
            && !FreezeScrollbarsScope::ScrollbarsAreFrozen()
            && (freeze_horizontal || freeze_vertical)
        {
            unsafe { &mut *state }.EstablishScrollbarRoot(freeze_horizontal, freeze_vertical);
            scope.freezer_ = Some(FreezeScrollbarsScope::new());
        }
        scope
    }
}

// The body clears the root before the optional freezer field is dropped.
// Source: core/paint/paint_layer_scrollable_area.cc:3054-3058.
impl Drop for FreezeScrollbarsRootScope {
    fn drop(&mut self) {
        if !self.scrollable_area_.is_null() {
            unsafe { &mut *self.scrollable_area_ }.ClearScrollbarRoot();
        }
    }
}

type ClampList = HeapVector<Member<ScrollOffsetClampTarget>>;
type ClampListHolder = DisallowNewWrapper<ClampList>;

// Source: core/paint/paint_layer_scrollable_area.h:258-273.
// Source: core/paint/paint_layer_scrollable_area.cc:3060.
thread_local! {
    static DELAY_COUNT: Cell<i32> = const { Cell::new(0) };
    // Source: core/paint/paint_layer_scrollable_area.cc:3089-3096.
    static NEEDS_CLAMP_LIST: OnceCell<Persistent<ClampListHolder>> = const { OnceCell::new() };
}

// Source: core/paint/paint_layer_scrollable_area.h:258-273.
pub struct DelayScrollOffsetClampScope {
    _thread_bound: PhantomData<Rc<()>>,
}

impl DelayScrollOffsetClampScope {
    // Source: core/paint/paint_layer_scrollable_area.cc:3062-3066.
    pub fn new() -> Self {
        let count = DELAY_COUNT.with(Cell::get);
        debug_assert!(count > 0 || unsafe { &*Self::NeedsClampList() }.is_empty());
        DELAY_COUNT.with(|count| count.set(count.get() + 1));
        Self {
            _thread_bound: PhantomData,
        }
    }

    // Source: core/paint/paint_layer_scrollable_area.h:265.
    pub fn ClampingIsDelayed() -> bool {
        DELAY_COUNT.with(|count| count.get() != 0)
    }

    // Source: core/paint/paint_layer_scrollable_area.h:266.
    // Source: core/paint/paint_layer_scrollable_area.cc:3074-3080.
    pub fn SetNeedsClamp(scrollable_area: *mut ScrollOffsetClampTarget) {
        assert!(!scrollable_area.is_null());
        if !unsafe { &*scrollable_area }.NeedsScrollOffsetClamp() {
            unsafe { &mut *scrollable_area }.SetNeedsScrollOffsetClamp(true);
            unsafe { &mut *Self::NeedsClampList() }.push_back(Member::from_ptr(scrollable_area));
        }
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:3082-3087.
    #[inline(never)]
    fn ClampScrollableAreas() {
        let list = Self::NeedsClampList();
        let len = unsafe { &*list }.len();
        for index in 0..len {
            let target = unsafe { (&*list)[index].Get() };
            unsafe { &mut *target }.ClampScrollOffsetAfterOverflowChange();
        }
        unsafe { &mut *list }.clear();
    }

    // The persistent wrapper keeps every Member in this thread-local queue
    // reachable by the collector until the outermost scope flushes it.
    // Source: core/paint/paint_layer_scrollable_area.cc:3089-3096.
    fn NeedsClampList() -> *mut ClampList {
        NEEDS_CLAMP_LIST.with(|slot| {
            let root = slot.get_or_init(|| {
                Persistent::from_ptr(MakeGarbageCollected(ClampListHolder::new(
                    ClampList::default(),
                )))
            });
            unsafe { &mut *root.Get() }.Value() as *mut ClampList
        })
    }
}

impl Default for DelayScrollOffsetClampScope {
    fn default() -> Self {
        Self::new()
    }
}

// Source: core/paint/paint_layer_scrollable_area.cc:3068-3072.
impl Drop for DelayScrollOffsetClampScope {
    fn drop(&mut self) {
        let remaining = DELAY_COUNT.with(|count| {
            debug_assert!(count.get() > 0);
            let remaining = count.get() - 1;
            count.set(remaining);
            remaining
        });
        if remaining == 0 {
            Self::ClampScrollableAreas();
        }
    }
}
