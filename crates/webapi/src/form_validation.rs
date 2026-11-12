//! The UA validation regexp service uses the existing JavaScript engine's
//! isolated ScriptRegexp context, including UnicodeSets and backtrack budget.
use interaction::default::core::html::forms::{
    base_text_input_type::{ScriptRegexp, ScriptRegexpFactory},
    input_validation::ValidationPatternEngine,
};
use javascript::script_regexp::{RegexpCaseSensitivity, RegexpUnicodeMode, ScriptRegexpContext};
use std::{cell::OnceCell, rc::Rc};
struct Regexp(javascript::script_regexp::ScriptRegexp);
impl ScriptRegexp for Regexp {
    fn IsValid(&self) -> bool {
        self.0.IsValid()
    }
    fn ExceptionMessage(&self) -> String {
        self.0.ExceptionMessage().to_owned()
    }
    fn Match(&self, value: &[u16], start: usize) -> (i32, usize) {
        self.0.Match(value, start)
    }
}
struct Engine {
    context: Rc<ScriptRegexpContext>,
    email: OnceCell<Rc<dyn ScriptRegexp>>,
}
impl ScriptRegexpFactory for Engine {
    fn CreateUnicodeSetsCaseSensitive(&self, pattern: &[u16]) -> Rc<dyn ScriptRegexp> {
        Rc::new(Regexp(self.context.Compile(
            pattern,
            RegexpCaseSensitivity::Sensitive,
            false,
            RegexpUnicodeMode::UnicodeSets,
        )))
    }
    fn InvalidPatternConsoleMessage(&self, _: &[u16], _: &str) {}
}
impl ValidationPatternEngine for Engine {
    fn EnsureEmailRegexp(&self) -> Rc<dyn ScriptRegexp> {
        self.email
            .get_or_init(|| {
                Rc::new(Regexp(
                    self.context.Compile(
                        &interaction::default::core::html::forms::email_input_type::kEmailPattern
                            .encode_utf16()
                            .collect::<Vec<_>>(),
                        RegexpCaseSensitivity::AsciiInsensitive,
                        false,
                        RegexpUnicodeMode::Disabled,
                    ),
                ))
            })
            .clone()
    }
}
pub fn CreateValidationPatternEngine() -> Rc<dyn ValidationPatternEngine> {
    Rc::new(Engine {
        context: ScriptRegexpContext::new(),
        email: OnceCell::new(),
    })
}
