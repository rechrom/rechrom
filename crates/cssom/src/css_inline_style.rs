//! Retained inline declaration ownership, independent of the parsing engine.
#![allow(non_snake_case)]
use std::{
    any::Any,
    cell::{OnceCell, RefCell},
    fmt,
    rc::Rc,
};

type LazySourceSerializer = Box<dyn FnOnce() -> foundation::String>;

struct InlineStyleData {
    // The declaration text. A CSSOM write retains the edited property set as
    // the owner and fills this projection only when the style attribute is
    // synchronized (Element::SynchronizeStyleAttributeInternal,
    // core/dom/element.cc at
    // 6c1d401fcca5e1b0030563a90c2f2bba168e0c15): the serializer runs at most
    // once, on the first read.
    source: OnceCell<foundation::String>,
    serializer: RefCell<Option<LazySourceSerializer>>,
    parsed: RefCell<Option<Rc<dyn Any>>>,
}

/// The value currently owned by an element's inline declaration. Style keeps
/// its concrete property-set type private, as it does for stylesheet backing.
/// Clones share an immutable declaration value; an accepted property mutation
/// installs a new value, so clones and in-flight resolver inputs stay valid.
#[derive(Clone)]
pub struct CSSInlineStyle(Rc<InlineStyleData>);

impl CSSInlineStyle {
    fn WithSource(source: foundation::String) -> Self {
        let cell = OnceCell::new();
        let _ = cell.set(source);
        Self(Rc::new(InlineStyleData {
            source: cell,
            serializer: RefCell::new(None),
            parsed: RefCell::new(None),
        }))
    }
    pub fn new(source: &str) -> Self {
        Self::WithSource(foundation::String::from(source))
    }

    pub fn FromUTF16(source: &[u16]) -> Self {
        Self::WithSource(foundation::String::from_utf16(source))
    }
    /// Adopt an already-owned WTF String (one shared StringImpl), as
    /// SetSynchronizedLazyAttribute stores the serializer's result without
    /// copying its code units (element.cc:13671-13676).
    pub fn FromString(source: foundation::String) -> Self {
        if source.IsNull() {
            // A declaration's source is never the null String.
            return Self::FromUTF16(&[]);
        }
        Self::WithSource(source)
    }
    /// A declaration whose owner is its retained property set (Blink's
    /// MutableCSSPropertyValueSet held by ElementData). The text projection is
    /// produced by `serialize` (StylePropertySerializer::AsText) only when a
    /// reader asks for it, as InlineCSSStyleDeclaration::DidMutate only marks
    /// the style attribute dirty (core/css/inline_css_style_declaration.cc:
    /// 36-55; core/dom/element.cc SetInlineStyleNeedsSync /
    /// ElementData::SetStyleAttributeIsDirty).
    pub fn FromLazySource(serialize: impl FnOnce() -> foundation::String + 'static) -> Self {
        Self(Rc::new(InlineStyleData {
            source: OnceCell::new(),
            serializer: RefCell::new(Some(Box::new(serialize))),
            parsed: RefCell::new(None),
        }))
    }
    /// True once the text projection exists (always for text-sourced values).
    pub fn IsSourceSynchronized(&self) -> bool {
        self.0.source.get().is_some()
    }
    /// The exact source value used by parsing, CSSOM and DOM attribute
    /// mutation. SourceText remains a display/legacy UTF-8 projection only.
    /// A lazily owned declaration is serialized here, once.
    pub fn SourceTextValue(&self) -> &foundation::String {
        self.0.source.get_or_init(|| {
            let serialize = self
                .0
                .serializer
                .borrow_mut()
                .take()
                .expect("a lazy inline declaration retains its serializer until first read");
            let source = serialize();
            if source.IsNull() {
                foundation::String::from_utf16(&[])
            } else {
                source
            }
        })
    }
    pub fn SourceTextUTF16(&self) -> &[u16] {
        self.SourceTextValue().Span16().unwrap_or_default()
    }
    pub fn SourceText(&self) -> &str {
        self.SourceTextValue().as_str()
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
        Rc::ptr_eq(&self.0, &other.0) || self.SourceTextValue() == other.SourceTextValue()
    }
}
impl Eq for CSSInlineStyle {}
