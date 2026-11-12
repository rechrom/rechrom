// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native fragment creation from core/editing/serializers/serialization.cc
//! 153–181,402–527. All nodes are allocated in the destination DOM arena.
use super::{
    html_parser::{InertHTMLParserHost, ParseHTMLFragmentUTF16WithPolicyAndHost},
    parser::parser_content_policy::ParserContentPolicy,
    HTMLParserHost,
};
use dom::persistent_document::{DOMNamespace, DOMNodeType, PersistentDocument};

pub const FRAGMENT_MARKER: &str = "webkit-fragment-marker";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FragmentContextError {
    InvalidOffsets,
}

/// serialization.cc:402–422. A detached body is the parser context, not a
/// wrapper in the returned fragment. Resource/script execution stays inert.
pub fn CreateFragmentFromMarkup(
    document: &mut PersistentDocument,
    markup: &[u16],
    base_url: &str,
    policy: ParserContentPolicy,
    scripting_flag: bool,
) -> usize {
    let mut host = InertHTMLParserHost;
    CreateFragmentFromMarkupAndHost(
        document,
        markup,
        base_url,
        policy,
        scripting_flag,
        &mut host,
    )
}

pub fn CreateFragmentFromMarkupAndHost(
    document: &mut PersistentDocument,
    markup: &[u16],
    base_url: &str,
    policy: ParserContentPolicy,
    scripting_flag: bool,
    host: &mut dyn HTMLParserHost,
) -> usize {
    let body = document.CreateElement(DOMNamespace::kHTML, "body".into(), Vec::new());
    let fragment = ParseHTMLFragmentUTF16WithPolicyAndHost(
        document,
        body,
        markup,
        policy,
        scripting_flag,
        host,
    );
    if !base_url.is_empty() && base_url != "about:blank" && base_url != document.BaseURL().AsStr() {
        document.CompleteMarkupURLs(fragment, base_url);
    }
    fragment
}
fn next(document: &PersistentDocument, node: usize, root: usize, children: bool) -> Option<usize> {
    if children {
        if let Some(first) = document.Node(node).Children().first() {
            return Some(*first);
        }
    }
    let mut current = node;
    while current != root {
        let parent = document.Node(current).Parent()?;
        let siblings = document.Node(parent).Children();
        let index = siblings.iter().position(|s| *s == current)?;
        if let Some(sibling) = siblings.get(index + 1) {
            return Some(*sibling);
        }
        current = parent;
    }
    None
}
fn contains(document: &PersistentDocument, ancestor: usize, node: usize) -> bool {
    let mut current = Some(node);
    while let Some(node) = current {
        if node == ancestor {
            return true;
        }
        current = document.Node(node).Parent();
    }
    false
}
fn find_markers(document: &PersistentDocument, fragment: usize) -> Option<(usize, usize)> {
    let mut node = document.Node(fragment).Children().first().copied();
    let mut before = None;
    while let Some(current) = node {
        if document.Node(current).Type() == DOMNodeType::kComment
            && document
                .CharacterDataUTF16Slice(current)
                .iter()
                .copied()
                .eq(FRAGMENT_MARKER.encode_utf16())
        {
            if let Some(before) = before {
                return Some((before, current));
            }
            before = Some(current);
        }
        node = next(document, current, fragment, true);
    }
    None
}
fn trim(document: &mut PersistentDocument, fragment: usize, before: usize, after: usize) {
    let mut node = document.Node(fragment).Children().first().copied();
    while let Some(current) = node {
        if current != before && contains(document, current, before) {
            node = next(document, current, fragment, true);
            continue;
        }
        node = next(document, current, fragment, false);
        debug_assert!(!contains(document, current, after));
        document.Remove(current);
        if current == before {
            break;
        }
    }
    let mut node = Some(after);
    while let Some(current) = node {
        node = next(document, current, fragment, false);
        document.Remove(current);
    }
}
fn non_cell_block(document: &PersistentDocument, node: usize) -> bool {
    let n = document.Node(node);
    n.Namespace() == DOMNamespace::kHTML
        && matches!(
            n.Name(),
            "listing" | "ol" | "pre" | "table" | "ul" | "xmp" | "h1" | "h2" | "h3" | "h4" | "h5"
        )
}
fn retain_structure(document: &PersistentDocument, common: usize) -> Option<usize> {
    // serialization.cc:183–225. EnclosingNodeOfType starts at the common
    // ancestor itself; no computed layout is consulted by this source helper.
    let mut block = Some(common);
    while let Some(node) = block {
        let n = document.Node(node);
        if non_cell_block(document, node) || n.IsHTMLElement("td") || n.IsHTMLElement("th") {
            return non_cell_block(document, node).then_some(node);
        }
        block = n.Parent();
    }
    None
}
/// serialization.cc:468–527. Offsets count UTF-16 code units, including lone
/// surrogates; marker comments and structure retain their original identities.
pub fn CreateFragmentFromMarkupWithContext(
    document: &mut PersistentDocument,
    markup: &[u16],
    start: usize,
    end: usize,
    base_url: &str,
    policy: ParserContentPolicy,
    scripting_flag: bool,
) -> Result<Option<usize>, FragmentContextError> {
    let mut host = InertHTMLParserHost;
    CreateFragmentFromMarkupWithContextAndHost(
        document,
        markup,
        start,
        end,
        base_url,
        policy,
        scripting_flag,
        &mut host,
    )
}

pub fn CreateFragmentFromMarkupWithContextAndHost(
    document: &mut PersistentDocument,
    markup: &[u16],
    start: usize,
    end: usize,
    base_url: &str,
    policy: ParserContentPolicy,
    scripting_flag: bool,
    host: &mut dyn HTMLParserHost,
) -> Result<Option<usize>, FragmentContextError> {
    if start > end || end > markup.len() {
        return Err(FragmentContextError::InvalidOffsets);
    }
    let marker: Vec<u16> = format!("<!--{FRAGMENT_MARKER}-->").encode_utf16().collect();
    let mut tagged = Vec::with_capacity(markup.len() + 2 * marker.len());
    tagged.extend_from_slice(&markup[..start]);
    tagged.extend_from_slice(&marker);
    tagged.extend_from_slice(&markup[start..end]);
    tagged.extend_from_slice(&marker);
    tagged.extend_from_slice(&markup[end..]);
    let tagged_fragment =
        CreateFragmentFromMarkupAndHost(document, &tagged, base_url, policy, scripting_flag, host);
    let Some((before, after)) = find_markers(document, tagged_fragment) else {
        return Ok(None);
    };
    let (tagged_document, root) = document.CreateMarkupContextDocument();
    document.TakeAllChildren(tagged_fragment, root);
    let before_parent = document
        .Node(before)
        .Parent()
        .expect("parsed marker has a parent");
    let after_parent = document
        .Node(after)
        .Parent()
        .expect("parsed marker has a parent");
    let mut common = before_parent;
    while !contains(document, common, after_parent) {
        common = document
            .Node(common)
            .Parent()
            .expect("markers share the tagged document");
    }
    let special = retain_structure(document, common);
    let fragment = document.CreateDocumentFragment();
    if let Some(special) = special {
        document.AppendChild(fragment, special);
    } else {
        document.TakeAllChildren(common, fragment);
    }
    trim(document, fragment, before, after);
    // The temporary generic Document remains an arena owner until normal
    // collection. The result was adopted into the destination document.
    let _ = tagged_document;
    Ok(Some(fragment))
}
