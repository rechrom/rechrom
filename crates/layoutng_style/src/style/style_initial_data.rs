use super::forward::{CSSValue, CSSVariableData};
use super::style_variables::StyleVariables;
use foundation::{AtomicString, HashSet, Visitor};

// cpp: layoutng_style/style/style_initial_data.h:21-62
#[derive(Default)]
pub struct StyleInitialData {
    variables_: StyleVariables,
    viewport_unit_flags_: u32,
}

#[allow(non_snake_case)]
impl StyleInitialData {
    // cpp: layoutng_style/style/style_initial_data.h:23
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.variables_);
    }
    // cpp: layoutng_style/style/style_initial_data.h:25
    // Equality has no supplied definition.
    pub fn Equals(&self, other: &Self) -> bool {
        unsafe { StyleInitialDataEquals(self, other) }
    }
    // cpp: layoutng_style/style/style_initial_data.h:27
    pub fn HasInitialVariables(&self) -> bool {
        !self.variables_.IsEmpty()
    }
    // cpp: layoutng_style/style/style_initial_data.h:29-31
    pub fn CollectVariableNames(&self, names: &mut HashSet<AtomicString>) {
        self.variables_.CollectNames(names);
    }
    // cpp: layoutng_style/style/style_initial_data.h:33-35
    pub fn GetVariableData(&self, name: &AtomicString) -> *mut CSSVariableData {
        self.variables_
            .GetData(name)
            .unwrap_or(std::ptr::null_mut())
    }
    // cpp: layoutng_style/style/style_initial_data.h:37-39
    pub fn GetVariableValue(&self, name: &AtomicString) -> *const CSSValue {
        self.variables_.GetValue(name).unwrap_or(std::ptr::null())
    }
    // cpp: layoutng_style/style/style_initial_data.h:41
    pub fn GetViewportUnitFlags(&self) -> u32 {
        self.viewport_unit_flags_
    }
}

unsafe extern "Rust" {
    fn StyleInitialDataEquals(first: &StyleInitialData, second: &StyleInitialData) -> bool;
}
