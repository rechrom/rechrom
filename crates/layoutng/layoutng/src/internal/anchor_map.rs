#![allow(non_snake_case)]

use foundation::{
    gfx, DynamicTo, GCedHeapHashSet, HeapHashMap, MakeGarbageCollected, Member, PhysicalOffset,
    PhysicalRect, PhysicalSize, TransformState, Visitor,
};
use layoutng_fragment_tree::physical_fragment::PhysicalFragment;

use super::anchor_map_services::anchor_map_internal::HasRunningTransformAnimation;
use super::anchor_scope::AnchorScopedName;
use super::layout_box::LayoutBox;
use super::layout_node_metadata::Element;
use super::layout_object::LayoutObject;
use super::transform_utils::UpdateTransformState;

// cpp: layoutng/internal/anchor_map.h:20-25
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorMapSetOptions {
    kInFlow,
    kOutOfFlow,
}

// cpp: layoutng/internal/anchor_map.h:39-93
pub struct PhysicalAnchorReference {
    transform_state_: TransformState,
    element_: Member<Element>,
    next_: Member<PhysicalAnchorReference>,
    display_locks_: Member<GCedHeapHashSet<Member<Element>>>,
    is_out_of_flow_: bool,
    has_running_transform_animation_: bool,
}

#[allow(non_snake_case)]
impl PhysicalAnchorReference {
    // cpp: layoutng/internal/anchor_map.h:42-51
    pub fn new(
        element: &Element,
        transform_state: &TransformState,
        is_out_of_flow: bool,
        has_running_transform_animation: bool,
        display_locks: *mut GCedHeapHashSet<Member<Element>>,
    ) -> Self {
        Self {
            transform_state_: transform_state.clone(),
            element_: Member::from_ptr(element as *const Element as *mut Element),
            next_: Member::default(),
            display_locks_: Member::from_ptr(display_locks),
            is_out_of_flow_: is_out_of_flow,
            has_running_transform_animation_: has_running_transform_animation,
        }
    }

    // cpp: layoutng/internal/anchor_map.h:53
    // cpp: layoutng/internal/anchor_map.cc:17-21
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.element_);
        visitor.Trace(&self.next_);
        visitor.Trace(&self.display_locks_);
    }

    // cpp: layoutng/internal/anchor_map.h:56
    // cpp: layoutng/internal/anchor_map.cc:23-38
    pub fn InsertInReverseTreeOrderInto(&mut self, head_ptr: &mut Member<PhysicalAnchorReference>) {
        let mut slot = head_ptr as *mut Member<PhysicalAnchorReference>;
        loop {
            let head = unsafe { &*slot }.Get();
            debug_assert!(head.is_null() || !unsafe { &*head }.GetLayoutObject().is_null());
            debug_assert!(!self.GetLayoutObject().is_null());
            if head.is_null()
                || unsafe { &*(*head).GetLayoutObject() }
                    .IsBeforeInPreOrderDefault(unsafe { &*self.GetLayoutObject() })
            {
                self.next_ = Member::from_ptr(head);
                unsafe { *slot = Member::from_ptr(self as *mut Self) };
                break;
            }
            slot = unsafe { &mut (*head).next_ };
        }
    }

    // cpp: layoutng/internal/anchor_map.h:58
    // cpp: layoutng/internal/anchor_map_services.cc:12-14
    pub fn GetLayoutObject(&self) -> *mut LayoutObject {
        unsafe { &*self.element_.Get() }
            .container
            .node
            .GetLayoutObject()
    }

    // cpp: layoutng/internal/anchor_map.h:60-63
    pub fn TransformedBoundingRect(&self) -> PhysicalRect {
        let rect = self.transform_state_.MappedQuad().BoundingBox();
        PhysicalRect::EnclosingRect(&rect)
    }

    // cpp: layoutng/internal/anchor_map.h:65
    pub fn GetTransformState(&self) -> &TransformState {
        &self.transform_state_
    }

    // cpp: layoutng/internal/anchor_map.h:66-68
    pub fn SetTransformState(&mut self, state: &TransformState) {
        self.transform_state_ = state.clone();
    }

    // cpp: layoutng/internal/anchor_map.h:70-75
    pub fn GetElement(&self) -> &Element {
        unsafe { &*self.element_.Get() }
    }
    pub fn Next(&self) -> *mut PhysicalAnchorReference {
        self.next_.Get()
    }
    pub fn GetDisplayLocks(&self) -> &Member<GCedHeapHashSet<Member<Element>>> {
        &self.display_locks_
    }
    pub fn IsOutOfFlow(&self) -> bool {
        self.is_out_of_flow_
    }

    // cpp: layoutng/internal/anchor_map.h:79-81
    pub fn HasRunningTransformAnimation(&self) -> bool {
        self.has_running_transform_animation_
    }
}

// The two arms are pointers, just as in the source variant. Keeping the
// discriminant avoids treating a scoped name as an implicit element key.
// cpp: layoutng/internal/anchor_map.h:95
#[derive(Clone, Copy)]
pub enum AnchorKey {
    Named(*const AnchorScopedName),
    Implicit(*const Element),
}

impl From<*const AnchorScopedName> for AnchorKey {
    fn from(value: *const AnchorScopedName) -> Self {
        Self::Named(value)
    }
}
impl From<*mut AnchorScopedName> for AnchorKey {
    fn from(value: *mut AnchorScopedName) -> Self {
        Self::Named(value)
    }
}
impl From<*const Element> for AnchorKey {
    fn from(value: *const Element) -> Self {
        Self::Implicit(value)
    }
}
impl From<*mut Element> for AnchorKey {
    fn from(value: *mut Element) -> Self {
        Self::Implicit(value)
    }
}
impl From<&AnchorKey> for AnchorKey {
    fn from(value: &AnchorKey) -> Self {
        *value
    }
}

type NamedAnchorMap = HeapHashMap<Member<AnchorScopedName>, Member<PhysicalAnchorReference>>;
type ImplicitAnchorMap = HeapHashMap<Member<Element>, Member<PhysicalAnchorReference>>;

// cpp: layoutng/internal/anchor_map.h:123-127
pub struct AddResult {
    pub stored_value: *mut Member<PhysicalAnchorReference>,
    pub is_new_entry: bool,
}

// cpp: layoutng/internal/anchor_map.h:147-150
#[derive(Clone, Copy)]
pub struct AnchorMapEntry {
    pub key: AnchorKey,
    pub value: *mut PhysicalAnchorReference,
}

// cpp: layoutng/internal/anchor_map.h:103-108
// cpp: layoutng/internal/anchor_map.h:236-238
#[derive(Default)]
pub struct AnchorMap {
    named_anchors_: NamedAnchorMap,
    implicit_anchors_: ImplicitAnchorMap,
}

#[allow(non_snake_case)]
impl AnchorMap {
    // cpp: layoutng/internal/anchor_map.h:110-112
    pub fn IsEmpty(&self) -> bool {
        self.named_anchors_.is_empty() && self.implicit_anchors_.is_empty()
    }

    // cpp: layoutng/internal/anchor_map.h:114-121
    // cpp: layoutng/internal/anchor_map.h:220-226
    pub fn GetAnchorReference(&self, key: &AnchorKey) -> *const PhysicalAnchorReference {
        match *key {
            AnchorKey::Named(name) => self
                .named_anchors_
                .get(&Member::from_ptr(name as *mut AnchorScopedName))
                .map_or(std::ptr::null(), |reference| reference.Get()),
            AnchorKey::Implicit(element) => self
                .implicit_anchors_
                .get(&Member::from_ptr(element as *mut Element))
                .map_or(std::ptr::null(), |reference| reference.Get()),
        }
    }

    // cpp: layoutng/internal/anchor_map.h:128-134
    // cpp: layoutng/internal/anchor_map.h:228-234
    pub fn insert(
        &mut self,
        key: &AnchorKey,
        reference: *mut PhysicalAnchorReference,
    ) -> AddResult {
        match *key {
            AnchorKey::Named(name) => {
                let (stored_value, is_new_entry) = match self
                    .named_anchors_
                    .entry(Member::from_ptr(name as *mut AnchorScopedName))
                {
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        (entry.insert(Member::from_ptr(reference)), true)
                    }
                    std::collections::hash_map::Entry::Occupied(entry) => (entry.into_mut(), false),
                };
                AddResult {
                    stored_value,
                    is_new_entry,
                }
            }
            AnchorKey::Implicit(element) => {
                let (stored_value, is_new_entry) = match self
                    .implicit_anchors_
                    .entry(Member::from_ptr(element as *mut Element))
                {
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        (entry.insert(Member::from_ptr(reference)), true)
                    }
                    std::collections::hash_map::Entry::Occupied(entry) => (entry.into_mut(), false),
                };
                AddResult {
                    stored_value,
                    is_new_entry,
                }
            }
        }
    }

    // C++ concatenates two map iterators. A value iterator retains named-map
    // then implicit-map order without exposing foundation iterator types.
    // cpp: layoutng/internal/anchor_map.h:136-183
    pub fn iter(&self) -> std::vec::IntoIter<AnchorMapEntry> {
        let mut entries = Vec::new();
        for (key, value) in self.named_anchors_.iter() {
            entries.push(AnchorMapEntry {
                key: AnchorKey::Named(key.Get()),
                value: value.Get(),
            });
        }
        for (key, value) in self.implicit_anchors_.iter() {
            entries.push(AnchorMapEntry {
                key: AnchorKey::Implicit(key.Get()),
                value: value.Get(),
            });
        }
        entries.into_iter()
    }

    // cpp: layoutng/internal/anchor_map.h:185-188
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.named_anchors_);
        visitor.Trace(&self.implicit_anchors_);
    }

    // cpp: layoutng/internal/anchor_map.h:195-198
    // cpp: layoutng/internal/anchor_map.cc:40-72
    pub fn AnchorReference<K: Into<AnchorKey>>(
        &self,
        query_box: &LayoutBox,
        containing_block: *const LayoutObject,
        key: K,
    ) -> *const PhysicalAnchorReference {
        let mut result = self.GetAnchorReference(&key.into());
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

    // cpp: layoutng/internal/anchor_map.h:199-200
    // cpp: layoutng/internal/anchor_map.cc:74-81
    pub fn AnchorLayoutObject<K: Into<AnchorKey>>(
        &self,
        query_box: &LayoutBox,
        key: K,
    ) -> *const LayoutObject {
        let reference = self.AnchorReference(query_box, std::ptr::null(), key);
        if !reference.is_null() {
            return unsafe { &*reference }.GetLayoutObject();
        }
        std::ptr::null()
    }

    // cpp: layoutng/internal/anchor_map.h:204-208
    // cpp: layoutng/internal/anchor_map_services.cc:34-50
    pub fn Set<K: Into<AnchorKey>>(
        &mut self,
        key: K,
        object: &LayoutObject,
        transform_state: &TransformState,
        options: AnchorMapSetOptions,
        element_for_display_lock: *mut Element,
    ) {
        let mut display_locks = std::ptr::null_mut();
        if !element_for_display_lock.is_null() {
            display_locks = MakeGarbageCollected(GCedHeapHashSet::default());
            unsafe { &mut *display_locks }.insert(Member::from_ptr(element_for_display_lock));
        }
        let node = object.GetNode();
        let element = DynamicTo::<Element>(node);
        assert!(!element.is_null());
        let reference = MakeGarbageCollected(PhysicalAnchorReference::new(
            unsafe { &*element },
            transform_state,
            options == AnchorMapSetOptions::kOutOfFlow,
            HasRunningTransformAnimation(object),
            display_locks,
        ));
        self.SetReference(key.into(), reference);
    }

    // cpp: layoutng/internal/anchor_map.h:209
    // cpp: layoutng/internal/anchor_map.cc:83-113
    pub fn SetReference(&mut self, key: AnchorKey, reference: *mut PhysicalAnchorReference) {
        debug_assert!(!reference.is_null());
        debug_assert!(unsafe { &*reference }.Next().is_null());
        let result = self.insert(&key, reference);
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
        unsafe { &mut *reference }.InsertInReverseTreeOrderInto(head_ptr);
    }

    // cpp: layoutng/internal/anchor_map.h:212-217
    // cpp: layoutng/internal/anchor_map.cc:115-160
    pub fn SetFromChild(
        &mut self,
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
                if !current.GetDisplayLocks().Get().is_null() || !element_for_display_lock.is_null()
                {
                    display_locks = MakeGarbageCollected(GCedHeapHashSet::default());
                }
                if !current.GetDisplayLocks().Get().is_null() {
                    unsafe { *display_locks = (&*current.GetDisplayLocks().Get()).clone() };
                }
                if !element_for_display_lock.is_null() {
                    unsafe { &mut *display_locks }
                        .insert(Member::from_ptr(element_for_display_lock));
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
                self.SetReference(entry.key, parent_reference);
                reference = current.Next();
            }
        }
    }
}

impl<'a> IntoIterator for &'a AnchorMap {
    type Item = AnchorMapEntry;
    type IntoIter = std::vec::IntoIter<AnchorMapEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
