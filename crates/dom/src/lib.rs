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
pub mod use_counter;
pub use use_counter::WebFeature;
pub mod attribute_lifecycle;
pub mod content_security_policy;
pub mod custom_elements;
pub mod document_base_url;
pub mod document_lifecycle;
pub mod document_metadata;
pub mod document_policy;
pub mod dom_mutation;
pub mod element_internals;
pub mod error;
pub mod event_path;
pub mod lazy_load_frame_observer;
pub use persistent_document::{DOMFormControlClass, DOMSelectCollection};
pub mod tree_mutation;
pub use event_path::{DOMEventPath, DOMEventPathEntry};
pub mod image_resource;
pub mod mutation_observers;
pub mod native_execution_context_lifetime;
pub mod node_references;
pub mod node_removal_observer;
pub mod persistent_document;
pub mod popup_state;
pub mod range;
pub use range::{DOMBoundaryPoint, DOMRangeError, DOMRangeHandle, DOMStaticRange};
pub mod style_state;
pub mod svg_path_parser;
pub use persistent_document::{
    DOMAutofillState, DOMInputType, DOMNamespace, DOMNodeType, PersistentDocument,
    PersistentDocument as Document, StyleDocument, DOM,
};

pub use ::image_resource::ImageResourceMetadata;

/// Immutable current-frame metadata retained by HTMLVideoElement. Pixel
/// storage remains in ConstraintSpace's PaintImage catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VideoFrameMetadata {
    pub resource_id: ::image_resource::ImageId,
    pub revision: u64,
    pub width: u32,
    pub height: u32,
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

pub mod public_url_manager;

pub mod network_permissions_policy;
pub mod permissions_policy_features;

pub mod frame_ad_evidence;

pub mod document_parser_lifecycle;

pub mod load_event_delay_count;

pub mod navigation_tokens;

pub mod find_buffer;

pub mod render_blocking_resource_manager;

pub mod render_blocking_elements;

pub mod font_loading;

pub mod opaque_range;
pub use opaque_range::DOMOpaqueRangeHandle;

pub mod range_geometry;

pub mod font_preload;

mod history_url;

pub use persistent_document::content_frames;
pub mod frame_lifecycle;
pub mod frame_owner_loading;

pub mod video_timing;

pub mod form_controller;

pub mod atomic_move;
pub mod input_value;
