// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! markup_accumulator.h:55–167; markup_accumulator.cc:93–630,702–785. The adapter borrows
//! canonical node/attribute storage; traversal never creates another DOM tree.
use super::markup_formatter::{self as fmt, ResolveUrls, SerializationType};
#[path = "markup_namespace.rs"]
mod markup_namespace;
use markup_namespace::NamespaceState;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkupNodeKind {
    Element,
    Text,
    Comment,
    Document,
    DocumentFragment,
    DocumentType,
    ProcessingInstruction,
}
pub struct MarkupAttribute<'a> {
    pub prefix: &'a [u16],
    pub local_name: &'a [u16],
    pub namespace: &'a [u16],
    pub value: &'a [u16],
}
pub trait MarkupDocument {
    type Node: Copy;
    fn Kind(&self, node: Self::Node) -> MarkupNodeKind;
    fn QualifiedName(&self, node: Self::Node) -> Vec<u16>;
    fn Data(&self, node: Self::Node) -> &[u16];
    fn ChildCount(&self, node: Self::Node) -> usize;
    fn ChildAt(&self, node: Self::Node, index: usize) -> Self::Node;
    fn TemplateContents(&self, node: Self::Node) -> Option<Self::Node>;
    fn IsHTMLVoidElement(&self, node: Self::Node) -> bool;
    fn TextIsRaw(&self, node: Self::Node) -> bool;
    fn AttributeCount(&self, node: Self::Node) -> usize;
    fn Attribute(&self, node: Self::Node, index: usize) -> MarkupAttribute<'_>;
    fn ResolvedAttributeValue(
        &self,
        node: Self::Node,
        index: usize,
        resolve: ResolveUrls,
    ) -> Option<Vec<u16>>;
    fn ElementIsValue(&self, node: Self::Node) -> Option<&[u16]>;
    fn SerializationType(&self) -> SerializationType;
    fn ElementNamespace(&self, node: Self::Node) -> &[u16];
    fn IsHTMLElement(&self, node: Self::Node) -> bool;
    fn IsCDATASection(&self, node: Self::Node) -> bool;
    fn DocumentTypeIdentifiers(&self, node: Self::Node) -> (&[u16], &[u16]);
    fn XMLDeclaration(&self) -> Option<(&[u16], &[u16], Option<bool>)>;
}
/// MarkupFormatter.cc:188–218,285–343. Non-element formatting is shared by
/// ordinary and range serializers, borrowing the original metadata/code units.
pub(super) fn AppendNonElement<D: MarkupDocument>(
    document: &D,
    node: D::Node,
    serialization: SerializationType,
    out: &mut Vec<u16>,
) {
    match document.Kind(node) {
        MarkupNodeKind::Text => {
            if serialization == SerializationType::Xml && document.IsCDATASection(node) {
                fmt::Append(out, "<![CDATA[");
                out.extend_from_slice(document.Data(node));
                fmt::Append(out, "]]>");
            } else {
                fmt::AppendCharactersReplacingEntities(
                    out,
                    document.Data(node),
                    if serialization == SerializationType::Xml {
                        fmt::PCDATA
                    } else if document.TextIsRaw(node) {
                        0
                    } else {
                        fmt::HTML_PCDATA
                    },
                );
            }
        }
        MarkupNodeKind::Comment => {
            fmt::Append(out, "<!--");
            out.extend_from_slice(document.Data(node));
            fmt::Append(out, "-->");
        }
        MarkupNodeKind::DocumentType => {
            let name = document.QualifiedName(node);
            if name.is_empty() {
                return;
            }
            fmt::Append(out, "<!DOCTYPE ");
            out.extend(name);
            let (public, system) = document.DocumentTypeIdentifiers(node);
            if !public.is_empty() {
                fmt::Append(out, " PUBLIC \"");
                out.extend_from_slice(public);
                out.push(34);
                if !system.is_empty() {
                    fmt::Append(out, " \"");
                    out.extend_from_slice(system);
                    out.push(34);
                }
            } else if !system.is_empty() {
                fmt::Append(out, " SYSTEM \"");
                out.extend_from_slice(system);
                out.push(34);
            }
            out.push(62);
        }
        MarkupNodeKind::ProcessingInstruction => {
            fmt::Append(out, "<?");
            out.extend(document.QualifiedName(node));
            out.push(32);
            out.extend_from_slice(document.Data(node));
            fmt::Append(out, "?>");
        }
        MarkupNodeKind::Document => {
            if let Some((version, encoding, standalone)) = document.XMLDeclaration() {
                fmt::Append(out, "<?xml version=\"");
                out.extend_from_slice(version);
                if !encoding.is_empty() {
                    fmt::Append(out, "\" encoding=\"");
                    out.extend_from_slice(encoding);
                }
                if let Some(standalone) = standalone {
                    fmt::Append(out, "\" standalone=\"");
                    fmt::Append(out, if standalone { "yes" } else { "no" });
                }
                fmt::Append(out, "\"?>");
            }
        }
        MarkupNodeKind::DocumentFragment => {}
        MarkupNodeKind::Element => unreachable!("element serializer owns element formatting"),
    }
}
pub(super) fn ShouldSelfClose<D: MarkupDocument>(
    document: &D,
    node: D::Node,
    serialization: SerializationType,
) -> bool {
    serialization == SerializationType::Xml
        && document.ChildCount(node) == 0
        && (!document.IsHTMLElement(node) || document.IsHTMLVoidElement(node))
}
fn AppendAttributes<D: MarkupDocument>(
    document: &D,
    node: D::Node,
    resolve: ResolveUrls,
    out: &mut Vec<u16>,
) {
    let has_is = (0..document.AttributeCount(node)).any(|i| {
        let a = document.Attribute(node, i);
        a.namespace.is_empty() && a.local_name.iter().copied().eq("is".encode_utf16())
    });
    if !has_is {
        if let Some(value) = document.ElementIsValue(node) {
            fmt::AppendAttribute(out, &[], &[105, 115], value);
        }
    }
    for index in 0..document.AttributeCount(node) {
        let a = document.Attribute(node, index);
        let namespace = String::from_utf16_lossy(a.namespace);
        let prefix = match namespace.as_str() {
            "http://www.w3.org/XML/1998/namespace" => Some("xml"),
            "http://www.w3.org/1999/xlink" => Some("xlink"),
            "http://www.w3.org/2000/xmlns/"
                if a.prefix.is_empty()
                    && !a.local_name.iter().copied().eq("xmlns".encode_utf16()) =>
            {
                Some("xmlns")
            }
            _ => None,
        }
        .map(|s| s.encode_utf16().collect::<Vec<_>>());
        let value = document.ResolvedAttributeValue(node, index, resolve);
        fmt::AppendAttribute(
            out,
            prefix.as_deref().unwrap_or(a.prefix),
            a.local_name,
            value.as_deref().unwrap_or(a.value),
        );
    }
}
fn SerializeNodes<D: MarkupDocument>(
    document: &D,
    node: D::Node,
    children_only: bool,
    resolve: ResolveUrls,
    namespaces: &mut NamespaceState,
    out: &mut Vec<u16>,
) {
    let kind = document.Kind(node);
    let serialization = document.SerializationType();
    let mut name = document.QualifiedName(node);
    if kind == MarkupNodeKind::Element && serialization == SerializationType::Xml {
        namespaces.Push();
    }
    if !children_only {
        if kind == MarkupNodeKind::Element {
            if serialization == SerializationType::Xml {
                name = namespaces.AppendElement(document, node, resolve, out);
            } else {
                out.push(60);
                out.extend_from_slice(&name);
                AppendAttributes(document, node, resolve, out);
            }
            if ShouldSelfClose(document, node, serialization) {
                if document.IsHTMLElement(node) {
                    out.push(32);
                }
                out.push(47);
            }
            out.push(62);
        } else {
            AppendNonElement(document, node, serialization, out);
        }
    }
    let has_end_tag = !(kind == MarkupNodeKind::Element
        && serialization == SerializationType::Html
        && document.IsHTMLVoidElement(node));
    if has_end_tag {
        let parent = document.TemplateContents(node).unwrap_or(node);
        for index in 0..document.ChildCount(parent) {
            SerializeNodes(
                document,
                document.ChildAt(parent, index),
                false,
                resolve,
                namespaces,
                out,
            );
        }
        if kind == MarkupNodeKind::Element
            && !children_only
            && !ShouldSelfClose(document, node, serialization)
        {
            fmt::Append(out, "</");
            out.extend(name);
            out.push(62);
        }
    }
    if kind == MarkupNodeKind::Element && serialization == SerializationType::Xml {
        namespaces.Pop();
    }
}
pub fn CreateMarkup<D: MarkupDocument>(
    document: &D,
    node: D::Node,
    children_only: bool,
    resolve: ResolveUrls,
) -> Vec<u16> {
    let mut result = Vec::new();
    SerializeNodes(
        document,
        node,
        children_only,
        resolve,
        &mut NamespaceState::new(),
        &mut result,
    );
    result
}

#[cfg(test)]
#[path = "markup_accumulator_tests.rs"]
mod tests;
