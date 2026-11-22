//! TreeScope::FindAnchorWithName/FindAnchor, tree_scope.cc:578-640.
//! Uses the canonical ID index and author-tree order; no geometry is inferred.
use super::{CompatibilityMode, DOMDocumentType, DOMNamespace, PersistentDocument};
impl PersistentDocument {
    /// Document.cc:1920-1922; ReadyState is the original document lifecycle owner.
    pub fn IsLoadCompleted(&self) -> bool {
        self.ReadyState() == crate::document_parser_lifecycle::DocumentReadyState::Complete
    }

    fn FindAnchorWithName(&self, name: &str) -> Option<usize> {
        if name.is_empty() {
            return None;
        }
        if let Some(element) = self
            .ElementsByIdInTreeScope(self.Root(), name)
            .into_iter()
            .next()
        {
            return Some(element);
        }
        let mut pending = self
            .Node(self.Root())
            .Children()
            .iter()
            .rev()
            .copied()
            .collect::<Vec<_>>();
        while let Some(node) = pending.pop() {
            let element = self.Node(node);
            pending.extend(element.Children().iter().rev().copied());
            if !element.IsElement("a", DOMNamespace::kHTML) {
                continue;
            }
            let name_value = element
                .FindAttribute("name")
                .map_or("", |attribute| attribute.value.as_str());
            if self.GetCompatibilityMode() == CompatibilityMode::kQuirks {
                if name_value == name {
                    self.CountUse(crate::use_counter::WebFeature::kAnchorCaseSensitiveMatch);
                    return Some(node);
                }
                if foundation::String::from(name_value)
                    .DeprecatedEqualIgnoringCase(&foundation::String::from(name))
                {
                    self.CountUse(crate::use_counter::WebFeature::kAnchorCaseInsensitiveMatch);
                    return Some(node);
                }
            } else if name_value == name {
                return Some(node);
            }
        }
        None
    }
    pub fn FindFragmentAnchor(&self, fragment: &str) -> Option<usize> {
        if self.DocumentType(self.Root()) != DOMDocumentType::kSVG {
            if let Some(anchor) = self.FindAnchorWithName(fragment) {
                return Some(anchor);
            }
        }
        let name = foundation::url_escape::DecodeUrlEscapeSequencesUtf8(fragment);
        if let Some(anchor) = self.FindAnchorWithName(&name) {
            return Some(anchor);
        }
        (fragment.is_empty() || name.eq_ignore_ascii_case("top")).then_some(self.Root())
    }
}

/// Source DisplayLockUtilities.cc:885-912 collects these original elements
/// before any beforematch listener can reenter. This is not a copied DOM tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoExpandableAncestorKind {
    Details,
    HiddenUntilFound,
}
impl PersistentDocument {
    pub fn AutoExpandableAncestors(&self, target: usize) -> Vec<(u64, AutoExpandableAncestorKind)> {
        let mut result = Vec::new();
        let mut current = Some(target);
        while let Some(ancestor) = current {
            let node = self.Node(ancestor);
            if node.Namespace() == DOMNamespace::kHTML
                && node
                    .FindAttribute("hidden")
                    .is_some_and(|a| a.value.eq_ignore_ascii_case("until-found"))
            {
                result.push((node.Id(), AutoExpandableAncestorKind::HiddenUntilFound));
            }
            if let Some(details) = node
                .Parent()
                .filter(|&parent| self.Node(parent).IsHTMLElement("details"))
            {
                if self.Node(details).FindAttribute("open").is_none()
                    && self
                        .DetailsContentSlot(details)
                        .is_some_and(|slot| self.AssignedSlot(ancestor) == Some(slot))
                {
                    result.push((self.Node(details).Id(), AutoExpandableAncestorKind::Details));
                }
            }
            current = self.FlatTreeParent(ancestor);
        }
        result
    }
}

impl PersistentDocument {
    /// TextFragmentFinder.cc:367-383; AnnotationAgentImpl.cc:304-310. The
    /// original Document::createRange starts collapsed; body selectNode is the
    /// source's only widening, rather than an invented whole-document range.
    pub fn CreateTextFragmentSearchRange(&mut self) -> crate::DOMRangeHandle {
        use crate::DOMBoundaryPoint as P;
        let root = P {
            node_id: self.Node(self.Root()).Id(),
            offset: 0,
        };
        let (start, end) = self
            .BodyElement()
            .and_then(|body| self.Node(body).Parent().map(|parent| (body, parent)))
            .map_or((root, root), |(body, parent)| {
                let offset = self
                    .Node(parent)
                    .Children()
                    .iter()
                    .position(|&child| child == body)
                    .expect("source body child index");
                (
                    P {
                        node_id: self.Node(parent).Id(),
                        offset,
                    },
                    P {
                        node_id: self.Node(parent).Id(),
                        offset: offset + 1,
                    },
                )
            });
        self.CreateRange(start, end)
            .expect("source document/body range")
    }
}
