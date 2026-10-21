//! Native bulk-loaded RTree. Node addresses remain stable after one exact reserve.
use crate::include::core::SkRefCnt::{SkRefCnt, SkRefCntBase, SkRefCntObject};
use crate::src::core::SkBBHFactory::SkBBoxHierarchy;
use crate::src::core::SkRect::SkRect;
use std::cell::UnsafeCell;
use std::mem::{size_of, MaybeUninit};

// Source: src/core/SkRTree.h:50-51
pub const kMinChildren: i32 = 6;
pub const kMaxChildren: i32 = 11;

// Source: src/core/SkRTree.h:54-68
// The native union retains its pointer/index representations; fLevel selects
// which member is read. Rust fields must be contiguous, so native class fields,
// nested types and methods are emitted as separate source-ordered groups.
#[repr(C)]
#[derive(Clone, Copy)]
union BranchTarget {
    fSubtree: *mut Node,
    fOpIndex: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Branch {
    target: BranchTarget,
    fBounds: SkRect,
}
#[repr(C)]
struct Node {
    fNumChildren: u16,
    fLevel: u16,
    fChildren: [Branch; kMaxChildren as usize],
}

// Source: src/core/SkRTree.h:80-83
struct RTreeState {
    fCount: i32,
    fRoot: MaybeUninit<Branch>,
    fNodes: Vec<Node>,
}

// Source: src/core/SkRTree.h:34-34
pub struct SkRTree {
    // UnsafeCell expresses native mutation through a retained hierarchy pointer.
    // The unsafe insertion interface requires exclusive access until publication.
    state: UnsafeCell<RTreeState>,
    // Native base destroys after derived fields. No secondary count is used.
    fRefCnt: SkRefCnt,
}
unsafe impl SkRefCntObject for SkRTree {
    // Native complete-object virtual disposal dispatch; this actual concrete
    // Sized type pairs the original raw allocation with its typed adapter.
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }

    fn ref_cnt_base(&self) -> &SkRefCntBase {
        self.fRefCnt.ref_cnt_base()
    }
    fn ref_(&self) {
        self.fRefCnt.ref_();
    }
}
// Native trees are shared read-only after insertion. The only mutating API is
// unsafe and explicitly excludes concurrent reads and writes.
unsafe impl Send for SkRTree {}
unsafe impl Sync for SkRTree {}

impl SkRTree {
    // Source: src/core/SkRTree.h:45-45
    pub fn getDepth(&self) -> i32 {
        let state = unsafe { &*self.state.get() };
        if state.fCount != 0 {
            let root = unsafe { state.fRoot.assume_init_ref() };
            i32::from(unsafe { (*root.target.fSubtree).fLevel }) + 1
        } else {
            0
        }
    }
    // Source: src/core/SkRTree.h:47-47
    pub fn getCount(&self) -> i32 {
        unsafe { (*self.state.get()).fCount }
    }

    // Source: src/core/SkRTree.cpp:13-13
    pub fn new() -> Self {
        Self {
            state: UnsafeCell::new(RTreeState {
                fCount: 0,
                fRoot: MaybeUninit::uninit(),
                fNodes: Vec::new(),
            }),
            fRefCnt: SkRefCnt::new(),
        }
    }

    // Source: src/core/SkRTree.cpp:15-47
    /// # Safety
    /// boundsArray has at least N elements, N is nonnegative, and the complete
    /// tree is exclusively accessible for this insertion. Native inserts once.
    pub unsafe fn insert(&self, boundsArray: &[SkRect], N: i32) {
        let state = unsafe { &mut *self.state.get() };
        debug_assert_eq!(0, state.fCount);
        let mut branches: Vec<Branch> = Vec::new();
        branches.reserve(N as usize);
        for i in 0..N {
            let bounds = boundsArray[i as usize];
            if bounds.isEmpty() {
                continue;
            }
            let b = Branch {
                fBounds: bounds,
                target: BranchTarget { fOpIndex: i },
            };
            branches.push(b);
        }
        state.fCount = branches.len() as i32;
        if state.fCount != 0 {
            if state.fCount == 1 {
                state.fNodes.reserve(1);
                let n = Self::allocateNodeAtLevel(state, 0);
                unsafe {
                    (*n).fNumChildren = 1;
                    (*n).fChildren[0] = branches[0];
                }
                state.fRoot.write(Branch {
                    target: BranchTarget { fSubtree: n },
                    fBounds: branches[0].fBounds,
                });
            } else {
                state
                    .fNodes
                    .reserve(Self::CountNodes(state.fCount) as usize);
                let root = Self::bulkLoad(state, &mut branches, 0);
                state.fRoot.write(root);
            }
        }
    }

    // Source: src/core/SkRTree.cpp:49-57
    fn allocateNodeAtLevel(state: &mut RTreeState, level: u16) -> *mut Node {
        #[cfg(sk_debug)]
        let p = state.fNodes.as_ptr();
        // Native Node{} value-initializes the scalar members and child array.
        state.fNodes.push(Node {
            fNumChildren: 0,
            fLevel: 0,
            fChildren: [Branch {
                target: BranchTarget {
                    fSubtree: std::ptr::null_mut(),
                },
                fBounds: SkRect::MakeEmpty(),
            }; kMaxChildren as usize],
        });
        #[cfg(sk_debug)]
        debug_assert_eq!(state.fNodes.as_ptr(), p);
        let out = state.fNodes.last_mut().expect("just pushed node");
        out.fNumChildren = 0;
        out.fLevel = level;
        out as *mut Node
    }

    // Source: src/core/SkRTree.cpp:60-92
    pub fn CountNodes(branches: i32) -> i32 {
        if branches == 1 {
            return 1;
        }
        let mut remainder = branches % kMaxChildren;
        if remainder > 0 {
            if remainder >= kMinChildren {
                remainder = 0;
            } else {
                remainder = kMinChildren - remainder;
            }
        }
        let mut currentBranch = 0;
        let mut nodes = 0;
        while currentBranch < branches {
            let mut incrementBy = kMaxChildren;
            if remainder != 0 {
                if remainder <= kMaxChildren - kMinChildren {
                    incrementBy -= remainder;
                    remainder = 0;
                } else {
                    incrementBy = kMinChildren;
                    remainder -= kMaxChildren - kMinChildren;
                }
            }
            nodes += 1;
            currentBranch += 1;
            let mut k = 1;
            while k < incrementBy && currentBranch < branches {
                currentBranch += 1;
                k += 1;
            }
        }
        nodes + Self::CountNodes(nodes)
    }

    // Source: src/core/SkRTree.cpp:94-145
    fn bulkLoad(state: &mut RTreeState, branches: &mut Vec<Branch>, level: i32) -> Branch {
        if branches.len() == 1 {
            return branches[0];
        }
        // Native deliberately preserves Blink's existing x/y order without sorting.
        let mut remainder = branches.len() as i32 % kMaxChildren;
        let mut newBranches = 0;
        if remainder > 0 {
            if remainder >= kMinChildren {
                remainder = 0;
            } else {
                remainder = kMinChildren - remainder;
            }
        }
        let mut currentBranch = 0;
        while currentBranch < branches.len() as i32 {
            let mut incrementBy = kMaxChildren;
            if remainder != 0 {
                if remainder <= kMaxChildren - kMinChildren {
                    incrementBy -= remainder;
                    remainder = 0;
                } else {
                    incrementBy = kMinChildren;
                    remainder -= kMaxChildren - kMinChildren;
                }
            }
            let n = Self::allocateNodeAtLevel(state, level as u16);
            unsafe {
                (*n).fNumChildren = 1;
                (*n).fChildren[0] = branches[currentBranch as usize];
            }
            let mut b = Branch {
                fBounds: branches[currentBranch as usize].fBounds,
                target: BranchTarget { fSubtree: n },
            };
            currentBranch += 1;
            let mut k = 1;
            while k < incrementBy && currentBranch < branches.len() as i32 {
                b.fBounds.join(&branches[currentBranch as usize].fBounds);
                unsafe {
                    (*n).fChildren[k as usize] = branches[currentBranch as usize];
                    (*n).fNumChildren += 1;
                }
                currentBranch += 1;
                k += 1;
            }
            branches[newBranches] = b;
            newBranches += 1;
        }
        branches.truncate(newBranches);
        Self::bulkLoad(state, branches, level + 1)
    }

    // Source: src/core/SkRTree.cpp:147-151
    pub fn search(&self, query: &SkRect, results: &mut Vec<i32>) {
        let state = unsafe { &*self.state.get() };
        if state.fCount > 0 {
            let root = unsafe { state.fRoot.assume_init_ref() };
            if SkRect::Intersects(&root.fBounds, query) {
                unsafe {
                    Self::search_node(root.target.fSubtree, query, results);
                }
            }
        }
    }

    // Source: src/core/SkRTree.cpp:153-163
    unsafe fn search_node(node: *const Node, query: &SkRect, results: &mut Vec<i32>) {
        let node = unsafe { &*node };
        for i in 0..node.fNumChildren {
            let child = &node.fChildren[i as usize];
            if SkRect::Intersects(&child.fBounds, query) {
                if node.fLevel == 0 {
                    results.push(unsafe { child.target.fOpIndex });
                } else {
                    unsafe {
                        Self::search_node(child.target.fSubtree, query, results);
                    }
                }
            }
        }
    }

    // Source: src/core/SkRTree.cpp:165-171
    pub fn bytesUsed(&self) -> usize {
        let state = unsafe { &*self.state.get() };
        let mut byteCount = size_of::<SkRTree>();
        byteCount += state.fNodes.capacity() * size_of::<Node>();
        byteCount
    }
}

// Source: src/core/SkRTree.h:38-40
impl SkBBoxHierarchy for SkRTree {
    unsafe fn insert(&self, bounds: &[SkRect], N: i32) {
        unsafe {
            SkRTree::insert(self, bounds, N);
        }
    }
    fn search(&self, query: &SkRect, results: &mut Vec<i32>) {
        SkRTree::search(self, query, results);
    }
    fn bytesUsed(&self) -> usize {
        SkRTree::bytesUsed(self)
    }
}
