//! EditingStyle retains its opaque declaration and the original HTML tag
//! identities. CSS property algebra remains in Style. Source editing_style.cc:
//! 782-791,2317-2337; delete_selection_command.cc:417-483.
use foundation::{CSSPropertyID as P, CSSValueID as V, QualifiedName};

#[derive(Clone)]
pub struct EditingStyle<D: Clone> {
    pub declaration: D,
    pub original_html_equivalent_tags: Vec<(P, V, QualifiedName)>,
}
impl<D: Clone> EditingStyle<D> {
    pub fn new(declaration: D) -> Self {
        Self {
            declaration,
            original_html_equivalent_tags: Vec::new(),
        }
    }
    pub fn RecordOriginalHtmlEquivalentTag(&mut self, property: P, value: V, tag: QualifiedName) {
        if !self
            .original_html_equivalent_tags
            .iter()
            .any(|(p, v, _)| *p == property && *v == value)
        {
            self.original_html_equivalent_tags
                .push((property, value, tag));
        }
    }
    pub fn HtmlEquivalent(tag: &str) -> Option<(P, V)> {
        Some(match tag {
            "strong" | "b" => (P::kFontWeight, V::kBold),
            "em" | "i" => (P::kFontStyle, V::kItalic),
            "u" => (P::kTextDecorationLine, V::kUnderline),
            "s" | "strike" => (P::kTextDecorationLine, V::kLineThrough),
            "sub" => (P::kVerticalAlign, V::kSub),
            "sup" => (P::kVerticalAlign, V::kSuper),
            _ => return None,
        })
    }
}
