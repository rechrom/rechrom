//! Retained inline declaration ownership, independent of the parsing engine.
#![allow(non_snake_case)]
use std::{any::Any, cell::RefCell, fmt, rc::Rc};

struct InlineStyleData {
    source: String,
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
            source: source.into(),
            parsed: RefCell::new(None),
        }))
    }

    pub fn SourceText(&self) -> &str {
        &self.0.source
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
        Rc::ptr_eq(&self.0, &other.0) || self.SourceText() == other.SourceText()
    }
}
impl Eq for CSSInlineStyle {}
