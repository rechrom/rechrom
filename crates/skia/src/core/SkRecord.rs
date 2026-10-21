use crate::include::{
    core::SkRefCnt::{SkRefCnt, SkRefCntBase, SkRefCntObject},
    private::SkTemplates::AutoTMalloc,
};
use crate::src::core::{
    SkArenaAlloc::SkArenaAlloc,
    SkRecords::{self, RecordCommand, Type},
};
use core::{
    mem::{align_of, size_of, ManuallyDrop},
    ptr,
};
// Source: src/core/SkRecord.h:33-38
// Source: src/core/SkRecord.h:173-183
// Native fields retain declaration order. Explicit member destruction below
// preserves C++ fAlloc-before-fRecords-before-base order.
pub struct SkRecord {
    // Source: src/core/SkRecord.h:176-176
    fCount: i32,
    // Source: src/core/SkRecord.h:177-177
    fReserved: i32,
    // Source: src/core/SkRecord.h:178-178
    fRecords: ManuallyDrop<AutoTMalloc<Record>>,
    // Source: src/core/SkRecord.h:182-182
    fAlloc: ManuallyDrop<SkArenaAlloc>,
    // Source: src/core/SkRecord.h:183-183
    fApproxBytesAllocated: usize,
    fBase: SkRefCnt,
}
unsafe impl SkRefCntObject for SkRecord {
    // Native complete-object virtual disposal dispatch; this actual concrete
    // Sized type pairs the original raw allocation with its typed adapter.
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &SkRefCntBase {
        self.fBase.ref_cnt_base()
    }
    fn ref_(&self) {
        self.fBase.ref_();
    }
}
impl SkRecord {
    // Source: src/core/SkRecord.h:35-35
    pub fn new() -> Self {
        Self {
            fCount: 0,
            fReserved: 0,
            fRecords: ManuallyDrop::new(AutoTMalloc::new()),
            fAlloc: ManuallyDrop::new(SkArenaAlloc::new(256)),
            fApproxBytesAllocated: 0,
            fBase: SkRefCnt::new(),
        }
    }
    // Source: src/core/SkRecord.h:39-39
    pub fn count(&self) -> i32 {
        self.fCount
    }
    // Source: src/core/SkRecord.h:45-48
    pub unsafe fn visit<F: RecordVisitor>(&self, i: i32, f: &mut F) -> F::Output {
        (*self.fRecords.get().add(i as usize)).visit(f)
    }
    // Source: src/core/SkRecord.h:54-57
    pub unsafe fn mutate<F: RecordMutator>(&mut self, i: i32, f: &mut F) -> F::Output {
        (*self.fRecords.data_mut().add(i as usize)).mutate(f)
    }
    // Source: src/core/SkRecord.h:61-68
    /// Safety: caller initializes each object before reading it; this raw allocation
    /// has no automatic element destructors, exactly native RawBytes storage.
    pub unsafe fn alloc<T>(&mut self, count: usize) -> *mut T {
        self.fApproxBytesAllocated = self.fApproxBytesAllocated.wrapping_add(
            count
                .wrapping_mul(size_of::<T>())
                .wrapping_add(align_of::<T>()),
        );
        self.fAlloc
            .makeBytesAlignedTo(count.wrapping_mul(size_of::<T>()), align_of::<T>())
            .cast()
    }
    // Source: src/core/SkRecord.h:72-78
    /// Safety: returned command slot must be placement-initialized before any visit,
    /// replace, destruction or operation that may observe the appended record.
    pub unsafe fn append<T: RecordCommand>(&mut self) -> *mut T {
        if self.fCount == self.fReserved {
            self.grow();
        }
        let ptr = self.allocCommand::<T>();
        let i = self.fCount;
        self.fCount += 1;
        Record::set(self.fRecords.data_mut().add(i as usize), ptr)
    }
    // Source: src/core/SkRecord.h:83-90
    pub unsafe fn replace<T: RecordCommand>(&mut self, i: i32) -> *mut T {
        crate::SkASSERT!(@source b"src/core/SkRecord.h\0".as_ptr(),85; @condition b"i < this->count()\0".as_ptr(); i<self.count());
        let mut destroyer = Destroyer;
        self.mutate(i, &mut destroyer);
        let ptr = self.allocCommand::<T>();
        Record::set(self.fRecords.data_mut().add(i as usize), ptr)
    }
    // Source: src/core/SkRecord.h:126-132
    unsafe fn allocCommand<T: RecordCommand>(&mut self) -> *mut T {
        if size_of::<T>() == 0 {
            // Native per-type empty singleton has no observable bytes. The aligned
            // dangling address is Rust's stable valid singleton storage for a ZST.
            core::ptr::NonNull::<T>::dangling().as_ptr()
        } else {
            self.alloc::<T>(1)
        }
    }
}
// Source: src/core/SkRecord.h:137-155
#[derive(Clone, Copy)]
struct Record {
    // Source: src/core/SkRecord.h:136-136
    fType: Type,
    // Source: src/core/SkRecord.h:137-137
    fPtr: *mut core::ffi::c_void,
}
impl Record {
    // Source: src/core/SkRecord.h:143-150
    unsafe fn set<T: RecordCommand>(slot: *mut Self, ptr: *mut T) -> *mut T {
        slot.write(Self {
            fType: T::TYPE,
            fPtr: ptr.cast(),
        });
        debug_assert!((*slot).ptr() == ptr.cast() && (*slot).type_() == T::TYPE);
        ptr
    }
    // Source: src/core/SkRecord.h:152-153
    fn type_(&self) -> Type {
        self.fType
    }
    fn ptr(&self) -> *mut core::ffi::c_void {
        self.fPtr
    }
}
// Source: src/core/SkRecord.h:120-124
struct Destroyer;
// Source: src/core/SkRecord.h:155-174
// C++ functor overloads become required typed visitor methods. This exact native
// macro list includes every record type, with no default/opaque replay callback.
macro_rules! visitors {($($name:ident:$variant:ident),*)=>{
 pub trait RecordVisitor {type Output;$(unsafe fn $name(&mut self,record:&SkRecords::$name)->Self::Output;)*}
 pub trait RecordMutator {type Output;$(unsafe fn $name(&mut self,record:*mut SkRecords::$name)->Self::Output;)*}
 impl RecordMutator for Destroyer {type Output=();$(unsafe fn $name(&mut self,record:*mut SkRecords::$name){ptr::drop_in_place(record);})*}
 impl Record {
  unsafe fn visit<F:RecordVisitor>(&self,f:&mut F)->F::Output {match self.type_(){$(Type::$variant=>f.$name(&*self.ptr().cast::<SkRecords::$name>()),)*}}
  unsafe fn mutate<F:RecordMutator>(&mut self,f:&mut F)->F::Output {match self.type_(){$(Type::$variant=>f.$name(self.ptr().cast::<SkRecords::$name>()),)*}}
 }
};}
SkRecords::SK_RECORD_TYPES!(visitors);
// Source: src/core/SkRecord.cpp:12-17
// Source: src/core/SkRecord.h:36-36
impl Drop for SkRecord {
    fn drop(&mut self) {
        let mut destroyer = Destroyer;
        for i in 0..self.count() {
            unsafe {
                self.mutate(i, &mut destroyer);
            }
        }
        unsafe {
            ManuallyDrop::drop(&mut self.fAlloc);
            ManuallyDrop::drop(&mut self.fRecords);
        }
    }
}
impl SkRecord {
    // Source: src/core/SkRecord.cpp:19-23
    fn grow(&mut self) {
        crate::SkASSERT!(@source b"src/core/SkRecord.cpp\0".as_ptr(),20; @condition b"fCount == fReserved\0".as_ptr(); (self.fCount)==(self.fReserved));
        self.fReserved = if self.fReserved != 0 {
            self.fReserved * 2
        } else {
            4
        };
        self.fRecords.realloc(self.fReserved as usize);
    }
    // Source: src/core/SkRecord.cpp:25-28
    // Source: src/core/SkRecord.h:95-95
    pub fn bytesUsed(&self) -> usize {
        let bytes = self.fApproxBytesAllocated.wrapping_add(size_of::<Self>());
        bytes
    }
    // Source: src/core/SkRecord.cpp:30-37
    // Source: src/core/SkRecord.h:99-99
    pub fn defrag(&mut self) {
        // std::remove_if's stable compaction of the native metadata array. Payloads
        // stay at their arena addresses and NoOp has no owned data to destroy.
        let mut noops = 0i32;
        for i in 0..self.fCount {
            unsafe {
                let record = self.fRecords.data_mut().add(i as usize).read();
                if record.type_() != Type::NoOp_Type {
                    if noops != i {
                        self.fRecords.data_mut().add(noops as usize).write(record);
                    }
                    noops += 1;
                }
            }
        }
        self.fCount = noops;
    }
}
