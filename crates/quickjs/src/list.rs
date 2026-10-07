//! list.h — intrusive circular doubly linked lists.
//! Copyright (c) 2016-2017 Fabrice Bellard. MIT; see ../LICENSE.
//!
//! The caller keeps each node at a stable address and keeps the containing
//! objects alive until they are unlinked. These are the original C preconditions.
use core::ptr;

#[repr(C)]
pub struct list_head {
    pub prev: *mut list_head,
    pub next: *mut list_head,
}

impl Default for list_head {
    fn default() -> Self {
        Self {
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }
}

/// LIST_HEAD_INIT requires an already addressable Rust allocation, since
/// moving a self-referential value would invalidate its links.
#[macro_export]
macro_rules! LIST_HEAD_INIT {
    ($el:expr) => {
        $crate::list::init_list_head($el)
    };
}

// c: list.h:init_list_head
pub unsafe fn init_list_head(head: *mut list_head) {
    (*head).prev = head;
    (*head).next = head;
}

// c: list.h:__list_add
pub unsafe fn __list_add(el: *mut list_head, prev: *mut list_head, next: *mut list_head) {
    (*prev).next = el;
    (*el).prev = prev;
    (*el).next = next;
    (*next).prev = el;
}

// c: list.h:list_add
pub unsafe fn list_add(el: *mut list_head, head: *mut list_head) {
    __list_add(el, head, (*head).next);
}

// c: list.h:list_add_tail
pub unsafe fn list_add_tail(el: *mut list_head, head: *mut list_head) {
    __list_add(el, (*head).prev, head);
}

// c: list.h:list_del
pub unsafe fn list_del(el: *mut list_head) {
    let prev = (*el).prev;
    let next = (*el).next;
    (*prev).next = next;
    (*next).prev = prev;
    (*el).prev = ptr::null_mut();
    (*el).next = ptr::null_mut();
}

// c: list.h:list_empty (preserve the int-valued BOOL interface)
pub unsafe fn list_empty(el: *mut list_head) -> i32 {
    ((*el).next == el) as i32
}

/// The C iteration macros expressed as Rust iterators; no allocation.
/// The safe-deletion variants save the next node before yielding the current
/// node. The ordinary variants read the link after the loop body, like C.
pub struct ListIter {
    head: *mut list_head,
    cursor: *mut list_head,
    reverse: bool,
    deletion_safe: bool,
}

impl ListIter {
    pub unsafe fn new(head: *mut list_head, reverse: bool, deletion_safe: bool) -> Self {
        let cursor = if deletion_safe {
            if reverse {
                (*head).prev
            } else {
                (*head).next
            }
        } else {
            head
        };
        Self {
            head,
            cursor,
            reverse,
            deletion_safe,
        }
    }

    unsafe fn link(&self, node: *mut list_head) -> *mut list_head {
        if self.reverse {
            (*node).prev
        } else {
            (*node).next
        }
    }
}

impl Iterator for ListIter {
    type Item = *mut list_head;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let node = if self.deletion_safe {
                self.cursor
            } else {
                self.link(self.cursor)
            };
            if node == self.head {
                return None;
            }
            self.cursor = if self.deletion_safe {
                self.link(node)
            } else {
                node
            };
            Some(node)
        }
    }
}

#[macro_export]
macro_rules! list_entry {
    ($el:expr, $ty:ty, $member:ident) => {
        ($el as *mut u8).wrapping_sub(core::mem::offset_of!($ty, $member)) as *mut $ty
    };
}

#[macro_export]
macro_rules! list_for_each {
    ($el:ident, $head:expr, $body:block) => {
        for $el in $crate::list::ListIter::new($head, false, false) $body
    };
}

#[macro_export]
macro_rules! list_for_each_prev {
    ($el:ident, $head:expr, $body:block) => {
        for $el in $crate::list::ListIter::new($head, true, false) $body
    };
}

#[macro_export]
macro_rules! list_for_each_safe {
    ($el:ident, $el1:ident, $head:expr, $body:block) => {
        for $el in $crate::list::ListIter::new($head, false, true) {
            let $el1 = (*$el).next;
            $body
        }
    };
}

#[macro_export]
macro_rules! list_for_each_prev_safe {
    ($el:ident, $el1:ident, $head:expr, $body:block) => {
        for $el in $crate::list::ListIter::new($head, true, true) {
            let $el1 = (*$el).prev;
            $body
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[repr(C)]
    struct Node {
        value: i32,
        link: list_head,
    }
    #[test]
    fn insertion_iteration_container_and_safe_deletion() {
        unsafe {
            let mut head = Box::new(list_head::default());
            init_list_head(&mut *head);
            assert_eq!(list_empty(&mut *head), 1);
            let mut nodes: Vec<_> = (0..4)
                .map(|value| {
                    Box::new(Node {
                        value,
                        link: list_head::default(),
                    })
                })
                .collect();
            for node in &mut nodes {
                list_add_tail(&mut node.link, &mut *head);
            }
            list_del(&mut nodes[3].link);
            list_add(&mut nodes[3].link, &mut *head);
            let forward: Vec<_> = ListIter::new(&mut *head, false, false)
                .map(|link| (*crate::list_entry!(link, Node, link)).value)
                .collect();
            let reverse: Vec<_> = ListIter::new(&mut *head, true, false)
                .map(|link| (*crate::list_entry!(link, Node, link)).value)
                .collect();
            assert_eq!(forward, [3, 0, 1, 2]);
            assert_eq!(reverse, [2, 1, 0, 3]);
            crate::list_for_each_safe!(link, next, &mut *head, {
                let _ = next;
                list_del(link);
                assert!((*link).next.is_null() && (*link).prev.is_null());
                continue; // the C macro must still advance after continue
            });
            assert_eq!(list_empty(&mut *head), 1);
            for node in &mut nodes {
                list_add_tail(&mut node.link, &mut *head);
            }
            crate::list_for_each_prev_safe!(link, previous, &mut *head, {
                let _ = previous;
                list_del(link);
            });
            assert_eq!(list_empty(&mut *head), 1);
        }
    }
}
