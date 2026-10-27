//! Native AnchorMap ordering, lookup, merge, and child-projection operations.
//!
//! Included by `anchor_map` at the reciprocal assembly boundary. Every scan
//! whose cost grows with anchor or fragment count retains an out-of-flow
//! algorithm sampling frame.

use super::*;

// cpp: layoutng/internal/anchor_map.cc:23-38
#[inline(never)]
pub(super) fn PhysicalAnchorReferenceInsertInReverseTreeOrderInto(
    reference: &mut PhysicalAnchorReference,
    head_ptr: &mut Member<PhysicalAnchorReference>,
) {
    let mut slot = head_ptr as *mut Member<PhysicalAnchorReference>;
    loop {
        let head = unsafe { &*slot }.Get();
        debug_assert!(head.is_null() || !unsafe { &*head }.GetLayoutObject().is_null());
        debug_assert!(!reference.GetLayoutObject().is_null());
        if head.is_null()
            || unsafe { &*(*head).GetLayoutObject() }
                .IsBeforeInPreOrderDefault(unsafe { &*reference.GetLayoutObject() })
        {
            reference.next_ = Member::from_ptr(head);
            unsafe { *slot = Member::from_ptr(reference as *mut PhysicalAnchorReference) };
            break;
        }
        slot = unsafe { &mut (*head).next_ };
    }
}

// cpp: layoutng/internal/anchor_map.h:136-183
#[inline(never)]
pub(super) fn AnchorMapEntries(map: &AnchorMap) -> std::vec::IntoIter<AnchorMapEntry> {
    let mut entries = Vec::new();
    for (key, value) in map.named_anchors_.iter() {
        entries.push(AnchorMapEntry {
            key: AnchorKey::Named(key.Get()),
            value: value.Get(),
        });
    }
    for (key, value) in map.implicit_anchors_.iter() {
        entries.push(AnchorMapEntry {
            key: AnchorKey::Implicit(key.Get()),
            value: value.Get(),
        });
    }
    entries.into_iter()
}

// cpp: layoutng/internal/anchor_map.cc:40-72
#[inline(never)]
pub(super) fn AnchorMapAnchorReference(
    map: &AnchorMap,
    query_box: &LayoutBox,
    containing_block: *const LayoutObject,
    key: AnchorKey,
) -> *const PhysicalAnchorReference {
    let mut result = map.GetAnchorReference(&key);
    if result.is_null() {
        return std::ptr::null();
    }
    while !result.is_null() {
        let reference = unsafe { &*result };
        let object = reference.GetLayoutObject();
        if object.is_null()
            || object == query_box as *const LayoutBox as *mut LayoutObject
            || (reference.IsOutOfFlow()
                && !unsafe { &*object }.IsBeforeInPreOrderDefault(unsafe {
                    &*(query_box as *const LayoutBox as *const LayoutObject)
                }))
        {
            result = reference.Next();
            continue;
        }
        if !containing_block.is_null()
            && !unsafe { &*(*object).Container() }.IsContainedBy(containing_block)
        {
            result = reference.Next();
            continue;
        }
        return result;
    }
    std::ptr::null()
}

// cpp: layoutng/internal/anchor_map.cc:83-113
#[inline(never)]
pub(super) fn AnchorMapSetReference(
    map: &mut AnchorMap,
    key: AnchorKey,
    reference: *mut PhysicalAnchorReference,
) {
    debug_assert!(!reference.is_null());
    debug_assert!(unsafe { &*reference }.Next().is_null());
    let result = map.insert(&key, reference);
    if result.is_new_entry {
        return;
    }
    let head_ptr = unsafe { &mut *result.stored_value };
    debug_assert!(!head_ptr.Get().is_null());
    debug_assert!(!unsafe { &*reference }.GetLayoutObject().is_null());
    let mut existing = head_ptr.Get();
    while !existing.is_null() {
        let current = unsafe { &mut *existing };
        debug_assert!(!current.GetLayoutObject().is_null());
        if current.GetLayoutObject() == unsafe { &*reference }.GetLayoutObject() {
            let mut rect = current.GetTransformState().MappedQuad().BoundingBox();
            rect.Union(
                unsafe { &*reference }
                    .GetTransformState()
                    .MappedQuad()
                    .BoundingBox(),
            );
            current.SetTransformState(&TransformState::new(
                TransformState::kApplyTransformDirection,
                gfx::QuadF::from(gfx::RectF::from(rect)),
            ));
            return;
        }
        existing = current.Next();
    }
    PhysicalAnchorReferenceInsertInReverseTreeOrderInto(unsafe { &mut *reference }, head_ptr);
}

// cpp: layoutng/internal/anchor_map.cc:115-160
#[inline(never)]
pub(super) fn AnchorMapSetFromChild(
    map: &mut AnchorMap,
    child: &PhysicalFragment,
    additional_offset: PhysicalOffset,
    container_object: &LayoutObject,
    container_size: PhysicalSize,
    options: AnchorMapSetOptions,
    element_for_display_lock: *mut Element,
) {
    let child_map = child.GetAnchorMap();
    debug_assert!(!child_map.is_null());
    for entry in unsafe { &*child_map } {
        let mut reference = entry.value;
        while !reference.is_null() {
            let current = unsafe { &*reference };
            let mut state = current.GetTransformState().clone();
            UpdateTransformState(
                child,
                additional_offset,
                container_object,
                container_size,
                &mut state,
            );

            let mut display_locks = std::ptr::null_mut();
            if !current.GetDisplayLocks().Get().is_null() || !element_for_display_lock.is_null() {
                display_locks = MakeGarbageCollected(GCedHeapHashSet::default());
            }
            if !current.GetDisplayLocks().Get().is_null() {
                unsafe { *display_locks = (&*current.GetDisplayLocks().Get()).clone() };
            }
            if !element_for_display_lock.is_null() {
                unsafe { &mut *display_locks }.insert(Member::from_ptr(element_for_display_lock));
            }
            debug_assert!(!current.GetLayoutObject().is_null());
            let child_object = child.GetLayoutObject();
            let animated = current.HasRunningTransformAnimation()
                || (!child_object.is_null()
                    && HasRunningTransformAnimation(unsafe { &*child_object }));
            let parent_reference = MakeGarbageCollected(PhysicalAnchorReference::new(
                current.GetElement(),
                &state,
                options == AnchorMapSetOptions::kOutOfFlow,
                animated,
                display_locks,
            ));
            AnchorMapSetReference(map, entry.key, parent_reference);
            reference = current.Next();
        }
    }
}
