//! Blink's user-agent regexp owner over the canonical JavaScript regexp engine.
//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15:
//! platform/bindings/script_regexp.cc:48-183. Its private context is separate
//! from author globals, so overridden RegExp/exec cannot affect HTML validation.
use super::adapter::{Context, Value};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy)]
pub enum RegexpCaseSensitivity {
    Sensitive,
    AsciiInsensitive,
}
#[derive(Clone, Copy)]
pub enum RegexpUnicodeMode {
    Disabled,
    Unicode,
    UnicodeSets,
}
pub struct ScriptRegexpContext {
    context: RefCell<Context>,
    compile: Value,
    execute: Value,
}
impl ScriptRegexpContext {
    pub fn new() -> Rc<Self> {
        let mut context = Context::new();
        // script_regexp.cc:48: kBacktrackLimit, counted by the native VM.
        context.set_regexp_backtrack_limit(1_000_000);
        let compile = context
            .eval("(function(pattern,flags){return new RegExp(pattern,flags)})")
            .expect("private regexp compiler bootstrap");
        let execute = context
            .eval("(function(regexp,subject){const m=regexp.exec(subject);return m===null?null:[m.index,m[0].length]})")
            .expect("private regexp execution bootstrap");
        Rc::new(Self {
            context: RefCell::new(context),
            compile,
            execute,
        })
    }
    pub fn Compile(
        self: &Rc<Self>,
        pattern: &[u16],
        sensitivity: RegexpCaseSensitivity,
        multiline: bool,
        unicode: RegexpUnicodeMode,
    ) -> ScriptRegexp {
        let mut flags = String::new();
        if matches!(sensitivity, RegexpCaseSensitivity::AsciiInsensitive) {
            flags.push('i');
        }
        if multiline {
            flags.push('m');
        }
        match unicode {
            RegexpUnicodeMode::Disabled => (),
            RegexpUnicodeMode::Unicode => flags.push('u'),
            RegexpUnicodeMode::UnicodeSets => flags.push('v'),
        }
        let mut context = self.context.borrow_mut();
        let pattern = Value::Str(context.intern_utf16(pattern));
        let flags = Value::Str(context.intern(&flags));
        let (regexp, error) =
            match context.call(self.compile.clone(), Value::Undefined, &[pattern, flags]) {
                Ok(regexp) => (Some(regexp), String::new()),
                Err(error) => (None, context.format_exception(&error)),
            };
        ScriptRegexp {
            context: self.clone(),
            regexp,
            error,
        }
    }
}
pub struct ScriptRegexp {
    context: Rc<ScriptRegexpContext>,
    regexp: Option<Value>,
    error: String,
}
impl ScriptRegexp {
    pub fn IsValid(&self) -> bool {
        self.regexp.is_some()
    }
    pub fn ExceptionMessage(&self) -> &str {
        &self.error
    }
    pub fn Match(&self, value: &[u16], start: usize) -> (i32, usize) {
        let Some(regexp) = &self.regexp else {
            return (-1, 0);
        };
        if value.len() > i32::MAX as usize || start > value.len() {
            return (-1, 0);
        }
        let mut context = self.context.context.borrow_mut();
        // Source passes a substring, then adds start_from to the match index.
        // JS strings and the returned match length retain UTF-16 code units.
        let subject = Value::Str(context.intern_utf16(&value[start..]));
        let Ok(result) = context.call(
            self.context.execute.clone(),
            Value::Undefined,
            &[regexp.clone(), subject],
        ) else {
            return (-1, 0);
        };
        if result.is_null() {
            return (-1, 0);
        }
        let key = super::adapter::PropKey::from_string(&context.intern("0"));
        let Ok(index) = context.get_property(&result, &key) else {
            return (-1, 0);
        };
        let key = super::adapter::PropKey::from_string(&context.intern("1"));
        let Ok(length) = context.get_property(&result, &key) else {
            return (-1, 0);
        };
        match (index.as_number(), length.as_number()) {
            (Some(index), Some(length)) => ((index as usize + start) as i32, length as usize),
            _ => (-1, 0),
        }
    }
}
