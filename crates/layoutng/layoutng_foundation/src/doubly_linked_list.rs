#![allow(non_snake_case)]
// Retain intrusive links in the original owning node, as the C++ CRTP base
// does. This list never takes ownership of raw-pointer entries.
// cpp: foundation/blink_base/wtf/doubly_linked_list.h:46-84
pub trait DoublyLinkedListNode: Sized {
    fn SetPrev(&mut self, prev: *mut Self);
    fn SetNext(&mut self, next: *mut Self);
    fn Prev(&self) -> *mut Self;
    fn Next(&self) -> *mut Self;
}
// cpp: foundation/blink_base/wtf/doubly_linked_list.h:110-118
pub struct AddResult<T> {
    pub node: *mut T,
    pub is_new_entry: bool,
}
impl<T> AddResult<T> {
    pub fn IsNewEntry(&self) -> bool {
        self.is_new_entry
    }
}
// cpp: foundation/blink_base/wtf/doubly_linked_list.h:86-160
pub struct DoublyLinkedList<T: DoublyLinkedListNode> {
    head_: *mut T,
    tail_: *mut T,
}
impl<T: DoublyLinkedListNode> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self {
            head_: std::ptr::null_mut(),
            tail_: std::ptr::null_mut(),
        }
    }
}
impl<T: DoublyLinkedListNode> DoublyLinkedList<T> {
    // cpp: foundation/blink_base/wtf/doubly_linked_list.h:166-197
    pub fn empty(&self) -> bool {
        self.head_.is_null()
    }
    pub fn size(&self) -> u32 {
        let mut size = 0u32;
        let mut node = self.head_;
        while !node.is_null() {
            size = size.wrapping_add(1);
            node = unsafe { &*node }.Next();
        }
        size
    }
    pub fn Clear(&mut self) {
        self.head_ = std::ptr::null_mut();
        self.tail_ = std::ptr::null_mut();
    }
    pub fn Head(&self) -> *mut T {
        self.head_
    }
    pub fn Tail(&self) -> *mut T {
        self.tail_
    }
    // cpp: foundation/blink_base/wtf/doubly_linked_list.h:199-215
    pub unsafe fn Push(&mut self, node: *mut T) {
        if self.head_.is_null() {
            debug_assert!(self.tail_.is_null());
            self.head_ = node;
            self.tail_ = node;
            (*node).SetPrev(std::ptr::null_mut());
            (*node).SetNext(std::ptr::null_mut());
            return;
        }
        debug_assert!(!self.tail_.is_null());
        (*self.head_).SetPrev(node);
        (*node).SetNext(self.head_);
        (*node).SetPrev(std::ptr::null_mut());
        self.head_ = node;
    }
    // cpp: foundation/blink_base/wtf/doubly_linked_list.h:217-233
    pub unsafe fn Append(&mut self, node: *mut T) {
        if self.tail_.is_null() {
            debug_assert!(self.head_.is_null());
            self.head_ = node;
            self.tail_ = node;
            (*node).SetPrev(std::ptr::null_mut());
            (*node).SetNext(std::ptr::null_mut());
            return;
        }
        debug_assert!(!self.head_.is_null());
        (*self.tail_).SetNext(node);
        (*node).SetPrev(self.tail_);
        (*node).SetNext(std::ptr::null_mut());
        self.tail_ = node;
    }
    // cpp: foundation/blink_base/wtf/doubly_linked_list.h:235-252
    pub unsafe fn Remove(&mut self, node: *mut T) {
        let prev = (*node).Prev();
        let next = (*node).Next();
        if !prev.is_null() {
            debug_assert_ne!(node, self.head_);
            (*prev).SetNext(next);
        } else {
            debug_assert_eq!(node, self.head_);
            self.head_ = next;
        }
        if !next.is_null() {
            debug_assert_ne!(node, self.tail_);
            (*next).SetPrev(prev);
        } else {
            debug_assert_eq!(node, self.tail_);
            self.tail_ = prev;
        }
    }
    // cpp: foundation/blink_base/wtf/doubly_linked_list.h:254-261
    pub unsafe fn RemoveHead(&mut self) -> *mut T {
        let node = self.Head();
        if !node.is_null() {
            self.Remove(node);
        }
        node
    }
    // cpp: foundation/blink_base/wtf/doubly_linked_list.h:263-275
    pub unsafe fn InsertOwned(
        &mut self,
        mut node: Box<T>,
        compare: impl Fn(*mut T, *mut T) -> i32,
    ) -> AddResult<T> {
        let result = self.Insert(&mut *node, compare);
        if result.is_new_entry {
            let _ = Box::into_raw(node);
        }
        result
    }
    // cpp: foundation/blink_base/wtf/doubly_linked_list.h:277-289
    pub unsafe fn Insert(
        &mut self,
        node: *mut T,
        compare: impl Fn(*mut T, *mut T) -> i32,
    ) -> AddResult<T> {
        debug_assert!(!node.is_null());
        let mut iter = self.head_;
        while !iter.is_null() && compare(iter, node) < 0 {
            iter = (*iter).Next();
        }
        if !iter.is_null() && compare(iter, node) == 0 {
            return AddResult {
                node: iter,
                is_new_entry: false,
            };
        }
        self.InsertAfter(
            node,
            if !iter.is_null() {
                (*iter).Prev()
            } else {
                self.tail_
            },
        )
    }
    // cpp: foundation/blink_base/wtf/doubly_linked_list.h:291-301
    pub unsafe fn InsertAfterOwned(&mut self, mut node: Box<T>, point: *mut T) -> AddResult<T> {
        let result = self.InsertAfter(&mut *node, point);
        if result.is_new_entry {
            let _ = Box::into_raw(node);
        }
        result
    }
    // cpp: foundation/blink_base/wtf/doubly_linked_list.h:279-300
    pub unsafe fn InsertAfter(&mut self, node: *mut T, point: *mut T) -> AddResult<T> {
        debug_assert!(!node.is_null());
        if point.is_null() {
            self.Push(node);
            return AddResult {
                node: self.head_,
                is_new_entry: true,
            };
        }
        let next = (*point).Next();
        (*node).SetNext(next);
        if !next.is_null() {
            (*next).SetPrev(node);
        }
        (*node).SetPrev(point);
        (*point).SetNext(node);
        if point == self.tail_ {
            self.tail_ = node;
        }
        AddResult {
            node,
            is_new_entry: true,
        }
    }
}
