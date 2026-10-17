//! Owner-thread DOM utilities for the Page embedder.
use browser::page::Page;
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use page_mutation::PageMutation;
use std::io;
pub(crate) fn find_id(page: &Page, name: &str) -> Option<u64> {
    let owner = page.Document();
    let document = owner.GetDocument();
    fn visit(
        document: &dom::persistent_document::PersistentDocument,
        index: usize,
        name: &str,
    ) -> Option<u64> {
        let node = document.Node(index);
        if node
            .FindAttribute("id")
            .is_some_and(|attribute| attribute.value == name)
        {
            return Some(node.Id());
        }
        node.Children()
            .iter()
            .find_map(|&child| visit(document, child, name))
    }
    // InnerHTML leaves detached arena nodes alive; only the connected tree is
    // the current chrome. An old tab label may have the same element id.
    visit(document, document.Root(), name)
}
pub(crate) fn chrome_hover_chain(page: &Page, target: Option<u64>) -> Vec<u64> {
    let owner = page.Document();
    let doc = owner.GetDocument();
    let mut index = target.and_then(|id| doc.FindNodeById(id));
    let mut chain = Vec::new();
    while let Some(i) = index {
        let node = doc.Node(i);
        if let Some(id) = node.FindAttribute("id") {
            if id.value.starts_with("tab-")
                || id.value.starts_with("close-")
                || matches!(
                    id.value.as_str(),
                    "omnibox"
                        | "back"
                        | "forward"
                        | "reload"
                        | "site"
                        | "menu"
                        | "tabsearch"
                        | "newtab"
                )
            {
                chain.push(node.Id());
            }
        }
        index = node.Parent();
    }
    chain
}
pub(crate) fn set_hover_class(page: &mut Page, id: u64, hovered: bool) -> io::Result<()> {
    let value = {
        let owner = page.Document();
        let doc = owner.GetDocument();
        let Some(i) = doc.FindNodeById(id) else {
            return Ok(());
        };
        let mut classes = doc.Node(i).FindAttribute("class").map_or(Vec::new(), |a| {
            a.value
                .split_whitespace()
                .filter(|class| *class != "hovered")
                .map(str::to_owned)
                .collect::<Vec<_>>()
        });
        if hovered {
            classes.push("hovered".into());
        }
        classes.join(" ")
    };
    page.Apply(PageMutation::DOMMutation(DOMMutation {
        mutation_type: DOMMutationType::kSetAttribute,
        target_node_id: id,
        name: "class".into(),
        value,
        ..Default::default()
    }))
}
pub(crate) fn element_id(page: &Page, target: u64) -> Option<String> {
    let owner = page.Document();
    let document = owner.GetDocument();
    let mut index = document.FindNodeById(target);
    while let Some(i) = index {
        let node = document.Node(i);
        if let Some(id) = node.FindAttribute("id") {
            return Some(id.value.clone());
        }
        index = node.Parent();
    }
    None
}
