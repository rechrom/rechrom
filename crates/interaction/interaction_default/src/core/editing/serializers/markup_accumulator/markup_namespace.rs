// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! markup_accumulator.h:119–167; markup_accumulator.cc:93–179,295–630.
//! XML namespace contexts are scoped to the original element traversal. Prefix
//! generation is shared across contexts; it never mutates the source attributes.
use super::{fmt, MarkupDocument, ResolveUrls, SerializationType};
const XML: &str = "http://www.w3.org/XML/1998/namespace";
const XMLNS: &str = "http://www.w3.org/2000/xmlns/";
fn Units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}
fn Equal(units: &[u16], text: &str) -> bool {
    units.iter().copied().eq(text.encode_utf16())
}
#[derive(Clone, Default)]
struct Context {
    default: Vec<u16>,
    // Keep insertion order as well as current mappings, matching Blink's
    // prefix_ns_map_ and ns_prefixes_map_ (including superseded entries).
    prefixes: Vec<(Vec<u16>, Vec<u16>)>,
}
impl Context {
    fn Lookup(&self, prefix: &[u16]) -> Option<&[u16]> {
        self.prefixes
            .iter()
            .rev()
            .find(|(p, _)| p == prefix)
            .map(|(_, ns)| ns.as_slice())
            .filter(|ns| !ns.is_empty())
    }
    fn Add(&mut self, prefix: Vec<u16>, ns: Vec<u16>) {
        self.prefixes.push((prefix, ns));
    }
    fn Preferred(&self, ns: &[u16], preferred: &[u16]) -> Option<Vec<u16>> {
        let mapped = self.Lookup(preferred);
        if !preferred.is_empty() && mapped == Some(ns) {
            return Some(preferred.to_vec());
        }
        for (prefix, candidate_ns) in self.prefixes.iter().rev() {
            if candidate_ns == ns && self.Lookup(prefix) == Some(ns) {
                return Some(prefix.clone());
            }
        }
        (!preferred.is_empty() && mapped.is_none()).then(|| preferred.to_vec())
    }
}
pub(super) struct NamespaceState {
    stack: Vec<Context>,
    index: u32,
}
impl NamespaceState {
    pub fn new() -> Self {
        let mut context = Context::default();
        context.Add(Units("xml"), Units(XML));
        Self {
            stack: vec![context],
            index: 1,
        }
    }
    pub fn Push(&mut self) {
        self.stack.push(self.stack.last().unwrap().clone());
    }
    pub fn Pop(&mut self) {
        self.stack.pop();
    }
    fn Context(&mut self) -> &mut Context {
        self.stack.last_mut().unwrap()
    }
    fn Generate(&mut self, ns: &[u16]) -> Vec<u16> {
        loop {
            let prefix = Units(&format!("ns{}", self.index));
            self.index += 1;
            if self.Context().Lookup(&prefix).is_none() {
                self.Context().Add(prefix.clone(), ns.to_vec());
                return prefix;
            }
        }
    }
    fn Namespace(out: &mut Vec<u16>, prefix: &[u16], ns: &[u16]) {
        if prefix.is_empty() {
            fmt::AppendAttributeWithType(out, &[], &Units("xmlns"), ns, SerializationType::Xml);
        } else {
            fmt::AppendAttributeWithType(out, &Units("xmlns"), prefix, ns, SerializationType::Xml);
        }
    }
    pub fn AppendElement<D: MarkupDocument>(
        &mut self,
        document: &D,
        node: D::Node,
        resolve: ResolveUrls,
        out: &mut Vec<u16>,
    ) -> Vec<u16> {
        let name = document.QualifiedName(node);
        let (prefix, local) = name
            .iter()
            .position(|c| *c == 58)
            .map_or((&[][..], name.as_slice()), |i| (&name[..i], &name[i + 1..]));
        let ns = document.ElementNamespace(node).to_vec();
        let mut local_default = None;
        for i in 0..document.AttributeCount(node) {
            let a = document.Attribute(node, i);
            if a.prefix.is_empty() && (Equal(a.local_name, "xmlns") || Equal(a.namespace, XMLNS)) {
                local_default = Some(a.value.to_vec());
            } else if Equal(a.prefix, "xmlns") {
                self.Context().Add(a.local_name.to_vec(), a.value.to_vec());
            }
        }
        let mut ignore_default = false;
        let serialized_prefix;
        let mut declaration = None;
        if self.Context().default == ns {
            ignore_default = local_default.is_some();
            serialized_prefix = Vec::new();
        } else {
            let candidate = if !ns.is_empty()
                && (!prefix.is_empty() || local_default.as_deref() != Some(ns.as_slice()))
            {
                self.Context().Preferred(&ns, prefix)
            } else {
                None
            };
            if let Some(candidate) = candidate.filter(|p| self.Context().Lookup(p).is_some()) {
                serialized_prefix = candidate;
                if let Some(default) = &local_default {
                    if !Equal(default, XML) {
                        self.Context().default = default.clone();
                    }
                }
            } else if !prefix.is_empty() {
                serialized_prefix = if (0..document.AttributeCount(node)).any(|i| {
                    let a = document.Attribute(node, i);
                    Equal(a.prefix, "xmlns") && a.local_name == prefix
                        || a.prefix.is_empty()
                            && a.local_name
                                .iter()
                                .copied()
                                .eq("xmlns:".encode_utf16().chain(prefix.iter().copied()))
                }) {
                    self.Generate(&ns)
                } else {
                    self.Context().Add(prefix.to_vec(), ns.clone());
                    prefix.to_vec()
                };
                declaration = Some((serialized_prefix.clone(), ns.clone()));
                if let Some(default) = &local_default {
                    self.Context().default = default.clone();
                }
            } else if local_default.as_deref() != Some(ns.as_slice()) {
                ignore_default = true;
                self.Context().default = ns.clone();
                serialized_prefix = prefix.to_vec();
                declaration = Some((Vec::new(), ns.clone()));
            } else {
                self.Context().default = ns.clone();
                serialized_prefix = prefix.to_vec();
            }
        }
        let mut qualified = serialized_prefix;
        if !qualified.is_empty() {
            qualified.push(58);
        }
        qualified.extend_from_slice(local);
        out.push(60);
        out.extend_from_slice(&qualified);
        if let Some((prefix, ns)) = declaration {
            Self::Namespace(out, &prefix, &ns);
        }
        for i in 0..document.AttributeCount(node) {
            let a = document.Attribute(node, i);
            if ignore_default
                && a.prefix.is_empty()
                && (Equal(a.local_name, "xmlns") || Equal(a.namespace, XMLNS))
                && a.value != ns.as_slice()
            {
                continue;
            }
            let resolved = document.ResolvedAttributeValue(node, i, resolve);
            let value = resolved.as_deref().unwrap_or(a.value);
            let mut candidate = if a.namespace.is_empty() {
                None
            } else {
                self.Context().Preferred(a.namespace, a.prefix)
            };
            if Equal(a.namespace, XMLNS) {
                if a.prefix.is_empty() && !Equal(a.local_name, "xmlns") {
                    candidate = Some(Units("xmlns"));
                }
            } else if !a.namespace.is_empty()
                && candidate
                    .as_ref()
                    .is_none_or(|p| self.Context().Lookup(p) != Some(a.namespace))
            {
                let prefix = if candidate
                    .as_ref()
                    .is_none_or(|p| self.Context().Lookup(p).is_some())
                {
                    self.Generate(a.namespace)
                } else {
                    let p = candidate.unwrap();
                    self.Context().Add(p.clone(), a.namespace.to_vec());
                    p
                };
                Self::Namespace(out, &prefix, a.namespace);
                candidate = Some(prefix);
            }
            fmt::AppendAttributeWithType(
                out,
                candidate.as_deref().unwrap_or(&[]),
                a.local_name,
                value,
                SerializationType::Xml,
            );
        }
        qualified
    }
}
