//! Opaque handles for the base objects that fragment data names by identity.
//!
//! Chromium's fragment data holds `Member<const LayoutObject>`,
//! `Member<Element>` and `Member<PaintLayer>` edges (for example
//! `ExclusionShapeData::layout_box`, `LineClampData::clamp_after_layout_object`,
//! `PhysicalFragment::layout_object_`). The concrete classes belong to base
//! (`layout_core`, above this crate), so fragment data stores a pointer to one
//! of the zero-sized opaque types below instead. This crate only stores,
//! copies, compares and traces such pointers; it never dereferences them.
//!
//! Base recovers the concrete pointer with a plain pointer cast, for example
//! `handle.cast::<LayoutBox>()`. Every class of the LayoutObject hierarchy is
//! `#[repr(C)]` with its base class as the first field, so one
//! `OpaqueLayoutObject` handle covers LayoutObject, LayoutBoxModelObject,
//! LayoutBox, LayoutInline, LayoutText and their subclasses at the same
//! address, exactly as the C++ upcast does. `Member<Opaque*>` traces like the
//! typed member it replaces: the layout heap marks the address and does not
//! need the pointee type.
//!
//! Fragment methods that need the object's behaviour are not here; they are
//! base extension traits implemented on the fragment types (see
//! `crates/layout/PACKAGE_MAP.md` §7, R4 status), so calls stay direct.

use std::marker::{PhantomData, PhantomPinned};

/// A LayoutObject (or any subclass) owned by base.
#[repr(C)]
pub struct OpaqueLayoutObject {
    _private: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

/// A Layout DOM projection node (Node, ContainerNode, Element and the form
/// element classes) owned by base.
#[repr(C)]
pub struct OpaqueNode {
    _private: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

/// A PaintLayer owned by base.
#[repr(C)]
pub struct OpaquePaintLayer {
    _private: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}
