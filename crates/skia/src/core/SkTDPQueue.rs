// Copyright 2015 Google Inc. BSD-style license.
// Native function template parameters become immutable function-pointer fields.
// T:Clone supplies native value copy construction/assignment; Rust moves transfer
// owners without an extra retain. This does not claim foreign move/ADL-swap ABI.
// Source: src/core/SkTDPQueue.h:30-34
pub type Index<T> = unsafe fn(&T) -> *mut i32;
pub struct SkTDPQueue<T> {
    // Source: src/core/SkTDPQueue.h:219-219
    fArray: Vec<T>,
    less: fn(&T, &T) -> bool,
    index: Option<Index<T>>,
}
impl<T: Clone> SkTDPQueue<T> {
    // Source: src/core/SkTDPQueue.h:35-36
    pub fn new(reserve: usize, less: fn(&T, &T) -> bool) -> Self {
        Self {
            fArray: Vec::with_capacity(reserve),
            less,
            index: None,
        }
    }
    pub fn empty(less: fn(&T, &T) -> bool) -> Self {
        Self::new(0, less)
    }
    /// Safety: INDEX projects the original live writable index for every element
    /// supplied to this queue. Its pointed storage remains valid while the native
    /// queue operation uses it; projection neither unwinds nor reenters the queue.
    pub unsafe fn with_index(reserve: usize, less: fn(&T, &T) -> bool, index: Index<T>) -> Self {
        Self {
            fArray: Vec::with_capacity(reserve),
            less,
            index: Some(index),
        }
    }
    // Source: src/core/SkTDPQueue.h:38-39
    pub fn move_from(that: &mut Self) -> Self {
        Self {
            fArray: core::mem::take(&mut that.fArray),
            less: that.less,
            index: that.index,
        }
    }
    pub fn move_assign(&mut self, that: &mut Self) -> &mut Self {
        self.fArray = core::mem::take(&mut that.fArray);
        self.less = that.less;
        self.index = that.index;
        self
    }
    // Source: src/core/SkTDPQueue.h:41-42
    // Copy construction and assignment remain unavailable (no Clone impl).
    // Source: src/core/SkTDPQueue.h:45-45
    pub fn count(&self) -> usize {
        self.fArray.len()
    }
    // Source: src/core/SkTDPQueue.h:48-48
    pub fn peek(&self) -> &T {
        &self.fArray[0]
    }
    // Source: src/core/SkTDPQueue.h:49-49
    pub fn peek_mut(&mut self) -> &mut T {
        &mut self.fArray[0]
    }
    // Source: src/core/SkTDPQueue.h:52-66
    /// Safety: queue is nonempty. If configured, INDEX obeys with_index's original writable-storage
    /// contract for all live elements through every move/copy in this operation.
    pub unsafe fn pop(&mut self) {
        unsafe {
            self.validate(-1);
            #[cfg(sk_debug)]
            if let Some(index) = self.index {
                *index(&self.fArray[0]) = -1;
            }
            if self.fArray.len() == 1 {
                self.fArray.pop();
                return;
            }
            let last = self.fArray.len() - 1;
            let p = self.fArray.as_mut_ptr();
            (*p).clone_from(&*p.add(last));
            self.setIndex(0);
            self.fArray.pop();
            self.percolateDownIfNecessary(0);
            self.validate(-1);
        }
    }
    // Source: src/core/SkTDPQueue.h:69-76
    /// Safety: preserve the configured INDEX projection/storage contract.
    pub unsafe fn insert(&mut self, entry: T) {
        unsafe {
            self.validate(-1);
            let index = self.fArray.len();
            self.fArray.push(entry);
            self.setIndex(self.fArray.len() - 1);
            self.percolateUpIfNecessary(index);
            self.validate(-1);
        }
    }
    // Source: src/core/SkTDPQueue.h:79-94
    /// Safety: INDEX is configured and entry identifies an original live queue
    /// element with its valid current index; preserve that storage through operation.
    pub unsafe fn remove(&mut self, entry: T) {
        unsafe {
            crate::SkASSERT!(@source b"src/core/SkTDPQueue.h\0".as_ptr(),80; @condition b"nullptr != INDEX\0".as_ptr(); self.index.is_some());
            let index = *self.index.unwrap()(&entry);
            debug_assert!(index >= 0 && (index as usize) < self.fArray.len());
            self.validate(-1);
            #[cfg(sk_debug)]
            {
                *self.index.unwrap()(&self.fArray[index as usize]) = -1;
            }
            let index = index as usize;
            if index == self.fArray.len() - 1 {
                self.fArray.pop();
                return;
            }
            let last = self.fArray.len() - 1;
            let p = self.fArray.as_mut_ptr();
            (*p.add(index)).clone_from(&*p.add(last));
            self.fArray.pop();
            self.setIndex(index);
            self.percolateUpOrDown(index);
            self.validate(-1);
        }
    }
    // Source: src/core/SkTDPQueue.h:99-106
    /// Safety: entry and configured INDEX identify the original live priority-
    /// modified queue element; projection/storage obey the native operation contract.
    pub unsafe fn priorityDidChange(&mut self, entry: T) {
        unsafe {
            crate::SkASSERT!(@source b"src/core/SkTDPQueue.h\0".as_ptr(),100; @condition b"nullptr != INDEX\0".as_ptr(); self.index.is_some());
            let index = *self.index.unwrap()(&entry);
            debug_assert!(index >= 0 && (index as usize) < self.fArray.len());
            self.validate(index);
            self.percolateUpOrDown(index as usize);
            self.validate(-1);
        }
    }
    // Source: src/core/SkTDPQueue.h:110-110
    pub fn at(&self, i: usize) -> T {
        self.fArray[i].clone()
    }
    // Source: src/core/SkTDPQueue.h:115-123
    /// Safety: preserve the configured INDEX projection/storage contract.
    pub unsafe fn sort(&mut self) {
        unsafe {
            if self.fArray.len() > 1 {
                crate::src::core::SkTSort::SkTQSort(&mut self.fArray, &self.less);
                for i in 0..self.fArray.len() {
                    self.setIndex(i);
                }
                self.validate(-1);
            }
        }
    }
    // Source: src/core/SkTDPQueue.h:126-126
    fn LeftOf(x: usize) -> usize {
        2 * x + 1
    }
    // Source: src/core/SkTDPQueue.h:127-127
    fn ParentOf(x: usize) -> usize {
        crate::SkASSERT!(@source b"src/core/SkTDPQueue.h\0".as_ptr(),127; @condition b"x > 0\0".as_ptr(); x>0);
        (x - 1) >> 1
    }
    // Source: src/core/SkTDPQueue.h:129-135
    unsafe fn percolateUpOrDown(&mut self, index: usize) {
        unsafe {
            if !self.percolateUpIfNecessary(index) {
                self.validate(index as i32);
                self.percolateDownIfNecessary(index);
            }
        }
    }
    // Source: src/core/SkTDPQueue.h:137-158
    unsafe fn percolateUpIfNecessary(&mut self, mut index: usize) -> bool {
        unsafe {
            let mut percolated = false;
            loop {
                if index == 0 {
                    self.setIndex(index);
                    return percolated;
                }
                let p = Self::ParentOf(index);
                if (self.less)(&self.fArray[index], &self.fArray[p]) {
                    self.fArray.swap(index, p);
                    self.setIndex(index);
                    index = p;
                    percolated = true;
                } else {
                    self.setIndex(index);
                    return percolated;
                }
                self.validate(index as i32);
            }
        }
    }
    // Source: src/core/SkTDPQueue.h:160-198
    unsafe fn percolateDownIfNecessary(&mut self, mut index: usize) {
        unsafe {
            loop {
                let mut child = Self::LeftOf(index);
                if child >= self.fArray.len() {
                    self.setIndex(index);
                    return;
                }
                if child + 1 >= self.fArray.len() {
                    if (self.less)(&self.fArray[child], &self.fArray[index]) {
                        self.fArray.swap(child, index);
                        self.setIndex(child);
                        self.setIndex(index);
                        return;
                    }
                } else if (self.less)(&self.fArray[child + 1], &self.fArray[child]) {
                    child += 1;
                }
                if (self.less)(&self.fArray[child], &self.fArray[index]) {
                    self.fArray.swap(child, index);
                    self.setIndex(index);
                    index = child;
                } else {
                    self.setIndex(index);
                    return;
                }
                self.validate(index as i32);
            }
        }
    }
    // Source: src/core/SkTDPQueue.h:200-205
    unsafe fn setIndex(&mut self, index: usize) {
        unsafe {
            crate::SkASSERT!(@source b"src/core/SkTDPQueue.h\0".as_ptr(),201; @condition b"index < fArray.size()\0".as_ptr(); index<self.fArray.len());
            if let Some(project) = self.index {
                *project(&self.fArray[index]) = index as i32;
            }
        }
    }
    // Source: src/core/SkTDPQueue.h:207-217
    unsafe fn validate(&self, _excludedIndex: i32) {
        #[cfg(sk_debug)]
        unsafe {
            for i in 1..self.fArray.len() {
                let p = Self::ParentOf(i);
                if _excludedIndex != p as i32 && _excludedIndex != i as i32 {
                    debug_assert!(!(self.less)(&self.fArray[i], &self.fArray[p]));
                    debug_assert!(
                        self.index.is_none() || *self.index.unwrap()(&self.fArray[i]) == i as i32
                    );
                }
            }
        }
    }
}
