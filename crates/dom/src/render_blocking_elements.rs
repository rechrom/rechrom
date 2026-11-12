//! Chromium HTMLLinkElement.cc575-624 and Element.cc3750-3788,8090-8097.
//! DOM emits only the source media-evaluation input; the loading owner evaluates
//! it synchronously before script reentry and registers on the same manager.
use crate::{
    persistent_document::PersistentDocument,
    render_blocking_resource_manager::{
        ExpectLinkOwner, HasRenderBlockingToken, RenderBlockingLevel,
    },
};
use std::rc::Rc;

pub enum RenderBlockingDocumentMutation {
    WillInsertBody,
    ScriptRemoved {
        script: crate::render_blocking_resource_manager::ScriptOwner,
        has_body: bool,
    },
    ScriptBlockingChanged {
        script: crate::render_blocking_resource_manager::ScriptOwner,
        has_body: bool,
        has_render_token: bool,
        asynchronous: bool,
        deferred: bool,
    },
    StylesheetRemoved {
        owner: crate::render_blocking_resource_manager::StylesheetOwner,
        has_body: bool,
    },
    MediaChanged {
        link: ExpectLinkOwner,
        target: Rc<[u16]>,
        media: String,
        has_body: bool,
        can_add: bool,
    },
    Add {
        link: ExpectLinkOwner,
        target: Rc<[u16]>,
        media: String,
        has_body: bool,
    },
    Remove {
        link: ExpectLinkOwner,
        target: Rc<[u16]>,
        has_body: bool,
    },
    Target {
        id: Rc<[u16]>,
        in_shadow_tree: bool,
        has_body: bool,
    },
}
impl RenderBlockingDocumentMutation {
    pub fn Media(&self) -> Option<&str> {
        match self {
            Self::Add { media, .. } | Self::MediaChanged { media, .. } => Some(media),
            _ => None,
        }
    }
}
impl PersistentDocument {
    fn IsExpectLink(&self, node: usize) -> bool {
        self.Node(node).IsHTMLElement("link")
            && self.Node(node).FindAttribute("rel").is_some_and(|a| {
                a.value
                    .split(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{000c}'))
                    .any(|token| token.eq_ignore_ascii_case("expect"))
            })
    }
    /// KURL::EqualIgnoringFragmentIdentifier, retaining the encoded fragment.
    fn ExpectSameDocumentId(&self, node: usize) -> Rc<[u16]> {
        let Some(href) = self
            .Node(node)
            .FindAttribute("href")
            .filter(|a| !a.value.is_empty())
        else {
            return Rc::from([]);
        };
        let Some(mut url) = self.BaseURL().CompleteURL(&href.value) else {
            return Rc::from([]);
        };
        let Some(fragment) = url.fragment().map(str::to_owned) else {
            return Rc::from([]);
        };
        let Ok(mut document_url) = url::Url::parse(self.DocumentURL()) else {
            return Rc::from([]);
        };
        url.set_fragment(None);
        document_url.set_fragment(None);
        if url != document_url {
            return Rc::from([]);
        }
        fragment.encode_utf16().collect::<Vec<_>>().into()
    }
    fn RecordRenderBlockingDocumentMutation(&self, request: RenderBlockingDocumentMutation) {
        self.document_metadata
            .get(&self.Node(self.Root()).Id())
            .expect("manager belongs to live Document metadata")
            .render_blocking_mutations
            .borrow_mut()
            .push_back(request);
    }
    pub(crate) fn QueueRenderBlockingWillInsertBody(&self) {
        if self.GetRenderBlockingResourceManager().is_some() {
            self.RecordRenderBlockingDocumentMutation(
                RenderBlockingDocumentMutation::WillInsertBody,
            );
        }
    }
    pub(crate) fn ScriptRenderBlockingAttributeChanged(&self, node: usize) {
        if !self.Node(node).IsHTMLElement("script") {
            return;
        }
        let Some(manager) = self.GetRenderBlockingResourceManager() else {
            return;
        };
        let has_body = manager.borrow().DocumentHasBody();
        let element = self.Node(node);
        self.RecordRenderBlockingDocumentMutation(
            RenderBlockingDocumentMutation::ScriptBlockingChanged {
                script: crate::render_blocking_resource_manager::ScriptOwner(element.Id()),
                has_body,
                has_render_token: element
                    .FindAttribute("blocking")
                    .is_some_and(|a| HasRenderBlockingToken(&a.value)),
                asynchronous: element.FindAttribute("async").is_some(),
                deferred: element.FindAttribute("defer").is_some(),
            },
        );
    }
    pub(crate) fn RenderBlockingElementRemoved(&self, node: usize) {
        // The caller is DOM's ordinary Remove/InsertBefore path. This package
        // has no moveBefore/state-preserving atomic-move operation.
        let Some(manager) = self.GetRenderBlockingResourceManager() else {
            return;
        };
        let has_body = manager.borrow().DocumentHasBody();
        if self.Node(node).IsHTMLElement("script") {
            self.RecordRenderBlockingDocumentMutation(
                RenderBlockingDocumentMutation::ScriptRemoved {
                    script: crate::render_blocking_resource_manager::ScriptOwner(
                        self.Node(node).Id(),
                    ),
                    has_body,
                },
            );
        } else if self.Node(node).IsStyleElement() || self.Node(node).IsHTMLElement("link") {
            self.RecordRenderBlockingDocumentMutation(
                RenderBlockingDocumentMutation::StylesheetRemoved {
                    owner: crate::render_blocking_resource_manager::StylesheetOwner(
                        self.Node(node).Id(),
                    ),
                    has_body,
                },
            );
        }
    }
    /// Called before changing rel/href/blocking/media and on actual removal.
    pub(crate) fn RemoveExpectRenderBlockingLink(&mut self, node: usize) {
        if !self.IsExpectLink(node) {
            return;
        }
        if let Some(manager) = self.GetRenderBlockingResourceManager() {
            let has_body = manager.borrow().DocumentHasBody();
            self.RecordRenderBlockingDocumentMutation(RenderBlockingDocumentMutation::Remove {
                target: self.ExpectSameDocumentId(node),
                link: ExpectLinkOwner(self.Node(node).Id()),
                has_body,
            });
        }
    }
    /// HTMLLinkElement.cc529-572: a matching new media query only adds; it
    /// must not transiently remove an already blocking expect link.
    pub(crate) fn ExpectLinkAttributeChanged(&mut self, node: usize, name: &str) {
        if name == "media" {
            if !self.IsExpectLink(node) {
                return;
            }
            let Some(manager) = self.GetRenderBlockingResourceManager() else {
                return;
            };
            let has_body = manager.borrow().DocumentHasBody();
            self.RecordRenderBlockingDocumentMutation(
                RenderBlockingDocumentMutation::MediaChanged {
                    link: ExpectLinkOwner(self.Node(node).Id()),
                    target: self.ExpectSameDocumentId(node),
                    media: self
                        .Node(node)
                        .FindAttribute("media")
                        .map_or_else(String::new, |a| a.value.clone()),
                    has_body,
                    can_add: self.IsConnectedToDocument(node)
                        && self
                            .Node(node)
                            .FindAttribute("blocking")
                            .is_some_and(|a| HasRenderBlockingToken(&a.value)),
                },
            );
        } else if name != "blocking"
            || self
                .Node(node)
                .FindAttribute("blocking")
                .is_some_and(|a| HasRenderBlockingToken(&a.value))
        {
            self.QueueExpectRenderBlockingLink(node);
        }
    }
    pub(crate) fn QueueExpectRenderBlockingLink(&mut self, node: usize) {
        if !self.IsExpectLink(node) {
            return;
        }
        self.CountWebDX(crate::use_counter::WebDXFeature::kLinkRelExpect);
        if !self.IsConnectedToDocument(node)
            || !self
                .Node(node)
                .FindAttribute("blocking")
                .is_some_and(|a| HasRenderBlockingToken(&a.value))
        {
            return;
        }
        if self.GetRenderBlockingResourceManager().is_none() {
            return;
        }
        let target = self.ExpectSameDocumentId(node);
        let media = self
            .Node(node)
            .FindAttribute("media")
            .map_or_else(String::new, |a| a.value.clone());
        let has_body = self
            .GetRenderBlockingResourceManager()
            .unwrap()
            .borrow()
            .DocumentHasBody();
        self.RecordRenderBlockingDocumentMutation(RenderBlockingDocumentMutation::Add {
            link: ExpectLinkOwner(self.Node(node).Id()),
            target,
            media,
            has_body,
        });
    }
    pub fn TakeRenderBlockingDocumentMutations(
        &self,
    ) -> std::collections::VecDeque<RenderBlockingDocumentMutation> {
        self.document_metadata
            .get(&self.Node(self.Root()).Id())
            .map_or_else(Default::default, |m| {
                std::mem::take(&mut *m.render_blocking_mutations.borrow_mut())
            })
    }
    pub fn ApplyRenderBlockingDocumentMutation(
        &self,
        request: RenderBlockingDocumentMutation,
        media_matches: bool,
    ) {
        if let Some(manager) = self.GetRenderBlockingResourceManager() {
            let mut manager = manager.borrow_mut();
            match request {
                RenderBlockingDocumentMutation::WillInsertBody => {
                    manager.WillInsertDocumentBody();
                    manager.BeginLifecycleUpdatesIfRenderingReady();
                }
                RenderBlockingDocumentMutation::ScriptRemoved { script, has_body } => {
                    manager.RemovePendingScript(script, has_body)
                }
                RenderBlockingDocumentMutation::ScriptBlockingChanged {
                    script,
                    has_body,
                    has_render_token,
                    asynchronous,
                    deferred,
                } => manager.ScriptBlockingAttributeChanged(
                    script,
                    has_render_token,
                    asynchronous,
                    deferred,
                    has_body,
                ),
                RenderBlockingDocumentMutation::StylesheetRemoved { owner, has_body } => {
                    manager.RemovePendingStylesheet(owner, has_body);
                }
                RenderBlockingDocumentMutation::MediaChanged {
                    link,
                    target,
                    has_body,
                    can_add,
                    ..
                } => {
                    if media_matches {
                        self.CountWebDX(crate::use_counter::WebDXFeature::kLinkRelExpect);
                        if can_add {
                            manager.AddPendingParsingElementLink(
                                target,
                                link,
                                RenderBlockingLevel::Block,
                                has_body,
                            );
                        }
                    } else {
                        manager.RemovePendingParsingElementLink(&target, link, has_body);
                    }
                }
                RenderBlockingDocumentMutation::Add {
                    target,
                    link,
                    has_body,
                    ..
                } if media_matches => manager.AddPendingParsingElementLink(
                    target,
                    link,
                    RenderBlockingLevel::Block,
                    has_body,
                ),
                RenderBlockingDocumentMutation::Add { .. } => {}
                RenderBlockingDocumentMutation::Remove {
                    target,
                    link,
                    has_body,
                } => manager.RemovePendingParsingElementLink(&target, link, has_body),
                RenderBlockingDocumentMutation::Target {
                    id,
                    in_shadow_tree,
                    has_body,
                } => manager.RemovePendingParsingElement(&id, in_shadow_tree, true, has_body),
            }
        }
    }
    pub(crate) fn RenderBlockingBeginParsingChildren(&mut self, element: usize) {
        let id = self.Node(element).Id();
        let root = self.Node(self.Root()).Id();
        self.document_metadata
            .entry(root)
            .or_default()
            .parsing_element_children
            .borrow_mut()
            .insert(id);
    }
    pub(crate) fn RenderBlockingFinishParsingChildren(&mut self, element: usize) {
        if let Some(metadata) = self.document_metadata.get(&self.Node(self.Root()).Id()) {
            metadata
                .parsing_element_children
                .borrow_mut()
                .remove(&self.Node(element).Id());
        }
        self.ProcessElementRenderBlocking(element);
        self.ProcessAnchorNameRenderBlocking(element);
    }
    pub(crate) fn ProcessAnchorNameRenderBlocking(&mut self, element: usize) {
        if !self.Node(element).IsHTMLElement("a") {
            return;
        }
        if let Some(name) = self.Node(element).AttributeValueUTF16("name") {
            self.ProcessRenderBlockingValue(element, name);
        }
    }
    pub(crate) fn ProcessElementRenderBlockingIdAttribute(&mut self, element: usize) {
        if self.GetCompatibilityMode() == crate::persistent_document::CompatibilityMode::kQuirks {
            if let Some(id) = self.Node(element).AttributeValueUTF16("id") {
                if id.iter().any(|c| (b'A' as u16..=b'Z' as u16).contains(c)) {
                    self.ProcessRenderBlockingValue(
                        element,
                        id.iter()
                            .map(|c| {
                                if (b'A' as u16..=b'Z' as u16).contains(c) {
                                    c + 32
                                } else {
                                    *c
                                }
                            })
                            .collect::<Vec<_>>()
                            .into(),
                    );
                }
            }
        }
        self.ProcessElementRenderBlocking(element);
    }
    pub(crate) fn ProcessElementRenderBlocking(&mut self, element: usize) {
        if let Some(id) = self.Node(element).AttributeValueUTF16("id") {
            self.ProcessRenderBlockingValue(element, id);
        }
    }
    fn ProcessRenderBlockingValue(&mut self, element: usize, id: Rc<[u16]>) {
        if self.GetRenderBlockingResourceManager().is_none() || !self.IsConnectedToDocument(element)
        {
            return;
        }
        if self
            .document_metadata
            .get(&self.Node(self.Root()).Id())
            .is_some_and(|m| {
                m.parsing_element_children
                    .borrow()
                    .contains(&self.Node(element).Id())
            })
        {
            return;
        }
        if id.is_empty() {
            return;
        }
        if let Some(manager) = self.GetRenderBlockingResourceManager() {
            let has_body = manager.borrow().DocumentHasBody();
            self.RecordRenderBlockingDocumentMutation(RenderBlockingDocumentMutation::Target {
                id,
                in_shadow_tree: self.IsInShadowTree(element),
                has_body,
            });
        }
    }
    /// Document::OnPrepareToStopParsing, document.cc8010-8015. Console payload
    /// is captured at this lifecycle point, never from later mutable attributes.
    pub fn ClearPendingParsingRenderBlockingElements(&self) {
        if let Some(manager) = self.GetRenderBlockingResourceManager() {
            let body = manager.borrow().DocumentHasBody();
            manager
                .borrow_mut()
                .ClearPendingParsingElements(body, |link| {
                    let index = self
                        .FindNodeById(link.0)
                        .expect("live source expect-link owner");
                    self.Node(index)
                        .FindAttribute("href")
                        .map_or_else(String::new, |a| a.value.clone())
                });
        }
    }
}
