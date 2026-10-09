#![allow(non_snake_case)]

//! Resource discovery for DOM nodes which become connected after parsing.
//!
//! DocumentEngine identifies the affected subtree. This service owns the
//! traversal and HTML resource semantics, while Page only routes its effects
//! into the document/rendering lifecycle.

use document_loader::ResourceFetcher;
use dom::{persistent_document::DOMNodeType, Document, DOM};
use std::{cell::RefCell, io, rc::Rc};

pub type ConnectedScriptPreparer = dyn Fn(&Document, usize) -> io::Result<()>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConnectedResourceEffects {
    pub connected: bool,
    pub style_sheet_changed: bool,
}

pub struct ConnectedResourceDiscovery {
    document: Rc<RefCell<DOM>>,
    resources: RefCell<Option<Rc<ResourceFetcher>>>,
    prepare_script: RefCell<Option<Rc<ConnectedScriptPreparer>>>,
}

impl ConnectedResourceDiscovery {
    pub fn new(document: Rc<RefCell<DOM>>) -> Self {
        Self {
            document,
            resources: RefCell::new(None),
            prepare_script: RefCell::new(None),
        }
    }

    pub fn SetResources(&self, resources: Rc<ResourceFetcher>) {
        *self.resources.borrow_mut() = Some(resources);
    }

    pub fn SetScriptPreparer(&self, prepare: Option<Rc<ConnectedScriptPreparer>>) {
        *self.prepare_script.borrow_mut() = prepare;
    }

    pub fn PrepareNode(
        &self,
        node: usize,
        scripts: bool,
        image_request: bool,
        dispatch_image: &mut dyn FnMut(u64, bool),
    ) -> io::Result<ConnectedResourceEffects> {
        {
            let owner = self.document.borrow();
            let tree = owner.GetDocument();
            let mut root = node;
            while let Some(parent) = tree.Node(root).Parent() {
                root = parent;
            }
            if tree.Node(root).Type() != DOMNodeType::kDocument {
                return Ok(ConnectedResourceEffects::default());
            }
        }

        let resources = self.resources.borrow().clone();
        let mut style_sheet_changed = false;
        if let Some(resources) = &resources {
            let image = {
                let owner = self.document.borrow();
                let tree = owner.GetDocument();
                let current = tree.Node(node);
                if image_request && current.IsHTMLElement("img") {
                    current
                        .FindAttribute("src")
                        .filter(|attribute| !attribute.value.is_empty())
                        .map(|attribute| {
                            (
                                current.Id(),
                                attribute.value.clone(),
                                tree.ImageResourceFor(&attribute.value).is_some(),
                                !current.FindAttribute("loading").is_some_and(|loading| {
                                    loading.value.eq_ignore_ascii_case("lazy")
                                }),
                            )
                        })
                } else {
                    None
                }
            };
            if let Some((id, source, cached, blocks_load)) = image {
                if cached {
                    dispatch_image(id, true);
                } else {
                    resources.QueueImageWithLoadBlocking(&source, None, blocks_load)?;
                }
            }

            let style = {
                let owner = self.document.borrow();
                let tree = owner.GetDocument();
                let style = if tree.Node(node).IsHTMLElement("style") {
                    Some(node)
                } else {
                    tree.Node(node)
                        .Parent()
                        .filter(|&parent| tree.Node(parent).IsHTMLElement("style"))
                };
                style
                    .filter(|&index| owner.NeedsStyleSheetParsing(tree.Node(index).Id()))
                    .map(|index| {
                        fn AppendText(document: &Document, index: usize, result: &mut String) {
                            if document.Node(index).Type() == DOMNodeType::kText {
                                result.push_str(document.Node(index).Data());
                            }
                            for &child in document.Node(index).Children() {
                                AppendText(document, child, result);
                            }
                        }
                        let mut css = String::new();
                        AppendText(tree, index, &mut css);
                        (tree.Node(index).Id(), css)
                    })
            };
            if let Some((owner_node_id, css)) = style {
                let mut sheet = style::ParseCSS(&css);
                sheet.owner_node_id = owner_node_id;
                resources.AddParsedStyleSheet(sheet, &resources.BaseURL())?;
                style_sheet_changed = true;
            }
        }

        if scripts {
            let prepare = self.prepare_script.borrow().clone();
            if let Some(prepare) = prepare {
                let owner = self.document.borrow();
                let tree = owner.GetDocument();
                if tree.Node(node).IsHTMLElement("script") {
                    prepare(tree, node)?;
                }
            }
        }
        Ok(ConnectedResourceEffects {
            connected: true,
            style_sheet_changed,
        })
    }

    pub fn PrepareSubtree(
        &self,
        node: usize,
        scripts: bool,
        dispatch_image: &mut dyn FnMut(u64, bool),
    ) -> io::Result<ConnectedResourceEffects> {
        let mut effects = self.PrepareNode(node, scripts, true, dispatch_image)?;
        if !effects.connected {
            return Ok(effects);
        }
        let children = self
            .document
            .borrow()
            .GetDocument()
            .Node(node)
            .Children()
            .to_vec();
        for child in children {
            let child_effects = self.PrepareSubtree(child, scripts, dispatch_image)?;
            effects.style_sheet_changed |= child_effects.style_sheet_changed;
        }
        Ok(effects)
    }
}
