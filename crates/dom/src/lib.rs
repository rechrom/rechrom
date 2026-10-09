// Test linkage to the real replaced-object provider used by browser layout.
#[cfg(test)]
extern crate layoutng_block as _;
#[cfg(test)]
extern crate layoutng_replaced as _;

// Compile the canonical HTML implementation against this unit-test DOM,
// rather than importing a second non-test DOM through an html dev-dependency.
#[cfg(test)]
extern crate self as dom;
#[cfg(test)]
#[path = "../../html/src/lib.rs"]
pub(crate) mod test_html;
// The shared HTML sources use these original crate-root module paths.
#[cfg(test)]
pub(crate) use test_html::{html_parser_host, html_tag_names, parser, text_decoder};

pub mod user_interaction_state;
pub use user_interaction_state::{InteractionStateMutation, UserInteractionState};
pub mod dom_mutation;
pub mod error;
pub mod image_resource;
pub mod persistent_document;
pub mod resolved_styles;
pub mod style_state;
pub mod svg_path_parser;
pub use persistent_document::{PersistentDocument as Document, DOM};
pub use resolved_styles::ResolvedStyles;

use std::collections::HashMap;

// cpp: dom/document.h:158-163
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageResourceMetadata {
    pub id: u64,
    pub natural_width: f64,
    pub natural_height: f64,
    pub resolution_scale: f64,
}

impl Default for ImageResourceMetadata {
    fn default() -> Self {
        Self {
            id: 0,
            natural_width: 0.0,
            natural_height: 0.0,
            resolution_scale: 1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Element {
    pub tag: String,
    pub namespace: persistent_document::DOMNamespace,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub inline_style: Option<String>,
    pub attributes: Vec<(String, String)>,
    pub parent: Option<usize>,
    pub children: Vec<Child>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Child {
    Element(usize),
    Text(usize),
}

// Narrow static-page adapter retained for existing HTML/CSS/layout consumers.
// The source-level Document API is re-exported from persistent_document.
#[derive(Clone, Debug, Default)]
pub struct ParsedDocument {
    pub elements: Vec<Element>,
    pub texts: Vec<String>,
    pub root: Option<usize>,
    pub style_sources: Vec<StyleSource>,
    pub script_sources: Vec<ScriptSource>,
    pub base_href: Option<String>,
    image_resources: HashMap<String, ImageResourceMetadata>,
}

#[derive(Clone, Debug)]
pub enum StyleSource {
    Inline(String),
    Link(String),
}

// Static adapter record for the parser's script element, retaining the source
// order and attributes needed by the browser script scheduler.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptSource {
    pub source: String,
    pub src: Option<String>,
    pub script_type: String,
    pub async_attribute: bool,
    pub defer_attribute: bool,
    pub no_module: bool,
}

impl ParsedDocument {
    // cpp: dom/document.h:164-165
    // cpp: dom/document.cc:366-372
    #[allow(non_snake_case)]
    pub fn SetImageResource(&mut self, source: String, metadata: ImageResourceMetadata) {
        match self.image_resources.entry(source) {
            std::collections::hash_map::Entry::Vacant(slot) => {
                slot.insert(metadata);
            }
            std::collections::hash_map::Entry::Occupied(_) => {
                panic!("image source must be unique");
            }
        }
    }

    // cpp: dom/document.h:165-165
    // cpp: dom/document.cc:374-378
    #[allow(non_snake_case)]
    pub fn ImageResourceFor(&self, source: &str) -> Option<&ImageResourceMetadata> {
        self.image_resources.get(source)
    }

    pub fn append(
        &mut self,
        parent: Option<usize>,
        tag: String,
        id: Option<String>,
        classes: Vec<String>,
        inline_style: Option<String>,
    ) -> usize {
        self.append_with_attributes(parent, tag, id, classes, inline_style, Vec::new())
    }

    pub fn append_with_attributes(
        &mut self,
        parent: Option<usize>,
        tag: String,
        id: Option<String>,
        classes: Vec<String>,
        inline_style: Option<String>,
        attributes: Vec<(String, String)>,
    ) -> usize {
        self.append_with_namespace(
            parent,
            tag,
            persistent_document::DOMNamespace::kHTML,
            id,
            classes,
            inline_style,
            attributes,
        )
    }

    pub fn append_with_namespace(
        &mut self,
        parent: Option<usize>,
        tag: String,
        namespace: persistent_document::DOMNamespace,
        id: Option<String>,
        classes: Vec<String>,
        inline_style: Option<String>,
        attributes: Vec<(String, String)>,
    ) -> usize {
        let index = self.elements.len();
        self.elements.push(Element {
            tag,
            namespace,
            id,
            classes,
            inline_style,
            attributes,
            parent,
            children: Vec::new(),
        });
        if let Some(parent) = parent {
            self.elements[parent].children.push(Child::Element(index));
        } else if self.root.replace(index).is_some() {
            panic!("multiple HTML roots are unsupported");
        }
        index
    }

    pub fn append_text(&mut self, parent: usize, text: &str) {
        if text.is_empty() {
            return;
        }
        // cpp: html/html_parser.cc:1062-1080
        let index = self.texts.len();
        self.texts.push(text.to_owned());
        self.elements[parent].children.push(Child::Text(index));
    }
}

pub mod layout_mapping;
extern crate layoutng_assembly as layoutng;

mod lifecycle;
