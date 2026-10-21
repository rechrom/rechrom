use crate::src::core::SkVertices::VertexMode;
use core::mem::MaybeUninit;
// Source: src/core/SkVertState.h:22-23
// A slice borrows the native index storage and does not copy it. MaybeUninit
// preserves the three native indices' uninitialized state until a proc succeeds.
pub struct VertState<'indices> {
    pub f0: MaybeUninit<i32>,
    pub f1: MaybeUninit<i32>,
    pub f2: MaybeUninit<i32>,
    // Source: src/core/SkVertState.h:50-50
    fCount: i32,
    // Source: src/core/SkVertState.h:51-51
    fCurrIndex: i32,
    // Source: src/core/SkVertState.h:52-52
    fIndices: Option<&'indices [u16]>,
}
// Source: src/core/SkVertState.h:29-37
impl<'indices> VertState<'indices> {
    pub fn new(v_count: i32, indices: Option<&'indices [u16]>, index_count: i32) -> Self {
        Self {
            f0: MaybeUninit::uninit(),
            f1: MaybeUninit::uninit(),
            f2: MaybeUninit::uninit(),
            fCurrIndex: 0,
            fCount: if indices.is_some() {
                index_count
            } else {
                v_count
            },
            fIndices: indices,
        }
    }
    // Source: src/core/SkVertState.h:47-47
    pub fn currIndex(&self) -> i32 {
        self.fCurrIndex
    }
}
// Source: src/core/SkVertState.h:39-39
pub type Proc<'indices> = fn(&mut VertState<'indices>) -> bool;
// Source: src/core/SkVertState.h:54-54
// Source: src/core/SkVertState.cpp:10-20
impl VertState<'_> {
    fn Triangles(state: &mut Self) -> bool {
        let index = state.fCurrIndex;
        if index + 3 > state.fCount {
            return false;
        }
        state.f0.write(index + 0);
        state.f1.write(index + 1);
        state.f2.write(index + 2);
        state.fCurrIndex = index + 3;
        true
    }
    // Source: src/core/SkVertState.h:55-55
    // Source: src/core/SkVertState.cpp:22-33
    fn TrianglesX(state: &mut Self) -> bool {
        let indices = state.fIndices.expect("native indexed topology");
        let index = state.fCurrIndex;
        if index + 3 > state.fCount {
            return false;
        }
        state.f0.write(indices[index as usize + 0] as i32);
        state.f1.write(indices[index as usize + 1] as i32);
        state.f2.write(indices[index as usize + 2] as i32);
        state.fCurrIndex = index + 3;
        true
    }
    // Source: src/core/SkVertState.h:56-56
    // Source: src/core/SkVertState.cpp:35-50
    fn TriangleStrip(state: &mut Self) -> bool {
        let index = state.fCurrIndex;
        if index + 3 > state.fCount {
            return false;
        }
        state.f2.write(index + 2);
        if index & 1 != 0 {
            state.f0.write(index + 1);
            state.f1.write(index + 0);
        } else {
            state.f0.write(index + 0);
            state.f1.write(index + 1);
        }
        state.fCurrIndex = index + 1;
        true
    }
    // Source: src/core/SkVertState.h:57-57
    // Source: src/core/SkVertState.cpp:52-68
    fn TriangleStripX(state: &mut Self) -> bool {
        let indices = state.fIndices.expect("native indexed topology");
        let index = state.fCurrIndex;
        if index + 3 > state.fCount {
            return false;
        }
        state.f2.write(indices[index as usize + 2] as i32);
        if index & 1 != 0 {
            state.f0.write(indices[index as usize + 1] as i32);
            state.f1.write(indices[index as usize + 0] as i32);
        } else {
            state.f0.write(indices[index as usize + 0] as i32);
            state.f1.write(indices[index as usize + 1] as i32);
        }
        state.fCurrIndex = index + 1;
        true
    }
    // Source: src/core/SkVertState.h:58-58
    // Source: src/core/SkVertState.cpp:70-80
    fn TriangleFan(state: &mut Self) -> bool {
        let index = state.fCurrIndex;
        if index + 3 > state.fCount {
            return false;
        }
        state.f0.write(0);
        state.f1.write(index + 1);
        state.f2.write(index + 2);
        state.fCurrIndex = index + 1;
        true
    }
    // Source: src/core/SkVertState.h:59-59
    // Source: src/core/SkVertState.cpp:82-93
    fn TriangleFanX(state: &mut Self) -> bool {
        let indices = state.fIndices.expect("native indexed topology");
        let index = state.fCurrIndex;
        if index + 3 > state.fCount {
            return false;
        }
        state.f0.write(indices[0] as i32);
        state.f1.write(indices[index as usize + 1] as i32);
        state.f2.write(indices[index as usize + 2] as i32);
        state.fCurrIndex = index + 1;
        true
    }
}
// Source: src/core/SkVertState.cpp:95-106
impl<'indices> VertState<'indices> {
    // Source: src/core/SkVertState.h:45-45
    pub fn chooseProc(&self, mode: VertexMode) -> Option<Proc<'indices>> {
        self.chooseProc_raw(mode as i32)
    }
    // Raw discriminant retains the native default/null branch without creating
    // an invalid Rust enum. The typed entry only supplies valid enum values.
    pub fn chooseProc_raw(&self, mode: i32) -> Option<Proc<'indices>> {
        match mode {
            0 => Some(if self.fIndices.is_some() {
                Self::TrianglesX
            } else {
                Self::Triangles
            }),
            1 => Some(if self.fIndices.is_some() {
                Self::TriangleStripX
            } else {
                Self::TriangleStrip
            }),
            2 => Some(if self.fIndices.is_some() {
                Self::TriangleFanX
            } else {
                Self::TriangleFan
            }),
            _ => None,
        }
    }
}
