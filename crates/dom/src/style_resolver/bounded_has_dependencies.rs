//! Restrict only proven descendant :has dependencies. Every other grammar
//! retains the original root fallback. Metadata observes syntax, never matches.
#![allow(non_snake_case)]
use crate::style_resolver::selector::ParsedSelector;

pub(super) fn BoundedHasHost(selector: &ParsedSelector) -> Result<Option<String>, ()> {
    let occurrences = selector
        .compounds
        .iter()
        .map(|part| part.to_ascii_lowercase().matches(":has(").count())
        .sum::<usize>();
    if occurrences == 0 {
        return Ok(None);
    }
    if occurrences != 1
        || selector
            .combinators
            .iter()
            .any(|c| !matches!(c, b' ' | b'>'))
    {
        return Err(());
    }
    if selector
        .compounds
        .iter()
        .skip(1)
        .any(|part| super::inline_style_dependencies::MayReadStyleAttribute(part))
    {
        return Err(());
    }
    let first = selector.compounds.first().ok_or(())?;
    let lower = first.to_ascii_lowercase();
    let start = lower.find(":has(").ok_or(())?;
    let host = &first[..start];
    if !SimpleHost(host) {
        return Err(());
    }
    let bytes = first.as_bytes();
    let mut i = start + 5;
    let argument_start = i;
    let mut depth = 1usize;
    let mut quote = 0u8;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' || !b.is_ascii() {
            return Err(());
        }
        if quote != 0 {
            if b == quote {
                quote = 0;
            }
        } else if matches!(b, b'\'' | b'"') {
            quote = b;
        } else if b == b'(' {
            depth += 1;
        } else if b == b')' {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
        i += 1;
    }
    if depth != 0 || quote != 0 || i == argument_start {
        return Err(());
    }
    // Sibling relative arguments, nested :has, unknown functions, namespaces,
    // escapes and attribute-style dependencies all keep the broad fallback.
    let argument = &first[argument_start..i];
    if argument.contains('+')
        || argument.contains('~')
        || super::inline_style_dependencies::MayReadStyleAttribute(argument)
    {
        return Err(());
    }
    // A suffix is unnecessary for the host necessary-condition gate, but only
    // admit simple class/id refinements. Structural/unknown host conditions
    // could change on a sibling that is outside the mutation's ancestor chain.
    let suffix = &first[i + 1..];
    if !suffix.is_empty() && (!matches!(suffix.as_bytes()[0], b'.' | b'#') || !SimpleHost(suffix)) {
        return Err(());
    }
    Ok(Some(host.to_owned()))
}

// Child insertion cannot change the container's own tag/class/id or sibling
// position. Its :empty state can change, including inside functional selectors.
// Record only a simple necessary-condition gate; all other child-state syntax
// keeps the original parent scope. :has is handled by the existing host proof.
pub(super) fn ChildStateHost(compound: &str) -> Result<Option<String>, ()> {
    if super::inline_style_dependencies::MayReadStyleAttribute(compound) {
        return Err(());
    }
    let lower = compound.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            // MayReadStyleAttribute already proved brackets/quotes well formed.
            let mut quote = 0;
            i += 1;
            while i < bytes.len() {
                let b = bytes[i];
                if quote != 0 {
                    if b == quote {
                        quote = 0;
                    }
                } else if matches!(b, b'\'' | b'"') {
                    quote = b;
                } else if b == b']' {
                    break;
                }
                i += 1;
            }
        } else if bytes[i] == b':' {
            i += 1;
            if bytes.get(i) == Some(&b':') {
                i += 1;
            }
            let start = i;
            while bytes
                .get(i)
                .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'-')
            {
                i += 1;
            }
            let name = &lower[start..i];
            // Keep unknown bare pseudo-classes conservative too. Recognized
            // non-structural state depends on attributes/interaction; head is
            // not a control and control ancestors are excluded by the caller.
            if ![
                "empty",
                "root",
                "first-child",
                "last-child",
                "only-child",
                "first-of-type",
                "last-of-type",
                "only-of-type",
                "nth-child",
                "nth-last-child",
                "nth-of-type",
                "nth-last-of-type",
                "has",
                "is",
                "not",
                "where",
                "placeholder-shown",
                "hover",
                "active",
                "focus",
                "focus-visible",
                "link",
                "visited",
                "disabled",
                "enabled",
                "checked",
                "before",
                "after",
                "first-letter",
                "placeholder",
            ]
            .contains(&name)
            {
                return Err(());
            }
            continue;
        }
        i += 1;
    }
    let Some(start) = lower.find(":empty") else {
        return Ok(None);
    };
    if lower.matches(":empty").count() != 1
        || &lower[start..] != ":empty"
        || !SimpleHost(&compound[..start])
    {
        return Err(());
    }
    Ok(Some(compound[..start].to_owned()))
}

// Child insertion does not change the container's tag, id or classes. Even
// when its remaining pseudo/attribute syntax cannot be classified, this fixed
// prefix is a necessary condition for that compound to observe the container.
// :has retains its separate relational invalidation; this proof never narrows
// that path. Escapes/namespaces and universal/functional prefixes stay broad.
pub(super) fn ChildStateFallbackHost(compound: &str) -> Option<&str> {
    if compound.contains('\\') {
        return None;
    }
    let end = compound.find([':', '[']).unwrap_or(compound.len());
    let host = &compound[..end];
    SimpleHost(host).then_some(host)
}

// Return a necessary condition for an element whose reverse sibling position
// can affect selector matching.  The prefix is deliberately weaker than the
// complete compound: checking it may over-invalidate, but can never omit an
// element that the compound could match.  Unknown/universal prefixes retain
// the conservative parent fallback.
pub(super) fn NthLastHost(compound: &str) -> Result<Option<String>, ()> {
    if compound.contains('\\') || !compound.is_ascii() {
        return Err(());
    }
    let lower = compound.to_ascii_lowercase();
    if !lower.contains(":nth-last-child(") && !lower.contains(":nth-last-of-type(") {
        return Ok(None);
    }
    let end = compound.find([':', '[']).unwrap_or(compound.len());
    let host = &compound[..end];
    if host.is_empty() || host == "*" || !SimpleHost(host) {
        return Err(());
    }
    Ok(Some(host.to_owned()))
}

fn SimpleHost(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if matches!(bytes[i], b'.' | b'#') {
            i += 1;
        } else if i != 0 {
            return false;
        }
        let start = i;
        if !bytes
            .get(i)
            .is_some_and(|b| b.is_ascii_alphabetic() || matches!(b, b'_' | b'-'))
        {
            return false;
        }
        while bytes
            .get(i)
            .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
        {
            i += 1;
        }
        if i == start {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conservative_child_state_keeps_only_proven_fixed_hosts() {
        for (compound, host) in [
            (".card:not(:empty)", ".card"),
            ("div.card[style]", "div.card"),
            ("head:future-state", "head"),
        ] {
            assert_eq!(ChildStateFallbackHost(compound), Some(host));
        }
        for compound in [
            ":not(:empty)",
            "*:future-state",
            "svg|g:future-state",
            r".c\61rd:empty",
        ] {
            assert_eq!(ChildStateFallbackHost(compound), None);
        }
        let sheet =
            cssom::ParseCSS(".card:not(:empty), div[style] {color:red} .a + .b {color:blue}");
        let rules = super::super::BuildCompiledRuleSet([&sheet]);
        assert!(!rules.children_container_dependency_fallback);
        assert_eq!(rules.children_container_fallback_hosts, [".card", "div"]);
        let sheet = cssom::ParseCSS(":not(:empty) {color:red}");
        assert!(
            super::super::BuildCompiledRuleSet([&sheet]).children_container_dependency_fallback
        );
    }
    #[test]
    fn reverse_position_host_is_a_safe_necessary_condition() {
        assert_eq!(
            NthLastHost("li.card:nth-last-child(2)").unwrap(),
            Some("li.card".into())
        );
        assert_eq!(
            NthLastHost(".card:not(:nth-last-of-type(2))").unwrap(),
            Some(".card".into())
        );
        assert!(NthLastHost(":nth-last-child(2)").is_err());
        assert_eq!(NthLastHost(".card:last-child").unwrap(), None);
    }
    fn proof(text: &str) -> Result<Option<String>, ()> {
        let selectors = super::super::CompileSelectors(text);
        assert_eq!(selectors.len(), 1);
        BoundedHasHost(&selectors[0].selector)
    }
    #[test]
    fn actual_google_has_shapes_are_bounded_by_two_literal_hosts() {
        for text in [
            ".doljm:has(.Egmwab :first-child)",
            ".vH6rvf:has(.jOCUFf) .Fyh8ub",
            ".vH6rvf:has(.jOCUFf) .rNHry[aria-checked]:after",
        ] {
            assert_eq!(
                proof(text).unwrap(),
                Some(
                    if text.starts_with(".doljm") {
                        ".doljm"
                    } else {
                        ".vH6rvf"
                    }
                    .into()
                )
            );
        }
        let sheet = cssom::ParseCSS(include_str!("tests/google_typed_selector_dependencies.css"));
        let rules = super::super::BuildCompiledRuleSet([&sheet]);
        assert!(!rules.has_dependency_fallback);
        let mut hosts = rules.bounded_has_hosts.clone();
        hosts.sort();
        assert_eq!(hosts, [".doljm", ".vH6rvf"]);
    }
    #[test]
    fn nonlocal_or_unknown_has_shapes_keep_root_fallback() {
        for text in [
            ".outer .host:has(.x)",
            ".host:has(.x) + .other",
            ".host:has(+ .x)",
            ".host:has(.x ~ .y)",
            ".host:has(:is(.x:has(.y)))",
            ".host:nth-child(2):has(.x)",
            ".host:has(.x):first-child",
            ".host:has(:future(.x))",
            ".host:has([ns|class])",
            r".host:has([cl\61ss])",
            ".host:has([style])",
        ] {
            assert!(proof(text).is_err(), "{text}");
        }
    }
}
