#![allow(non_snake_case)]

use foundation::{GCedHeapHashMap, HeapHashMap, MakeGarbageCollected, Member};

use super::layout_object::LayoutObject;

// C++ Member<const LayoutObject> retains traced identity; Rust's Member<T>
// handle does not encode C++ constness, so access remains const below.
// cpp: layoutng/internal/layout_object.h:438-440
pub type LayoutObjectIndexMap = GCedHeapHashMap<Member<LayoutObject>, u32>;
pub type IndexCache = HeapHashMap<Member<LayoutObject>, Member<LayoutObjectIndexMap>>;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object_tree.cc:33-41
    pub fn IsDescendantOf(&self, obj: *const LayoutObject) -> bool {
        self.CheckIsNotDestroyed();
        let mut current: *const LayoutObject = self;
        while !current.is_null() {
            if current == obj {
                return true;
            }
            current = unsafe { &*current }.Parent();
        }
        false
    }

    // cpp: layoutng/internal/layout_object_tree.cc:43-50
    pub fn NextInPreOrderDefault(&self) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        let first = self.SlowFirstChild();
        if !first.is_null() {
            return first;
        }
        self.NextInPreOrderAfterChildrenDefault()
    }

    // cpp: layoutng/internal/layout_object_tree.cc:52-66
    pub fn NextInPreOrderAfterChildrenDefault(&self) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        let mut next = self.NextSibling();
        if next.is_null() {
            next = self.Parent();
            while !next.is_null() && unsafe { &*next }.NextSibling().is_null() {
                next = unsafe { &*next }.Parent();
            }
            if !next.is_null() {
                next = unsafe { &*next }.NextSibling();
            }
        }
        next
    }

    // cpp: layoutng/internal/layout_object_tree.cc:68-76
    pub fn NextInPreOrder(&self, stay_within: *const LayoutObject) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        let first = self.SlowFirstChild();
        if !first.is_null() {
            return first;
        }
        self.NextInPreOrderAfterChildren(stay_within)
    }

    // cpp: layoutng/internal/layout_object_tree.cc:78-86
    pub fn PreviousInPostOrder(&self, stay_within: *const LayoutObject) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        let last = self.SlowLastChild();
        if !last.is_null() {
            return last;
        }
        self.PreviousInPostOrderBeforeChildren(stay_within)
    }

    // cpp: layoutng/internal/layout_object_tree.cc:88-104
    pub fn NextInPreOrderAfterChildren(
        &self,
        stay_within: *const LayoutObject,
    ) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        if std::ptr::eq(self, stay_within) {
            return std::ptr::null_mut();
        }
        let mut current: *const LayoutObject = self;
        let mut next = self.NextSibling();
        while next.is_null() {
            current = unsafe { &*current }.Parent();
            if current.is_null() || current == stay_within {
                return std::ptr::null_mut();
            }
            next = unsafe { &*current }.NextSibling();
        }
        next
    }

    // cpp: layoutng/internal/layout_object_tree.cc:106-122
    pub fn PreviousInPostOrderBeforeChildren(
        &self,
        stay_within: *const LayoutObject,
    ) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        if std::ptr::eq(self, stay_within) {
            return std::ptr::null_mut();
        }
        let mut current: *const LayoutObject = self;
        let mut previous = self.PreviousSibling();
        while previous.is_null() {
            current = unsafe { &*current }.Parent();
            if current.is_null() || current == stay_within {
                return std::ptr::null_mut();
            }
            previous = unsafe { &*current }.PreviousSibling();
        }
        previous
    }

    // cpp: layoutng/internal/layout_object_tree.cc:124-134
    pub fn PreviousInPreOrderDefault(&self) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        let mut previous = self.PreviousSibling();
        if !previous.is_null() {
            loop {
                let last = unsafe { &*previous }.SlowLastChild();
                if last.is_null() {
                    break;
                }
                previous = last;
            }
            return previous;
        }
        self.Parent()
    }

    // cpp: layoutng/internal/layout_object_tree.cc:136-144
    pub fn PreviousInPreOrder(&self, stay_within: *const LayoutObject) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        if std::ptr::eq(self, stay_within) {
            return std::ptr::null_mut();
        }
        self.PreviousInPreOrderDefault()
    }
}

// C++ STACK_ALLOCATED only restricts allocation; this is an ordinary local
// value in Rust with the same three pointer fields.
// cpp: layoutng/internal/layout_object_tree.cc:184-192
struct CommonAncestorResult {
    common_ancestor: *const LayoutObject,
    last: *const LayoutObject,
    other_last: *const LayoutObject,
}

// cpp: layoutng/internal/layout_object_tree.cc:194-220
fn CommonAncestorInternal(
    mut object: *const LayoutObject,
    mut other_object: *const LayoutObject,
) -> CommonAncestorResult {
    let mut depth = unsafe { &*object }.Depth();
    let mut other_depth = unsafe { &*other_object }.Depth();
    while depth > other_depth {
        object = unsafe { &*object }.Parent();
        depth -= 1;
    }
    while other_depth > depth {
        other_object = unsafe { &*other_object }.Parent();
        other_depth -= 1;
    }
    let mut last = std::ptr::null();
    let mut other_last = std::ptr::null();
    while object != other_object {
        last = object;
        other_last = other_object;
        object = unsafe { &*object }.Parent();
        other_object = unsafe { &*other_object }.Parent();
    }
    CommonAncestorResult {
        common_ancestor: object,
        last,
        other_last,
    }
}

impl LayoutObject {
    // cpp: layoutng/internal/layout_object_tree.cc:224-228
    pub fn CommonAncestor(&self, other: &LayoutObject) -> *const LayoutObject {
        self.CheckIsNotDestroyed();
        CommonAncestorInternal(self, other).common_ancestor
    }

    // cpp: layoutng/internal/layout_object.h:441-442
    pub fn IsBeforeInPreOrderDefault(&self, other: &LayoutObject) -> bool {
        self.IsBeforeInPreOrder(other, std::ptr::null_mut())
    }

    // cpp: layoutng/internal/layout_object_tree.cc:230-285
    pub fn IsBeforeInPreOrder(&self, other: &LayoutObject, index_cache: *mut IndexCache) -> bool {
        self.CheckIsNotDestroyed();
        debug_assert!(!std::ptr::eq(self, other));
        let result = CommonAncestorInternal(self, other);
        if std::ptr::eq(self, result.common_ancestor) {
            return true;
        }
        if std::ptr::eq(other, result.common_ancestor) {
            return false;
        }
        debug_assert!(!result.last.is_null());
        debug_assert!(!result.other_last.is_null());

        if !index_cache.is_null() {
            let ancestor_key = Member::from_ptr(result.common_ancestor as *mut LayoutObject);
            let is_new_entry = unsafe { &mut *index_cache }
                .insert(ancestor_key, Member::<LayoutObjectIndexMap>::default());
            if is_new_entry {
                let index_map = MakeGarbageCollected(LayoutObjectIndexMap::default());
                let mut index = 0u32;
                let mut child = unsafe { &*result.common_ancestor }.SlowFirstChild();
                while !child.is_null() {
                    unsafe { &mut *index_map }.insert(Member::from_ptr(child), index);
                    index = index.wrapping_add(1);
                    child = unsafe { &*child }.NextSibling();
                }
                unsafe { &mut *index_cache }
                    .entry(ancestor_key)
                    .and_modify(|value| *value = Member::from_ptr(index_map));
            }
            let index_map = unsafe { &*index_cache }.get(&ancestor_key).unwrap().Get();
            let first = unsafe { &*index_map }
                .get(&Member::from_ptr(result.last as *mut LayoutObject))
                .unwrap();
            let second = unsafe { &*index_map }
                .get(&Member::from_ptr(result.other_last as *mut LayoutObject))
                .unwrap();
            return first < second;
        }

        let mut forward = result.last;
        let mut backward = result.other_last;
        while !forward.is_null() && !backward.is_null() {
            forward = unsafe { &*forward }.NextSibling();
            if !forward.is_null() && forward == backward {
                return true;
            }
            backward = unsafe { &*backward }.PreviousSibling();
            if !forward.is_null() && forward == backward {
                return true;
            }
        }
        false
    }
}
