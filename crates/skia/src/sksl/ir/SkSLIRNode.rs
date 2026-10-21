// Copyright 2016 Google Inc. BSD-style license.
use crate::src::sksl::{SkSLPool::Poolable, SkSLPosition::Position};
use core::{marker::PhantomPinned, ops::Deref, pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLIRNode.h:19-30
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgramElementKind {
    kExtension = 0,
    kFunction = 1,
    kFunctionPrototype = 2,
    kGlobalVar = 3,
    kInterfaceBlock = 4,
    kModifiers = 5,
    kStructDefinition = 6,
}
impl ProgramElementKind {
    pub const kFirst: Self = Self::kExtension;
    pub const kLast: Self = Self::kStructDefinition;
}
// Source: src/sksl/ir/SkSLIRNode.h:32-41
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    kExternal = 7,
    kField = 8,
    kFunctionDeclaration = 9,
    kType = 10,
    kVariable = 11,
}
impl SymbolKind {
    pub const kFirst: Self = Self::kExternal;
    pub const kLast: Self = Self::kVariable;
}
// Source: src/sksl/ir/SkSLIRNode.h:43-60
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatementKind {
    kBlock = 12,
    kBreak = 13,
    kContinue = 14,
    kDiscard = 15,
    kDo = 16,
    kExpression = 17,
    kFor = 18,
    kIf = 19,
    kNop = 20,
    kReturn = 21,
    kSwitch = 22,
    kSwitchCase = 23,
    kVarDeclaration = 24,
}
impl StatementKind {
    pub const kFirst: Self = Self::kBlock;
    pub const kLast: Self = Self::kVarDeclaration;
}
// Source: src/sksl/ir/SkSLIRNode.h:62-92
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExpressionKind {
    kBinary = 25,
    kChildCall = 26,
    kConstructorArray = 27,
    kConstructorArrayCast = 28,
    kConstructorCompound = 29,
    kConstructorCompoundCast = 30,
    kConstructorDiagonalMatrix = 31,
    kConstructorMatrixResize = 32,
    kConstructorScalarCast = 33,
    kConstructorSplat = 34,
    kConstructorStruct = 35,
    kEmpty = 36,
    kFieldAccess = 37,
    kFunctionReference = 38,
    kFunctionCall = 39,
    kIndex = 40,
    kLiteral = 41,
    kMethodReference = 42,
    kPoison = 43,
    kPostfix = 44,
    kPrefix = 45,
    kSetting = 46,
    kSwizzle = 47,
    kTernary = 48,
    kTypeReference = 49,
    kVariableReference = 50,
}
impl ExpressionKind {
    pub const kFirst: Self = Self::kBinary;
    pub const kLast: Self = Self::kVariableReference;
}
// Source: src/sksl/ir/SkSLIRNode.h:100-100
// Adapted polymorphic unique_ptr/virtual destructor ownership. Uses the actual
// dynamic destructor and inherited Poolable delete; no parallel allocation/counter.
impl<T: ?Sized> Drop for IRNodeBox<T> {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.ptr.as_ptr());
            Poolable::operator_delete(self.ptr.as_ptr().cast::<u8>());
        }
    }
}
// Source: src/sksl/ir/SkSLIRNode.h:102-102
// Abstract native virtual protocol. No Clone or concrete/empty description is
// provided. Concrete IR owners supply real description, base access and Drop.
pub trait IRNode {
    // Native std::string owns arbitrary bytes; do not impose UTF-8 validity.
    fn description(&self) -> Vec<u8>;
    // Uncounted Rust projections of native inheritance, not alternative state.
    fn node(&self) -> &IRNodeData;
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData>;
}
// Native template static type tag (T::kIRNodeKind). This carries no object state.
pub trait IRNodeType {
    const kIRNodeKind: i32;
}
// Source: src/sksl/ir/SkSLIRNode.h:109-109
// Source: src/sksl/ir/SkSLIRNode.h:149-149
// Adapted: split Rust base data retains native fPosition then fKind order.
// PhantomPinned is a zero-size language constraint; no second state/owner.
#[repr(C)]
pub struct IRNodeData {
    pub fPosition: Position,
    pub(crate) fKind: i32,
    _pin: PhantomPinned,
}
impl IRNodeData {
    // Source: src/sksl/ir/SkSLIRNode.h:111-113
    pub fn position(&self) -> Position {
        self.fPosition
    }
    // Source: src/sksl/ir/SkSLIRNode.h:115-117
    pub fn setPosition(self: Pin<&mut Self>, p: Position) {
        unsafe {
            self.get_unchecked_mut().fPosition = p;
        }
    }
    // Source: src/sksl/ir/SkSLIRNode.h:123-126
    pub fn is<T: IRNodeType>(&self) -> bool {
        self.fKind == T::kIRNodeKind
    }
    // Source: src/sksl/ir/SkSLIRNode.h:132-136
    /// Safety: this actual base is the first field, recursively, of a live T;
    /// original native downcast contract and valid shared alias leases apply.
    pub unsafe fn as_node<T: IRNodeType>(&self) -> &T {
        debug_assert!(self.is::<T>());
        &*(self as *const Self as *const T)
    }
    // Source: src/sksl/ir/SkSLIRNode.h:138-142
    /// Safety: actual base begins the initialized T; caller owns exclusive access
    /// to the whole T, and never moves its pinned storage through this projection.
    pub unsafe fn as_node_mut<T: IRNodeType>(self: Pin<&mut Self>) -> Pin<&mut T> {
        debug_assert!(self.as_ref().get_ref().is::<T>());
        Pin::new_unchecked(&mut *(self.get_unchecked_mut() as *mut Self as *mut T))
    }
    // Source: src/sksl/ir/SkSLIRNode.h:145-147
    pub fn new(position: Position, kind: i32) -> Self {
        Self {
            fPosition: position,
            fKind: kind,
            _pin: PhantomPinned,
        }
    }
}
// Language-only unique ownership bridge for Poolable polymorphic IR objects.
// This is not a new IR node or default implementation. One raw owner retains
// native object address; Rust trait metadata carries virtual destructor dispatch.
// Drop runs the actual concrete destructor BEFORE the same Poolable delete.
pub struct IRNodeBox<T: ?Sized> {
    ptr: NonNull<T>,
}
impl<T> IRNodeBox<T> {
    /// Safety: init fully initializes exactly one T at destination without
    /// unwinding; native allocation alignment covers T. The same pool must be
    /// attached whenever this object's concrete destructor/delete eventually run.
    pub unsafe fn new_with(init: impl FnOnce(*mut T)) -> Self {
        if core::mem::align_of::<T>() > crate::src::sksl::SkSLMemoryPool::kAlignment {
            std::process::abort();
        }
        let ptr =
            NonNull::new_unchecked(Poolable::operator_new(core::mem::size_of::<T>()).cast::<T>());
        init(ptr.as_ptr());
        Self { ptr }
    }
    pub fn erase<'a>(self) -> IRNodeBox<dyn IRNode + 'a>
    where
        T: IRNode + 'a,
    {
        let ptr = unsafe { NonNull::new_unchecked(self.ptr.as_ptr() as *mut (dyn IRNode + 'a)) };
        core::mem::forget(self);
        IRNodeBox { ptr }
    }
}
impl<T: ?Sized> IRNodeBox<T> {
    /// Language-only converting unique_ptr move: callback may change trait
    /// metadata only, retaining the identical initialized allocation address.
    /// Safety: returned pointer is nonnull, owns exactly the same allocation and
    /// dynamic destructor, and the correct pool remains attached at destruction.
    pub unsafe fn cast<U: ?Sized>(self, cast: impl FnOnce(*mut T) -> *mut U) -> IRNodeBox<U> {
        let ptr = NonNull::new_unchecked(cast(self.ptr.as_ptr()));
        core::mem::forget(self);
        IRNodeBox { ptr }
    }

    pub fn as_pin_mut(&mut self) -> Pin<&mut T> {
        unsafe { Pin::new_unchecked(self.ptr.as_mut()) }
    }
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }
}
impl<T: ?Sized> Deref for IRNodeBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}
