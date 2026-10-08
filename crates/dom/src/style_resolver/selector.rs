#![allow(non_snake_case)]

use crate::{Child, Element, ParsedDocument as Document};

pub(crate) trait SourceSpace {
    fn source_space(self) -> bool;
}
impl SourceSpace for u8 {
    fn source_space(self) -> bool {
        matches!(self, b' ' | b'\t'..=b'\r')
    }
}
impl SourceSpace for char {
    fn source_space(self) -> bool {
        matches!(self, ' ' | '\t'..='\r')
    }
}

// cpp: style_resolver/style_resolver.cc:176-182
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Specificity {
    pub ids: u32,
    pub classes: u32,
    pub types: u32,
}

impl std::ops::Add for Specificity {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            ids: self.ids + other.ids,
            classes: self.classes + other.classes,
            types: self.types + other.types,
        }
    }
}

pub(crate) use cssom::compiled_rules::ParsedSelector;
pub use cssom::compiled_rules::PseudoTarget;

// cpp: style_resolver/style_resolver.cc:581-619
pub(crate) fn ExtractTerminalPseudo(selector: &mut ParsedSelector) -> PseudoTarget {
    let Some(last) = selector.compounds.last_mut() else {
        return PseudoTarget::Invalid;
    };
    let targets = [
        ("::before", PseudoTarget::Before),
        ("::after", PseudoTarget::After),
        ("::first-letter", PseudoTarget::FirstLetter),
        ("::placeholder", PseudoTarget::Placeholder),
        (":before", PseudoTarget::Before),
        (":after", PseudoTarget::After),
        (":first-letter", PseudoTarget::FirstLetter),
    ];
    let mut target = PseudoTarget::Element;
    for (spelling, candidate) in targets {
        if last.ends_with(spelling) {
            last.truncate(last.len() - spelling.len());
            if last.is_empty() {
                *last = "*".to_owned();
            }
            target = candidate;
            break;
        }
    }
    if selector.compounds.iter().any(|part| part.contains("::")) {
        PseudoTarget::Invalid
    } else {
        target
    }
}

// cpp: style_resolver/style_resolver.cc:200-225
pub(crate) fn SplitSelectorList(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut selectors = Vec::new();
    let mut start = 0;
    let mut square = 0;
    let mut round = 0;
    let mut quote = 0;
    for i in 0..=bytes.len() {
        let c = if i < bytes.len() { bytes[i] } else { b',' };
        if quote != 0 {
            if c == quote && (i == 0 || bytes[i - 1] != b'\\') {
                quote = 0;
            }
            continue;
        }
        match c {
            b'\'' | b'"' => quote = c,
            b'[' => square += 1,
            b']' => square -= 1,
            b'(' => round += 1,
            b')' => round -= 1,
            b',' if square == 0 && round == 0 => {
                let selector = text[start..i].trim_matches(|c: char| c.source_space());
                if !selector.is_empty() {
                    selectors.push(selector.to_owned());
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    selectors
}

// cpp: style_resolver/style_resolver.cc:227-280
pub(crate) fn ParseSelector(text: &str) -> ParsedSelector {
    let bytes = text.as_bytes();
    let mut result = ParsedSelector {
        compounds: Vec::new(),
        combinators: Vec::new(),
        valid: true,
    };
    let mut i = 0;
    let mut needs_descendant = false;
    while i < bytes.len() {
        let mut saw_space = false;
        while i < bytes.len() && bytes[i].source_space() {
            saw_space = true;
            i += 1;
        }
        if i == bytes.len() {
            break;
        }
        if !result.compounds.is_empty()
            && result.combinators.len() + 1 == result.compounds.len()
            && !matches!(bytes[i], b'>' | b'+' | b'~')
            && (saw_space || needs_descendant)
        {
            result.combinators.push(b' ');
        }
        if matches!(bytes[i], b'>' | b'+' | b'~') {
            if result.compounds.is_empty() || result.combinators.len() == result.compounds.len() {
                result.valid = false;
                return result;
            }
            result.combinators.push(bytes[i]);
            i += 1;
            needs_descendant = false;
            continue;
        }
        let begin = i;
        let mut square = 0;
        let mut round = 0;
        let mut quote = 0;
        while i < bytes.len() {
            let c = bytes[i];
            if quote != 0 {
                if c == quote && (i == 0 || bytes[i - 1] != b'\\') {
                    quote = 0;
                }
            } else {
                match c {
                    b'\'' | b'"' => quote = c,
                    b'[' => square += 1,
                    b']' => square -= 1,
                    b'(' => round += 1,
                    b')' => round -= 1,
                    _ => {}
                }
                if square == 0
                    && round == 0
                    && (c.source_space() || matches!(c, b'>' | b'+' | b'~'))
                {
                    break;
                }
            }
            i += 1;
        }
        result.compounds.push(text[begin..i].to_owned());
        needs_descendant = true;
    }
    if result.compounds.is_empty()
        || result.combinators.len() + 1 != result.compounds.len()
        || result
            .compounds
            .iter()
            .any(|part| !CompoundSyntaxValid(part))
    {
        result.valid = false;
    }
    result
}

// A selector list is invalid as a whole when one of its ordinary selectors is
// syntactically invalid. Keep this validation independent from matching: a
// malformed selector must not start matching merely because another selector
// in the same comma-separated rule is valid.
fn CompoundSyntaxValid(compound: &str) -> bool {
    let bytes = compound.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut i = 0;
    if bytes[0] == b'*' {
        i = 1;
    } else if !matches!(bytes[0], b'.' | b'#' | b'[' | b':') {
        while i < bytes.len() && !matches!(bytes[i], b'.' | b'#' | b'[' | b':') {
            i += 1;
        }
    }
    while i < bytes.len() {
        match bytes[i] {
            b'.' | b'#' => {
                i += 1;
                let start = i;
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'-' | b'_'))
                {
                    i += 1;
                }
                if i == start {
                    return false;
                }
            }
            b'[' => {
                i += 1;
                let start = i;
                let mut quote = 0;
                while i < bytes.len() {
                    let c = bytes[i];
                    if quote != 0 {
                        if c == quote && (i == 0 || bytes[i - 1] != b'\\') {
                            quote = 0;
                        }
                    } else if matches!(c, b'\'' | b'"') {
                        quote = c;
                    } else if c == b']' {
                        break;
                    }
                    i += 1;
                }
                if i == bytes.len() || i == start {
                    return false;
                }
                i += 1;
            }
            b':' => {
                i += 1;
                if bytes.get(i) == Some(&b':') {
                    i += 1;
                }
                let start = i;
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'-' | b'_'))
                {
                    i += 1;
                }
                if i == start {
                    return false;
                }
                if bytes.get(i) == Some(&b'(') {
                    i += 1;
                    let mut depth = 1;
                    let mut quote = 0;
                    while i < bytes.len() && depth > 0 {
                        let c = bytes[i];
                        if quote != 0 {
                            if c == quote && bytes[i - 1] != b'\\' {
                                quote = 0;
                            }
                        } else if matches!(c, b'\'' | b'"') {
                            quote = c;
                        } else if c == b'(' {
                            depth += 1;
                        } else if c == b')' {
                            depth -= 1;
                        }
                        i += 1;
                    }
                    if depth != 0 {
                        return false;
                    }
                }
            }
            _ => return false,
        }
    }
    true
}

pub(crate) fn ParseStrictSelectorList(text: &str) -> Option<Vec<ParsedSelector>> {
    let selectors: Vec<_> = SplitSelectorList(text)
        .into_iter()
        .map(|item| ParseSelector(&item))
        .collect();
    (!selectors.is_empty() && selectors.iter().all(|selector| selector.valid)).then_some(selectors)
}

fn attribute<'a>(element: &'a Element, name: &str) -> Option<&'a str> {
    element
        .attributes
        .iter()
        .find(|(candidate, _)| candidate == name)
        .map(|(_, value)| value.as_str())
}

// cpp: style_resolver/style_resolver.cc:282-325
fn MatchAttribute(element: &Element, content: &str) -> bool {
    let content = content.trim_matches(|c: char| c.source_space());
    for operator in ["~=", "|=", "^=", "$=", "*=", "=", ""] {
        let Some(at) = (if operator.is_empty() {
            Some(0)
        } else {
            content.find(operator)
        }) else {
            continue;
        };
        let name = (if operator.is_empty() {
            content
        } else {
            &content[..at]
        })
        .trim_matches(|c: char| c.source_space())
        .to_ascii_lowercase();
        let Some(actual) = attribute(element, &name) else {
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
        if (wanted.starts_with('"') && wanted.ends_with('"'))
            || (wanted.starts_with('\'') && wanted.ends_with('\''))
        {
            wanted = &wanted[1..wanted.len() - 1];
        }
        let (actual, wanted) = if insensitive {
            (actual.to_ascii_lowercase(), wanted.to_ascii_lowercase())
        } else {
            (actual.to_owned(), wanted.to_owned())
        };
        return match operator {
            "=" => actual == wanted,
            "~=" => actual
                .split(|c: char| c.source_space())
                .filter(|s| !s.is_empty())
                .any(|part| part == wanted),
            "^=" => actual.starts_with(&wanted),
            "$=" => actual.ends_with(&wanted),
            "*=" => actual.contains(&wanted),
            "|=" => actual == wanted || actual.starts_with(&format!("{wanted}-")),
            _ => false,
        };
    }
    false
}

fn PreviousElementSibling(document: &Document, index: usize) -> Option<usize> {
    let parent = document.elements[index].parent?;
    let mut previous = None;
    for child in &document.elements[parent].children {
        if let Child::Element(candidate) = *child {
            if candidate == index {
                return previous;
            }
            previous = Some(candidate);
        }
    }
    None
}

fn ElementSiblingIndex(document: &Document, index: usize, from_end: bool, same_type: bool) -> i32 {
    let Some(parent) = document.elements[index].parent else {
        return 0;
    };
    let children = &document.elements[parent].children;
    let mut position = 0;
    let iter: Box<dyn Iterator<Item = &Child> + '_> = if from_end {
        Box::new(children.iter().rev())
    } else {
        Box::new(children.iter())
    };
    for child in iter {
        let Child::Element(candidate) = *child else {
            continue;
        };
        if !same_type || document.elements[candidate].tag == document.elements[index].tag {
            position += 1;
        }
        if candidate == index {
            return position;
        }
    }
    0
}

pub(crate) fn MatchesAnPlusB(index: i32, expression: &str) -> bool {
    if index < 1 {
        return false;
    }
    let value: String = expression
        .bytes()
        .filter(|byte| !byte.source_space())
        .map(|byte| (byte as char).to_ascii_lowercase())
        .collect();
    let formula = match value.as_str() {
        "even" => Some((2, 0)),
        "odd" => Some((2, 1)),
        _ => {
            if let Some(n) = value.find('n') {
                if value[n + 1..].contains('n') {
                    None
                } else {
                    let a = match &value[..n] {
                        "" | "+" => Some(1),
                        "-" => Some(-1),
                        other => other.parse().ok(),
                    };
                    let b = if value[n + 1..].is_empty() {
                        Some(0)
                    } else {
                        value[n + 1..].parse().ok()
                    };
                    a.zip(b)
                }
            } else {
                value.parse().ok().map(|b| (0, b))
            }
        }
    };
    let Some((a, b)) = formula else {
        return false;
    };
    if a == 0 {
        return index == b;
    }
    let delta = index - b;
    delta % a == 0 && delta / a >= 0
}

// cpp: style_resolver/style_resolver.cc:350-543
fn MatchCompound(document: &Document, index: usize, compound: &str) -> Option<Specificity> {
    let element = &document.elements[index];
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
        if compound[start..i].to_ascii_lowercase() != element.tag {
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
                if !element.classes.iter().any(|class| class == name) {
                    return None;
                }
            } else {
                specificity.ids += 1;
                if element.id.as_deref() != Some(name) {
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
            let pseudo = compound[start..i].to_ascii_lowercase();
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
            if matches!(pseudo.as_str(), "not" | "is" | "where") {
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
            specificity.classes += 1;
            let matches = match pseudo.as_str() {
                "root" if argument.is_empty() => element.parent.is_none(),
                "first-child" if argument.is_empty() => {
                    PreviousElementSibling(document, index).is_none()
                }
                "last-child" if argument.is_empty() => element.parent.is_some_and(|parent| {
                    document.elements[parent]
                        .children
                        .iter()
                        .filter_map(|child| match *child {
                            Child::Element(index) => Some(index),
                            _ => None,
                        })
                        .last()
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
    document: &Document,
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
        return MatchParsed(
            document,
            selector,
            part - 1,
            document.elements[index].parent?,
        )
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
    let mut ancestor = document.elements[index].parent;
    while let Some(candidate) = ancestor {
        if let Some(left) = MatchParsed(document, selector, part - 1, candidate) {
            return Some(left + local);
        }
        ancestor = document.elements[candidate].parent;
    }
    None
}

// cpp: style_resolver/style_resolver.cc:633-669
#[allow(non_snake_case)]
pub fn MatchSelector(document: &Document, index: usize, text: &str) -> Option<Specificity> {
    MatchSelectorTarget(document, index, text, PseudoTarget::Element)
}

// cpp: style_resolver/style_resolver.cc:633-669
#[allow(non_snake_case)]
pub fn MatchSelectorTarget(
    document: &Document,
    index: usize,
    text: &str,
    target: PseudoTarget,
) -> Option<Specificity> {
    ParseStrictSelectorList(text)?
        .into_iter()
        .filter_map(|mut selector| {
            if ExtractTerminalPseudo(&mut selector) != target {
                return None;
            }
            let mut specificity =
                MatchParsed(document, &selector, selector.compounds.len() - 1, index)?;
            if target != PseudoTarget::Element {
                specificity.types += 1;
            }
            Some(specificity)
        })
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_descendants_attributes_and_combinators() {
        let page = crate::test_html::Parse("<html><body><main><div class='card'><a target='_blank' id='x'>x</a><a>y</a></div></main></body></html>");
        assert!(MatchSelector(&page, 4, "main > div.card a#x[target='_blank']").is_some());
        assert!(MatchSelector(&page, 5, "a + a:last-child").is_some());
        assert!(MatchSelector(&page, 4, "a:hover").is_none());
        assert!(MatchSelector(&page, 4, "header a").is_none());
        assert!(MatchSelector(&page, 3, "div.card::after").is_none());
        assert_eq!(
            MatchSelectorTarget(&page, 3, "div.card::after", PseudoTarget::After),
            Some(Specificity {
                ids: 0,
                classes: 1,
                types: 2,
            })
        );
    }

    #[test]
    fn invalid_selector_invalidates_the_entire_list() {
        let page = crate::test_html::Parse("<html><body><h1>title</h1></body></html>");
        assert!(MatchSelector(&page, 2, "h1, h2").is_some());
        assert!(MatchSelector(&page, 2, "h1, h5.").is_none());
    }
}
