#![allow(non_snake_case, non_camel_case_types)]

use dom::persistent_document::DOMNode;

// cpp: html/html_parser_host.h:11-14
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ParserElementPhase {
    #[default]
    kInserted,
    kChildrenFinished,
}

// cpp: html/html_parser_host.h:16-19
pub struct ParserElementEvent<'a> {
    pub element: &'a DOMNode,
    pub phase: ParserElementPhase,
}

impl<'a> ParserElementEvent<'a> {
    pub fn new(element: &'a DOMNode) -> Self {
        Self {
            element,
            phase: ParserElementPhase::kInserted,
        }
    }
}

// cpp: html/html_parser_host.h:23-27
// Parser-host notification is observational. Hosts inspect the inserted node
// to schedule resources; DOM mutation remains on Document's explicit APIs so
// a read cannot accidentally trigger style/layout invalidation.
pub trait HTMLParserHost {
    fn HandleParserElement(&mut self, event: ParserElementEvent<'_>);

    // Rust arena adapter: source DOMNode children are pointers, while Rust
    // children are indices. Hosts inspect the node synchronously at this event.
    fn HandleParserElementInDocument(
        &mut self,
        document: &mut dom::Document,
        element: usize,
        phase: ParserElementPhase,
    ) {
        self.HandleParserElement(ParserElementEvent {
            element: document.Node(element),
            phase,
        });
    }
}
