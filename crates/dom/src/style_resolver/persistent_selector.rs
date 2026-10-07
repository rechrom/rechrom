#![allow(non_snake_case)]
use crate::style_resolver::selector::SourceSpace;
// Persistent DOM matcher used by source MatchesSelector, without cloning or
// reparsing the tree. Selector syntax is shared with the static layout adapter.
use crate::persistent_document::{DOMNamespace, DOMNode, DOMNodeType};
use crate::style_resolver::selector::{
    MatchesAnPlusB, ParsedSelector, PseudoTarget, Specificity, SplitSelectorList,
};
use crate::Document;

// Source MatchSelector's optional Document* distinguishes public DOM queries
// from style matching, where live form-control state participates.
struct MatchContext<'a> {
    document: &'a Document,
    use_control_value: bool,
}
impl std::ops::Deref for MatchContext<'_> {
    type Target = Document;
    fn deref(&self) -> &Document {
        self.document
    }
}

// cpp: style_resolver/style_resolver.cc:48-78
pub(crate) fn split_whitespace(value: &str) -> impl Iterator<Item = &str> {
    // Keep the source tokenizer's parenthesis/space rules, but visit borrowed
    // spans lazily. Selector checks and rule-index lookups need neither a Vec
    // allocation nor the remaining tokens after a match has been found.
    let mut cursor = 0;
    let mut begin = 0;
    let mut depth = 0i32;
    std::iter::from_fn(move || {
        while cursor <= value.len() {
            let i = cursor;
            cursor += 1;
            let c = value.as_bytes().get(i).copied().unwrap_or(b' ');
            if c == b'(' {
                depth += 1;
            } else if c == b')' {
                depth -= 1;
            }
            if depth == 0 && c.source_space() {
                let token = &value[begin..i];
                begin = i + 1;
                if !token.is_empty() {
                    return Some(token);
                }
            }
        }
        None
    })
}
fn has_class(node: &DOMNode, wanted: &str) -> bool {
    node.FindAttribute("class")
        .is_some_and(|a| split_whitespace(&a.value).any(|s| s == wanted))
}
// cpp: style_resolver/style_resolver.cc:80-94
fn PreviousElementSibling(document: &MatchContext<'_>, index: usize) -> Option<usize> {
    let parent = document.Node(index).Parent()?;
    let mut previous = None;
    for &child in document.Node(parent).Children() {
        if child == index {
            return previous;
        }
        if document.Node(child).Type() == DOMNodeType::kElement {
            previous = Some(child);
        }
    }
    None
}
// cpp: style_resolver/style_resolver.cc:152-174
fn ElementSiblingIndex(
    document: &MatchContext<'_>,
    index: usize,
    from_end: bool,
    same_type: bool,
) -> i32 {
    let node = document.Node(index);
    let Some(parent) = node.Parent() else {
        return 0;
    };
    let children = document.Node(parent).Children();
    let mut position = 0;
    for offset in 0..children.len() {
        let child = children[if from_end {
            children.len() - 1 - offset
        } else {
            offset
        }];
        let candidate = document.Node(child);
        let accepts = candidate.Type() == DOMNodeType::kElement
            && (!same_type
                || (candidate.Namespace() == node.Namespace() && candidate.Name() == node.Name()));
        if accepts {
            position += 1;
        }
        if child == index {
            return if accepts { position } else { 0 };
        }
    }
    0
}
// cpp: style_resolver/style_resolver.cc:331-348
fn any_descendant_matches(
    document: &MatchContext<'_>,
    index: usize,
    selector: &str,
) -> Option<Specificity> {
    for &child in document.Node(index).Children() {
        if document.Node(child).Type() != DOMNodeType::kElement {
            continue;
        }
        if let Some(matched) = MatchSelector(document, child, selector) {
            return Some(matched);
        }
        if let Some(matched) = any_descendant_matches(document, child, selector) {
            return Some(matched);
        }
    }
    None
}
// CSS names overwhelmingly arrive lowercase; retain their borrowed storage.
fn ascii_lowercase(value: &str) -> std::borrow::Cow<'_, str> {
    if value.bytes().any(|byte| byte.is_ascii_uppercase()) {
        std::borrow::Cow::Owned(value.to_ascii_lowercase())
    } else {
        std::borrow::Cow::Borrowed(value)
    }
}
// cpp: style_resolver/style_resolver.cc:282-325
fn MatchAttribute(element: &DOMNode, content: &str) -> bool {
    let content = content.trim_matches(|c: char| c.source_space());
    for operator in ["~=", "|=", "^=", "$=", "*=", "=", ""] {
        let Some(at) = (if operator.is_empty() {
            Some(0)
        } else {
            content.find(operator)
        }) else {
            continue;
        };
        let raw_name = (if operator.is_empty() {
            content
        } else {
            &content[..at]
        })
        .trim_matches(|c: char| c.source_space());
        let name = if element.Namespace() == DOMNamespace::kHTML {
            ascii_lowercase(raw_name)
        } else {
            std::borrow::Cow::Borrowed(raw_name)
        };
        let Some(actual) = element.FindAttribute(&name).map(|a| a.value.as_str()) else {
            return false;
        };
        if operator.is_empty() {
            return true;
        }
        let mut wanted = content[at + operator.len()..].trim_matches(|c: char| c.source_space());
        let mut insensitive = false;
        if wanted.len() >= 2 {
            let bytes = wanted.as_bytes();
            let suffix = bytes[bytes.len() - 1];
            if bytes[bytes.len() - 2].source_space() && matches!(suffix, b'i' | b'I' | b's' | b'S')
            {
                insensitive = matches!(suffix, b'i' | b'I');
                wanted = wanted[..wanted.len() - 2].trim_matches(|c: char| c.source_space());
            }
        }
        if wanted.len() >= 2
            && ((wanted.starts_with('"') && wanted.ends_with('"'))
                || (wanted.starts_with('\'') && wanted.ends_with('\'')))
        {
            wanted = &wanted[1..wanted.len() - 1];
        }
        // Like Blink AttributeValueMatches, compare borrowed attribute data.
        // ASCII folding is only needed when the selector explicitly requests it.
        let (folded_actual, folded_wanted);
        if insensitive {
            folded_actual = actual.to_ascii_lowercase();
            folded_wanted = wanted.to_ascii_lowercase();
        } else {
            folded_actual = String::new();
            folded_wanted = String::new();
        }
        let (actual, wanted) = if insensitive {
            (folded_actual.as_str(), folded_wanted.as_str())
        } else {
            (actual, wanted)
        };
        return match operator {
            "=" => actual == wanted,
            "~=" => split_whitespace(actual).any(|part| part == wanted),
            "^=" => actual.starts_with(wanted),
            "$=" => actual.ends_with(wanted),
            "*=" => actual.contains(wanted),
            "|=" => {
                actual == wanted
                    || actual
                        .strip_prefix(wanted)
                        .is_some_and(|rest| rest.starts_with('-'))
            }
            _ => false,
        };
    }
    false
}

// cpp: style_resolver/style_resolver.cc:350-543
fn MatchCompound(document: &MatchContext<'_>, index: usize, compound: &str) -> Option<Specificity> {
    let element = document.Node(index);
    if element.Type() != DOMNodeType::kElement {
        return None;
    }
    let bytes = compound.as_bytes();
    let mut specificity = Specificity::default();
    let mut i = 0;
    if bytes.first() == Some(&b'*') {
        i += 1;
    } else if !bytes.is_empty() && !matches!(bytes[0], b'.' | b'#' | b'[' | b':') {
        let start = i;
        while i < bytes.len() && !matches!(bytes[i], b'.' | b'#' | b'[' | b':') {
            i += 1;
        }
        let type_name = if element.Namespace() == DOMNamespace::kHTML {
            ascii_lowercase(&compound[start..i])
        } else {
            std::borrow::Cow::Borrowed(&compound[start..i])
        };
        if type_name != element.Name() {
            return None;
        }
        specificity.types += 1;
    }
    while i < bytes.len() {
        let marker = bytes[i];
        i += 1;
        if marker == b'.' || marker == b'#' {
            let start = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'-' | b'_'))
            {
                i += 1;
            }
            if start == i {
                return None;
            }
            let name = &compound[start..i];
            if marker == b'.' {
                specificity.classes += 1;
                if !has_class(element, name) {
                    return None;
                }
            } else {
                specificity.ids += 1;
                if element.FindAttribute("id").map(|a| a.value.as_str()) != Some(name) {
                    return None;
                }
            }
        } else if marker == b'[' {
            let end = compound[i..].find(']')? + i;
            specificity.classes += 1;
            if !MatchAttribute(element, &compound[i..end]) {
                return None;
            }
            i = end + 1;
        } else if marker == b':' {
            if bytes.get(i) == Some(&b':') {
                return None;
            }
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
                i += 1;
            }
            let pseudo = ascii_lowercase(&compound[start..i]);
            let mut argument = "";
            if bytes.get(i) == Some(&b'(') {
                i += 1;
                let argument_start = i;
                let mut depth = 1;
                let mut quote = 0;
                while i < bytes.len() && depth > 0 {
                    let character = bytes[i];
                    if quote != 0 {
                        if character == quote && (i == 0 || bytes[i - 1] != b'\\') {
                            quote = 0;
                        }
                    } else if matches!(character, b'\'' | b'"') {
                        quote = character;
                    } else if character == b'(' {
                        depth += 1;
                    } else if character == b')' {
                        depth -= 1;
                    }
                    i += 1;
                }
                if depth > 0 {
                    return None;
                }
                argument = compound[argument_start..i - 1].trim_matches(|c: char| c.source_space());
            }
            if matches!(pseudo.as_ref(), "not" | "is" | "where") {
                if argument.is_empty() {
                    return None;
                }
                let matched = MatchSelector(document, index, argument);
                if (pseudo == "not" && matched.is_some()) || (pseudo != "not" && matched.is_none())
                {
                    return None;
                }
                if pseudo != "where" {
                    if let Some(matched) = matched {
                        specificity = specificity + matched;
                    } else {
                        // cpp: style_resolver/style_resolver.cc:434-456
                        let mut argument_specificity = Specificity::default();
                        for selector in SplitSelectorList(argument) {
                            let mut candidate = Specificity::default();
                            if !selector.as_bytes().first().is_some_and(|byte| {
                                matches!(byte, b'*' | b'#' | b'.' | b'[' | b':')
                            }) {
                                candidate.types += 1;
                            }
                            for byte in selector.bytes() {
                                match byte {
                                    b'#' => candidate.ids += 1,
                                    b'.' | b'[' | b':' => candidate.classes += 1,
                                    _ => {}
                                }
                            }
                            argument_specificity = argument_specificity.max(candidate);
                        }
                        specificity = specificity + argument_specificity;
                    }
                }
                continue;
            }
            if pseudo == "has" {
                if argument.is_empty() {
                    return None;
                }
                let mut matched = None;
                for selector in SplitSelectorList(argument) {
                    let relative = selector.trim_matches(|c: char| c.source_space());
                    if let Some(child_selector) = relative.strip_prefix('>') {
                        let child_selector =
                            child_selector.trim_matches(|c: char| c.source_space());
                        for &child in element.Children() {
                            if document.Node(child).Type() != DOMNodeType::kElement {
                                continue;
                            }
                            if let Some(candidate) = MatchSelector(document, child, child_selector)
                            {
                                matched = Some(candidate);
                                break;
                            }
                        }
                    } else {
                        matched = any_descendant_matches(document, index, relative);
                    }
                    if matched.is_some() {
                        break;
                    }
                }
                specificity = specificity + matched?;
                continue;
            }
            specificity.classes += 1;
            let matches = match pseudo.as_ref() {
                "root" if argument.is_empty() => element
                    .Parent()
                    .is_some_and(|parent| document.Node(parent).Type() == DOMNodeType::kDocument),
                "first-child" if argument.is_empty() => {
                    PreviousElementSibling(document, index).is_none()
                }
                "last-child" if argument.is_empty() => element.Parent().is_some_and(|parent| {
                    document
                        .Node(parent)
                        .Children()
                        .iter()
                        .rev()
                        .copied()
                        .find(|&child| document.Node(child).Type() == DOMNodeType::kElement)
                        == Some(index)
                }),
                "first-of-type" if argument.is_empty() => {
                    ElementSiblingIndex(document, index, false, true) == 1
                }
                "last-of-type" if argument.is_empty() => {
                    ElementSiblingIndex(document, index, true, true) == 1
                }
                "nth-child" | "nth-last-child" | "nth-of-type" | "nth-last-of-type"
                    if !argument.is_empty() =>
                {
                    MatchesAnPlusB(
                        ElementSiblingIndex(
                            document,
                            index,
                            pseudo.contains("last"),
                            pseudo.contains("of-type"),
                        ),
                        argument,
                    )
                }
                "placeholder-shown" if argument.is_empty() => {
                    (element.IsHTMLElement("input") || element.IsHTMLElement("textarea"))
                        && element.FindAttribute("placeholder").is_some()
                        && if document.use_control_value {
                            document.ControlValue(index).is_empty()
                        } else {
                            element
                                .FindAttribute("value")
                                .is_none_or(|a| a.value.is_empty())
                        }
                }
                _ => false,
            };
            if !matches {
                return None;
            }
        } else {
            return None;
        }
    }
    Some(specificity)
}

// cpp: style_resolver/style_resolver.cc:545-581
fn MatchParsed(
    document: &MatchContext<'_>,
    selector: &ParsedSelector,
    part: usize,
    index: usize,
) -> Option<Specificity> {
    let local = MatchCompound(document, index, &selector.compounds[part])?;
    if part == 0 {
        return Some(local);
    }
    let combinator = selector.combinators[part - 1];
    if combinator == b'>' {
        return MatchParsed(document, selector, part - 1, document.Node(index).Parent()?)
            .map(|left| left + local);
    }
    if combinator == b'+' {
        return MatchParsed(
            document,
            selector,
            part - 1,
            PreviousElementSibling(document, index)?,
        )
        .map(|left| left + local);
    }
    if combinator == b'~' {
        let mut sibling = PreviousElementSibling(document, index);
        while let Some(candidate) = sibling {
            if let Some(left) = MatchParsed(document, selector, part - 1, candidate) {
                return Some(left + local);
            }
            sibling = PreviousElementSibling(document, candidate);
        }
        return None;
    }
    let mut ancestor = document.Node(index).Parent();
    while let Some(candidate) = ancestor {
        if let Some(left) = MatchParsed(document, selector, part - 1, candidate) {
            return Some(left + local);
        }
        ancestor = document.Node(candidate).Parent();
    }
    None
}

// cpp: style_resolver/style_resolver.cc:633-669
fn MatchSelector(document: &MatchContext<'_>, index: usize, text: &str) -> Option<Specificity> {
    MatchSelectorForTarget(document, index, text, PseudoTarget::Element)
}
fn MatchSelectorForTarget(
    document: &MatchContext<'_>,
    index: usize,
    text: &str,
    target: PseudoTarget,
) -> Option<Specificity> {
    // Blink SelectorQueryCache (core/css/selector_query.cc) keeps up to 256
    // parsed queries. Queries traverse the current DOM on every call; only
    // parsing is reused, including when the query is checked on many nodes.
    let cache = if document.use_control_value {
        &document.StyleState().rules.selectors
    } else {
        &document.StyleState().rules.query_selectors
    };
    let cached = cache.borrow().get(text).cloned();
    let selectors = cached.unwrap_or_else(|| {
        let compiled = crate::style_resolver::style_rule_index::CompileSelectors(text);
        let mut entries = cache.borrow_mut();
        if !document.use_control_value && entries.len() >= 256 {
            if let Some(key) = entries.keys().next().cloned() {
                entries.remove(&key);
            }
        }
        entries.insert(text.to_owned(), compiled.clone());
        compiled
    });
    MatchCompiled(document, index, &selectors, target)
}
fn MatchCompiled(
    document: &MatchContext<'_>,
    index: usize,
    selectors: &[cssom::compiled_rules::CompiledSelector],
    target: PseudoTarget,
) -> Option<Specificity> {
    selectors
        .iter()
        .filter_map(|compiled| {
            let selector = &compiled.selector;
            if !selector.valid || compiled.target != target {
                return None;
            }
            let mut specificity =
                MatchParsed(document, selector, selector.compounds.len() - 1, index)?;
            if target != PseudoTarget::Element {
                specificity.types += 1;
            }
            Some(specificity)
        })
        .max()
}
pub(crate) fn MatchCompiledSelectorTarget(
    document: &Document,
    index: usize,
    selectors: &[cssom::compiled_rules::CompiledSelector],
    target: PseudoTarget,
) -> Option<Specificity> {
    MatchCompiled(
        &MatchContext {
            document,
            use_control_value: true,
        },
        index,
        selectors,
        target,
    )
}
// cpp: style_resolver/style_resolver.cc:8078-8081
pub fn MatchesSelector(document: &Document, index: usize, text: &str) -> bool {
    document.Node(index).Type() == DOMNodeType::kElement
        && MatchSelector(
            &MatchContext {
                document,
                use_control_value: false,
            },
            index,
            text,
        )
        .is_some()
}

#[cfg(test)]
mod borrowed_match_tests {
    use super::*;

    // Retained source algorithm: compare every token, including unmatched
    // parentheses and non-ASCII whitespace, before using the lazy matcher.
    fn source_tokens(value: &str) -> Vec<&str> {
        let mut result = Vec::new();
        let mut begin = 0;
        let mut depth = 0;
        for i in 0..=value.len() {
            let c = value.as_bytes().get(i).copied().unwrap_or(b' ');
            if c == b'(' {
                depth += 1;
            } else if c == b')' {
                depth -= 1;
            }
            if depth == 0 && c.source_space() {
                if i > begin {
                    result.push(&value[begin..i]);
                }
                begin = i + 1;
            }
        }
        result
    }

    #[test]
    fn borrowed_selector_tokens_match_source_exhaustively() {
        let alphabet = ["a", " ", "\t", "\n", "\r", "\x0c", "(", ")", "é", "\u{a0}"];
        for n in 0usize..100_000 {
            let mut value = String::new();
            let mut code = n;
            for _ in 0..5 {
                value.push_str(alphabet[code % alphabet.len()]);
                code /= alphabet.len();
            }
            assert_eq!(
                split_whitespace(&value).collect::<Vec<_>>(),
                source_tokens(&value),
                "{value:?}"
            );
        }
    }

    #[test]
    fn borrowed_attribute_matches_operators_case_and_live_changes() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<div id=x class='alpha beta' data-word='en-US two café'></div>",
        );
        let index = (0..owner.GetDocument().NodeCount())
            .find(|&i| {
                owner
                    .GetDocument()
                    .Node(i)
                    .FindAttribute("id")
                    .is_some_and(|a| a.value == "x")
            })
            .unwrap();
        for (selector, expected) in [
            (".alpha.beta", true),
            ("DIV", true),
            ("[DATA-WORD]", true),
            (":IS(.alpha)", true),
            (".alph", false),
            ("[data-word]", true),
            ("[data-word='en-US two café']", true),
            ("[data-word='EN-us TWO CAFé' i]", true),
            ("[data-word='EN-us TWO CAFé' s]", false),
            ("[data-word~='two']", true),
            ("[data-word~='tw']", false),
            ("[data-word^='en-US']", true),
            ("[data-word$='café']", true),
            ("[data-word*='US two']", true),
            ("[data-word|='en']", true),
            ("[data-word|='e']", false),
        ] {
            assert_eq!(
                MatchesSelector(owner.GetDocument(), index, selector),
                expected,
                "{selector}"
            );
        }
        owner.GetDocumentMut().SetAttribute(
            index,
            crate::persistent_document::DOMAttribute {
                local_name: "class".into(),
                value: "gamma delta".into(),
                ..Default::default()
            },
        );
        assert!(!MatchesSelector(owner.GetDocument(), index, ".alpha"));
        assert!(MatchesSelector(owner.GetDocument(), index, ".delta"));
    }

    #[test]
    fn parsed_query_cache_matches_fresh_queries_after_live_mutations_and_is_bounded() {
        let mut owner = crate::test_html::html_parser::ParseHTML("<main><div id=x class='alpha'><span class=leaf></span></div><input id=q placeholder=Search value=''></main>");
        let selectors = [
            "main .leaf",
            "main > .alpha",
            ".alpha:has(.leaf)",
            "input:placeholder-shown",
            "input:not(:placeholder-shown)",
            "[id=x]",
            ".changed",
            "main > :first-child",
            ":is(.leaf,.changed)",
            ":not(.alpha)",
        ];
        for round in 0..3 {
            let document = owner.GetDocument();
            for selector in selectors {
                for i in 0..document.NodeCount() {
                    let context = MatchContext {
                        document,
                        use_control_value: false,
                    };
                    let fresh = document.Node(i).Type() == DOMNodeType::kElement
                        && MatchCompiled(
                            &context,
                            i,
                            &crate::style_resolver::style_rule_index::CompileSelectors(selector),
                            PseudoTarget::Element,
                        )
                        .is_some();
                    assert_eq!(
                        MatchesSelector(document, i, selector),
                        fresh,
                        "{round} {selector} {i}"
                    );
                }
            }
            let index = (0..document.NodeCount())
                .find(|&i| {
                    document
                        .Node(i)
                        .FindAttribute("id")
                        .is_some_and(|a| a.value == "x")
                })
                .unwrap();
            owner.GetDocumentMut().SetAttribute(
                index,
                crate::persistent_document::DOMAttribute {
                    local_name: "class".into(),
                    value: if round == 0 { "changed" } else { "alpha" }.into(),
                    ..Default::default()
                },
            );
        }
        let document = owner.GetDocument();
        let index = (0..document.NodeCount())
            .find(|&i| document.Node(i).Type() == DOMNodeType::kElement)
            .unwrap();
        for i in 0..300 {
            MatchesSelector(document, index, &format!(".unmatched-{i}"));
        }
        assert_eq!(
            document.StyleState().rules.query_selectors.borrow().len(),
            256
        );
        assert!(
            document.StyleState().rules.selectors.borrow().is_empty(),
            "public queries must not populate the cascade cache"
        );
    }

    #[test]
    fn parsed_query_cache_keeps_public_and_live_control_matching_distinct() {
        let mut owner =
            crate::test_html::html_parser::ParseHTML("<input placeholder=Search value=''>");
        let index = (0..owner.GetDocument().NodeCount())
            .find(|&i| owner.GetDocument().Node(i).IsHTMLElement("input"))
            .unwrap();
        let selector = "input:placeholder-shown";
        assert!(MatchesSelector(owner.GetDocument(), index, selector));
        owner
            .GetDocumentMut()
            .SetControlValue(index, "typed".into());
        let document = owner.GetDocument();
        // The preexisting public matcher reads the value attribute; style
        // matching reads live control state. Reusing syntax must not reuse
        // either result or the other call site's context.
        assert!(MatchesSelector(document, index, selector));
        assert!(MatchSelector(
            &MatchContext {
                document,
                use_control_value: true
            },
            index,
            selector
        )
        .is_none());
        assert!(MatchesSelector(document, index, selector));
        let first = document
            .StyleState()
            .rules
            .query_selectors
            .borrow()
            .get(selector)
            .unwrap()
            .clone();
        MatchesSelector(document, index, selector);
        let second = document
            .StyleState()
            .rules
            .query_selectors
            .borrow()
            .get(selector)
            .unwrap()
            .clone();
        assert!(
            std::sync::Arc::ptr_eq(&first, &second),
            "syntax should be parsed once"
        );
    }

    #[test]
    #[ignore = "paired full selector traversal microbenchmark; no layout/raster"]
    fn profile_parsed_query_traversal() {
        let markup = format!("<main>{}</main>", "<section class='row'><span class='title'>text</span><a class='link'>result</a></section>".repeat(300));
        let owner = crate::test_html::html_parser::ParseHTML(&markup);
        let document = owner.GetDocument();
        let selector = "main > section.row > a.link:not(.missing)";
        let context = MatchContext {
            document,
            use_control_value: false,
        };
        let mut baseline = Vec::new();
        let mut cached = Vec::new();
        // Alternate paired cold/warm document traversals; every candidate is
        // matched in each run. A traversal is one query, not one whole frame.
        // The baseline reparses the outer selector per node; nested :not
        // queries retain the same cache in both paths (a conservative baseline).
        for _ in 0..20 {
            for use_cache in [false, true] {
                let begin = std::time::Instant::now();
                let mut count = 0;
                for i in 0..document.NodeCount() {
                    if document.Node(i).Type() != DOMNodeType::kElement {
                        continue;
                    }
                    let matched = if use_cache {
                        MatchesSelector(document, i, selector)
                    } else {
                        MatchCompiled(
                            &context,
                            i,
                            &crate::style_resolver::style_rule_index::CompileSelectors(selector),
                            PseudoTarget::Element,
                        )
                        .is_some()
                    };
                    count += usize::from(matched);
                }
                assert_eq!(count, 300);
                let ms = begin.elapsed().as_secs_f64() * 1000.;
                if use_cache {
                    cached.push(ms);
                } else {
                    baseline.push(ms);
                }
            }
        }
        eprintln!(
            "query_traversal nodes={} baseline_ms={baseline:?} cached_ms={cached:?}",
            document.NodeCount()
        );
        eprintln!(
            "query_traversal baseline_max_ms={:.3} cached_max_ms={:.3}",
            baseline.iter().copied().fold(0., f64::max),
            cached.iter().copied().fold(0., f64::max)
        );
    }

    #[test]
    #[ignore = "paired selector microbenchmark; run without concurrent performance work"]
    fn profile_borrowed_class_matching() {
        let classes = std::hint::black_box("a b c d e f g h i j k l m n o p q r s t u v w x y z");
        let queries = ["a", "n", "z", "missing"];
        for wanted in queries {
            let start = std::time::Instant::now();
            for _ in 0..100_000 {
                std::hint::black_box(
                    source_tokens(classes)
                        .into_iter()
                        .any(|s| s == std::hint::black_box(wanted)),
                );
            }
            let source = start.elapsed();
            let start = std::time::Instant::now();
            for _ in 0..100_000 {
                std::hint::black_box(
                    split_whitespace(classes).any(|s| s == std::hint::black_box(wanted)),
                );
            }
            eprintln!(
                "class={wanted} source_ms={:.3} borrowed_ms={:.3}",
                source.as_secs_f64() * 1000.,
                start.elapsed().as_secs_f64() * 1000.
            );
        }
    }
}
