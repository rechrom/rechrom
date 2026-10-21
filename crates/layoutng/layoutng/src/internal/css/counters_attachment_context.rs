#![allow(non_snake_case)]

use foundation::{AtomicString, GCedHeapHashMap, GCedHeapVector, Member, Vector, Visitor};
use layoutng_style::style::counter_directives::CounterDirectives;

use crate::internal::layout_node_metadata::{Element, Node};
use crate::internal::layout_object::LayoutObject;

// Bodies of the non-inline members are outside the selected production set.
// Typed links keep the native counter-attachment behavior at its owner.
unsafe extern "Rust" {
    fn CountersAttachmentContextNew() -> CountersAttachmentContext;
    fn CountersAttachmentContextDeepClone(
        this: &CountersAttachmentContext,
    ) -> CountersAttachmentContext;
    fn CountersAttachmentContextEnterObject(
        this: &mut CountersAttachmentContext,
        object: &LayoutObject,
        is_page_box: bool,
    );
    fn CountersAttachmentContextLeaveObject(
        this: &mut CountersAttachmentContext,
        object: &LayoutObject,
        is_page_box: bool,
    );
    fn CountersAttachmentContextGetCounterValues(
        this: &mut CountersAttachmentContext,
        object: &LayoutObject,
        name: &AtomicString,
        only_last: bool,
    ) -> Vector<i32>;
    fn CountersAttachmentContextElementGeneratesListItemCounter(element: &Element) -> bool;
    fn CountersAttachmentContextCalculateInitialValueForReversed(
        node: &Node,
        name: &AtomicString,
        directives: &CounterDirectives,
    ) -> i32;
    fn CountersAttachmentContextProcessCounter(
        this: &mut CountersAttachmentContext,
        object: &LayoutObject,
        name: &AtomicString,
        counter_type: u32,
        value_argument: i32,
        is_page_box: bool,
    );
    fn CountersAttachmentContextObscurePageCounterIfNeeded(
        this: &mut CountersAttachmentContext,
        object: &LayoutObject,
        name: &AtomicString,
        counter_type: u32,
        value_argument: i32,
        is_page_box: bool,
    ) -> bool;
    fn CountersAttachmentContextUnobscurePageCounterIfNeeded(
        this: &mut CountersAttachmentContext,
        name: &AtomicString,
        counter_type: u32,
        is_page_box: bool,
    );
    fn CountersAttachmentContextCreateCounter(
        this: &mut CountersAttachmentContext,
        object: &LayoutObject,
        name: &AtomicString,
        value: i32,
    );
    fn CountersAttachmentContextRemoveStaleCounters(
        this: &mut CountersAttachmentContext,
        object: &LayoutObject,
        name: &AtomicString,
    );
    fn CountersAttachmentContextRemoveCounterIfAncestorExists(
        this: &mut CountersAttachmentContext,
        object: &LayoutObject,
        name: &AtomicString,
    );
    fn CountersAttachmentContextUpdateCounterValue(
        this: &mut CountersAttachmentContext,
        object: &LayoutObject,
        name: &AtomicString,
        counter_type: u32,
        counter_value: i32,
    );
    fn CountersAttachmentContextMaybeCreateListItemCounter(
        this: &mut CountersAttachmentContext,
        element: &Element,
    );
    fn CountersAttachmentContextEnterStyleContainmentScope(this: &mut CountersAttachmentContext);
    fn CountersAttachmentContextLeaveStyleContainmentScope(this: &mut CountersAttachmentContext);
}

// cpp: layoutng/internal/css/counters_attachment_context.h:27-35
pub struct CounterEntry {
    pub layout_object: Member<LayoutObject>,
    pub value: i32,
}

impl CounterEntry {
    pub fn new(layout_object: &LayoutObject, value: i32) -> Self {
        Self {
            layout_object: Member::from_ptr(
                layout_object as *const LayoutObject as *mut LayoutObject,
            ),
            value,
        }
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.layout_object);
    }
}

pub type CounterStack = GCedHeapVector<Member<CounterEntry>>;
pub type CounterInheritanceTable = GCedHeapHashMap<AtomicString, Member<CounterStack>>;

// cpp: layoutng/internal/css/counters_attachment_context.h:40-45
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    kIncrementType = 1,
    kResetType = 2,
    kSetType = 4,
}

// cpp: layoutng/internal/css/counters_attachment_context.h:47-129
#[derive(Clone)]
pub struct CountersAttachmentContext {
    pub(crate) list_item_: AtomicString,
    pub(crate) attachment_root_is_document_element_: bool,
    pub(crate) counter_inheritance_table_: *mut CounterInheritanceTable,
}

impl CountersAttachmentContext {
    pub fn new() -> Self {
        unsafe { CountersAttachmentContextNew() }
    }

    // cpp: layoutng/internal/css/counters_attachment_context.h:56-58
    pub fn ShallowClone(&self) -> Self {
        self.clone()
    }

    // cpp: layoutng/internal/css/counters_attachment_context.h:61-85
    pub fn DeepClone(&self) -> Self {
        unsafe { CountersAttachmentContextDeepClone(self) }
    }

    pub fn EnterObject(&mut self, object: &LayoutObject, is_page_box: bool) {
        unsafe { CountersAttachmentContextEnterObject(self, object, is_page_box) }
    }

    pub fn LeaveObject(&mut self, object: &LayoutObject, is_page_box: bool) {
        unsafe { CountersAttachmentContextLeaveObject(self, object, is_page_box) }
    }

    pub fn GetCounterValues(
        &mut self,
        object: &LayoutObject,
        name: &AtomicString,
        only_last: bool,
    ) -> Vector<i32> {
        unsafe { CountersAttachmentContextGetCounterValues(self, object, name, only_last) }
    }

    pub fn SetAttachmentRootIsDocumentElement(&mut self) {
        self.attachment_root_is_document_element_ = true;
    }

    pub fn AttachmentRootIsDocumentElement(&self) -> bool {
        self.attachment_root_is_document_element_
    }

    pub fn ElementGeneratesListItemCounter(element: &Element) -> bool {
        unsafe { CountersAttachmentContextElementGeneratesListItemCounter(element) }
    }

    pub fn CalculateInitialValueForReversed(
        node: &Node,
        name: &AtomicString,
        directives: &CounterDirectives,
    ) -> i32 {
        unsafe { CountersAttachmentContextCalculateInitialValueForReversed(node, name, directives) }
    }

    // cpp: layoutng/internal/css/counters_attachment_context.h:90-116
    fn ProcessCounter(
        &mut self,
        object: &LayoutObject,
        name: &AtomicString,
        counter_type: u32,
        value_argument: i32,
        is_page_box: bool,
    ) {
        unsafe {
            CountersAttachmentContextProcessCounter(
                self,
                object,
                name,
                counter_type,
                value_argument,
                is_page_box,
            )
        }
    }

    fn ObscurePageCounterIfNeeded(
        &mut self,
        object: &LayoutObject,
        name: &AtomicString,
        counter_type: u32,
        value_argument: i32,
        is_page_box: bool,
    ) -> bool {
        unsafe {
            CountersAttachmentContextObscurePageCounterIfNeeded(
                self,
                object,
                name,
                counter_type,
                value_argument,
                is_page_box,
            )
        }
    }

    fn UnobscurePageCounterIfNeeded(
        &mut self,
        name: &AtomicString,
        counter_type: u32,
        is_page_box: bool,
    ) {
        unsafe {
            CountersAttachmentContextUnobscurePageCounterIfNeeded(
                self,
                name,
                counter_type,
                is_page_box,
            )
        }
    }

    fn CreateCounter(&mut self, object: &LayoutObject, name: &AtomicString, value: i32) {
        unsafe { CountersAttachmentContextCreateCounter(self, object, name, value) }
    }

    fn RemoveStaleCounters(&mut self, object: &LayoutObject, name: &AtomicString) {
        unsafe { CountersAttachmentContextRemoveStaleCounters(self, object, name) }
    }

    fn RemoveCounterIfAncestorExists(&mut self, object: &LayoutObject, name: &AtomicString) {
        unsafe { CountersAttachmentContextRemoveCounterIfAncestorExists(self, object, name) }
    }

    fn UpdateCounterValue(
        &mut self,
        object: &LayoutObject,
        name: &AtomicString,
        counter_type: u32,
        counter_value: i32,
    ) {
        unsafe {
            CountersAttachmentContextUpdateCounterValue(
                self,
                object,
                name,
                counter_type,
                counter_value,
            )
        }
    }

    fn MaybeCreateListItemCounter(&mut self, element: &Element) {
        unsafe { CountersAttachmentContextMaybeCreateListItemCounter(self, element) }
    }

    fn EnterStyleContainmentScope(&mut self) {
        unsafe { CountersAttachmentContextEnterStyleContainmentScope(self) }
    }

    fn LeaveStyleContainmentScope(&mut self) {
        unsafe { CountersAttachmentContextLeaveStyleContainmentScope(self) }
    }
}
