use super::forward::{CSSValue, CSSVariableData};
use super::style_variables::StyleVariables;
use foundation::{AtomicString, HashSet, Visitor};

// cpp: layoutng_style/style/style_inherited_variables.h:21-63
#[derive(Clone, Default)]
pub struct StyleInheritedVariables {
    variables_: StyleVariables,
}

#[allow(non_snake_case)]
impl StyleInheritedVariables {
    // cpp: layoutng_style/style/style_inherited_variables.h:27
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.variables_);
    }
    // cpp: layoutng_style/style/style_inherited_variables.h:33
    // No definition is supplied in this package.
    pub fn SetData(&mut self, name: &AtomicString, value: *mut CSSVariableData) {
        unsafe { StyleInheritedVariablesSetData(self, name, value) }
    }
    // cpp: layoutng_style/style/style_inherited_variables.h:34-36
    pub fn GetData(&self, name: &AtomicString) -> Option<*mut CSSVariableData> {
        self.variables_.GetData(name)
    }
    // cpp: layoutng_style/style/style_inherited_variables.h:38-43
    pub fn SetValue(&mut self, name: &AtomicString, value: *const CSSValue) {
        self.variables_.SetValue(name, value);
    }
    pub fn GetValue(&self, name: &AtomicString) -> Option<*const CSSValue> {
        self.variables_.GetValue(name)
    }
    // cpp: layoutng_style/style/style_inherited_variables.h:48-50
    pub fn CollectNames(&self, names: &mut HashSet<AtomicString>) {
        self.variables_.CollectNames(names);
    }
    // cpp: layoutng_style/style/style_inherited_variables.h:52-53
    pub fn IsEmpty(&self) -> bool {
        self.variables_.IsEmpty()
    }
    pub fn GetHash(&self) -> u32 {
        self.variables_.GetHash()
    }
}

// cpp: layoutng_style/style/style_inherited_variables.h:29-31
impl PartialEq for StyleInheritedVariables {
    fn eq(&self, other: &Self) -> bool {
        self.variables_ == other.variables_
    }
}

// cpp: layoutng_style/style/style_inherited_variables.h:56-59
impl std::fmt::Display for StyleInheritedVariables {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.variables_, formatter)
    }
}

unsafe extern "Rust" {
    fn StyleInheritedVariablesSetData(
        value: &mut StyleInheritedVariables,
        name: &AtomicString,
        data: *mut CSSVariableData,
    );
}
