//! HTMLSourceElement's typed dynamic class and parsed media owner.
//! Chromium 6c1d401f html_source_element.cc:61-85,157-162,179-186.
use super::*;
use style_parser::production_media_query::{CompileHTMLSourceMediaQuery, HTMLSourceMediaQuerySet};
#[derive(Clone, Default)]
pub(super) struct HTMLSourceElementData {
    media: Option<Rc<HTMLSourceMediaQuerySet>>,
    error_generation: Cell<u64>,
    error_task: Cell<Option<u64>>,
}
impl std::fmt::Debug for HTMLSourceElementData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HTMLSourceElement")
            .field("has_media_query_list", &self.media.is_some())
            .finish()
    }
}
impl PersistentDocument {
    pub fn IsHTMLSourceElement(&self, node: usize) -> bool {
        matches!(self.arena[node].class, DOMNodeClass::HTMLSourceElement(_))
    }
    pub fn HTMLSourceMediaQuery(&self, node: usize) -> Option<&Rc<HTMLSourceMediaQuerySet>> {
        match &self.arena[node].class {
            DOMNodeClass::HTMLSourceElement(data) => data.media.as_ref(),
            _ => None,
        }
    }
    pub(super) fn InitializeHTMLSourceElement(&mut self, node: usize) {
        if !self.IsHTMLSourceElement(node) {
            return;
        }
        let media = self.arena[node]
            .FindAttribute("media")
            .map_or("", |attr| attr.value.as_str());
        let query = CompileHTMLSourceMediaQuery(media);
        if let DOMNodeClass::HTMLSourceElement(data) = &mut self.arena[node].class {
            data.media = query;
        }
    }
    pub(super) fn HTMLSourceAttributeChanged(&mut self, node: usize, name: &str, namespace: &str) {
        if namespace.is_empty() && name == "media" {
            self.InitializeHTMLSourceElement(node);
        }
    }
}

impl PersistentDocument {
    pub fn SourceCancelPendingErrorEvent(&self, node: usize) -> Option<u64> {
        let DOMNodeClass::HTMLSourceElement(data) = &self.arena[node].class else {
            unreachable!("native source class")
        };
        data.error_generation.set(
            data.error_generation
                .get()
                .checked_add(1)
                .expect("source error generation"),
        );
        data.error_task.take()
    }
    pub fn SourceErrorGeneration(&self, node: usize) -> u64 {
        let DOMNodeClass::HTMLSourceElement(data) = &self.arena[node].class else {
            unreachable!()
        };
        data.error_generation.get()
    }
    pub fn SourceSetErrorTask(&self, node: usize, task: Option<u64>) {
        let DOMNodeClass::HTMLSourceElement(data) = &self.arena[node].class else {
            unreachable!()
        };
        data.error_task.set(task);
    }
}
