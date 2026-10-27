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
pub mod custom_elements;
pub mod dom_mutation;
pub mod error;
pub mod event_path;
pub use event_path::{DOMEventPath, DOMEventPathEntry};
pub mod image_resource;
pub mod mutation_observers;
pub mod node_references;
pub mod persistent_document;
pub mod style_state;
pub mod svg_path_parser;
pub use persistent_document::{DOMAutofillState, PersistentDocument as Document, DOM};

// cpp: dom/document.h:158-163
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageResourceMetadata {
    pub id: u64,
    pub natural_width: f64,
    pub natural_height: f64,
    pub resolution_scale: f64,
    pub orientation: ::image_resource::ImageOrientation,
}

impl Default for ImageResourceMetadata {
    fn default() -> Self {
        Self {
            id: 0,
            natural_width: 0.0,
            natural_height: 0.0,
            resolution_scale: 1.0,
            orientation: Default::default(),
        }
    }
}

impl ImageResourceMetadata {
    pub fn NaturalSize(&self, respect_orientation: bool) -> (f64, f64) {
        if respect_orientation {
            self.orientation
                .oriented_size(self.natural_width, self.natural_height)
        } else {
            (self.natural_width, self.natural_height)
        }
    }

    /// CSS intrinsic size for one HTML responsive-image use. Candidate
    /// density is element-owned; resource resolution remains the fallback for
    /// ordinary `src` and CSS image consumers.
    pub fn NaturalSizeForCandidate(
        &self,
        respect_orientation: bool,
        candidate_density: Option<f64>,
    ) -> (f64, f64) {
        let size = self.NaturalSize(respect_orientation);
        let density = candidate_density
            .filter(|density| density.is_finite() && *density > 0.0)
            .unwrap_or(self.resolution_scale);
        let scale = self.resolution_scale / density;
        (size.0 * scale, size.1 * scale)
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

pub mod layout_mapping;
extern crate layoutng_assembly as layoutng;

mod lifecycle;
