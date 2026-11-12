//! CSSRepeatValue: css_repeat_value.h:20-49/.cc:11-60, Chromium 6c1d401fcca5.
use crate::{
    css_value::{CSSValuePayload, CSSValueRandom, CSSValueSubclass},
    production_css_value::{CSSValueList, Value},
};
use foundation::String;
use std::rc::Rc;
pub struct CSSRepeatValue {
    repetitions: Option<Rc<Value>>,
    values: Rc<Value>,
}
impl CSSRepeatValue {
    pub fn new(repetitions: Option<Rc<Value>>, values: Rc<Value>) -> Self {
        assert!(values.IsValueList());
        assert!(repetitions
            .as_ref()
            .is_none_or(|value| value.IsNumericLiteralValue() || value.IsMathFunctionValue()));
        Self {
            repetitions,
            values,
        }
    }
    pub fn Repetitions(&self) -> &Value {
        self.repetitions
            .as_ref()
            .expect("source non-auto repetitions")
            .as_ref()
    }
    pub fn IsAutoRepeatValue(&self) -> bool {
        self.repetitions.is_none()
    }
    pub fn Values(&self) -> &CSSValueList {
        self.values.ValueListView()
    }
}
impl CSSValueSubclass for CSSRepeatValue {
    fn CustomCSSText(&self) -> String {
        let mut out = String::from("repeat(");
        if let Some(count) = &self.repetitions {
            out.push_string(&count.CssText());
        } else {
            out.push_str("auto");
        }
        out.push_str(", ");
        out.push_string(&self.Values().CustomCSSText());
        out.push(')');
        out
    }
    fn Equals(&self, other: &Self) -> bool {
        let count = match (&self.repetitions, &other.repetitions) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        };
        count && self.Values().Equals(other.Values())
    }
}
impl CSSValueRandom for CSSRepeatValue {
    fn HasRandomFunctions(&self) -> bool {
        self.repetitions
            .as_ref()
            .is_some_and(|value| value.HasRandomFunctions())
            || self.values.HasRandomFunctions()
    }
}
pub fn repeat(repetitions: Option<Rc<Value>>, values: Rc<Value>) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kRepeatClass(
        CSSRepeatValue::new(repetitions, values),
    )))
}
