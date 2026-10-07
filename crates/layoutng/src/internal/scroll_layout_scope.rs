#![allow(non_snake_case)]

use std::cell::{Cell, OnceCell};
use std::marker::PhantomData;
use std::rc::Rc;

use foundation::{DisallowNewWrapper, HeapVector, MakeGarbageCollected, Member, Persistent};

use super::layout_box::LayoutBox;

// The concrete scroll host owns the GC object and supplies virtual dispatch.
// cpp: layoutng/internal/scroll_layout_scope.h:57-65
#[repr(C)]
pub struct ScrollOffsetClampTarget {
    _private: (),
}

impl ScrollOffsetClampTarget {
    pub fn NeedsScrollOffsetClamp(&self) -> bool {
        unsafe { ScrollOffsetClampTargetNeedsScrollOffsetClamp(self) }
    }

    pub fn SetNeedsScrollOffsetClamp(&mut self, value: bool) {
        unsafe { ScrollOffsetClampTargetSetNeedsScrollOffsetClamp(self, value) }
    }

    pub fn ClampScrollOffsetAfterOverflowChange(&mut self) {
        unsafe { ScrollOffsetClampTargetClampScrollOffsetAfterOverflowChange(self) }
    }
}

unsafe extern "Rust" {
    fn ScrollOffsetClampTargetNeedsScrollOffsetClamp(target: &ScrollOffsetClampTarget) -> bool;
    fn ScrollOffsetClampTargetSetNeedsScrollOffsetClamp(
        target: &mut ScrollOffsetClampTarget,
        value: bool,
    );
    fn ScrollOffsetClampTargetClampScrollOffsetAfterOverflowChange(
        target: &mut ScrollOffsetClampTarget,
    );
}

// cpp: layoutng/internal/scroll_layout_scope.h:78-79
// cpp: layoutng/internal/scroll_layout_scope.cc:53
thread_local! {
    static FREEZE_COUNT: Cell<i32> = const { Cell::new(0) };
}

// cpp: layoutng/internal/scroll_layout_scope.h:67-80
pub struct FreezeScrollbarsScope {
    _thread_bound: PhantomData<Rc<()>>,
}

impl FreezeScrollbarsScope {
    // cpp: layoutng/internal/scroll_layout_scope.h:71
    pub fn new() -> Self {
        FREEZE_COUNT.with(|count| count.set(count.get() + 1));
        Self {
            _thread_bound: PhantomData,
        }
    }

    // cpp: layoutng/internal/scroll_layout_scope.h:76
    pub fn ScrollbarsAreFrozen() -> bool {
        FREEZE_COUNT.with(|count| count.get() != 0)
    }
}

impl Default for FreezeScrollbarsScope {
    fn default() -> Self {
        Self::new()
    }
}

// cpp: layoutng/internal/scroll_layout_scope.h:72
impl Drop for FreezeScrollbarsScope {
    fn drop(&mut self) {
        FREEZE_COUNT.with(|count| {
            debug_assert!(count.get() > 0);
            count.set(count.get() - 1);
        });
    }
}

// cpp: layoutng/internal/scroll_layout_scope.h:82-103
#[derive(Default)]
pub struct ScrollbarFreezeState {
    is_scrollbar_freeze_root_: bool,
    is_horizontal_scrollbar_frozen_: bool,
    is_vertical_scrollbar_frozen_: bool,
}

impl ScrollbarFreezeState {
    // cpp: layoutng/internal/scroll_layout_scope.h:87
    // cpp: layoutng/internal/scroll_layout_scope.cc:74-80
    pub fn EstablishScrollbarRoot(&mut self, freeze_horizontal: bool, freeze_vertical: bool) {
        debug_assert!(!FreezeScrollbarsScope::ScrollbarsAreFrozen());
        self.is_scrollbar_freeze_root_ = true;
        self.is_horizontal_scrollbar_frozen_ = freeze_horizontal;
        self.is_vertical_scrollbar_frozen_ = freeze_vertical;
    }

    // cpp: layoutng/internal/scroll_layout_scope.h:88
    // cpp: layoutng/internal/scroll_layout_scope.cc:82-86
    pub fn ClearScrollbarRoot(&mut self) {
        self.is_scrollbar_freeze_root_ = false;
        self.is_horizontal_scrollbar_frozen_ = false;
        self.is_vertical_scrollbar_frozen_ = false;
    }

    // cpp: layoutng/internal/scroll_layout_scope.h:89-93
    pub fn IsHorizontalScrollbarFrozen(&self) -> bool {
        if self.is_scrollbar_freeze_root_ {
            self.is_horizontal_scrollbar_frozen_
        } else {
            FreezeScrollbarsScope::ScrollbarsAreFrozen()
        }
    }

    // cpp: layoutng/internal/scroll_layout_scope.h:94-98
    pub fn IsVerticalScrollbarFrozen(&self) -> bool {
        if self.is_scrollbar_freeze_root_ {
            self.is_vertical_scrollbar_frozen_
        } else {
            FreezeScrollbarsScope::ScrollbarsAreFrozen()
        }
    }
}

// cpp: layoutng/internal/scroll_layout_scope.h:105-118
pub struct FreezeScrollbarsRootScope {
    scrollable_area_: *mut ScrollbarFreezeState,
    freezer_: Option<FreezeScrollbarsScope>,
}

impl FreezeScrollbarsRootScope {
    // cpp: layoutng/internal/scroll_layout_scope.h:108-109
    // cpp: layoutng/internal/layout_node_data.cc:62-65
    pub fn from_box(box_: &LayoutBox, freeze_horizontal: bool, freeze_vertical: bool) -> Self {
        Self::new(
            box_.GetScrollbarFreezeState(),
            freeze_horizontal,
            freeze_vertical,
        )
    }

    // cpp: layoutng/internal/scroll_layout_scope.h:110-111
    // cpp: layoutng/internal/scroll_layout_scope.cc:55-66
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
// cpp: layoutng/internal/scroll_layout_scope.cc:68-72
impl Drop for FreezeScrollbarsRootScope {
    fn drop(&mut self) {
        if !self.scrollable_area_.is_null() {
            unsafe { &mut *self.scrollable_area_ }.ClearScrollbarRoot();
        }
    }
}

type ClampList = HeapVector<Member<ScrollOffsetClampTarget>>;
type ClampListHolder = DisallowNewWrapper<ClampList>;

// cpp: layoutng/internal/scroll_layout_scope.h:138
// cpp: layoutng/internal/scroll_layout_scope.cc:88
thread_local! {
    static DELAY_COUNT: Cell<i32> = const { Cell::new(0) };
    // cpp: layoutng/internal/scroll_layout_scope.cc:117-124
    static NEEDS_CLAMP_LIST: OnceCell<Persistent<ClampListHolder>> = const { OnceCell::new() };
}

// cpp: layoutng/internal/scroll_layout_scope.h:120-139
pub struct DelayScrollOffsetClampScope {
    _thread_bound: PhantomData<Rc<()>>,
}

impl DelayScrollOffsetClampScope {
    // cpp: layoutng/internal/scroll_layout_scope.cc:90-94
    pub fn new() -> Self {
        let count = DELAY_COUNT.with(Cell::get);
        debug_assert!(count > 0 || unsafe { &*Self::NeedsClampList() }.is_empty());
        DELAY_COUNT.with(|count| count.set(count.get() + 1));
        Self {
            _thread_bound: PhantomData,
        }
    }

    // cpp: layoutng/internal/scroll_layout_scope.h:132
    pub fn ClampingIsDelayed() -> bool {
        DELAY_COUNT.with(|count| count.get() != 0)
    }

    // cpp: layoutng/internal/scroll_layout_scope.h:133
    // cpp: layoutng/internal/scroll_layout_scope.cc:102-108
    pub fn SetNeedsClamp(scrollable_area: *mut ScrollOffsetClampTarget) {
        assert!(!scrollable_area.is_null());
        if !unsafe { &*scrollable_area }.NeedsScrollOffsetClamp() {
            unsafe { &mut *scrollable_area }.SetNeedsScrollOffsetClamp(true);
            unsafe { &mut *Self::NeedsClampList() }.push_back(Member::from_ptr(scrollable_area));
        }
    }

    // cpp: layoutng/internal/scroll_layout_scope.cc:110-115
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
    // cpp: layoutng/internal/scroll_layout_scope.cc:117-124
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

// cpp: layoutng/internal/scroll_layout_scope.cc:96-100
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
