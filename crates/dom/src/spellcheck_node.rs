//! Element's source spellcheck inheritance. The original input password-history
//! owner is read directly; no browser/default-action mirror is stored.
use super::*;
impl PersistentDocument {
    /// Source core/dom/element.cc:11862-11904. None means the document has no
    /// Page; a Page supplies its actual Settings::SpellCheckEnabledByDefault.
    pub fn IsSpellCheckingEnabledForElement(
        &self,
        node: usize,
        page_default: Option<bool>,
    ) -> bool {
        let mut current = Some(node);
        while let Some(node) = current {
            if let Some(attribute) = self.Node(node).FindAttribute("spellcheck") {
                if attribute.value.is_empty() || attribute.value.eq_ignore_ascii_case("true") {
                    return true;
                }
                if attribute.value.eq_ignore_ascii_case("false") {
                    return false;
                }
            }
            if self.Node(node).IsHTMLElement("input") && self.InputHasBeenPasswordField(node) {
                return false;
            }
            current = self.ParentOrShadowHostElement(node);
        }
        page_default.unwrap_or(true)
    }
}
