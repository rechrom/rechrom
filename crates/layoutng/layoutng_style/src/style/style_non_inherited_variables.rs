use super::forward::{CSSValue, CSSVariableData};
use super::style_variables::StyleVariables;
use foundation::{AtomicString, HashSet, Visitor};

// cpp: layoutng_style/style/style_non_inherited_variables.h:22-62
#[derive(Clone, Default)]
pub struct StyleNonInheritedVariables {
    variables_: StyleVariables,
}

#[allow(non_snake_case)]
impl StyleNonInheritedVariables {
    // cpp: layoutng_style/style/style_non_inherited_variables.h:26
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.variables_);
    }
    // cpp: layoutng_style/style/style_non_inherited_variables.h:32
    // No definition is supplied in this package.
    pub fn SetData(&mut self, name: &AtomicString, value: *mut CSSVariableData) {
        unsafe { StyleNonInheritedVariablesSetData(self, name, value) }
    }
    // cpp: layoutng_style/style/style_non_inherited_variables.h:33-35
    pub fn GetData(&self, name: &AtomicString) -> Option<*mut CSSVariableData> {
        self.variables_.GetData(name)
    }
    // cpp: layoutng_style/style/style_non_inherited_variables.h:37-42
    pub fn SetValue(&mut self, name: &AtomicString, value: *const CSSValue) {
        self.variables_.SetValue(name, value);
    }
    pub fn GetValue(&self, name: &AtomicString) -> Option<*const CSSValue> {
        self.variables_.GetValue(name)
    }
    // cpp: layoutng_style/style/style_non_inherited_variables.h:44-46
    pub fn CollectNames(&self, names: &mut HashSet<AtomicString>) {
        self.variables_.CollectNames(names);
    }
    // cpp: layoutng_style/style/style_non_inherited_variables.h:48
    pub fn IsEmpty(&self) -> bool {
        self.variables_.IsEmpty()
    }
}

// cpp: layoutng_style/style/style_non_inherited_variables.h:28-30
impl PartialEq for StyleNonInheritedVariables {
    fn eq(&self, other: &Self) -> bool {
        self.variables_ == other.variables_
    }
}

// cpp: layoutng_style/style/style_non_inherited_variables.h:50-62
impl std::fmt::Display for StyleNonInheritedVariables {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.variables_, formatter)
    }
}

unsafe extern "Rust" {
    fn StyleNonInheritedVariablesSetData(
        value: &mut StyleNonInheritedVariables,
        name: &AtomicString,
        data: *mut CSSVariableData,
    );
}
