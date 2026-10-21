//! Native intrusive doubly linked list; nodes and list stay at stable addresses
//! while linked. Unsafe operations retain the native caller's lifetime/exclusivity contract.
use std::ptr::null_mut;

// Source: src/core/SkTInternalLList.h:19-24
// Rust field projection replaces the native friend macro. Links are stored inline
// in the original node, never in a separate registry or owning list allocation.
pub struct Links<T> {
    #[cfg(sk_debug)]
    fList: *const SkTInternalLList<T>,
    fPrev: *mut T,
    fNext: *mut T,
}
impl<T> Default for Links<T> {
    fn default() -> Self {
        Self {
            #[cfg(sk_debug)]
            fList: std::ptr::null(),
            fPrev: null_mut(),
            fNext: null_mut(),
        }
    }
}
/// The projection must return the same inline links of a live, stable node.
pub unsafe trait Linked: Sized {
    unsafe fn links(node: *mut Self) -> *mut Links<Self>;
}

impl<T: Linked> SkTInternalLList<T> {
    // Source: src/core/SkTInternalLList.h:31-31
    pub fn new() -> Self {
        Self {
            fHead: null_mut(),
            fTail: null_mut(),
        }
    }
    // Source: src/core/SkTInternalLList.h:33-36
    // Native reset only forgets endpoints; it does not detach or destroy nodes.
    pub fn reset(&mut self) {
        self.fHead = null_mut();
        self.fTail = null_mut();
    }
    // Source: src/core/SkTInternalLList.h:38-62
    pub unsafe fn remove(&mut self, entry: *mut T) {
        crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),39; @condition b"fHead && fTail\0".as_ptr(); !self.fHead.is_null() && !self.fTail.is_null());
        #[cfg(sk_debug)]
        crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),40; @condition b"this->isInList(entry)\0".as_ptr(); self.isInList(entry));
        let prev = (*T::links(entry)).fPrev;
        let next = (*T::links(entry)).fNext;
        if !prev.is_null() {
            (*T::links(prev)).fNext = next;
        } else {
            self.fHead = next;
        }
        if !next.is_null() {
            (*T::links(next)).fPrev = prev;
        } else {
            self.fTail = prev;
        }
        (*T::links(entry)).fPrev = null_mut();
        (*T::links(entry)).fNext = null_mut();
        #[cfg(sk_debug)]
        {
            (*T::links(entry)).fList = std::ptr::null();
        }
    }
    // Source: src/core/SkTInternalLList.h:64-81
    pub unsafe fn addToHead(&mut self, entry: *mut T) {
        crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),65; @condition b"nullptr == entry->fPrev && nullptr == entry->fNext\0".as_ptr(); (*T::links(entry)).fPrev.is_null() && (*T::links(entry)).fNext.is_null());
        #[cfg(sk_debug)]
        crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),66; @condition b"nullptr == entry->fList\0".as_ptr(); (*T::links(entry)).fList.is_null());
        (*T::links(entry)).fPrev = null_mut();
        (*T::links(entry)).fNext = self.fHead;
        if !self.fHead.is_null() {
            (*T::links(self.fHead)).fPrev = entry;
        }
        self.fHead = entry;
        if self.fTail.is_null() {
            self.fTail = entry;
        }
        #[cfg(sk_debug)]
        {
            (*T::links(entry)).fList = self;
        }
    }
    // Source: src/core/SkTInternalLList.h:83-100
    pub unsafe fn addToTail(&mut self, entry: *mut T) {
        crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),84; @condition b"nullptr == entry->fPrev && nullptr == entry->fNext\0".as_ptr(); (*T::links(entry)).fPrev.is_null() && (*T::links(entry)).fNext.is_null());
        #[cfg(sk_debug)]
        crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),85; @condition b"nullptr == entry->fList\0".as_ptr(); (*T::links(entry)).fList.is_null());
        (*T::links(entry)).fPrev = self.fTail;
        (*T::links(entry)).fNext = null_mut();
        if !self.fTail.is_null() {
            (*T::links(self.fTail)).fNext = entry;
        }
        self.fTail = entry;
        if self.fHead.is_null() {
            self.fHead = entry;
        }
        #[cfg(sk_debug)]
        {
            (*T::links(entry)).fList = self;
        }
    }
    // Source: src/core/SkTInternalLList.h:107-129
    pub unsafe fn addBefore(&mut self, newEntry: *mut T, existingEntry: *mut T) {
        crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),108; @condition b"newEntry\0".as_ptr(); !newEntry.is_null());
        if existingEntry.is_null() {
            self.addToTail(newEntry);
            return;
        }
        #[cfg(sk_debug)]
        crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),115; @condition b"this->isInList(existingEntry)\0".as_ptr(); self.isInList(existingEntry));
        (*T::links(newEntry)).fNext = existingEntry;
        let prev = (*T::links(existingEntry)).fPrev;
        (*T::links(existingEntry)).fPrev = newEntry;
        (*T::links(newEntry)).fPrev = prev;
        if prev.is_null() {
            crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),121; @condition b"fHead == existingEntry\0".as_ptr(); self.fHead == existingEntry);
            self.fHead = newEntry;
        } else {
            (*T::links(prev)).fNext = newEntry;
        }
        #[cfg(sk_debug)]
        {
            (*T::links(newEntry)).fList = self;
        }
    }
    // Source: src/core/SkTInternalLList.h:136-158
    pub unsafe fn addAfter(&mut self, newEntry: *mut T, existingEntry: *mut T) {
        crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),137; @condition b"newEntry\0".as_ptr(); !newEntry.is_null());
        if existingEntry.is_null() {
            self.addToHead(newEntry);
            return;
        }
        #[cfg(sk_debug)]
        crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),144; @condition b"this->isInList(existingEntry)\0".as_ptr(); self.isInList(existingEntry));
        (*T::links(newEntry)).fPrev = existingEntry;
        let next = (*T::links(existingEntry)).fNext;
        (*T::links(existingEntry)).fNext = newEntry;
        (*T::links(newEntry)).fNext = next;
        if next.is_null() {
            crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),150; @condition b"fTail == existingEntry\0".as_ptr(); self.fTail == existingEntry);
            self.fTail = newEntry;
        } else {
            (*T::links(next)).fPrev = newEntry;
        }
        #[cfg(sk_debug)]
        {
            (*T::links(newEntry)).fList = self;
        }
    }
    // Source: src/core/SkTInternalLList.h:160-183
    pub unsafe fn concat(&mut self, list: &mut Self) {
        if list.isEmpty() {
            return;
        }
        (*T::links(list.fHead)).fPrev = self.fTail;
        if self.fHead.is_null() {
            crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),167; @condition b"!list.fHead->fPrev\0".as_ptr(); (*T::links(list.fHead)).fPrev.is_null());
            self.fHead = list.fHead;
        } else {
            crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),170; @condition b"fTail\0".as_ptr(); !self.fTail.is_null());
            (*T::links(self.fTail)).fNext = list.fHead;
        }
        self.fTail = list.fTail;
        #[cfg(sk_debug)]
        {
            let mut node = list.fHead;
            while !node.is_null() {
                crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),177; @condition b"node->fList == &list\0".as_ptr(); std::ptr::eq((*T::links(node)).fList, list));
                (*T::links(node)).fList = self;
                node = (*T::links(node)).fNext;
            }
        }
        list.fHead = null_mut();
        list.fTail = null_mut();
    }
    // Source: src/core/SkTInternalLList.h:185-188
    pub fn isEmpty(&self) -> bool {
        crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),186; @condition b"SkToBool(fHead) == SkToBool(fTail)\0".as_ptr(); (self.fHead.is_null())==(self.fTail.is_null()));
        self.fHead.is_null()
    }
    // Source: src/core/SkTInternalLList.h:190-191
    pub fn head(&self) -> *mut T {
        self.fHead
    }
    pub fn tail(&self) -> *mut T {
        self.fTail
    }
}

// Source: src/core/SkTInternalLList.h:195-198
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IterStart {
    kHead_IterStart,
    kTail_IterStart,
}
impl<T> Iter<T> {
    // Source: src/core/SkTInternalLList.h:200-200
    pub fn new() -> Self {
        Self { fCurr: null_mut() }
    }
}
// Source: src/core/SkTInternalLList.h:201-201
impl<T> Clone for Iter<T> {
    fn clone(&self) -> Self {
        Self { fCurr: self.fCurr }
    }
}
impl<T> Iter<T> {
    // Source: src/core/SkTInternalLList.h:202-202
    pub fn assign_from(&mut self, iter: &Self) -> &mut Self {
        self.fCurr = iter.fCurr;
        self
    }
    // Source: src/core/SkTInternalLList.h:204-213
    pub fn init(&mut self, list: &SkTInternalLList<T>, startLoc: IterStart) -> *mut T {
        self.fCurr = if startLoc == IterStart::kHead_IterStart {
            list.fHead
        } else {
            crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),208; @condition b"kTail_IterStart == startLoc\0".as_ptr(); startLoc==IterStart::kTail_IterStart);
            list.fTail
        };
        self.fCurr
    }
    // Source: src/core/SkTInternalLList.h:215-215
    pub fn get(&self) -> *mut T {
        self.fCurr
    }
}
impl<T: Linked> Iter<T> {
    // Source: src/core/SkTInternalLList.h:220-227
    pub unsafe fn next(&mut self) -> *mut T {
        if self.fCurr.is_null() {
            return null_mut();
        }
        self.fCurr = (*T::links(self.fCurr)).fNext;
        self.fCurr
    }
    // Source: src/core/SkTInternalLList.h:229-236
    pub unsafe fn prev(&mut self) -> *mut T {
        if self.fCurr.is_null() {
            return null_mut();
        }
        self.fCurr = (*T::links(self.fCurr)).fPrev;
        self.fCurr
    }
}
// Source: src/core/SkTInternalLList.h:241-241
impl<T> PartialEq for Iter<T> {
    fn eq(&self, other: &Self) -> bool {
        self.fCurr == other.fCurr
    }
}
impl<T: Linked> Iter<T> {
    // Source: src/core/SkTInternalLList.h:242-243
    pub fn deref(&self) -> *mut T {
        self.get()
    }
    pub unsafe fn advance(&mut self) {
        self.next();
    }
}
// Source: src/core/SkTInternalLList.h:245-246
pub struct Iter<T> {
    fCurr: *mut T,
}

impl<T: Linked> SkTInternalLList<T> {
    // Source: src/core/SkTInternalLList.h:249-253
    pub fn begin(&self) -> Iter<T> {
        let mut iter = Iter::new();
        iter.init(self, IterStart::kHead_IterStart);
        iter
    }
    // Source: src/core/SkTInternalLList.h:255-255
    pub fn end(&self) -> Iter<T> {
        Iter::new()
    }
    // Source: src/core/SkTInternalLList.h:258-274
    #[cfg(sk_debug)]
    pub unsafe fn validate(&self) {
        crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),259; @condition b"!fHead == !fTail\0".as_ptr(); (self.fHead.is_null())==(self.fTail.is_null()));
        let mut iter = Iter::new();
        let mut item = iter.init(self, IterStart::kHead_IterStart);
        while !item.is_null() {
            crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),262; @condition b"this->isInList(item)\0".as_ptr(); self.isInList(item));
            let prev = (*T::links(item)).fPrev;
            let next = (*T::links(item)).fNext;
            if prev.is_null() {
                crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),264; @condition b"fHead == item\0".as_ptr(); self.fHead == item);
            } else {
                crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),266; @condition b"item->fPrev->fNext == item\0".as_ptr(); (*T::links(prev)).fNext == item);
            }
            if next.is_null() {
                crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),269; @condition b"fTail == item\0".as_ptr(); self.fTail == item);
            } else {
                crate::SkASSERT!(@source b"src/core/SkTInternalLList.h\0".as_ptr(),271; @condition b"item->fNext->fPrev == item\0".as_ptr(); (*T::links(next)).fPrev == item);
            }
            item = iter.next();
        }
    }
    // Source: src/core/SkTInternalLList.h:280-282
    #[cfg(sk_debug)]
    pub unsafe fn isInList(&self, entry: *mut T) -> bool {
        std::ptr::eq((*T::links(entry)).fList, self)
    }
    // Source: src/core/SkTInternalLList.h:287-293
    #[cfg(sk_debug)]
    pub unsafe fn countEntries(&self) -> i32 {
        let mut count = 0;
        let mut entry = self.fHead;
        while !entry.is_null() {
            count += 1;
            entry = (*T::links(entry)).fNext;
        }
        count
    }
}
// Source: src/core/SkTInternalLList.h:296-301
// Rust has no implicit Copy/Clone for the owner.
pub struct SkTInternalLList<T> {
    fHead: *mut T,
    fTail: *mut T,
}
