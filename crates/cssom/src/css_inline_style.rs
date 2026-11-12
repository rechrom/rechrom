//! Retained inline declaration ownership, independent of the parsing engine.
#![allow(non_snake_case)]
use std::{any::Any, cell::RefCell, fmt, rc::Rc};

struct InlineStyleData {
    source: foundation::String,
    parsed: RefCell<Option<Rc<dyn Any>>>,
}

/// The value currently owned by an element's inline declaration. Style keeps
/// its concrete property-set type private, as it does for stylesheet backing.
/// Clones share an immutable declaration value; an accepted property mutation
/// installs a new value, so clones and in-flight resolver inputs stay valid.
#[derive(Clone)]
pub struct CSSInlineStyle(Rc<InlineStyleData>);

impl CSSInlineStyle {
    pub fn new(source: &str) -> Self {
        Self(Rc::new(InlineStyleData {
            source: foundation::String::from(source),
            parsed: RefCell::new(None),
        }))
    }

    pub fn FromUTF16(source: &[u16]) -> Self {
        Self(Rc::new(InlineStyleData {
            source: foundation::String::from_utf16(source),
            parsed: RefCell::new(None),
        }))
    }
    /// The exact source value used by parsing, CSSOM and DOM attribute
    /// mutation. SourceText remains a display/legacy UTF-8 projection only.
    pub fn SourceTextValue(&self) -> &foundation::String {
        &self.0.source
    }
    pub fn SourceTextUTF16(&self) -> &[u16] {
        self.0.source.Span16().unwrap_or_default()
    }
    pub fn SourceText(&self) -> &str {
        self.0.source.as_str()
    }

    pub fn AttachParsedBacking<T: Any>(&self, parsed: Rc<T>) {
        *self.0.parsed.borrow_mut() = Some(parsed);
    }

    pub fn ParsedBacking<T: Any>(&self) -> Option<Rc<T>> {
        self.0
            .parsed
            .borrow()
            .clone()
            .and_then(|owner| owner.downcast().ok())
    }
}

impl fmt::Debug for CSSInlineStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CSSInlineStyle")
            .field(&self.SourceText())
            .finish()
    }
}
impl PartialEq for CSSInlineStyle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0) || self.0.source == other.0.source
    }
}
impl Eq for CSSInlineStyle {}
