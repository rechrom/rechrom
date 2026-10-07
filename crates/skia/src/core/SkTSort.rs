//! SkTSort.h introsort, including tie ordering used by analytic edges.
//! Copyright 2006 The Android Open Source Project. BSD-3-Clause; ../../LICENSE.

pub fn sort<T: Clone>(values: &mut [T], less: impl Fn(&T, &T) -> bool) {
    fn sift_down<T: Clone>(
        a: &mut [T],
        mut root: usize,
        bottom: usize,
        less: &impl Fn(&T, &T) -> bool,
    ) {
        let value = a[root - 1].clone();
        let mut child = root * 2;
        while child <= bottom {
            if child < bottom && less(&a[child - 1], &a[child]) {
                child += 1;
            }
            if !less(&value, &a[child - 1]) {
                break;
            }
            a[root - 1] = a[child - 1].clone();
            root = child;
            child = root * 2;
        }
        a[root - 1] = value;
    }
    fn sift_up<T: Clone>(
        a: &mut [T],
        mut root: usize,
        bottom: usize,
        less: &impl Fn(&T, &T) -> bool,
    ) {
        let value = a[root - 1].clone();
        let start = root;
        let mut child = root * 2;
        while child <= bottom {
            if child < bottom && less(&a[child - 1], &a[child]) {
                child += 1;
            }
            a[root - 1] = a[child - 1].clone();
            root = child;
            child = root * 2;
        }
        let mut parent = root / 2;
        while parent >= start && less(&a[parent - 1], &value) {
            a[root - 1] = a[parent - 1].clone();
            root = parent;
            parent = root / 2;
        }
        a[root - 1] = value;
    }
    fn intro<T: Clone>(mut a: &mut [T], mut depth: u32, less: &impl Fn(&T, &T) -> bool) {
        loop {
            let count = a.len();
            if count <= 32 {
                for next in 1..count {
                    if !less(&a[next], &a[next - 1]) {
                        continue;
                    }
                    let value = a[next].clone();
                    let mut hole = next;
                    loop {
                        a[hole] = a[hole - 1].clone();
                        hole -= 1;
                        if hole == 0 || !less(&value, &a[hole - 1]) {
                            break;
                        }
                    }
                    a[hole] = value;
                }
                return;
            }
            if depth == 0 {
                for root in (1..=count / 2).rev() {
                    sift_down(a, root, count, less);
                }
                for end in (1..count).rev() {
                    a.swap(0, end);
                    sift_up(a, 1, end, less);
                }
                return;
            }
            depth -= 1;
            let middle = (count - 1) / 2;
            let pivot = a[middle].clone();
            a.swap(middle, count - 1);
            let mut new_pivot = 0;
            for left in 0..count - 1 {
                if less(&a[left], &pivot) {
                    a.swap(left, new_pivot);
                    new_pivot += 1;
                }
            }
            a.swap(new_pivot, count - 1);
            let (left, right) = a.split_at_mut(new_pivot);
            intro(left, depth, less);
            a = &mut right[1..];
        }
    }
    if values.len() > 1 {
        let depth = 2 * (usize::BITS - (values.len() - 2).leading_zeros());
        intro(values, depth, &less);
    }
}
