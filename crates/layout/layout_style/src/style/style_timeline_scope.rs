use foundation::{AtomicString, Vector};

// cpp: layoutng_style/style/style_timeline_scope.h:16
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum StyleTimelineScopeType {
    kNone,
    kAll,
    kNames,
}

// cpp: layoutng_style/style/style_timeline_scope.h:14-34
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleTimelineScope {
    type_: StyleTimelineScopeType,
    names_: Vector<AtomicString>,
}

#[allow(non_snake_case)]
impl StyleTimelineScope {
    // cpp: layoutng_style/style/style_timeline_scope.h:19-20
    pub fn new(type_: StyleTimelineScopeType, names: Vector<AtomicString>) -> Self {
        Self {
            type_,
            names_: names,
        }
    }

    // cpp: layoutng_style/style/style_timeline_scope.h:26-27
    pub fn IsNone(&self) -> bool {
        self.type_ == StyleTimelineScopeType::kNone
    }
    pub fn IsAll(&self) -> bool {
        self.type_ == StyleTimelineScopeType::kAll
    }

    // cpp: layoutng_style/style/style_timeline_scope.h:29
    pub fn Names(&self) -> &Vector<AtomicString> {
        &self.names_
    }
}

// cpp: layoutng_style/style/style_timeline_scope.h:18
impl Default for StyleTimelineScope {
    fn default() -> Self {
        Self {
            type_: StyleTimelineScopeType::kNone,
            names_: Vector::default(),
        }
    }
}
