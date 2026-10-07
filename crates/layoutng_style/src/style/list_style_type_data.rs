use std::cell::RefCell;

impl foundation::Traceable for ListStyleTypeData {
    fn Trace(&self, visitor: &mut foundation::Visitor<'_>) {
        ListStyleTypeData::Trace(self, visitor);
    }
}

use super::forward::{CSSSymbolsValue, CounterStyle};
use foundation::{AtomicString, MakeGarbageCollected, Member, TreeScope, Visitor, WeakMember};

// cpp: layoutng_style/style/list_style_type_data.h:29
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ListStyleType {
    kCounterStyle,
    kString,
}

// cpp: layoutng_style/style/list_style_type_data.h:24-27
// cpp: layoutng_style/style/list_style_type_data.h:73-87
pub struct ListStyleTypeData {
    type_: ListStyleType,
    name_or_string_value_: AtomicString,
    tree_scope_: WeakMember<TreeScope>,
    counter_style_: RefCell<Member<CounterStyle>>,
}

#[allow(non_snake_case)]
impl ListStyleTypeData {
    // cpp: layoutng_style/style/list_style_type_data.h:31-38
    pub fn new(
        type_: ListStyleType,
        name_or_string_value: AtomicString,
        tree_scope: *const TreeScope,
        symbols_counter_style: *const CounterStyle,
    ) -> Self {
        Self {
            type_,
            name_or_string_value_: name_or_string_value,
            tree_scope_: WeakMember::from_ptr(tree_scope.cast_mut()),
            counter_style_: RefCell::new(Member::from_ptr(symbols_counter_style.cast_mut())),
        }
    }

    // cpp: layoutng_style/style/list_style_type_data.h:31-34
    pub fn new_without_symbols_counter_style(
        type_: ListStyleType,
        name_or_string_value: AtomicString,
        tree_scope: *const TreeScope,
    ) -> Self {
        Self::new(type_, name_or_string_value, tree_scope, std::ptr::null())
    }

    // cpp: layoutng_style/style/list_style_type_data.h:27
    // cpp: layoutng_style/style/list_style_type_data.cc:7-10
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.tree_scope_);
        visitor.Trace(&*self.counter_style_.borrow());
    }

    // cpp: layoutng_style/style/list_style_type_data.h:40
    // cpp: layoutng_style/style/list_style_type_data.cc:13-15
    pub fn CreateString(value: &AtomicString) -> *mut Self {
        MakeGarbageCollected(Self::new(
            ListStyleType::kString,
            value.clone(),
            std::ptr::null(),
            std::ptr::null(),
        ))
    }

    // cpp: layoutng_style/style/list_style_type_data.h:41-42
    // cpp: layoutng_style/style/list_style_type_data.cc:18-23
    pub fn CreateCounterStyle(name: &AtomicString, tree_scope: *const TreeScope) -> *mut Self {
        MakeGarbageCollected(Self::new(
            ListStyleType::kCounterStyle,
            name.clone(),
            tree_scope,
            std::ptr::null(),
        ))
    }

    // cpp: layoutng_style/style/list_style_type_data.h:43-44
    // No definition exists in the supplied C++ tree.
    pub fn CreateSymbolsFunction(value: &CSSSymbolsValue) -> *mut Self {
        unsafe { ListStyleTypeDataCreateSymbolsFunction(value) }
    }

    // cpp: layoutng_style/style/list_style_type_data.h:48
    pub fn IsCounterStyle(&self) -> bool {
        self.type_ == ListStyleType::kCounterStyle
    }

    // cpp: layoutng_style/style/list_style_type_data.h:49
    pub fn IsString(&self) -> bool {
        self.type_ == ListStyleType::kString
    }

    // cpp: layoutng_style/style/list_style_type_data.h:50-52
    pub fn IsSymbolsFunction(&self) -> bool {
        self.type_ == ListStyleType::kCounterStyle && self.name_or_string_value_.empty()
    }

    // cpp: layoutng_style/style/list_style_type_data.h:55-58
    pub fn GetCounterStyleName(&self) -> &AtomicString {
        assert!(self.type_ == ListStyleType::kCounterStyle);
        &self.name_or_string_value_
    }

    // cpp: layoutng_style/style/list_style_type_data.h:60-63
    pub fn GetStringValue(&self) -> &AtomicString {
        assert!(self.type_ == ListStyleType::kString);
        &self.name_or_string_value_
    }

    // cpp: layoutng_style/style/list_style_type_data.h:65-69
    pub fn GetSymbolsCounterStyle(&self) -> &CounterStyle {
        assert!(self.IsSymbolsFunction());
        let counter_style = self.counter_style_.borrow().Get();
        assert!(!counter_style.is_null());
        unsafe { &*counter_style }
    }

    // cpp: layoutng_style/style/list_style_type_data.h:71
    pub fn GetTreeScope(&self) -> *const TreeScope {
        self.tree_scope_.Get()
    }
}

// cpp: layoutng_style/style/list_style_type_data.h:46
// cpp: layoutng_style/style/list_style_type_data.cc:25-30
impl PartialEq for ListStyleTypeData {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_
            && self.name_or_string_value_ == other.name_or_string_value_
            && self.tree_scope_ == other.tree_scope_
            && (!self.IsSymbolsFunction()
                || self.counter_style_.borrow().Get() == other.counter_style_.borrow().Get())
    }
}

unsafe extern "Rust" {
    fn ListStyleTypeDataCreateSymbolsFunction(value: &CSSSymbolsValue) -> *mut ListStyleTypeData;
}
