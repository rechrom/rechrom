//! Chromium 6c1d401f: FragmentDirective.cc:28-61,181-209;
//! text_fragment_selector.cc:16-143; text_directive.cc:16-88.
//! Pure URL microsyntax and the Document's shared directive identities.
use std::{
    cell::{Cell, OnceCell, RefCell},
    collections::HashSet,
    rc::Rc,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectorType {
    Invalid,
    Exact,
    Range,
}
#[derive(Clone, Debug)]
pub struct TextFragmentSelector<T = String> {
    pub kind: SelectorType,
    pub start: T,
    pub end: T,
    pub prefix: T,
    pub suffix: T,
}
impl TextFragmentSelector {
    pub fn FromTextDirective(directive: &str) -> Option<Self> {
        debug_assert!(!directive.contains('&'));
        if crate::url_escape::HasInvalidUrlEscapeSequences(directive) {
            return None;
        }
        let mut terms: Vec<_> = directive.split(',').collect();
        if terms.is_empty() || terms.len() > 4 {
            return None;
        }
        let valid = |term: &str| !term.is_empty() && !term.contains('-');
        let mut prefix = "";
        let mut suffix = "";
        if terms[0].ends_with('-') {
            prefix = terms.remove(0).strip_suffix('-').expect("source prefix");
            if !valid(prefix) || terms.is_empty() {
                return None;
            }
        }
        if terms.last().expect("source term").starts_with('-') {
            suffix = terms
                .pop()
                .expect("source suffix")
                .strip_prefix('-')
                .expect("source suffix");
            if !valid(suffix) || terms.is_empty() {
                return None;
            }
        }
        if terms.len() > 2 || !valid(terms[0]) {
            return None;
        }
        let start = terms[0];
        let end = if terms.len() == 2 {
            if !valid(terms[1]) {
                return None;
            }
            terms[1]
        } else {
            ""
        };
        let decode = crate::url_escape::DecodeUrlEscapeSequencesUtf8;
        Some(Self {
            kind: if terms.len() == 2 {
                SelectorType::Range
            } else {
                SelectorType::Exact
            },
            start: decode(start),
            end: decode(end),
            prefix: decode(prefix),
            suffix: decode(suffix),
        })
    }
    pub fn ToString(&self) -> String {
        let escape =
            |s: &str| crate::url_escape::EncodeWithUrlEscapeSequences(s).replace('-', "%2D");
        let mut result = String::new();
        if !self.prefix.is_empty() {
            result.push_str(&escape(&self.prefix));
            result.push_str("-,");
        }
        result.push_str(&escape(&self.start));
        if !self.end.is_empty() {
            result.push(',');
            result.push_str(&escape(&self.end));
        }
        if !self.suffix.is_empty() {
            result.push_str(",-");
            result.push_str(&escape(&self.suffix));
        }
        result
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDirectiveBehavior {
    SharedHighlight,
    ScrollOnly,
}
pub struct TextDirective {
    matching_state: OnceCell<Rc<dyn std::any::Any>>,
    pub selector: TextFragmentSelector,
    pub behavior: TextDirectiveBehavior,
}
impl TextDirective {
    pub fn Create(value: &str, behavior: TextDirectiveBehavior) -> Option<Rc<Self>> {
        Some(Rc::new(Self {
            matching_state: OnceCell::new(),
            selector: TextFragmentSelector::FromTextDirective(value)?,
            behavior,
        }))
    }
    /// The native SelectorDirective supplement retains its actual typed range
    /// owner. The parser package never stores a copied boundary/JS projection.
    pub fn MatchingState(&self) -> Option<Rc<dyn std::any::Any>> {
        self.matching_state.get().cloned()
    }
    pub fn InstallMatchingState(&self, state: Rc<dyn std::any::Any>) {
        assert!(
            self.matching_state.set(state).is_ok(),
            "one native SelectorDirective state"
        );
    }
    pub fn ToString(&self) -> String {
        format!("text={}", self.selector.ToString())
    }
}
#[derive(Default)]
pub struct FragmentDirective {
    last_navigation_had_fragment_directive: Cell<bool>,
    directive_string_length: Cell<usize>,
    directives: RefCell<Vec<Rc<TextDirective>>>,
}
impl FragmentDirective {
    pub fn LastNavigationHadFragmentDirective(&self) -> bool {
        self.last_navigation_had_fragment_directive.get()
    }
    pub fn HasTextDirective(&self) -> bool {
        !self.directives.borrow().is_empty()
    }
    pub fn Items(&self) -> Vec<Rc<TextDirective>> {
        self.directives.borrow().clone()
    }
    pub fn FragmentDirectiveStringLength(&self) -> usize {
        self.directive_string_length.get()
    }
    pub fn ConsumeFragmentDirective(&self, url: &str) -> String {
        let fragment = url.split_once('#').map_or("", |(_, fragment)| fragment);
        let start = fragment.find(":~:");
        self.last_navigation_had_fragment_directive
            .set(start.is_some());
        self.directive_string_length.set(0);
        let Some(start) = start else {
            return url.to_owned();
        };
        let directive = &fragment[start + 3..];
        self.directive_string_length
            .set(directive.encode_utf16().count());
        self.ParseDirectives(directive);
        let base = url.split_once('#').expect("source fragment delimiter").0;
        if start == 0 {
            base.to_owned()
        } else {
            format!("{base}#{}", &fragment[..start])
        }
    }
    fn ParseDirectives(&self, value: &str) {
        let mut directives = Vec::new();
        let mut seen = HashSet::new();
        for item in value.split('&') {
            let Some(value) = item.strip_prefix("text=") else {
                continue;
            };
            if value.is_empty()
                || (crate::RuntimeEnabledFeatures::ScrollToTextFragmentUniqueFragmentsEnabled()
                    && !seen.insert(value.to_ascii_lowercase()))
            {
                continue;
            }
            if let Some(directive) =
                TextDirective::Create(value, TextDirectiveBehavior::SharedHighlight)
            {
                directives.push(directive);
            }
        }
        *self.directives.borrow_mut() = directives;
    }
}
