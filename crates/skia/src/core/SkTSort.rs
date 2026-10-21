// Copyright 2006 The Android Open Source Project. BSD-style license.
// Native copy construction/assignment use Rust Clone::clone/clone_from. Native
// moves and std::swap use Rust ownership relocation: no duplicate live owner and
// no drop of an uninitialized/moved-out slot. C++ user-defined move/ADL-swap side
// effects have no automatic Rust analogue; this is a Rust ownership adaptation,
// not a promise to reproduce arbitrary foreign specializations.
// Source: src/core/SkTSort.h:34-57
pub fn SkTHeapSort_SiftUp<T: Clone>(
    array: &mut [T],
    mut root: usize,
    bottom: usize,
    lessThan: &impl Fn(&T, &T) -> bool,
) {
    assert!(bottom <= array.len());
    let x = array[root - 1].clone();
    let start = root;
    let mut j = root << 1;
    while j <= bottom {
        if j < bottom && lessThan(&array[j - 1], &array[j]) {
            j += 1;
        }
        // Distinct parent/child entries retain native copy assignment.
        unsafe {
            let p = array.as_mut_ptr();
            (*p.add(root - 1)).clone_from(&*p.add(j - 1));
        }
        root = j;
        j = root << 1;
    }
    j = root >> 1;
    while j >= start {
        if lessThan(&array[j - 1], &x) {
            // Distinct parent/child entries retain native copy assignment.
            unsafe {
                let p = array.as_mut_ptr();
                (*p.add(root - 1)).clone_from(&*p.add(j - 1));
            }
            root = j;
            j = root >> 1;
        } else {
            break;
        }
    }
    array[root - 1].clone_from(&x);
}
// Source: src/core/SkTSort.h:68-84
pub fn SkTHeapSort_SiftDown<T: Clone>(
    array: &mut [T],
    mut root: usize,
    bottom: usize,
    lessThan: &impl Fn(&T, &T) -> bool,
) {
    assert!(bottom <= array.len());
    let x = array[root - 1].clone();
    let mut child = root << 1;
    while child <= bottom {
        if child < bottom && lessThan(&array[child - 1], &array[child]) {
            child += 1;
        }
        if lessThan(&x, &array[child - 1]) {
            unsafe {
                let p = array.as_mut_ptr();
                (*p.add(root - 1)).clone_from(&*p.add(child - 1));
            }
            root = child;
            child = root << 1;
        } else {
            break;
        }
    }
    array[root - 1].clone_from(&x);
}
// Source: src/core/SkTSort.h:93-103
pub fn SkTHeapSort<T: Clone>(array: &mut [T], lessThan: &impl Fn(&T, &T) -> bool) {
    // Native count-1 requires nonempty input; introsort always satisfies this.
    assert!(!array.is_empty());
    let count = array.len();
    for i in (1..=(count >> 1)).rev() {
        SkTHeapSort_SiftDown(array, i, count, lessThan);
    }
    for i in (1..count).rev() {
        array.swap(0, i);
        SkTHeapSort_SiftUp(array, 1, i, lessThan);
    }
}
// Source: src/core/SkTSort.h:106-108
pub fn SkTHeapSortNatural<T: Clone + PartialOrd>(array: &mut [T]) {
    SkTHeapSort(array, &|a, b| a < b);
}
// Source: src/core/SkTSort.h:114-128
pub fn SkTInsertionSort<T>(left: &mut [T], lessThan: &impl Fn(&T, &T) -> bool) {
    // A native moved-from object remains destructible. Rust instead leaves one
    // uninitialized hole. This guard restores the unique saved value even if
    // the comparator unwinds; no slice/reference spans an uninitialized slot.
    struct Hole<T> {
        value: core::mem::ManuallyDrop<T>,
        place: *mut T,
    }
    impl<T> Drop for Hole<T> {
        fn drop(&mut self) {
            unsafe {
                core::ptr::copy_nonoverlapping(&*self.value, self.place, 1);
            }
        }
    }
    for next in 1..left.len() {
        if !lessThan(&left[next], &left[next - 1]) {
            continue;
        }
        unsafe {
            let base = left.as_mut_ptr();
            let mut saved = Hole {
                value: core::mem::ManuallyDrop::new(base.add(next).read()),
                place: base.add(next),
            };
            let mut hole = next;
            loop {
                // Relocate the predecessor into the current hole. The old
                // predecessor slot immediately becomes the sole new hole.
                core::ptr::copy_nonoverlapping(base.add(hole - 1), saved.place, 1);
                hole -= 1;
                saved.place = base.add(hole);
                if !(0 < hole && lessThan(&*saved.value, &*base.add(hole - 1))) {
                    break;
                }
            }
            // Drop places the saved value into the final hole exactly once.
        }
    }
}
// Source: src/core/SkTSort.h:133-148
pub fn SkTQSort_Partition<T: Clone>(
    left: &mut [T],
    pivot: usize,
    lessThan: &impl Fn(&T, &T) -> bool,
) -> usize {
    let right = left.len() - 1;
    let pivotValue = left[pivot].clone();
    left.swap(pivot, right);
    let mut newPivot = 0;
    for i in 0..right {
        if lessThan(&left[i], &pivotValue) {
            left.swap(i, newPivot);
            newPivot += 1;
        }
    }
    left.swap(newPivot, right);
    newPivot
}
// Source: src/core/SkTSort.h:163-184
pub fn SkTIntroSort<T: Clone>(
    mut depth: i32,
    mut left: &mut [T],
    lessThan: &impl Fn(&T, &T) -> bool,
) {
    loop {
        let count = left.len();
        if count <= 32 {
            SkTInsertionSort(left, lessThan);
            return;
        }
        if depth == 0 {
            SkTHeapSort(left, lessThan);
            return;
        }
        depth -= 1;
        let middle = (count - 1) >> 1;
        let pivot = SkTQSort_Partition(left, middle, lessThan);
        let (lo, hi) = left.split_at_mut(pivot);
        SkTIntroSort(depth, lo, lessThan);
        left = &mut hi[1..];
    }
}
// Source: src/core/SkTSort.h:194-202
pub fn SkTQSort<T: Clone>(array: &mut [T], lessThan: &impl Fn(&T, &T) -> bool) {
    let n = i32::try_from(array.len()).expect("SkTQSort count must fit int");
    if n <= 1 {
        return;
    }
    let depth = 2 * crate::src::core::SkMathPriv::SkNextLog2((n - 1) as u32);
    SkTIntroSort(depth, array, lessThan);
}
// Source: src/core/SkTSort.h:205-207
pub fn SkTQSortNatural<T: Clone + PartialOrd>(array: &mut [T]) {
    SkTQSort(array, &|a, b| a < b);
}
// Source: src/core/SkTSort.h:210-212
/// Safety: every pointer must reference a valid T for each comparison.
pub unsafe fn SkTQSortPointers<T: PartialOrd>(array: &mut [*const T]) {
    // Pointer values are the native trivial-copy specialization. Route this
    // existing CPU entry through the actual operation implementation/table.
    SkTQSortWithOps(array, &|a, b| **a < **b, &mut SkTSortCopyOps);
}

// Rust language-operation glue; no separate native source-line credit.
// Explicit language-operation adapter for native specializations. Each method
// denotes the named native operation, not a guessed Clone equivalence. All
// arguments/array slots remain initialized, including moved-from sources. An
// implementation must uphold that Rust invariant even when it panics. Callbacks
// may record operation effects/state; algorithm ordering stays native.
pub trait SkTSortNativeOps<T> {
    fn copy_construct(&mut self, source: &T) -> T;
    fn copy_assign(&mut self, destination: &mut T, source: &T);
    fn move_construct(&mut self, source: &mut T) -> T;
    fn move_assign(&mut self, destination: &mut T, source: &mut T);
    // Indices may be equal, exactly as for native ADL-selected swap.
    fn swap(&mut self, array: &mut [T], a: usize, b: usize);
}
// Borrow two distinct initialized array entries for native assignment.
fn native_assign<T, O: SkTSortNativeOps<T>>(
    array: &mut [T],
    dst: usize,
    src: usize,
    ops: &mut O,
    moving: bool,
) {
    assert_ne!(dst, src);
    let (destination, source) = if dst < src {
        let (lo, hi) = array.split_at_mut(src);
        (&mut lo[dst], &mut hi[0])
    } else {
        let (lo, hi) = array.split_at_mut(dst);
        (&mut hi[0], &mut lo[src])
    };
    if moving {
        ops.move_assign(destination, source);
    } else {
        ops.copy_assign(destination, source);
    }
}
// Source: src/core/SkTSort.h:33-57
pub fn SkTHeapSort_SiftUpWithOps<T, O: SkTSortNativeOps<T>>(
    array: &mut [T],
    mut root: usize,
    bottom: usize,
    lessThan: &impl Fn(&T, &T) -> bool,
    ops: &mut O,
) {
    assert!(bottom <= array.len());
    let x = ops.copy_construct(&array[root - 1]);
    let start = root;
    let mut j = root << 1;
    while j <= bottom {
        if j < bottom && lessThan(&array[j - 1], &array[j]) {
            j += 1;
        }
        // Distinct parent/child entries retain native copy assignment.
        native_assign(array, root - 1, j - 1, ops, false);
        root = j;
        j = root << 1;
    }
    j = root >> 1;
    while j >= start {
        if lessThan(&array[j - 1], &x) {
            // Distinct parent/child entries retain native copy assignment.
            native_assign(array, root - 1, j - 1, ops, false);
            root = j;
            j = root >> 1;
        } else {
            break;
        }
    }
    ops.copy_assign(&mut array[root - 1], &x);
}
// Source: src/core/SkTSort.h:67-84
pub fn SkTHeapSort_SiftDownWithOps<T, O: SkTSortNativeOps<T>>(
    array: &mut [T],
    mut root: usize,
    bottom: usize,
    lessThan: &impl Fn(&T, &T) -> bool,
    ops: &mut O,
) {
    assert!(bottom <= array.len());
    let x = ops.copy_construct(&array[root - 1]);
    let mut child = root << 1;
    while child <= bottom {
        if child < bottom && lessThan(&array[child - 1], &array[child]) {
            child += 1;
        }
        if lessThan(&x, &array[child - 1]) {
            native_assign(array, root - 1, child - 1, ops, false);
            root = child;
            child = root << 1;
        } else {
            break;
        }
    }
    ops.copy_assign(&mut array[root - 1], &x);
}
// Source: src/core/SkTSort.h:93-103
pub fn SkTHeapSortWithOps<T, O: SkTSortNativeOps<T>>(
    array: &mut [T],
    lessThan: &impl Fn(&T, &T) -> bool,
    ops: &mut O,
) {
    // Native count-1 requires nonempty input; introsort always satisfies this.
    assert!(!array.is_empty());
    let count = array.len();
    for i in (1..=(count >> 1)).rev() {
        SkTHeapSort_SiftDownWithOps(array, i, count, lessThan, ops);
    }
    for i in (1..count).rev() {
        ops.swap(array, 0, i);
        SkTHeapSort_SiftUpWithOps(array, 1, i, lessThan, ops);
    }
}
// Source: src/core/SkTSort.h:113-128
pub fn SkTInsertionSortWithOps<T, O: SkTSortNativeOps<T>>(
    left: &mut [T],
    lessThan: &impl Fn(&T, &T) -> bool,
    ops: &mut O,
) {
    for next in 1..left.len() {
        if !lessThan(&left[next], &left[next - 1]) {
            continue;
        }
        let mut insert = ops.move_construct(&mut left[next]);
        let mut hole = next;
        loop {
            native_assign(left, hole, hole - 1, ops, true);
            hole -= 1;
            if !(0 < hole && lessThan(&insert, &left[hole - 1])) {
                break;
            }
        }
        ops.move_assign(&mut left[hole], &mut insert);
        // insert's final moved-from owner is dropped at the native local scope.
    }
}
// Source: src/core/SkTSort.h:132-148
pub fn SkTQSort_PartitionWithOps<T, O: SkTSortNativeOps<T>>(
    left: &mut [T],
    pivot: usize,
    lessThan: &impl Fn(&T, &T) -> bool,
    ops: &mut O,
) -> usize {
    let right = left.len() - 1;
    let pivotValue = ops.copy_construct(&left[pivot]);
    ops.swap(left, pivot, right);
    let mut newPivot = 0;
    for i in 0..right {
        if lessThan(&left[i], &pivotValue) {
            ops.swap(left, i, newPivot);
            newPivot += 1;
        }
    }
    ops.swap(left, newPivot, right);
    newPivot
}
// Source: src/core/SkTSort.h:162-184
pub fn SkTIntroSortWithOps<T, O: SkTSortNativeOps<T>>(
    mut depth: i32,
    mut left: &mut [T],
    lessThan: &impl Fn(&T, &T) -> bool,
    ops: &mut O,
) {
    loop {
        let count = left.len();
        if count <= 32 {
            SkTInsertionSortWithOps(left, lessThan, ops);
            return;
        }
        if depth == 0 {
            SkTHeapSortWithOps(left, lessThan, ops);
            return;
        }
        depth -= 1;
        let middle = (count - 1) >> 1;
        let pivot = SkTQSort_PartitionWithOps(left, middle, lessThan, ops);
        let (lo, hi) = left.split_at_mut(pivot);
        SkTIntroSortWithOps(depth, lo, lessThan, ops);
        left = &mut hi[1..];
    }
}
// Source: src/core/SkTSort.h:193-202
pub fn SkTQSortWithOps<T, O: SkTSortNativeOps<T>>(
    array: &mut [T],
    lessThan: &impl Fn(&T, &T) -> bool,
    ops: &mut O,
) {
    let n = i32::try_from(array.len()).expect("SkTQSort count must fit int");
    if n <= 1 {
        return;
    }
    let depth = 2 * crate::src::core::SkMathPriv::SkNextLog2((n - 1) as u32);
    SkTIntroSortWithOps(depth, array, lessThan, ops);
}

// Rust ownership-operation adapter for the native trivial-copy CPU domain.
// Copy values have no destructor/moved-from ownership. Raw-pointer assignment
// preserves the pointer value; native scalar std::swap has these same three
// value assignments. No custom C++ special-member/ADL swap is inferred here.
// This adapter and wiring carry zero additional native source-line credit.
pub struct SkTSortCopyOps;
impl<T: Copy> SkTSortNativeOps<T> for SkTSortCopyOps {
    fn copy_construct(&mut self, source: &T) -> T {
        *source
    }
    fn copy_assign(&mut self, destination: &mut T, source: &T) {
        *destination = *source;
    }
    fn move_construct(&mut self, source: &mut T) -> T {
        *source
    }
    fn move_assign(&mut self, destination: &mut T, source: &mut T) {
        *destination = *source;
    }
    fn swap(&mut self, array: &mut [T], a: usize, b: usize) {
        // Preserve native self-swap too. Array indexing retains checked Rust
        // bounds; Copy ensures these assignments cannot run user destructors.
        let temporary = array[a];
        array[a] = array[b];
        array[b] = temporary;
    }
}
