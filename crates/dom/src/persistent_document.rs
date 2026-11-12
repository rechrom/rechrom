#![allow(non_snake_case)]

use crate::error::{invalid_argument, logic_error};
use crate::{ImageResourceMetadata, VideoFrameMetadata};
use cssom::{CSSDeclaration, CSSOMAdoptionError, CSSOMEvent, CSSOMMutation, CSSStyleSheet, CSSOM};
use font_engine::text::implementation::bidi_paragraph::BidiParagraph;
use foundation::Persistent;
use foundation::{StringView, TextDirection};
use layoutng_assembly::internal::layout_input::{Offset, TrackedElementSubRect};
use layoutng_style::style::computed_style::ComputedStyle as StyleComputedStyle;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

#[path = "svg_document_extensions.rs"]
mod svg_document_extensions;
#[path = "svg_use_instances.rs"]
mod svg_use_instances;
pub use svg_use_instances::SVGUseInstanceStatus;

#[path = "date_time_custom_style.rs"]
mod date_time_custom_style;
pub use date_time_custom_style::{
    DateTimeCustomStyleError, DateTimeCustomStyleTextMetrics, DateTimeEditCustomStyle,
    DateTimeStyleChild,
};

#[path = "user_action_elements.rs"]
mod user_action_elements;
pub use user_action_elements::FocusedElementChangeObserver;

#[path = "image_resource_content.rs"]
mod image_resource_content;
pub use image_resource_content::{DOMDecodedImage, DOMImageResourceContent, ImageContentStatus};
#[path = "html_image_loader.rs"]
mod html_image_loader;
pub use html_image_loader::{
    HTMLImageLoader, HTMLImageLoaderEventTicket, HTMLImageLoaderTask, ImageDecodeResult,
    ImageDecodeState, ImageLoaderEvent, ImageLoaderInputContent, ImageLoaderTaskKind,
    ImageLoaderUpdate,
};
#[path = "hit_test_element.rs"]
mod hit_test_element;
pub use hit_test_element::DOMHitTestElementKind;
#[path = "content_frames.rs"]
pub mod content_frames;
#[path = "editing_serialization_mutation.rs"]
mod editing_serialization_mutation;
#[path = "html_inner_text.rs"]
mod html_inner_text;
#[path = "html_source_element.rs"]
mod html_source_element;
#[path = "html_track_element.rs"]
mod html_track_element;
#[path = "media_layout_children.rs"]
mod media_layout_children;
pub use media_layout_children::{
    MediaChildKind, MediaControlDivClass, MediaControlInputClass, MediaControlsEventState,
};
#[path = "media_element_state.rs"]
mod media_element_state;
pub use media_element_state::{
    DOMMediaPlayerLayoutFacts, DOMMediaPlayerLayoutSource, DOMMediaSourceAttachment,
    DOMMediaSourceHandle, DOMMediaSrcObject, DOMMediaStreamDescriptor, DOMPictureInPictureState,
    DOMVideoVisibilityTracker, HTMLMediaElementState, MediaDownloadFactsForURL, VideoPosterUpdate,
};
#[path = "canvas_element_state.rs"]
mod canvas_element_state;
#[path = "canvas_subtree.rs"]
mod canvas_subtree;
pub use canvas_element_state::DOMCanvasDrawElementGeometry;
#[path = "radio_button_group_scope.rs"]
mod radio_button_group_scope;
pub use radio_button_group_scope::DOMRadioGroupScope;
#[path = "view_transition_participation.rs"]
mod view_transition_participation;
pub use view_transition_participation::{
    ViewTransition, ViewTransitionCSSDiscoveryError, ViewTransitionCSSLayerFacts,
    ViewTransitionCSSPaintOrder, ViewTransitionParticipationError, ViewTransitionRandomToken,
    ViewTransitionSupplement, ViewTransitionTerminalState, ViewTransitionTrackerState,
};

#[path = "table_row_cells.rs"]
mod table_row_cells;

#[path = "focusgroup.rs"]
mod focusgroup;
use focusgroup::ElementFocusgroupData;

#[path = "security_context.rs"]
mod security_context;
pub use security_context::{
    ExecutionContextIsolation, SecurityContext, SecurityOrigin, SecurityOriginSchemeRegistry,
    SecurityOriginTuple,
};
#[path = "descendant_cursor.rs"]
mod descendant_cursor;
#[path = "document_versions.rs"]
mod document_versions;
pub use descendant_cursor::DOMInclusiveDescendantCursor;
#[path = "editing_mutation_facts.rs"]
mod editing_mutation_facts;
#[path = "layout_tree_detachment.rs"]
mod layout_tree_detachment;
pub use layout_tree_detachment::{DocumentLayoutTreeDetachment, PendingSVGResourceInvalidation};
#[path = "editing_node.rs"]
mod editing_node;
#[path = "pseudo_element_node.rs"]
mod pseudo_element_node;
#[path = "scroll_marker_group_data.rs"]
mod scroll_marker_group_data;
pub use editing_mutation_facts::DocumentEditingMutation;
pub use scroll_marker_group_data::{DOMScrollMarkerGroupState, DOMScrollMarkerInvalidation};
#[path = "spellcheck_node.rs"]
mod spellcheck_node;
pub use pseudo_element_node::DOMPseudoElementFacts;
#[path = "text_iterator_node.rs"]
mod text_iterator_node;
pub use text_iterator_node::InputAltTextLocale;

#[path = "element_attribute_reference.rs"]
mod element_attribute_reference;
#[path = "html_data_list_owner.rs"]
mod html_data_list_owner;
#[path = "html_image_map.rs"]
mod html_image_map;
#[path = "html_select_owner.rs"]
mod html_select_owner;
pub use html_image_map::{DOMImageMapAreaShape, DOMImageMapContainer};
#[path = "html_select_shadow.rs"]
mod html_select_shadow;
pub use html_select_owner::{DOMSelectSourceFeatures, DOMSelectUAPartKind};
#[path = "html_details_owner.rs"]
mod html_details_owner;
#[path = "popup_top_layer.rs"]
mod popup_top_layer;
use editing_node::DOMNodeClass;
pub use element_attribute_reference::{ContextMenuAutofillDOMFacts, HTMLElementCommandClass};
#[path = "display_lock.rs"]
mod display_lock;
pub use display_lock::{
    DisplayLockActivationReason, DisplayLockForcedEffect, DisplayLockForcedPhase,
    DisplayLockForcedScopeHandle, DisplayLockLifecycleInputs, DisplayLockRenderAffectingState,
};
use display_lock::{DisplayLockContext, DisplayLockDocumentState};

#[path = "input_image_fallback.rs"]
mod input_image_fallback;

#[path = "input_type_shadow.rs"]
mod input_type_shadow;

#[path = "overscroll_area_tracker.rs"]
mod overscroll_area_tracker;
pub use overscroll_area_tracker::{
    DOMOverscrollAction, DOMOverscrollGeometry, DOMOverscrollOffset, DOMOverscrollOpenBox,
    DOMOverscrollRect, DOMOverscrollSnapData, DOMOverscrollToggleTargets,
};
use overscroll_area_tracker::{ElementOverscrollData, OverscrollDocumentData};

#[path = "form_control_state.rs"]
mod form_control_state;
pub use form_control_state::{DOMFormControlClass, DOMSelectCollection};

#[path = "focus_navigation.rs"]
mod focus_navigation;
#[path = "fragment_anchor.rs"]
mod fragment_anchor;
pub use fragment_anchor::AutoExpandableAncestorKind;

#[path = "editable_facts.rs"]
mod editable_facts;
pub use editable_facts::DOMEditableFacts;

#[path = "node_child_removal_tracker.rs"]
mod node_child_removal_tracker;
pub use node_child_removal_tracker::NodeChildRemovalTracker;
use user_action_elements::UserActionElements;
pub use user_action_elements::UserActionStateUpdate;

static NEXT_ARENA_ID: AtomicU64 = AtomicU64::new(1);

// A C++ DOMNode* can name a document node in a different Document arena.
// Arena indices alone cannot carry that identity across Rust documents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DOMOwnerHandle {
    arena_id: u64,
    node_id: u64,
}

impl DOMOwnerHandle {
    // Identity adapter for source uses of OwnerDocumentNode()->Id().
    // The arena component remains part of handle equality and adoption.
    pub fn Id(self) -> u64 {
        self.node_id
    }
    /// Opaque identity for source pointer-equality checks across documents.
    /// Node Id alone can be equal in distinct canonical arenas.
    pub fn Identity(self) -> u128 {
        ((self.arena_id as u128) << 64) | self.node_id as u128
    }
}

// cpp: dom/document.h:19-27
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DOMNodeType {
    kDocument,
    kDocumentType,
    kElement,
    kText,
    kComment,
    kProcessingInstruction,
    kDocumentFragment,
}

// cpp: document.h IsHTMLDocument. Document kind is independent of element
// namespace: an XHTML element in an XML document is still not in an HTML
// document for selector matching.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DOMDocumentType {
    kHTML,
    kXML,
    kSVG,
}

// cpp: dom/document.h:29-29
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DOMNamespace {
    kNone,
    kHTML,
    kSVG,
    kMathML,
}

// cpp: third_party/blink/renderer/core/html/forms/input_type.h:57-92.
// HTMLInputElement owns the normalized type, while callers consume this typed
// state instead of interpreting the content attribute independently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DOMRangeShadowPart {
    Thumb,
    Container,
    Track,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DOMInputType {
    kButton,
    kColor,
    kFile,
    kHidden,
    kImage,
    kNumber,
    kRange,
    kReset,
    kSubmit,
    kRadio,
    kCheckbox,
    kDate,
    kDateTimeLocal,
    kMonth,
    kTime,
    kWeek,
    kEmail,
    kPassword,
    kSearch,
    kTelephone,
    kURL,
    kText,
}

// cpp: third_party/blink/public/web/web_autofill_state.h:10.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DOMAutofillState {
    #[default]
    kNotFilled,
    kPreviewed,
    kAutofilled,
}

impl DOMInputType {
    fn FromTypeAttribute(value: Option<&str>) -> Self {
        match value.unwrap_or("").to_ascii_lowercase().as_str() {
            "button" => Self::kButton,
            "color" => Self::kColor,
            "file" => Self::kFile,
            "hidden" => Self::kHidden,
            "image" => Self::kImage,
            "number" => Self::kNumber,
            "range" => Self::kRange,
            "reset" => Self::kReset,
            "submit" => Self::kSubmit,
            "radio" => Self::kRadio,
            "checkbox" => Self::kCheckbox,
            "date" => Self::kDate,
            "datetime-local" => Self::kDateTimeLocal,
            "month" => Self::kMonth,
            "time" => Self::kTime,
            "week" => Self::kWeek,
            "email" => Self::kEmail,
            "password" => Self::kPassword,
            "search" => Self::kSearch,
            "tel" => Self::kTelephone,
            "url" => Self::kURL,
            // The missing-value and invalid-value defaults are both Text.
            _ => Self::kText,
        }
    }

    // cpp: input_type.h:167-169.
    pub fn IsTextFieldInputType(self) -> bool {
        matches!(
            self,
            Self::kEmail
                | Self::kPassword
                | Self::kSearch
                | Self::kTelephone
                | Self::kURL
                | Self::kText
                | Self::kNumber
        )
    }

    // cpp: input_type.cc:368-371; color_input_type.cc:105-107;
    //      range_input_type.cc:145-147; submit_input_type.cc:61-63.
    pub fn SupportsRequired(self) -> bool {
        !matches!(
            self,
            Self::kButton
                | Self::kColor
                | Self::kHidden
                | Self::kImage
                | Self::kRange
                | Self::kReset
                | Self::kSubmit
        )
    }

    // cpp: base_temporal_input_type.cc:197-199;
    //      text_field_input_type.cc:612-614.
    pub fn SupportsReadOnly(self) -> bool {
        self.IsTextFieldInputType()
            || matches!(
                self,
                Self::kDate | Self::kDateTimeLocal | Self::kMonth | Self::kTime | Self::kWeek
            )
    }

    // cpp: base_text_input_type.cc:142-144; number_input_type.cc:408-410.
    pub fn SupportsPlaceholder(self) -> bool {
        self.IsTextFieldInputType()
    }

    // cpp: input_type.cc:920-922; base_temporal_input_type.cc:201-203;
    //      color_input_type.cc:305-307; range_input_type.cc:396-398;
    //      text_field_input_type.cc:714-716. PasswordInputType overrides the
    //      TextFieldInputType default because exposing saved suggestions would
    //      disclose password values.
    pub fn ShouldRespectListAttribute(self) -> bool {
        (self.IsTextFieldInputType() && self != Self::kPassword)
            || matches!(
                self,
                Self::kColor
                    | Self::kRange
                    | Self::kDate
                    | Self::kDateTimeLocal
                    | Self::kMonth
                    | Self::kTime
                    | Self::kWeek
            )
    }
}

// cpp: third_party/blink/renderer/core/dom/shadow_root.h:45-51.
// The mode is retained by the DOM owner; style only asks whether a root is a
// user-agent root and never reconstructs that fact from attributes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DOMShadowRootMode {
    kOpen,
    kClosed,
    kUserAgent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DOMSlotAssignmentMode {
    kNamed,
    kManual,
}

// cpp: dom/document.h:31-36
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DOMAttribute {
    pub prefix: String,
    pub local_name: String,
    pub namespace_uri: String,
    pub value: String,
}

/// Canonical WTF-16 storage borrowed while the Document read guard is held.
/// DOMAttribute's UTF-8 fields remain the legacy scalar projection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DOMQualifiedNameUTF16 {
    pub prefix: Rc<[u16]>,
    pub local_name: Rc<[u16]>,
    pub namespace_uri: Rc<[u16]>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DOMAttributeUTF16 {
    pub name: DOMQualifiedNameUTF16,
    pub value: Rc<[u16]>,
}
impl DOMAttributeUTF16 {
    fn FromAttribute(attribute: &DOMAttribute) -> Self {
        Self {
            name: DOMQualifiedNameUTF16 {
                prefix: attribute.prefix.encode_utf16().collect::<Vec<_>>().into(),
                local_name: attribute
                    .local_name
                    .encode_utf16()
                    .collect::<Vec<_>>()
                    .into(),
                namespace_uri: attribute
                    .namespace_uri
                    .encode_utf16()
                    .collect::<Vec<_>>()
                    .into(),
            },
            value: attribute.value.encode_utf16().collect::<Vec<_>>().into(),
        }
    }
}

// cpp: svg/svg_view_spec.h; svg/graphics/svg_image.h SVGImageViewInfo.
//
// SVG images share one inert Document across all of their container variants.
// The active view is therefore document state, rather than an authored root
// attribute.  Keeping it separate also prevents svgView() from overwriting an
// author's viewBox, preserveAspectRatio or transform while another variant is
// recorded.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SVGImagePreserveAspectRatio {
    pub preserve_none: bool,
    pub slice: bool,
    pub align_x: i8,
    pub align_y: i8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SVGImageViewSpec {
    pub zoom_and_pan: Option<bool>,
    pub view_box: Option<[f64; 4]>,
    pub preserve_aspect_ratio: Option<SVGImagePreserveAspectRatio>,
    /// SVG's 2D affine order: a, b, c, d, e, f.
    pub transform: Option<[f64; 6]>,
}

// cpp: third_party/blink/renderer/core/dom/names_map.h:24-55;
//      third_party/blink/renderer/core/dom/names_map.cc:17-242.
//
// Retained parsed form of an Element's `exportparts` attribute. The map is
// owned by PersistentDocument for the lifetime of the element, mirroring the
// NamesMap field in Blink's NodeRareData. Values preserve SpaceSplitString's
// insertion order and uniqueness.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DOMPartNamesMap {
    data: BTreeMap<String, Vec<String>>,
}

impl DOMPartNamesMap {
    fn Add(&mut self, key: String, value: String) {
        let values = self.data.entry(key).or_default();
        if !values.contains(&value) {
            values.push(value);
        }
    }

    fn Parse(source: &str) -> Self {
        #[derive(Clone, Copy)]
        enum State {
            PreKey,
            Key,
            PostKey,
            PreValue,
            Value,
            PostValue,
            Error,
        }

        fn is_html_space(character: char) -> bool {
            matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' ')
        }

        let characters = source.char_indices().collect::<Vec<_>>();
        let mut result = Self::default();
        let mut state = State::PreKey;
        let mut start = 0usize;
        let mut key = String::new();
        let mut value = String::new();

        for (byte, current) in characters.iter().copied() {
            match state {
                State::PreKey => {
                    if is_html_space(current) || current == ',' {
                        continue;
                    }
                    if current == ':' {
                        state = State::Error;
                        continue;
                    }
                    start = byte;
                    state = State::Key;
                }
                State::Key => {
                    if current == ',' {
                        key = source[start..byte].to_owned();
                        result.Add(key.clone(), key.clone());
                        state = State::PreKey;
                    } else if current == ':' {
                        key = source[start..byte].to_owned();
                        state = State::PreValue;
                    } else if is_html_space(current) {
                        key = source[start..byte].to_owned();
                        state = State::PostKey;
                    }
                }
                State::PostKey => {
                    if current == ',' {
                        result.Add(key.clone(), key.clone());
                        state = State::PreKey;
                    } else if current == ':' {
                        state = State::PreValue;
                    } else if !is_html_space(current) {
                        key.clear();
                        state = State::Error;
                    }
                }
                State::PreValue => {
                    if current == ':' {
                        state = State::Error;
                    } else if current == ',' {
                        state = State::PreKey;
                    } else if !is_html_space(current) {
                        start = byte;
                        state = State::Value;
                    }
                }
                State::Value => {
                    if current == ',' {
                        value = source[start..byte].to_owned();
                        result.Add(key.clone(), value.clone());
                        state = State::PreKey;
                    } else if is_html_space(current) || current == ':' {
                        value = source[start..byte].to_owned();
                        state = State::PostValue;
                    }
                }
                State::PostValue => {
                    if current == ',' {
                        result.Add(key.clone(), value.clone());
                        state = State::PreKey;
                    } else if !is_html_space(current) {
                        state = State::Error;
                    }
                }
                State::Error => {
                    if current == ',' {
                        state = State::PreKey;
                    }
                }
            }
        }

        match state {
            State::Key => {
                key = source[start..].to_owned();
                result.Add(key.clone(), key);
            }
            State::PostKey => result.Add(key.clone(), key),
            State::Value => {
                value = source[start..].to_owned();
                result.Add(key, value);
            }
            State::PostValue => result.Add(key, value),
            State::PreKey | State::PreValue | State::Error => {}
        }
        result
    }

    pub fn Get(&self, name: &str) -> Option<&[String]> {
        self.data.get(name).map(Vec::as_slice)
    }

    pub fn Len(&self) -> usize {
        self.data.len()
    }

    pub fn IsEmpty(&self) -> bool {
        self.data.is_empty()
    }
}

// cpp: dom/document.h:43-77
// Stable arena indices stand in for stable C++ node addresses. No Rust reference
// to an arena element is retained across allocation or reparenting.
#[derive(Clone, Default)]
struct ElementStyleSlot {
    style: Option<std::sync::Arc<ResolvedNodeStyle>>,
    resources: ResolvedStyleResources,
    // cpp: NodeRareData's scrollbar_pseudo_element_styles_depend_on_font_metrics.
    scrollbar_pseudo_element_styles_depend_on_font_metrics: bool,
}

impl std::fmt::Debug for ElementStyleSlot {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ElementStyleSlot")
            .field("has_style", &self.style.is_some())
            .finish()
    }
}

#[derive(Clone)]
pub struct DOMNode {
    id: u64,
    node_type: DOMNodeType,
    class: DOMNodeClass,
    svg_element_kind: Option<svg_element_kind::SVGElementKind>,
    resize_observer: RefCell<Option<Rc<resize_observer_data::ElementResizeObserverState>>>,
    intersection_observer:
        RefCell<Option<Rc<intersection_observer_data::ElementIntersectionObserverState>>>,
    // HTMLInputElement/HTMLOptionElement nearest select/select-child members.
    // Used only by the FilterableSelect source lifecycle, never a tree mirror.
    select_related_parts: Option<html_select_shadow::SelectRelatedParts>,
    pub(crate) content_frame: Option<Rc<dyn crate::content_frames::ContentFrameSource>>,
    pub(crate) frame_element_loading:
        Option<Rc<crate::frame_owner_loading::FrameElementLoadingState>>,
    pub(crate) lazy_frame_observer:
        Option<Rc<dyn crate::content_frames::LazyLoadFrameObserverSource>>,
    pub(crate) connected_subframe_count: u32,
    // Node::SetHasCustomStyleCallbacks (node.h:1356-1358).
    has_custom_style_callbacks: bool,
    is_in_canvas_subtree: bool,
    // HTMLOptionElement/HTMLOptGroupElement::owner_select_; updated before
    // post-insertion callbacks by the existing ContainerNode lifecycle walk.
    select_owner: Option<usize>,
    edit_context: Option<Rc<dyn foundation::focus::ElementEditContextFocus>>,
    display_lock_context: RefCell<Option<DisplayLockContext>>,
    scroll_marker_group_data: Option<Box<scroll_marker_group_data::ScrollMarkerGroupData>>,
    focusgroup: Option<Box<ElementFocusgroupData>>,
    overscroll: Option<Rc<RefCell<ElementOverscrollData>>>,
    // Element/NodeRareData undo-root membership, never a selection snapshot.
    has_undo_stack: std::cell::Cell<bool>,
    namespace: DOMNamespace,
    tag_qname: foundation::QualifiedName,
    tag_qname_utf16: DOMQualifiedNameUTF16,
    name_utf16: Rc<[u16]>,
    attribute_utf16: Vec<DOMAttributeUTF16>,
    // ElementRareData::IsValue, set at creation and preserved by clone;
    // subsequent mutations of the is attribute do not alter this owner.
    is_value: Option<Rc<[u16]>>,
    serialization_attribute_values:
        RefCell<Option<serialization_attributes::SerializationAttributeValues>>,
    name: String,
    data: String,
    attributes: Vec<DOMAttribute>,
    image_loader: RefCell<Option<Rc<HTMLImageLoader>>>,
    html_image_layout_disposition: Cell<html_image_element::HTMLImageLayoutDisposition>,
    html_image_is_lcp_element: Cell<bool>,
    inline_style: RefCell<Option<cssom::CSSInlineStyle>>,
    // SVGAnimatedLength owns the compiled CSS value, retained across unrelated
    // style/layout updates and replaced only by its authored attribute change.
    svg_marker_lengths:
        RefCell<Option<std::sync::Arc<style_parser::svg_length_parser::SVGMarkerLengthInputs>>>,
    // Chromium stores the canonical ComputedStyle on Element. Keep the same
    // element-local lifetime and O(1) arena lookup instead of a document-wide
    // NodeId hash table. The Arc roots the immutable boundary record while
    // Layout retains the canonical traced ComputedStyle owner inside it.
    resolved_style: ElementStyleSlot,
    /// Source: Element/NodeRareData RegionCaptureCropId. This is a one-shot,
    /// non-null token installed by the CropTarget API.
    region_capture_crop_id: Option<[u64; 2]>,
    /// Source: Element/NodeRareData TrackedElementSubRects. Chromium stores a
    /// flat_map keyed by feature; this vector is kept sorted by that key.
    tracked_element_sub_rects: Vec<TrackedElementSubRect>,
    owner_document_handle: Option<DOMOwnerHandle>,
    // Node::tree_scope_ is retained independently of parentNode. Removing a
    // node does not move it out of its Document/ShadowRoot TreeScope; only
    // adoption into another scope changes this owner.
    tree_scope_root: usize,
    parent: Option<usize>,
    previous_sibling: Option<usize>,
    next_sibling: Option<usize>,
    children: Vec<usize>,
    selector_attribute_revision: u64,
    selector_identifier_revision: u64,
    selector_subtree_revision: u64,
}

impl std::fmt::Debug for DOMNode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DOMNode")
            .field("id", &self.id)
            .field("node_type", &self.node_type)
            .field("class", &self.class)
            .field("namespace", &self.namespace)
            .field("name", &self.name)
            .field("has_edit_context", &self.edit_context.is_some())
            .finish_non_exhaustive()
    }
}

impl DOMNode {
    // cpp: dom/document.h:45-53
    pub fn IsStyleElement(&self) -> bool {
        matches!(self.class, DOMNodeClass::StyleElement)
    }
    pub fn IsCDATASection(&self) -> bool {
        matches!(self.class, DOMNodeClass::CDATASection)
    }
    pub fn Type(&self) -> DOMNodeType {
        self.node_type
    }
    pub fn Namespace(&self) -> DOMNamespace {
        self.namespace
    }
    pub fn Id(&self) -> u64 {
        self.id
    }
    // Source element.cc:9270-9278.
    pub fn HasUndoStack(&self) -> bool {
        self.has_undo_stack.get()
    }
    pub fn SetHasUndoStack(&self, value: bool) {
        debug_assert!(self.node_type == DOMNodeType::kElement);
        self.has_undo_stack.set(value);
    }
    pub fn TagQName(&self) -> &foundation::QualifiedName {
        &self.tag_qname
    }
    pub fn NameUTF16(&self) -> &[u16] {
        &self.name_utf16
    }
    pub fn QualifiedNameUTF16(&self) -> &DOMQualifiedNameUTF16 {
        &self.tag_qname_utf16
    }
    pub fn AttributesUTF16(&self) -> &[DOMAttributeUTF16] {
        &self.attribute_utf16
    }
    pub fn AttributeValueUTF16(&self, name: &str) -> Option<Rc<[u16]>> {
        let index = self
            .attributes
            .iter()
            .position(|a| a.namespace_uri.is_empty() && a.local_name == name)?;
        Some(self.attribute_utf16[index].value.clone())
    }
    pub fn Name(&self) -> &str {
        &self.name
    }
    pub fn Data(&self) -> &str {
        &self.data
    }
    pub fn Attributes(&self) -> &[DOMAttribute] {
        &self.attributes
    }
    pub fn RegionCaptureCropId(&self) -> Option<[u64; 2]> {
        self.region_capture_crop_id
    }
    pub fn TrackedElementSubRects(&self) -> &[TrackedElementSubRect] {
        &self.tracked_element_sub_rects
    }

    pub(crate) fn SVGLengthContainerSizes(
        &self,
    ) -> Option<layoutng_style::css::css_to_length_conversion_data::ContainerSizes> {
        self.resolved_style.resources.svg_length_container_sizes
    }
    pub fn SVGMarkerLengthInputs(
        &self,
    ) -> std::sync::Arc<style_parser::svg_length_parser::SVGMarkerLengthInputs> {
        self.svg_marker_lengths
            .borrow_mut()
            .get_or_insert_with(|| {
                std::sync::Arc::new(
                    style_parser::svg_length_parser::SVGMarkerLengthInputs::Parse(|name| {
                        self.FindAttribute(name)
                            .map(|attribute| attribute.value.clone())
                    }),
                )
            })
            .clone()
    }
    pub fn SVGMarkerLengthValues(
        &self,
    ) -> std::sync::Arc<layoutng_style::css::svg_length_value::SVGMarkerLengthValues> {
        self.resolved_style
            .resources
            .svg_marker_resolved_lengths
            .clone()
            .unwrap_or_else(|| self.SVGMarkerLengthInputs().Fallback())
    }

    /// ElementData retains parsed inline declarations. Initializing their
    /// backing is derived state; it does not create or modify an attribute.
    pub fn InlineStyle(&self) -> cssom::CSSInlineStyle {
        let source = self
            .AttributeValueUTF16("style")
            .unwrap_or_else(|| Rc::from([]));
        let mut retained = self.inline_style.borrow_mut();
        if let Some(style) = retained
            .as_ref()
            .filter(|style| style.SourceTextUTF16() == source.as_ref())
        {
            return style.clone();
        }
        let style = cssom::CSSInlineStyle::FromUTF16(&source);
        *retained = Some(style.clone());
        style
    }

    pub(crate) fn RetainInlineStyle(&self, style: cssom::CSSInlineStyle) {
        let source = self
            .AttributeValueUTF16("style")
            .unwrap_or_else(|| Rc::from([]));
        if style.SourceTextUTF16() == source.as_ref() {
            *self.inline_style.borrow_mut() = Some(style);
        }
    }
    pub fn OwnerDocumentNode(&self) -> Option<DOMOwnerHandle> {
        self.owner_document_handle
    }
    pub fn Parent(&self) -> Option<usize> {
        self.parent
    }
    pub fn PreviousSibling(&self) -> Option<usize> {
        self.previous_sibling
    }
    pub fn NextSibling(&self) -> Option<usize> {
        self.next_sibling
    }
    pub fn Children(&self) -> &[usize] {
        &self.children
    }

    // cpp: dom/document.cc:24-28
    pub fn FindAttribute(&self, local_name: &str) -> Option<&DOMAttribute> {
        self.attributes
            .iter()
            .find(|attribute| attribute.local_name == local_name)
    }

    // cpp: dom/document.cc:30-34
    pub fn IsElement(&self, local_name: &str, node_namespace: DOMNamespace) -> bool {
        self.node_type == DOMNodeType::kElement
            && self.namespace == node_namespace
            && self.name == local_name
    }
    // cpp: dom/document.h:56-57
    pub fn IsHTMLElement(&self, local_name: &str) -> bool {
        self.IsElement(local_name, DOMNamespace::kHTML)
    }
}

// cpp: dom/document.h:169-196
#[derive(Clone)]
pub enum PseudoContent {
    Text(String),
    Image {
        resource_id: u64,
        shader: Option<std::sync::Arc<layoutng_assembly::internal::paint_input::PaintShader>>,
        natural_width: Option<f64>,
        natural_height: Option<f64>,
        resolution_scale: f64,
        known_to_be_opaque: bool,
    },
}

/// The currently selected request for an HTML responsive image. The DOM image
/// loader owns this per-element state: decoded resources remain keyed by URL,
/// while candidate density belongs to the particular `img` use.
#[derive(Clone, Debug, PartialEq)]
pub struct ResponsiveImageSelection {
    pub source: String,
    /// `Some` for a `srcset` candidate, including an implicit 1x descriptor.
    /// Plain `src` keeps `None` so resource-owned resolution metadata remains
    /// authoritative for non-responsive images.
    pub density: Option<f64>,
}

impl PartialEq for PseudoContent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => a == b,
            (
                Self::Image {
                    resource_id: a_id,
                    shader: a_shader,
                    natural_width: a_width,
                    natural_height: a_height,
                    resolution_scale: a_scale,
                    known_to_be_opaque: a_opaque,
                },
                Self::Image {
                    resource_id: b_id,
                    shader: b_shader,
                    natural_width: b_width,
                    natural_height: b_height,
                    resolution_scale: b_scale,
                    known_to_be_opaque: b_opaque,
                },
            ) => {
                a_id == b_id
                    && a_shader == b_shader
                    && a_width == b_width
                    && a_height == b_height
                    && a_scale == b_scale
                    && a_opaque == b_opaque
            }
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct PseudoElement {
    /// Owns the immutable Chromium style used by DOM, Layout and Paint.
    pub computed_style_owner: Persistent<StyleComputedStyle>,
    pub text: String,
    /// Ordered non-alt generated content. Chromium creates one anonymous
    /// layout child per ContentData node; `text` remains the concatenated view
    /// used by the current first-letter adapter.
    pub content: Vec<PseudoContent>,
    pub display_contents: bool,
    /// A generated ::before/::after whose computed display is list-item owns
    /// its own ::marker pseudo. The box and the cached pseudo style share the
    /// same canonical Chromium style owner.
    pub marker: Option<Box<PseudoElement>>,
}

impl PartialEq for PseudoElement {
    fn eq(&self, other: &Self) -> bool {
        self.computed_style_owner.Get() == other.computed_style_owner.Get()
            && self.text == other.text
            && self.content == other.content
            && self.display_contents == other.display_contents
            && self.marker == other.marker
    }
}

impl Default for PseudoElement {
    fn default() -> Self {
        Self {
            computed_style_owner: Persistent::default(),
            text: String::new(),
            content: Vec::new(),
            display_contents: false,
            marker: None,
        }
    }
}

impl PseudoElement {
    fn SamePaintInput(&self, other: &Self) -> bool {
        let equal = match (
            unsafe { self.computed_style_owner.Get().as_ref() },
            unsafe { other.computed_style_owner.Get().as_ref() },
        ) {
            (Some(a), Some(b)) => {
                layoutng_style::style::computed_style_base::ComputedStyleBase::FieldInvalidationDiff(
                    a, b,
                ) == 0
            }
            (None, None) => true,
            _ => false,
        };
        equal
            && self.text == other.text
            && self.content == other.content
            && self.display_contents == other.display_contents
            && match (&self.marker, &other.marker) {
                (Some(a), Some(b)) => a.SamePaintInput(b),
                (None, None) => true,
                _ => false,
            }
    }
    pub fn SetComputedStyleOwner(&mut self, owner: Persistent<StyleComputedStyle>) {
        self.computed_style_owner = owner;
    }
}

#[derive(Clone)]
pub struct ResolvedNodeStyle {
    /// Roots the immutable Chromium style used by the live LayoutObject tree.
    pub computed_style_owner: Persistent<StyleComputedStyle>,
    pub generates_box: bool,
    pub own_generates_box: bool,
    pub own_display_contents: bool,
    pub display_contents: bool,
    pub custom_properties: std::sync::Arc<HashMap<String, Option<String>>>,
    /// StyleBuilder's explicit-inheritance dependency after variable resolution.
    /// None denotes an external/old style without a dependency proof.
    pub has_explicit_inheritance: Option<bool>,
    // Blink resolves pending StyleImages while producing ComputedStyle. Keep
    // the corresponding immutable dependency list with that resolved result;
    // resource discovery must not re-walk every canonical style field on each
    // rendering opportunity.
    referenced_style_image_sources: std::sync::Arc<[String]>,
    pub before: Option<PseudoElement>,
    pub after: Option<PseudoElement>,
    pub first_letter: Option<PseudoElement>,
    pub placeholder: Option<PseudoElement>,
    /// Generated list marker style owned by the originating list item. The
    /// same canonical object is cached on the originating style under
    /// kPseudoIdMarker and is borrowed by Layout during attachment.
    pub marker: Option<PseudoElement>,
}

#[derive(Clone, Default, PartialEq)]
pub struct ResolvedStyleResources {
    pub svg_mask_resource:
        Option<std::sync::Arc<layoutng_assembly::internal::paint_input::SvgMaskGraph>>,
    pub svg_pattern_content_target: Option<u64>,
    // Source ContainerQueryEvaluator dimensions for SVG animated length units.
    // None means the embedding has not supplied a container-query environment;
    // Some(empty dimensions) means source selection found no eligible size.
    pub svg_length_container_sizes:
        Option<layoutng_style::css::css_to_length_conversion_data::ContainerSizes>,
    /// The same SVG animated-length numeric AST after StyleCascade token
    /// substitution. Layout borrows it without parser or custom-property data.
    pub svg_marker_resolved_lengths:
        Option<std::sync::Arc<layoutng_style::css::svg_length_value::SVGMarkerLengthValues>>,
    pub svg_fill_server: Option<layoutng_assembly::internal::paint_input::SvgPaintServer>,
    pub svg_stroke_server: Option<layoutng_assembly::internal::paint_input::SvgPaintServer>,
    pub svg_markers: Vec<layoutng_assembly::internal::paint_input::SvgMarkerInstance>,
}

impl PartialEq for ResolvedNodeStyle {
    fn eq(&self, other: &Self) -> bool {
        self.computed_style_owner.Get() == other.computed_style_owner.Get()
            && self.generates_box == other.generates_box
            && self.own_generates_box == other.own_generates_box
            && self.own_display_contents == other.own_display_contents
            && self.display_contents == other.display_contents
            && (std::sync::Arc::ptr_eq(&self.custom_properties, &other.custom_properties)
                || self.custom_properties == other.custom_properties)
            && self.has_explicit_inheritance == other.has_explicit_inheritance
            && self.before == other.before
            && self.after == other.after
            && self.first_letter == other.first_letter
            && self.placeholder == other.placeholder
            && self.marker == other.marker
    }
}

impl Default for ResolvedNodeStyle {
    fn default() -> Self {
        Self {
            computed_style_owner: Persistent::default(),
            generates_box: true,
            own_generates_box: true,
            own_display_contents: false,
            display_contents: false,
            custom_properties: Default::default(),
            has_explicit_inheritance: None,
            referenced_style_image_sources: Default::default(),
            before: None,
            after: None,
            first_letter: None,
            placeholder: None,
            marker: None,
        }
    }
}

impl ResolvedNodeStyle {
    /// Borrow the canonical Chromium style rooted by this element.
    pub fn ComputedStyle(&self) -> Option<&StyleComputedStyle> {
        unsafe { self.computed_style_owner.Get().as_ref() }
    }

    pub fn SetComputedStyleOwner(&mut self, owner: Persistent<StyleComputedStyle>) {
        self.computed_style_owner = owner;
        self.referenced_style_image_sources = Default::default();
    }

    /// URLs referenced by the canonical computed style, including
    /// generated-content pseudos. Resource loading consumes this dependency
    /// view without re-parsing CSS or flattening StyleImage into DOM state.
    pub fn ReferencedStyleImageSources(&self) -> &[String] {
        self.referenced_style_image_sources.as_ref()
    }

    fn RefreshReferencedStyleImageSources(&mut self) {
        if self.computed_style_owner.Get().is_null() {
            return;
        }
        self.referenced_style_image_sources = StyleImageSources(self).into();
    }
}

fn CollectStyleImageSources(style: &StyleComputedStyle, output: &mut Vec<String>) {
    use layoutng_style::style::{
        content_data::ImageContentData, fill_layer::FillLayer, style_fetched_image::FetchedImage,
        style_image::StyleImage,
    };
    fn append(image: *mut StyleImage, output: &mut Vec<String>) {
        let Some(image) = (unsafe { image.as_ref() }) else {
            return;
        };
        let image = layoutng_style::style::style_mask_source_image::MaskSourceImage(image)
            .and_then(|mask| mask.FetchedImage())
            .unwrap_or(image);
        if let Some(image) = FetchedImage(image) {
            output.push(image.SourceURL().to_owned());
        }
    }
    fn layers(mut layer: *const FillLayer, output: &mut Vec<String>) {
        while let Some(current) = unsafe { layer.as_ref() } {
            append(current.GetImage(), output);
            layer = current.Next();
        }
    }
    layers(style.BackgroundLayers(), output);
    layers(style.MaskLayers(), output);
    append(style.BorderImageSource(), output);
    append(style.MaskBoxImageSource(), output);
    if let Some(reflection) = unsafe { style.BoxReflect().as_ref() } {
        append(reflection.Mask().GetImage(), output);
    }
    append(style.ListStyleImage().Get(), output);
    if let Some(shape) = unsafe { style.ShapeOutside().as_ref() } {
        append(shape.GetImage(), output);
    }
    if let Some(cursors) = unsafe { style.Cursors().as_ref() } {
        for cursor in cursors.iter() {
            append(cursor.GetImage(), output);
        }
    }
    let mut content = style.GetContentData();
    while let Some(pointer) = content {
        let data = unsafe { &*pointer };
        if data.IsImage() {
            append(
                unsafe { &*(pointer as *const ImageContentData) }.GetImage(),
                output,
            );
        }
        content = data.Next();
    }
}

fn StyleImageSources(style: &ResolvedNodeStyle) -> Vec<String> {
    let mut output = Vec::new();
    fn append_pseudo(pseudo: &PseudoElement, output: &mut Vec<String>) {
        if let Some(source) = unsafe { pseudo.computed_style_owner.Get().as_ref() } {
            CollectStyleImageSources(source, output);
        }
        if let Some(marker) = pseudo.marker.as_deref() {
            append_pseudo(marker, output);
        }
    }
    if let Some(source) = unsafe { style.computed_style_owner.Get().as_ref() } {
        CollectStyleImageSources(source, &mut output);
    }
    for pseudo in style
        .before
        .iter()
        .chain(style.after.iter())
        .chain(style.first_letter.iter())
        .chain(style.placeholder.iter())
        .chain(style.marker.iter())
    {
        append_pseudo(pseudo, &mut output);
    }
    output
}

// cpp: dom/document.h:79-85
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CompatibilityMode {
    kNoQuirks,
    kLimitedQuirks,
    kQuirks,
}

// ImageResourceContent's terminal state is published by the resource owner.
// No entry means the request has not completed; errors retain the source key
// without manufacturing renderable image metadata.
enum ImageResourceState {
    Pending(Rc<DOMImageResourceContent>),
    Loaded(ImageResourceMetadata, Rc<DOMImageResourceContent>),
    Failed(Rc<DOMImageResourceContent>),
}

// cpp: dom/document.h:207-232
// Re-exported as dom::Document.
struct DOMTextControlRuntimeState {
    opaque_ranges: Rc<RefCell<crate::opaque_range::TextControlOpaqueRanges>>,
    user_interaction: u8,
    force_user_valid: bool,
    blocks_form_submission: bool,
    needs_validity_check: bool,
    will_validate: Option<bool>,
    cached_validity: bool,
    custom_validation_message: Rc<[u16]>,
    value_revision: u64,
    needs_update_view_value: bool,
    has_been_password_field: bool,
    should_reveal_password: bool,
    should_draw_caps_lock_indicator: bool,
}
impl Default for DOMTextControlRuntimeState {
    fn default() -> Self {
        Self {
            opaque_ranges: Rc::new(RefCell::new(Default::default())),
            user_interaction: 0,
            force_user_valid: false,
            blocks_form_submission: false,
            needs_validity_check: true,
            will_validate: None,
            cached_validity: true,
            custom_validation_message: Rc::from([]),
            value_revision: 0,
            needs_update_view_value: true,
            has_been_password_field: false,
            should_reveal_password: false,
            should_draw_caps_lock_indicator: false,
        }
    }
}
pub struct PersistentDocument {
    document_url: String,
    is_for_markup_sanitization: bool,
    pub(super) base_url: crate::document_base_url::DocumentBaseURL,
    navigation_document_token: Rc<crate::navigation_tokens::DocumentTokenOwner>,
    fragment_directive: Rc<foundation::fragment_directive::FragmentDirective>,
    document_speculation_rules: RefCell<Option<Rc<dyn DocumentSpeculationRules>>>,
    form_state_drops_referenced_file_paths: Cell<bool>,
    // Document owns the actual tracker Rc; Any only avoids a DOM -> default dependency.
    navigation_state: RefCell<Option<Rc<NavigationStateSupplement>>>,
    skeleton_loader: RefCell<Option<Rc<SkeletonLoaderSupplement>>>,
    window_navigation_api: RefCell<Option<Rc<dyn WindowNavigationApiOwner>>>,
    anchor_viewport_tracker: RefCell<Option<Rc<dyn DocumentAnchorViewportOwner>>>,
    vtt_originating_element: Cell<Option<usize>>,
    vtt_contains_style_block_samples: RefCell<std::collections::VecDeque<bool>>,
    anchor_element_interaction_tracker: RefCell<Option<Rc<dyn std::any::Any>>>,
    // Supplement<Document>::AnnotationAgentContainerImpl. Any erases only the
    // concrete downstream owner type, preserving its single Rc identity.
    annotation_agent_container: RefCell<Option<Rc<dyn std::any::Any>>>,
    // Native execution-context task lifetime; dropping this token cancels
    // pending actor requests without a DOM query or script callback.
    anchor_navigation_context_lifetime: RefCell<Option<Rc<dyn std::any::Any>>>,
    native_execution_context_lifetime:
        Rc<crate::native_execution_context_lifetime::NativeExecutionContextLifetime>,
    security_context: SecurityContext,
    frame_ad_evidence: RefCell<Option<crate::frame_ad_evidence::FrameAdEvidence>>,
    execution_context_isolation: ExecutionContextIsolation,
    navigation_security_initialized: bool,
    mobile_metrics: RefCell<Vec<DocumentTappedBadTarget>>,
    // Parsing arena handoff transfers this SAME lifecycle owner with Document.
    lifecycle: Rc<crate::document_lifecycle::DocumentLifecycle>,
    view_transitions: RefCell<Option<Rc<ViewTransitionSupplement>>>,
    media_elements: RefCell<HashMap<u64, Rc<HTMLMediaElementState>>>,
    media_source_attachments: RefCell<HashMap<String, Rc<dyn DOMMediaSourceAttachment>>>,
    load_event_delay_counts:
        RefCell<HashMap<DOMOwnerHandle, Rc<crate::load_event_delay_count::LoadEventDelayCounter>>>,
    picture_in_picture_state: std::cell::OnceCell<Rc<DOMPictureInPictureState>>,
    canvas_elements: RefCell<HashMap<u64, Rc<RefCell<canvas::HTMLCanvasElementState>>>>,
    canvas_transforms: HashMap<u64, foundation::gfx::Transform>,
    canvas_runtime_features: std::cell::Cell<(bool, bool)>,
    canvas_document_owner_state: HashMap<DOMOwnerHandle, bool>,
    drag_js_file_objects: std::cell::Cell<bool>,
    date_time_custom_styles: HashMap<u64, date_time_custom_style::DateTimeCustomStyleEntry>,
    canvas_child_records: RefCell<Option<std::sync::Arc<canvas::CanvasChildRecordPublication>>>,
    display_lock_document_state: RefCell<DisplayLockDocumentState>,
    focusgroup_features: foundation::focusgroup_flags::FocusgroupFeatures,
    unload_started: std::cell::Cell<bool>,
    arena: Vec<DOMNode>,
    // cpp: TreeScope::elements_by_id_. Connected document and shadow tree
    // scopes own the live ID membership. Duplicate IDs remain an unordered
    // set here; the uncommon ordered query walks only to order those matches,
    // as Blink's TreeOrderedMap does lazily.
    tree_scope_elements_by_id: HashMap<usize, HashMap<String, HashSet<usize>>>,
    // SlotAssignment's native named-slot membership; the same owner answers
    // AssignedSlot and drives canvas flat-tree lifecycle, not a second tree.
    slots_by_name: HashMap<usize, HashMap<Rc<[u16]>, HashSet<usize>>>,
    image_map_state: RefCell<html_image_map::ImageMapDocumentState>,
    node_reference_observers: crate::node_references::NodeReferenceObservers,
    node_removal_observer:
        Option<std::rc::Weak<dyn crate::node_removal_observer::DocumentNodeRemovalObserver>>,
    attribute_lifecycle: Rc<crate::attribute_lifecycle::AttributeLifecycleQueue>,
    // HTML input/button identities are stable across attribute/type changes.
    // Notifications still check live type, form and connectedness.
    form_observer_elements: Option<Vec<usize>>,
    arena_id: u64,
    pub(crate) custom_elements: crate::custom_elements::CustomElementRegistry,
    pub(super) sequential_focus_navigation_starting_point: Option<crate::range::DOMRangeHandle>,
    pub(crate) ranges: Vec<std::rc::Weak<std::cell::RefCell<crate::range::RangeData>>>,
    pub(crate) mutation_observers: crate::mutation_observers::MutationObservers,
    pub(crate) mutation_record_captures: crate::mutation_observers::MutationRecordCaptures,
    collection_membership_revision: u64,
    stateful_form_control_revision: u64,
    pub(crate) form_controller:
        RefCell<Option<std::rc::Weak<dyn crate::form_controller::DocumentFormController>>>,
    html_base_target: RefCell<Option<(u64, String)>>,
    form_collection_state: RefCell<form_control_state::FormCollectionState>,
    pending_control_validity: BTreeSet<u64>,
    overscroll_document: OverscrollDocumentData,
    selector_filter_generation: u64,
    // Local SVG resource identity/content is owned by this document. Frames
    // retain immutable graph Arcs; style replacement never rewrites them.
    svg_clip_resources:
        Option<std::sync::Arc<layoutng_assembly::internal::paint_input::SvgClipResourceSnapshot>>,
    svg_clip_resource_inputs: Option<(u64, u64)>,
    svg_filter_graphs:
        std::sync::Arc<Vec<layoutng_assembly::internal::paint_input::SvgFilterGraph>>,
    resolved_style_revision: u64,
    // SVGResources tracks changes to resource definitions/descendants, rather
    // than invalidating the catalog for an unrelated HTML color or hover edit.
    svg_resource_dom_revision: u64,
    svg_resource_style_revision: u64,
    root: usize,
    next_id: u64,
    editing_mutations: Vec<DocumentEditingMutation>,
    layout_tree_detachments: Vec<DocumentLayoutTreeDetachment>,
    svg_resources_needing_invalidation: HashMap<usize, Rc<dyn PendingSVGResourceInvalidation>>,
    dom_tree_version: std::cell::Cell<u64>,
    compatibility: CompatibilityMode,
    // cpp: Settings::GetTextAreasAreResizable. Resize::ApplyValue consumes
    // this document setting for the UA-only -internal-textarea-auto value.
    text_areas_are_resizable: bool,
    scripting_enabled: bool,
    user_action_elements: UserActionElements,
    // cpp: Settings::GetDefaultFontSize/GetDefaultFixedFontSize. The
    // proportional value is also consumed by LayoutThemeFontProvider for
    // system-font shorthands; both values belong to the document settings
    // owner and are passed into Style's FontBuilder conversion boundary.
    anchor_metrics_owner:
        RefCell<Option<Rc<dyn anchor_metrics_supplement::DocumentAnchorMetricsOwner>>>,
    default_font_size: f32,
    default_fixed_font_size: f32,
    editing_font_settings: editing_font_settings::EditingFontSettings,
    document_types: HashMap<u64, DOMDocumentType>,
    pub(crate) content_security_policy:
        std::sync::Arc<crate::content_security_policy::DocumentContentSecurityPolicy>,
    pub(crate) document_metadata: HashMap<u64, crate::document_metadata::DocumentMetadata>,
    pub(crate) navigation_lifecycle:
        Rc<crate::document_parser_lifecycle::DocumentNavigationLifecycle>,
    // Document::design_mode_ is off at construction. The JS setter is not
    // exposed until its Style/permissions lifecycle has a translated owner.
    document_design_mode: HashSet<u64>,
    template_contents: HashMap<u64, usize>,
    // cpp: element.h:1129-1132; shadow_root.h:97-110.  Shadow roots are not
    // ordinary children of their hosts.  These two maps retain the two sides
    // of Blink's Element/ShadowRoot ownership without changing Parent().
    shadow_roots: HashMap<usize, (usize, DOMShadowRootMode, DOMSlotAssignmentMode)>,
    shadow_roots_delegating_focus: HashSet<usize>,
    shadow_hosts: HashMap<usize, usize>,
    shadow_reference_targets: HashMap<usize, String>,
    explicitly_set_element_attributes: HashMap<(usize, String), usize>,
    shadow_root_reference_target_enabled: bool,
    select_source_features: DOMSelectSourceFeatures,
    select_elements_allocated: usize,
    details_summary_label: Option<String>,
    details_toggle_during_parsing: bool,
    // cpp: html_slot_element.h manually_assigned_nodes_ and
    // flat_tree_node_data.h manually_assigned_slot_. The forward sequence
    // preserves assign() order; the reverse owner survives disconnection and
    // is validated against the current manual ShadowRoot on lookup.
    manually_assigned_nodes: HashMap<usize, Vec<usize>>,
    manually_assigned_slots: HashMap<usize, usize>,
    // cpp: html/forms/text_control_inner_elements.h; input_type.cc.
    // Text controls retain their UA-shadow implementation element separately
    // from the light-DOM child list. CSS ::placeholder addresses this actual
    // element through its ShadowPseudoId, rather than a virtual pseudo node.
    shadow_pseudo_ids: HashMap<usize, String>,
    ultimate_originating_elements: HashMap<usize, usize>,
    ua_shadow_pseudo_elements: HashMap<usize, HashMap<String, usize>>,
    generated_pseudo_elements:
        HashMap<usize, HashMap<layoutng_style::style::computed_style_constants::PseudoId, usize>>,
    generated_pseudo_facts: HashMap<usize, DOMPseudoElementFacts>,
    range_slider_shadow_parts: HashMap<usize, (usize, DOMRangeShadowPart)>,
    text_field_spin_button_hosts: HashMap<usize, usize>,
    text_control_placeholders: HashMap<usize, usize>,
    text_control_inner_editors: HashMap<usize, usize>,
    text_control_inner_editor_hosts: HashMap<usize, usize>,
    text_control_containers: HashMap<usize, usize>,
    email_verification_indicator_hosts: HashMap<usize, usize>,
    // cpp: SearchInputType::CreateShadowSubtree. SearchFieldCancelButtonElement
    // is a retained implementation element placed after the editing viewport.
    search_cancel_buttons: HashMap<usize, usize>,
    // cpp: ColorInputType::CreateShadowSubtree/UpdateView. The wrapper and
    // swatch are retained implementation elements; the swatch owns the
    // sanitized control value as its inline background color.
    color_swatch_wrappers: HashMap<usize, usize>,
    color_swatches: HashMap<usize, usize>,
    // cpp: FileInputType::CreateShadowSubtree. File controls retain the real
    // input[type=button] and status span used by html.css and layout. The
    // platform FileList/chooser remains outside this DOM projection.
    file_upload_buttons: HashMap<usize, usize>,
    file_status_elements: HashMap<usize, usize>,
    suggested_value_state: HashMap<u64, String>,
    // cpp: HTMLSelectElement::suggested_option_. Retain option identity so
    // removal and reassociation clear preview state by owner, not by value.
    select_suggested_options: HashMap<u64, u64>,
    // cpp: HTMLMeterElement::DidAddUserAgentShadowRoot/UpdateValueAppearance.
    // The three implementation divs are real retained nodes. Their changing
    // ShadowPseudoId selects the corresponding typed html.css rule.
    meter_value_elements: HashMap<usize, usize>,
    // cpp: HTMLProgressElement::DidAddUserAgentShadowRoot. The retained value
    // implementation element owns the value/max-derived inline-size while
    // :indeterminate remains a state of the host.
    progress_value_elements: HashMap<usize, usize>,
    // cpp: Element::CachedDirectionality. Directionality is DOM-owned state:
    // selectors and StyleResolver consume it without rescanning text.
    cached_directionality: HashMap<usize, TextDirection>,
    // cpp: element.cc:3809; node_rare_data.cc:376-382. Parsed exportparts
    // mappings are element-owned retained state, not a style-time string
    // projection.
    part_names_maps: HashMap<usize, DOMPartNamesMap>,
    // cpp: css_animation_update.h:318-332. Sampled animations and sampled
    // transitions are distinct retained cascade buckets even when an effect
    // producer publishes the same declaration representation for both.
    animated_styles: HashMap<u64, BTreeMap<u64, Vec<CSSDeclaration>>>,
    transitioned_styles: HashMap<u64, BTreeMap<u64, Vec<CSSDeclaration>>>,
    cssom: CSSOM<DOMOwnerHandle>,
    // Ownerless sheets appended through Document are explicitly attached to
    // the document TreeScope here. Constructed sheets use CSSOM's separate
    // adopted relation and are never inferred from owner_node_id == 0.
    ownerless_style_sheet_scopes: HashMap<u64, usize>,
    image_resources: HashMap<String, ImageResourceState>,
    broken_image_resources:
        RefCell<Option<image_resource::platform_resources::BrokenImageResourceVariants>>,
    // HTMLVideoElement owns current-frame identity and natural dimensions;
    // pixels remain in the immutable PaintImage resource catalog.
    video_frames: HashMap<u64, VideoFrameMetadata>,
    // SVGImage/ImageResourceContent type is resource-owned metadata. Style
    // needs it to apply image-set resolution only to bitmap images, matching
    // StyleFetchedImage::ApplyImageResolution without depending on Decode.
    document_image_resource_ids: HashSet<u64>,
    /// Image::MaybeAnimated resource fact. Kept beside the resource catalog so
    /// LayoutImage invalidation can defer animated repaints without querying
    /// the paint/compositor snapshot.
    maybe_animated_image_resource_ids: HashSet<u64>,
    content_dpr_by_image_resource_id: HashMap<u64, f64>,
    // core/frame/UseCounter is sticky document state. LayoutImage reports the
    // two image features through exact resource-client and paint facts below.
    use_counter: crate::use_counter::UseCounter,
    responsive_image_selections: HashMap<usize, ResponsiveImageSelection>,
    // HTMLImageLoader/ImageResourceContent client relation. Attribute and
    // responsive candidate changes retain the exact <img> consumers so
    // completion does not rediscover them by scanning the DOM.
    replaced_image_consumers: HashMap<String, BTreeSet<usize>>,
    // HTMLImageElement::last_auto_sizes_width_. Kept on the DOM/image-loader
    // owner because it survives source reselection but is reset by removal.
    responsive_image_auto_sizes: HashMap<usize, f64>,
    scroll_offsets: HashMap<u64, Offset>,
    // StyleFetchedImage::AddClient/RemoveClient: only resolved CSS consumers
    // need their background/mask resource IDs refreshed on image completion.
    style_image_consumers: HashMap<String, BTreeSet<usize>>,
    style_state: crate::style_state::StyleState,
    control_checked_state: HashMap<u64, bool>,
    radio_groups: HashMap<(DOMRadioGroupScope, String), radio_button_group_scope::RadioButtonGroup>,
    radio_memberships: HashMap<usize, (DOMRadioGroupScope, String)>,
    input_type_lifecycle_owners: HashSet<u64>,
    input_file_no_file_labels: HashMap<u64, String>,
    input_file_lists: RefCell<HashMap<u64, Rc<fileapi::FileList>>>,
    input_image_click_locations: RefCell<HashMap<u64, Rc<std::cell::Cell<(i32, i32)>>>>,
    input_image_fallback_states: RefCell<HashMap<u64, Rc<std::cell::Cell<bool>>>>,
    input_image_locales: RefCell<HashMap<u64, Rc<dyn InputAltTextLocale>>>,
    // cpp: HTMLOptionElement::is_selected_/is_dirty_. Selectedness is live
    // state; the selected content attribute is only defaultSelected and stops
    // driving selectedness after the IDL selection has become dirty.
    option_selected_state: HashMap<u64, bool>,
    option_dirty_state: HashSet<u64>,
    // cpp: html_input_element.h is_indeterminate_. This is live IDL state,
    // independent of the checked content attribute and retained across input
    // type changes.
    control_indeterminate_state: HashMap<u64, bool>,
    // Canonical WTF-16 value; UTF-8 ControlValue is only a display projection.
    control_value_state: HashMap<u64, foundation::NullableUTF16String>,
    control_value_dirty: HashSet<u64>,
    input_value_attribute_utf16: HashMap<u64, Rc<[u16]>>,
    input_value_sanitizer: Option<Rc<dyn crate::input_value::DOMInputValueSanitizer>>,
    text_control_runtime: HashMap<u64, DOMTextControlRuntimeState>,
    should_update_selection_after_layout: bool,
    scheduled_selectionchange_targets: HashSet<u64>,
    // CharacterData retains exact code units on the same canonical DOM node.
    // The existing DOMNode::data is the scalar rendering/parser projection.
    character_data_utf16: HashMap<u64, Rc<[u16]>>,
    text_control_last_change_was_user_edit: HashSet<u64>,
    // cpp: HTMLFormControlElement::autofill_state_. This is live form state,
    // separate from attributes and from the suggested text rendered in the UA
    // shadow tree.
    control_autofill_state: HashMap<u64, DOMAutofillState>,
    // cpp: document.h TopLayerElements; html_dialog_element.h is_modal_;
    //      popover_data.h visibility_state_. Attribute `open` is deliberately
    // not used as modal or top-layer state.
    top_layer_elements: Vec<u64>,
    pub(crate) state_preserving_atomic_move_in_progress: Rc<Cell<bool>>,
    popup_lifecycle_state: crate::popup_state::PopupLifecycleState,
    open_popovers: HashSet<u64>,
    invoked_popovers: HashMap<usize, usize>,
    // Document::PopoverPickerPointerdownInfo, retained across the input pair.
    popover_picker_pointerdown: Option<(DOMOwnerHandle, (f32, f32))>,
    // cpp: fullscreen/fullscreen.cc FullscreenParamsMap plus Document's top
    // layer ordering. Every entry owns the fullscreen flag; the last entry is
    // the document's current fullscreen element.
    fullscreen_element_stack: Vec<u64>,
    // cpp: document.h css_target_. The fragment-navigation owner updates this
    // retained Document state; selector matching only observes it.
    css_target: Option<u64>,
    // SVGSVGElement::view_spec_. Only an isolated SVG document-image owner
    // writes this state, immediately before its Style/Layout/Paint lifecycle.
    viewport_style: Option<Persistent<StyleComputedStyle>>,
    svg_pan_translate: [f32; 2],
    svg_user_transforms: HashMap<usize, [f32; 2]>,
    svg_image_view_root: Option<u64>,
    svg_image_view_spec: Option<SVGImageViewSpec>,
    // cpp: svg/svg_element_rare_data.h corresponding_element_. SVG <use>
    // instance-tree construction owns this relation; selector matching only
    // observes it. Ordinary SVG elements have no corresponding element.
    corresponding_svg_elements: HashMap<u64, u64>,
    svg_use_instances: HashMap<usize, svg_use_instances::SVGUseInstancePlan>,
    svg_use_instance_status: HashMap<usize, SVGUseInstanceStatus>,
    svg_use_inputs_revision: Option<u64>,
}

impl PersistentDocument {
    // cpp: dom/document.cc:36-39
    fn new() -> Self {
        let mut document = Self {
            form_state_drops_referenced_file_paths: Cell::new(false),
            arena: Vec::new(),
            tree_scope_elements_by_id: HashMap::new(),
            slots_by_name: HashMap::new(),
            image_map_state: RefCell::new(Default::default()),
            node_reference_observers: Vec::new(),
            node_removal_observer: None,
            attribute_lifecycle: Rc::new(
                crate::attribute_lifecycle::AttributeLifecycleQueue::default(),
            ),
            form_observer_elements: Some(Vec::new()),
            arena_id: NEXT_ARENA_ID.fetch_add(1, Ordering::Relaxed),
            document_url: String::new(),
            is_for_markup_sanitization: false,
            base_url: Default::default(),
            navigation_document_token: Rc::new(Default::default()),
            document_speculation_rules: Default::default(),
            fragment_directive: Rc::new(Default::default()),
            vtt_originating_element: Cell::new(None),
            vtt_contains_style_block_samples: RefCell::new(Default::default()),
            anchor_element_interaction_tracker: RefCell::new(None),
            anchor_viewport_tracker: Default::default(),
            window_navigation_api: Default::default(),
            navigation_state: Default::default(),
            skeleton_loader: Default::default(),
            annotation_agent_container: RefCell::new(None),
            anchor_navigation_context_lifetime: RefCell::new(None),
            native_execution_context_lifetime: Rc::new(
                crate::native_execution_context_lifetime::NativeExecutionContextLifetime::new(),
            ),
            frame_ad_evidence: RefCell::new(None),
            security_context: SecurityContext::NewInitial(Rc::new(
                SecurityOriginSchemeRegistry::default(),
            )),
            lifecycle: Rc::new(crate::document_lifecycle::DocumentLifecycle::Inactive()),
            view_transitions: RefCell::new(None),
            media_elements: RefCell::new(HashMap::new()),
            media_source_attachments: RefCell::new(HashMap::new()),
            load_event_delay_counts: RefCell::new(HashMap::new()),
            picture_in_picture_state: std::cell::OnceCell::new(),
            canvas_elements: RefCell::new(HashMap::new()),
            canvas_transforms: HashMap::new(),
            canvas_runtime_features: std::cell::Cell::new((false, false)),
            canvas_document_owner_state: HashMap::new(),
            // RuntimeEnabledFeatures.json5:2518-2523: test-only at fixed revision.
            drag_js_file_objects: std::cell::Cell::new(false),
            date_time_custom_styles: HashMap::new(),
            canvas_child_records: RefCell::new(None),
            display_lock_document_state: RefCell::new(DisplayLockDocumentState::default()),
            focusgroup_features: Default::default(),
            unload_started: std::cell::Cell::new(false),
            custom_elements: Default::default(),
            ranges: Vec::new(),
            sequential_focus_navigation_starting_point: None,
            mutation_observers: Default::default(),
            mutation_record_captures: Default::default(),
            collection_membership_revision: 0,
            stateful_form_control_revision: 0,
            form_controller: RefCell::new(None),
            html_base_target: RefCell::new(None),
            form_collection_state: RefCell::new(Default::default()),
            pending_control_validity: BTreeSet::new(),
            overscroll_document: Default::default(),
            selector_filter_generation: 0,
            svg_clip_resources: None,
            svg_filter_graphs: std::sync::Arc::new(Vec::new()),
            svg_clip_resource_inputs: None,
            resolved_style_revision: 0,
            svg_resource_dom_revision: 0,
            svg_resource_style_revision: 0,
            root: 0,
            next_id: 1,
            editing_mutations: Vec::new(),
            layout_tree_detachments: Vec::new(),
            svg_resources_needing_invalidation: HashMap::new(),
            dom_tree_version: std::cell::Cell::new(0),
            execution_context_isolation: ExecutionContextIsolation::ForNonIsolatedAgentCluster(),
            navigation_security_initialized: false,
            mobile_metrics: RefCell::new(Vec::new()),
            compatibility: CompatibilityMode::kNoQuirks,
            text_areas_are_resizable: true,
            scripting_enabled: false,
            user_action_elements: UserActionElements::default(),
            anchor_metrics_owner: RefCell::new(None),
            default_font_size: 16.0,
            default_fixed_font_size: 13.0,
            editing_font_settings: editing_font_settings::EditingFontSettings::new(),
            document_types: HashMap::new(),
            content_security_policy: Default::default(),
            document_metadata: HashMap::new(),
            navigation_lifecycle: Rc::new(Default::default()),
            document_design_mode: HashSet::new(),
            template_contents: HashMap::new(),
            shadow_roots: HashMap::new(),
            shadow_roots_delegating_focus: HashSet::new(),
            shadow_hosts: HashMap::new(),
            shadow_reference_targets: HashMap::new(),
            explicitly_set_element_attributes: HashMap::new(),
            // runtime_enabled_features.json5:5685-5687: stable.
            shadow_root_reference_target_enabled: true,
            select_source_features: DOMSelectSourceFeatures::default(),
            select_elements_allocated: 0,
            details_summary_label: None,
            details_toggle_during_parsing: false,
            manually_assigned_nodes: HashMap::new(),
            manually_assigned_slots: HashMap::new(),
            shadow_pseudo_ids: HashMap::new(),
            ultimate_originating_elements: HashMap::new(),
            ua_shadow_pseudo_elements: HashMap::new(),
            generated_pseudo_elements: HashMap::new(),
            generated_pseudo_facts: HashMap::new(),
            range_slider_shadow_parts: HashMap::new(),
            text_field_spin_button_hosts: HashMap::new(),
            text_control_placeholders: HashMap::new(),
            text_control_inner_editors: HashMap::new(),
            text_control_inner_editor_hosts: HashMap::new(),
            text_control_containers: HashMap::new(),
            email_verification_indicator_hosts: HashMap::new(),
            search_cancel_buttons: HashMap::new(),
            color_swatch_wrappers: HashMap::new(),
            color_swatches: HashMap::new(),
            file_upload_buttons: HashMap::new(),
            file_status_elements: HashMap::new(),
            suggested_value_state: HashMap::new(),
            select_suggested_options: HashMap::new(),
            meter_value_elements: HashMap::new(),
            progress_value_elements: HashMap::new(),
            cached_directionality: HashMap::new(),
            part_names_maps: HashMap::new(),
            animated_styles: HashMap::new(),
            transitioned_styles: HashMap::new(),
            cssom: CSSOM::default(),
            ownerless_style_sheet_scopes: HashMap::new(),
            image_resources: HashMap::new(),
            broken_image_resources: RefCell::new(None),
            video_frames: HashMap::new(),
            document_image_resource_ids: HashSet::new(),
            maybe_animated_image_resource_ids: HashSet::new(),
            content_dpr_by_image_resource_id: HashMap::new(),
            use_counter: Default::default(),
            responsive_image_selections: HashMap::new(),
            replaced_image_consumers: HashMap::new(),
            responsive_image_auto_sizes: HashMap::new(),
            scroll_offsets: HashMap::new(),
            style_image_consumers: HashMap::new(),
            style_state: Default::default(),
            control_checked_state: HashMap::new(),
            radio_groups: HashMap::new(),
            radio_memberships: HashMap::new(),
            input_type_lifecycle_owners: HashSet::new(),
            input_file_no_file_labels: HashMap::new(),
            input_file_lists: RefCell::new(HashMap::new()),
            input_image_click_locations: RefCell::new(HashMap::new()),
            input_image_fallback_states: RefCell::new(HashMap::new()),
            input_image_locales: RefCell::new(HashMap::new()),
            option_selected_state: HashMap::new(),
            option_dirty_state: HashSet::new(),
            control_indeterminate_state: HashMap::new(),
            control_value_state: HashMap::new(),
            control_value_dirty: HashSet::new(),
            input_value_attribute_utf16: HashMap::new(),
            input_value_sanitizer: None,
            text_control_runtime: HashMap::new(),
            should_update_selection_after_layout: false,
            scheduled_selectionchange_targets: HashSet::new(),
            character_data_utf16: HashMap::new(),
            text_control_last_change_was_user_edit: HashSet::new(),
            control_autofill_state: HashMap::new(),
            top_layer_elements: Vec::new(),
            state_preserving_atomic_move_in_progress: Default::default(),
            popup_lifecycle_state: Default::default(),
            open_popovers: HashSet::new(),
            invoked_popovers: HashMap::new(),
            popover_picker_pointerdown: None,
            fullscreen_element_stack: Vec::new(),
            css_target: None,
            viewport_style: None,
            svg_pan_translate: [0.; 2],
            svg_user_transforms: HashMap::new(),
            svg_image_view_root: None,
            svg_image_view_spec: None,
            corresponding_svg_elements: HashMap::new(),
            svg_use_instances: HashMap::new(),
            svg_use_instance_status: HashMap::new(),
            svg_use_inputs_revision: None,
        };
        document.root = document.Allocate(
            DOMNodeType::kDocument,
            DOMNamespace::kNone,
            "#document".into(),
            String::new(),
            Vec::new(),
        );
        document
            .document_types
            .insert(document.arena[document.root].id, DOMDocumentType::kHTML);
        document
    }

    // cpp: dom/document.h:87-89
    // Source: Document::UnloadStarted (load-event progress is owned by the
    // document; DocumentLoader supplies the transition at unload dispatch).
    pub fn UnloadStarted(&self) -> bool {
        self.unload_started.get()
    }
    pub fn SetUnloadStarted(&self, started: bool) {
        self.unload_started.set(started);
    }
    pub fn InDesignMode(&self) -> bool {
        self.document_design_mode
            .contains(&self.arena[self.root].id)
    }
    pub fn DocumentInDesignMode(&self, document: DOMOwnerHandle) -> bool {
        document.arena_id == self.arena_id && self.document_design_mode.contains(&document.node_id)
    }
    /// HTMLElement::draggable, html_element.cc:3670-3673.
    pub fn IsHTMLElementDraggable(&self, node: usize) -> bool {
        self.Node(node).Type() == DOMNodeType::kElement
            && self.Node(node).Namespace() == DOMNamespace::kHTML
            && self
                .Node(node)
                .FindAttribute("draggable")
                .is_some_and(|a| a.value.eq_ignore_ascii_case("true"))
    }
    pub fn DragAndDropJSFileObjectsEnabled(&self) -> bool {
        self.drag_js_file_objects.get()
    }
    pub fn SetDragAndDropJSFileObjectsEnabled(&self, enabled: bool) {
        self.drag_js_file_objects.set(enabled);
    }
    pub fn IsActive(&self) -> bool {
        self.lifecycle.IsActive()
    }
    pub fn Lifecycle(&self) -> &Rc<crate::document_lifecycle::DocumentLifecycle> {
        &self.lifecycle
    }
    pub fn SetLifecycleActive(&self, active: bool) {
        self.SetLifecycleActiveForWindowTransition(active, false);
    }
    /// Document::Shutdown is separate from LocalDOMWindow::FrameDestroyed.
    /// A secure initial transition retires the old Document while keeping
    /// its Window execution context and public-URL owner for the replacement.
    pub fn ShutdownForWindowReuse(&self) {
        self.SetLifecycleActiveForWindowTransition(false, true);
    }
    fn SetLifecycleActiveForWindowTransition(&self, active: bool, reuse_window: bool) {
        use crate::document_lifecycle::LifecycleState;
        if active && !self.lifecycle.IsActive() {
            self.lifecycle.AdvanceTo(LifecycleState::StyleClean);
        } else if !active {
            // Document::Shutdown, document.cc:3412. Observers belong to this
            // Document lifetime, including a retained old navigation document.
            self.ClearFocusedElementChangeObservers();
            if !reuse_window {
                self.native_execution_context_lifetime.ContextDestroyed();
                self.security_context.PublicURLManager().ContextDestroyed();
            }
            // Release this Document's alias. The actual Window retains its
            // navigation_ member across a secure initial-window transition.
            self.window_navigation_api.borrow_mut().take();
            if self.lifecycle.GetState() < LifecycleState::Stopping {
                self.lifecycle.AdvanceTo(LifecycleState::Stopping);
            }
            let navigation_lifetime = self.anchor_navigation_context_lifetime.borrow_mut().take();
            drop(navigation_lifetime);
            if self.lifecycle.GetState() == LifecycleState::Stopping {
                self.lifecycle.AdvanceTo(LifecycleState::Stopped);
            }
        }
    }
    pub fn Token(&self) -> [u64; 2] {
        self.navigation_document_token.Token()
    }
    pub fn NavigationDocumentTokenOwner(&self) -> Rc<crate::navigation_tokens::DocumentTokenOwner> {
        self.navigation_document_token.clone()
    }
    /// Actual new-document response commit, distinct from SetDocumentURL.
    pub fn CommitNavigationDocumentToken(&self) {
        self.navigation_document_token.CommitNewDocument();
    }
    pub fn SetFrameLifecycleState(
        &self,
        state: crate::native_execution_context_lifetime::FrameLifecycleState,
    ) -> Option<crate::native_execution_context_lifetime::ExecutionContextLifecycleChange> {
        use crate::native_execution_context_lifetime::{
            ExecutionContextLifecycle as C, FrameLifecycleState as F,
        };
        self.SetExecutionContextLifecycle(match state {
            F::Running => C::Running,
            F::Paused => C::Paused,
            F::Frozen => C::Frozen,
        })
    }
    pub fn ExecutionContextLifecycle(
        &self,
    ) -> crate::native_execution_context_lifetime::ExecutionContextLifecycle {
        self.native_execution_context_lifetime.Lifecycle()
    }
    pub fn SetExecutionContextLifecycle(
        &self,
        state: crate::native_execution_context_lifetime::ExecutionContextLifecycle,
    ) -> Option<crate::native_execution_context_lifetime::ExecutionContextLifecycleChange> {
        self.native_execution_context_lifetime.SetLifecycle(state)
    }
    pub fn ExecutionContextLifecycleObservers(
        &self,
    ) -> Rc<crate::native_execution_context_lifetime::ExecutionContextLifecycleStateObservers> {
        self.native_execution_context_lifetime.LifecycleObservers()
    }
    pub fn IsContextPaused(&self) -> bool {
        self.ExecutionContextLifecycle()
            == crate::native_execution_context_lifetime::ExecutionContextLifecycle::Paused
    }
    pub fn IsContextFrozenOrPaused(&self) -> bool {
        matches!(
            self.ExecutionContextLifecycle(),
            crate::native_execution_context_lifetime::ExecutionContextLifecycle::Paused
                | crate::native_execution_context_lifetime::ExecutionContextLifecycle::Frozen
        )
    }
    pub fn IsContextDestroyed(&self) -> bool {
        self.ExecutionContextLifecycle()
            == crate::native_execution_context_lifetime::ExecutionContextLifecycle::Destroyed
    }
    pub fn NativeExecutionContextObservers(
        &self,
    ) -> Rc<crate::native_execution_context_lifetime::NativeExecutionContextObservers> {
        self.native_execution_context_lifetime.Observers()
    }
    pub fn NativeExecutionContextLifetime(&self) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
        self.native_execution_context_lifetime.ReadOnlyHandle()
    }
    pub fn Root(&self) -> usize {
        self.root
    }
    // Identity adapter for C++ &Document::Root(), which is distinct from
    // Root().OwnerDocumentNode(): a document node has no owner document.
    pub fn NodeHandle(&self, node: usize) -> DOMOwnerHandle {
        DOMOwnerHandle {
            arena_id: self.arena_id,
            node_id: self.arena[node].id,
        }
    }
    /// Source Document::documentElement/body reads immediate canonical children.
    pub fn DocumentElement(&self) -> Option<usize> {
        self.arena[self.Root()]
            .children
            .iter()
            .copied()
            .find(|&node| self.arena[node].node_type == DOMNodeType::kElement)
    }
    /// Document.cc:4112-4124. Dynamic classes are selected by the native
    /// element factory, including createElementNS in a non-HTML document.
    pub fn BodyElement(&self) -> Option<usize> {
        let element = self.DocumentElement()?;
        if !matches!(self.arena[element].class, DOMNodeClass::HTMLHtmlElement) {
            return None;
        }
        self.arena[element].children.iter().copied().find(|&node| {
            matches!(
                self.arena[node].class,
                DOMNodeClass::HTMLBodyElement | DOMNodeClass::HTMLFrameSetElement
            )
        })
    }
    /// Document.cc:3687-3691. Document kind is the parser/creation-owned fact.
    pub fn IsFrameSet(&self) -> bool {
        self.IsInHTMLDocument(self.Root())
            && self.BodyElement().is_some_and(|body| {
                matches!(self.arena[body].class, DOMNodeClass::HTMLFrameSetElement)
            })
    }

    pub fn RootHandle(&self) -> DOMOwnerHandle {
        DOMOwnerHandle {
            arena_id: self.arena_id,
            node_id: self.arena[self.root].id,
        }
    }

    /// Identity of the node's document, including an adopted external owner.
    /// A document names itself rather than its null ownerDocument.
    pub fn NodeDocumentHandle(&self, node: usize) -> DOMOwnerHandle {
        let node = &self.arena[node];
        if node.Type() == DOMNodeType::kDocument {
            DOMOwnerHandle {
                arena_id: self.arena_id,
                node_id: node.Id(),
            }
        } else {
            node.OwnerDocumentNode()
                .expect("a non-document node has an owner document")
        }
    }
    // The arena key moves with parser document swaps. Exhaustion permanently
    // disables reuse; an old cached revision can never become valid again.
    pub fn CollectionMembershipKey(&self) -> Option<(DOMOwnerHandle, u64)> {
        (self.collection_membership_revision != u64::MAX)
            .then(|| (self.RootHandle(), self.collection_membership_revision))
    }
    fn BumpCollectionMembershipRevision(&mut self) {
        self.collection_membership_revision = self.collection_membership_revision.saturating_add(1);
    }
    pub(crate) fn InvalidateElementInternalsCollections(&mut self) {
        self.BumpCollectionMembershipRevision();
    }
    pub(crate) fn StatefulCustomElementDidUpgrade(&mut self, element: usize) {
        self.stateful_form_control_revision = self
            .stateful_form_control_revision
            .checked_add(1)
            .expect("form-control invalidation revision");
        self.AttributeLifecycle()
            .RecordFormAssociatedCustomElementUpgraded(self.arena[element].id);
    }

    // cpp: LocalSVGResource::TargetChanged / SVGElement resource invalidation.
    // Collection membership also changes for every ordinary HTML attribute.
    // SVG resources depend on ID targets, connected topology, SVG contents and
    // their computed style, not unrelated HTML class/style/value edits.
    fn BumpSVGResourceDOMRevision(&mut self) {
        self.svg_resource_dom_revision = self.svg_resource_dom_revision.saturating_add(1);
    }
    fn InvalidateSVGAttribute(&mut self, element: usize, name: &str) {
        if self.IsConnectedToDocument(element)
            && (name == "id" || self.IsSVGResourceStyleNode(element))
        {
            self.BumpSVGResourceDOMRevision();
        }
    }

    // ElementData/Element's selector-filter inputs stay with the DOM owner.
    // Style owns the actual hashes and bloom filters. A typed mutation changes
    // only local attributes or the affected ancestor path; character data
    // cannot change these attribute/class facts. Unknown NodeMut edits revoke
    // the complete generation instead of publishing possibly stale revisions.
    pub fn SelectorFilterKey(&self) -> Option<(DOMOwnerHandle, u64)> {
        (self.collection_membership_revision != u64::MAX
            && self.selector_filter_generation != u64::MAX)
            .then(|| (self.RootHandle(), self.selector_filter_generation))
    }

    pub fn SelectorFilterRevisions(&self, index: usize) -> (u64, u64) {
        let node = &self.arena[index];
        (
            node.selector_identifier_revision,
            node.selector_subtree_revision,
        )
    }

    pub fn SelectorAttributeRevision(&self, index: usize) -> u64 {
        self.arena[index].selector_attribute_revision
    }

    fn InvalidateSelectorSubtree(&mut self, node: usize) {
        let revision = self.collection_membership_revision;
        let mut current = Some(node);
        while let Some(index) = current {
            self.arena[index].selector_subtree_revision = revision;
            current = self.arena[index].parent;
        }
    }

    fn InvalidateSelectorAttributes(&mut self, node: usize, name: &str, presence_changed: bool) {
        self.arena[node].selector_attribute_revision = self.collection_membership_revision;
        // cpp: Element::AppendAttributeInternal/ClassAttributeChanged and
        // SelectorFilter::CollectElementIdentifierHashes. Ordinary attribute
        // values do not enter identifier hashes or subtree filters; style is
        // excluded entirely. Attribute-value matching still observes every
        // change through its separate local revision above.
        if name != "style" && (presence_changed || matches!(name, "id" | "class")) {
            self.arena[node].selector_identifier_revision = self.collection_membership_revision;
            self.InvalidateSelectorSubtree(node);
        }
    }

    /// Opaque, document-qualified OriginatingTreeScope identity. SVG use
    /// instances resolve local resources in their corresponding element's
    /// scope; generated pseudo styles are supplied their originating index.
    pub fn SVGResourceScope(
        &self,
        index: usize,
    ) -> layoutng_style::style::reference_clip_path_operation::SvgClipResourceScope {
        let origin = self.CorrespondingSVGElement(index).unwrap_or(index);
        layoutng_style::style::reference_clip_path_operation::SvgClipResourceScope {
            document_id: self.arena_id,
            tree_scope_id: self.arena[self.TreeScopeRoot(origin)].id,
        }
    }

    fn SVGResourceInputKey(&self) -> Option<(u64, u64)> {
        (self.svg_resource_dom_revision != u64::MAX && self.svg_resource_style_revision != u64::MAX)
            .then_some((
                self.svg_resource_dom_revision,
                self.svg_resource_style_revision,
            ))
    }

    /// Document's source parser profile, set by the sanitizer document constructor.
    pub fn IsForMarkupSanitization(&self) -> bool {
        self.is_for_markup_sanitization
    }
    pub fn SetIsForMarkupSanitization(&mut self) {
        self.is_for_markup_sanitization = true;
    }
    /// A targeted style update may use the catalog only after the document's
    /// current DOM and computed styles were observed by the resource lifecycle.
    pub fn DocumentURL(&self) -> &str {
        &self.document_url
    }

    /// Original Document UkmRecorder output. Records remain with this Document
    /// identity until the metrics consumer drains them; no Page callback runs.
    pub fn RecordTappedBadTarget(&self, record: DocumentTappedBadTarget) {
        self.mobile_metrics.borrow_mut().push(record);
    }
    pub fn TakeTappedBadTargets(&self) -> Vec<DocumentTappedBadTarget> {
        self.mobile_metrics.borrow_mut().drain(..).collect()
    }

    pub fn HasNavigationSecurityContext(&self) -> bool {
        self.navigation_security_initialized
    }
    // Document.cc:4068-4082, document.h anchor_element_interaction_tracker_.
    // Same arena lifetime; native callers clone the identity, never its state.
    pub(crate) fn ClearAnchorNavigationContextLifetime(&self) {
        self.anchor_navigation_context_lifetime.borrow_mut().take();
    }
    pub fn InstallAnchorNavigationContextLifetime(&self, lifetime: Rc<dyn std::any::Any>) {
        let previous = self
            .anchor_navigation_context_lifetime
            .borrow_mut()
            .replace(lifetime);
        assert!(
            previous.is_none(),
            "one canonical navigation execution context"
        );
    }
    pub fn AnchorElementInteractionTracker(&self) -> Option<Rc<dyn std::any::Any>> {
        self.anchor_element_interaction_tracker.borrow().clone()
    }
    pub fn AnnotationAgentContainer(&self) -> Option<Rc<dyn std::any::Any>> {
        self.annotation_agent_container.borrow().clone()
    }
    pub fn InstallAnnotationAgentContainer(&self, owner: Rc<dyn std::any::Any>) {
        assert!(
            self.annotation_agent_container
                .borrow_mut()
                .replace(owner)
                .is_none(),
            "one AnnotationAgentContainer supplement per Document"
        );
    }
    pub fn InstallAnchorElementInteractionTracker(&self, owner: Rc<dyn std::any::Any>) {
        let previous = self
            .anchor_element_interaction_tracker
            .borrow_mut()
            .replace(owner);
        assert!(
            previous.is_none(),
            "Document anchor tracker constructs once"
        );
    }
    /// Browser composition admits the original StoragePartition resource
    /// service before Window/loader registration. It contains no foreign DOM.
    pub fn InstallBlobURLRegistry(
        &mut self,
        registry: std::sync::Arc<fileapi::blob_url_registry::BlobURLRegistry>,
    ) {
        self.security_context.InstallBlobURLRegistry(registry);
    }
    /// A new Document in the same Page/StoragePartition keeps the original
    /// blob URL service without exposing the registry outside DOM.
    pub fn InheritBlobURLRegistryFrom(&mut self, previous: &PersistentDocument) {
        self.security_context
            .InstallBlobURLRegistry(previous.security_context.PublicURLManager().Registry());
    }
    pub fn SetFrameAdEvidence(&self, evidence: crate::frame_ad_evidence::FrameAdEvidence) {
        assert!(evidence.IsComplete());
        let mut old = self.frame_ad_evidence.borrow_mut();
        assert!(old.as_ref().is_none_or(|old| evidence.CanReplace(old)));
        *old = Some(evidence);
    }
    pub fn IsAdFrame(&self) -> bool {
        self.frame_ad_evidence
            .borrow()
            .as_ref()
            .is_some_and(|evidence| evidence.IndicatesAdFrame())
    }
    pub fn SecurityContext(&self) -> &SecurityContext {
        &self.security_context
    }
    /// Navigation's actual response endpoint fact, produced by the native
    /// network service; never inferred from the Document URL or root element.
    pub fn SetResponseAddressSpace(
        &mut self,
        space: foundation::network_address_space::IPAddressSpace,
    ) {
        self.security_context.SetAddressSpace(space);
    }
    pub fn ExecutionContextIsolation(&self) -> &ExecutionContextIsolation {
        &self.execution_context_isolation
    }
    pub fn InitializeSecurityContextForNavigation(
        &mut self,
        origin: Rc<SecurityOrigin>,
        has_insecure_ancestor: bool,
    ) {
        self.content_security_policy
            .BindSecurityOrigin(origin.as_ref());
        self.security_context = SecurityContext::ForNavigationInStoragePartition(
            origin,
            has_insecure_ancestor,
            self.security_context.PublicURLManager().Registry(),
        );
        self.execution_context_isolation = ExecutionContextIsolation::ForNonIsolatedAgentCluster();
        self.navigation_security_initialized = true;
    }
    /// Loader CSP application before parser/script entry. The Window's origin
    /// is sandboxed once; later additions cannot replace its opaque identity.
    pub fn ApplyResponsePolicySandbox(&mut self) {
        let flags = self.content_security_policy.SandboxMask();
        let old = self.security_context.SandboxFlags();
        if flags.Contains(foundation::security::SandboxFlags::ORIGIN)
            && !old.Contains(foundation::security::SandboxFlags::ORIGIN)
        {
            let trustworthy = self.security_context.Origin().IsPotentiallyTrustworthy();
            let origin = self.security_context.Origin().DeriveNewOpaqueOrigin();
            if trustworthy {
                origin.SetOpaqueOriginIsPotentiallyTrustworthy(true);
            }
            let address_space = self.security_context.AddressSpace();
            self.security_context = SecurityContext::ForNavigationInStoragePartition(
                origin,
                self.security_context.HasInsecureAncestor(),
                self.security_context.PublicURLManager().Registry(),
            );
            self.security_context.SetAddressSpace(address_space);
            self.content_security_policy
                .BindSecurityOrigin(self.security_context.Origin());
        }
        self.security_context
            .SetSandboxFlags(foundation::security::SandboxFlags(old.0 | flags.0));
    }
    pub fn SetDocumentURL(&mut self, url: String) {
        let url = if url.is_empty() {
            String::from("about:blank")
        } else {
            url
        };
        if url == self.document_url {
            return;
        }
        let url = self.fragment_directive.ConsumeFragmentDirective(&url);
        self.content_security_policy.UpdateDocumentURL(&url);
        self.document_url = url;
        self.UpdateBaseURL();
    }
    pub fn FragmentDirectiveState(&self) -> Rc<foundation::fragment_directive::FragmentDirective> {
        self.fragment_directive.clone()
    }

    pub fn SVGResources(
        &self,
    ) -> Option<&std::sync::Arc<layoutng_assembly::internal::paint_input::SvgClipResourceSnapshot>>
    {
        let inputs = self.SVGResourceInputKey()?;
        (self.svg_clip_resource_inputs == Some(inputs))
            .then_some(self.svg_clip_resources.as_ref())
            .flatten()
    }

    pub(crate) fn SVGFilterGraphs(
        &self,
    ) -> Option<&std::sync::Arc<Vec<layoutng_assembly::internal::paint_input::SvgFilterGraph>>>
    {
        self.SVGResources()?;
        Some(&self.svg_filter_graphs)
    }

    pub(crate) fn PublishSVGResources(
        &mut self,
        mut snapshot: std::sync::Arc<
            layoutng_assembly::internal::paint_input::SvgClipResourceSnapshot,
        >,
        filters: std::sync::Arc<Vec<layoutng_assembly::internal::paint_input::SvgFilterGraph>>,
    ) -> bool {
        // cpp: SVGFilterElement::PrimitiveAttributeChanged/InvalidateFilterChain.
        // Filter content participates in the same document resource lifecycle;
        // equal clip entries cannot hide a changed filter primitive or style.
        let filters_changed = self.svg_filter_graphs != filters;
        if filters_changed {
            self.svg_filter_graphs = filters;
        }
        let inputs_changed = self.svg_clip_resource_inputs != self.SVGResourceInputKey();
        self.svg_clip_resource_inputs = self.SVGResourceInputKey();
        if !inputs_changed
            && self
                .svg_clip_resources
                .as_ref()
                .is_some_and(|old| old.entries == snapshot.entries)
        {
            return filters_changed;
        }
        let revision = self.svg_clip_resources.as_ref().map_or(1, |old| {
            old.version
                .revision
                .checked_add(1)
                .expect("SVG resource revision exhausted")
        });
        std::sync::Arc::make_mut(&mut snapshot).version =
            layoutng_assembly::internal::paint_input::SvgClipResourceVersion {
                document_id: self.arena_id,
                revision,
            };
        self.svg_clip_resources = Some(snapshot);
        true
    }
    pub fn GetCompatibilityMode(&self) -> CompatibilityMode {
        self.compatibility
    }

    // cpp: settings.json5 TextAreasAreResizable; Settings getter/setter.
    /// Execution-context setting used by HTMLNoScriptElement::LayoutObjectIsNeeded.
    pub fn ScriptingEnabled(&self) -> bool {
        self.scripting_enabled
    }
    pub fn SetScriptingEnabled(&mut self, value: bool) {
        self.scripting_enabled = value;
    }
    pub fn TextAreasAreResizable(&self) -> bool {
        self.text_areas_are_resizable
    }
    pub fn SetTextAreasAreResizable(&mut self, value: bool) {
        self.text_areas_are_resizable = value;
    }
    pub fn DefaultFontSize(&self) -> f32 {
        self.default_font_size
    }
    pub fn SetDefaultFontSize(&mut self, value: f32) {
        // Settings stores the requested value verbatim.  The system-font
        // provider applies its zero/uninitialised fallback and minimum when it
        // resolves the pending value, not when the setting changes.
        self.default_font_size = value;
    }
    pub fn DefaultFixedFontSize(&self) -> f32 {
        self.default_fixed_font_size
    }
    pub fn SetDefaultFixedFontSize(&mut self, value: f32) {
        self.default_fixed_font_size = value;
    }
    pub fn SetCompatibilityMode(&mut self, mode: CompatibilityMode) {
        self.compatibility = mode;
    }

    pub fn ViewportStyle(
        &self,
    ) -> Option<&foundation::Persistent<layoutng_style::style::computed_style::ComputedStyle>> {
        self.viewport_style.as_ref()
    }
    pub fn SetViewportStyle(
        &mut self,
        style: foundation::Persistent<layoutng_style::style::computed_style::ComputedStyle>,
    ) {
        if self
            .viewport_style
            .as_ref()
            .is_some_and(|old| unsafe { &*old.Get() } == unsafe { &*style.Get() })
        {
            return;
        }
        self.viewport_style = Some(style);
        self.RecordLayoutInputChange(self.Root());
    }

    pub fn DocumentType(&self, document: usize) -> DOMDocumentType {
        assert_eq!(self.arena[document].Type(), DOMNodeType::kDocument);
        self.document_types
            .get(&self.arena[document].id)
            .copied()
            .unwrap_or(DOMDocumentType::kXML)
    }

    pub fn SetDocumentType(&mut self, document: usize, kind: DOMDocumentType) {
        assert_eq!(self.arena[document].Type(), DOMNodeType::kDocument);
        if self.DocumentType(document) == kind {
            return;
        }
        self.document_types.insert(self.arena[document].id, kind);
        self.InvalidateAllStyles();
    }

    pub fn IsInHTMLDocument(&self, node: usize) -> bool {
        let document_id = if self.arena[node].Type() == DOMNodeType::kDocument {
            self.arena[node].id
        } else {
            let Some(owner) = self.arena[node].OwnerDocumentNode() else {
                return false;
            };
            if owner.arena_id != self.arena_id {
                return false;
            }
            owner.node_id
        };
        self.document_types.get(&document_id) == Some(&DOMDocumentType::kHTML)
    }

    // Chromium element.cc:11663-11668; html_parser_idioms.cc:142-172.
    // HTML integer parsing accepts a leading plus and trailing non-digits,
    // while an absent/invalid/out-of-range attribute keeps its default.
    pub fn IntegralAttribute(&self, element: usize, name: &str) -> Option<i32> {
        let value = &self.Node(element).FindAttribute(name)?.value;
        let bytes = value.as_bytes();
        let mut index = 0;
        while bytes
            .get(index)
            .is_some_and(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b'\x0c'))
        {
            index += 1;
        }
        let start = index;
        if bytes
            .get(index)
            .is_some_and(|byte| matches!(byte, b'+' | b'-'))
        {
            index += 1;
        }
        let digits = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == digits {
            return None;
        }
        value[start..index].parse().ok()
    }
    pub fn Node(&self, index: usize) -> &DOMNode {
        &self.arena[index]
    }
    // cpp: html_input_element.h:305; html_input_element.cc:1150-1152.
    pub fn InputType(&self, element: usize) -> Option<DOMInputType> {
        self.arena[element]
            .IsElement("input", DOMNamespace::kHTML)
            .then(|| {
                DOMInputType::FromTypeAttribute(
                    self.arena[element]
                        .attributes
                        .iter()
                        .find(|attribute| {
                            attribute.local_name == "type" && attribute.namespace_uri.is_empty()
                        })
                        .map(|attribute| attribute.value.as_str()),
                )
            })
    }

    // cpp: html_select_element.cc:219-235. HTMLSelectElement retains this
    // state and SelectorChecker consumes UsesMenuList(); keeping the query in
    // DOM preserves that owner across the StyleEngine package boundary.
    pub fn SelectUsesMenuList(&self, element: usize) -> bool {
        matches!(&self.arena[element].class, DOMNodeClass::HTMLSelectElement(data) if data.uses_menu_list)
    }

    // cpp: html_select_element.cc:2155-2195;
    //      html_option_element.h:87-89,219-229. The association follows the
    // DOM ancestor chain and stops at the same related-part barriers used by
    // HTMLSelectElement::WalkAncestorsForRelatedParts.
    pub fn OptionOwnerSelect(&self, option: usize) -> Option<usize> {
        matches!(
            self.arena[option].class,
            DOMNodeClass::HTMLOptionElement { .. }
        )
        .then_some(self.arena[option].select_owner)
        .flatten()
    }
    // html_opt_group_element.cc:197-204. Read the same owning class member.
    pub fn OptGroupOwnerSelect(&self, optgroup: usize) -> Option<usize> {
        matches!(
            self.arena[optgroup].class,
            DOMNodeClass::HTMLOptGroupElement
        )
        .then_some(self.arena[optgroup].select_owner)
        .flatten()
    }

    // cpp: html_select_element.h:289; html_select_element.cc GetOptionList().
    // Retain source tree order while using OptionOwnerSelect as the single
    // association owner for nested customizable-select markup.
    pub fn SelectOptionNodeIds(&self, select: usize) -> Vec<u64> {
        self.SelectCollection(select).OptionNodeIds().to_vec()
    }

    // cpp: core/dom/node.cc Node::isEqualNode. Compare DOM children, never
    // attached shadow roots, listeners, live control state or layout state.
    pub fn IsEqualNode(&self, node: usize, other: &Self, other_node: usize) -> bool {
        let mut pending = vec![(node, other_node)];
        while let Some((left, right)) = pending.pop() {
            let left = self.Node(left);
            let right = other.Node(right);
            if left.Type() != right.Type()
                || left.Name() != right.Name()
                || left.Namespace() != right.Namespace()
                || left.Data() != right.Data()
                || left.Children().len() != right.Children().len()
                || left.Attributes().len() != right.Attributes().len()
                || !left.Attributes().iter().all(|a| {
                    right.Attributes().iter().any(|b| {
                        a.local_name == b.local_name
                            && a.namespace_uri == b.namespace_uri
                            && a.value == b.value
                    })
                })
            {
                return false;
            }
            pending.extend(
                left.Children()
                    .iter()
                    .copied()
                    .zip(right.Children().iter().copied()),
            );
        }
        true
    }

    // cpp: html_input_element.cc:2064-2075. The pseudo class observes the
    // actual datalist association, not merely a non-empty `list` attribute.
    pub fn InputHasDataList(&self, element: usize) -> bool {
        self.InputDataList(element).is_some()
    }

    // cpp: html_parser_idioms.cc:174-220. Select's cached size_ uses the HTML
    // non-negative-integer algorithm, including leading HTML whitespace,
    // leading plus, minus-zero and accepted trailing garbage.
    fn ParseHTMLNonNegativeInteger(value: &str) -> Option<u32> {
        let value = value
            .trim_start_matches(|character| matches!(character, '\t' | '\n' | '\x0c' | '\r' | ' '));
        let (negative, value) = match value.as_bytes().first() {
            Some(b'+') => (false, &value[1..]),
            Some(b'-') => (true, &value[1..]),
            _ => (false, value),
        };
        let digits = value
            .as_bytes()
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        if digits == 0 {
            return None;
        }
        let parsed = value[..digits].parse::<u32>().ok()?;
        (!negative || parsed == 0).then_some(parsed)
    }

    pub fn TreeScopeContainsMultipleElementsWithId(&self, scope: usize, id: &str) -> bool {
        !id.is_empty()
            && self
                .tree_scope_elements_by_id
                .get(&scope)
                .and_then(|ids| ids.get(id))
                .is_some_and(|elements| elements.len() > 1)
    }

    pub fn ElementsByIdInTreeScope(&self, scope: usize, id: &str) -> Vec<usize> {
        let Some(elements) = self
            .tree_scope_elements_by_id
            .get(&scope)
            .and_then(|ids| ids.get(id))
            .filter(|elements| !elements.is_empty())
        else {
            return Vec::new();
        };
        if elements.len() == 1 {
            return elements.iter().copied().collect();
        }
        let mut ordered = Vec::with_capacity(elements.len());
        let mut pending = vec![scope];
        while let Some(node) = pending.pop() {
            if elements.contains(&node) {
                ordered.push(node);
            }
            pending.extend(self.arena[node].children.iter().rev().copied());
        }
        ordered
    }

    pub fn FindElementByIdInTreeScope(&self, element: usize, id: &str) -> Option<usize> {
        let scope = self.TreeScopeRoot(element);
        if scope == self.root || self.shadow_hosts.contains_key(&scope) {
            return self.ElementsByIdInTreeScope(scope, id).into_iter().next();
        }
        fn find(document: &PersistentDocument, node: usize, id: &str) -> Option<usize> {
            if document.arena[node].node_type == DOMNodeType::kElement
                && document.arena[node]
                    .FindAttribute("id")
                    .is_some_and(|attribute| attribute.value == id)
            {
                return Some(node);
            }
            document.arena[node]
                .children
                .iter()
                .find_map(|&child| find(document, child, id))
        }
        find(self, scope, id)
    }

    /// Returns the last access-key match in TreeScope traversal order. Shadow
    /// roots are searched immediately after their host and may replace an
    /// earlier light-tree result, matching TreeScope's recursive source walk.
    // Source: third_party/blink/renderer/core/dom/tree_scope.cc:880-896.
    pub fn GetElementByAccessKey(&self, scope: usize, key: &[u16]) -> Option<usize> {
        if key.is_empty() {
            return None;
        }
        let key = foundation::String::from_utf16(key);
        fn visit(
            document: &PersistentDocument,
            parent: usize,
            key: &foundation::String,
            result: &mut Option<usize>,
        ) {
            for &node in document.arena[parent].children.iter() {
                if document.arena[node].node_type == DOMNodeType::kElement {
                    if document.arena[node]
                        .FindAttribute("accesskey")
                        .is_some_and(|attribute| {
                            foundation::String::from(attribute.value.as_str())
                                .DeprecatedEqualIgnoringCase(key)
                        })
                    {
                        *result = Some(node);
                    }
                    if let Some(shadow_root) = document.ShadowRoot(node) {
                        visit(document, shadow_root, key, result);
                    }
                }
                visit(document, node, key, result);
            }
        }
        let mut result = None;
        visit(self, scope, &key, &mut result);
        result
    }

    pub fn IsLabelableElement(&self, element: usize) -> bool {
        let node = &self.arena[element];
        if node.Namespace() != DOMNamespace::kHTML {
            return false;
        }
        match node.Name() {
            "button" | "meter" | "output" | "progress" | "select" | "textarea" => true,
            "input" => self.InputType(element) != Some(DOMInputType::kHidden),
            _ => self.IsFormAssociatedCustomElement(element),
        }
    }

    // cpp: html_label_element.cc:81-113. The explicit `for` association is
    // resolved in the label's TreeScope; without it the first labelable
    // descendant in tree order owns the association.
    pub fn LabelControl(&self, label: usize) -> Option<usize> {
        if !self.arena[label].IsHTMLElement("label") {
            return None;
        }
        if let Some(target) = self.arena[label].FindAttribute("for") {
            return self
                .FindElementByIdInTreeScope(label, &target.value)
                .filter(|&element| self.IsLabelableElement(element));
        }
        fn first(document: &PersistentDocument, node: usize) -> Option<usize> {
            for &child in &document.arena[node].children {
                if document.arena[child].node_type == DOMNodeType::kElement
                    && document.IsLabelableElement(child)
                {
                    return Some(child);
                }
                if let Some(element) = first(document, child) {
                    return Some(element);
                }
            }
            None
        }
        first(self, label)
    }

    // cpp: html_input_element.cc:1869-1871; html_select_element.cc:651-653;
    //      html_text_area_element.h:117-120; html_button_element.h:113.
    pub fn IsRequiredFormControl(&self, element: usize) -> bool {
        let has_required = self.arena[element].attributes.iter().any(|attribute| {
            attribute.local_name == "required" && attribute.namespace_uri.is_empty()
        });
        if let Some(input_type) = self.InputType(element) {
            return input_type.SupportsRequired() && has_required;
        }
        (self.arena[element].IsElement("select", DOMNamespace::kHTML)
            || self.arena[element].IsElement("textarea", DOMNamespace::kHTML))
            && has_required
    }

    pub fn IsOptionalFormControl(&self, element: usize) -> bool {
        let supports_optional = self.InputType(element).is_some()
            || self.arena[element].IsElement("select", DOMNamespace::kHTML)
            || self.arena[element].IsElement("textarea", DOMNamespace::kHTML)
            || self.arena[element].IsElement("button", DOMNamespace::kHTML);
        supports_optional && !self.IsRequiredFormControl(element)
    }

    // cpp: text_control_element.cc:265-269.
    pub fn TextControlPlaceholderIsVisible(&self, element: usize) -> bool {
        let supports_placeholder = if self.arena[element].IsElement("textarea", DOMNamespace::kHTML)
        {
            true
        } else {
            self.InputType(element)
                .is_some_and(DOMInputType::SupportsPlaceholder)
        };
        supports_placeholder
            && self
                .SuggestedValue(element)
                .is_none_or(|suggested| suggested.is_empty())
            && self.arena[element].attributes.iter().any(|attribute| {
                attribute.local_name == "placeholder" && attribute.namespace_uri.is_empty()
            })
            && self.ControlValue(element).is_empty()
    }

    fn HasBooleanHTMLAttribute(&self, element: usize, name: &str) -> bool {
        self.arena[element]
            .attributes
            .iter()
            .any(|attribute| attribute.local_name == name && attribute.namespace_uri.is_empty())
    }

    fn FirstLegendChild(&self, fieldset: usize) -> Option<usize> {
        self.arena[fieldset]
            .children
            .iter()
            .copied()
            .find(|&child| self.arena[child].IsElement("legend", DOMNamespace::kHTML))
    }

    // cpp: listed_element.cc:693-721. The last legend seen while walking
    // ancestors is the legend that can exempt this control from a disabled
    // fieldset, and only when it is that fieldset's first legend child.
    fn IsDisabledByFieldsetAncestor(&self, element: usize) -> bool {
        let mut last_legend_ancestor = None;
        let mut ancestor = self.arena[element].parent;
        while let Some(current) = ancestor {
            if self.arena[current].IsElement("legend", DOMNamespace::kHTML) {
                last_legend_ancestor = Some(current);
            } else if self.arena[current].IsElement("fieldset", DOMNamespace::kHTML)
                && self.HasBooleanHTMLAttribute(current, "disabled")
            {
                if last_legend_ancestor == self.FirstLegendChild(current) {
                    ancestor = self.arena[current].parent;
                    continue;
                }
                return true;
            }
            ancestor = self.arena[current].parent;
        }
        false
    }

    fn IsActuallyDisabledFormControl(&self, element: usize) -> bool {
        self.HasBooleanHTMLAttribute(element, "disabled")
            || self.IsDisabledByFieldsetAncestor(element)
    }

    // Source virtual Element::IsDisabledFormControl: form_control.cc:313-334;
    // option.cc:537-542, opt_group.cc:66-68; fieldset/output override false.
    pub fn IsDisabledFormControl(&self, element: usize) -> bool {
        if self.ElementInternalsTargetIsFormAssociated(element) {
            return self.IsActuallyDisabledFormControl(element);
        }
        match &self.arena[element].class {
            DOMNodeClass::HTMLOptionElement { .. } => {
                self.HasBooleanHTMLAttribute(element, "disabled")
                    || self
                        .OptionOwnerOptgroup(element)
                        .is_some_and(|group| self.IsDisabledFormControl(group))
            }
            DOMNodeClass::HTMLOptGroupElement => self.HasBooleanHTMLAttribute(element, "disabled"),
            DOMNodeClass::HTMLButtonElement
            | DOMNodeClass::HTMLInputElement
            | DOMNodeClass::MediaControlInput { .. }
            | DOMNodeClass::HTMLSelectElement(_)
            | DOMNodeClass::HTMLTextAreaElement => self.IsActuallyDisabledFormControl(element),
            _ => false,
        }
    }

    fn NearestAncestorElementNamed(&self, element: usize, name: &str) -> Option<usize> {
        let mut ancestor = self.arena[element].parent;
        while let Some(current) = ancestor {
            if self.arena[current].IsElement(name, DOMNamespace::kHTML) {
                return Some(current);
            }
            ancestor = self.arena[current].parent;
        }
        None
    }

    // cpp: html_form_control_element.cc:313-338;
    //      html_field_set_element.cc:240-243,278-285;
    //      html_option_element.cc:225-243,537-542;
    //      html_opt_group_element.cc:66-68,94-111.
    pub fn MatchesDisabledPseudoClass(&self, element: usize) -> bool {
        if self.IsFormAssociatedCustomElement(element) {
            return self.IsActuallyDisabledFormControl(element);
        }
        let node = &self.arena[element];
        if matches!(node.class, DOMNodeClass::HTMLOptionElement { .. }) {
            return self.IsDisabledFormControl(element)
                || self
                    .OptionOwnerSelect(element)
                    .is_some_and(|select| self.IsDisabledFormControl(select));
        }
        if node.IsElement("optgroup", DOMNamespace::kHTML) {
            if self.HasBooleanHTMLAttribute(element, "disabled") {
                return true;
            }
            return self
                .NearestAncestorElementNamed(element, "select")
                .is_some_and(|select| self.IsActuallyDisabledFormControl(select));
        }
        if matches!(
            node.Name(),
            "button" | "fieldset" | "input" | "select" | "textarea"
        ) && node.Namespace() == DOMNamespace::kHTML
        {
            return self.IsActuallyDisabledFormControl(element);
        }
        false
    }

    pub fn MatchesEnabledPseudoClass(&self, element: usize) -> bool {
        let node = &self.arena[element];
        let supports_state = node.Namespace() == DOMNamespace::kHTML
            && matches!(
                node.Name(),
                "button" | "fieldset" | "input" | "select" | "textarea" | "option" | "optgroup"
            );
        supports_state && !self.MatchesDisabledPseudoClass(element)
    }
    // Index-to-reference adapter for source callbacks receiving DOMNode&.
    pub fn NodeMut(&mut self, index: usize) -> &mut DOMNode {
        self.InvalidateTableRowCells(index);
        if let Some(parent) = self.arena[index].parent {
            self.InvalidateTableRowCells(parent);
        }
        // This escape hatch can replace the whole node, including its tag.
        self.form_observer_elements = None;
        self.BumpCollectionMembershipRevision();
        self.BumpSVGResourceDOMRevision();
        self.selector_filter_generation = self.selector_filter_generation.saturating_add(1);
        self.InvalidateNodeStyle(index);
        self.MarkStyleElementTextDirty(index);
        if self.IsConnectedToDocument(index) {
            self.style_state.layout_inputs_require_tree = true;
            self.style_state
                .impact
                .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        }
        &mut self.arena[index]
    }
    // cpp: dom/document.cc:11-22
    // cpp: dom/document.cc:41-51
    fn Allocate(
        &mut self,
        node_type: DOMNodeType,
        namespace: DOMNamespace,
        name: String,
        data: String,
        attributes: Vec<DOMAttribute>,
    ) -> usize {
        self.BumpCollectionMembershipRevision();
        let index = self.arena.len();
        let id = self.next_id;
        self.next_id += 1;
        let owner_document_handle = (node_type != DOMNodeType::kDocument).then(|| DOMOwnerHandle {
            arena_id: self.arena_id,
            node_id: self.arena[self.root].id,
        });
        let namespace_uri = match namespace {
            DOMNamespace::kHTML => "http://www.w3.org/1999/xhtml",
            DOMNamespace::kSVG => "http://www.w3.org/2000/svg",
            DOMNamespace::kMathML => "http://www.w3.org/1998/Math/MathML",
            DOMNamespace::kNone => "",
        };
        let tag_qname = foundation::QualifiedName::new(
            foundation::AtomicString::default(),
            foundation::AtomicString::from_str(&name),
            if namespace_uri.is_empty() {
                foundation::AtomicString::default()
            } else {
                foundation::AtomicString::from_str(namespace_uri)
            },
        );
        let mut class = DOMNodeClass::Allocate(node_type, namespace, &name);
        if let Some(relations) = class.AnchorLinkRelationsMut() {
            *relations = foundation::anchor_element_utils::ParseRelAttribute(
                attributes
                    .iter()
                    .find(|a| a.namespace_uri.is_empty() && a.local_name == "rel")
                    .map_or("", |a| a.value.as_str()),
            );
        }
        let name_utf16: Rc<[u16]> = name.encode_utf16().collect::<Vec<_>>().into();
        let tag_qname_utf16 = DOMQualifiedNameUTF16 {
            prefix: Rc::from([]),
            local_name: name_utf16.clone(),
            namespace_uri: namespace_uri.encode_utf16().collect::<Vec<_>>().into(),
        };
        let attribute_utf16 = attributes
            .iter()
            .map(DOMAttributeUTF16::FromAttribute)
            .collect();
        self.character_data_utf16
            .insert(id, data.encode_utf16().collect::<Vec<_>>().into());
        self.arena.push(DOMNode {
            id,
            node_type,
            class,
            svg_element_kind: (node_type == DOMNodeType::kElement
                && namespace == DOMNamespace::kSVG)
                .then(|| svg_element_kind::SVGElementKind::FromFactoryName(&name)),
            resize_observer: RefCell::new(None),
            intersection_observer: RefCell::new(None),
            select_related_parts: None,
            content_frame: None,
            frame_element_loading: None,
            lazy_frame_observer: None,
            connected_subframe_count: 0,
            has_custom_style_callbacks: false,
            is_in_canvas_subtree: false,
            select_owner: None,
            edit_context: None,
            display_lock_context: RefCell::new(None),
            scroll_marker_group_data: None,
            focusgroup: None,
            overscroll: None,
            has_undo_stack: std::cell::Cell::new(false),
            namespace,
            tag_qname,
            tag_qname_utf16,
            name_utf16,
            attribute_utf16,
            is_value: None,
            serialization_attribute_values: RefCell::new(None),
            name,
            data,
            attributes,
            image_loader: RefCell::new(None),
            html_image_is_lcp_element: Cell::new(false),
            html_image_layout_disposition: Cell::new(
                html_image_element::HTMLImageLayoutDisposition::Primary,
            ),
            inline_style: RefCell::new(None),
            svg_marker_lengths: RefCell::new(None),
            resolved_style: Default::default(),
            region_capture_crop_id: None,
            tracked_element_sub_rects: Vec::new(),
            owner_document_handle,
            tree_scope_root: if node_type == DOMNodeType::kDocument {
                index
            } else {
                self.root
            },
            parent: None,
            previous_sibling: None,
            next_sibling: None,
            children: Vec::new(),
            selector_attribute_revision: self.collection_membership_revision,
            selector_identifier_revision: self.collection_membership_revision,
            selector_subtree_revision: self.collection_membership_revision,
        });
        if node_type == DOMNodeType::kDocument {
            self.document_metadata.entry(id).or_default();
        }
        self.NotifyNodeReference(crate::node_references::NodeReferenceChange::Object {
            id,
            document_root: index == self.root && node_type == DOMNodeType::kDocument,
        });
        if let Some(owner) = owner_document_handle {
            self.NotifyNodeReference(crate::node_references::NodeReferenceChange::OwnerDocument {
                node: id,
                document: owner.node_id,
            });
        }
        if self.arena[index].IsHTMLElement("input") || self.arena[index].IsHTMLElement("button") {
            if let Some(elements) = self.form_observer_elements.as_mut() {
                elements.push(index);
            }
        }
        if self.arena[index].IsHTMLElement("canvas") {
            self.CanvasElementState(index);
        }
        if self.IsMediaElement(index) {
            self.ConstructMediaElementState(index);
            self.AttributeLifecycle().RecordMediaInitialized(id);
        }
        self.InitializeHTMLSourceElement(index);
        self.InitializeFrameElementLoading(index);
        if self.FormControlClass(index).is_some() {
            self.SetControlNeedsValidityCheck(index);
        }
        index
    }

    // cpp: dom/document.cc:53-58
    pub fn AdoptNode(&mut self, node: usize, document_node: usize) {
        self.SetTreeScopeForSubtree(node, document_node);
        self.AdoptNodeByHandle(
            node,
            DOMOwnerHandle {
                arena_id: self.arena_id,
                node_id: self.arena[document_node].id,
            },
        );
    }

    pub fn AdoptNodeByHandle(&mut self, node: usize, owner: DOMOwnerHandle) {
        self.BumpCollectionMembershipRevision();
        if self.arena[node].node_type == DOMNodeType::kDocument {
            return;
        }
        if owner.arena_id == self.arena_id {
            let index = self
                .FindNodeById(owner.node_id)
                .expect("owner document must exist in the local arena");
            assert_eq!(self.arena[index].node_type, DOMNodeType::kDocument);
        }
        let old_policy_owner_document = self.arena[node].OwnerDocumentNode();
        let owner_changed = self.arena[node].owner_document_handle != Some(owner);
        if owner_changed && self.IsConnectedToDocument(node) {
            self.BumpSVGResourceDOMRevision();
        }
        self.arena[node].owner_document_handle = Some(owner);
        if owner_changed {
            if self.arena[node].IsHTMLElement("iframe") {
                self.IFrameDocumentPolicyOwnerChanged(node, old_policy_owner_document);
            }
            if let Some(loader) = self.ExistingHTMLImageLoader(node) {
                loader.ElementDidMoveToNewDocument(self.LoadEventDelayCounterForDocument(owner));
            }
            self.InitializeHTMLSourceElement(node);
        }
        if owner_changed && owner.arena_id == self.arena_id {
            self.NotifyNodeReference(crate::node_references::NodeReferenceChange::OwnerDocument {
                node: self.arena[node].id,
                document: owner.node_id,
            });
        }
        let children = self.arena[node].children.clone();
        for child in children {
            self.AdoptNodeByHandle(child, owner);
        }
        if let Some(contents) = self.TemplateContents(node) {
            self.AdoptNodeByHandle(contents, owner);
        }
    }

    // cpp: dom/document.cc:60-68
    pub fn CreateHTMLDocument(&mut self, title: Option<String>) -> usize {
        let root = self.Allocate(
            DOMNodeType::kDocument,
            DOMNamespace::kNone,
            "#document".into(),
            String::new(),
            Vec::new(),
        );
        self.document_types
            .insert(self.arena[root].id, DOMDocumentType::kHTML);
        let doctype = self.CreateDocumentType("html".into(), String::new(), String::new());
        self.AppendChild(root, doctype);
        let html = self.CreateElement(DOMNamespace::kHTML, "html".into(), Vec::new());
        self.AppendChild(root, html);
        let head = self.CreateElement(DOMNamespace::kHTML, "head".into(), Vec::new());
        self.AppendChild(html, head);
        if let Some(title) = title {
            let element = self.CreateElement(DOMNamespace::kHTML, "title".into(), Vec::new());
            self.AppendChild(head, element);
            self.AppendText(element, &title);
        }
        let body = self.CreateElement(DOMNamespace::kHTML, "body".into(), Vec::new());
        self.AppendChild(html, body);
        root
    }

    // cpp: dom/document.cc:70-80
    pub fn CreateDocumentType(
        &mut self,
        name: String,
        public_identifier: String,
        system_identifier: String,
    ) -> usize {
        let identifiers = vec![
            DOMAttribute {
                local_name: "public".into(),
                value: public_identifier,
                ..Default::default()
            },
            DOMAttribute {
                local_name: "system".into(),
                value: system_identifier,
                ..Default::default()
            },
        ];
        self.Allocate(
            DOMNodeType::kDocumentType,
            DOMNamespace::kNone,
            name,
            String::new(),
            identifiers,
        )
    }

    // cpp: dom/document.h:95-97
    pub fn CreateDocumentTypeDefault(&mut self, name: String) -> usize {
        self.CreateDocumentType(name, String::new(), String::new())
    }

    // cpp: dom/document.cc:82-89
    pub fn CreateElement(
        &mut self,
        namespace: DOMNamespace,
        name: String,
        attributes: Vec<DOMAttribute>,
    ) -> usize {
        let namespace_uri = match namespace {
            DOMNamespace::kHTML => "http://www.w3.org/1999/xhtml",
            DOMNamespace::kSVG => "http://www.w3.org/2000/svg",
            DOMNamespace::kMathML => "http://www.w3.org/1998/Math/MathML",
            DOMNamespace::kNone => "",
        };
        self.CreateElementWithQualifiedName(
            namespace,
            foundation::QualifiedName::new(
                foundation::AtomicString::default(),
                foundation::AtomicString::from_str(&name),
                if namespace_uri.is_empty() {
                    foundation::AtomicString::default()
                } else {
                    foundation::AtomicString::from_str(namespace_uri)
                },
            ),
            attributes,
        )
    }
    /// CustomElement::CreateUncustomizedOrUndefinedElementTemplate cc:142-158;
    /// Element::SetIsValue cc:7595-7605. This is not getAttribute("is").
    pub fn SetElementIsValueUTF16(&mut self, node: usize, value: Rc<[u16]>) {
        assert!(
            self.arena[node].is_value.is_none(),
            "Element::SetIsValue is one-shot"
        );
        self.arena[node].is_value = Some(value);
    }
    pub fn ElementIsValueUTF16(&self, node: usize) -> Option<&[u16]> {
        self.arena[node].is_value.as_deref()
    }
    pub fn CreateElementWithQualifiedName(
        &mut self,
        namespace: DOMNamespace,
        tag: foundation::QualifiedName,
        attributes: Vec<DOMAttribute>,
    ) -> usize {
        let is_value = attributes
            .iter()
            .find(|a| a.namespace_uri.is_empty() && a.prefix.is_empty() && a.local_name == "is")
            .map(|a| Rc::<[u16]>::from(a.value.encode_utf16().collect::<Vec<_>>()));
        let name = tag.LocalName().Utf8();
        let element = self.Allocate(
            DOMNodeType::kElement,
            namespace,
            name,
            String::new(),
            attributes,
        );
        self.arena[element].name_utf16 =
            Rc::from(tag.LocalName().utf16_units().unwrap_or_default());
        self.arena[element].tag_qname_utf16 = DOMQualifiedNameUTF16 {
            prefix: Rc::from(tag.Prefix().utf16_units().unwrap_or_default()),
            local_name: self.arena[element].name_utf16.clone(),
            namespace_uri: Rc::from(tag.NamespaceURI().utf16_units().unwrap_or_default()),
        };
        self.arena[element].tag_qname = tag;
        if let Some(is_value) = is_value {
            self.SetElementIsValueUTF16(element, is_value);
        }
        if self.arena[element].IsElement("template", DOMNamespace::kHTML) {
            self.EnsureTemplateContents(element);
        }
        // Element::AttributeChanged's presentation-attribute lifecycle also
        // runs for parser-supplied initial attributes (element.cc:3882-3887).
        if namespace == DOMNamespace::kHTML
            && self.arena[element].attributes.iter().any(|attribute| {
                attribute.namespace_uri.is_empty()
                    && attribute.local_name == "hidden"
                    && attribute.value == "until-found"
            })
        {
            self.SetDisplayLockHiddenUntilFound(element, true);
        }
        if self.arena[element].IsHTMLElement("img")
            || self.InputType(element) == Some(DOMInputType::kImage)
        {
            let source = self.arena[element]
                .FindAttribute("src")
                .map(|a| a.value.clone());
            self.EnsureHTMLImageLoader(element).InitializeSource(source);
        }
        self.InitializeImageMapElement(element);
        if self.arena[element].IsHTMLElement("iframe") {
            self.UpdateIFrameRequiredDocumentPolicy(element);
        }
        self.SyncPartNamesMap(element, false);
        self.UpdateFocusgroup(element);
        if self.details_summary_label.is_some() {
            self.EnsureDetailsUserAgentShadowRoot(element);
        }
        self.EnsureOptionUserAgentShadowTree(element);
        self.InitializeSelectElement(element);
        self.InitializeDataListElement(element);
        self.InitializeInputValue(element);
        self.EnsureTextControlUAShadowTree(element);
        self.EnsureColorUAShadowTree(element);
        self.EnsureFileUAShadowTree(element);
        self.EnsureMeterUAShadowTree(element);
        self.EnsureProgressUAShadowTree(element);
        self.UpdateDirectionalitySubtree(element);
        if self.arena[element].class.IsHTMLInputElement() {
            self.attribute_lifecycle
                .RecordInputInitialized(self.arena[element].id);
        }
        element
    }

    // cpp: dom/document.h:98-100
    pub fn CreateElementDefault(&mut self, namespace: DOMNamespace, name: String) -> usize {
        self.CreateElement(namespace, name, Vec::new())
    }

    // cpp: dom/document.cc:91-109
    pub fn CreateDocumentFragment(&mut self) -> usize {
        self.Allocate(
            DOMNodeType::kDocumentFragment,
            DOMNamespace::kNone,
            "#document-fragment".into(),
            String::new(),
            Vec::new(),
        )
    }
    pub fn CharacterDataUTF16Slice(&self, node: usize) -> &[u16] {
        self.character_data_utf16
            .get(&self.arena[node].id)
            .map_or(&[], |units| units.as_ref())
    }
    pub fn CharacterDataUTF16(&self, node: usize) -> Rc<[u16]> {
        self.character_data_utf16
            .get(&self.arena[node].id)
            .cloned()
            .unwrap_or_else(|| Rc::from([]))
    }
    pub fn CreateTextUTF16(&mut self, units: &[u16]) -> usize {
        let node = self.Allocate(
            DOMNodeType::kText,
            DOMNamespace::kNone,
            "#text".into(),
            String::from_utf16_lossy(units),
            Vec::new(),
        );
        self.character_data_utf16
            .insert(self.arena[node].id, Rc::from(units));
        node
    }
    pub fn CreateText(&mut self, data: String) -> usize {
        let units: Rc<[u16]> = data.encode_utf16().collect::<Vec<_>>().into();
        let node = self.Allocate(
            DOMNodeType::kText,
            DOMNamespace::kNone,
            "#text".into(),
            data,
            Vec::new(),
        );
        self.character_data_utf16.insert(self.arena[node].id, units);
        node
    }
    // CDATASection is a Text subclass, with its own retained dynamic class.
    pub fn CreateCDATASection(&mut self, data: String) -> usize {
        let node = self.CreateText(data);
        self.arena[node].class = DOMNodeClass::CDATASection;
        node
    }
    pub fn CreateCommentUTF16(&mut self, units: &[u16]) -> usize {
        let node = self.CreateComment(String::from_utf16_lossy(units));
        self.character_data_utf16
            .insert(self.arena[node].id, Rc::from(units));
        node
    }
    pub fn CreateCDATASectionUTF16(&mut self, units: &[u16]) -> usize {
        let node = self.CreateTextUTF16(units);
        self.arena[node].class = DOMNodeClass::CDATASection;
        node
    }
    pub fn CreateComment(&mut self, data: String) -> usize {
        self.Allocate(
            DOMNodeType::kComment,
            DOMNamespace::kNone,
            "#comment".into(),
            data,
            Vec::new(),
        )
    }
    pub fn CreateProcessingInstruction(&mut self, target: String, data: String) -> usize {
        self.Allocate(
            DOMNodeType::kProcessingInstruction,
            DOMNamespace::kNone,
            target,
            data,
            Vec::new(),
        )
    }

    // cpp: dom/document.cc:111-135
    pub fn CloneNode(&mut self, source: usize, deep: bool) -> usize {
        self.CloneNodeWithOwner(source, deep, self.arena[source].owner_document_handle)
    }

    // Blink Document::importNode -> Node::Clone(factory, cloning_data).
    // The receiving document is the cloning factory, including descendants.
    pub fn ImportNode(&mut self, source: usize, document: usize, deep: bool) -> usize {
        if self.arena[document].node_type != DOMNodeType::kDocument
            || self.arena[source].node_type == DOMNodeType::kDocument
            || self.ShadowHost(source).is_some()
        {
            invalid_argument("invalid node import");
        }
        let owner = DOMOwnerHandle {
            arena_id: self.arena_id,
            node_id: self.arena[document].id,
        };
        self.CloneNodeWithOwner(source, deep, Some(owner))
    }

    fn CloneNodeWithOwner(
        &mut self,
        source: usize,
        deep: bool,
        owner: Option<DOMOwnerHandle>,
    ) -> usize {
        // Clone the node payload and attributes, never its retained native
        // class state (select slots, selectedcontent membership or caches).
        let original = &self.arena[source];
        let (kind, namespace, name, data, attributes, tag) = (
            original.node_type,
            original.namespace,
            original.name.clone(),
            original.data.clone(),
            original.attributes.clone(),
            original.tag_qname.clone(),
        );
        let is_cdata = original.IsCDATASection();
        let original_units = self.character_data_utf16.get(&original.id).cloned();
        let clone = self.Allocate(kind, namespace, name, data, attributes);
        self.VTTElementCloned(source, clone);
        self.arena[clone].tag_qname = tag;
        self.arena[clone].tag_qname_utf16 = self.arena[source].tag_qname_utf16.clone();
        self.arena[clone].attribute_utf16 = self.arena[source].attribute_utf16.clone();
        self.arena[clone].is_value = self.arena[source].is_value.clone();
        if is_cdata {
            self.arena[clone].class = DOMNodeClass::CDATASection;
        }
        if let Some(units) = original_units {
            self.character_data_utf16
                .insert(self.arena[clone].id, units);
        }
        if let Some(units) = self
            .input_value_attribute_utf16
            .get(&self.arena[source].id)
            .cloned()
        {
            self.input_value_attribute_utf16
                .insert(self.arena[clone].id, units);
        }
        if let Some(files) = self.InputFiles(source) {
            let cloned = Rc::new(fileapi::FileList::default());
            for i in 0..files.length() {
                cloned.Append(files.item(i).expect("file list index").Clone());
            }
            self.StoreInputFiles(clone, cloned);
        }
        if let Some(owner_document) = owner {
            self.AdoptNodeByHandle(clone, owner_document);
        }
        self.InitializeImageMapElement(clone);
        self.SyncPartNamesMap(clone, false);
        self.UpdateFocusgroup(clone);
        if self.arena[source].IsElement("template", DOMNamespace::kHTML) {
            if let Some(contents) = self.TemplateContents(source) {
                let cloned_contents = self.EnsureTemplateContents(clone);
                if deep {
                    let children = self.arena[contents].children.clone();
                    for child in children {
                        let cloned_child = self.CloneNodeWithOwner(child, true, owner);
                        self.AppendChild(cloned_contents, cloned_child);
                    }
                }
            }
        }
        if self.details_summary_label.is_some() {
            self.EnsureDetailsUserAgentShadowRoot(clone);
        }
        self.EnsureOptionUserAgentShadowTree(clone);
        self.InitializeSelectElement(clone);
        self.InitializeDataListElement(clone);
        self.EnsureTextControlUAShadowTree(clone);
        self.EnsureColorUAShadowTree(clone);
        self.EnsureFileUAShadowTree(clone);
        self.EnsureMeterUAShadowTree(clone);
        self.EnsureProgressUAShadowTree(clone);
        if deep {
            let children = self.arena[source].children.clone();
            for child in children {
                let cloned_child = self.CloneNodeWithOwner(child, true, owner);
                self.AppendChild(clone, cloned_child);
            }
        }
        let source_id = self.arena[source].id;
        let clone_id = self.arena[clone].id;
        if let Some(&checked) = self.control_checked_state.get(&source_id) {
            self.control_checked_state.insert(clone_id, checked);
        }
        if self.control_value_dirty.contains(&source_id) {
            self.control_value_dirty.insert(clone_id);
        }
        if let Some(value) = self.control_value_state.get(&source_id).cloned() {
            self.control_value_state.insert(clone_id, value);
        }
        // The UA subtree is created before live control state is copied. Bring
        // its implementation element back in sync with the cloned value.
        if self.arena[clone].IsElement("input", DOMNamespace::kHTML)
            || self.arena[clone].IsElement("textarea", DOMNamespace::kHTML)
        {
            self.SetTextControlInnerEditorValueUTF16(clone, &self.ControlValueUTF16(clone));
            self.SyncTextControlPlaceholderElement(clone);
            self.SyncSearchCancelButtonVisibility(clone);
            self.SyncColorSwatchAppearance(clone);
        }
        if self.arena[clone].IsHTMLElement("img")
            || self.InputType(clone) == Some(DOMInputType::kImage)
        {
            let source = self.arena[clone]
                .FindAttribute("src")
                .map(|attribute| attribute.value.clone());
            self.EnsureHTMLImageLoader(clone).InitializeSource(source);
        }
        self.UpdateDirectionalitySubtree(clone);
        if self.arena[clone].class.IsHTMLInputElement() {
            self.attribute_lifecycle
                .RecordInputInitialized(self.arena[clone].id);
        }
        clone
    }

    pub fn EnsureTextControlUAShadowTree(&mut self, element: usize) {
        self.EnsureTextControlUAShadowTreeInternal(element, true);
    }
    pub fn CreateInputTypeShadowSubtreeWithoutUpdatingView(&mut self, element: usize) {
        self.EnsureTextControlUAShadowTreeInternal(element, false);
        self.EnsureColorUAShadowTree(element);
        self.EnsureFileUAShadowTree(element);
    }
    fn EnsureTextControlUAShadowTreeInternal(&mut self, element: usize, update_view: bool) {
        let is_textarea = self.arena[element].IsElement("textarea", DOMNamespace::kHTML);
        let is_text_field = self
            .InputType(element)
            .is_some_and(DOMInputType::IsTextFieldInputType);
        if !is_textarea && !is_text_field {
            self.RemoveTextControlUAShadowTree(element);
            return;
        }
        if self.ShadowRoot(element).is_none() {
            self.AttachShadowRoot(element, DOMShadowRootMode::kUserAgent);
        }
        let root = self
            .ShadowRoot(element)
            .expect("text control UA shadow root must exist");
        let editor = if let Some(&editor) = self.text_control_inner_editors.get(&element) {
            editor
        } else {
            // cpp: TextControlElement::CreateInnerEditorElement. The subclass
            // is retained as a real HTMLDivElement; Style/Layout identify it
            // through the typed owner relation below rather than a public id.
            let editor = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
            self.text_control_inner_editors.insert(element, editor);
            self.text_control_inner_editor_hosts.insert(editor, element);
            self.AppendChild(root, editor);
            editor
        };
        let needs_container = matches!(
            self.InputType(element),
            Some(DOMInputType::kSearch | DOMInputType::kNumber)
        ) || (self.InputType(element) == Some(DOMInputType::kPassword)
            && foundation::RuntimeEnabledFeatures::PasswordRevealEnabled())
            || (self.InputType(element) == Some(DOMInputType::kEmail)
                && foundation::RuntimeEnabledFeatures::EmailVerificationStatusIndicatorEnabled());
        if needs_container && !self.text_control_containers.contains_key(&element) {
            // cpp: TextFieldInputType::CreateShadowSubtree. Search controls
            // require the decoration container; number controls are steppable.
            let container = self.CreateElement(
                DOMNamespace::kHTML,
                "div".into(),
                vec![DOMAttribute {
                    local_name: "id".into(),
                    value: "text-field-container".into(),
                    ..Default::default()
                }],
            );
            self.SetUAShadowPseudoElementIdentity(
                element,
                container,
                "-webkit-textfield-decoration-container",
            );
            let viewport = self.CreateElement(
                DOMNamespace::kHTML,
                "div".into(),
                vec![DOMAttribute {
                    local_name: "id".into(),
                    value: "editing-view-port".into(),
                    ..Default::default()
                }],
            );
            self.AppendChild(root, container);
            self.AppendChild(container, viewport);
            self.AppendChild(viewport, editor);
            self.text_control_containers.insert(element, container);
        } else if !needs_container {
            self.RemoveSearchCancelButton(element);
            if let Some(container) = self.text_control_containers.remove(&element) {
                self.AppendChild(root, editor);
                self.RemoveUAShadowPseudoElementIdentity(element, container);
                self.Remove(container);
            }
        }
        if self.InputType(element) == Some(DOMInputType::kSearch) {
            if !self.search_cancel_buttons.contains_key(&element) {
                // cpp: search_input_type.cc:70-82. Search appends the cancel
                // button immediately after the editing viewport in the
                // decoration container created by TextFieldInputType.
                let cancel = self.CreateElement(
                    DOMNamespace::kHTML,
                    "div".into(),
                    vec![DOMAttribute {
                        local_name: "id".into(),
                        value: "search-clear".into(),
                        ..Default::default()
                    }],
                );
                let container = self.text_control_containers[&element];
                self.AppendChild(container, cancel);
                self.SetUAShadowPseudoElementIdentity(
                    element,
                    cancel,
                    "-webkit-search-cancel-button",
                );
                self.search_cancel_buttons.insert(element, cancel);
            }
            self.SyncSearchCancelButtonVisibility(element);
        } else {
            self.RemoveSearchCancelButton(element);
        }
        if update_view {
            self.SetTextControlInnerEditorValueUTF16(element, &self.ControlValueUTF16(element));
        }
        self.SyncTextControlPlaceholderElement(element);
    }

    pub fn DestroyInputTypeShadowSubtree(&mut self, element: usize) {
        for pseudo in [
            "-webkit-slider-container",
            "-webkit-slider-runnable-track",
            "-webkit-slider-thumb",
            "-internal-reveal",
        ] {
            if let Some(part) = self.UAShadowPseudoElement(element, pseudo) {
                self.RemoveUAShadowPseudoElementIdentity(element, part);
            }
        }

        self.RemoveTextControlUAShadowTree(element);
        if let Some(root) = self.ShadowRoot(element) {
            if self.ShadowRootMode(root) == Some(DOMShadowRootMode::kUserAgent) {
                for child in self.arena[root].children.clone() {
                    self.Remove(child);
                }
            }
        }
        self.color_swatch_wrappers.remove(&element);
        self.color_swatches.remove(&element);
        self.file_upload_buttons.remove(&element);
        self.file_status_elements.remove(&element);
    }
    /// Element::SetForceReattachLayoutTree. Route the canonical node through
    /// the existing LayoutTreeUpdate::Reattach owner even when CSS is equal.
    pub fn ForceInputTypeLayoutReattach(&mut self, element: usize) {
        self.ForceElementLayoutReattach(element);
    }
    pub fn ForceElementLayoutReattach(&mut self, element: usize) {
        self.style_state
            .layout_reattach_nodes
            .insert(self.arena[element].id);
        self.RecordLayoutGeometryChange(element);
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::TREE);
    }
    pub fn InvalidateInputTypeStyle(&mut self, element: usize) {
        self.InvalidateNodeStyle(element);
        self.RecordLayoutInputChange(element);
    }
    /// HTMLInputElement::ParseAttribute(value), cc:931-934. This is a forced
    /// subtree recalc, independent of selector descendant dependencies.
    pub fn SetInputValueAttributeSubtreeStyleRecalc(&mut self, element: usize) {
        if self.IsConnectedToDocument(element) {
            self.style_state.subtree_style_recalc_roots.insert(element);
            self.RecordStyleChange(crate::style_state::StyleChange::Node(element));
        }
    }
    pub fn UpdateInputTypeDirectionality(&mut self, element: usize) {
        self.UpdateDirectionalityForMutation(element);
    }
    fn RemoveTextControlUAShadowTree(&mut self, element: usize) {
        if let Some(placeholder) = self.text_control_placeholders.remove(&element) {
            self.RemoveUAShadowPseudoElementIdentity(element, placeholder);
            self.Remove(placeholder);
        }
        self.RemoveSearchCancelButton(element);
        if let Some(container) = self.text_control_containers.remove(&element) {
            self.RemoveUAShadowPseudoElementIdentity(element, container);
            self.Remove(container);
        }
        if let Some(editor) = self.text_control_inner_editors.remove(&element) {
            self.text_control_inner_editor_hosts.remove(&editor);
            self.Remove(editor);
        }
    }

    fn RemoveSearchCancelButton(&mut self, element: usize) {
        let Some(cancel) = self.search_cancel_buttons.remove(&element) else {
            return;
        };
        self.RemoveUAShadowPseudoElementIdentity(element, cancel);
        self.Remove(cancel);
    }

    // cpp: search_input_type.cc:147-162. Empty visible values suppress the
    // implementation button through its own inline style. Nonempty values
    // remove those declarations so the typed html.css rules become effective.
    fn SyncSearchCancelButtonVisibility(&mut self, element: usize) {
        let Some(&cancel) = self.search_cancel_buttons.get(&element) else {
            return;
        };
        if self.ControlValue(element).is_empty() {
            self.SetAttribute(
                cancel,
                DOMAttribute {
                    local_name: "style".into(),
                    value: "opacity:0;pointer-events:none".into(),
                    ..Default::default()
                },
            );
        } else {
            self.RemoveAttributeDefault(cancel, "style");
        }
    }

    // cpp: color_input_type.cc:180-210. ColorInputAcceptsCSSColors is stable
    // at the fixed revision. The control stores an opaque lower-case sRGB
    // value; invalid input and currentcolor resolve to black.
    fn SanitizeColorInputValue(value: &str) -> String {
        if value.trim().eq_ignore_ascii_case("currentcolor") {
            return "#000000".into();
        }
        let Some(color) = layoutng_assembly::css_color_parser::ParseCSSColor(value) else {
            return "#000000".into();
        };
        let channel = |value: f32| -> u8 { (value.clamp(0.0, 1.0) * 255.0).round() as u8 };
        format!(
            "#{:02x}{:02x}{:02x}",
            channel(color.red),
            channel(color.green),
            channel(color.blue)
        )
    }

    // cpp: color_input_type.cc:225-239 ColorInputType::CreateShadowSubtree.
    fn EnsureColorUAShadowTree(&mut self, element: usize) {
        // Registered source InputTypeViews own shadow creation and updates.
        if self.attribute_lifecycle.HasOwner()
            && self
                .input_type_lifecycle_owners
                .contains(&self.arena[element].id)
        {
            return;
        }
        if self.InputType(element) != Some(DOMInputType::kColor) {
            if let Some(wrapper) = self.color_swatch_wrappers.remove(&element) {
                if let Some(swatch) = self.color_swatches.remove(&element) {
                    self.RemoveUAShadowPseudoElementIdentity(element, swatch);
                }
                self.RemoveUAShadowPseudoElementIdentity(element, wrapper);
                self.Remove(wrapper);
            }
            return;
        }
        if self.color_swatches.contains_key(&element) {
            self.SyncColorSwatchAppearance(element);
            return;
        }
        let root = self
            .ShadowRoot(element)
            .unwrap_or_else(|| self.AttachShadowRoot(element, DOMShadowRootMode::kUserAgent));
        let wrapper = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        let swatch = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        self.AppendChild(wrapper, swatch);
        self.AppendChild(root, wrapper);
        self.SetUAShadowPseudoElementIdentity(element, wrapper, "-webkit-color-swatch-wrapper");
        self.SetUAShadowPseudoElementIdentity(element, swatch, "-webkit-color-swatch");
        self.color_swatch_wrappers.insert(element, wrapper);
        self.color_swatches.insert(element, swatch);
        self.SyncColorSwatchAppearance(element);
    }

    // cpp: color_input_type.cc:378-385 ColorInputType::UpdateView.
    fn SyncColorSwatchAppearance(&mut self, element: usize) {
        let Some(&swatch) = self.color_swatches.get(&element) else {
            return;
        };
        let value = Self::SanitizeColorInputValue(&self.ControlValue(element));
        self.SetAttribute(
            swatch,
            DOMAttribute {
                local_name: "style".into(),
                value: format!("background-color:{value}"),
                ..Default::default()
            },
        );
    }

    // cpp: file_input_type.cc:381-408 FileInputType::CreateShadowSubtree.
    // Blink creates an actual input[type=button] followed by a status span.
    // Keeping those nodes retained lets the typed custom-pseudo selector and
    // the layout projection consume the same style owner.
    fn EnsureFileUAShadowTree(&mut self, element: usize) {
        // Registered source InputTypeViews own shadow creation and updates.
        if self.attribute_lifecycle.HasOwner()
            && self
                .input_type_lifecycle_owners
                .contains(&self.arena[element].id)
        {
            return;
        }
        if self.InputType(element) != Some(DOMInputType::kFile) {
            if let Some(button) = self.file_upload_buttons.remove(&element) {
                self.RemoveUAShadowPseudoElementIdentity(element, button);
                self.Remove(button);
            }
            if let Some(status) = self.file_status_elements.remove(&element) {
                self.Remove(status);
            }
            return;
        }
        if self.file_upload_buttons.contains_key(&element) {
            self.SyncFileUAShadowTree(element);
            return;
        }
        let root = self
            .ShadowRoot(element)
            .unwrap_or_else(|| self.AttachShadowRoot(element, DOMShadowRootMode::kUserAgent));
        let mut button_attributes = vec![
            DOMAttribute {
                local_name: "type".into(),
                value: "button".into(),
                ..Default::default()
            },
            DOMAttribute {
                local_name: "value".into(),
                value: self.FileUploadButtonLabel(element).into(),
                ..Default::default()
            },
            DOMAttribute {
                local_name: "id".into(),
                value: "file-upload-button".into(),
                ..Default::default()
            },
            DOMAttribute {
                local_name: "aria-hidden".into(),
                value: "true".into(),
                ..Default::default()
            },
        ];
        if self.HasBooleanHTMLAttribute(element, "disabled") {
            button_attributes.push(DOMAttribute {
                local_name: "disabled".into(),
                value: String::new(),
                ..Default::default()
            });
        }
        let button = self.CreateElement(DOMNamespace::kHTML, "input".into(), button_attributes);
        let status = self.CreateElement(
            DOMNamespace::kHTML,
            "span".into(),
            vec![DOMAttribute {
                local_name: "aria-hidden".into(),
                value: "true".into(),
                ..Default::default()
            }],
        );
        let status_text = self.FileInputStatusText(element);
        self.AppendText(status, status_text);
        self.AppendChild(root, button);
        self.AppendChild(root, status);
        self.SetUAShadowPseudoElementIdentity(element, button, "-webkit-file-upload-button");
        self.file_upload_buttons.insert(element, button);
        self.file_status_elements.insert(element, status);
    }

    fn FileUploadButtonLabel(&self, element: usize) -> &'static str {
        if self.HasBooleanHTMLAttribute(element, "multiple") {
            "Choose files"
        } else {
            "Choose file"
        }
    }

    // cpp: FileInputType::FileStatusText/UpdateView. The retained DOM has no
    // FileList owner yet, so its only exact state is the empty-list label.
    pub fn FileInputStatusText(&self, element: usize) -> &'static str {
        if self.InputType(element) != Some(DOMInputType::kFile) {
            invalid_argument("file status requires an input[type=file]");
        }
        "No file selected"
    }

    // cpp: FileInputType::DisabledAttributeChanged/MultipleAttributeChanged.
    fn SyncFileUAShadowTree(&mut self, element: usize) {
        let Some(&button) = self.file_upload_buttons.get(&element) else {
            return;
        };
        let label = self.FileUploadButtonLabel(element);
        self.SetAttribute(
            button,
            DOMAttribute {
                local_name: "value".into(),
                value: label.into(),
                ..Default::default()
            },
        );
        if self.HasBooleanHTMLAttribute(element, "disabled") {
            self.SetAttribute(
                button,
                DOMAttribute {
                    local_name: "disabled".into(),
                    value: String::new(),
                    ..Default::default()
                },
            );
        } else {
            self.RemoveAttributeDefault(button, "disabled");
        }
    }

    pub fn FileUploadButtonElement(&self, element: usize) -> Option<usize> {
        self.file_upload_buttons.get(&element).copied()
    }

    pub fn FileStatusElement(&self, element: usize) -> Option<usize> {
        self.file_status_elements.get(&element).copied()
    }

    fn SyncPartNamesMap(&mut self, element: usize, retain_empty: bool) {
        let value = self.arena[element]
            .attributes
            .iter()
            .find(|attribute| {
                attribute.local_name == "exportparts" && attribute.namespace_uri.is_empty()
            })
            .map(|attribute| attribute.value.clone());
        if let Some(value) = value {
            self.part_names_maps
                .insert(element, DOMPartNamesMap::Parse(&value));
        } else if retain_empty {
            self.part_names_maps
                .insert(element, DOMPartNamesMap::default());
        } else {
            self.part_names_maps.remove(&element);
        }
    }

    fn SupportsPlaceholderElement(&self, element: usize) -> bool {
        if self.arena[element].IsElement("textarea", DOMNamespace::kHTML) {
            return true;
        }
        self.InputType(element)
            .is_some_and(DOMInputType::SupportsPlaceholder)
    }

    fn SyncTextControlPlaceholderElement(&mut self, element: usize) {
        const PLACEHOLDER_SHADOW_PSEUDO_ID: &str = "-webkit-input-placeholder";
        const SUGGESTED_SHADOW_PSEUDO_ID: &str = "-internal-input-suggested";
        let existing = self.text_control_placeholders.get(&element).copied();
        let suggested = self
            .suggested_value_state
            .get(&self.arena[element].id)
            .cloned();
        let is_suggested_value = suggested.as_ref().is_some_and(|value| !value.is_empty());
        let has_placeholder = self.arena[element].FindAttribute("placeholder").is_some();
        let should_exist =
            self.SupportsPlaceholderElement(element) && (has_placeholder || is_suggested_value);
        if !should_exist {
            if let Some(placeholder) = existing {
                self.text_control_placeholders.remove(&element);
                self.RemoveUAShadowPseudoElementIdentity(element, placeholder);
                self.Remove(placeholder);
            }
            return;
        }
        let root = self
            .ShadowRoot(element)
            .expect("text control UA shadow root must exist");
        let placeholder = existing.unwrap_or_else(|| {
            // cpp: TextFieldInputType::UpdatePlaceholderText;
            // HTMLTextAreaElement::UpdatePlaceholderText.
            let placeholder = self.CreateElement(
                DOMNamespace::kHTML,
                "div".into(),
                vec![DOMAttribute {
                    local_name: "id".into(),
                    value: "placeholder".into(),
                    ..Default::default()
                }],
            );
            let carrier = self
                .text_control_containers
                .get(&element)
                .or_else(|| self.text_control_inner_editors.get(&element))
                .copied()
                .expect("text control UA shadow tree must retain an editor");
            self.InsertBefore(root, placeholder, carrier);
            self.text_control_placeholders.insert(element, placeholder);
            placeholder
        });
        let visible =
            suggested.is_none() && has_placeholder && self.ControlValue(element).is_empty();
        let display = if visible || suggested.is_some() {
            "block"
        } else {
            "none"
        };
        let style = if is_suggested_value {
            format!("display:{display}!important;user-select:none!important")
        } else {
            format!("display:{display}!important")
        };
        self.SetAttribute(
            placeholder,
            DOMAttribute {
                local_name: "style".into(),
                value: style,
                ..Default::default()
            },
        );
        let pseudo_id = if is_suggested_value {
            SUGGESTED_SHADOW_PSEUDO_ID
        } else {
            PLACEHOLDER_SHADOW_PSEUDO_ID
        };
        self.ua_shadow_pseudo_elements
            .entry(element)
            .or_default()
            .retain(|_, candidate| *candidate != placeholder);
        self.ua_shadow_pseudo_elements
            .entry(element)
            .or_default()
            .insert(pseudo_id.into(), placeholder);
        self.shadow_pseudo_ids.insert(placeholder, pseudo_id.into());
        self.ultimate_originating_elements
            .insert(placeholder, element);
        let text = suggested
            .filter(|value| !value.is_empty())
            .or_else(|| {
                self.arena[element]
                    .FindAttribute("placeholder")
                    .map(|attribute| attribute.value.clone())
            })
            .unwrap_or_default();
        let mut current_text = String::new();
        self.AppendDescendantText(placeholder, &mut current_text);
        if current_text != text {
            self.SetTextContent(placeholder, text);
        }
    }

    pub fn TextControlPlaceholderElement(&self, element: usize) -> Option<usize> {
        self.text_control_placeholders.get(&element).copied()
    }

    pub fn TextControlInnerEditorElement(&self, element: usize) -> Option<usize> {
        self.text_control_inner_editors.get(&element).copied()
    }

    pub fn TextControlInnerEditorHost(&self, editor: usize) -> Option<usize> {
        self.text_control_inner_editor_hosts.get(&editor).copied()
    }

    /// cpp: TextControlElement::SetInnerEditorValue / AppendTextOrBr. The
    /// shadow editor owns the actual editing nodes; neither Layout nor IME
    /// invents a second value tree. Equal values retain their node identities.
    pub fn SetTextControlInnerEditorValue(&mut self, element: usize, value: String) {
        self.SetTextControlInnerEditorValueUTF16(
            element,
            &value.encode_utf16().collect::<Vec<_>>(),
        );
    }
    pub fn SetTextControlInnerEditorValueUTF16(&mut self, element: usize, value: &[u16]) {
        let Some(editor) = self.TextControlInnerEditorElement(element) else {
            return;
        };
        if self.TextControlInnerEditorValueUTF16(element).as_ref() == value
            && !self.arena[editor].children.is_empty()
        {
            return;
        }
        for child in self.arena[editor].children.clone() {
            self.Remove(child);
        }
        if self.arena[element].IsHTMLElement("textarea") {
            self.AppendTextOrBrUTF16(editor, value);
        } else {
            self.AppendTextInChunksUTF16(editor, value);
        }
        self.AdjustTextControlPlaceholderBreak(element);
    }

    // TextControlElement::AppendText, text_control_element.cc:130-140.
    fn AppendTextInChunksUTF16(&mut self, container: usize, value: &[u16]) {
        for chunk in value.chunks(8192) {
            let text = self.CreateTextUTF16(chunk);
            self.AppendChild(container, text);
        }
    }
    /// TextControlElement::AppendTextOrBr, cc:1099-1117. Detached fragment
    /// preparation and the canonical inner editor share this original owner.
    pub fn AppendTextOrBrUTF16(&mut self, container: usize, value: &[u16]) {
        let mut start = 0;
        while start < value.len() {
            let Some(i) = value[start..]
                .iter()
                .position(|&unit| unit == 10)
                .map(|i| i + start)
            else {
                self.AppendTextInChunksUTF16(container, &value[start..]);
                break;
            };
            if i != start {
                self.AppendTextInChunksUTF16(container, &value[start..i]);
            }
            let br = self.CreateElementDefault(DOMNamespace::kHTML, "br".into());
            self.AppendChild(container, br);
            start = i + 1;
        }
    }
    pub fn TextControlIsPlaceholderBreak(&self, node: usize) -> bool {
        self.arena[node].IsHTMLElement("br")
            && self.arena[node]
                .FindAttribute("id")
                .is_some_and(|attribute| attribute.value == "placeholder-break")
    }

    // cpp: TextControlElement::AdjustPlaceholderBreakElement. The final BR
    // provides the caret line and is excluded from plain-text serialization.
    pub fn AdjustTextControlPlaceholderBreak(&mut self, element: usize) {
        let Some(editor) = self.TextControlInnerEditorElement(element) else {
            return;
        };
        let multiline = self.arena[element].IsHTMLElement("textarea");
        if !multiline {
            return;
        }
        while let Some(last) = self.arena[editor].children.last().copied() {
            if self.arena[last].node_type != DOMNodeType::kText || !self.arena[last].data.is_empty()
            {
                break;
            }
            self.Remove(last);
        }
        let last = self.arena[editor].children.last().copied();
        if let Some(last) = last.filter(|&last| self.TextControlIsPlaceholderBreak(last)) {
            let children = &self.arena[editor].children;
            if children.len() > 1
                && self.TextControlIsPlaceholderBreak(children[children.len() - 2])
            {
                self.Remove(last);
            }
            return;
        }
        let needs_break = last.is_none_or(|last| {
            self.arena[last].IsHTMLElement("br")
                || (self.arena[last].node_type == DOMNodeType::kText
                    && (self.arena[last].data.ends_with('\n')
                        || self.arena[last].data.ends_with('\r')))
        });
        if needs_break {
            let br = self.CreateElement(
                DOMNamespace::kHTML,
                "br".into(),
                vec![DOMAttribute {
                    local_name: "id".into(),
                    value: "placeholder-break".into(),
                    ..Default::default()
                }],
            );
            self.AppendChild(editor, br);
        }
    }

    /// cpp: TextControlElement::SerializeInnerEditorValue. Textarea hard
    /// breaks contribute one LF; the final placeholder break contributes none.
    pub fn TextControlInnerEditorValue(&self, element: usize) -> String {
        String::from_utf16_lossy(&self.TextControlInnerEditorValueUTF16(element))
    }
    pub fn TextControlInnerEditorValueUTF16(&self, element: usize) -> Rc<[u16]> {
        let Some(editor) = self.TextControlInnerEditorElement(element) else {
            return Rc::from([]);
        };
        let mut value = Vec::new();
        for &child in &self.arena[editor].children {
            if self.arena[child].node_type == DOMNodeType::kText {
                value.extend_from_slice(&self.CharacterDataUTF16(child));
            } else if self.arena[child].IsHTMLElement("br")
                && !self.TextControlIsPlaceholderBreak(child)
            {
                value.push(b'\n' as u16);
            }
        }
        value.into()
    }

    pub fn TextControlInnerEditorTextNode(&self, element: usize) -> Option<usize> {
        let editor = self.TextControlInnerEditorElement(element)?;
        self.arena[editor]
            .children
            .iter()
            .copied()
            .find(|&child| self.arena[child].node_type == DOMNodeType::kText)
    }

    /// PlainTextRange's UTF-16 coordinates resolve onto the existing UA
    /// editing nodes, including empty editors and positions around BRs.
    pub fn TextControlInnerEditorPosition(
        &self,
        element: usize,
        mut offset: usize,
    ) -> Option<crate::range::DOMBoundaryPoint> {
        let editor = self.TextControlInnerEditorElement(element)?;
        for (index, &child) in self.arena[editor].children.iter().enumerate() {
            if self.arena[child].node_type == DOMNodeType::kText {
                let length = self.CharacterDataUTF16(child).len();
                if offset <= length {
                    return Some(crate::range::DOMBoundaryPoint {
                        node_id: self.arena[child].id,
                        offset,
                    });
                }
                offset -= length;
            } else if self.TextControlIsPlaceholderBreak(child) || offset == 0 {
                return Some(crate::range::DOMBoundaryPoint {
                    node_id: self.arena[editor].id,
                    offset: index,
                });
            } else if self.arena[child].IsHTMLElement("br") {
                offset -= 1;
            }
        }
        Some(crate::range::DOMBoundaryPoint {
            node_id: self.arena[editor].id,
            offset: self.arena[editor].children.len(),
        })
    }

    pub fn TextControlInnerEditorOffset(
        &self,
        element: usize,
        point: crate::range::DOMBoundaryPoint,
    ) -> Option<usize> {
        let editor = self.TextControlInnerEditorElement(element)?;
        let node = self.FindNodeById(point.node_id)?;
        let children = &self.arena[editor].children;
        if node == editor && point.offset > children.len() {
            return None;
        }
        let mut offset = 0;
        for (index, &child) in children.iter().enumerate() {
            if node == editor && point.offset == index {
                return Some(offset);
            }
            let length = if self.arena[child].node_type == DOMNodeType::kText {
                self.CharacterDataUTF16(child).len()
            } else {
                usize::from(
                    self.arena[child].IsHTMLElement("br")
                        && !self.TextControlIsPlaceholderBreak(child),
                )
            };
            if node == child {
                let valid = if self.arena[child].node_type == DOMNodeType::kText {
                    point.offset <= length
                } else {
                    point.offset == 0
                };
                return valid.then_some(offset + point.offset);
            }
            offset += length;
        }
        (node == editor && point.offset == children.len()).then_some(offset)
    }

    /// cpp: HTMLInputElement/HTMLTextAreaElement::SubtreeHasChanged. Editing
    /// mutates resident shadow nodes first, then updates the host value without
    /// replacing those nodes (which may be retained by a composition Range).
    pub fn SyncTextControlValueFromInnerEditor(&mut self, element: usize) {
        if self.TextControlInnerEditorElement(element).is_none() {
            return;
        }
        self.AdjustTextControlPlaceholderBreak(element);
        let visible = self.TextControlInnerEditorValueUTF16(element);
        let value = self.SanitizeInputValue(element, visible);
        // TextFieldInputType::SubtreeHasChanged keeps the user's visible
        // number text even when the canonical value sanitizes to empty.
        self.SetStoredControlValueOwner(
            element,
            foundation::NullableUTF16String::FromOwner(value),
            true,
        );
        self.UpdateTextControlPlaceholder(element);
    }

    fn SyncTextAreaDefaultValue(&mut self, element: usize) {
        if self.arena[element].IsHTMLElement("textarea") && !self.HasDirtyControlValue(element) {
            self.control_value_state.remove(&self.arena[element].id);
            self.SetTextControlInnerEditorValueUTF16(element, &self.ControlValueUTF16(element));
            self.SyncTextControlPlaceholderElement(element);
        }
    }

    // cpp: TextControlInnerEditorElement::SetVisibility. SuggestedValue's
    // nullness owns editor visibility; an empty live value needs no opacity
    // suppression because it contributes no painted text.
    pub fn TextControlInnerEditorIsVisible(&self, editor: usize) -> bool {
        self.TextControlInnerEditorHost(editor).is_some_and(|host| {
            self.SuggestedValue(host).is_none() || self.ControlValue(host).is_empty()
        })
    }

    // TextFieldInputType::CreateShadowSubtree cc:479-517; SpinButtonElement
    // constructor cc:46-59. This part is last in the canonical container.
    pub fn EnsureTextFieldSpinButton(&mut self, input: usize) -> usize {
        assert_eq!(self.InputType(input), Some(DOMInputType::kNumber));
        if let Some(spin) = self.TextFieldSpinButton(input) {
            return spin;
        }
        let container = self
            .TextControlContainerElement(input)
            .expect("number decoration container");
        let spin = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        self.text_field_spin_button_hosts.insert(spin, input);
        self.SetUAShadowPseudoElementIdentity(input, spin, "-webkit-inner-spin-button");
        self.SetAttribute(
            spin,
            DOMAttribute {
                local_name: "id".into(),
                value: "spin".into(),
                ..Default::default()
            },
        );
        self.AppendChild(container, spin);
        spin
    }
    pub fn TextFieldSpinButton(&self, input: usize) -> Option<usize> {
        self.UAShadowPseudoElement(input, "-webkit-inner-spin-button")
            .filter(|spin| self.ContainingShadowRoot(*spin) == self.ShadowRoot(input))
    }
    /// Native constructor identity survives detach and is never inferred by id.
    pub fn TextFieldSpinButtonHost(&self, node: usize) -> Option<usize> {
        self.text_field_spin_button_hosts.get(&node).copied()
    }
    pub fn TextControlContainerElement(&self, element: usize) -> Option<usize> {
        self.text_control_containers.get(&element).copied()
    }

    // cpp: text_control_element.cc:1318-1358. A null suggested value is
    // observably different from an empty suggested value: both make the
    // implementation element visible, while only a nonempty value switches
    // its shadow pseudo id to -internal-input-suggested.
    pub fn SuggestedValue(&self, element: usize) -> Option<&str> {
        self.suggested_value_state
            .get(&self.arena[element].id)
            .map(String::as_str)
    }

    // cpp: html_select_element.cc:362-400.
    pub fn SelectSuggestedOption(&self, select: usize) -> Option<usize> {
        if !self.arena[select].IsElement("select", DOMNamespace::kHTML) {
            return None;
        }
        let option_id = self
            .select_suggested_options
            .get(&self.arena[select].id)
            .copied()?;
        let option = self.FindNodeById(option_id)?;
        (self.OptionOwnerSelect(option) == Some(select)).then_some(option)
    }

    pub fn SetSelectSuggestedOption(&mut self, select: usize, option: Option<usize>) {
        if !self.arena[select].IsElement("select", DOMNamespace::kHTML) {
            invalid_argument("suggested option requires a select element");
        }
        if option.is_some_and(|option| self.OptionOwnerSelect(option) != Some(select)) {
            // Blink ignores a stale option supplied by an external producer.
            return;
        }
        let select_id = self.arena[select].id;
        if self.SelectSuggestedOption(select) == option
            && (option.is_some() || !self.select_suggested_options.contains_key(&select_id))
        {
            return;
        }
        self.SetAutofillState(
            select,
            if option.is_some() {
                DOMAutofillState::kPreviewed
            } else {
                DOMAutofillState::kNotFilled
            },
        );
        if let Some(option) = option {
            self.select_suggested_options
                .insert(select_id, self.arena[option].id);
            let value = self.OptionValue(option);
            self.suggested_value_state.insert(select_id, value);
        } else {
            self.select_suggested_options.remove(&select_id);
            self.suggested_value_state.remove(&select_id);
        }
        // MenuListSelectType/ListBoxSelectType::DidSetSuggestedOption updates
        // rendered content even when the state remains kPreviewed.
        self.InvalidateNodeStyle(select);
        self.RecordLayoutInputChange(select);
    }

    fn RefreshSelectSuggestedValues(&mut self) {
        let owners = self
            .select_suggested_options
            .keys()
            .copied()
            .collect::<Vec<_>>();
        for select_id in owners {
            let Some(select) = self.FindNodeById(select_id) else {
                continue;
            };
            let Some(option) = self.SelectSuggestedOption(select) else {
                self.SetSelectSuggestedOption(select, None);
                continue;
            };
            let value = self.OptionValue(option);
            if self.suggested_value_state.get(&select_id) != Some(&value) {
                self.suggested_value_state.insert(select_id, value);
                self.InvalidateNodeStyle(select);
                self.RecordLayoutInputChange(select);
            }
        }
    }

    pub fn SetSuggestedValue(&mut self, element: usize, value: Option<String>) {
        if self.arena[element].IsElement("select", DOMNamespace::kHTML) {
            let option = value.as_deref().and_then(|value| {
                self.SelectOptionNodeIds(element)
                    .into_iter()
                    .filter_map(|id| self.FindNodeById(id))
                    .find(|&option| self.OptionValue(option) == value)
            });
            self.SetSelectSuggestedOption(element, option);
            return;
        }
        if !self.arena[element].IsElement("input", DOMNamespace::kHTML)
            && !self.arena[element].IsElement("textarea", DOMNamespace::kHTML)
        {
            invalid_argument("suggested value requires a text control");
        }
        // cpp: HTMLInputElement::SetSuggestedValue:1330-1336. InputType owns
        // whether a suggested value can be displayed; the fixed source clears
        // both preview text and autofill state for every non-text-field input.
        if self
            .InputType(element)
            .is_some_and(|input_type| !input_type.IsTextFieldInputType())
        {
            let removed = self
                .suggested_value_state
                .remove(&self.arena[element].id)
                .is_some();
            self.SetAutofillState(element, DOMAutofillState::kNotFilled);
            if removed {
                self.InvalidateNodeStyle(element);
                self.RecordLayoutInputChange(element);
            }
            return;
        }
        let value = value.map(|value| {
            let Some(max_length) = self.arena[element]
                .FindAttribute("maxlength")
                .and_then(|attribute| attribute.value.trim().parse::<i32>().ok())
                .filter(|length| *length >= 0)
                .map(|length| length as usize)
            else {
                return value;
            };
            let utf16 = value.encode_utf16().collect::<Vec<_>>();
            if utf16.len() <= max_length {
                value
            } else {
                // Blink truncates WTF::String in UTF-16 code units. Rust's
                // String cannot retain a lone surrogate, so the DOM projection
                // uses the standard replacement character at a split pair.
                String::from_utf16_lossy(&utf16[..max_length])
            }
        });
        let id = self.arena[element].id;
        let changed = self.SuggestedValue(element) != value.as_deref();
        // cpp: HTMLInputElement::SetSuggestedValue and
        // HTMLTextAreaElement::SetSuggestedValue. Suggested preview text is
        // the real producer for kPreviewed; empty/null text clears it.
        self.SetAutofillState(
            element,
            if value.as_deref().is_some_and(|value| !value.is_empty()) {
                DOMAutofillState::kPreviewed
            } else {
                DOMAutofillState::kNotFilled
            },
        );
        match value {
            Some(value) => {
                self.suggested_value_state.insert(id, value);
            }
            None => {
                self.suggested_value_state.remove(&id);
            }
        }
        if !changed {
            return;
        }
        self.SyncTextControlPlaceholderElement(element);
        if let Some(editor) = self.TextControlInnerEditorElement(element) {
            self.InvalidateNodeStyle(editor);
        }
        self.InvalidateNodeStyle(element);
        self.RecordLayoutInputChange(element);
    }

    fn SetUAShadowPseudoElementIdentity(&mut self, host: usize, element: usize, pseudo_id: &str) {
        self.ua_shadow_pseudo_elements
            .entry(host)
            .or_default()
            .retain(|_, candidate| *candidate != element);
        self.ua_shadow_pseudo_elements
            .entry(host)
            .or_default()
            .insert(pseudo_id.into(), element);
        self.shadow_pseudo_ids.insert(element, pseudo_id.into());
        self.ultimate_originating_elements.insert(element, host);
    }

    fn RemoveUAShadowPseudoElementIdentity(&mut self, host: usize, element: usize) {
        if let Some(elements) = self.ua_shadow_pseudo_elements.get_mut(&host) {
            elements.retain(|_, candidate| *candidate != element);
            if elements.is_empty() {
                self.ua_shadow_pseudo_elements.remove(&host);
            }
        }
        self.shadow_pseudo_ids.remove(&element);
        self.ultimate_originating_elements.remove(&element);
    }

    // cpp: html_meter_element.cc:156-176 DidAddUserAgentShadowRoot. These are
    // ordinary retained div elements in a UA ShadowRoot, matching Blink's
    // implementation tree and allowing SelectorChecker's kUAShadow relation
    // to cross from each typed custom pseudo target to the <meter> host.
    fn EnsureMeterUAShadowTree(&mut self, element: usize) {
        if !self.arena[element].IsElement("meter", DOMNamespace::kHTML) {
            return;
        }
        if self.meter_value_elements.contains_key(&element) {
            self.SyncMeterValueAppearance(element);
            return;
        }
        let root = self
            .ShadowRoot(element)
            .unwrap_or_else(|| self.AttachShadowRoot(element, DOMShadowRootMode::kUserAgent));
        let inner = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        let bar = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        let value = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        self.AppendChild(root, inner);
        self.AppendChild(inner, bar);
        self.AppendChild(bar, value);
        self.SetUAShadowPseudoElementIdentity(element, inner, "-webkit-meter-inner-element");
        self.SetUAShadowPseudoElementIdentity(element, bar, "-webkit-meter-bar");
        self.meter_value_elements.insert(element, value);
        self.SyncMeterValueAppearance(element);
    }

    fn MeterFloatingPointAttribute(&self, element: usize, name: &str, fallback: f64) -> f64 {
        self.arena[element]
            .FindAttribute(name)
            .and_then(|attribute| attribute.value.trim().parse::<f64>().ok())
            .filter(|value| value.is_finite())
            .unwrap_or(fallback)
    }

    // cpp: html_meter_element.cc:95-154,178-199. Attribute values are parsed,
    // clamped and classified before the retained value node changes identity.
    // The inline percentage is element-owned source state, not a style-engine
    // reconstruction of the meter's rendering.
    fn SyncMeterValueAppearance(&mut self, element: usize) {
        let Some(&value_element) = self.meter_value_elements.get(&element) else {
            return;
        };
        let min = self.MeterFloatingPointAttribute(element, "min", 0.0);
        let max = self
            .MeterFloatingPointAttribute(element, "max", 1.0_f64.max(min))
            .max(min);
        let value = self
            .MeterFloatingPointAttribute(element, "value", 0.0)
            .clamp(min, max);
        let low = self
            .MeterFloatingPointAttribute(element, "low", min)
            .clamp(min, max);
        let high = self
            .MeterFloatingPointAttribute(element, "high", max)
            .clamp(low, max);
        let optimum = self
            .MeterFloatingPointAttribute(element, "optimum", (max + min) / 2.0)
            .clamp(min, max);
        let pseudo_id = if optimum < low {
            if value <= low {
                "-webkit-meter-optimum-value"
            } else if value <= high {
                "-webkit-meter-suboptimum-value"
            } else {
                "-webkit-meter-even-less-good-value"
            }
        } else if high < optimum {
            if high <= value {
                "-webkit-meter-optimum-value"
            } else if low <= value {
                "-webkit-meter-suboptimum-value"
            } else {
                "-webkit-meter-even-less-good-value"
            }
        } else if low <= value && value <= high {
            "-webkit-meter-optimum-value"
        } else {
            "-webkit-meter-suboptimum-value"
        };
        let pseudo_changed = self.ShadowPseudoId(value_element) != Some(pseudo_id);
        self.SetUAShadowPseudoElementIdentity(element, value_element, pseudo_id);
        if pseudo_changed {
            self.InvalidateNodeStyle(value_element);
        }
        let percentage = if max <= min {
            0.0
        } else {
            (value - min) / (max - min) * 100.0
        };
        self.SetAttribute(
            value_element,
            DOMAttribute {
                local_name: "style".into(),
                value: format!("inline-size:{percentage}%;block-size:100%"),
                ..Default::default()
            },
        );
    }

    // cpp: html_progress_element.cc:135-153
    //      HTMLProgressElement::DidAddUserAgentShadowRoot. Progress uses three
    // real implementation elements. The typed custom pseudo selector path
    // reaches them through their retained ShadowPseudoId and UA ShadowRoot.
    fn EnsureProgressUAShadowTree(&mut self, element: usize) {
        if !self.arena[element].IsElement("progress", DOMNamespace::kHTML) {
            return;
        }
        if self.progress_value_elements.contains_key(&element) {
            self.SyncProgressValueAppearance(element);
            return;
        }
        let root = self
            .ShadowRoot(element)
            .unwrap_or_else(|| self.AttachShadowRoot(element, DOMShadowRootMode::kUserAgent));
        let inner = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        let bar = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        let value = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        self.AppendChild(root, inner);
        self.AppendChild(inner, bar);
        self.AppendChild(bar, value);
        self.SetUAShadowPseudoElementIdentity(element, inner, "-webkit-progress-inner-element");
        self.SetUAShadowPseudoElementIdentity(element, bar, "-webkit-progress-bar");
        self.SetUAShadowPseudoElementIdentity(element, value, "-webkit-progress-value");
        self.progress_value_elements.insert(element, value);
        self.SyncProgressValueAppearance(element);
    }

    fn ProgressFloatingPointAttribute(&self, element: usize, name: &str) -> Option<f64> {
        self.arena[element]
            .FindAttribute(name)
            .and_then(|attribute| attribute.value.trim().parse::<f64>().ok())
            .filter(|value| value.is_finite())
    }

    // cpp: html_progress_element.cc:92-133,166-168. Attribute presence owns
    // determinate state. Parsed values are normalized exactly before the
    // retained value element receives its author-equivalent inline lengths.
    fn SyncProgressValueAppearance(&mut self, element: usize) {
        let Some(&value_element) = self.progress_value_elements.get(&element) else {
            return;
        };
        let maximum = self
            .ProgressFloatingPointAttribute(element, "max")
            .filter(|value| *value > 0.0)
            .unwrap_or(1.0);
        let position = if self.arena[element].FindAttribute("value").is_none() {
            -1.0
        } else {
            let value = self
                .ProgressFloatingPointAttribute(element, "value")
                .unwrap_or(0.0)
                .max(0.0)
                .min(maximum);
            value / maximum
        };
        self.SetAttribute(
            value_element,
            DOMAttribute {
                local_name: "style".into(),
                value: format!("inline-size:{}%;block-size:100%", position * 100.0),
                ..Default::default()
            },
        );
    }

    // Chromium range_input_type.cc:274-290; native subclass pseudo identities
    // remain attached to the same UA DOM nodes consumed by style and layout.
    pub fn EnsureRangeUAShadowTree(&mut self, element: usize) {
        if self.InputType(element) != Some(DOMInputType::kRange) {
            return;
        }
        let root = self
            .ShadowRoot(element)
            .unwrap_or_else(|| self.AttachShadowRoot(element, DOMShadowRootMode::kUserAgent));
        if self.RangeSliderThumb(element).is_some() {
            return;
        }
        let track = self.CreateElement(
            DOMNamespace::kHTML,
            "div".into(),
            vec![DOMAttribute {
                local_name: "id".into(),
                value: "track".into(),
                ..Default::default()
            }],
        );
        self.range_slider_shadow_parts
            .insert(track, (element, DOMRangeShadowPart::Track));
        self.SetUAShadowPseudoElementIdentity(element, track, "-webkit-slider-runnable-track");
        let thumb = self.CreateElement(
            DOMNamespace::kHTML,
            "div".into(),
            vec![DOMAttribute {
                local_name: "id".into(),
                value: "thumb".into(),
                ..Default::default()
            }],
        );
        self.range_slider_shadow_parts
            .insert(thumb, (element, DOMRangeShadowPart::Thumb));
        self.SetUAShadowPseudoElementIdentity(element, thumb, "-webkit-slider-thumb");
        self.AppendChild(track, thumb);
        let container = self.CreateElement(DOMNamespace::kHTML, "div".into(), Vec::new());
        self.range_slider_shadow_parts
            .insert(container, (element, DOMRangeShadowPart::Container));
        self.SetUAShadowPseudoElementIdentity(element, container, "-webkit-slider-container");
        self.AppendChild(container, track);
        self.AppendChild(root, container);
    }
    /// Native subclass identity survives detach, just as the DOM node does.
    /// EventTarget factories must not infer these classes from ids or tags.
    pub fn RangeSliderShadowPart(&self, node: usize) -> Option<(usize, DOMRangeShadowPart)> {
        self.range_slider_shadow_parts.get(&node).copied()
    }
    pub fn RangeSliderThumb(&self, element: usize) -> Option<usize> {
        if self.InputType(element) != Some(DOMInputType::kRange) {
            return None;
        }
        self.UAShadowPseudoElement(element, "-webkit-slider-thumb")
            .filter(|thumb| self.ContainingShadowRoot(*thumb) == self.ShadowRoot(element))
    }

    pub fn UAShadowPseudoElement(&self, element: usize, pseudo_id: &str) -> Option<usize> {
        self.ua_shadow_pseudo_elements
            .get(&element)
            .and_then(|elements| elements.get(pseudo_id))
            .copied()
    }

    pub fn ShadowPseudoId(&self, element: usize) -> Option<&str> {
        self.shadow_pseudo_ids.get(&element).map(String::as_str)
    }

    // cpp: SearchFieldCancelButtonElement::DefaultEventHandler. Interaction
    // asks DOM for the typed retained-owner relation instead of decoding an id
    // string or treating arbitrary UA-shadow divs as search buttons.
    pub fn SearchCancelButtonHost(&self, element: usize) -> Option<usize> {
        (self.ShadowPseudoId(element) == Some("-webkit-search-cancel-button"))
            .then(|| self.ultimate_originating_elements.get(&element).copied())
            .flatten()
            .filter(|host| self.InputType(*host) == Some(DOMInputType::kSearch))
    }

    pub fn UltimateOriginatingElement(&self, element: usize) -> usize {
        self.ultimate_originating_elements
            .get(&element)
            .copied()
            .unwrap_or(element)
    }

    // cpp: dom/document.cc:137-151
    pub fn EnsureTemplateContents(&mut self, template_element: usize) -> usize {
        if !self.arena[template_element].IsElement("template", DOMNamespace::kHTML) {
            invalid_argument("template contents require an HTML template element");
        }
        if let Some(existing) = self.TemplateContents(template_element) {
            return existing;
        }
        let fragment = self.CreateDocumentFragment();
        if let Some(owner) = self.arena[template_element].owner_document_handle {
            self.AdoptNodeByHandle(fragment, owner);
        }
        self.template_contents
            .insert(self.arena[template_element].id, fragment);
        self.NotifyNodeReference(crate::node_references::NodeReferenceChange::Template {
            owner: self.arena[template_element].id,
            content: self.arena[fragment].id,
        });
        fragment
    }
    pub fn TemplateContents(&self, template_element: usize) -> Option<usize> {
        self.template_contents
            .get(&self.arena[template_element].id)
            .copied()
    }

    // cpp: element.cc:7768-7888; shadow_root.h:97-110.
    pub fn AttachShadowRoot(&mut self, host: usize, mode: DOMShadowRootMode) -> usize {
        self.AttachShadowRootWithSlotAssignment(host, mode, DOMSlotAssignmentMode::kNamed)
    }

    pub fn AttachShadowRootWithSlotAssignment(
        &mut self,
        host: usize,
        mode: DOMShadowRootMode,
        slot_assignment: DOMSlotAssignmentMode,
    ) -> usize {
        if self.arena[host].node_type != DOMNodeType::kElement {
            invalid_argument("a shadow host must be an element");
        }
        if self.shadow_roots.contains_key(&host) {
            invalid_argument("a shadow root is already attached");
        }
        let root = self.CreateDocumentFragment();
        if let Some(owner) = self.arena[host].owner_document_handle {
            self.AdoptNodeByHandle(root, owner);
        }
        self.shadow_roots
            .insert(host, (root, mode, slot_assignment));
        self.shadow_hosts.insert(root, host);
        self.SetTreeScopeForSubtree(root, root);
        self.NotifyNodeReference(crate::node_references::NodeReferenceChange::Shadow {
            host: self.arena[host].id,
            root: self.arena[root].id,
        });
        // cpp: Element::CreateAndAttachShadowRoot; Node::RemovedFromFlatTree.
        // Even an empty root removes the old light children from layout. This
        // changes attachment/geometry independently of computed-style equality.
        self.RecordFlatTreeChange(host);
        self.UpdateDirectionalityForMutation(host);
        self.InvalidateNodeStyle(host);
        // Element::CreateAndAttachShadowRoot removes this host's light
        // children from the flat tree. Revisit that subtree, including its
        // new shadow scope, without invalidating unrelated document styles.
        if self.ShouldRecordStyleChange(host) {
            self.style_state.inserted_style_subtrees.insert(host);
        }
        root
    }

    // Chromium: ShadowRoot::delegatesFocus. Retain the actual root's flag;
    // a JS wrapper property cannot substitute for the canonical DOM owner.
    pub fn ShadowRootDelegatesFocus(&self, host: usize) -> bool {
        self.ShadowRoot(host)
            .is_some_and(|root| self.shadow_roots_delegating_focus.contains(&root))
    }
    pub fn SetShadowRootDelegatesFocus(&mut self, root: usize, delegates: bool) {
        assert!(self.shadow_hosts.contains_key(&root));
        if delegates {
            self.shadow_roots_delegating_focus.insert(root);
        } else {
            self.shadow_roots_delegating_focus.remove(&root);
        }
    }
    pub fn ShadowRoot(&self, host: usize) -> Option<usize> {
        self.shadow_roots.get(&host).map(|&(root, _, _)| root)
    }

    pub fn ShadowRootMode(&self, root: usize) -> Option<DOMShadowRootMode> {
        let host = self.shadow_hosts.get(&root)?;
        self.shadow_roots.get(host).map(|&(_, mode, _)| mode)
    }

    pub fn ShadowRootSlotAssignment(&self, root: usize) -> Option<DOMSlotAssignmentMode> {
        let host = self.shadow_hosts.get(&root)?;
        self.shadow_roots
            .get(host)
            .map(|&(_, _, slot_assignment)| slot_assignment)
    }

    pub fn ShadowHost(&self, root: usize) -> Option<usize> {
        self.shadow_hosts.get(&root).copied()
    }

    // cpp: node.h:1363; tree_scope_adopter.cc:53-69,290-294. TreeScope is a
    // retained Node owner and survives ordinary removal from the parent tree.
    pub fn TreeScopeRoot(&self, node: usize) -> usize {
        self.arena[node].tree_scope_root
    }

    /// The current ordinary-tree root is distinct from retained TreeScope.
    /// Connectivity and TreeScope element registration use this live relation.
    fn TreeRoot(&self, node: usize) -> usize {
        let mut root = node;
        while let Some(parent) = self.arena[root].parent {
            root = parent;
        }
        root
    }

    fn SetTreeScopeForSubtree(&mut self, node: usize, scope: usize) {
        let scope = if self.shadow_hosts.contains_key(&node) {
            node
        } else {
            scope
        };
        self.arena[node].tree_scope_root = scope;
        let children = self.arena[node].children.clone();
        for child in children {
            self.SetTreeScopeForSubtree(child, scope);
        }
    }

    fn RegisteredTreeScope(&self, node: usize) -> Option<usize> {
        let scope = self.TreeRoot(node);
        (scope == self.root || self.shadow_hosts.contains_key(&scope)).then_some(scope)
    }

    fn AddElementById(&mut self, scope: usize, id: &str, element: usize) {
        if id.is_empty() {
            return;
        }
        self.tree_scope_elements_by_id
            .entry(scope)
            .or_default()
            .entry(id.to_owned())
            .or_default()
            .insert(element);
    }

    fn RemoveElementById(&mut self, scope: usize, id: &str, element: usize) {
        let mut remove_id = false;
        if let Some(elements) = self
            .tree_scope_elements_by_id
            .get_mut(&scope)
            .and_then(|ids| ids.get_mut(id))
        {
            elements.remove(&element);
            remove_id = elements.is_empty();
        }
        if remove_id {
            let ids = self
                .tree_scope_elements_by_id
                .get_mut(&scope)
                .expect("registered TreeScope ID owner");
            ids.remove(id);
            if ids.is_empty() {
                self.tree_scope_elements_by_id.remove(&scope);
            }
        }
    }

    fn UpdateSubtreeIdsInTreeScope(&mut self, root: usize, scope: usize, add: bool) {
        let is_shadow_scope = self.shadow_hosts.contains_key(&scope);
        let slots = self.slots_by_name.entry(scope).or_default();
        let arena = &self.arena;
        let scopes = &mut self.tree_scope_elements_by_id;
        let ids = scopes.entry(scope).or_default();
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            if is_shadow_scope && arena[node].IsHTMLElement("slot") {
                let name = arena[node]
                    .AttributeValueUTF16("name")
                    .unwrap_or_else(|| Rc::from([]));
                if add {
                    slots.entry(name.clone()).or_default().insert(node);
                } else if let Some(nodes) = slots.get_mut(&name) {
                    nodes.remove(&node);
                }
            }
            if let Some(id) = (arena[node].node_type == DOMNodeType::kElement)
                .then(|| arena[node].FindAttribute("id"))
                .flatten()
                .map(|attribute| attribute.value.as_str())
                .filter(|id| !id.is_empty())
            {
                if add {
                    ids.entry(id.to_owned()).or_default().insert(node);
                } else if let Some(elements) = ids.get_mut(id) {
                    elements.remove(&node);
                }
            }
            pending.extend_from_slice(&arena[node].children);
        }
        if !add {
            slots.retain(|_, nodes| !nodes.is_empty());
            ids.retain(|_, elements| !elements.is_empty());
            if ids.is_empty() {
                scopes.remove(&scope);
            }
        }
    }

    fn UpdateElementIdInTreeScope(
        &mut self,
        element: usize,
        old_id: Option<&str>,
        new_id: Option<&str>,
    ) {
        let Some(scope) = self.RegisteredTreeScope(element) else {
            return;
        };
        if let Some(id) = old_id {
            self.RemoveElementById(scope, id, element);
        }
        if let Some(id) = new_id {
            self.AddElementById(scope, id, element);
        }
    }

    pub fn ContainingShadowRoot(&self, node: usize) -> Option<usize> {
        let root = self.TreeScopeRoot(node);
        self.shadow_hosts.contains_key(&root).then_some(root)
    }

    pub fn OwnerShadowHost(&self, node: usize) -> Option<usize> {
        self.ContainingShadowRoot(node)
            .and_then(|root| self.ShadowHost(root))
    }

    // cpp: element.cc:12855-12864. The retained map may exist but be empty
    // after an exportparts removal; HasPartNamesMap follows NamesMap::size().
    pub fn HasPartNamesMap(&self, element: usize) -> bool {
        self.PartNamesMap(element).is_some_and(|map| !map.IsEmpty())
    }

    pub fn PartNamesMap(&self, element: usize) -> Option<&DOMPartNamesMap> {
        self.part_names_maps.get(&element)
    }

    // Chromium: core/dom/user_action_element_traversal.h:41-56.
    // UserActionPseudosStopAtTopLayer is stable in the translated revision.
    pub fn UserActionElementParent(&self, node: usize) -> Option<usize> {
        if self.IsInTopLayer(node) {
            None
        } else {
            self.FlatTreeParentElement(node)
        }
    }

    // Chromium: core/dom/node.cc:1896-1930, specialized for the user-action
    // parent used by Document::SetFocusedElement. Reads the resident flat tree.
    pub fn UserActionElementCommonAncestor(&self, node: usize, other: usize) -> Option<usize> {
        if node == other {
            return Some(node);
        }
        if self.NodeDocumentHandle(node) != self.NodeDocumentHandle(other) {
            return None;
        }
        let mut depth = 0;
        let mut current = Some(node);
        while let Some(index) = current {
            if index == other {
                return Some(index);
            }
            depth += 1;
            current = self.UserActionElementParent(index);
        }
        let mut other_depth = 0;
        current = Some(other);
        while let Some(index) = current {
            if index == node {
                return Some(node);
            }
            other_depth += 1;
            current = self.UserActionElementParent(index);
        }
        let mut first = Some(node);
        let mut second = Some(other);
        while depth > other_depth {
            first = first.and_then(|i| self.UserActionElementParent(i));
            depth -= 1;
        }
        while other_depth > depth {
            second = second.and_then(|i| self.UserActionElementParent(i));
            other_depth -= 1;
        }
        while let Some(index) = first {
            if first == second {
                return first;
            }
            first = self.UserActionElementParent(index);
            second = second.and_then(|i| self.UserActionElementParent(i));
        }
        debug_assert!(second.is_none());
        None
    }

    pub fn ParentOrShadowHostElement(&self, node: usize) -> Option<usize> {
        if let Some(parent) = self.arena[node].parent {
            if self.arena[parent].node_type == DOMNodeType::kElement {
                return Some(parent);
            }
            if let Some(host) = self.ShadowHost(parent) {
                return Some(host);
            }
            return None;
        }
        self.ShadowHost(node)
    }

    /// Canonical FlatTreeTraversal::TraverseParent, including Document and
    /// ordinary DocumentFragment parents (core/dom/flat_tree_traversal.cc:161-191).
    pub fn FlatTreeParent(&self, node: usize) -> Option<usize> {
        let parent = self.arena[node].parent?;
        if self.arena[parent].node_type == DOMNodeType::kElement {
            if self.ShadowRoot(parent).is_some() {
                return self.AssignedSlot(node);
            }
            if self.IsSlotSupportingAssignment(parent) {
                return self.AssignedNodes(parent).is_empty().then_some(parent);
            }
        }
        Some(self.ShadowHost(parent).unwrap_or(parent))
    }

    /// Element-only projection used by UserActionElementTraversal and style
    /// attachment. All flat-tree parent decisions stay in the canonical owner.
    pub fn FlatTreeParentElement(&self, node: usize) -> Option<usize> {
        self.FlatTreeParent(node)
            .filter(|&parent| self.Node(parent).Type() == DOMNodeType::kElement)
    }
    // Source: LayoutTreeBuilderTraversal::Parent/IsLayoutParent/LayoutParent,
    // layout_tree_builder_traversal.cc:53-78. DOM nodes exclude computed pseudo
    // owners; their source relations are supplied separately by pseudo owners.
    pub fn LayoutTreeBuilderParent(&self, node: usize) -> Option<usize> {
        let mut parent = self.FlatTreeParent(node);
        while let Some(node) = parent {
            if !self
                .ResolvedStyleFor(node)
                .is_some_and(|style| style.own_display_contents)
            {
                return Some(node);
            }
            parent = self.FlatTreeParent(node);
        }
        None
    }

    /// Direct active flat-tree children, retaining slot elements as attachment
    /// and inheritance parents. This is not assignedNodes({flatten:true}).
    /// cpp: Element::AttachLayoutTree; HTMLSlotElement::RecalcFlatTreeChildren.
    pub fn FlatTreeChildren(&self, node: usize) -> std::borrow::Cow<'_, [usize]> {
        if let Some(root) = self.ShadowRoot(node) {
            return std::borrow::Cow::Borrowed(self.arena[root].Children());
        }
        if self.IsSlotSupportingAssignment(node) {
            let assigned = self.AssignedNodes(node);
            if !assigned.is_empty() {
                return std::borrow::Cow::Owned(assigned);
            }
            return std::borrow::Cow::Owned(
                self.arena[node]
                    .children
                    .iter()
                    .copied()
                    .filter(|&child| {
                        matches!(
                            self.arena[child].node_type,
                            DOMNodeType::kElement | DOMNodeType::kText
                        )
                    })
                    .collect(),
            );
        }
        std::borrow::Cow::Borrowed(self.arena[node].Children())
    }

    // cpp: Node::FlatTreeParentChanged / RemovedFromFlatTree. Reattachment
    // revokes query geometry even when all newly inherited CSS values compare
    // equal; Page and Style consume these existing document-owned facts.
    fn MayHaveRenderedSubtree(&self, node: usize) -> bool {
        // cpp: ContainerNode::ChildrenChanged (1477-1493). A clean element
        // without a rendered subtree has no layout children to invalidate.
        // DOM, stylesheet, directionality and semantic-owner notifications
        // still run; a later null -> non-null style attaches the entire branch.
        if !self.IsConnectedToDocument(node) {
            return false;
        }
        if !self.style_state.all_dirty
            && self.arena[node].node_type == DOMNodeType::kElement
            && self.ResolvedStyleFor(node).is_none_or(|style| {
                unsafe { style.computed_style_owner.Get().as_ref() }
                    .is_some_and(|source| source.IsEnsuredInDisplayNone())
                    || (!style.generates_box && !style.display_contents)
            })
        {
            return false;
        }
        true
    }
    fn RecordLayoutInputChange(&mut self, node: usize) {
        if !self.IsConnectedToDocument(node) {
            return;
        }
        self.style_state
            .layout_tree_roots
            .insert(self.arena[node].id);
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
    }
    fn RecordLayoutGeometryChange(&mut self, node: usize) -> bool {
        if !self.MayHaveRenderedSubtree(node) {
            return false;
        }
        self.style_state
            .layout_tree_roots
            .insert(self.arena[node].id);
        if self.AffectsSVGResourceClients(node) {
            self.style_state.layout_inputs_require_tree = true;
        }
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        self.style_state.measurement_geometry_revision = self
            .style_state
            .measurement_geometry_revision
            .wrapping_add(1);
        true
    }
    fn RecordFlatTreeChange(&mut self, node: usize) {
        self.CanvasFlatTreeChanged(node);
        if self.IsConnectedToDocument(node) {
            self.BumpSVGResourceDOMRevision();
        }
        if self.RecordLayoutGeometryChange(node) {
            self.style_state
                .impact
                .Merge(crate::style_state::StyleUpdateImpact::TREE);
        }
    }

    // cpp: HTMLOptionElement::ChildrenChanged/DidChangeTextContent ->
    // HTMLSelectElement::OptionElementChildrenChanged. The rendered menu-list
    // label can depend on text in an option without its own layout object.
    fn RecordSelectContentGeometryChange(&mut self, node: usize) {
        let select = if self.arena[node].IsHTMLElement("select") {
            Some(node)
        } else {
            self.NearestAncestorElementNamed(node, "select")
        };
        if let Some(select) = select {
            // OptionElementChildrenChanged cc:538-540; option/optgroup
            // insertion/removal cc:962,1019,1039 use this same dirty cache.
            self.SetControlNeedsValidityCheck(select);
            self.RecordLayoutGeometryChange(select);
        }
    }

    fn AttributeMayChangeFlatTree(&self, element: usize, name: &str, namespace: &str) -> bool {
        namespace.is_empty()
            && ((name == "slot"
                && self.arena[element]
                    .parent
                    .is_some_and(|parent| self.ShadowRoot(parent).is_some()))
                || (name == "name" && self.IsSlotSupportingAssignment(element)))
    }

    pub fn IsInShadowTree(&self, node: usize) -> bool {
        self.ContainingShadowRoot(node).is_some()
    }

    // cpp: html/media/html_media_element.h:145. HTMLMediaElement is the
    // shared retained DOM owner for HTMLAudioElement and HTMLVideoElement.
    pub fn IsMediaElement(&self, node: usize) -> bool {
        matches!(
            self.arena[node].class,
            DOMNodeClass::HTMLMediaElement { .. }
        )
    }

    pub fn IsVideoElement(&self, node: usize) -> bool {
        matches!(
            self.arena[node].class,
            DOMNodeClass::HTMLMediaElement { is_video: true }
        )
    }
    pub fn IsAudioElement(&self, node: usize) -> bool {
        matches!(
            self.arena[node].class,
            DOMNodeClass::HTMLMediaElement { is_video: false }
        )
    }

    // cpp: html_select_element.cc:2164-2186,2287-2294. Inputs are counted for
    // their nearest associated select. option/hr/datalist and invalid nested
    // optgroup structures terminate that association walk.
    pub fn NumDescendantInputs(&self, select: usize) -> usize {
        match &self.arena[select].class {
            DOMNodeClass::HTMLSelectElement(data) => data.num_descendant_inputs,
            _ => 0,
        }
    }

    // cpp: style_engine.cc:290-316,648-725. Each connected TreeScope owns a
    // separate active stylesheet collection. ShadowRoot is connected through
    // its host edge, never through Parent().
    pub fn IsConnectedToDocument(&self, node: usize) -> bool {
        let mut root = self.TreeRoot(node);
        loop {
            if root == self.root {
                return true;
            }
            let Some(host) = self.ShadowHost(root) else {
                return false;
            };
            root = self.TreeRoot(host);
        }
    }

    pub fn ActiveTreeScopes(&self) -> Vec<usize> {
        let mut scopes = self
            .shadow_hosts
            .keys()
            .copied()
            .filter(|&root| self.IsConnectedToDocument(root))
            .collect::<Vec<_>>();
        scopes.sort_unstable();
        scopes.insert(0, self.root);
        scopes
    }

    fn TreeScopeHandle(&self, scope: usize) -> Option<DOMOwnerHandle> {
        (scope == self.root || self.shadow_hosts.contains_key(&scope)).then(|| DOMOwnerHandle {
            arena_id: self.arena_id,
            node_id: self.arena[scope].id,
        })
    }

    pub fn StyleSheetTreeScope(&self, sheet: &CSSStyleSheet) -> Option<usize> {
        if sheet.owner_node_id == 0 {
            return self
                .ownerless_style_sheet_scopes
                .get(&sheet.Identity())
                .copied();
        }
        let owner = self.FindNodeById(sheet.owner_node_id)?;
        self.IsConnectedToDocument(owner)
            .then(|| self.TreeScopeRoot(owner))
    }

    /// Registers one `new CSSStyleSheet()` backing in its constructor
    /// Document. It stays inactive until a Document or ShadowRoot adopts it.
    pub fn RegisterConstructedStyleSheet(
        &mut self,
        sheet: CSSStyleSheet,
    ) -> Result<u64, CSSOMAdoptionError> {
        let identity = sheet.Identity();
        let was_active = self.cssom.IsConstructedStyleSheetAdopted(identity);
        let changed = self.cssom.RegisterConstructedStyleSheet(sheet)?;
        if changed && was_active {
            self.style_state.sheet_revision += 1;
        }
        Ok(identity)
    }

    /// Replaces a TreeScope's observable adoptedStyleSheets sequence. The
    /// identity list intentionally preserves order and duplicates, matching
    /// Blink's ObservableArray callbacks and per-scope adoption counts.
    pub fn SetAdoptedStyleSheets(
        &mut self,
        scope: usize,
        identities: Vec<u64>,
    ) -> Result<bool, CSSOMAdoptionError> {
        let Some(scope) = self.TreeScopeHandle(scope) else {
            return Err(CSSOMAdoptionError::InvalidTreeScope);
        };
        let changed = self
            .cssom
            .SetAdoptedStyleSheets(scope, identities, |_| {})?;
        if changed {
            self.style_state.sheet_revision += 1;
        }
        Ok(changed)
    }

    pub fn AdoptedStyleSheetsForTreeScope(&self, scope: usize) -> Vec<&CSSStyleSheet> {
        self.TreeScopeHandle(scope)
            .map(|scope| self.cssom.GetAdoptedStyleSheets(&scope))
            .unwrap_or_default()
    }

    pub fn IsStyleSheetAdoptedByTreeScope(&self, identity: u64, scope: usize) -> bool {
        self.TreeScopeHandle(scope)
            .is_some_and(|scope| self.cssom.IsStyleSheetAdoptedBy(identity, &scope))
    }

    pub fn StyleSheetAppliesToTreeScope(&self, sheet: &CSSStyleSheet, scope: usize) -> bool {
        self.StyleSheetTreeScope(sheet) == Some(scope)
            || self.IsStyleSheetAdoptedByTreeScope(sheet.Identity(), scope)
    }

    /// cpp: scoped_style_resolver.cc:446-455. A Document-adopted sheet has no
    /// implicit-scope trigger. A ShadowRoot-adopted sheet is triggered by that
    /// root's host; this is independent of owner_node_id.
    pub fn AdoptedStyleSheetImplicitScopeTrigger(
        &self,
        identity: u64,
        scope: usize,
    ) -> Option<usize> {
        self.IsStyleSheetAdoptedByTreeScope(identity, scope)
            .then(|| self.ShadowHost(scope))
            .flatten()
    }

    pub fn ConstructedStyleSheet(&self, identity: u64) -> Option<&CSSStyleSheet> {
        self.cssom.GetConstructedStyleSheet(identity)
    }

    pub fn ReplaceConstructedStyleSheet(
        &mut self,
        identity: u64,
        sheet: CSSStyleSheet,
    ) -> Result<bool, CSSOMAdoptionError> {
        let active = self.cssom.IsConstructedStyleSheetAdopted(identity);
        let changed = self
            .cssom
            .ReplaceConstructedStyleSheet(identity, sheet, |_| {})?;
        if changed && active {
            self.style_state.sheet_revision += 1;
        }
        Ok(changed)
    }

    pub fn ConstructedStyleSheetOwnerDocument(&self, identity: u64) -> Option<DOMOwnerHandle> {
        self.ConstructedStyleSheet(identity)
            .map(|_| self.RootHandle())
    }

    pub fn AdoptedTreeScopesForStyleSheet(&self, identity: u64) -> Vec<usize> {
        let mut scopes = self.shadow_hosts.keys().copied().collect::<Vec<_>>();
        scopes.push(self.root);
        scopes.sort_unstable();
        scopes.retain(|&scope| self.IsStyleSheetAdoptedByTreeScope(identity, scope));
        scopes
    }

    // cpp: slot_assignment.cc:372-383. Named slot assignment selects the
    // first slot of the same name in shadow-tree order. Manual slotting is a
    // separate retained owner below.
    pub fn AssignedSlot(&self, node: usize) -> Option<usize> {
        if !matches!(
            self.arena[node].node_type,
            DOMNodeType::kElement | DOMNodeType::kText
        ) {
            return None;
        }
        let host = self.arena[node].parent?;
        let root = self.ShadowRoot(host)?;
        if self.ShadowRootSlotAssignment(root) == Some(DOMSlotAssignmentMode::kManual) {
            let slot = self.ManuallyAssignedSlot(node)?;
            return (self.ContainingShadowRoot(slot) == Some(root)).then_some(slot);
        }
        let requested_name = self.arena[node]
            .AttributeValueUTF16("slot")
            .unwrap_or_else(|| Rc::from([]));
        let matches = self.slots_by_name.get(&root)?.get(&requested_name)?;
        if matches.len() == 1 {
            return matches.iter().copied().next();
        }
        let mut stack = vec![root];
        while let Some(current) = stack.pop() {
            if matches.contains(&current) {
                return Some(current);
            }
            stack.extend(self.arena[current].children.iter().rev().copied());
        }
        None
    }

    /// cpp: html_slot_element.cc:199-260. Retains the ordered, de-duplicated
    /// imperative assignment owner and moves nodes away from previous manual
    /// slots. Assignment may be recorded while a slot is disconnected; the
    /// flat-tree relation becomes active only after AssignedSlot validates the
    /// slot/node against the same manual ShadowRoot and host.
    pub fn AssignSlotNodes(&mut self, slot: usize, nodes: Vec<usize>) -> bool {
        if slot >= self.arena.len() || !self.arena[slot].IsHTMLElement("slot") {
            invalid_argument("manual slot assignment requires an HTML slot element");
        }
        let mut seen = HashSet::new();
        let mut added = Vec::new();
        for node in nodes {
            if node >= self.arena.len()
                || !matches!(
                    self.arena[node].node_type,
                    DOMNodeType::kElement | DOMNodeType::kText
                )
            {
                invalid_argument("manual slot assignment accepts only Element or Text nodes");
            }
            if seen.insert(node) {
                added.push(node);
            }
        }

        let old = self
            .manually_assigned_nodes
            .get(&slot)
            .cloned()
            .unwrap_or_default();
        let mut changed_slots = HashSet::new();
        for &node in &added {
            if let Some(previous) = self.manually_assigned_slots.insert(node, slot) {
                if previous != slot {
                    if let Some(previous_nodes) = self.manually_assigned_nodes.get_mut(&previous) {
                        previous_nodes.retain(|&candidate| candidate != node);
                        if previous_nodes.is_empty() {
                            self.manually_assigned_nodes.remove(&previous);
                        }
                    }
                    changed_slots.insert(previous);
                }
            }
        }
        for &node in &old {
            if !seen.contains(&node) && self.manually_assigned_slots.get(&node) == Some(&slot) {
                self.manually_assigned_slots.remove(&node);
            }
        }
        if added.is_empty() {
            self.manually_assigned_nodes.remove(&slot);
        } else {
            self.manually_assigned_nodes.insert(slot, added.clone());
        }
        if old != added {
            changed_slots.insert(slot);
        }
        if changed_slots.is_empty() {
            return false;
        }
        for changed_slot in changed_slots {
            self.RecordNonAppendChildrenChange(changed_slot, None);
        }
        true
    }

    pub fn ManuallyAssignedSlot(&self, node: usize) -> Option<usize> {
        self.manually_assigned_slots.get(&node).copied()
    }

    pub fn ManuallyAssignedNodes(&self, slot: usize) -> &[usize] {
        self.manually_assigned_nodes
            .get(&slot)
            .map_or(&[], Vec::as_slice)
    }

    pub fn AssignedSlotWithoutRecalc(&self, node: usize) -> Option<usize> {
        self.AssignedSlot(node)
    }
    pub fn HasAssignedNodesNoRecalc(&self, slot: usize) -> bool {
        self.AssignedNodesIterator(slot).next().is_some()
    }
    fn AssignedNodesIterator(&self, slot: usize) -> Box<dyn Iterator<Item = usize> + '_> {
        if !self.IsSlotSupportingAssignment(slot) {
            return Box::new(std::iter::empty());
        }
        let Some(root) = self.ContainingShadowRoot(slot) else {
            return Box::new(std::iter::empty());
        };
        let Some(host) = self.ShadowHost(root) else {
            return Box::new(std::iter::empty());
        };
        match self.ShadowRootSlotAssignment(root) {
            Some(DOMSlotAssignmentMode::kManual) => Box::new(
                self.ManuallyAssignedNodes(slot)
                    .iter()
                    .copied()
                    .filter(move |&node| {
                        self.arena[node].parent == Some(host)
                            && self.ManuallyAssignedSlot(node) == Some(slot)
                    }),
            ),
            Some(DOMSlotAssignmentMode::kNamed) => Box::new(
                self.arena[host]
                    .children
                    .iter()
                    .copied()
                    .filter(move |&node| self.AssignedSlot(node) == Some(slot)),
            ),
            None => Box::new(std::iter::empty()),
        }
    }
    pub fn AssignedNodes(&self, slot: usize) -> Vec<usize> {
        self.AssignedNodesIterator(slot).collect()
    }

    /// Blink HTMLSlotElement::FlattenedAssignedNodes / CollectFlattenedAssignedNodes.
    /// Assignment and fallback are resolved in their source owner, never in a
    /// script wrapper or a second layout-specific distribution algorithm.
    pub fn FlattenedAssignedNodes(&self, slot: usize) -> Vec<usize> {
        if !self.IsSlotSupportingAssignment(slot) {
            return Vec::new();
        }
        let mut nodes = self.AssignedNodes(slot);
        if nodes.is_empty() {
            nodes.extend(self.arena[slot].children.iter().copied().filter(|&child| {
                matches!(
                    self.arena[child].node_type,
                    DOMNodeType::kElement | DOMNodeType::kText
                )
            }));
        }
        let mut flattened = Vec::new();
        for node in nodes {
            if self.IsSlotSupportingAssignment(node) {
                flattened.extend(self.FlattenedAssignedNodes(node));
            } else {
                flattened.push(node);
            }
        }
        flattened
    }

    pub fn IsSlotSupportingAssignment(&self, node: usize) -> bool {
        self.arena[node].IsHTMLElement("slot") && self.IsInShadowTree(node)
    }

    fn DirectionAttribute(&self, element: usize) -> Option<&str> {
        (self.arena[element].Namespace() == DOMNamespace::kHTML)
            .then(|| self.arena[element].FindAttribute("dir"))
            .flatten()
            .map(|attribute| attribute.value.as_str())
            .filter(|value| {
                value.eq_ignore_ascii_case("ltr")
                    || value.eq_ignore_ascii_case("rtl")
                    || value.eq_ignore_ascii_case("auto")
            })
    }

    pub fn HasDirectionAuto(&self, element: usize) -> bool {
        self.DirectionAttribute(element)
            .is_some_and(|value| value.eq_ignore_ascii_case("auto"))
            || (self.arena[element].IsHTMLElement("bdi")
                && self.DirectionAttribute(element).is_none())
    }

    // cpp: html_element.cc:419-433; html_input_element.cc:934-939.
    // `dir=auto` is materialized by StyleResolver from CachedDirectionality;
    // the remaining valid direction values are ordinary presentation hints.
    // Telephone inputs also establish an LTR hint when no valid `dir` value
    // overrides the type's default direction.
    pub fn DirectionalityPresentationalHint(&self, element: usize) -> Option<TextDirection> {
        match self.DirectionAttribute(element) {
            Some(value) if value.eq_ignore_ascii_case("ltr") => Some(TextDirection::kLtr),
            Some(value) if value.eq_ignore_ascii_case("rtl") => Some(TextDirection::kRtl),
            Some(_) => None,
            None if self.arena[element].IsHTMLElement("input")
                && self.arena[element]
                    .FindAttribute("type")
                    .is_some_and(|attribute| attribute.value.eq_ignore_ascii_case("tel")) =>
            {
                Some(TextDirection::kLtr)
            }
            None => None,
        }
    }

    fn AutoDirectionalityUsesControlValue(&self, element: usize) -> bool {
        if self.arena[element].IsHTMLElement("textarea") {
            return true;
        }
        if !self.arena[element].IsHTMLElement("input") {
            return false;
        }
        self.arena[element]
            .FindAttribute("type")
            .map(|attribute| attribute.value.to_ascii_lowercase())
            .is_none_or(|kind| {
                matches!(
                    kind.as_str(),
                    "" | "text"
                        | "search"
                        | "email"
                        | "tel"
                        | "url"
                        | "password"
                        | "number"
                        | "button"
                        | "submit"
                        | "reset"
                        | "hidden"
                )
            })
    }

    fn TextDirection(text: &str) -> Option<TextDirection> {
        BidiParagraph::BaseDirectionForString(&StringView::from(text), None)
    }

    fn IncludeInAutoDirectionalityTraversal(&self, element: usize) -> bool {
        if matches!(
            self.arena[element].Name(),
            "bdi" | "script" | "style" | "textarea"
        ) || self.ShadowPseudoId(element) == Some("-webkit-input-placeholder")
        {
            return false;
        }
        self.DirectionAttribute(element).is_none()
    }

    fn ContainedTextDirectionality(
        &self,
        root: usize,
        cache: &HashMap<usize, TextDirection>,
    ) -> Option<TextDirection> {
        for &child in self.arena[root].Children() {
            if self.arena[child].Type() == DOMNodeType::kText {
                if let Some(direction) = Self::TextDirection(self.arena[child].Data()) {
                    return Some(direction);
                }
                continue;
            }
            if self.arena[child].Type() != DOMNodeType::kElement
                || !self.IncludeInAutoDirectionalityTraversal(child)
            {
                continue;
            }
            if self.IsSlotSupportingAssignment(child) {
                if let Some(host) = self.OwnerShadowHost(child) {
                    return Some(cache.get(&host).copied().unwrap_or(TextDirection::kLtr));
                }
            }
            if let Some(direction) = self.ContainedTextDirectionality(child, cache) {
                return Some(direction);
            }
        }
        None
    }

    fn ResolveAutoDirectionality(
        &self,
        element: usize,
        cache: &HashMap<usize, TextDirection>,
    ) -> Option<TextDirection> {
        if self.AutoDirectionalityUsesControlValue(element) {
            return Self::TextDirection(&self.ControlValue(element)).or(Some(TextDirection::kLtr));
        }
        if self.IsSlotSupportingAssignment(element) {
            let assigned = self.AssignedNodes(element);
            if !assigned.is_empty() {
                for node in assigned {
                    if self.arena[node].Type() == DOMNodeType::kText {
                        if let Some(direction) = Self::TextDirection(self.arena[node].Data()) {
                            return Some(direction);
                        }
                    } else if self.arena[node].Type() == DOMNodeType::kElement
                        && self.IncludeInAutoDirectionalityTraversal(node)
                    {
                        if let Some(direction) = self.ContainedTextDirectionality(node, cache) {
                            return Some(direction);
                        }
                    }
                }
                return None;
            }
        }
        self.ContainedTextDirectionality(element, cache)
    }

    // cpp: Element::RecomputeDirectionFromParent and
    // UpdateDirectionalityAndDescendant. Cached values belong to elements;
    // mutations never rebuild directionality for the document's arena.
    fn ResolveElementDirectionality(&self, element: usize) -> TextDirection {
        match self.DirectionAttribute(element) {
            Some(value) if value.eq_ignore_ascii_case("ltr") => TextDirection::kLtr,
            Some(value) if value.eq_ignore_ascii_case("rtl") => TextDirection::kRtl,
            _ if self.HasDirectionAuto(element) => self
                .ResolveAutoDirectionality(element, &self.cached_directionality)
                .unwrap_or(TextDirection::kLtr),
            _ if self.arena[element].IsHTMLElement("input")
                && self.arena[element]
                    .FindAttribute("type")
                    .is_some_and(|a| a.value.eq_ignore_ascii_case("tel")) =>
            {
                TextDirection::kLtr
            }
            _ => self
                .ParentOrShadowHostElement(element)
                .map_or(TextDirection::kLtr, |parent| {
                    self.CachedDirectionality(parent)
                }),
        }
    }

    fn UpdateDirectionalitySubtree(&mut self, root: usize) {
        let mut trace = browser_tracing::span("dom", "DOM.UpdateDirectionality");
        let mut visited = 0;
        let mut changed = 0;
        let mut pending = vec![root];
        while let Some(element) = pending.pop() {
            if self.arena[element].Type() != DOMNodeType::kElement {
                pending.extend_from_slice(self.arena[element].Children());
                continue;
            }
            visited += 1;
            let direction = self.ResolveElementDirectionality(element);
            if self.CachedDirectionality(element) == direction {
                continue;
            }
            changed += 1;
            self.cached_directionality.insert(element, direction);
            self.RecordStyleChange(crate::style_state::StyleChange::Directionality(element));
            pending.extend_from_slice(self.arena[element].Children());
            if let Some(shadow) = self.ShadowRoot(element) {
                pending.extend_from_slice(self.arena[shadow].Children());
            }
        }
        trace.set("visited_elements", visited as f64);
        trace.set("changed_elements", changed as f64);
    }

    // cpp: Element::UpdateAncestorWithDirAuto. An explicit direction, bdi,
    // script/style or text control terminates the ancestor text dependency.
    // Source: hidden_input_type.cc:109-113. Use the canonical ancestor
    // directionality owner, including the hidden control itself.
    pub fn UpdateAncestorWithDirAutoIncludingSelf(&mut self, element: usize) {
        self.UpdateAutoDirectionalityAncestors(Some(element));
    }
    fn UpdateAutoDirectionalityAncestors(&mut self, mut element: Option<usize>) {
        while let Some(current) = element {
            if self.HasDirectionAuto(current) {
                self.UpdateDirectionalitySubtree(current);
            }
            if self.DirectionAttribute(current).is_some()
                || matches!(
                    self.arena[current].Name(),
                    "bdi" | "script" | "style" | "textarea"
                )
            {
                break;
            }
            element = self.ParentOrShadowHostElement(current);
        }
    }

    // cpp: Element::UpdateDirectionalityAndDescendant's slot notification and
    // HTMLSlotElement::CalculateAndAdjustAutoDirectionality. Only shadow roots
    // on the changed node's ancestor chain can receive different assignments
    // or inherit a changed host direction; unrelated trees are never visited.
    fn UpdateDirectionalitySlots(&mut self, root: usize) {
        let mut hosts = HashSet::new();
        let mut ancestor = Some(root);
        while let Some(current) = ancestor {
            if self.ShadowRoot(current).is_some() {
                hosts.insert(current);
            }
            if let Some(host) = self.OwnerShadowHost(current) {
                hosts.insert(host);
            }
            ancestor = self.ParentOrShadowHostElement(current);
        }
        for host in hosts {
            let mut pending = vec![self.ShadowRoot(host).unwrap()];
            while let Some(current) = pending.pop() {
                if self.IsSlotSupportingAssignment(current) {
                    self.UpdateDirectionalitySubtree(current);
                    self.UpdateAutoDirectionalityAncestors(self.ParentOrShadowHostElement(current));
                }
                pending.extend_from_slice(self.arena[current].Children());
            }
        }
    }

    fn UpdateDirectionalityForMutation(&mut self, element: usize) {
        self.UpdateDirectionalitySubtree(element);
        self.UpdateAutoDirectionalityAncestors(self.ParentOrShadowHostElement(element));
        self.UpdateDirectionalitySlots(element);
    }

    pub fn CachedDirectionality(&self, element: usize) -> TextDirection {
        self.cached_directionality
            .get(&element)
            .copied()
            .unwrap_or(TextDirection::kLtr)
    }

    /// ContainerNode::NotifyNodeRemoved's inclusive descendant order. The
    /// shared transaction retains only source node identities, never node data.
    pub(crate) fn RemovalDescendants(&self, child: usize) -> Vec<usize> {
        let mut nodes = Vec::new();
        let mut pending = vec![child];
        while let Some(node) = pending.pop() {
            nodes.push(node);
            pending.extend(self.arena[node].children.iter().rev().copied());
            if let Some(shadow) = self.ShadowRoot(node) {
                pending.push(shadow);
            }
        }
        nodes
    }

    // cpp: dom/document.cc:153-162
    pub fn Remove(&mut self, child: usize) {
        let Some(parent) = self.arena[child].parent else {
            return;
        };
        let _removal = self.TrackChildRemoval(child);
        let removed_nodes = self.RemovalDescendants(child);
        self.RemoveWithDescendants(child, parent, removed_nodes);
    }
    pub(crate) fn RemoveWithDescendants(
        &mut self,
        child: usize,
        parent: usize,
        removed_nodes: Vec<usize>,
    ) {
        assert_eq!(self.arena[child].parent, Some(parent));
        let opaque_range_insertion_point_connected = self.IsConnectedToDocument(parent);
        let stateful_insertion_point_connected =
            self.IsConnectedToDocument(parent) && !self.IsInShadowTree(parent);
        self.MutationObserversNodeWillDetach(child);
        let old_scope = self.RegisteredTreeScope(child);
        self.CustomElementSubtreeWillBeRemoved(child);
        let removed_set = removed_nodes.iter().copied().collect::<HashSet<_>>();
        let removed_ids = removed_nodes
            .iter()
            .map(|&node| self.arena[node].id)
            .collect::<HashSet<_>>();
        if self
            .fullscreen_element_stack
            .last()
            .is_some_and(|current| removed_ids.contains(current))
        {
            // Fullscreen::ElementRemoved exits the document when its current
            // fullscreen element is detached.
            self.ExitFullscreen();
        } else {
            self.fullscreen_element_stack
                .retain(|candidate| !removed_ids.contains(candidate));
        }
        let mut suggested_selects = HashSet::new();
        let mut selected_selects = HashSet::new();
        let mut affected_selects = HashSet::new();
        for &node in &removed_nodes {
            if self.FormControlClass(node).is_some() || self.IsFormAssociatedCustomElement(node) {
                self.FormAssociationNeedsValidityCheck(node);
                self.SetControlNeedsValidityCheck(node);
            }
            if !self.arena[node].IsElement("option", DOMNamespace::kHTML) {
                continue;
            }
            let Some(select) = self.OptionOwnerSelect(node) else {
                continue;
            };
            if removed_set.contains(&select) {
                continue;
            }
            affected_selects.insert(select);
            if let DOMNodeClass::HTMLSelectElement(data) = &mut self.arena[select].class {
                if data.last_on_change_option == Some(node) {
                    data.last_on_change_option = None;
                }
            }
            if self.SelectSuggestedOption(select) == Some(node) {
                suggested_selects.insert(select);
            }
            if self.OptionSelected(node) {
                selected_selects.insert(select);
            }
        }
        let position = self.arena[parent]
            .children
            .iter()
            .position(|&index| index == child)
            .unwrap_or_else(|| logic_error("DOM parent/child links are inconsistent"));
        let boundary = crate::style_state::ChildrenChangeBoundary {
            before: (self.arena[child].node_type == DOMNodeType::kElement)
                .then(|| {
                    self.arena[parent].children[..position]
                        .iter()
                        .rev()
                        .copied()
                        .find(|&i| self.arena[i].node_type == DOMNodeType::kElement)
                })
                .flatten(),
            after: (self.arena[child].node_type == DOMNodeType::kElement)
                .then(|| {
                    self.arena[parent].children[position + 1..]
                        .iter()
                        .copied()
                        .find(|&i| self.arena[i].node_type == DOMNodeType::kElement)
                })
                .flatten(),
        };
        self.CaptureChildListMutation(
            parent,
            &[],
            &[child],
            position
                .checked_sub(1)
                .map(|index| self.arena[parent].children[index]),
            self.arena[parent].children.get(position + 1).copied(),
        );
        self.RangeNodeWillBeRemoved(child);
        // Document::NodeWillBeRemoved notifies EventHandler after Range has
        // adjusted its boundaries and before RemoveBetween changes ancestry.
        // This is one callback for the removed connected subtree root, not the
        // later per-descendant Element::RemovedFrom walk.
        if opaque_range_insertion_point_connected {
            let observer = self
                .node_removal_observer
                .as_ref()
                .and_then(std::rc::Weak::upgrade);
            if let Some(observer) = observer {
                observer.NodeWillBeRemoved(self, child);
            }
        }
        // ContainerNode::RemoveBetween detaches the old layout tree before
        // changing tree/slot identities. Page consumes only these value facts
        // after this exclusive DOM transaction releases its borrow.
        self.RecordRemovedLayoutTree(child);
        let previous = self.arena[child].previous_sibling;
        let next = self.arena[child].next_sibling;
        if let Some(previous) = previous {
            self.arena[previous].next_sibling = next;
        }
        if let Some(next) = next {
            self.arena[next].previous_sibling = previous;
        }
        self.arena[child].previous_sibling = None;
        self.arena[child].next_sibling = None;
        self.arena[parent].children.remove(position);
        if let Some(scope) = old_scope {
            self.UpdateSubtreeIdsInTreeScope(child, scope, false);
        }
        self.arena[child].parent = None;
        self.SetIsInCanvasSubtree(child, false);
        self.NotifyNodeReference(crate::node_references::NodeReferenceChange::Child {
            parent: self.arena[parent].id,
            child: self.arena[child].id,
            attached: false,
        });
        for select in suggested_selects {
            self.SetSelectSuggestedOption(select, None);
        }
        for &select in &selected_selects {
            self.SetAutofillState(select, DOMAutofillState::kNotFilled);
        }
        for select in affected_selects {
            self.ResetSelectToDefaultSelection(select);
        }
        self.StyleSheetConnectivityChanged(child);
        let mut detached = vec![child];
        while let Some(node) = detached.pop() {
            if opaque_range_insertion_point_connected
                && foundation::RuntimeEnabledFeatures::OpaqueRangeEnabled()
            {
                self.DisconnectAllOpaqueRanges(node);
            }
            self.RecordFontPreload(node, crate::font_preload::FontPreloadReason::Removed);
            if stateful_insertion_point_connected
                && (self.arena[node].IsHTMLElement("a") || self.arena[node].IsHTMLElement("area"))
            {
                self.attribute_lifecycle
                    .RecordAnchorRemoved(self.arena[node].id);
            }
            self.RemoveExpectRenderBlockingLink(node);
            if stateful_insertion_point_connected && self.IsTextScaleMetaElement(node) {
                self.TextScaleMetaChanged();
            }
            self.RenderBlockingElementRemoved(node);
            if self.arena[node].IsHTMLElement("body") || self.arena[node].IsHTMLElement("frameset")
            {
                self.SyncRenderBlockingBodyPresence();
            }
            if stateful_insertion_point_connected && self.arena[node].IsHTMLElement("base") {
                self.ProcessBaseElementURL();
            }
            if stateful_insertion_point_connected
                && self.ClassSupportsFormStateRestore(node)
                && !self.IsInShadowTree(node)
            {
                self.stateful_form_control_revision = self
                    .stateful_form_control_revision
                    .checked_add(1)
                    .expect("form-control invalidation revision");
            }
            if self.IsHTMLSourceElement(node) {
                let media = self
                    .Node(node)
                    .Parent()
                    .or((node == child).then_some(parent));
                if let Some(media) = media {
                    if self.IsMediaElement(media) {
                        self.AttributeLifecycle().RecordMediaSourceChanged(
                            self.Node(media).Id(),
                            self.Node(node).Id(),
                            false,
                        );
                    }
                }
            }
            if self.IsMediaElement(node) {
                self.AttributeLifecycle()
                    .RecordMediaRemoved(self.Node(node).Id());
            }
            self.VTTNodeRemoved(node);
            self.ImageMapElementRemoved(node);
            self.SelectCountedElementRemoved(node);
            self.SelectedContentRemovedFrom(node, parent);
            self.ResetMenuItemAncestorCache(node);
            self.SetReplacedImageConsumerConnected(node, false);
            // Popup/dialog source lifecycle has already completed through the
            // canonical shared removal transaction. Do not replace it with
            // flag-only mutations here (nor clear state during atomic moves).
            // Source Element::RemovedFrom, element.cc:4699-4702. Reuse
            // this owning detach walk, including implicit moves and shadow
            // descendants. Page drains the fact before reactions/reentry.
            if self.arena[node].Type() == DOMNodeType::kElement {
                if opaque_range_insertion_point_connected {
                    let observer = self
                        .node_removal_observer
                        .as_ref()
                        .and_then(std::rc::Weak::upgrade);
                    if let Some(observer) = observer {
                        observer.ElementRemoved(self, node);
                    }
                }
                self.editing_mutations
                    .push(DocumentEditingMutation::EditorElementRemoved {
                        node: self.arena[node].id,
                        had_undo_stack: self.arena[node].HasUndoStack(),
                    });
            }
            // Element::RemovedFrom, element.cc:4710-4721. Reuse this
            // existing removal walk; no second Page/document traversal.
            self.RemoveOverscrollCommandInvoker(node);
            if self.arena[node]
                .FindAttribute("id")
                .is_some_and(|id| !id.value.is_empty())
            {
                self.MarkOverscrollCommandTargetsDirty();
            }
            self.DetachOverscroll(node);
            self.RemoveAllOverscrollAreas(node);
            self.RemoveResolvedStyle(node);
            self.DisplayLockElementDisconnected(node);
            self.UserActionElementDetached(node);
            self.responsive_image_selections.remove(&node);
            self.responsive_image_auto_sizes.remove(&node);
            // Element::DetachLayoutTree descends into its ShadowRoot as well.
            // The shadow tree remains owned by the detached host, while its
            // document-computed styles and layout/resource clients are released.
            if let Some(root) = self.ShadowRoot(node) {
                detached.push(root);
            }
            detached.extend_from_slice(&self.arena[node].children);
        }
        self.UpdateDirectionalitySubtree(child);
        self.RecordNonAppendChildrenChange(parent, Some(boundary));
    }

    /// Installs Document's script-free EventHandler removal endpoint. The
    /// execution owner retains the observer; Document keeps only a weak edge.
    pub fn SetNodeRemovalObserver(
        &mut self,
        observer: Option<
            std::rc::Weak<dyn crate::node_removal_observer::DocumentNodeRemovalObserver>,
        >,
    ) {
        self.node_removal_observer = observer;
    }

    // cpp: dom/document.cc:164-175
    pub fn AppendChild(&mut self, parent: usize, child: usize) {
        if parent == child {
            invalid_argument("a DOM node cannot contain itself");
        }
        // DOM pre-insertion validity includes shadow hosts. This also keeps
        // NotifyNodeInsertedInternal's original shadow traversal acyclic.
        if self.IsShadowIncludingInclusiveAncestor(child, parent) {
            invalid_argument("DOM insertion would create a cycle");
        }
        self.Remove(child);
        let previous_last = self.arena[parent]
            .children
            .iter()
            .rev()
            .copied()
            .find(|&index| self.arena[index].node_type == DOMNodeType::kElement);
        let previous_same_type = (self.arena[child].node_type == DOMNodeType::kElement)
            .then(|| {
                self.arena[parent]
                    .children
                    .iter()
                    .rev()
                    .copied()
                    .find(|&index| {
                        self.arena[index].node_type == DOMNodeType::kElement
                            && self.arena[index].name == self.arena[child].name
                            && self.arena[index].namespace == self.arena[child].namespace
                    })
            })
            .flatten();
        self.CaptureChildListMutation(
            parent,
            &[child],
            &[],
            self.arena[parent].children.last().copied(),
            None,
        );
        let previous_node = self.arena[parent].children.last().copied();
        self.arena[parent].children.push(child);
        self.arena[child].previous_sibling = previous_node;
        self.arena[child].next_sibling = None;
        if let Some(previous) = previous_node {
            self.arena[previous].next_sibling = Some(child);
        }
        self.arena[child].parent = Some(parent);
        self.SetTreeScopeForSubtree(child, self.TreeScopeRoot(parent));
        if let Some(scope) = self.RegisteredTreeScope(child) {
            self.UpdateSubtreeIdsInTreeScope(child, scope, true);
        }
        self.NotifyNodeReference(crate::node_references::NodeReferenceChange::Child {
            parent: self.arena[parent].id,
            child: self.arena[child].id,
            attached: true,
        });
        self.UpdateDirectionalitySubtree(child);
        self.HTMLElementSubtreeInserted(child);
        self.StyleSheetConnectivityChanged(child);
        self.RecordAppendChildrenChange(parent, child, previous_last, previous_same_type);
        let owner = if self.arena[parent].node_type == DOMNodeType::kDocument {
            Some(DOMOwnerHandle {
                arena_id: self.arena_id,
                node_id: self.arena[parent].id,
            })
        } else {
            self.arena[parent].owner_document_handle
        };
        if let Some(owner) = owner {
            self.AdoptNodeByHandle(child, owner);
        }
        self.NotifySubtreeConnected(child, true);
        self.CustomElementSubtreeInserted(child);
    }

    // cpp: dom/document.cc:177-193
    pub fn InsertBefore(&mut self, parent: usize, child: usize, next_child: usize) {
        if self.arena[next_child].parent != Some(parent) {
            invalid_argument("DOM insertion point is not a child");
        }
        if child == next_child {
            return;
        }
        // DOM pre-insertion validity includes shadow hosts. This also keeps
        // NotifyNodeInsertedInternal's original shadow traversal acyclic.
        if self.IsShadowIncludingInclusiveAncestor(child, parent) {
            invalid_argument("DOM insertion would create a cycle");
        }
        self.Remove(child);
        let position = self.arena[parent]
            .children
            .iter()
            .position(|&index| index == next_child)
            .expect("DOM insertion point is not a child");
        let boundary = crate::style_state::ChildrenChangeBoundary {
            before: (self.arena[child].node_type == DOMNodeType::kElement)
                .then(|| {
                    self.arena[parent].children[..position]
                        .iter()
                        .rev()
                        .copied()
                        .find(|&i| self.arena[i].node_type == DOMNodeType::kElement)
                })
                .flatten(),
            after: (self.arena[child].node_type == DOMNodeType::kElement)
                .then(|| {
                    self.arena[parent].children[position..]
                        .iter()
                        .copied()
                        .find(|&i| self.arena[i].node_type == DOMNodeType::kElement)
                })
                .flatten(),
        };
        self.CaptureChildListMutation(
            parent,
            &[child],
            &[],
            position
                .checked_sub(1)
                .map(|index| self.arena[parent].children[index]),
            Some(next_child),
        );
        self.arena[parent].children.insert(position, child);
        let previous = self.arena[next_child].previous_sibling;
        self.arena[child].previous_sibling = previous;
        self.arena[child].next_sibling = Some(next_child);
        self.arena[next_child].previous_sibling = Some(child);
        if let Some(previous) = previous {
            self.arena[previous].next_sibling = Some(child);
        }
        self.arena[child].parent = Some(parent);
        self.SetTreeScopeForSubtree(child, self.TreeScopeRoot(parent));
        if let Some(scope) = self.RegisteredTreeScope(child) {
            self.UpdateSubtreeIdsInTreeScope(child, scope, true);
        }
        self.NotifyNodeReference(crate::node_references::NodeReferenceChange::Child {
            parent: self.arena[parent].id,
            child: self.arena[child].id,
            attached: true,
        });
        self.UpdateDirectionalitySubtree(child);
        self.HTMLElementSubtreeInserted(child);
        self.StyleSheetConnectivityChanged(child);
        self.RecordNonAppendChildrenChange(parent, Some(boundary));
        // InsertBefore attaches the same fresh/stale-free subtree as
        // AppendChild. Record it explicitly so style invalidation resolves the
        // inserted branch and treats existing siblings through selector
        // invalidation sets instead of rescanning the whole parent subtree.
        if self.ShouldRecordStyleChange(parent)
            && self.arena[child].node_type == DOMNodeType::kElement
        {
            self.style_state.inserted_style_subtrees.insert(child);
        }
        let owner = if self.arena[parent].node_type == DOMNodeType::kDocument {
            Some(DOMOwnerHandle {
                arena_id: self.arena_id,
                node_id: self.arena[parent].id,
            })
        } else {
            self.arena[parent].owner_document_handle
        };
        if let Some(owner) = owner {
            self.AdoptNodeByHandle(child, owner);
        }
        self.NotifySubtreeConnected(child, true);
        self.CustomElementSubtreeInserted(child);
    }

    // cpp: dom/document.cc:195-204
    pub fn AppendText(&mut self, parent: usize, data: &str) -> usize {
        if let Some(&last) = self.arena[parent].children.last() {
            if self.arena[last].node_type == DOMNodeType::kText
                && !self.arena[last].IsCDATASection()
            {
                let was_empty = self.arena[last].data.is_empty();
                let mut units = self.CharacterDataUTF16(last).to_vec();
                units.extend(data.encode_utf16());
                self.character_data_utf16
                    .insert(self.arena[last].id, units.into());
                self.arena[last].data.push_str(data);
                self.IncDOMTreeVersion();
                self.RecordCharacterDataChange(parent, was_empty, self.arena[last].data.is_empty());
                return last;
            }
        }
        let text = self.CreateText(data.into());
        self.AppendChild(parent, text);
        text
    }

    // cpp: dom/document.cc:206-222
    pub fn InsertTextBefore(&mut self, parent: usize, next_child: usize, data: &str) -> usize {
        if self.arena[next_child].parent != Some(parent) {
            invalid_argument("DOM insertion point is not a child");
        }
        let next_position = self.arena[parent]
            .children
            .iter()
            .position(|&index| index == next_child)
            .expect("DOM insertion point is not a child");
        if next_position > 0 {
            let previous = self.arena[parent].children[next_position - 1];
            if self.arena[previous].node_type == DOMNodeType::kText {
                let was_empty = self.arena[previous].data.is_empty();
                let mut units = self.CharacterDataUTF16(previous).to_vec();
                units.extend(data.encode_utf16());
                self.character_data_utf16
                    .insert(self.arena[previous].id, units.into());
                self.arena[previous].data.push_str(data);
                self.IncDOMTreeVersion();
                self.RecordCharacterDataChange(
                    parent,
                    was_empty,
                    self.arena[previous].data.is_empty(),
                );
                return previous;
            }
        }
        let text = self.CreateText(data.into());
        self.InsertBefore(parent, text, next_child);
        text
    }

    // cpp: dom/document.cc:224-236
    pub fn AttributeLifecycle(&self) -> &Rc<crate::attribute_lifecycle::AttributeLifecycleQueue> {
        &self.attribute_lifecycle
    }

    pub fn SetAttribute(&mut self, element: usize, attribute: DOMAttribute) {
        self.SetAttributeWithInlineStyleChange(element, attribute, false);
    }

    pub fn SetAttributeWithReason(
        &mut self,
        element: usize,
        attribute: DOMAttribute,
        reason: crate::attribute_lifecycle::AttributeModificationReason,
    ) {
        self.SetAttributeWithReasonAndInlineStyleChange(
            element, attribute, false, reason, None, None,
        );
    }
    pub(crate) fn SetAttributeWithInlineStyleChange(
        &mut self,
        element: usize,
        attribute: DOMAttribute,
        independent: bool,
    ) {
        self.SetAttributeWithReasonAndInlineStyleChange(
            element,
            attribute,
            independent,
            crate::attribute_lifecycle::AttributeModificationReason::Directly,
            None,
            None,
        );
    }
    /// Element::setAttribute's DOMString value, retaining lone surrogates.
    pub fn SetAttributeUTF16(&mut self, element: usize, attribute: DOMAttribute, value: &[u16]) {
        self.SetAttributeUTF16WithInlineStyleChange(element, attribute, Rc::from(value), false);
    }
    pub(crate) fn SetAttributeUTF16WithInlineStyleChange(
        &mut self,
        element: usize,
        mut attribute: DOMAttribute,
        value: Rc<[u16]>,
        independent: bool,
    ) {
        attribute.value = String::from_utf16_lossy(&value);
        self.SetAttributeWithReasonAndInlineStyleChange(
            element,
            attribute,
            independent,
            crate::attribute_lifecycle::AttributeModificationReason::Directly,
            Some(value),
            None,
        );
    }
    /// Element::setAttribute(QualifiedName, AtomicString), preserving both
    /// names and WTF-16 values retained by elementary edit commands.
    pub fn SetAttributeByQualifiedNameUTF16(
        &mut self,
        element: usize,
        name: &foundation::QualifiedName,
        value: Rc<[u16]>,
        independent: bool,
    ) {
        let attribute = DOMAttribute {
            prefix: name
                .Prefix()
                .utf16_units()
                .map(String::from_utf16_lossy)
                .unwrap_or_default(),
            local_name: String::from_utf16_lossy(
                name.LocalName().utf16_units().unwrap_or_default(),
            ),
            namespace_uri: name
                .NamespaceURI()
                .utf16_units()
                .map(String::from_utf16_lossy)
                .unwrap_or_default(),
            value: String::from_utf16_lossy(&value),
        };
        self.SetAttributeWithReasonAndInlineStyleChange(
            element,
            attribute,
            independent,
            crate::attribute_lifecycle::AttributeModificationReason::Directly,
            Some(value),
            Some(name),
        );
    }
    fn SetAttributeWithReasonAndInlineStyleChange(
        &mut self,
        element: usize,
        mut attribute: DOMAttribute,
        independent: bool,
        reason: crate::attribute_lifecycle::AttributeModificationReason,
        exact_value: Option<Rc<[u16]>>,
        exact_name: Option<&foundation::QualifiedName>,
    ) {
        if self.arena[element].node_type != DOMNodeType::kElement {
            invalid_argument("attributes require an element");
        }
        // HTMLMediaElement::ParseAttribute, cc:841-845. Parser initialization
        // affects legacy muted_ only; subsequent script content attributes do
        // not overwrite its IDL state.
        if attribute.namespace_uri.is_empty()
            && attribute.local_name == "muted"
            && reason == crate::attribute_lifecycle::AttributeModificationReason::ByParser
            && self.IsMediaElement(element)
        {
            self.MediaElementState(element).muted.set(true);
        }
        if attribute.namespace_uri.is_empty()
            && matches!(
                attribute.local_name.as_str(),
                "markerWidth" | "markerHeight" | "refX" | "refY"
            )
        {
            self.arena[element].svg_marker_lengths.get_mut().take();
        }
        if attribute.namespace_uri.is_empty() {
            self.explicitly_set_element_attributes
                .remove(&(element, attribute.local_name.clone()));
        }
        if attribute.namespace_uri.is_empty() && attribute.local_name == "style" {
            let retained = self.arena[element].inline_style.get_mut();
            if retained
                .as_ref()
                .is_some_and(|style| style.SourceText() != attribute.value)
            {
                retained.take();
            }
        }
        let script_blocking_attribute_changed = attribute.namespace_uri.is_empty()
            && attribute.local_name == "blocking"
            && self.arena[element].IsHTMLElement("script");
        let render_blocking_name_changed = attribute.namespace_uri.is_empty()
            && attribute.local_name == "name"
            && self.arena[element].IsHTMLElement("a");
        let expect_link_attribute = attribute.namespace_uri.is_empty()
            && matches!(
                attribute.local_name.as_str(),
                "rel" | "href" | "blocking" | "media"
            )
            && self.arena[element].IsHTMLElement("link");
        let expect_link_attribute_name = match attribute.local_name.as_str() {
            "rel" => "rel",
            "href" => "href",
            "blocking" => "blocking",
            "media" => "media",
            _ => "",
        };
        if expect_link_attribute
            && (matches!(expect_link_attribute_name, "rel" | "href")
                || (expect_link_attribute_name == "blocking"
                    && !crate::render_blocking_resource_manager::HasRenderBlockingToken(
                        &attribute.value,
                    )))
        {
            self.RemoveExpectRenderBlockingLink(element);
        }
        let existing = if let Some(name) = exact_name {
            self.arena[element]
                .attribute_utf16
                .iter()
                .position(|candidate| {
                    candidate.name.local_name.as_ref()
                        == name.LocalName().utf16_units().unwrap_or_default()
                        && candidate.name.namespace_uri.as_ref()
                            == name.NamespaceURI().utf16_units().unwrap_or_default()
                })
        } else {
            self.arena[element].attributes.iter().position(|candidate| {
                candidate.local_name == attribute.local_name
                    && candidate.namespace_uri == attribute.namespace_uri
            })
        };
        // Element::SetAttributeInternal, element.cc:13777-13801 updates the
        // value of an existing attribute without replacing its qualified name.
        if let Some(index) = existing.filter(|_| exact_name.is_some()) {
            attribute.prefix = self.arena[element].attributes[index].prefix.clone();
        }
        // Element::WillModifyAttribute queues a callback even for equal
        // values. The style-only no-change return below must not bypass it.
        if self.HasCustomElementDefinitions()
            && self.CustomElementDefinitionForElement(element).is_some()
        {
            let old_value =
                existing.map(|index| self.arena[element].attributes[index].value.clone());
            self.CustomElementAttributeWillChange(
                element,
                &attribute.local_name,
                &attribute.namespace_uri,
                old_value,
                Some(attribute.value.clone()),
            );
        }
        self.CaptureAttributeMutation(element, &attribute.local_name, &attribute.namespace_uri);
        let option_selectedness_changed = existing.is_none()
            && attribute.local_name == "selected"
            && attribute.namespace_uri.is_empty()
            && self.arena[element].IsElement("option", DOMNamespace::kHTML);
        let option_selectedness_before =
            option_selectedness_changed.then(|| self.OptionSelected(element));
        let select_multiple_changed = existing.is_none()
            && attribute.local_name == "multiple"
            && attribute.namespace_uri.is_empty()
            && self.arena[element].IsElement("select", DOMNamespace::kHTML);
        let select_size_changed = attribute.local_name == "size"
            && attribute.namespace_uri.is_empty()
            && self.arena[element].IsElement("select", DOMNamespace::kHTML);
        let old_selected_option = (select_multiple_changed || select_size_changed)
            .then(|| self.FirstSelectedOption(element))
            .flatten();
        let exportparts_changed =
            attribute.local_name == "exportparts" && attribute.namespace_uri.is_empty();
        let disabled_state_changed =
            attribute.local_name == "disabled" && attribute.namespace_uri.is_empty();
        let datalist_id_changed =
            attribute.local_name == "id" && attribute.namespace_uri.is_empty();
        let default_button_may_change = attribute.namespace_uri.is_empty()
            && matches!(attribute.local_name.as_str(), "type" | "form" | "id");
        let radio_group_may_change = attribute.namespace_uri.is_empty()
            && matches!(
                attribute.local_name.as_str(),
                "checked" | "form" | "id" | "name" | "type" | "required"
            );
        let directionality_changed = attribute.namespace_uri.is_empty()
            && (matches!(
                attribute.local_name.as_str(),
                "dir" | "type" | "value" | "slot"
            ) || (attribute.local_name == "name" && self.arena[element].IsHTMLElement("slot")));
        let meter_state_changed = attribute.namespace_uri.is_empty()
            && self.arena[element].IsHTMLElement("meter")
            && matches!(
                attribute.local_name.as_str(),
                "value" | "min" | "max" | "low" | "high" | "optimum"
            );
        let progress_state_changed = attribute.namespace_uri.is_empty()
            && self.arena[element].IsHTMLElement("progress")
            && matches!(attribute.local_name.as_str(), "value" | "max");
        if attribute.namespace_uri.is_empty()
            && attribute.local_name == "hidden"
            && self.arena[element].namespace == DOMNamespace::kHTML
        {
            self.SetDisplayLockHiddenUntilFound(element, attribute.value == "until-found");
        }
        self.IncDOMTreeVersion();
        let mut utf16_attribute = DOMAttributeUTF16::FromAttribute(&attribute);
        if let Some(name) = exact_name {
            utf16_attribute.name = existing
                .map(|index| self.arena[element].attribute_utf16[index].name.clone())
                .unwrap_or_else(|| DOMQualifiedNameUTF16 {
                    prefix: Rc::from(name.Prefix().utf16_units().unwrap_or_default()),
                    local_name: Rc::from(name.LocalName().utf16_units().unwrap_or_default()),
                    namespace_uri: Rc::from(name.NamespaceURI().utf16_units().unwrap_or_default()),
                });
        }
        if let Some(value) = exact_value {
            utf16_attribute.value = value;
        }
        let changed = !existing.is_some_and(|i| {
            self.arena[element].attributes[i] == attribute
                && self.arena[element].attribute_utf16[i] == utf16_attribute
        });
        if std::env::var_os("BROWSER_PROFILE_STYLE_INVALIDATION").is_some() {
            eprintln!(
                "style-attribute-mutation-profile node_id={} name={:?} namespace={:?} changed={} inline_style={}",
                self.arena[element].id,
                attribute.local_name,
                attribute.namespace_uri,
                changed,
                attribute.local_name == "style" && attribute.namespace_uri.is_empty()
            );
        }
        if !changed {
            if script_blocking_attribute_changed {
                self.ScriptRenderBlockingAttributeChanged(element);
            }
            if render_blocking_name_changed {
                self.ProcessAnchorNameRenderBlocking(element);
            }
            if expect_link_attribute {
                self.ExpectLinkAttributeChanged(element, expect_link_attribute_name);
            }
            if attribute.namespace_uri.is_empty() && attribute.local_name == "id" {
                self.ProcessElementRenderBlockingIdAttribute(element);
            }
            self.FontPreloadAttributeChanged(
                element,
                &attribute.local_name,
                &attribute.namespace_uri,
            );
            self.HTMLSourceAttributeChanged(
                element,
                &attribute.local_name,
                &attribute.namespace_uri,
            );
            self.CanvasDimensionAttributeChanged(
                element,
                &attribute.local_name,
                &attribute.namespace_uri,
            );
            self.ImageMapAttributeChanged(
                element,
                &attribute.local_name,
                &attribute.namespace_uri,
                Some(&attribute.value),
            );
            return;
        }
        let id_changed = attribute.namespace_uri.is_empty() && attribute.local_name == "id";
        let old_tree_scope_id = id_changed
            .then(|| existing.map(|index| self.arena[element].attributes[index].value.clone()))
            .flatten();
        let new_tree_scope_id = id_changed.then(|| attribute.value.clone());
        let image_map_paint_owner_may_change = attribute.namespace_uri.is_empty()
            && ((self.arena[element].IsElement("area", DOMNamespace::kHTML)
                && matches!(attribute.local_name.as_str(), "shape" | "coords"))
                || (self.arena[element].IsElement("map", DOMNamespace::kHTML)
                    && matches!(attribute.local_name.as_str(), "name" | "id"))
                || (self.arena[element].IsElement("img", DOMNamespace::kHTML)
                    && attribute.local_name == "usemap"));
        let image_map_paint_owners_before = image_map_paint_owner_may_change
            .then(|| self.ImageMapPaintOwnersForElementMutation(element))
            .unwrap_or_default();
        let old_replaced_image_source = (attribute.namespace_uri.is_empty()
            && attribute.local_name == "src"
            && self.arena[element].IsHTMLElement("img"))
        .then(|| self.CurrentImageSource(element).map(str::to_owned))
        .flatten();
        self.RecordAttributeSelectorKeys(
            element,
            &attribute.local_name,
            &attribute.namespace_uri,
            Some(&attribute.value),
        );
        // cpp: HTMLStyleElement::ParseAttribute. Media changes refresh the
        // owner's sheet conditions; type changes reprocess its contents. Feed
        // both through the existing stylesheet synchronization dirty queue.
        if attribute.namespace_uri.is_empty()
            && matches!(attribute.local_name.as_str(), "media" | "type")
            && self.arena[element].IsStyleElement()
        {
            self.MarkStyleElementTextDirty(element);
        }
        let sibling_sensitive_before = self.NodeMayAffectSiblingSelectors(element);
        let descendant_sensitive_before = self.NodeMayAffectDescendantSelectors(element);
        if self.AttributeMayChangeFlatTree(element, &attribute.local_name, &attribute.namespace_uri)
        {
            self.RecordFlatTreeChange(element);
        }
        self.BumpCollectionMembershipRevision();
        self.InvalidateSelectorAttributes(element, &attribute.local_name, existing.is_none());
        self.InvalidateSVGAttribute(element, &attribute.local_name);
        let affects_metadata = !matches!(attribute.local_name.as_str(), "style" | "class" | "id");
        let selector_only = if attribute.local_name == "style" && attribute.namespace_uri.is_empty()
        {
            let inline = crate::style_state::StyleChange::InlineStyle(element);
            let independent = independent
                && (!self.style_state.changes.contains(&inline)
                    || self
                        .style_state
                        .independent_inline_styles
                        .contains(&element));
            self.RecordStyleChange(inline);
            if independent && self.ShouldRecordStyleChange(element) {
                self.style_state.independent_inline_styles.insert(element);
            } else {
                self.style_state.independent_inline_styles.remove(&element);
            }
            if sibling_sensitive_before && self.ShouldRecordStyleChange(element) {
                self.style_state.sibling_sensitive_nodes.insert(element);
            }
            false
        } else {
            self.RecordAttributeStyleChange(
                element,
                &attribute.local_name,
                &attribute.namespace_uri,
                sibling_sensitive_before,
            )
        };
        // These admitted attributes have no layout metadata consumer. Any
        // selector/attr() presentation change is merged by the style resolver.
        if affects_metadata && !selector_only && self.IsConnectedToDocument(element) {
            if self.MayHaveRenderedSubtree(element) {
                self.style_state
                    .geometry_input_nodes
                    .insert(self.arena[element].id);
            }
            self.style_state
                .layout_tree_roots
                .insert(self.arena[element].id);
            if self.AffectsSVGResourceClients(element) {
                self.style_state.layout_inputs_require_tree = true;
            }
            self.style_state
                .impact
                .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        }
        let lifecycle_change = crate::attribute_lifecycle::AttributeChange {
            node: self.arena[element].Id(),
            name: attribute.local_name.clone(),
            namespace: attribute.namespace_uri.clone(),
            old_value: existing.map(|index| self.arena[element].attributes[index].value.clone()),
            new_value: Some(attribute.value.clone()),
            old_value_utf16: existing
                .map(|index| self.arena[element].attribute_utf16[index].value.clone()),
            new_value_utf16: Some(utf16_attribute.value.clone()),
            reason,
            was_connected: self.IsConnectedToDocument(element),
        };
        if attribute.namespace_uri.is_empty()
            && matches!(
                attribute.local_name.as_str(),
                "color" | "face" | "size" | "background"
            )
        {
            self.arena[element]
                .serialization_attribute_values
                .get_mut()
                .take();
        }
        if let Some(index) = existing {
            self.arena[element].attributes[index] = attribute;
            self.arena[element].attribute_utf16[index] = utf16_attribute;
        } else {
            self.arena[element].attributes.push(attribute);
            self.arena[element].attribute_utf16.push(utf16_attribute);
        }
        let lifecycle_change_is_input_type = lifecycle_change.namespace.is_empty()
            && lifecycle_change.name == "type"
            && self.arena[element].IsHTMLElement("input");
        if lifecycle_change_is_input_type
            && DOMInputType::FromTypeAttribute(lifecycle_change.old_value.as_deref())
                != self.InputType(element).expect("input class")
        {
            self.input_file_lists
                .borrow_mut()
                .remove(&self.arena[element].id);
        }

        if lifecycle_change.namespace.is_empty() && lifecycle_change.name == "value" {
            self.input_value_attribute_utf16
                .remove(&self.arena[element].id);
            if self.arena[element].IsHTMLElement("input") && !self.HasDirtyControlValue(element) {
                self.control_value_state.remove(&self.arena[element].id);
            }
        }
        if lifecycle_change.namespace.is_empty() && lifecycle_change.name == "focusgroup" {
            self.UpdateFocusgroup(element);
        }
        self.ControlValidationAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        self.InputValueAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );

        self.CanvasSlotAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        self.BaseElementURLAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        self.IFrameDocumentPolicyAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        self.FrameElementLoadingAttributeChanged(element, &lifecycle_change);
        self.FontPreloadAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        self.HTMLSourceAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        self.EditingMetaAttributeChanged(element, &lifecycle_change);
        self.CanvasDimensionAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        self.OverscrollAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        let select_text_changed = lifecycle_change.namespace.is_empty()
            && self.ParseSelectAttribute(element, &lifecycle_change.name);
        self.ImageMapAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
            lifecycle_change.old_value.as_deref(),
        );
        self.AnchorRelAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        if lifecycle_change.namespace.is_empty() && lifecycle_change.name == "label" {
            self.OptionDidChangeTextContent(element);
        }
        if lifecycle_change.namespace.is_empty() && lifecycle_change.name == "hidden" {
            self.RecordFragmentAnchorContentChanged();
        }
        self.attribute_lifecycle.Record(lifecycle_change);
        if id_changed {
            self.UpdateElementIdInTreeScope(
                element,
                old_tree_scope_id.as_deref(),
                new_tree_scope_id.as_deref(),
            );
        }
        if image_map_paint_owner_may_change {
            let image_map_paint_owners_after = self.ImageMapPaintOwnersForElementMutation(element);
            self.RecordImageMapPaintOwnerChanges(
                image_map_paint_owners_before
                    .into_iter()
                    .chain(image_map_paint_owners_after),
            );
        }
        self.SyncReplacedImageConsumer(element, old_replaced_image_source.as_deref());
        if self.arena[element].IsElement("option", DOMNamespace::kHTML) {
            self.RecordSelectContentGeometryChange(element);
            self.RefreshSelectSuggestedValues();
        }
        if disabled_state_changed
            && matches!(
                self.arena[element].Name(),
                "fieldset" | "select" | "optgroup"
            )
            && self.arena[element].Namespace() == DOMNamespace::kHTML
        {
            self.InvalidateFormDisabledDescendants(element);
        }
        if datalist_id_changed {
            self.InvalidateDatalistInputs();
        }
        if default_button_may_change {
            self.InvalidateDefaultPseudoCandidates();
        }
        if radio_group_may_change
            && !(lifecycle_change_is_input_type
                && self.attribute_lifecycle.HasOwner()
                && self
                    .input_type_lifecycle_owners
                    .contains(&self.arena[element].id))
        {
            if datalist_id_changed {
                self.InvalidateRadioInputs();
            } else {
                self.SynchronizeRadioInput(element);
                self.InvalidateRadioGroup(element);
            }
        }
        if exportparts_changed {
            self.SyncPartNamesMap(element, false);
            self.RecordStyleChange(crate::style_state::StyleChange::ExportParts(element));
        }
        if (self.arena[element].IsElement("input", DOMNamespace::kHTML)
            || self.arena[element].IsElement("textarea", DOMNamespace::kHTML))
            && !(lifecycle_change_is_input_type && self.attribute_lifecycle.HasOwner())
        {
            self.EnsureTextControlUAShadowTree(element);
            self.EnsureColorUAShadowTree(element);
            self.EnsureFileUAShadowTree(element);
        }
        if meter_state_changed {
            self.SyncMeterValueAppearance(element);
        }
        if progress_state_changed {
            self.SyncProgressValueAppearance(element);
        }
        if directionality_changed
            && !(lifecycle_change_is_input_type && self.attribute_lifecycle.HasOwner())
        {
            self.UpdateDirectionalityForMutation(element);
        }
        if option_selectedness_changed {
            self.OptionSelectedAttributeChanged(
                element,
                option_selectedness_before.unwrap_or(false),
                true,
            );
        }
        if select_multiple_changed {
            self.SelectMultipleAttributeChanged(element, false, old_selected_option);
        } else if select_size_changed
            && select_text_changed
            && !self.arena[element].children.is_empty()
        {
            self.ResetSelectToDefaultSelection(element);
        }
        if select_text_changed {
            self.attribute_lifecycle
                .RecordSelectTextStyleAndContentChanged(self.arena[element].id);
        }
        if expect_link_attribute {
            self.ExpectLinkAttributeChanged(element, expect_link_attribute_name);
        }
        if id_changed {
            self.ProcessElementRenderBlockingIdAttribute(element);
        }
        if render_blocking_name_changed {
            self.ProcessAnchorNameRenderBlocking(element);
        }
        if script_blocking_attribute_changed {
            self.ScriptRenderBlockingAttributeChanged(element);
        }
        // Keep both sides of the mutation: removing a matching class/id is as
        // relevant as adding one, and the old state is no longer observable.
        self.RecordSiblingSensitivity(element, false);
        self.RecordDescendantSensitivity(element, descendant_sensitive_before);
        self.SynchronizeAllElementInternals();
    }

    // cpp: dom/document.cc:238-247
    pub fn RemoveAttribute(&mut self, element: usize, local_name: &str, namespace_uri: &str) {
        self.RemoveAttributeWithReason(
            element,
            local_name,
            namespace_uri,
            crate::attribute_lifecycle::AttributeModificationReason::Directly,
        );
    }
    pub fn RemoveAttributeWithReason(
        &mut self,
        element: usize,
        local_name: &str,
        namespace_uri: &str,
        reason: crate::attribute_lifecycle::AttributeModificationReason,
    ) {
        self.RemoveAttributeWithQualifiedNameAndReason(
            element,
            local_name,
            namespace_uri,
            reason,
            None,
        );
    }
    /// Element::RemoveAllAttributes, element.cc:1317-1325. Keep the source
    /// last-attribute order and cloning reason in the canonical DOM owner.
    pub fn RemoveAllAttributesForCloning(&mut self, element: usize) {
        while let Some(attribute) = self.arena[element].attribute_utf16.last() {
            let name = foundation::QualifiedName::new(
                foundation::AtomicString::from_utf16(&attribute.name.prefix),
                foundation::AtomicString::from_utf16(&attribute.name.local_name),
                foundation::AtomicString::from_utf16(&attribute.name.namespace_uri),
            );
            let local = String::from_utf16_lossy(&attribute.name.local_name);
            let namespace = String::from_utf16_lossy(&attribute.name.namespace_uri);
            self.RemoveAttributeWithQualifiedNameAndReason(
                element,
                &local,
                &namespace,
                crate::attribute_lifecycle::AttributeModificationReason::ByCloning,
                Some(&name),
            );
        }
    }
    pub fn RemoveAttributeByQualifiedName(
        &mut self,
        element: usize,
        name: &foundation::QualifiedName,
    ) {
        let local_name =
            String::from_utf16_lossy(name.LocalName().utf16_units().unwrap_or_default());
        let namespace =
            String::from_utf16_lossy(name.NamespaceURI().utf16_units().unwrap_or_default());
        self.RemoveAttributeWithQualifiedNameAndReason(
            element,
            &local_name,
            &namespace,
            crate::attribute_lifecycle::AttributeModificationReason::Directly,
            Some(name),
        );
    }
    fn RemoveAttributeWithQualifiedNameAndReason(
        &mut self,
        element: usize,
        local_name: &str,
        namespace_uri: &str,
        reason: crate::attribute_lifecycle::AttributeModificationReason,
        exact_name: Option<&foundation::QualifiedName>,
    ) {
        if self.arena[element].node_type != DOMNodeType::kElement {
            invalid_argument("attributes require an element");
        }
        if namespace_uri.is_empty() {
            self.explicitly_set_element_attributes
                .remove(&(element, local_name.to_owned()));
        }
        let index = if let Some(name) = exact_name {
            self.arena[element]
                .attribute_utf16
                .iter()
                .position(|attribute| {
                    attribute.name.local_name.as_ref()
                        == name.LocalName().utf16_units().unwrap_or_default()
                        && attribute.name.namespace_uri.as_ref()
                            == name.NamespaceURI().utf16_units().unwrap_or_default()
                })
        } else {
            self.arena[element].attributes.iter().position(|attribute| {
                attribute.local_name == local_name && attribute.namespace_uri == namespace_uri
            })
        };
        let Some(attribute_index) = index else {
            return;
        };
        let expect_link_attribute = namespace_uri.is_empty()
            && matches!(local_name, "rel" | "href" | "blocking" | "media")
            && self.arena[element].IsHTMLElement("link");
        if expect_link_attribute && local_name != "media" {
            self.RemoveExpectRenderBlockingLink(element);
        }
        self.IncDOMTreeVersion();
        if self.HasCustomElementDefinitions()
            && self.CustomElementDefinitionForElement(element).is_some()
        {
            let old_value = Some(
                self.arena[element].attributes[attribute_index]
                    .value
                    .clone(),
            );
            self.CustomElementAttributeWillChange(
                element,
                local_name,
                namespace_uri,
                old_value,
                None,
            );
        }
        self.CaptureAttributeMutation(element, local_name, namespace_uri);
        if namespace_uri.is_empty()
            && matches!(local_name, "markerWidth" | "markerHeight" | "refX" | "refY")
        {
            self.arena[element].svg_marker_lengths.get_mut().take();
        }
        if namespace_uri.is_empty() && local_name == "style" {
            self.arena[element].inline_style.get_mut().take();
        }
        let removed_tree_scope_id = (namespace_uri.is_empty() && local_name == "id")
            .then(|| {
                self.arena[element]
                    .FindAttribute("id")
                    .map(|attribute| attribute.value.clone())
            })
            .flatten();
        let image_map_paint_owner_may_change = namespace_uri.is_empty()
            && ((self.arena[element].IsElement("area", DOMNamespace::kHTML)
                && matches!(local_name, "shape" | "coords"))
                || (self.arena[element].IsElement("map", DOMNamespace::kHTML)
                    && matches!(local_name, "name" | "id"))
                || (self.arena[element].IsElement("img", DOMNamespace::kHTML)
                    && local_name == "usemap"));
        let image_map_paint_owners_before = image_map_paint_owner_may_change
            .then(|| self.ImageMapPaintOwnersForElementMutation(element))
            .unwrap_or_default();
        let old_replaced_image_source = (namespace_uri.is_empty()
            && local_name == "src"
            && self.arena[element].IsHTMLElement("img"))
        .then(|| self.CurrentImageSource(element).map(str::to_owned))
        .flatten();
        self.RecordAttributeSelectorKeys(element, local_name, namespace_uri, None);
        let sibling_sensitive_before = self.NodeMayAffectSiblingSelectors(element);
        let descendant_sensitive_before = self.NodeMayAffectDescendantSelectors(element);
        if self.AttributeMayChangeFlatTree(element, local_name, namespace_uri) {
            self.RecordFlatTreeChange(element);
        }
        let option_selectedness_before = (local_name == "selected"
            && namespace_uri.is_empty()
            && self.arena[element].IsElement("option", DOMNamespace::kHTML))
        .then(|| self.OptionSelected(element));
        let select_multiple_changed = local_name == "multiple"
            && namespace_uri.is_empty()
            && self.arena[element].IsElement("select", DOMNamespace::kHTML);
        let select_size_changed = local_name == "size"
            && namespace_uri.is_empty()
            && self.arena[element].IsElement("select", DOMNamespace::kHTML);
        let old_selected_option = (select_multiple_changed || select_size_changed)
            .then(|| self.FirstSelectedOption(element))
            .flatten();
        let default_button_may_change =
            namespace_uri.is_empty() && matches!(local_name, "type" | "form" | "id");
        self.BumpCollectionMembershipRevision();
        self.InvalidateSelectorAttributes(element, local_name, true);
        self.InvalidateSVGAttribute(element, local_name);
        if std::env::var_os("BROWSER_PROFILE_STYLE_INVALIDATION").is_some() {
            eprintln!(
                "style-attribute-mutation-profile node_id={} name={:?} namespace={:?} changed=true inline_style={} removed=true",
                self.arena[element].id,
                local_name,
                namespace_uri,
                local_name == "style" && namespace_uri.is_empty()
            );
        }
        let selector_only = if local_name == "style" && namespace_uri.is_empty() {
            self.RecordStyleChange(crate::style_state::StyleChange::InlineStyle(element));
            self.style_state.independent_inline_styles.remove(&element);
            if sibling_sensitive_before && self.ShouldRecordStyleChange(element) {
                self.style_state.sibling_sensitive_nodes.insert(element);
            }
            false
        } else {
            self.RecordAttributeStyleChange(
                element,
                local_name,
                namespace_uri,
                sibling_sensitive_before,
            )
        };
        if !selector_only
            && !matches!(local_name, "style" | "class" | "id")
            && self.IsConnectedToDocument(element)
        {
            if self.MayHaveRenderedSubtree(element) {
                self.style_state
                    .geometry_input_nodes
                    .insert(self.arena[element].id);
            }
            self.style_state
                .layout_tree_roots
                .insert(self.arena[element].id);
            if self.AffectsSVGResourceClients(element) {
                self.style_state.layout_inputs_require_tree = true;
            }
            self.style_state
                .impact
                .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        }
        let lifecycle_change = crate::attribute_lifecycle::AttributeChange {
            node: self.arena[element].Id(),
            name: local_name.to_owned(),
            namespace: namespace_uri.to_owned(),
            old_value: Some(
                self.arena[element].attributes[attribute_index]
                    .value
                    .clone(),
            ),
            new_value: None,
            old_value_utf16: Some(
                self.arena[element].attribute_utf16[attribute_index]
                    .value
                    .clone(),
            ),
            new_value_utf16: None,
            reason,
            was_connected: self.IsConnectedToDocument(element),
        };
        self.arena[element].attributes.remove(attribute_index);
        self.arena[element].attribute_utf16.remove(attribute_index);
        if namespace_uri.is_empty()
            && local_name == "hidden"
            && self.arena[element].namespace == DOMNamespace::kHTML
        {
            self.SetDisplayLockHiddenUntilFound(element, false);
        }
        let lifecycle_change_is_input_type = lifecycle_change.namespace.is_empty()
            && lifecycle_change.name == "type"
            && self.arena[element].IsHTMLElement("input");
        if lifecycle_change_is_input_type
            && DOMInputType::FromTypeAttribute(lifecycle_change.old_value.as_deref())
                != self.InputType(element).expect("input class")
        {
            self.input_file_lists
                .borrow_mut()
                .remove(&self.arena[element].id);
        }

        if lifecycle_change.namespace.is_empty() && lifecycle_change.name == "value" {
            self.input_value_attribute_utf16
                .remove(&self.arena[element].id);
            if self.arena[element].IsHTMLElement("input") && !self.HasDirtyControlValue(element) {
                self.control_value_state.remove(&self.arena[element].id);
            }
        }
        if lifecycle_change.namespace.is_empty() && lifecycle_change.name == "focusgroup" {
            self.UpdateFocusgroup(element);
        }
        self.ControlValidationAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        self.InputValueAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );

        self.CanvasSlotAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        self.BaseElementURLAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        self.IFrameDocumentPolicyAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        self.FontPreloadAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        self.FrameElementLoadingAttributeChanged(element, &lifecycle_change);
        self.HTMLSourceAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        self.EditingMetaAttributeChanged(element, &lifecycle_change);
        self.CanvasDimensionAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        self.OverscrollAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        let select_text_changed = lifecycle_change.namespace.is_empty()
            && self.ParseSelectAttribute(element, &lifecycle_change.name);
        self.ImageMapAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
            lifecycle_change.old_value.as_deref(),
        );
        self.AnchorRelAttributeChanged(
            element,
            &lifecycle_change.name,
            &lifecycle_change.namespace,
        );
        if lifecycle_change.namespace.is_empty() && lifecycle_change.name == "label" {
            self.OptionDidChangeTextContent(element);
        }
        if lifecycle_change.namespace.is_empty() && lifecycle_change.name == "hidden" {
            self.RecordFragmentAnchorContentChanged();
        }
        self.attribute_lifecycle.Record(lifecycle_change);
        if let Some(id) = removed_tree_scope_id.as_deref() {
            self.UpdateElementIdInTreeScope(element, Some(id), None);
        }
        if image_map_paint_owner_may_change {
            let image_map_paint_owners_after = self.ImageMapPaintOwnersForElementMutation(element);
            self.RecordImageMapPaintOwnerChanges(
                image_map_paint_owners_before
                    .into_iter()
                    .chain(image_map_paint_owners_after),
            );
        }
        self.SyncReplacedImageConsumer(element, old_replaced_image_source.as_deref());
        // Removal restores the same owner defaults as HTMLStyleElement's
        // ParseAttribute with a null new value.
        if namespace_uri.is_empty()
            && matches!(local_name, "media" | "type")
            && self.arena[element].IsStyleElement()
        {
            self.MarkStyleElementTextDirty(element);
        }
        if default_button_may_change {
            self.InvalidateDefaultPseudoCandidates();
        }
        if self.arena[element].IsElement("option", DOMNamespace::kHTML) {
            self.RecordSelectContentGeometryChange(element);
            self.RefreshSelectSuggestedValues();
        }
        if local_name == "selected"
            && namespace_uri.is_empty()
            && self.arena[element].IsElement("option", DOMNamespace::kHTML)
        {
            self.OptionSelectedAttributeChanged(
                element,
                option_selectedness_before.unwrap_or(false),
                false,
            );
        }
        if select_multiple_changed {
            self.SelectMultipleAttributeChanged(element, true, old_selected_option);
        } else if select_size_changed
            && select_text_changed
            && !self.arena[element].children.is_empty()
        {
            self.ResetSelectToDefaultSelection(element);
        }
        if select_text_changed {
            self.attribute_lifecycle
                .RecordSelectTextStyleAndContentChanged(self.arena[element].id);
        }
        // Popover type changes and dialog open removal run their translated
        // lifecycle after releasing this borrow through AttributeLifecycle.
        // Do not clear visibility/modal/top-layer state before beforetoggle.
        if local_name == "disabled"
            && namespace_uri.is_empty()
            && matches!(
                self.arena[element].Name(),
                "fieldset" | "select" | "optgroup"
            )
            && self.arena[element].Namespace() == DOMNamespace::kHTML
        {
            self.InvalidateFormDisabledDescendants(element);
        }
        if local_name == "id" && namespace_uri.is_empty() {
            self.InvalidateDatalistInputs();
        }
        if namespace_uri.is_empty()
            && matches!(
                local_name,
                "checked" | "form" | "id" | "name" | "type" | "required"
            )
            && !(lifecycle_change_is_input_type
                && self.attribute_lifecycle.HasOwner()
                && self
                    .input_type_lifecycle_owners
                    .contains(&self.arena[element].id))
        {
            if local_name == "id" {
                self.InvalidateRadioInputs();
            } else {
                self.SynchronizeRadioInput(element);
                self.InvalidateRadioGroup(element);
            }
        }
        if local_name == "exportparts" && namespace_uri.is_empty() {
            self.SyncPartNamesMap(element, true);
            self.RecordStyleChange(crate::style_state::StyleChange::ExportParts(element));
        }
        if (self.arena[element].IsElement("input", DOMNamespace::kHTML)
            || self.arena[element].IsElement("textarea", DOMNamespace::kHTML))
            && !(lifecycle_change_is_input_type && self.attribute_lifecycle.HasOwner())
        {
            self.EnsureTextControlUAShadowTree(element);
            self.EnsureColorUAShadowTree(element);
            self.EnsureFileUAShadowTree(element);
        }
        if namespace_uri.is_empty()
            && self.arena[element].IsHTMLElement("meter")
            && matches!(
                local_name,
                "value" | "min" | "max" | "low" | "high" | "optimum"
            )
        {
            self.SyncMeterValueAppearance(element);
        }
        if namespace_uri.is_empty()
            && self.arena[element].IsHTMLElement("progress")
            && matches!(local_name, "value" | "max")
        {
            self.SyncProgressValueAppearance(element);
        }
        if namespace_uri.is_empty()
            && (matches!(local_name, "dir" | "type" | "value" | "slot")
                || (local_name == "name" && self.arena[element].IsHTMLElement("slot")))
        {
            self.UpdateDirectionalityForMutation(element);
        }
        self.RecordSiblingSensitivity(element, false);
        self.RecordDescendantSensitivity(element, descendant_sensitive_before);
        if expect_link_attribute {
            self.ExpectLinkAttributeChanged(element, local_name);
        }
        if namespace_uri.is_empty() && local_name == "blocking" {
            self.ScriptRenderBlockingAttributeChanged(element);
        }
        self.SynchronizeAllElementInternals();
    }

    // cpp: dom/document.h:115-117
    pub fn RemoveAttributeDefault(&mut self, element: usize, local_name: &str) {
        self.RemoveAttribute(element, local_name, "")
    }

    // cpp: core/dom/character_data.cc CharacterData::replaceData.
    pub fn ReplaceData(
        &mut self,
        node: usize,
        offset: usize,
        count: usize,
        data: &str,
    ) -> Result<(), crate::range::DOMRangeError> {
        self.ReplaceDataUTF16(
            node,
            offset,
            count,
            &data.encode_utf16().collect::<Vec<_>>(),
        )
    }
    pub fn ReplaceDataUTF16(
        &mut self,
        node: usize,
        offset: usize,
        count: usize,
        data: &[u16],
    ) -> Result<(), crate::range::DOMRangeError> {
        use crate::range::DOMRangeError;
        if !matches!(
            self.arena[node].node_type,
            DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
        ) {
            return Err(DOMRangeError::InvalidNodeType);
        }
        let old = self.CharacterDataUTF16(node);
        if offset > old.len() {
            return Err(DOMRangeError::InvalidOffset);
        }
        let count = count.min(old.len() - offset);
        let inserted = data;
        let mut replacement = Vec::with_capacity(old.len() - count + inserted.len());
        replacement.extend_from_slice(&old[..offset]);
        replacement.extend_from_slice(&inserted);
        replacement.extend_from_slice(&old[offset + count..]);
        // Keep WTF-16 on the canonical node; only its rendering projection
        // replaces isolated surrogates with the Unicode replacement glyph.
        let was_empty = self.arena[node].data.is_empty();
        self.RecordEditingNodeFactsChange(node);
        self.IncDOMTreeVersion();
        self.CaptureCharacterDataMutation(node);
        self.RangeCharacterDataReplaced(node, offset, count, inserted.len());
        self.arena[node].data = String::from_utf16_lossy(&replacement);
        self.character_data_utf16
            .insert(self.arena[node].id, replacement.into());
        if let Some(parent) = self.arena[node].parent {
            self.RecordCharacterDataChange(parent, was_empty, self.arena[node].data.is_empty());
        }
        self.RecordEditingCharacterDataReplacement(node, offset, count, inserted.len());
        Ok(())
    }

    // cpp: core/dom/text.cc Text::splitText.
    pub fn SplitText(
        &mut self,
        node: usize,
        offset: usize,
    ) -> Result<usize, crate::range::DOMRangeError> {
        use crate::range::DOMRangeError;
        if self.arena[node].node_type != DOMNodeType::kText {
            return Err(DOMRangeError::InvalidNodeType);
        }
        let old = self.CharacterDataUTF16(node);
        if offset > old.len() {
            return Err(DOMRangeError::InvalidOffset);
        }
        let next = self.CreateTextUTF16(&old[offset..]);
        self.RecordEditingNodeFactsChange(node);
        self.IncDOMTreeVersion();
        self.CaptureCharacterDataMutation(node);
        let was_empty = self.arena[node].data.is_empty();
        self.arena[node].data = String::from_utf16_lossy(&old[..offset]);
        self.character_data_utf16
            .insert(self.arena[node].id, Rc::from(&old[..offset]));
        if let Some(parent) = self.arena[node].parent {
            self.RecordCharacterDataChange(parent, was_empty, self.arena[node].data.is_empty());
            let position = self.arena[parent]
                .children
                .iter()
                .position(|&child| child == node)
                .unwrap();
            if let Some(&before) = self.arena[parent].children.get(position + 1) {
                self.InsertBefore(parent, next, before);
            } else {
                self.AppendChild(parent, next);
            }
            self.RangeTextNodeSplit(node, next, offset);
        } else {
            self.RangeCharacterDataReplaced(node, offset, old.len() - offset, 0);
        }
        Ok(next)
    }

    // cpp: dom/document.cc:249-265
    // Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
    // Source: third_party/blink/renderer/core/dom/node.cc:2388-2404
    /// Non-null container textContent, preserving the canonical WTF-16
    /// CharacterData owners and live child order without a projected DOM tree.
    pub fn TextContentUTF16(&self, node: usize) -> Rc<[u16]> {
        debug_assert!(matches!(
            self.Node(node).Type(),
            DOMNodeType::kElement | DOMNodeType::kDocumentFragment
        ));
        fn append(document: &PersistentDocument, index: usize, text: &mut Vec<u16>) {
            let node = document.Node(index);
            if node.Type() == DOMNodeType::kText {
                text.extend_from_slice(&document.CharacterDataUTF16(index));
                return;
            }
            for &child in node.Children() {
                append(document, child, text);
            }
        }
        let mut text = Vec::new();
        append(self, node, &mut text);
        text.into()
    }

    pub fn SetTextContentUTF16(&mut self, node: usize, data: &[u16]) {
        if matches!(
            self.arena[node].node_type,
            DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
        ) {
            self.ReplaceDataUTF16(node, 0, self.CharacterDataUTF16(node).len(), data)
                .expect("character data owner");
            return;
        }
        assert!(matches!(
            self.arena[node].node_type,
            DOMNodeType::kElement | DOMNodeType::kDocumentFragment
        ));
        let children = self.arena[node].children.clone();
        for child in children {
            self.Remove(child);
        }
        if !data.is_empty() {
            let child = self.CreateTextUTF16(data);
            self.AppendChild(node, child);
        }
    }
    pub fn DescendantTextUTF16(&self, node: usize) -> Rc<[u16]> {
        fn append(document: &PersistentDocument, node: usize, result: &mut Vec<u16>) {
            for &child in &document.arena[node].children {
                if document.arena[child].node_type == DOMNodeType::kText {
                    result.extend_from_slice(&document.CharacterDataUTF16(child));
                } else {
                    append(document, child, result);
                }
            }
        }
        let mut result = Vec::new();
        append(self, node, &mut result);
        result.into()
    }
    pub fn SetTextContent(&mut self, node: usize, data: String) {
        if matches!(
            self.arena[node].node_type,
            DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
        ) {
            let units = data.encode_utf16().collect::<Vec<_>>();
            self.ReplaceDataUTF16(node, 0, self.CharacterDataUTF16(node).len(), &units)
                .expect("character data owner");
            return;
        }
        if !matches!(
            self.arena[node].node_type,
            DOMNodeType::kElement | DOMNodeType::kDocumentFragment
        ) {
            invalid_argument("text content requires a character or container node");
        }
        let children = self.arena[node].children.clone();
        for child in children {
            self.Remove(child);
        }
        if !data.is_empty() {
            self.AppendText(node, &data);
        }
    }

    // cpp: dom/document.cc:267-278
    pub fn ControlChecked(&self, node: usize) -> bool {
        self.control_checked_state
            .get(&self.arena[node].id)
            .copied()
            .unwrap_or_else(|| self.arena[node].FindAttribute("checked").is_some())
    }
    // cpp: HTMLInputElement::dirty_checkedness_. Retain the current value in
    // the same checked-state owner even when assigning an equal checked value.
    pub fn ClearControlCheckednessDirty(&mut self, node: usize) {
        // ResetImpl has just restored checkedness from this same attribute.
        debug_assert_eq!(
            self.ControlChecked(node),
            self.arena[node].FindAttribute("checked").is_some()
        );
        self.control_checked_state.remove(&self.arena[node].id);
    }
    pub fn SetControlCheckednessDirty(&mut self, node: usize) {
        let checked = self.ControlChecked(node);
        self.control_checked_state
            .insert(self.arena[node].id, checked);
    }

    pub fn SetControlChecked(&mut self, node: usize, checked: bool) {
        if !self.arena[node].IsElement("input", DOMNamespace::kHTML) {
            invalid_argument("checked state requires an input element");
        }
        let changed = self.ControlChecked(node) != checked;
        self.control_checked_state
            .insert(self.arena[node].id, checked);
        if !changed {
            return;
        }
        if self.InputType(node) == Some(DOMInputType::kRadio) {
            self.UpdateRadioCheckedState(node);
            self.InvalidateNodeStyle(node);
        } else {
            self.InvalidateNodeStyle(node);
        }
        self.RecordLayoutInputChange(node);
    }

    pub fn ControlIndeterminate(&self, node: usize) -> bool {
        self.control_indeterminate_state
            .get(&self.arena[node].id)
            .copied()
            .unwrap_or(false)
    }

    // cpp: html_input_element.cc:1262-1276.
    pub fn SetControlIndeterminate(&mut self, node: usize, indeterminate: bool) {
        if !self.arena[node].IsElement("input", DOMNamespace::kHTML) {
            invalid_argument("indeterminate state requires an input element");
        }
        if self.ControlIndeterminate(node) == indeterminate {
            return;
        }
        self.control_indeterminate_state
            .insert(self.arena[node].id, indeterminate);
        self.InvalidateNodeStyle(node);
        self.RecordLayoutInputChange(node);
    }

    // cpp: checkbox_input_type.cc:127-129; radio_input_type.cc:317-365;
    //      html_progress_element.cc:170-172.
    pub fn ShouldAppearIndeterminate(&self, element: usize) -> bool {
        match self.InputType(element) {
            Some(DOMInputType::kCheckbox) => self.ControlIndeterminate(element),
            Some(DOMInputType::kRadio) => !self.RadioGroupHasCheckedButton(element),
            _ => {
                self.arena[element].IsElement("progress", DOMNamespace::kHTML)
                    && self.arena[element].FindAttribute("value").is_none()
            }
        }
    }

    // cpp: element.cc:1855-1890; html_dialog_element.cc:552-616;
    //      html_element.cc ShowPopoverInternal/HidePopoverInternal.
    pub fn PopoverPickerPointerdown(&self) -> Option<(DOMOwnerHandle, (f32, f32))> {
        self.popover_picker_pointerdown.clone()
    }

    pub fn SetPopoverPickerPointerdown(&mut self, value: Option<(DOMOwnerHandle, (f32, f32))>) {
        self.popover_picker_pointerdown = value;
    }

    pub fn IsPopover(&self, element: usize) -> bool {
        self.arena[element].Namespace() == DOMNamespace::kHTML
            && self
                .popup_lifecycle_state
                .popovers
                .get(&self.arena[element].id)
                .is_some_and(|data| {
                    data.popover_type != foundation::popup_types::PopoverValueType::None
                })
    }

    pub fn PopoverOpen(&self, element: usize) -> bool {
        self.IsPopover(element)
            && self
                .popup_lifecycle_state
                .popovers
                .get(&self.arena[element].id)
                .is_some_and(|data| {
                    data.visibility_state
                        == foundation::popup_types::PopoverVisibilityState::Showing
                })
    }

    pub fn DialogIsModal(&self, element: usize) -> bool {
        self.arena[element].IsHTMLElement("dialog")
            && self
                .popup_lifecycle_state
                .dialogs
                .get(&self.arena[element].id)
                .is_some_and(|data| data.is_modal)
    }

    pub fn IsDialogInTopLayer(&self, element: usize) -> bool {
        self.DialogIsModal(element) && self.top_layer_elements.contains(&self.arena[element].id)
    }

    pub fn IsPopoverInTopLayer(&self, element: usize) -> bool {
        self.PopoverOpen(element) && self.top_layer_elements.contains(&self.arena[element].id)
    }

    pub fn IsInTopLayer(&self, element: usize) -> bool {
        self.top_layer_elements.contains(&self.arena[element].id)
    }

    pub fn AddToTopLayer(&mut self, element: usize) {
        if self.IsInTopLayer(element) && self.IsScheduledForTopLayerRemoval(element).is_some() {
            self.RemoveFromTopLayerImmediately(element);
        }
        let id = self.arena[element].id;
        if !self.top_layer_elements.contains(&id) {
            self.top_layer_elements.push(id);
            self.NotifyDisplayLockTopLayerAdded(element);
            // Element::SetIsInTopLayer forces reattachment even if removing
            // and re-adding left computed overlay style unchanged. Top-layer
            // ordering belongs to the LayoutView, not a DOM-local subtree.
            self.style_state.layout_inputs_require_tree = true;
            self.style_state.measurement_geometry_revision = self
                .style_state
                .measurement_geometry_revision
                .wrapping_add(1);
        }
    }

    fn RemoveFromTopLayerIfUnused(&mut self, element: usize) {
        let id = self.arena[element].id;
        if !self
            .popup_lifecycle_state
            .dialogs
            .get(&id)
            .is_some_and(|data| data.is_modal)
            && !self.open_popovers.contains(&id)
            && !self.fullscreen_element_stack.contains(&id)
        {
            let previous = self.top_layer_elements.len();
            self.top_layer_elements.retain(|candidate| *candidate != id);
            if self.top_layer_elements.len() != previous {
                self.ScheduleDisplayLockTopLayerCheck(element);
                self.style_state.layout_inputs_require_tree = true;
                self.style_state.measurement_geometry_revision = self
                    .style_state
                    .measurement_geometry_revision
                    .wrapping_add(1);
            }
        }
    }

    // cpp: fullscreen/fullscreen.cc GoFullscreen, Unfullscreen,
    //      FullscreenElementFrom, IsInFullscreenElementStack.
    pub fn RequestFullscreen(&mut self, element: usize) {
        if self.arena[element].Type() != DOMNodeType::kElement {
            invalid_argument("requestFullscreen requires an element");
        }
        if !self.IsConnectedToDocument(element) {
            invalid_argument("requestFullscreen requires a connected element");
        }
        let id = self.arena[element].id;
        if self.fullscreen_element_stack.last() == Some(&id) {
            return;
        }
        // A flagged element is moved to the top exactly as Blink removes and
        // re-adds an existing top-layer element in GoFullscreen.
        self.fullscreen_element_stack
            .retain(|candidate| *candidate != id);
        self.top_layer_elements.retain(|candidate| *candidate != id);
        self.fullscreen_element_stack.push(id);
        self.AddToTopLayer(element);
        // Fullscreen changes inertness across most of the document and also
        // changes :fullscreen, :modal and ancestor selector state.
        self.InvalidateAllStyles();
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::TREE);
    }

    /// Fullscreen::ElementRemoved, fullscreen.cc. Called at Element's removal
    /// stage through the same native fullscreen stack, before dialog teardown.
    pub fn FullscreenElementRemoved(&mut self, element: usize) {
        let id = self.arena[element].id;
        if self.fullscreen_element_stack.last() == Some(&id) {
            self.ExitFullscreen();
        } else {
            self.fullscreen_element_stack
                .retain(|&candidate| candidate != id);
        }
    }

    pub fn ExitFullscreen(&mut self) {
        if self.fullscreen_element_stack.is_empty() {
            return;
        }
        let flagged = std::mem::take(&mut self.fullscreen_element_stack);
        for id in flagged {
            if let Some(element) = self.FindNodeById(id) {
                self.RemoveFromTopLayerIfUnused(element);
            } else {
                self.top_layer_elements.retain(|candidate| *candidate != id);
            }
        }
        self.InvalidateAllStyles();
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::TREE);
    }

    pub fn HasFullscreenElements(&self) -> bool {
        !self.fullscreen_element_stack.is_empty()
    }

    pub fn FullscreenElement(&self) -> Option<usize> {
        self.fullscreen_element_stack
            .last()
            .and_then(|id| self.FindNodeById(*id))
    }

    // cpp: dom/document.h CssTarget; dom/document.cc SetCSSTarget.
    // Fragment navigation owns updates to this state. SelectorChecker only
    // observes the retained element identity, including the ordinary null
    // state for documents whose URL has no fragment.
    pub fn CssTarget(&self) -> Option<usize> {
        self.css_target.and_then(|id| self.FindNodeById(id))
    }

    pub fn SetCSSTarget(&mut self, target: Option<usize>) {
        let target = target.filter(|&element| {
            self.arena[element].Type() == DOMNodeType::kElement
                && self.IsConnectedToDocument(element)
        });
        let next = target.map(|element| self.arena[element].id);
        if self.css_target == next {
            return;
        }
        if let Some(previous) = self.CssTarget() {
            self.InvalidateNodeStyle(previous);
        }
        self.css_target = next;
        if let Some(target) = target {
            self.InvalidateNodeStyle(target);
        }
    }

    // cpp: SVGImage::ApplyViewInfo / SVGSVGElement::SetViewSpec.
    // Passing None is intentional: every draw clears a preceding variant's
    // view state before laying out the requested container.
    pub fn SetSVGImageViewInfo(
        &mut self,
        root: usize,
        target: Option<usize>,
        view_spec: Option<SVGImageViewSpec>,
    ) {
        if !self.arena[root].IsElement("svg", DOMNamespace::kSVG)
            || !self.IsConnectedToDocument(root)
        {
            invalid_argument("SVG image view requires a connected root svg element");
        }
        self.SetCSSTarget(target);
        let root_id = self.arena[root].id;
        if self.svg_image_view_root == Some(root_id) && self.svg_image_view_spec == view_spec {
            return;
        }
        self.svg_image_view_root = Some(root_id);
        self.svg_image_view_spec = view_spec;
        self.style_state.layout_inputs_require_tree = true;
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
    }

    pub fn SVGImageViewSpecFor(&self, root: usize) -> Option<SVGImageViewSpec> {
        (self.svg_image_view_root == Some(self.arena[root].id))
            .then_some(self.svg_image_view_spec)
            .flatten()
    }

    /// SVGElement's retained instance registry. Callers still select connected
    /// instances; detached former instances cannot affect the active tree.
    pub fn SVGInstanceElements(&self) -> impl Iterator<Item = usize> + '_ {
        self.corresponding_svg_elements
            .keys()
            .filter_map(|id| self.FindNodeById(*id))
    }

    pub fn CorrespondingSVGElement(&self, element: usize) -> Option<usize> {
        self.corresponding_svg_elements
            .get(&self.arena[element].id)
            .and_then(|id| self.FindNodeById(*id))
    }

    pub fn SetCorrespondingSVGElement(&mut self, element: usize, corresponding: Option<usize>) {
        self.BumpCollectionMembershipRevision();
        if self.IsConnectedToDocument(element) {
            self.BumpSVGResourceDOMRevision();
        }
        self.InvalidateNodeStyle(element);
        let id = self.arena[element].id;
        if let Some(corresponding) = corresponding {
            self.corresponding_svg_elements
                .insert(id, self.arena[corresponding].id);
        } else {
            self.corresponding_svg_elements.remove(&id);
        }
    }

    pub fn FullscreenFlag(&self, element: usize) -> bool {
        self.fullscreen_element_stack
            .contains(&self.arena[element].id)
    }

    pub fn ContainsFullscreenElement(&self, element: usize) -> bool {
        let Some(mut current) = self.FullscreenElement() else {
            return false;
        };
        while let Some(parent) = self.FlatTreeParentElement(current) {
            if parent == element {
                return true;
            }
            current = parent;
        }
        false
    }

    pub fn ShowModalDialog(&mut self, element: usize) {
        if !self.arena[element].IsHTMLElement("dialog") {
            invalid_argument("showModal requires a dialog element");
        }
        if !self.IsConnectedToDocument(element) {
            invalid_argument("showModal requires a connected dialog");
        }
        if self.DialogIsModal(element) {
            return;
        }
        if self.arena[element].FindAttribute("open").is_some() {
            invalid_argument("a non-modal open dialog cannot be shown modally");
        }
        if self.PopoverOpen(element) {
            invalid_argument("a dialog shown as a popover cannot be shown modally");
        }
        self.AddToTopLayer(element);
        self.popup_lifecycle_state
            .dialogs
            .entry(self.arena[element].id)
            .or_default()
            .is_modal = true;
        self.SetAttribute(
            element,
            DOMAttribute {
                local_name: "open".into(),
                ..DOMAttribute::default()
            },
        );
        self.InvalidateNodeStyle(element);
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::TREE);
    }

    pub fn CloseDialog(&mut self, element: usize) {
        if !self.arena[element].IsHTMLElement("dialog") {
            invalid_argument("close requires a dialog element");
        }
        if self.arena[element].FindAttribute("open").is_none() {
            return;
        }
        self.RemoveAttribute(element, "open", "");
        if let Some(data) = self
            .popup_lifecycle_state
            .dialogs
            .get_mut(&self.arena[element].id)
        {
            data.is_modal = false;
        }
        self.RemoveFromTopLayerIfUnused(element);
        self.InvalidateNodeStyle(element);
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::TREE);
    }

    // Source: HTMLElement::GetOpenPopoverTarget and PopoverData::invoker.
    // Both associations retain the same canonical nodes; no Page-side map is
    // allowed to reconstruct popover focus ownership from attributes.
    pub fn OpenPopoverInvoker(&self, element: usize) -> Option<usize> {
        self.PopoverOpen(element)
            .then(|| {
                self.popup_lifecycle_state
                    .popovers
                    .get(&self.arena[element].id)
                    .and_then(|data| data.invoker)
            })
            .flatten()
            .and_then(|id| self.FindNodeById(id))
    }
    pub fn OpenPopoverTarget(&self, invoker: usize) -> Option<usize> {
        let popover = *self.invoked_popovers.get(&invoker)?;
        self.PopoverOpen(popover).then_some(popover)
    }
    pub fn SetPopoverInvoker(&mut self, element: usize, invoker: Option<usize>) {
        let id = self.arena[element].id;
        let old = self
            .popup_lifecycle_state
            .popovers
            .get(&id)
            .and_then(|data| data.invoker)
            .and_then(|id| self.FindNodeById(id));
        if let Some(old) = old {
            if self.invoked_popovers.get(&old) == Some(&element) {
                self.invoked_popovers.remove(&old);
            }
        }
        let invoker = invoker.filter(|&invoker| {
            self.NodeDocumentHandle(invoker) == self.NodeDocumentHandle(element)
        });
        if let Some(data) = self.popup_lifecycle_state.popovers.get_mut(&id) {
            data.invoker = invoker.map(|invoker| self.arena[invoker].id);
        }
        if let Some(invoker) = invoker {
            self.invoked_popovers.insert(invoker, element);
        }
    }
    pub fn ShowPopoverFromInvoker(&mut self, element: usize, invoker: Option<usize>) {
        if self.PopoverOpen(element) {
            return;
        }
        self.ShowPopover(element);
        self.SetPopoverInvoker(element, invoker);
    }
    pub fn ShowPopover(&mut self, element: usize) {
        if !self.IsPopover(element) {
            invalid_argument("showPopover requires a popover element");
        }
        if !self.IsConnectedToDocument(element) {
            invalid_argument("showPopover requires a connected element");
        }
        if self.PopoverOpen(element) {
            return;
        }
        if self.DialogIsModal(element) {
            invalid_argument("a modal dialog cannot be shown as a popover");
        }
        self.AddToTopLayer(element);
        let kind = self.arena[element]
            .FindAttribute("popover")
            .map(|attr| attr.value.as_str());
        let kind = match kind {
            Some(value) if value.is_empty() || value.eq_ignore_ascii_case("auto") => {
                foundation::popup_types::PopoverValueType::Auto
            }
            Some(value) if value.eq_ignore_ascii_case("hint") => {
                foundation::popup_types::PopoverValueType::Hint
            }
            Some(_) => foundation::popup_types::PopoverValueType::Manual,
            None => foundation::popup_types::PopoverValueType::None,
        };
        let data = self
            .popup_lifecycle_state
            .popovers
            .entry(self.arena[element].id)
            .or_default();
        data.popover_type = kind;
        data.visibility_state = foundation::popup_types::PopoverVisibilityState::Showing;
        self.open_popovers.insert(self.arena[element].id);
        self.InvalidateNodeStyle(element);
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::TREE);
    }

    pub fn HidePopover(&mut self, element: usize) {
        if !self.IsPopover(element) {
            invalid_argument("hidePopover requires a popover element");
        }
        if !self.PopoverOpen(element) {
            return;
        }
        self.open_popovers.remove(&self.arena[element].id);
        if let Some(data) = self
            .popup_lifecycle_state
            .popovers
            .get_mut(&self.arena[element].id)
        {
            data.visibility_state = foundation::popup_types::PopoverVisibilityState::Hidden;
        }
        self.SetPopoverInvoker(element, None);
        self.RemoveFromTopLayerIfUnused(element);
        self.InvalidateNodeStyle(element);
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::TREE);
    }

    // cpp: listed_element.cc form-owner reset algorithm. This compact owner
    // covers both source listed controls and ElementInternals targets.
    pub fn FormOwner(&self, control: usize) -> Option<usize> {
        if self.ElementInternalsTargetIsFormAssociated(control) {
            return self.ElementInternalsFormOwner(control);
        }
        self.ComputeListedFormOwner(control)
    }
    pub(crate) fn ComputeListedFormOwner(&self, control: usize) -> Option<usize> {
        if let Some(id) = self.arena[control]
            .FindAttribute("form")
            .map(|attribute| attribute.value.as_str())
            .filter(|_| self.IsConnectedToDocument(control))
        {
            return self
                .FindElementByIdInTreeScope(control, id)
                .filter(|&element| self.arena[element].IsElement("form", DOMNamespace::kHTML));
        }
        let mut ancestor = self.arena[control].parent;
        while let Some(element) = ancestor {
            if self.arena[element].IsElement("form", DOMNamespace::kHTML) {
                return Some(element);
            }
            ancestor = self.arena[element].parent;
        }
        None
    }

    // cpp: submit_input_type.cc:73-75; image_input_type.cc:153-155;
    //      html_button_element.cc:287-289.
    pub fn CanBeSuccessfulSubmitButton(&self, element: usize) -> bool {
        if self.arena[element].IsElement("input", DOMNamespace::kHTML) {
            return matches!(
                self.InputType(element),
                Some(DOMInputType::kSubmit | DOMInputType::kImage)
            );
        }
        if !self.arena[element].IsElement("button", DOMNamespace::kHTML) {
            return false;
        }
        let is_submit = !self.arena[element]
            .FindAttribute("type")
            .is_some_and(|attribute| {
                matches!(
                    attribute.value.to_ascii_lowercase().as_str(),
                    "button" | "reset"
                )
            });
        if !is_submit {
            return false;
        }
        // A customizable-select's slotted button is not a form submit button.
        let owner_select = self.arena[element].Parent().filter(|&parent| {
            self.arena[parent].IsElement("select", DOMNamespace::kHTML)
                && self.SelectSlottedButton(parent) == Some(element)
        });
        owner_select.is_none()
    }

    // cpp: html_form_element.cc:1485-1495. ListedElements is in tree order,
    // including controls associated through the form attribute.
    fn DefaultButtonForForm(&self, form: usize) -> Option<usize> {
        if !self.arena[form].IsElement("form", DOMNamespace::kHTML) {
            return None;
        }
        fn visit(document: &PersistentDocument, node: usize, form: usize) -> Option<usize> {
            if document.CanBeSuccessfulSubmitButton(node) && document.FormOwner(node) == Some(form)
            {
                return Some(node);
            }
            for &child in document.arena[node].Children() {
                if let Some(button) = visit(document, child, form) {
                    return Some(button);
                }
            }
            None
        }
        visit(self, self.TreeScopeRoot(form), form)
    }

    // cpp: base_checkable_input_type.cc:102-104;
    //      html_option_element.cc:217-219;
    //      base_button_input_type.cc:103-108;
    //      html_button_element.cc:332-337.
    pub fn MatchesDefaultPseudoClass(&self, element: usize) -> bool {
        if self.arena[element].IsElement("option", DOMNamespace::kHTML) {
            return self.arena[element].FindAttribute("selected").is_some();
        }
        if self.arena[element].IsElement("input", DOMNamespace::kHTML)
            && matches!(
                self.InputType(element),
                Some(DOMInputType::kCheckbox | DOMInputType::kRadio)
            )
        {
            // :default observes the content attribute, not live checkedness.
            return self.arena[element].FindAttribute("checked").is_some();
        }
        if self.arena[element].IsElement("menuitem", DOMNamespace::kHTML) {
            return self.arena[element]
                .FindAttribute("defaultchecked")
                .is_some();
        }
        if !self.CanBeSuccessfulSubmitButton(element) {
            return false;
        }
        let Some(form) = self.FormOwner(element) else {
            return false;
        };
        self.DefaultButtonForForm(form) == Some(element)
    }

    fn RadioGroupMembers(&self, radio: usize) -> Vec<usize> {
        self.ResidentRadioGroupMembers(radio)
    }

    // Minimal boundary consumed by Interaction's checkbox/radio activation
    // transaction. DOM remains the owner of form and TreeScope grouping.
    pub fn RadioGroupNodeIds(&self, radio: usize) -> Vec<u64> {
        self.RadioGroupMembers(radio)
            .into_iter()
            .map(|member| self.arena[member].id)
            .collect()
    }

    fn RadioGroupHasCheckedButton(&self, radio: usize) -> bool {
        self.ControlChecked(radio)
            || self
                .RadioGroupMembers(radio)
                .into_iter()
                .any(|member| self.ControlChecked(member))
    }

    fn InvalidateRadioGroup(&mut self, radio: usize) {
        for member in self.RadioGroupMembers(radio) {
            self.InvalidateNodeStyle(member);
        }
    }

    // cpp: dom/document.cc:280-326
    fn AppendDescendantText(&self, node: usize, output: &mut String) {
        if self.arena[node].node_type == DOMNodeType::kText {
            output.push_str(&self.arena[node].data);
        }
        for &child in &self.arena[node].children {
            self.AppendDescendantText(child, output);
        }
    }

    // HTMLOptionElement::value, html_option_element.cc:339-345.
    pub fn OptionValue(&self, option: usize) -> String {
        String::from_utf16_lossy(&self.OptionValueUTF16(option))
    }

    pub fn OptionSelected(&self, option: usize) -> bool {
        if !self.arena[option].IsElement("option", DOMNamespace::kHTML) {
            return false;
        }
        self.option_selected_state
            .get(&self.arena[option].id)
            .copied()
            .unwrap_or_else(|| self.arena[option].FindAttribute("selected").is_some())
    }

    pub fn OptionSelectionIsDirty(&self, option: usize) -> bool {
        self.arena[option].IsElement("option", DOMNamespace::kHTML)
            && self.option_dirty_state.contains(&self.arena[option].id)
    }

    // Source html_option_element.h SetDirty. Source interaction defaults keep
    // selectedness and dirtiness distinct from the IDL SetSelected transaction.
    pub fn SetOptionDirty(&mut self, option: usize, dirty: bool) {
        if dirty {
            self.option_dirty_state.insert(self.arena[option].id);
        } else {
            self.option_dirty_state.remove(&self.arena[option].id);
        }
    }

    pub fn UpdateOptionMutationObserver(&mut self, option: usize, in_style_recalc: bool) {
        self.attribute_lifecycle
            .RecordOptionMutationObserverChanged(self.arena[option].id, in_style_recalc);
    }

    // cpp: html_option_element.cc:386-412.
    pub fn SetOptionSelectedState(&mut self, option: usize, selected: bool) -> bool {
        self.SetOptionSelectedStateWithObserverUpdate(option, selected, false)
    }
    pub fn SetOptionSelectedStateWithObserverUpdate(
        &mut self,
        option: usize,
        selected: bool,
        skip_observer: bool,
    ) -> bool {
        if !self.arena[option].IsElement("option", DOMNamespace::kHTML) {
            invalid_argument("selected state requires an option element");
        }
        if self.OptionSelected(option) == selected {
            return false;
        }
        self.option_selected_state
            .insert(self.arena[option].id, selected);
        self.InvalidateNodeStyle(option);
        if let Some(select) = self.OptionOwnerSelect(option) {
            self.control_value_state.remove(&self.arena[select].id);
            self.InvalidateNodeStyle(select);
        }
        self.style_state.layout_inputs_require_tree = true;
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        if !skip_observer {
            self.attribute_lifecycle
                .RecordOptionMutationObserverChanged(self.arena[option].id, false);
        }
        true
    }

    pub fn SelectIsMultiple(&self, select: usize) -> bool {
        match &self.arena[select].class {
            DOMNodeClass::HTMLSelectElement(data) => data.is_multiple,
            _ => false,
        }
    }

    pub fn SelectSize(&self, select: usize) -> u32 {
        match &self.arena[select].class {
            DOMNodeClass::HTMLSelectElement(data) => data.size,
            _ => 0,
        }
    }

    // cpp: html_select_element.cc:1050-1111.
    fn SelectOptionInternal(
        &mut self,
        select: usize,
        option: Option<usize>,
        deselect_others: bool,
        make_option_dirty: bool,
        autofill_state: DOMAutofillState,
    ) {
        if !self.arena[select].IsElement("select", DOMNamespace::kHTML) {
            invalid_argument("selection owner requires a select element");
        }
        if option.is_some_and(|option| self.OptionOwnerSelect(option) != Some(select)) {
            return;
        }
        self.SetAutofillState(
            select,
            if option.is_some() {
                autofill_state
            } else {
                DOMAutofillState::kNotFilled
            },
        );
        if let Some(option) = option {
            self.SetOptionSelectedState(option, true);
            if make_option_dirty {
                self.option_dirty_state.insert(self.arena[option].id);
            }
        }
        if deselect_others {
            for id in self.SelectOptionNodeIds(select) {
                let Some(candidate) = self.FindNodeById(id) else {
                    continue;
                };
                if Some(candidate) != option {
                    self.SetOptionSelectedState(candidate, false);
                }
            }
        }
        if let DOMNodeClass::HTMLSelectElement(data) = &mut self.arena[select].class {
            if data.uses_menu_list {
                data.last_on_change_option = option;
            }
        }
        self.control_value_state.remove(&self.arena[select].id);
        // SelectType::DidSelectOption invalidates the same ListedElement cache.
        self.SetControlNeedsValidityCheck(select);
        self.InvalidateNodeStyle(select);
        self.style_state.layout_inputs_require_tree = true;
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
    }

    // cpp: html_select_element.cc:778-821.
    fn ResetSelectToDefaultSelection(&mut self, select: usize) {
        if self.SelectIsMultiple(select) {
            return;
        }
        let options = self
            .SelectOptionNodeIds(select)
            .into_iter()
            .filter_map(|id| self.FindNodeById(id))
            .collect::<Vec<_>>();
        let mut first_enabled = None;
        let mut last_selected = None;
        for option in options {
            if self.OptionSelected(option) {
                if let Some(previous) = last_selected {
                    self.SetOptionSelectedState(previous, false);
                }
                last_selected = Some(option);
            }
            if first_enabled.is_none() && !self.MatchesDisabledPseudoClass(option) {
                first_enabled = Some(option);
            }
        }
        if last_selected.is_none() && self.SelectSize(select) <= 1 {
            self.SelectOptionInternal(
                select,
                first_enabled,
                true,
                false,
                DOMAutofillState::kNotFilled,
            );
            last_selected = first_enabled;
        }
        // ResetToDefaultSelection writes this member even for a listbox.
        if let DOMNodeClass::HTMLSelectElement(data) = &mut self.arena[select].class {
            data.last_on_change_option = last_selected;
        }
    }

    fn FirstSelectedOption(&self, select: usize) -> Option<usize> {
        self.SelectOptionNodeIds(select)
            .into_iter()
            .filter_map(|id| self.FindNodeById(id))
            .find(|&option| self.OptionSelected(option))
    }

    // cpp: html_select_element.cc:1290-1313. Preserve the first live
    // selection while switching rendering/selection mode, then enforce the
    // single-select invariant when needed.
    fn SelectMultipleAttributeChanged(
        &mut self,
        select: usize,
        old_multiple: bool,
        old_selected_option: Option<usize>,
    ) {
        if old_multiple == self.SelectIsMultiple(select) {
            return;
        }
        if let Some(option) = old_selected_option {
            self.SelectOptionInternal(
                select,
                Some(option),
                true,
                false,
                DOMAutofillState::kNotFilled,
            );
        } else {
            self.ResetSelectToDefaultSelection(select);
        }
    }

    // cpp: html_select_element.cc:923-963. Insertion order matters: a newly
    // inserted selected option wins in a single select even when inserted
    // before an existing selected option.
    fn HTMLElementSubtreeInserted(&mut self, root: usize) {
        enum PostInsertion {
            SelectedContent,
            Option,
            Frame,
        }
        let mut options = Vec::new();
        let mut post_notifications = Vec::new();
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            self.VTTNodeInserted(node);
            self.CanvasSubtreeInsertedInto(node);
            if (self.arena[node].IsHTMLElement("a") || self.arena[node].IsHTMLElement("area"))
                && self.IsConnectedToDocument(node)
            {
                self.attribute_lifecycle
                    .RecordAnchorInserted(self.arena[node].id);
            }
            if self.IsTextScaleMetaElement(node) && self.IsConnectedToDocument(node) {
                self.TextScaleMetaChanged();
            }
            if self.arena[node].IsHTMLElement("base") && self.IsConnectedToDocument(node) {
                self.ProcessBaseElementURL();
            }
            if self.ClassSupportsFormStateRestore(node)
                && self.IsConnectedToDocument(node)
                && !self.IsInShadowTree(node)
            {
                self.stateful_form_control_revision = self
                    .stateful_form_control_revision
                    .checked_add(1)
                    .expect("form-control invalidation revision");
                self.attribute_lifecycle
                    .RecordStatefulFormControlConnected(self.arena[node].id);
            }
            if self.SelectedContentInsertedInto(node) {
                post_notifications.push((node, PostInsertion::SelectedContent));
            }
            if self.IsFrameOwnerElement(node) {
                post_notifications.push((node, PostInsertion::Frame));
            }
            // Element.cc:4239-4248: re-register retained observations only
            // after this insertion transaction releases the Document borrow.
            if self.IsConnectedToDocument(node) {
                self.RecordIntersectionObserverConnected(node);
            }
            if self.IsMediaElement(node) {
                self.AttributeLifecycle()
                    .RecordMediaInserted(self.arena[node].id);
            }
            if self.IsHTMLSourceElement(node) {
                if let Some(parent) = self.Node(node).Parent() {
                    if self.IsMediaElement(parent) {
                        self.AttributeLifecycle().RecordMediaSourceChanged(
                            self.Node(parent).Id(),
                            self.Node(node).Id(),
                            true,
                        );
                    }
                }
            }
            self.ImageMapElementInserted(node);
            self.SelectCountedElementInserted(node);
            self.UpdateOverscrollCommandInvoker(node);
            self.ResetMenuItemAncestorCache(node);
            if self.arena[node]
                .FindAttribute("id")
                .is_some_and(|id| !id.value.is_empty())
            {
                self.MarkOverscrollCommandTargetsDirty();
            }
            // ContainerNode::NotifyNodeInserted traverses every inserted
            // descendant once. Reuse this native insertion walk for details'
            // source virtual; no second subtree snapshot or Page walk.
            if matches!(self.arena[node].class, DOMNodeClass::HTMLDialogElement) {
                self.attribute_lifecycle.RecordDialogInsertedInto(
                    self.arena[node].id,
                    self.IsConnectedToDocument(node),
                );
            }
            if matches!(self.arena[node].class, DOMNodeClass::HTMLDetailsElement(_)) {
                self.attribute_lifecycle
                    .RecordDetailsInsertedInto(self.arena[node].id);
            }
            if self.arena[node].IsElement("option", DOMNamespace::kHTML) {
                options.push(node);
                post_notifications.push((node, PostInsertion::Option));
            }
            pending.extend(self.arena[node].Children().iter().rev().copied());
            // ContainerNode::NotifyNodeInsertedInternal cc:1401-1403 visits
            // an attached host's existing shadow descendants in this same turn.
            if let Some(root) = self.ShadowRoot(node) {
                pending.push(root);
            }
        }
        for &option in &options {
            let Some(select) = self.OptionOwnerSelect(option) else {
                continue;
            };
            if self.OptionSelected(option) {
                self.SelectOptionInternal(
                    select,
                    Some(option),
                    !self.SelectIsMultiple(select),
                    false,
                    DOMAutofillState::kNotFilled,
                );
            } else if self.FirstSelectedOption(select).is_none()
                && self.SelectSize(select) <= 1
                && !self.MatchesDisabledPseudoClass(option)
            {
                self.ResetSelectToDefaultSelection(select);
            }
        }
        // ContainerNode::DidInsertNodeVector performs post-connection steps
        // only after all InsertedInto calls have populated TreeOrderedList.
        for (node, notification) in post_notifications {
            if self.IsConnectedToDocument(node) {
                match notification {
                    PostInsertion::SelectedContent => {
                        self.SelectedContentDidNotifySubtreeInsertions(node)
                    }
                    PostInsertion::Option => self.OptionDidNotifySubtreeInsertions(node),
                    PostInsertion::Frame => self.FrameElementDidNotifySubtreeInsertions(node),
                }
            }
        }
    }

    // cpp: html_option_element.cc:356-384. This is the typed IDL mutation
    // boundary; no JavaScript surface is synthesized here.
    pub fn SetOptionSelected(&mut self, option: usize, selected: bool) {
        if !self.arena[option].IsElement("option", DOMNamespace::kHTML) {
            invalid_argument("selected state requires an option element");
        }
        let owner_before = self.OptionOwnerSelect(option);
        let was_selected = self.OptionSelected(option);
        if self.SetOptionSelectedState(option, selected) {
            self.OptionSelectionChanged(option, selected);
        }
        if owner_before.is_none() || was_selected != self.OptionSelected(option) {
            self.option_dirty_state.insert(self.arena[option].id);
        }
    }

    // cpp: html_option_element.cc:326-329. defaultSelected mutations only
    // update live selectedness while the option remains non-dirty.
    fn OptionSelectedAttributeChanged(
        &mut self,
        option: usize,
        old_selectedness: bool,
        selected_attribute_present: bool,
    ) {
        if self.OptionSelectionIsDirty(option) {
            return;
        }
        self.option_selected_state
            .insert(self.arena[option].id, selected_attribute_present);
        if old_selectedness == selected_attribute_present {
            return;
        }
        self.InvalidateNodeStyle(option);
        self.OptionSelectionChanged(option, selected_attribute_present);
    }

    // cpp: html_select_element.cc:912-920.
    fn OptionSelectionChanged(&mut self, option: usize, option_is_selected: bool) {
        let Some(select) = self.OptionOwnerSelect(option) else {
            return;
        };
        if option_is_selected {
            self.SelectOptionInternal(
                select,
                Some(option),
                !self.SelectIsMultiple(select),
                false,
                DOMAutofillState::kNotFilled,
            );
        } else if !self.SelectUsesMenuList(select) || self.SelectIsMultiple(select) {
            self.SelectOptionInternal(
                select,
                None,
                !self.SelectIsMultiple(select),
                false,
                DOMAutofillState::kNotFilled,
            );
        } else {
            self.ResetSelectToDefaultSelection(select);
        }
    }

    fn VisitOptions(&self, node: usize, first: &mut Option<usize>, selected: &mut Option<usize>) {
        for &child in &self.arena[node].children {
            if self.arena[child].IsElement("option", DOMNamespace::kHTML) {
                if first.is_none() {
                    *first = Some(child);
                }
                if selected.is_none() && self.OptionSelected(child) {
                    *selected = Some(child);
                }
            }
            self.VisitOptions(child, first, selected);
        }
    }
    pub fn MarkSelectionchangeEventScheduled(&mut self, target: u64) -> bool {
        self.scheduled_selectionchange_targets.insert(target)
    }
    pub fn ClearScheduledSelectionchangeEvent(&mut self, target: u64) {
        self.scheduled_selectionchange_targets.remove(&target);
    }
    pub fn ShouldUpdateSelectionAfterLayout(&self) -> bool {
        self.should_update_selection_after_layout
    }
    pub fn SetShouldUpdateSelectionAfterLayout(&mut self, pending: bool) {
        self.should_update_selection_after_layout = pending;
    }
    pub(crate) fn TextControlOpaqueRangeState(
        &mut self,
        node: usize,
    ) -> Rc<RefCell<crate::opaque_range::TextControlOpaqueRanges>> {
        self.text_control_runtime
            .entry(self.arena[node].id)
            .or_default()
            .opaque_ranges
            .clone()
    }
    pub(crate) fn ExistingTextControlOpaqueRangeState(
        &self,
        node: usize,
    ) -> Option<Rc<RefCell<crate::opaque_range::TextControlOpaqueRanges>>> {
        self.text_control_runtime
            .get(&self.arena[node].id)
            .map(|state| state.opaque_ranges.clone())
    }
    pub fn TextControlSetUserEdited(&mut self, node: usize, edited: bool) {
        let state = self
            .text_control_runtime
            .entry(self.arena[node].id)
            .or_default();
        if !edited {
            // HTMLFormControlElementWithState::ClearUserHasEditedTheField, h:54-57.
            state.force_user_valid = false;
        }
        state.user_interaction = if edited {
            state.user_interaction.max(1)
        } else {
            0
        };
    }
    pub fn TextControlDidDispatchChange(&mut self, node: usize) {
        if self
            .text_control_runtime
            .get(&self.arena[node].id)
            .is_some_and(|state| state.user_interaction == 1)
        {
            self.SetUserHasEditedTheFieldAndBlurred(node);
        }
    }
    /// ListedElement::UpdateWillValidateCache, listed_element.cc:396-435.
    /// Returns a changed eligibility value for the optional validation UI.
    pub fn UpdateControlWillValidate(&mut self, node: usize) -> Option<bool> {
        let eligible = self.ControlWillValidate(node);
        let state = self
            .text_control_runtime
            .entry(self.arena[node].id)
            .or_default();
        if state.will_validate == Some(eligible) {
            return None;
        }
        state.will_validate = Some(eligible);
        state.needs_validity_check = false;
        self.SetControlNeedsValidityCheck(node);
        Some(eligible)
    }
    pub fn ControlWillValidate(&self, node: usize) -> bool {
        let class = self.FormControlClass(node);
        // Source subclass overrides: button.cc:320-322, fieldset.h:68,
        // output.h:51. Output's IDL override is always false.
        let class_supports =
            match class {
                Some(DOMFormControlClass::Button) => self.arena[node]
                    .FindAttribute("type")
                    .is_none_or(|attribute| {
                        !matches!(
                            attribute.value.to_ascii_lowercase().as_str(),
                            "button" | "reset"
                        )
                    }),
                Some(DOMFormControlClass::Fieldset | DOMFormControlClass::Output) => false,
                Some(_) => true,
                None => self.ElementInternalsTargetIsFormAssociated(node),
            };
        let supported = class_supports
            && self.InputType(node).is_none_or(|kind| {
                !matches!(
                    kind,
                    DOMInputType::kButton
                        | DOMInputType::kReset
                        | DOMInputType::kHidden
                        | DOMInputType::kImage
                )
            });
        let readonly_prevents = self.arena[node].IsHTMLElement("textarea")
            || self
                .InputType(node)
                .is_some_and(DOMInputType::SupportsReadOnly);
        let mut parent = self.arena[node].parent;
        let mut in_datalist = false;
        while let Some(ancestor) = parent {
            if self.arena[ancestor].IsHTMLElement("datalist") {
                in_datalist = true;
                break;
            }
            parent = self.arena[ancestor].parent;
        }
        // ElementSpecificReadOnlyConstraintValidation is stable at this
        // revision; readOnly bars exactly the concrete supporting classes.
        supported
            && !in_datalist
            && !self.IsDisabledFormControl(node)
            && !(readonly_prevents && self.arena[node].FindAttribute("readonly").is_some())
    }
    pub fn PendingControlValidityNodeIds(&self) -> Vec<u64> {
        self.pending_control_validity.iter().copied().collect()
    }
    pub fn ControlCustomValidationMessageUTF16(&self, node: usize) -> Rc<[u16]> {
        self.text_control_runtime
            .get(&self.arena[node].id)
            .map(|state| state.custom_validation_message.clone())
            .unwrap_or_else(|| Rc::from([]))
    }
    /// ListedElement::SetCustomValidationMessage normalizes CRLF/CR to LF.
    pub fn SetControlCustomValidationMessageUTF16(&mut self, node: usize, value: Rc<[u16]>) {
        let mut normalized = Vec::with_capacity(value.len());
        let mut units = value.iter().copied().peekable();
        while let Some(unit) = units.next() {
            if unit == 0x0d {
                if units.peek() == Some(&0x0a) {
                    units.next();
                }
                normalized.push(0x0a);
            } else {
                normalized.push(unit);
            }
        }
        let value: Rc<[u16]> = normalized.into();
        self.text_control_runtime
            .entry(self.arena[node].id)
            .or_default()
            .custom_validation_message = value;
        self.SetControlNeedsValidityCheck(node);
    }
    pub fn CachedControlValidity(&self, node: usize) -> Option<bool> {
        self.text_control_runtime
            .get(&self.arena[node].id)
            .and_then(|state| {
                if state.needs_validity_check {
                    None
                } else {
                    Some(state.cached_validity)
                }
            })
    }
    pub fn CommitControlValidity(&mut self, node: usize, valid: bool) {
        self.pending_control_validity.remove(&self.arena[node].id);
        let state = self
            .text_control_runtime
            .entry(self.arena[node].id)
            .or_default();
        state.cached_validity = valid;
        state.needs_validity_check = false;
    }
    pub fn CachedControlWillValidate(&self, node: usize) -> Option<bool> {
        self.text_control_runtime
            .get(&self.arena[node].id)
            .and_then(|state| state.will_validate)
    }
    /// ListedElement::SetNeedsValidityCheck, listed_element.cc:654-665.
    /// Form/fieldset ownership and dirty gates remain on the canonical DOM.
    pub fn SetControlNeedsValidityCheck(&mut self, node: usize) {
        self.pending_control_validity.insert(self.arena[node].id);
        let state = self
            .text_control_runtime
            .entry(self.arena[node].id)
            .or_default();
        if state.needs_validity_check {
            return;
        }
        state.needs_validity_check = true;
        self.InvalidateValidityPseudoClasses(node);
        self.FormAssociationNeedsValidityCheck(node);
    }
    fn FormAssociationNeedsValidityCheck(&mut self, node: usize) {
        if let Some(form) = self.FormOwner(node) {
            self.InvalidateValidityPseudoClasses(form);
        }
        let mut parent = self.arena[node].parent;
        while let Some(ancestor) = parent {
            if self.arena[ancestor].IsHTMLElement("fieldset") {
                self.InvalidateValidityPseudoClasses(ancestor);
            }
            parent = self.arena[ancestor].parent;
        }
    }
    fn InvalidateValidityPseudoClasses(&mut self, node: usize) {
        use crate::style_state::ControlPseudoState as P;
        for pseudo in [P::Valid, P::Invalid, P::UserValid, P::UserInvalid] {
            self.ControlPseudoStateChanged(node, pseudo);
        }
    }
    pub fn InputHasBeenPasswordField(&self, node: usize) -> bool {
        self.text_control_runtime
            .get(&self.arena[node].id)
            .is_some_and(|state| state.has_been_password_field)
    }
    pub fn UpdateInputHasBeenPasswordField(&mut self, node: usize) {
        if self.InputType(node) == Some(DOMInputType::kPassword) {
            self.text_control_runtime
                .entry(self.arena[node].id)
                .or_default()
                .has_been_password_field = true;
        }
    }
    pub fn InputShouldRevealPassword(&self, node: usize) -> bool {
        self.text_control_runtime
            .get(&self.arena[node].id)
            .is_some_and(|state| state.should_reveal_password)
    }
    // Source: password_input_type.h:75; password_input_type.cc:127-150.
    pub fn InputShouldDrawCapsLockIndicator(&self, node: usize) -> bool {
        self.InputType(node) == Some(DOMInputType::kPassword)
            && self
                .text_control_runtime
                .get(&self.arena[node].id)
                .is_some_and(|state| state.should_draw_caps_lock_indicator)
    }
    pub fn SetInputShouldDrawCapsLockIndicator(&mut self, node: usize, draw: bool) {
        self.text_control_runtime
            .entry(self.arena[node].id)
            .or_default()
            .should_draw_caps_lock_indicator = draw;
    }
    pub fn SetInputShouldRevealPassword(&mut self, node: usize, reveal: bool) {
        self.text_control_runtime
            .entry(self.arena[node].id)
            .or_default()
            .should_reveal_password = reveal;
        self.InvalidateInputTypeStyle(node);
    }
    pub fn InputNeedsUpdateViewValue(&self, node: usize) -> bool {
        self.text_control_runtime
            .get(&self.arena[node].id)
            .is_none_or(|state| state.needs_update_view_value)
    }
    pub fn SetInputNeedsUpdateViewValue(&mut self, node: usize, update: bool) {
        self.text_control_runtime
            .entry(self.arena[node].id)
            .or_default()
            .needs_update_view_value = update;
    }
    pub fn InvalidateInputDefaultButtonStyle(&mut self, node: usize) {
        if self.FormOwner(node).is_some() {
            self.InvalidateDefaultPseudoCandidates();
        }
    }
    pub fn ReapplyInputSizeAttributeInvalidations(&mut self, node: usize) {
        let attributes = ["height", "width", "align"]
            .into_iter()
            .filter_map(|name| self.arena[node].FindAttribute(name).cloned())
            .collect::<Vec<_>>();
        // AttributeChangedWithInvalidations does not write the attribute or
        // produce a MutationObserver record when replaying the existing value.
        for attribute in attributes {
            self.InvalidateNodeStyle(node);
            self.RecordLayoutGeometryChange(node);
            self.attribute_lifecycle
                .Record(crate::attribute_lifecycle::AttributeChange {
                    node: self.arena[node].id,
                    old_value_utf16: self.arena[node].AttributeValueUTF16(&attribute.local_name),
                    new_value_utf16: self.arena[node].AttributeValueUTF16(&attribute.local_name),
                    name: attribute.local_name,
                    namespace: attribute.namespace_uri,
                    old_value: Some(attribute.value.clone()),
                    new_value: Some(attribute.value),
                    reason: crate::attribute_lifecycle::AttributeModificationReason::Directly,
                    was_connected: self.IsConnectedToDocument(node),
                });
        }
    }
    pub fn InputValueAttributeUTF16(&self, node: usize) -> Option<Rc<[u16]>> {
        self.input_value_attribute_utf16
            .get(&self.arena[node].id)
            .cloned()
            .or_else(|| self.arena[node].AttributeValueUTF16("value"))
    }
    pub fn SetInputValueAttributeUTF16(&mut self, node: usize, value: Rc<[u16]>) {
        self.SetAttribute(
            node,
            DOMAttribute {
                local_name: "value".into(),
                value: String::from_utf16_lossy(&value),
                ..Default::default()
            },
        );
        self.input_value_attribute_utf16
            .insert(self.arena[node].id, value);
    }
    pub fn InputNonAttributeValueUTF16(&self, node: usize) -> Rc<[u16]> {
        self.InputNonAttributeValueOwner(node).Owner()
    }
    pub fn InputNonAttributeValueOwner(&self, node: usize) -> foundation::NullableUTF16String {
        self.control_value_state
            .get(&self.arena[node].id)
            .cloned()
            .unwrap_or_else(foundation::NullableUTF16String::Null)
    }
    pub fn ClearStoredControlValue(&mut self, node: usize) {
        self.control_value_state.remove(&self.arena[node].id);
        self.SetControlValueDirty(node, false);
    }
    /// Exact code-unit value for bindings, editing, maxlength and selection.
    pub fn ControlValueUTF16(&self, node: usize) -> Rc<[u16]> {
        if matches!(self.arena[node].class, DOMNodeClass::HTMLOutputElement(_)) {
            return self.OutputValueUTF16(node);
        }
        if self.InputType(node) == Some(DOMInputType::kFile) {
            return self
                .ControlValue(node)
                .encode_utf16()
                .collect::<Vec<_>>()
                .into();
        }
        if let Some(value) = self.control_value_state.get(&self.arena[node].id) {
            return value.Owner();
        }
        if self.arena[node].IsHTMLElement("input") {
            if let Some(value) = self.InputValueAttributeUTF16(node) {
                return value;
            }
        }
        if self.arena[node].IsHTMLElement("textarea") {
            return self.DescendantTextUTF16(node);
        }
        self.ControlValue(node)
            .encode_utf16()
            .collect::<Vec<_>>()
            .into()
    }
    pub fn ControlValueRevision(&self, node: usize) -> u64 {
        self.text_control_runtime
            .get(&self.arena[node].id)
            .map_or(0, |state| state.value_revision)
    }
    fn AdvanceControlValueRevision(&mut self, node: usize) {
        let state = self
            .text_control_runtime
            .entry(self.arena[node].id)
            .or_default();
        state.value_revision = state.value_revision.wrapping_add(1);
    }
    pub fn HasDirtyControlValue(&self, node: usize) -> bool {
        self.control_value_dirty.contains(&self.arena[node].id)
    }
    pub fn SetControlValueDirty(&mut self, node: usize, dirty: bool) {
        if dirty {
            self.control_value_dirty.insert(self.arena[node].id);
        } else {
            self.control_value_dirty.remove(&self.arena[node].id);
        }
    }
    pub fn ControlValue(&self, node: usize) -> String {
        if matches!(self.arena[node].class, DOMNodeClass::HTMLOutputElement(_)) {
            return self.OutputValue(node);
        }
        // Source: FileInputType::ValueInFilenameValueMode. Read the same
        // canonical FileList owner used by the InputType and bindings.
        if self.InputType(node) == Some(DOMInputType::kFile) {
            return self
                .InputFiles(node)
                .and_then(|files| files.item(0))
                .map(|file| format!("C:\\fakepath\\{}", file.name()))
                .unwrap_or_default();
        }
        if let Some(value) = self.control_value_state.get(&self.arena[node].id) {
            return String::from_utf16_lossy(value);
        }
        if self.arena[node].IsElement("input", DOMNamespace::kHTML) {
            // cpp: HTMLInputElement::Value, InputType::GetValueMode and
            // BaseCheckableInputType::GetValueMode. The default-on mode
            // distinguishes a missing value from an explicitly empty value.
            let value = self.arena[node]
                .FindAttribute("value")
                .map(|value| value.value.clone())
                .unwrap_or_else(|| {
                    if matches!(
                        self.InputType(node),
                        Some(DOMInputType::kCheckbox | DOMInputType::kRadio)
                    ) {
                        "on".to_owned()
                    } else {
                        String::new()
                    }
                });
            return if self.InputType(node) == Some(DOMInputType::kColor) {
                Self::SanitizeColorInputValue(&value)
            } else {
                value
            };
        }
        if self.arena[node].IsElement("textarea", DOMNamespace::kHTML) {
            let mut value = String::new();
            self.AppendDescendantText(node, &mut value);
            return value;
        }
        if self.arena[node].IsElement("select", DOMNamespace::kHTML) {
            let mut first = None;
            let mut selected = None;
            self.VisitOptions(node, &mut first, &mut selected);
            let Some(option) = selected.or(first) else {
                return String::new();
            };
            return self.OptionValue(option);
        }
        invalid_argument("control value requires an input, textarea, or select");
    }

    /// Source: HTMLTextFormControlElement::maxLength and BaseTextInputType::TooLong.
    /// Keep dirty/user-edit state on the canonical control, not in an IME adapter.
    pub fn TextControlMinLength(&self, node: usize) -> i32 {
        self.arena[node]
            .FindAttribute("minlength")
            .and_then(|a| Self::ParseHTMLNonNegativeInteger(&a.value))
            .filter(|&v| v <= i32::MAX as u32)
            .map_or(-1, |v| v as i32)
    }
    pub fn TextControlMaxLength(&self, node: usize) -> i32 {
        self.arena[node]
            .FindAttribute("maxlength")
            .and_then(|attribute| Self::ParseHTMLNonNegativeInteger(&attribute.value))
            .filter(|value| *value <= i32::MAX as u32)
            .map_or(-1, |value| value as i32)
    }
    pub fn TextControlLastChangeWasUserEdit(&self, node: usize) -> bool {
        self.text_control_last_change_was_user_edit
            .contains(&self.arena[node].id)
    }
    pub fn TextControlTooLong(&self, node: usize) -> bool {
        let textarea = self.arena[node].IsHTMLElement("textarea");
        let text_input = matches!(
            self.InputType(node),
            Some(
                DOMInputType::kText
                    | DOMInputType::kSearch
                    | DOMInputType::kTelephone
                    | DOMInputType::kURL
                    | DOMInputType::kEmail
                    | DOMInputType::kPassword
            )
        );
        if (!textarea && !text_input)
            || !self.TextControlLastChangeWasUserEdit(node)
            || !self.HasDirtyControlValue(node)
        {
            return false;
        }
        // TextArea::TooLong also observes willValidate(). Text input's source
        // BaseTextInputType predicate has no willValidate guard.
        if textarea {
            if self.IsDisabledFormControl(node)
                || self.arena[node].FindAttribute("readonly").is_some()
            {
                return false;
            }
            let mut ancestor = self.arena[node].parent;
            while let Some(node) = ancestor {
                if self.arena[node].IsHTMLElement("datalist") {
                    return false;
                }
                ancestor = self.arena[node].parent;
            }
        }
        let max = self.TextControlMaxLength(node);
        max >= 0 && self.ControlValueUTF16(node).len() > max as usize
    }

    // cpp: dom/document.cc:328-333
    /// HTMLInputElement::SetNonAttributeValue's storage step, before its
    /// InputType view update. The state stays on the canonical element owner.
    pub fn SetStoredControlValueUTF16(&mut self, node: usize, value: &[u16], user_edit: bool) {
        self.SetStoredControlValueOwner(
            node,
            foundation::NullableUTF16String::from(value),
            user_edit,
        );
    }
    pub fn SetStoredControlValueOwner(
        &mut self,
        node: usize,
        value: foundation::NullableUTF16String,
        user_edit: bool,
    ) {
        self.AdvanceControlValueRevision(node);
        self.control_value_state.insert(self.arena[node].id, value);
        self.SetControlValueDirty(node, true);
        self.SetTextControlLastChangeWasUserEdit(node, user_edit);
        self.SetControlNeedsValidityCheck(node);
    }
    pub fn SetTextControlLastChangeWasUserEdit(&mut self, node: usize, user_edit: bool) {
        if user_edit {
            self.text_control_last_change_was_user_edit
                .insert(self.arena[node].id);
        } else {
            self.text_control_last_change_was_user_edit
                .remove(&self.arena[node].id);
        }
    }
    pub fn UpdateTextControlPlaceholder(&mut self, node: usize) {
        self.SyncTextControlPlaceholderElement(node);
    }
    pub fn UpdateTextControlView(&mut self, node: usize) {
        self.EnsureTextControlUAShadowTree(node);
        self.SyncTextControlPlaceholderElement(node);
        self.SyncSearchCancelButtonVisibility(node);
        self.RecordLayoutGeometryChange(node);
        self.RecordLayoutInputChange(node);
    }
    pub fn SetControlValue(&mut self, node: usize, value: String) {
        self.SetControlValueUTF16(node, &value.encode_utf16().collect::<Vec<_>>());
    }
    pub fn SetControlValueUTF16(&mut self, node: usize, units: &[u16]) {
        self.AdvanceControlValueRevision(node);
        let sanitized = self.SanitizeInputValue(node, Rc::from(units));
        let units = sanitized.as_ref();
        let value = String::from_utf16_lossy(units);
        if let (Some(kind), Some(sanitizer)) =
            (self.InputType(node), self.input_value_sanitizer.as_ref())
        {
            if matches!(
                sanitizer.ValueMode(kind),
                crate::input_value::DOMInputValueMode::Default
                    | crate::input_value::DOMInputValueMode::DefaultOn
            ) {
                self.SetInputValueAttributeUTF16(node, sanitized);
                return;
            }
        }
        if matches!(self.arena[node].class, DOMNodeClass::HTMLOutputElement(_)) {
            self.SetOutputValueUTF16(node, units);
            return;
        }
        if self.InputType(node).is_some()
            && self.input_value_sanitizer.is_none()
            && self.attribute_lifecycle.HasOwner()
        {
            self.attribute_lifecycle
                .RecordInputValueAssigned(self.arena[node].id, Rc::from(units));
            return;
        }
        self.text_control_last_change_was_user_edit
            .remove(&self.arena[node].id);
        if !self.arena[node].IsElement("input", DOMNamespace::kHTML)
            && !self.arena[node].IsElement("textarea", DOMNamespace::kHTML)
            && !self.arena[node].IsElement("select", DOMNamespace::kHTML)
        {
            invalid_argument("control value requires an input, textarea, or select");
        }
        if self.InputType(node) == Some(DOMInputType::kFile) {
            // cpp: FileInputType::CanSetValue. Script may clear a file input,
            // but cannot manufacture a filename without a FileList/chooser.
            if !value.is_empty() {
                invalid_argument("a file input value can only be cleared");
            }
            if self.attribute_lifecycle.HasOwner() {
                self.attribute_lifecycle
                    .RecordInputFilesCleared(self.arena[node].id);
                return;
            }
            if let Some(files) = self.InputFiles(node) {
                files.clear();
            }
            self.control_value_state.remove(&self.arena[node].id);
            self.SyncFileUAShadowTree(node);
            self.RecordLayoutGeometryChange(node);
            self.InvalidateNodeStyle(node);
            self.RecordLayoutInputChange(node);
            return;
        }
        if self.arena[node].IsElement("select", DOMNamespace::kHTML) {
            // SelectOptionByElement clears suggested_option_ before applying
            // either script selection or an autofill selection.
            self.SetSelectSuggestedOption(node, None);
            self.RecordLayoutGeometryChange(node);
            let option = self
                .SelectOptionNodeIds(node)
                .into_iter()
                .filter_map(|id| self.FindNodeById(id))
                .find(|&option| self.OptionValue(option) == value);
            self.SelectOptionInternal(node, option, true, true, DOMAutofillState::kNotFilled);
            return;
        }
        // cpp: HTMLInputElement::SetValue, HTMLTextAreaElement::SetValue and
        // HTMLSelectElement::SetValue default their WebAutofillState argument
        // to kNotFilled. Explicit autofill callers use SetAutofillValue below.
        let autofill_changed = self.AutofillState(node) != DOMAutofillState::kNotFilled;
        self.SetAutofillState(node, DOMAutofillState::kNotFilled);
        let value: Rc<[u16]> = if self.InputType(node) == Some(DOMInputType::kColor) {
            Self::SanitizeColorInputValue(&value)
                .encode_utf16()
                .collect::<Vec<_>>()
                .into()
        } else {
            Rc::from(units)
        };
        // HTMLInputElement/HTMLTextAreaElement clear the base-class suggested
        // value before their early-return-on-equal-value check.
        let placeholder_was_visible = self.TextControlPlaceholderIsVisible(node);
        let had_suggested_value = if self.arena[node].IsElement("input", DOMNamespace::kHTML)
            || self.arena[node].IsElement("textarea", DOMNamespace::kHTML)
        {
            self.suggested_value_state
                .remove(&self.arena[node].id)
                .is_some()
        } else {
            false
        };
        let previous = self.ControlValueUTF16(node);
        let changed = previous != value;
        self.control_value_state.insert(
            self.arena[node].id,
            foundation::NullableUTF16String::FromOwner(value.clone()),
        );
        self.SetControlValueDirty(node, true);
        self.SetControlNeedsValidityCheck(node);
        self.NotifyInputValueCommitted(node, &value);
        self.SetTextControlInnerEditorValueUTF16(node, &value);
        // cpp: HTMLTextAreaElement::SetValueCommon (603-643). An equal value
        // preserves source geometry; actual editor/suggestion changes revoke
        // it here, at the DOM owner, before Document projects lifecycle effects.
        if changed || had_suggested_value || autofill_changed {
            self.RecordLayoutGeometryChange(node);
        }
        if !changed {
            if had_suggested_value {
                self.SyncTextControlPlaceholderElement(node);
                if let Some(editor) = self.TextControlInnerEditorElement(node) {
                    self.InvalidateNodeStyle(editor);
                }
                self.InvalidateNodeStyle(node);
                self.RecordLayoutInputChange(node);
            }
            return;
        }
        if self.arena[node].IsElement("input", DOMNamespace::kHTML)
            || self.arena[node].IsElement("textarea", DOMNamespace::kHTML)
        {
            self.SyncTextControlPlaceholderElement(node);
            self.SyncSearchCancelButtonVisibility(node);
            self.SyncColorSwatchAppearance(node);
            if had_suggested_value {
                if let Some(editor) = self.TextControlInnerEditorElement(node) {
                    self.InvalidateNodeStyle(editor);
                }
            }
        }
        self.UpdateDirectionalityForMutation(node);
        // cpp: TextControlElement::UpdatePlaceholderVisibility (282-316).
        // Only an actual placeholder visibility transition changes this pseudo
        // state. An empty/nonempty edit without a placeholder is not a selector
        // mutation; every edit still updates the retained layout below.
        if placeholder_was_visible != self.TextControlPlaceholderIsVisible(node) {
            self.RecordStyleChange(crate::style_state::StyleChange::PlaceholderShown(node));
        }
        self.RecordLayoutInputChange(node);
    }

    fn IsAutofillFormControl(&self, node: usize) -> bool {
        self.arena[node].Namespace() == DOMNamespace::kHTML
            && matches!(self.arena[node].Name(), "input" | "textarea" | "select")
    }

    pub fn AutofillState(&self, node: usize) -> DOMAutofillState {
        if !self.IsAutofillFormControl(node) {
            return DOMAutofillState::kNotFilled;
        }
        self.control_autofill_state
            .get(&self.arena[node].id)
            .copied()
            .unwrap_or_default()
    }

    // cpp: html_form_control_element.cc:226-235.
    pub fn SetAutofillState(&mut self, node: usize, state: DOMAutofillState) {
        if !self.IsAutofillFormControl(node) {
            invalid_argument("autofill state requires an input, textarea, or select");
        }
        if self.AutofillState(node) == state {
            return;
        }
        let id = self.arena[node].id;
        if state == DOMAutofillState::kNotFilled {
            self.control_autofill_state.remove(&id);
        } else {
            self.control_autofill_state.insert(id, state);
        }
        self.InvalidateNodeStyle(node);
        self.RecordLayoutInputChange(node);
    }

    // cpp: WebFormControlElement::SetAutofillValue. Value and state are one
    // typed producer operation so ordinary SetControlValue continues to clear
    // stale autofill state.
    pub fn SetAutofillValue(&mut self, node: usize, value: String, state: DOMAutofillState) {
        if self.arena[node].IsElement("select", DOMNamespace::kHTML) {
            self.SetSelectSuggestedOption(node, None);
            let option = self
                .SelectOptionNodeIds(node)
                .into_iter()
                .filter_map(|id| self.FindNodeById(id))
                .find(|&option| self.OptionValue(option) == value);
            self.SelectOptionInternal(node, option, true, true, state);
            return;
        }
        self.SetControlValue(node, value);
        self.SetAutofillState(node, state);
    }

    // cpp: dom/document.cc:335-349
    pub fn FindNodeById(&self, id: u64) -> Option<usize> {
        if id == 0 || id > self.arena.len() as u64 {
            return None;
        }
        let index = id as usize - 1;
        (self.arena[index].id == id).then_some(index)
    }

    // cpp: html_map_element.cc:61-76; html_area_element.cc:193-198.
    // The map owns the association, and the first matching HTML image in
    // document order is authoritative when several images name one map.
    /// Source HTMLAreaElement::ImageElement -> HTMLMapElement::ImageElement.
    pub fn ImageMapImageForAreaId(&self, area_id: u64) -> Option<u64> {
        let area = self.FindNodeById(area_id)?;
        self.ImageElementForArea(area)
            .map(|image| self.arena[image].id)
    }

    fn ImageMapPaintOwnersForElementMutation(&self, element: usize) -> Vec<Option<u64>> {
        if self.arena[element].IsElement("area", DOMNamespace::kHTML) {
            return vec![self.ImageMapImageForAreaId(self.arena[element].Id())];
        }
        if self.arena[element].IsElement("map", DOMNamespace::kHTML)
            || self.arena[element].IsElement("img", DOMNamespace::kHTML)
        {
            // Changing a map's name/id or an image's usemap can promote a
            // different image to HTMLMapElement::ImageElement(). Capture all
            // current first-image owners on both sides of the mutation.
            return self.RegisteredImageMapPaintOwners();
        }
        Vec::new()
    }

    pub fn InvalidateImageMapPaintOwnerForAreaId(&mut self, area_id: u64) {
        let owner = self.ImageMapImageForAreaId(area_id);
        self.RecordImageMapPaintOwnerChanges([owner]);
    }

    fn RecordImageMapPaintOwnerChanges(&mut self, owners: impl IntoIterator<Item = Option<u64>>) {
        let mut changed = false;
        for owner in owners.into_iter().flatten() {
            changed |= self.style_state.paint_style_nodes.insert(owner);
        }
        if changed {
            self.style_state
                .impact
                .Merge(crate::style_state::StyleUpdateImpact {
                    paint: true,
                    ..Default::default()
                });
        }
    }
    pub fn TakeAllChildren(&mut self, source: usize, destination: usize) {
        let children = self.arena[source].children.clone();
        for child in children {
            self.AppendChild(destination, child);
        }
    }

    pub fn NeedsStyleSheetParsing(&self, node_id: u64) -> bool {
        !self.style_state.parsed_style_elements.contains(&node_id)
            || self.style_state.dirty_style_elements.contains(&node_id)
    }

    // cpp: dom/document.h:134-141
    pub fn AppendStyleSheet(&mut self, sheet: CSSStyleSheet) {
        if sheet.owner_node_id == 0 && sheet.Identity() != 0 {
            // This API explicitly attaches an ownerless non-constructed sheet
            // to the Document scope. Constructed sheets use the separate
            // registration/adoption relation above.
            self.ownerless_style_sheet_scopes
                .insert(sheet.Identity(), self.root);
        }
        if self
            .FindNodeById(sheet.owner_node_id)
            .is_some_and(|node| self.arena[node].IsStyleElement())
        {
            self.style_state
                .parsed_style_elements
                .insert(sheet.owner_node_id);
            self.style_state
                .dirty_style_elements
                .remove(&sheet.owner_node_id);
        }
        let root_handle = DOMOwnerHandle {
            arena_id: self.arena_id,
            node_id: self.arena[self.root].id,
        };
        let detached_owner = self.FindNodeById(sheet.owner_node_id).and_then(|owner| {
            let document = self.arena[owner]
                .owner_document_handle
                .expect("a stylesheet owner must have an owner document");
            (document != root_handle).then_some(document)
        });
        let affects_active_sheets = detached_owner.is_none();
        let state = &mut self.style_state;
        self.cssom.ApplyMutation(
            CSSOMMutation::AddStyleSheet {
                sheet,
                detached_owner,
            },
            |event| match event {
                CSSOMEvent::StyleSheetChanged if affects_active_sheets => {
                    state.sheet_revision += 1;
                    // StyleEngine owns the active-sheet diff. It invalidates
                    // against both the old and new RuleSets, so replacing or
                    // removing a sheet does not itself imply a document-wide
                    // recalc. This matches Blink's ApplyRuleSetChanges path.
                }
                CSSOMEvent::StyleSheetChanged => {}
            },
        );
    }

    // cpp: dom/document.h:142-147
    // Stable arena adapter for source StyleSheets(DOMNode*). This names the
    // receiver itself, including a detached Document; ownerDocument is distinct.
    pub fn StyleSheetsForNode(&self, node: usize) -> &[CSSStyleSheet] {
        self.StyleSheets(Some(DOMOwnerHandle {
            arena_id: self.arena_id,
            node_id: self.arena[node].id,
        }))
    }

    fn StyleSheetConnectivityChanged(&mut self, subtree: usize) {
        // cpp: StyleEngine::{Add,Remove}StyleSheetCandidateNode. Connectivity
        // changes notify through candidates in the affected shadow-including
        // subtree, rather than searching every sheet and its owner ancestry.
        let mut node = subtree;
        let mut pending = Vec::new();
        loop {
            if self.cssom.HasStyleSheetForOwner(self.arena[node].id)
                || self
                    .TreeScopeHandle(node)
                    .is_some_and(|scope| self.cssom.HasAdoptedStyleSheets(&scope))
            {
                self.style_state.sheet_revision += 1;
                // StyleEngine compares the retained active-sheet sequence,
                // including inline/adopted sheets carried by nested hosts.
                return;
            }
            if let Some(root) = self.ShadowRoot(node) {
                pending.push(root);
            }
            pending.extend_from_slice(&self.arena[node].children);
            let Some(next) = pending.pop() else {
                return;
            };
            node = next;
        }
    }
    pub fn ActiveStyleSheets(&self) -> impl Iterator<Item = &CSSStyleSheet> {
        self.ActiveStyleSheetsForTreeScope(self.root)
    }

    // cpp: StyleEngine::SetPreferredStylesheetSetNameIfNotSet and
    // CSSStyleSheet::CanBeActivated. The first persistent titled sheet owns
    // the initial preferred set for the document.
    pub fn PreferredStyleSheetSetName(&self) -> &str {
        self.cssom
            .GetStyleSheets(None)
            .iter()
            .find(|sheet| !sheet.disabled && !sheet.alternate && !sheet.title.is_empty())
            .map_or("", |sheet| sheet.title.as_str())
    }

    pub fn StyleSheetCanBeActivated(&self, sheet: &CSSStyleSheet) -> bool {
        let current = self.PreferredStyleSheetSetName();
        let preferred =
            if current.is_empty() && !sheet.disabled && !sheet.alternate && !sheet.title.is_empty()
            {
                sheet.title.as_str()
            } else {
                current
            };
        sheet.CanBeActivated(preferred)
    }

    pub fn ActiveStyleSheetsForTreeScope(
        &self,
        scope: usize,
    ) -> impl Iterator<Item = &CSSStyleSheet> {
        let adopted = self.AdoptedStyleSheetsForTreeScope(scope);
        let preferred_set_name = self.PreferredStyleSheetSetName();
        let mut sheets = self
            .cssom
            .GetStyleSheets(None)
            .iter()
            .filter(move |sheet| {
                sheet.CanBeActivated(preferred_set_name)
                    && self.StyleSheetTreeScope(sheet) == Some(scope)
            })
            .collect::<Vec<_>>();
        // cpp: StyleSheetCollection::PrepareUpdateActiveStyleSheets. Candidate
        // nodes are a TreeOrderedList, not CSSOM's registration/completion order.
        // Derive each candidate's position from the DOM owner so subtree moves
        // and asynchronously completed link sheets use the same source order.
        // Ordinary Parent links deliberately do not cross a ShadowRoot's host.
        sheets.sort_by_cached_key(|sheet| {
            if sheet.owner_node_id == 0 {
                // AppendStyleSheet explicitly attaches ownerless sheets to the
                // Document. They occupy the injected-author prefix, preserving
                // their append order; constructed sheets are appended below.
                return None;
            }
            let mut node = self
                .FindNodeById(sheet.owner_node_id)
                .expect("an active stylesheet must have a retained owner");
            let mut path = Vec::new();
            while node != scope {
                let parent = self.arena[node]
                    .parent
                    .expect("an active stylesheet owner must belong to its tree scope");
                path.push(
                    self.arena[parent]
                        .children
                        .iter()
                        .position(|&child| child == node)
                        .expect("DOM parent/child links are inconsistent"),
                );
                node = parent;
            }
            path.reverse();
            Some(path)
        });
        sheets.into_iter().chain(
            adopted
                .into_iter()
                .filter(move |sheet| sheet.CanBeActivated(preferred_set_name)),
        )
    }

    pub fn ActiveStyleSheetsAllScopes(&self) -> impl Iterator<Item = &CSSStyleSheet> {
        let preferred_set_name = self.PreferredStyleSheetSetName();
        self.cssom.GetStyleSheets(None).iter().filter(move |sheet| {
            sheet.CanBeActivated(preferred_set_name) && self.StyleSheetTreeScope(sheet).is_some()
        })
    }

    /// Scope-preserving active sheet view for StyleEngine. A constructed sheet
    /// may appear for multiple scopes, and duplicate entries within one scope
    /// retain the ObservableArray's multiplicity.
    pub fn ActiveStyleSheetsWithTreeScopes(&self) -> Vec<(usize, &CSSStyleSheet)> {
        self.ActiveTreeScopes()
            .into_iter()
            .flat_map(|scope| {
                self.ActiveStyleSheetsForTreeScope(scope)
                    .map(move |sheet| (scope, sheet))
            })
            .collect()
    }

    pub fn StyleSheets(&self, owner_document: Option<DOMOwnerHandle>) -> &[CSSStyleSheet] {
        let Some(owner_document) = owner_document else {
            return self.cssom.GetStyleSheets(None);
        };
        let root_handle = DOMOwnerHandle {
            arena_id: self.arena_id,
            node_id: self.arena[self.root].id,
        };
        if owner_document == root_handle {
            return self.cssom.GetStyleSheets(None);
        }
        self.cssom.GetStyleSheets(Some(&owner_document))
    }

    // cpp: dom/document.h:149-156
    pub fn SetAnimationStyle(&mut self, node: u64, effect: u64, declarations: Vec<CSSDeclaration>) {
        Self::SetSampledStyle(&mut self.animated_styles, node, effect, declarations);
        if let Some(index) = self.FindNodeById(node) {
            self.RecordStyleChange(crate::style_state::StyleChange::Animation(index));
        }
    }
    pub fn SetTransitionStyle(
        &mut self,
        node: u64,
        effect: u64,
        declarations: Vec<CSSDeclaration>,
    ) {
        Self::SetSampledStyle(&mut self.transitioned_styles, node, effect, declarations);
        if let Some(index) = self.FindNodeById(node) {
            self.RecordStyleChange(crate::style_state::StyleChange::Animation(index));
        }
    }
    fn SetSampledStyle(
        styles: &mut HashMap<u64, BTreeMap<u64, Vec<CSSDeclaration>>>,
        node: u64,
        effect: u64,
        declarations: Vec<CSSDeclaration>,
    ) {
        let effects = styles.entry(node).or_default();
        if declarations.is_empty() {
            effects.remove(&effect);
        } else {
            effects.insert(effect, declarations);
        }
        if effects.is_empty() {
            styles.remove(&node);
        }
    }
    pub fn AnimationStyles(&self, node: u64) -> &BTreeMap<u64, Vec<CSSDeclaration>> {
        static EMPTY: OnceLock<BTreeMap<u64, Vec<CSSDeclaration>>> = OnceLock::new();
        self.animated_styles
            .get(&node)
            .unwrap_or_else(|| EMPTY.get_or_init(BTreeMap::new))
    }
    pub fn TransitionStyles(&self, node: u64) -> &BTreeMap<u64, Vec<CSSDeclaration>> {
        static EMPTY: OnceLock<BTreeMap<u64, Vec<CSSDeclaration>>> = OnceLock::new();
        self.transitioned_styles
            .get(&node)
            .unwrap_or_else(|| EMPTY.get_or_init(BTreeMap::new))
    }

    fn SyncReplacedImageConsumer(&mut self, node: usize, old_source: Option<&str>) {
        if !self.arena[node].IsHTMLElement("img") {
            return;
        }
        let new_source = self.CurrentImageSource(node).map(str::to_owned);
        if old_source == new_source.as_deref() {
            return;
        }
        if let Some(old_source) = old_source {
            if let Some(consumers) = self.replaced_image_consumers.get_mut(old_source) {
                consumers.remove(&node);
                if consumers.is_empty() {
                    self.replaced_image_consumers.remove(old_source);
                }
            }
        }
        if let Some(new_source) = new_source.filter(|_| self.IsConnectedToDocument(node)) {
            self.replaced_image_consumers
                .entry(new_source)
                .or_default()
                .insert(node);
        }
    }

    // cpp: core/layout/layout_image.cc:67-73. LayoutImageResource::Shutdown
    // removes the resource observer when the layout image detaches. Keep the
    // same exact client ownership at the DOM/image-loader boundary, and put it
    // back when an existing subtree is reattached.
    fn SetReplacedImageConsumerConnected(&mut self, node: usize, connected: bool) {
        if !self.arena[node].IsHTMLElement("img") {
            return;
        }
        let Some(source) = self.CurrentImageSource(node).map(str::to_owned) else {
            return;
        };
        if connected {
            self.replaced_image_consumers
                .entry(source)
                .or_default()
                .insert(node);
        } else if let Some(consumers) = self.replaced_image_consumers.get_mut(&source) {
            consumers.remove(&node);
            if consumers.is_empty() {
                self.replaced_image_consumers.remove(&source);
            }
        }
    }

    fn NotifySubtreeConnected(&mut self, root: usize, connected: bool) {
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            if self.FormControlClass(node).is_some() || self.IsFormAssociatedCustomElement(node) {
                self.FormAssociationNeedsValidityCheck(node);
                self.SetControlNeedsValidityCheck(node);
            }
            if self.arena[node].IsHTMLElement("body") || self.arena[node].IsHTMLElement("frameset")
            {
                self.SyncRenderBlockingBodyPresence();
                if connected && self.arena[node].IsHTMLElement("frameset") && self.HasDOMWindow() {
                    self.WillInsertBody();
                }
            }
            self.RecordFontPreload(
                node,
                if connected {
                    crate::font_preload::FontPreloadReason::Attribute
                } else {
                    crate::font_preload::FontPreloadReason::Removed
                },
            );
            if connected {
                if self.arena[node].IsHTMLElement("link") && !self.IsInShadowTree(node) {
                    self.QueueExpectRenderBlockingLink(node);
                }
                self.ProcessAnchorNameRenderBlocking(node);
            } else {
                self.RemoveExpectRenderBlockingLink(node);
            }
            self.SetReplacedImageConsumerConnected(node, connected);
            if connected {
                self.DisplayLockElementConnected(node);
            } else {
                self.DisplayLockElementDisconnected(node);
            }
            if let Some(shadow_root) = self.ShadowRoot(node) {
                pending.push(shadow_root);
            }
            pending.extend_from_slice(&self.arena[node].children);
        }
    }

    fn InvalidateImageResourceConsumers(&mut self, source: &str) {
        // cpp: core/layout/layout_image.cc:139-153. Resource notification is
        // admitted only for the retained LayoutImage which still observes the
        // exact completing resource. CSS decoration consumers take their
        // independent immutable StyleImage refresh path below.
        // Completion changes resource state and intrinsic geometry, not
        // selector matching across the entire document. Immutable style-image
        // snapshots must be refreshed only for their actual consumers;
        // replaced-image metadata is synchronized by the layout tree builder.
        let style_consumers = self
            .style_image_consumers
            .get(source)
            .map(|consumers| consumers.iter().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        for index in style_consumers {
            self.RecordStyleChange(crate::style_state::StyleChange::Resource(index));
            // Blink's StyleFetchedImage retains a mutable
            // ImageResourceContent, so the existing ComputedStyle observes
            // completion in place. Rechrom's cross-package style snapshot is
            // immutable: keeping it here leaves the unpublished resource id
            // installed even though Style recorded the consumer as dirty.
            // Drop only that consumer's snapshot; the scheduled style update
            // rebuilds it from the same declarations and re-registers its
            // image dependencies with the completed resource binding.
            self.InvalidateResolvedStyleSnapshot(index);
        }
        // ImageResourceContent notifies only the LayoutObjects which observe
        // that resource. Carry the same ownership fact across the DOM ->
        // Layout boundary: a loaded <img> refreshes its existing Node input,
        // including natural size, candidate density and paint resource id.
        // It does not rebuild every LayoutObject in the document.
        let replaced_consumers = self
            .replaced_image_consumers
            .get(source)
            .map(|consumers| consumers.iter().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        // cpp: core/layout/layout_image.cc:175-183. This notification is the
        // translated LayoutImage::ImageChanged admission point: count only if
        // a still-connected LayoutImage observes this exact resource and the
        // completing response carried Content-DPR.
        if !replaced_consumers.is_empty()
            && self.ImageResourceFor(source).is_some_and(|resource| {
                self.ImageResourceContentDPR(resource.id).is_some()
                    && replaced_consumers.iter().any(|&index| {
                        self.IsConnectedToDocument(index)
                            && self.CurrentImageSource(index) == Some(source)
                    })
            })
        {
            self.CountUse(crate::use_counter::WebFeature::kClientHintsContentDPR);
        }
        for index in replaced_consumers {
            if self.IsConnectedToDocument(index)
                && self.CurrentImageSource(index) == Some(source)
                && self.RecordLayoutGeometryChange(index)
            {
                self.style_state
                    .geometry_input_nodes
                    .insert(self.arena[index].id);
            }
        }
    }
    // cpp: dom/document.cc:366-378
    pub fn EnsureImageResourceContent(&mut self, source: &str) -> Rc<DOMImageResourceContent> {
        if let Some(content) = self.ImageResourceContentFor(source) {
            return content;
        }
        let content = DOMImageResourceContent::Pending();
        self.image_resources.insert(
            source.to_owned(),
            ImageResourceState::Pending(content.clone()),
        );
        content
    }
    pub fn ResetFailedImageResource(&mut self, source: &str) {
        if matches!(
            self.image_resources.get(source),
            Some(ImageResourceState::Failed(_))
        ) {
            self.image_resources.remove(source);
        }
    }
    pub fn SetImageResource(&mut self, source: String, metadata: ImageResourceMetadata) {
        self.SetImageResourceWithContent(source, metadata, None);
    }
    pub fn SetImageResourceWithContent(
        &mut self,
        source: String,
        metadata: ImageResourceMetadata,
        content: Option<Rc<DOMImageResourceContent>>,
    ) {
        let content = content.unwrap_or_else(|| match self.image_resources.get(&source) {
            Some(ImageResourceState::Pending(content)) => content.clone(),
            _ => DOMImageResourceContent::Pending(),
        });
        self.image_resources.insert(
            source.clone(),
            ImageResourceState::Loaded(metadata, content.clone()),
        );
        content.Complete(Some(metadata), false);
        self.InvalidateImageLoaderObservers(&content);
        self.InvalidateImageResourceConsumers(&source);
    }
    pub fn SetImageResourceFailed(&mut self, source: String) -> bool {
        if matches!(
            self.image_resources.get(&source),
            Some(ImageResourceState::Loaded(..) | ImageResourceState::Failed(_))
        ) {
            return false;
        }
        let content = self.EnsureImageResourceContent(&source);
        self.image_resources
            .insert(source.clone(), ImageResourceState::Failed(content.clone()));
        content.Complete(None, false);
        self.InvalidateImageLoaderObservers(&content);
        self.InvalidateImageResourceConsumers(&source);
        true
    }
    fn InvalidateImageLoaderObservers(&mut self, content: &Rc<DOMImageResourceContent>) {
        for loader in content.Observers() {
            if let Some(node) = self.FindNodeById(loader.Element().node_id) {
                if self.RecordLayoutGeometryChange(node) {
                    self.style_state
                        .geometry_input_nodes
                        .insert(self.arena[node].id);
                }
            }
        }
    }
    pub fn ImageResourceContentFor(&self, source: &str) -> Option<Rc<DOMImageResourceContent>> {
        self.image_resources.get(source).map(|state| match state {
            ImageResourceState::Pending(owner)
            | ImageResourceState::Loaded(_, owner)
            | ImageResourceState::Failed(owner) => owner.clone(),
        })
    }
    /// Resolve the same retained content owner selected by LayoutImage's
    /// resource id, including generated-content and SVG image consumers.
    pub fn ImageResourceContentById(&self, id: u64) -> Option<Rc<DOMImageResourceContent>> {
        self.image_resources.values().find_map(|state| {
            let owner = match state {
                ImageResourceState::Pending(owner)
                | ImageResourceState::Loaded(_, owner)
                | ImageResourceState::Failed(owner) => owner,
            };
            owner
                .Metadata()
                .is_some_and(|metadata| metadata.id == id)
                .then(|| owner.clone())
        })
    }
    pub fn CompleteHTMLImageResource(
        &mut self,
        content: &Rc<DOMImageResourceContent>,
        metadata: Option<ImageResourceMetadata>,
        access_check_failed: bool,
    ) {
        if content.IsLoaded() {
            return;
        }
        content.Complete(metadata, access_check_failed);
        self.InvalidateImageLoaderObservers(content);
    }
    pub fn ImageCurrentSrc(&self, node: usize) -> String {
        let Some(loader) = self.ExistingHTMLImageLoader(node) else {
            return String::new();
        };
        if loader.HadError() {
            return loader.CurrentSource().unwrap_or_default();
        }
        loader
            .GetContent()
            .filter(|content| content.ErrorOccurred() || content.HasImage())
            .map_or_else(String::new, |content| content.URL().to_owned())
    }
    pub fn ImageLoaderLayoutUpdate(
        &self,
        node: usize,
    ) -> Option<image_resource::ImageResourceContentUpdate> {
        self.ExistingHTMLImageLoader(node)
            .map(|loader| loader.LayoutUpdate())
    }
    pub fn ImageResourceFor(&self, source: &str) -> Option<&ImageResourceMetadata> {
        match self.image_resources.get(source) {
            Some(ImageResourceState::Loaded(metadata, _)) => Some(metadata),
            _ => None,
        }
    }
    // cpp: core/html/media/html_video_element.cc:769-805
    pub fn SetVideoFrameMetadata(&mut self, node_id: u64, metadata: VideoFrameMetadata) -> bool {
        let node = self
            .FindNodeById(node_id)
            .unwrap_or_else(|| invalid_argument("video frame node is missing"));
        if !self.arena[node].IsElement("video", DOMNamespace::kHTML) {
            invalid_argument("video frame must target an HTML video element");
        }
        let previous = self.video_frames.insert(node_id, metadata);
        let geometry_changed = previous.is_none_or(|previous| {
            previous.width != metadata.width || previous.height != metadata.height
        });
        if geometry_changed && self.RecordLayoutGeometryChange(node) {
            self.style_state.geometry_input_nodes.insert(node_id);
        }
        geometry_changed
    }
    /// Source: Element::SetRegionCaptureCropId/GetRegionCaptureCropId.
    /// The token is installed at most once and causes a full paint update so
    /// the capture ID crosses the immutable Layout/Paint boundary.
    pub fn SetRegionCaptureCropId(&mut self, node_id: u64, crop_id: [u64; 2]) -> bool {
        if crop_id == [0; 2] {
            invalid_argument("region capture crop id must be non-null");
        }
        let node = self
            .FindNodeById(node_id)
            .unwrap_or_else(|| invalid_argument("region capture element is missing"));
        if self.arena[node].Type() != DOMNodeType::kElement {
            invalid_argument("region capture target must be an element");
        }
        if self.arena[node].region_capture_crop_id.is_some() {
            invalid_argument("region capture crop id is already installed");
        }
        self.arena[node].region_capture_crop_id = Some(crop_id);
        self.style_state.paint_style_nodes.insert(node_id);
        // SVG paint-only fast paths intentionally reject generic live SVG
        // objects. Queue the canonical NodeInput as well; geometry admission
        // below classifies this token as paint-only and keeps layout intact.
        self.style_state.layout_style_nodes.insert(node_id);
        self.style_state.paint_style_requires_export = true;
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact {
                paint: true,
                ..Default::default()
            });
        true
    }
    /// Source: Element::SetTrackedElementSubRect. The descriptor is retained
    /// on the Element, sorted by feature, and forces a fresh fragment/paint
    /// export. Layout admission remains authoritative for inline-box creation.
    pub fn SetTrackedElementSubRect(
        &mut self,
        node_id: u64,
        tracked: TrackedElementSubRect,
    ) -> bool {
        if tracked.id == [0; 2] {
            invalid_argument("tracked element id must be non-null");
        }
        if let Some(relative) = tracked.sub_rect {
            if relative.width < 0 || relative.height < 0 {
                invalid_argument("tracked element sub-rect size must be nonnegative");
            }
        }
        let node = self
            .FindNodeById(node_id)
            .unwrap_or_else(|| invalid_argument("tracked element is missing"));
        if self.arena[node].Type() != DOMNodeType::kElement {
            invalid_argument("tracked element target must be an element");
        }
        if self.arena[node]
            .tracked_element_sub_rects
            .iter()
            .any(|current| current.feature == tracked.feature)
        {
            invalid_argument("tracked element feature is already installed");
        }
        self.arena[node].tracked_element_sub_rects.push(tracked);
        self.arena[node]
            .tracked_element_sub_rects
            .sort_by_key(|current| current.feature);
        self.ScheduleElementTrackingUpdate(node_id);
        true
    }
    /// Source: Element::ClearTrackedElementSubRect.
    pub fn ClearTrackedElementSubRect(&mut self, node_id: u64, feature: i32) -> bool {
        let Some(node) = self.FindNodeById(node_id) else {
            return false;
        };
        let previous = self.arena[node].tracked_element_sub_rects.len();
        self.arena[node]
            .tracked_element_sub_rects
            .retain(|current| current.feature != feature);
        if self.arena[node].tracked_element_sub_rects.len() == previous {
            return false;
        }
        self.ScheduleElementTrackingUpdate(node_id);
        true
    }
    fn ScheduleElementTrackingUpdate(&mut self, node_id: u64) {
        self.style_state.paint_style_nodes.insert(node_id);
        self.style_state.layout_style_nodes.insert(node_id);
        self.style_state.paint_style_requires_export = true;
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact {
                paint: true,
                ..Default::default()
            });
    }
    pub fn VideoFrameMetadataFor(&self, node_id: u64) -> Option<VideoFrameMetadata> {
        self.video_frames.get(&node_id).copied()
    }
    pub fn ImageResourceFailedFor(&self, source: &str) -> bool {
        matches!(
            self.image_resources.get(source),
            Some(ImageResourceState::Failed(_))
        )
    }
    pub fn SetImageResourceIsDocument(&mut self, id: u64, is_document_image: bool) {
        if is_document_image {
            self.document_image_resource_ids.insert(id);
        } else {
            self.document_image_resource_ids.remove(&id);
        }
    }
    pub fn ImageResourceIsDocument(&self, id: u64) -> bool {
        self.document_image_resource_ids.contains(&id)
    }
    pub fn SetImageResourceMaybeAnimated(&mut self, id: u64, maybe_animated: bool) {
        if maybe_animated {
            self.maybe_animated_image_resource_ids.insert(id);
        } else {
            self.maybe_animated_image_resource_ids.remove(&id);
        }
    }
    pub fn ImageResourceMaybeAnimated(&self, id: u64) -> bool {
        self.maybe_animated_image_resource_ids.contains(&id)
    }
    pub fn SetImageResourceContentDPR(&mut self, id: u64, content_dpr: Option<f64>) {
        if let Some(content_dpr) = content_dpr {
            self.content_dpr_by_image_resource_id
                .insert(id, content_dpr);
        } else {
            self.content_dpr_by_image_resource_id.remove(&id);
        }
    }
    pub fn ImageResourceContentDPR(&self, id: u64) -> Option<f64> {
        self.content_dpr_by_image_resource_id.get(&id).copied()
    }
    /// ImageResourceContent::NumberOfObservers() > 2 for an animated image is
    /// equivalent here to more than one live LayoutImage consumer: the loader
    /// and first LayoutImage form Blink's baseline two observers.
    ///
    /// cpp: core/layout/layout_image.cc:305-313
    pub fn ImageResourceUsedByMultipleReplacedImages(&self, id: u64) -> bool {
        self.image_resources.iter().any(|(source, state)| {
            matches!(state, ImageResourceState::Loaded(metadata,_) if metadata.id == id)
                && self
                    .replaced_image_consumers
                    .get(source)
                    .is_some_and(|consumers| {
                        consumers
                            .iter()
                            .filter(|&&index| {
                                self.IsConnectedToDocument(index)
                                    && self.CurrentImageSource(index) == Some(source.as_str())
                            })
                            .take(2)
                            .count()
                            > 1
                    })
        })
    }
    pub fn CSSUseCounter(&self) -> Rc<foundation::css_use_counter::CSSUseCounter> {
        self.use_counter.CSS()
    }
    pub fn CountWebDX(&self, feature: crate::use_counter::WebDXFeature) {
        self.use_counter.CountWebDX(feature);
    }
    pub fn IsWebDXCounted(&self, feature: crate::use_counter::WebDXFeature) -> bool {
        self.use_counter.IsWebDXCounted(feature)
    }
    pub fn CountUse(&self, feature: crate::use_counter::WebFeature) {
        self.use_counter.Count(feature);
    }
    pub fn IsUseCounted(&self, feature: crate::use_counter::WebFeature) -> bool {
        self.use_counter.IsCounted(feature)
    }
    pub fn ResponsiveImageSelectionFor(&self, node: usize) -> Option<&ResponsiveImageSelection> {
        self.responsive_image_selections.get(&node)
    }
    pub fn AllowsResponsiveImageAutoSizes(&self, node: usize) -> bool {
        let image = self.Node(node);
        if !image.IsHTMLElement("img")
            || !image
                .FindAttribute("loading")
                .is_some_and(|attribute| attribute.value.eq_ignore_ascii_case("lazy"))
        {
            return false;
        }
        image.FindAttribute("sizes").is_some_and(|attribute| {
            attribute.value.eq_ignore_ascii_case("auto")
                || attribute
                    .value
                    .get(..5)
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case("auto,"))
        })
    }
    pub fn ResponsiveImageAutoSize(&self, node: usize) -> Option<f64> {
        self.responsive_image_auto_sizes.get(&node).copied()
    }
    /// HTMLImageElement::UpdateLastAutoSizesWidth. Width is the rounded,
    /// absolute-zoom-adjusted physical content-box width from Layout.
    pub fn SetResponsiveImageAutoSize(&mut self, node: usize, width: f64) -> bool {
        if !width.is_finite() || width < 0.0 {
            return false;
        }
        let changed = self.responsive_image_auto_sizes.get(&node).copied() != Some(width);
        if changed {
            self.responsive_image_auto_sizes.insert(node, width);
        }
        changed
    }
    pub fn CurrentImageSource(&self, node: usize) -> Option<&str> {
        self.ResponsiveImageSelectionFor(node)
            .map(|selection| selection.source.as_str())
            .or_else(|| {
                self.Node(node)
                    .FindAttribute("src")
                    .map(|attribute| attribute.value.as_str())
            })
            .filter(|source| !source.is_empty())
    }
    pub fn SetResponsiveImageSelection(
        &mut self,
        node: usize,
        selection: Option<ResponsiveImageSelection>,
    ) -> bool {
        let changed = self.responsive_image_selections.get(&node) != selection.as_ref();
        if !changed {
            return false;
        }
        let old_source = self.CurrentImageSource(node).map(str::to_owned);
        if let Some(selection) = selection {
            self.responsive_image_selections.insert(node, selection);
        } else {
            self.responsive_image_selections.remove(&node);
        }
        self.SyncReplacedImageConsumer(node, old_source.as_deref());
        // Candidate changes alter the replaced resource id, intrinsic size
        // and image_device_pixel_ratio without changing selector matching.
        if self.RecordLayoutGeometryChange(node) {
            self.style_state
                .geometry_input_nodes
                .insert(self.arena[node].id);
        }
        true
    }
    pub fn UpdateImageResourceIntrinsicSize(
        &mut self,
        source: &str,
        metadata: ImageResourceMetadata,
    ) -> bool {
        let Some(ImageResourceState::Loaded(resource, _)) = self.image_resources.get_mut(source)
        else {
            return false;
        };
        *resource = metadata;
        self.InvalidateImageResourceConsumers(source);
        true
    }

    // cpp: dom/document.h:197-199
    // cpp: dom/document.cc:351-364
    pub fn ResolvedStyleFor(&self, node: usize) -> Option<&ResolvedNodeStyle> {
        self.arena[node]
            .resolved_style
            .style
            .as_ref()
            .map(std::sync::Arc::as_ref)
    }
    // cpp: element.cc:9414-9416; node_rare_data.h:425-430.
    pub fn SetScrollbarPseudoElementStylesDependOnFontMetrics(&mut self, node: usize, value: bool) {
        assert!(self.arena[node].node_type == DOMNodeType::kElement);
        self.arena[node]
            .resolved_style
            .scrollbar_pseudo_element_styles_depend_on_font_metrics = value;
    }
    pub fn ScrollbarPseudoElementStylesDependOnFontMetrics(&self, node: usize) -> bool {
        self.arena[node]
            .resolved_style
            .scrollbar_pseudo_element_styles_depend_on_font_metrics
    }

    pub fn ResolvedStyleResourcesFor(&self, node: usize) -> &ResolvedStyleResources {
        &self.arena[node].resolved_style.resources
    }
    /// Borrow the canonical resolved output without traversing allocation
    /// history or materializing a second style collection.
    pub fn ResolvedStyles(&self) -> impl Iterator<Item = (usize, &ResolvedNodeStyle)> {
        self.arena.iter().enumerate().filter_map(|(node, record)| {
            record
                .resolved_style
                .style
                .as_ref()
                .map(|style| (node, style.as_ref()))
        })
    }
    /// Compatibility entry for a full style invalidation. Keep old values for
    /// comparison and sharing; style reads update before exposing current data.
    pub fn ClearResolvedStyles(&mut self) {
        self.InvalidateAllStyles();
    }
    pub fn StyleState(&self) -> &crate::style_state::StyleState {
        &self.style_state
    }
    pub fn StyleStateMut(&mut self) -> &mut crate::style_state::StyleState {
        &mut self.style_state
    }
    pub fn InvalidateAllStyles(&mut self) {
        self.style_state.all_dirty = true;
        self.style_state.changes.clear();
        self.style_state.independent_inline_styles.clear();
        self.style_state.attribute_changes.clear();
        self.style_state.sibling_sensitive_nodes.clear();
        self.style_state.append_only_children.clear();
        self.style_state.non_append_children.clear();
        self.style_state.child_change_boundaries.clear();
        self.style_state.inserted_style_subtrees.clear();
        self.style_state.append_affected_subtrees.clear();
        self.style_state.layout_inputs_require_tree = true;
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
    }

    /// Mark only styles whose computed values consumed font metrics.
    ///
    /// Chromium's font-update invalidation walks the styled subtree and gives
    /// these elements a local style change; ordinary font-family users keep
    /// their computed style and pick up the completed face during Layout.
    /// This is materially narrower than `InvalidateAllStyles()` for a document
    /// with a small number of `ch`/`ex`/`cap`/`ic` or `font-size-adjust`
    /// consumers.
    // cpp: core/dom/node.cc:1488-1515
    // cpp: core/dom/element.cc:11249-11263
    // cpp: core/css/style_engine.cc:1342-1362
    pub fn InvalidateStylesForFontMetrics(&mut self) -> usize {
        fn pseudo_depends_on_font_metrics(pseudo: &PseudoElement) -> bool {
            unsafe { pseudo.computed_style_owner.Get().as_ref() }
                .is_some_and(StyleComputedStyle::DependsOnFontMetrics)
                || pseudo
                    .marker
                    .as_deref()
                    .is_some_and(pseudo_depends_on_font_metrics)
        }

        fn style_depends_on_font_metrics(style: &ResolvedNodeStyle) -> bool {
            unsafe { style.computed_style_owner.Get().as_ref() }
                .is_some_and(StyleComputedStyle::DependsOnFontMetrics)
                || [
                    &style.before,
                    &style.after,
                    &style.first_letter,
                    &style.placeholder,
                    &style.marker,
                ]
                .into_iter()
                .flatten()
                .any(pseudo_depends_on_font_metrics)
        }

        if self.style_state.all_dirty {
            return 0;
        }
        let nodes = self
            .ResolvedStyles()
            .filter_map(|(node, style)| {
                // cpp: Element::PseudoElementStylesDependOnFontMetrics checks
                // the retained scrollbar flag before walking generated pseudos.
                (self.ScrollbarPseudoElementStylesDependOnFontMetrics(node)
                    || style_depends_on_font_metrics(style))
                .then_some(node)
            })
            .collect::<Vec<_>>();
        for &node in &nodes {
            // Font completion does not change selector inputs. Match Blink's
            // kLocalStyleChange instead of publishing sibling/descendant
            // selector sensitivity through the ordinary DOM mutation helper.
            self.RecordStyleChange(crate::style_state::StyleChange::Node(node));
        }
        nodes.len()
    }
    // cpp: StyleEngine::ShouldSkipInvalidationFor / Node::SetNeedsStyleRecalc.
    // Detached preparation keeps DOM/selector data current, but cannot dirty
    // the active document. Insertion records the complete newly connected tree.
    fn ShouldRecordStyleChange(&self, node: usize) -> bool {
        !self.style_state.all_dirty && self.IsConnectedToDocument(node)
    }
    fn RecordStyleChange(&mut self, change: crate::style_state::StyleChange) {
        use crate::style_state::StyleChange as C;
        let (C::Node(node)
        | C::AttributeValue(node)
        | C::PlaceholderShown(node)
        | C::EmptyState(node)
        | C::Directionality(node)
        | C::PseudoSelection(node)
        | C::Defined(node)
        | C::ControlPseudo(node, _)
        | C::UserAction(node, _)
        | C::Dragged(node, _)
        | C::InlineStyle(node)
        | C::ExportParts(node)
        | C::Children(node)
        | C::Animation(node)
        | C::Resource(node)) = change;
        if self.IsConnectedToDocument(node) {
            self.lifecycle.ScheduleVisualUpdate();
        }
        if self.ShouldRecordStyleChange(node) {
            // These producers are direct local SetNeedsStyleRecalc sources;
            // selector/pseudo/child invalidation facts remain separate.
            if matches!(
                change,
                C::Node(_)
                    | C::InlineStyle(_)
                    | C::Directionality(_)
                    | C::Animation(_)
                    | C::Resource(_)
            ) {
                self.style_state.style_recalc_dirty_nodes.insert(node);
            }
            if !matches!(change, C::InlineStyle(_)) {
                self.style_state.independent_inline_styles.remove(&node);
            }
            self.style_state.changes.insert(change);
        }
    }
    fn NodeMayAffectSiblingSelectors(&self, node: usize) -> bool {
        let _ = node;
        // DOM records the structural fact only.  Selector-dependent filtering
        // belongs to StyleEngine and is applied when it consumes this batch.
        true
    }
    fn NodeMayAffectDescendantSelectors(&self, node: usize) -> bool {
        let _ = node;
        true
    }
    fn RecordSiblingSensitivity(&mut self, node: usize, conservative: bool) {
        if !self.ShouldRecordStyleChange(node) {
            return;
        }
        let sensitive = conservative || self.NodeMayAffectSiblingSelectors(node);
        if sensitive {
            self.style_state.sibling_sensitive_nodes.insert(node);
        }
    }
    fn RecordDescendantSensitivity(&mut self, node: usize, conservative: bool) {
        if !self.ShouldRecordStyleChange(node) {
            return;
        }
        if conservative || self.NodeMayAffectDescendantSelectors(node) {
            self.style_state.descendant_sensitive_nodes.insert(node);
        }
    }
    fn MarkStyleElementTextDirty(&mut self, node: usize) {
        let mut ancestor = Some(node);
        while let Some(index) = ancestor {
            if self.arena[index].IsStyleElement() {
                self.style_state
                    .dirty_style_elements
                    .insert(self.arena[index].id);
                break;
            }
            ancestor = self.arena[index].parent;
        }
    }
    // Changing a coalesced nonempty text node does not add/remove an element
    // sibling or change :empty. Retain text layout and stylesheet reparsing,
    // without promoting every parser body chunk through sibling/:has restyles.
    // Textarea default content also feeds its control value/placeholder state.
    fn RecordCharacterDataChange(&mut self, parent: usize, was_empty: bool, is_empty: bool) {
        self.RecordFragmentAnchorContentChanged();
        self.OptionDidChangeTextContent(parent);
        self.BumpCollectionMembershipRevision();
        if self.IsConnectedToDocument(parent) && self.IsSVGResourceStyleNode(parent) {
            self.BumpSVGResourceDOMRevision();
        }
        self.MarkStyleElementTextDirty(parent);
        self.SyncTextAreaDefaultValue(parent);
        if was_empty != is_empty || self.arena[parent].IsHTMLElement("textarea") {
            self.RecordStyleChange(crate::style_state::StyleChange::EmptyState(parent));
        }
        self.RecordLayoutGeometryChange(parent);
        self.RecordSelectContentGeometryChange(parent);
        self.UpdateAutoDirectionalityAncestors(Some(parent));
        self.UpdateDirectionalitySlots(parent);
        self.RefreshSelectSuggestedValues();
    }
    fn RecordChildrenChange(&mut self, parent: usize) {
        self.RecordFragmentAnchorContentChanged();
        self.InvalidateDocumentImagesCollection(parent);
        self.IncDOMTreeVersion();
        self.OptionChildrenChanged(parent);
        self.AssignDetailsSlots(parent);
        self.AssignSelectSlots(parent);

        self.InvalidateTableRowCells(parent);
        self.RecordEditingNodeFactsChange(parent);
        self.BumpCollectionMembershipRevision();
        self.InvalidateSelectorSubtree(parent);
        self.MarkStyleElementTextDirty(parent);
        self.SyncTextAreaDefaultValue(parent);
        self.RecordStyleChange(crate::style_state::StyleChange::Children(parent));
        self.RecordFlatTreeChange(parent);
        self.RecordSelectContentGeometryChange(parent);
        if self.arena[parent].IsElement("fieldset", DOMNamespace::kHTML) {
            self.InvalidateFormDisabledDescendants(parent);
        }
        // cpp: id_target_observer.cc / HTMLInputElement::ListAttributeTargetChanged.
        // Inserting, removing or reordering an ID target can change which
        // datalist a list-bearing input resolves in its TreeScope. The compact
        // DOM owner does not retain Blink's per-ID observer registry, so it
        // publishes the equivalent input invalidations here.
        // cpp: ListedElement::InsertedInto/RemovedFrom and
        // HTMLInputElement::ResetListAttributeTargetObserver. Mutations in a
        // detached fragment cannot change the active document's form/ID target
        // observers. Attachment/removal still notifies the connected parent.
        if !self.IsConnectedToDocument(parent) {
            self.SynchronizeRadioGroups();
        }
        if self.IsConnectedToDocument(parent) {
            let _trace = browser_tracing::span("dom", "DOM.InvalidateFormObservers");
            self.InvalidateDatalistInputs();
            self.InvalidateRadioInputs();
            self.InvalidateDefaultPseudoCandidates();
        }
        self.InvalidateAncestorSelectContainsInput(parent);
        self.UpdateAutoDirectionalityAncestors(Some(parent));
        self.UpdateDirectionalitySlots(parent);
        self.RefreshSelectSuggestedValues();
        self.SynchronizeAllElementInternals();
    }

    fn InvalidateAncestorSelectContainsInput(&mut self, node: usize) {
        if self.style_state.all_dirty {
            return;
        }
        let mut current = Some(node);
        while let Some(element) = current {
            if self.arena[element].IsHTMLElement("select") {
                self.InvalidateNodeStyle(element);
            }
            current = self.arena[element].Parent();
        }
    }

    // Fieldset/optgroup/select disabledness changes the pseudo state of
    // descendants without changing their own attributes. Blink walks these
    // descendants from the mutation producer; retain the same ownership here
    // and publish ordinary node invalidations to StyleEngine.
    fn InvalidateFormDisabledDescendants(&mut self, root: usize) {
        let mut pending = self.arena[root].children.clone();
        while let Some(node) = pending.pop() {
            pending.extend_from_slice(&self.arena[node].children);
            if self.FormControlClass(node).is_some() || self.IsFormAssociatedCustomElement(node) {
                self.SetControlNeedsValidityCheck(node);
            }
            if self.arena[node].Namespace() == DOMNamespace::kHTML
                && matches!(
                    self.arena[node].Name(),
                    "button" | "fieldset" | "input" | "select" | "textarea" | "option" | "optgroup"
                )
            {
                self.InvalidateNodeStyle(node);
            }
        }
    }

    // cpp: ListedElement/form-control and ID-target observer ownership. Keep a
    // conservative subject index; every notification evaluates the same live
    // predicates, including detached controls and dynamic input types.
    fn FormObserverElements(&mut self) -> Vec<usize> {
        if self.form_observer_elements.is_none() {
            self.form_observer_elements = Some(
                (0..self.arena.len())
                    .filter(|&node| {
                        self.arena[node].IsHTMLElement("input")
                            || self.arena[node].IsHTMLElement("button")
                    })
                    .collect(),
            );
        }
        self.form_observer_elements.as_ref().unwrap().clone()
    }

    fn InvalidateDatalistInputs(&mut self) {
        if self.style_state.all_dirty {
            return;
        }
        let inputs = self
            .FormObserverElements()
            .into_iter()
            .filter(|&node| {
                self.InputType(node).is_some_and(|input_type| {
                    input_type.ShouldRespectListAttribute()
                        && self.arena[node]
                            .FindAttribute("list")
                            .is_some_and(|attribute| !attribute.value.is_empty())
                })
            })
            .collect::<Vec<_>>();
        for input in inputs {
            if self.IsConnectedToDocument(input) {
                self.RecordStyleChange(crate::style_state::StyleChange::ControlPseudo(
                    input,
                    crate::style_state::ControlPseudoState::HasDatalist,
                ));
            }
        }
    }

    fn InvalidateRadioInputs(&mut self) {
        self.SynchronizeRadioGroups();
        if self.style_state.all_dirty {
            return;
        }
        let radios = self
            .FormObserverElements()
            .into_iter()
            .filter(|&node| self.InputType(node) == Some(DOMInputType::kRadio))
            .collect::<Vec<_>>();
        for radio in radios {
            if self.IsConnectedToDocument(radio) {
                self.RecordStyleChange(crate::style_state::StyleChange::ControlPseudo(
                    radio,
                    crate::style_state::ControlPseudoState::Indeterminate,
                ));
            }
        }
    }

    // HTMLFormElement::FindDefaultButton is tree-order dependent. A form
    // association, submit-button type, form id, or children mutation can move
    // :default from one control to another without mutating the new control.
    fn InvalidateDefaultPseudoCandidates(&mut self) {
        if self.style_state.all_dirty {
            return;
        }
        let candidates = self
            .FormObserverElements()
            .into_iter()
            .filter(|&node| {
                self.CanBeSuccessfulSubmitButton(node)
                    && self.FormOwner(node).is_some()
                    && self.IsConnectedToDocument(node)
            })
            .collect::<Vec<_>>();
        for candidate in candidates {
            // cpp: HTMLFormElement::InvalidateDefaultButtonStyle. Affected
            // listed submit controls receive PseudoStateChanged(kPseudoDefault),
            // rather than ordinary attribute/sibling invalidation.
            self.RecordStyleChange(crate::style_state::StyleChange::ControlPseudo(
                candidate,
                crate::style_state::ControlPseudoState::Default,
            ));
        }
    }
    fn RecordChildrenBoundary(
        &mut self,
        parent: usize,
        boundary: Option<crate::style_state::ChildrenChangeBoundary>,
    ) {
        if !self.ShouldRecordStyleChange(parent) {
            return;
        }
        let entry = self
            .style_state
            .child_change_boundaries
            .entry(parent)
            .or_insert_with(|| Some(Vec::new()));
        if let Some(boundary) = boundary {
            if let Some(boundaries) = entry {
                boundaries.push(boundary);
            }
        } else {
            *entry = None;
        }
    }
    fn RecordNonAppendChildrenChange(
        &mut self,
        parent: usize,
        boundary: Option<crate::style_state::ChildrenChangeBoundary>,
    ) {
        self.RecordChildrenChange(parent);
        self.RecordChildrenBoundary(parent, boundary);
        if !self.ShouldRecordStyleChange(parent) {
            return;
        }
        self.style_state.non_append_children.insert(parent);
        self.style_state.append_only_children.remove(&parent);
    }
    fn RecordAppendChildrenChange(
        &mut self,
        parent: usize,
        child: usize,
        previous_last: Option<usize>,
        previous_same_type: Option<usize>,
    ) {
        self.RecordChildrenChange(parent);
        self.RecordChildrenBoundary(
            parent,
            Some(crate::style_state::ChildrenChangeBoundary {
                before: (self.arena[child].node_type == DOMNodeType::kElement)
                    .then_some(previous_last)
                    .flatten(),
                after: None,
            }),
        );
        if !self.ShouldRecordStyleChange(parent) {
            return;
        }
        if !self.style_state.non_append_children.contains(&parent) {
            self.style_state.append_only_children.insert(parent);
        }
        if self.arena[child].node_type == DOMNodeType::kElement {
            self.style_state.inserted_style_subtrees.insert(child);
        }
        for node in [previous_last, previous_same_type].into_iter().flatten() {
            self.style_state.append_affected_subtrees.insert(node);
        }
    }
    // cpp: StyleEngine::ClassChangedForElement / IdChangedForElement.
    // Preserve removed keys before changing the DOM; Style owns rule lookup.
    fn RecordAttributeSelectorKeys(
        &mut self,
        node: usize,
        name: &str,
        namespace: &str,
        value: Option<&str>,
    ) {
        if !self.ShouldRecordStyleChange(node) {
            return;
        }
        let old = self.arena[node]
            .attributes
            .iter()
            .find(|a| a.local_name == name && a.namespace_uri == namespace)
            .map(|a| a.value.as_str());
        let change = self.style_state.attribute_changes.entry(node).or_default();
        change.names.insert(name.to_string());
        if namespace.is_empty() && name == "id" {
            change.ids.extend(
                old.into_iter()
                    .chain(value)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string),
            );
        }
        if namespace.is_empty() && name == "class" {
            let old = old
                .unwrap_or_default()
                .split_ascii_whitespace()
                .collect::<HashSet<_>>();
            let new = value
                .unwrap_or_default()
                .split_ascii_whitespace()
                .collect::<HashSet<_>>();
            change
                .classes
                .extend(old.symmetric_difference(&new).map(|s| (*s).to_string()));
        }
    }
    fn RecordAttributeStyleChange(
        &mut self,
        node: usize,
        name: &str,
        namespace: &str,
        sibling_sensitive: bool,
    ) -> bool {
        self.RecordStyleChange(crate::style_state::StyleChange::AttributeValue(node));
        if sibling_sensitive && self.ShouldRecordStyleChange(node) {
            self.style_state.sibling_sensitive_nodes.insert(node);
        }
        // Custom data and script loading metadata have no HTML presentation
        // effect. Style still observes their ordinary attribute selector keys.
        namespace.is_empty()
            && self.arena[node].namespace == DOMNamespace::kHTML
            && (name.starts_with("data-")
                || (self.arena[node].IsHTMLElement("script") && matches!(name, "src" | "nonce")))
    }

    pub fn InvalidateNodeStyle(&mut self, node: usize) {
        self.RecordStyleChange(crate::style_state::StyleChange::Node(node));
        self.RecordSiblingSensitivity(node, true);
        self.RecordDescendantSensitivity(node, true);
    }
    pub(crate) fn RecordDefinedStyleChange(&mut self, node: usize) {
        self.RecordStyleChange(crate::style_state::StyleChange::Defined(node));
    }
    pub fn ResolvedStyleHandle(&self, node: usize) -> Option<std::sync::Arc<ResolvedNodeStyle>> {
        self.arena[node].resolved_style.style.clone()
    }
    /// Resolved-style owners with pending image dependencies. The source list
    /// preserves source ComputedStyle field order; callers may place owners in
    /// shadow-including document order without scanning unrelated DOM nodes.
    pub fn ReferencedStyleImageConsumers(&self) -> Vec<(usize, std::sync::Arc<[String]>)> {
        let mut nodes = BTreeSet::new();
        for consumers in self.style_image_consumers.values() {
            nodes.extend(consumers.iter().copied());
        }
        nodes
            .into_iter()
            .filter_map(|node| {
                self.ResolvedStyleFor(node)
                    .map(|style| (node, style.referenced_style_image_sources.clone()))
            })
            .collect()
    }
    fn IsSVGResourceStyleNode(&self, node: usize) -> bool {
        let mut current = Some(node);
        while let Some(index) = current {
            if self.arena[index].namespace == DOMNamespace::kSVG {
                return true;
            }
            current = self.arena[index].parent;
        }
        false
    }
    // LayoutSVGResourceContainer::StyleChanged/MarkForLayoutAndParentResourceInvalidation.
    // These are the seven LayoutSVGResourceType owners. Ordinary SVG nodes
    // still invalidate the resource catalog above (e.g. external clipPath use
    // targets), but only resource owners require cross-branch client updates.
    fn AffectsSVGResourceClients(&self, node: usize) -> bool {
        let mut current = Some(node);
        while let Some(index) = current {
            let element = self.Node(index);
            if element.Namespace() == DOMNamespace::kSVG
                && matches!(
                    element.Name(),
                    "clipPath"
                        | "mask"
                        | "filter"
                        | "marker"
                        | "linearGradient"
                        | "radialGradient"
                        | "pattern"
                )
            {
                return true;
            }
            current = element.Parent();
        }
        false
    }
    pub fn SetResolvedStyle(&mut self, node: usize, mut style: ResolvedNodeStyle) {
        if self.ResolvedStyleFor(node) == Some(&style) {
            return;
        }
        // EnsureComputedStyle caches hidden query results. They have no layout
        // or paint client and must not start resource loads or invalidate frames.
        if unsafe { style.computed_style_owner.Get().as_ref() }
            .is_some_and(|source| source.IsEnsuredInDisplayNone())
        {
            debug_assert!(!style.generates_box);
            self.arena[node].resolved_style.resources = Default::default();
            self.arena[node].resolved_style.style = Some(std::sync::Arc::new(style));
            return;
        }
        self.BindGeneratedPseudoElements(node, &style);
        self.RecordEditingStyleChange(node);
        style.RefreshReferencedStyleImageSources();
        let same_pseudo = |a: &Option<PseudoElement>, b: &Option<PseudoElement>| match (a, b) {
            (Some(a), Some(b)) => a.SamePaintInput(b),
            (None, None) => true,
            _ => false,
        };
        let requires_export = self.ResolvedStyleFor(node).is_none_or(|old| {
            old.display_contents
                || style.display_contents
                || !same_pseudo(&old.before, &style.before)
                || !same_pseudo(&old.after, &style.after)
                || !same_pseudo(&old.first_letter, &style.first_letter)
                || !same_pseudo(&old.placeholder, &style.placeholder)
                || !same_pseudo(&old.marker, &style.marker)
        }) || self.IsSVGResourceStyleNode(node);
        self.style_state
            .layout_style_nodes
            .insert(self.arena[node].id);
        self.style_state
            .paint_style_nodes
            .insert(self.arena[node].id);
        if requires_export {
            // Generated/flattened children need attachment context once. A
            // newly styled element has no resident attachment yet, so start
            // at its parent rather than enqueueing every new child separately.
            let root = if self.ResolvedStyleFor(node).is_some() {
                node
            } else {
                self.FlatTreeParentElement(node).unwrap_or(node)
            };
            self.style_state
                .layout_tree_roots
                .insert(self.arena[root].id);
        }
        self.style_state.paint_style_requires_export |= requires_export;
        if self.AffectsSVGResourceClients(node) {
            self.style_state.layout_inputs_require_tree = true;
        }
        self.resolved_style_revision = self.resolved_style_revision.saturating_add(1);
        if self.IsSVGResourceStyleNode(node) {
            self.svg_resource_style_revision = self.svg_resource_style_revision.saturating_add(1);
        }
        let new_sources = style.referenced_style_image_sources.clone();
        let same_sources = self
            .ResolvedStyleFor(node)
            .is_some_and(|old| old.referenced_style_image_sources == new_sources);
        if !same_sources {
            self.RemoveResolvedStyle(node);
            for source in new_sources.iter() {
                self.style_image_consumers
                    .entry(source.clone())
                    .or_default()
                    .insert(node);
            }
        }
        self.arena[node].resolved_style.resources = Default::default();
        self.arena[node].resolved_style.style = Some(std::sync::Arc::new(style));
        self.VTTNodeDidRecalcStyle(node);
        if matches!(self.arena[node].class, DOMNodeClass::HTMLSelectElement(_)) {
            self.attribute_lifecycle
                .RecordSelectDidRecalcStyle(self.arena[node].id);
        }
        if matches!(
            self.SelectUAPart(node),
            Some((_, DOMSelectUAPartKind::Picker))
        ) {
            self.attribute_lifecycle
                .RecordSelectPickerDidRecalcStyle(self.arena[node].id);
        }
    }
    pub fn SetResolvedStyleResources(&mut self, node: usize, resources: ResolvedStyleResources) {
        if self.arena[node].resolved_style.resources == resources {
            return;
        }
        if self.arena[node]
            .resolved_style
            .resources
            .svg_length_container_sizes
            != resources.svg_length_container_sizes
            || self.arena[node]
                .resolved_style
                .resources
                .svg_marker_resolved_lengths
                != resources.svg_marker_resolved_lengths
        {
            self.style_state
                .geometry_input_nodes
                .insert(self.arena[node].id);
        }
        self.arena[node].resolved_style.resources = resources;
        self.style_state
            .paint_style_nodes
            .insert(self.arena[node].id);
        self.resolved_style_revision = self.resolved_style_revision.saturating_add(1);
    }
    /// Style's normal lifecycle can publish absence of computed style, just as
    /// Element::SetComputedStyle(nullptr) does for display:none subtrees.
    pub fn ClearResolvedStyle(&mut self, node: usize) {
        self.BindGeneratedPseudoElements(node, &ResolvedNodeStyle::default());
        self.DetachOverscroll(node);
        self.DisplayLockElementDisconnected(node);
        let Some(old) = self.ResolvedStyleFor(node) else {
            return;
        };
        if unsafe { old.computed_style_owner.Get().as_ref() }
            .is_some_and(|style| style.IsEnsuredInDisplayNone())
        {
            self.arena[node].resolved_style.style = None;
            self.arena[node].resolved_style.resources = Default::default();
            return;
        }
        // Style recalc clears a suppressed subtree parent-first. Once an
        // ancestor is queued for attachment, that transaction owns every
        // descendant's layout and paint removal; enqueueing each cleared child
        // repeats local lookup work before attachment roots are coalesced.
        let covered_by_attachment_ancestor = self.style_state.layout_inputs_require_tree || {
            let mut ancestor = self.FlatTreeParentElement(node);
            let mut covered = false;
            while let Some(index) = ancestor {
                if self
                    .style_state
                    .layout_tree_roots
                    .contains(&self.arena[index].id)
                {
                    covered = true;
                    break;
                }
                ancestor = self.FlatTreeParentElement(index);
            }
            covered
        };
        if !covered_by_attachment_ancestor {
            self.style_state
                .layout_style_nodes
                .insert(self.arena[node].id);
            self.style_state
                .layout_tree_roots
                .insert(self.arena[node].id);
            self.style_state
                .paint_style_nodes
                .insert(self.arena[node].id);
        }
        if self.AffectsSVGResourceClients(node) {
            self.style_state.layout_inputs_require_tree = true;
        }
        self.style_state.paint_style_requires_export = true;
        self.RemoveResolvedStyle(node);
        self.UserActionElementDetached(node);
        self.VTTNodeDidRecalcStyle(node);
        if matches!(self.arena[node].class, DOMNodeClass::HTMLSelectElement(_)) {
            self.attribute_lifecycle
                .RecordSelectDidRecalcStyle(self.arena[node].id);
        }
        if matches!(
            self.SelectUAPart(node),
            Some((_, DOMSelectUAPartKind::Picker))
        ) {
            self.attribute_lifecycle
                .RecordSelectPickerDidRecalcStyle(self.arena[node].id);
        }
    }
    /// Element::DetachLayoutTree's children then parent style release. The
    /// reattach caller retains styles and therefore does not enter this path.
    pub fn ClearDetachedLayoutSubtreeStyles(&mut self, node: usize) {
        let children = self.FlatTreeChildren(node).into_owned();
        for child in children {
            self.ClearDetachedLayoutSubtreeStyles(child);
        }
        if self.arena[node].node_type == DOMNodeType::kElement {
            self.ClearResolvedStyle(node);
        }
    }
    fn InvalidateResolvedStyleSnapshot(&mut self, node: usize) {
        if self.arena[node].resolved_style.style.take().is_none() {
            return;
        }
        // Keep style_image_consumers until the replacement style is
        // installed. Multiple resources can complete in one owner turn, and
        // every dependency of this stale snapshot must continue to notify the
        // same node. RemoveResolvedStyle cleans these retained registrations
        // before the replacement is registered.
        self.arena[node].resolved_style.resources = Default::default();
        self.resolved_style_revision = self.resolved_style_revision.saturating_add(1);
        if self.IsSVGResourceStyleNode(node) {
            self.svg_resource_style_revision = self.svg_resource_style_revision.saturating_add(1);
        }
    }
    fn RemoveResolvedStyle(&mut self, node: usize) {
        self.RecordEditingStyleChange(node);
        let Some(style) = self.arena[node].resolved_style.style.take() else {
            // A resource completion can invalidate the immutable style
            // snapshot while deliberately retaining its dependency registry
            // until replacement. A simultaneous declaration/removal must not
            // leave those old registrations behind.
            self.style_image_consumers.retain(|_, consumers| {
                consumers.remove(&node);
                !consumers.is_empty()
            });
            return;
        };
        self.arena[node].resolved_style.resources = Default::default();
        self.resolved_style_revision = self.resolved_style_revision.saturating_add(1);
        if self.IsSVGResourceStyleNode(node) {
            self.svg_resource_style_revision = self.svg_resource_style_revision.saturating_add(1);
        }
        for source in style.referenced_style_image_sources.iter() {
            if let Some(consumers) = self.style_image_consumers.get_mut(source) {
                consumers.remove(&node);
                if consumers.is_empty() {
                    self.style_image_consumers.remove(source);
                }
            }
        }
    }

    // cpp: dom/document.h:166-167
    // cpp: dom/document.cc:380-388
    pub fn SetScrollOffset(&mut self, node: usize, offset: Offset) {
        if self.ScrollOffsetFor(node) == offset {
            return;
        }
        let id = self.arena[node].id;
        self.scroll_offsets.insert(id, offset);
        // Geometry queries consume the scroll translation even when no CSS
        // style changed. Do not depend on a full tree projection to carry it.
        self.style_state.geometry_input_nodes.insert(id);
    }
    pub fn ScrollOffsetFor(&self, node: usize) -> Offset {
        self.scroll_offsets
            .get(&self.arena[node].id)
            .copied()
            .unwrap_or_default()
    }

    // cpp: dom/document.h:132-132
    /// Existing DOM connectivity semantics, sampled only by opt-in diagnostics.
    pub fn MemoryDiagnostics(&self) -> DOMMemoryDiagnostics {
        let connected = (0..self.arena.len())
            .filter(|&i| self.IsConnectedToDocument(i))
            .count();
        DOMMemoryDiagnostics {
            arena_nodes: self.arena.len(),
            connected_nodes: connected,
            detached_nodes: self.arena.len() - connected,
            arena_slot_capacity_bytes: self.arena.capacity() * std::mem::size_of::<DOMNode>(),
        }
    }
    /// Subscribe to this arena's source reference writes. The stream moves
    /// with parser document swaps; no consumer borrows DOM during GC.
    pub fn SubscribeNodeReferences(&mut self) -> Rc<crate::node_references::NodeReferenceQueue> {
        use crate::node_references::{NodeReferenceChange as C, NodeReferenceQueue};
        let queue = Rc::new(NodeReferenceQueue::default());
        for (index, node) in self.arena.iter().enumerate() {
            queue.push(C::Object {
                id: node.id,
                document_root: index == self.root,
            });
            if let Some(owner) = node
                .owner_document_handle
                .filter(|owner| owner.arena_id == self.arena_id)
            {
                queue.push(C::OwnerDocument {
                    node: node.id,
                    document: owner.node_id,
                });
            }
            for &child in &node.children {
                queue.push(C::Child {
                    parent: node.id,
                    child: self.arena[child].id,
                    attached: true,
                });
            }
        }
        for (&host, &(root, _, _)) in &self.shadow_roots {
            queue.push(C::Shadow {
                host: self.arena[host].id,
                root: self.arena[root].id,
            });
        }
        for (&owner, &content) in &self.template_contents {
            queue.push(C::Template {
                owner,
                content: self.arena[content].id,
            });
        }
        self.node_reference_observers
            .retain(|observer| observer.strong_count() != 0);
        self.node_reference_observers.push(Rc::downgrade(&queue));
        queue
    }
    fn NotifyNodeReference(&self, change: crate::node_references::NodeReferenceChange) {
        for observer in &self.node_reference_observers {
            if let Some(queue) = observer.upgrade() {
                queue.push(change);
            }
        }
    }
    pub fn NodeCount(&self) -> usize {
        self.arena.len()
    }
    // cpp: dom/document.h:129-129
    pub fn NextNodeId(&self) -> u64 {
        self.next_id
    }
}

// cpp: dom/document.h:237-252
// cpp: dom/document.cc:390-399
pub struct DOM {
    document: PersistentDocument,
}

/// The live document used by style resolution.
///
/// This is the whole boundary: Style receives the canonical document arena by
/// reference, without receiving the surrounding DOM owner or a copied view.
pub trait StyleDocument {
    fn DocumentForStyleMut(&mut self) -> &mut PersistentDocument;
}

impl StyleDocument for PersistentDocument {
    fn DocumentForStyleMut(&mut self) -> &mut PersistentDocument {
        self
    }
}

impl StyleDocument for DOM {
    fn DocumentForStyleMut(&mut self) -> &mut PersistentDocument {
        &mut self.document
    }
}

impl Default for DOM {
    fn default() -> Self {
        Self::new()
    }
}
impl DOM {
    pub fn new() -> Self {
        Self {
            document: PersistentDocument::new(),
        }
    }
    pub fn GetDocument(&self) -> &PersistentDocument {
        &self.document
    }
    pub fn GetDocumentMut(&mut self) -> &mut PersistentDocument {
        &mut self.document
    }

    pub fn GetCSSOM(&self) -> &CSSOM<DOMOwnerHandle> {
        &self.document.cssom
    }

    /// Apply a mutation using the existing DOM validation/application algorithm.
    /// The Document's ordinary mutation methods continue accumulating style
    /// invalidations; this facade neither resolves styles nor runs layout.
    pub fn ApplyMutation(
        &mut self,
        mutation: &crate::dom_mutation::DOMMutation,
    ) -> Result<(), crate::dom_mutation::DOMMutationError> {
        crate::dom_mutation::TryApplyDOMMutations(
            &mut self.document,
            std::slice::from_ref(mutation),
        )
    }

    /// HTML depends on DOM, so its fragment parser is injected by the owner.
    /// Only innerHTML invokes it; all other mutations use ApplyMutation.
    pub fn ApplyMutationWithFragmentParser(
        &mut self,
        mutation: &crate::dom_mutation::DOMMutation,
        parser: impl FnOnce(
            &mut PersistentDocument,
            &crate::dom_mutation::DOMMutation,
        ) -> Result<(), crate::dom_mutation::DOMMutationError>,
    ) -> Result<(), crate::dom_mutation::DOMMutationError> {
        if mutation.mutation_type != crate::dom_mutation::DOMMutationType::kSetInnerHTML {
            return self.ApplyMutation(mutation);
        }
        crate::dom_mutation::ValidateDOMMutation(&self.document, mutation, true)?;
        parser(&mut self.document, mutation)
    }

    // A paused HTML parser owns an exclusive Rust borrow of its Document.
    // Swapping transfers that same arena to the script host for one task,
    // then returns it before parsing resumes. DOM node IDs and arena identity
    // stay with the Document value; this performs no cloning or reparsing.
    pub fn SwapDocument(&mut self, document: &mut PersistentDocument) {
        std::mem::swap(&mut self.document, document);
    }

    // During a parser pause the script host temporarily owns the same arena.
    // Restore it on both normal return and unwinding, before parsing resumes.
    pub fn WithDocument<R>(
        owner: &Rc<RefCell<Self>>,
        document: &mut PersistentDocument,
        task: impl FnOnce() -> R,
    ) -> R {
        struct RestoreDocument<'a> {
            owner: Rc<RefCell<DOM>>,
            document: &'a mut PersistentDocument,
        }
        impl Drop for RestoreDocument<'_> {
            fn drop(&mut self) {
                self.owner.borrow_mut().SwapDocument(self.document);
            }
        }
        owner.borrow_mut().SwapDocument(document);
        let _restore = RestoreDocument {
            owner: owner.clone(),
            document,
        };
        task()
    }
}

#[cfg(test)]
mod task_ownership_tests {
    use super::*;

    #[test]
    fn exportparts_retains_names_map_and_source_parser_semantics() {
        let mut document = PersistentDocument::new();
        let host = document.CreateElement(
            DOMNamespace::kHTML,
            "x-host".into(),
            vec![DOMAttribute {
                local_name: "exportparts".into(),
                value: "inner: outer, inner: badge, identity, bad key, : nope, tail: end".into(),
                ..Default::default()
            }],
        );

        let map = document
            .PartNamesMap(host)
            .expect("the element owns its parsed exportparts map");
        assert_eq!(map.Len(), 3);
        assert_eq!(
            map.Get("inner"),
            Some(&["outer".into(), "badge".into()][..])
        );
        assert_eq!(map.Get("identity"), Some(&["identity".into()][..]));
        assert_eq!(map.Get("tail"), Some(&["end".into()][..]));
        assert_eq!(map.Get("bad"), None);
        assert!(document.HasPartNamesMap(host));

        let clone = document.CloneNode(host, false);
        assert_eq!(document.PartNamesMap(clone), document.PartNamesMap(host));

        document.SetAttribute(
            host,
            DOMAttribute {
                local_name: "exportparts".into(),
                value: "inner: renamed".into(),
                ..Default::default()
            },
        );
        assert_eq!(
            document.PartNamesMap(host).and_then(|map| map.Get("inner")),
            Some(&["renamed".into()][..])
        );

        document.RemoveAttributeDefault(host, "exportparts");
        assert!(!document.HasPartNamesMap(host));
        assert_eq!(
            document.PartNamesMap(host).map(DOMPartNamesMap::Len),
            Some(0)
        );
    }

    #[test]
    fn text_control_retains_placeholder_in_user_agent_shadow_tree() {
        let mut document = PersistentDocument::new();
        let input = document.CreateElement(
            DOMNamespace::kHTML,
            "input".into(),
            vec![DOMAttribute {
                local_name: "placeholder".into(),
                value: "fixture".into(),
                ..Default::default()
            }],
        );
        let placeholder = document
            .UAShadowPseudoElement(input, "-webkit-input-placeholder")
            .expect("input must retain its placeholder implementation element");
        let root = document
            .ShadowRoot(input)
            .expect("input must retain its UA shadow root");

        assert_eq!(
            document.ShadowRootMode(root),
            Some(DOMShadowRootMode::kUserAgent)
        );
        assert_eq!(document.Node(placeholder).Parent(), Some(root));
        assert_eq!(document.ContainingShadowRoot(placeholder), Some(root));
        assert_eq!(document.OwnerShadowHost(placeholder), Some(input));
        assert_eq!(
            document.ShadowPseudoId(placeholder),
            Some("-webkit-input-placeholder")
        );
        assert_eq!(document.UltimateOriginatingElement(placeholder), input);

        let clone = document.CloneNode(input, false);
        let cloned_placeholder = document
            .UAShadowPseudoElement(clone, "-webkit-input-placeholder")
            .expect("cloned input must own a distinct placeholder element");
        assert_ne!(cloned_placeholder, placeholder);
        assert_eq!(document.OwnerShadowHost(cloned_placeholder), Some(clone));

        document.RemoveAttributeDefault(input, "placeholder");
        assert!(document
            .UAShadowPseudoElement(input, "-webkit-input-placeholder")
            .is_none());
    }

    #[test]
    fn image_completion_tracks_current_style_and_pseudo_consumers() {
        use crate::style_state::StyleChange;
        fn reset(document: &mut PersistentDocument) {
            let state = document.StyleStateMut();
            state.all_dirty = false;
            state.changes.clear();
            state.impact = Default::default();
        }
        let mut document = PersistentDocument::new();
        let first = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        let parent = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        let second = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        let unrelated = document.CreateElementDefault(DOMNamespace::kHTML, "img".into());
        for node in [first, parent, unrelated] {
            document.AppendChild(document.Root(), node);
        }
        document.AppendChild(parent, second);
        let mut style = ResolvedNodeStyle::default();
        style.referenced_style_image_sources =
            ["shared", "before", "after", "first-letter", "placeholder"]
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
                .into();
        document.SetResolvedStyle(first, style);
        let mut style = ResolvedNodeStyle::default();
        style.referenced_style_image_sources = vec!["shared".to_owned()].into();
        document.SetResolvedStyle(second, style);
        document.SetResolvedStyle(unrelated, Default::default());
        reset(&mut document);
        document.SetImageResource("shared".into(), Default::default());
        assert_eq!(
            document.StyleState().changes,
            [StyleChange::Resource(first), StyleChange::Resource(second)]
                .into_iter()
                .collect()
        );
        assert!(document.StyleState().impact.layout); // Preserve replaced-image geometry invalidation.
        for source in ["before", "after", "first-letter", "placeholder"] {
            reset(&mut document);
            document.SetImageResource(source.into(), Default::default());
            assert_eq!(
                document.StyleState().changes,
                [StyleChange::Resource(first)].into_iter().collect()
            );
        }
        let mut replacement = ResolvedNodeStyle::default();
        replacement.referenced_style_image_sources = vec!["replacement".to_owned()].into();
        document.SetResolvedStyle(first, replacement);
        assert_eq!(
            document.style_image_consumers.get("shared"),
            Some(&BTreeSet::from([second]))
        );
        assert!(!document.style_image_consumers.contains_key("before"));
        document.Remove(parent); // Removing a subtree also removes descendant clients.
        assert!(!document.style_image_consumers.contains_key("shared"));
        assert!(document.ResolvedStyleFor(second).is_none());
        reset(&mut document);
        document.SetImageResource("unreferenced".into(), Default::default());
        assert!(document.StyleState().changes.is_empty());
        assert!(document.StyleState().impact.layout);
        reset(&mut document);
        document.SetImageResource("replacement".into(), Default::default());
        assert_eq!(
            document.StyleState().changes,
            [StyleChange::Resource(first)].into_iter().collect()
        );
    }

    #[test]
    fn image_failure_updates_only_consumers_and_preserves_terminal_state() {
        use crate::style_state::StyleChange;
        fn reset(document: &mut PersistentDocument) {
            let state = document.StyleStateMut();
            state.all_dirty = false;
            state.changes.clear();
            state.impact = Default::default();
        }
        let mut document = PersistentDocument::new();
        let consumer = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        let unrelated = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        for node in [consumer, unrelated] {
            document.AppendChild(document.Root(), node);
        }
        let mut style = ResolvedNodeStyle::default();
        style.referenced_style_image_sources = vec!["mask.svg#icon".to_owned()].into();
        document.SetResolvedStyle(consumer, style);
        document.SetResolvedStyle(unrelated, Default::default());
        reset(&mut document);
        assert!(document.ImageResourceFor("mask.svg#icon").is_none());
        assert!(!document.ImageResourceFailedFor("mask.svg#icon"));
        assert!(document.SetImageResourceFailed("mask.svg#icon".into()));
        assert!(document.ImageResourceFailedFor("mask.svg#icon"));
        assert!(document.ImageResourceFor("mask.svg#icon").is_none());
        assert_eq!(
            document.StyleState().changes,
            [StyleChange::Resource(consumer)].into_iter().collect()
        );
        assert!(document.StyleState().impact.layout);
        assert!(!document.ImageResourceFailedFor("mask.svg"));
        reset(&mut document);
        assert!(!document.SetImageResourceFailed("mask.svg#icon".into()));
        assert!(!document.UpdateImageResourceIntrinsicSize("mask.svg#icon", Default::default()));
        assert!(document.ImageResourceFailedFor("mask.svg#icon"));
        assert!(document.StyleState().changes.is_empty());
        assert!(!document.StyleState().impact.layout);
        document.SetImageResource(
            "mask.svg#icon".into(),
            ImageResourceMetadata {
                id: 7,
                ..Default::default()
            },
        );
        assert!(!document.ImageResourceFailedFor("mask.svg#icon"));
        assert_eq!(document.ImageResourceFor("mask.svg#icon").unwrap().id, 7);
        assert!(document.UpdateImageResourceIntrinsicSize(
            "mask.svg#icon",
            ImageResourceMetadata {
                id: 7,
                natural_width: 40.0,
                natural_height: 20.0,
                ..Default::default()
            }
        ));
        assert_eq!(
            document
                .ImageResourceFor("mask.svg#icon")
                .unwrap()
                .natural_width,
            40.0
        );
        let duplicate = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            document.SetImageResource("mask.svg#icon".into(), Default::default());
        }));
        assert!(duplicate.is_err(), "loaded source publication stays unique");
        assert_eq!(document.ImageResourceFor("mask.svg#icon").unwrap().id, 7);
        reset(&mut document);
        assert!(!document.SetImageResourceFailed("mask.svg#icon".into()));
        assert!(!document.ImageResourceFailedFor("mask.svg#icon"));
        assert_eq!(document.ImageResourceFor("mask.svg#icon").unwrap().id, 7);
        assert!(document.StyleState().changes.is_empty());
        assert!(!document.StyleState().impact.layout);
    }

    #[test]
    fn replaced_image_resource_client_follows_subtree_connectivity() {
        let mut document = PersistentDocument::new();
        let image = document.CreateElement(
            DOMNamespace::kHTML,
            "img".into(),
            vec![DOMAttribute {
                local_name: "src".into(),
                value: "hero.png".into(),
                ..Default::default()
            }],
        );
        assert!(!document.replaced_image_consumers.contains_key("hero.png"));

        document.AppendChild(document.Root(), image);
        assert_eq!(
            document.replaced_image_consumers.get("hero.png"),
            Some(&BTreeSet::from([image]))
        );

        document.Remove(image);
        assert!(!document.replaced_image_consumers.contains_key("hero.png"));

        document.AppendChild(document.Root(), image);
        assert_eq!(
            document.replaced_image_consumers.get("hero.png"),
            Some(&BTreeSet::from([image]))
        );
    }

    #[test]
    fn script_task_restores_the_same_dom_on_unwind() {
        let owner = Rc::new(RefCell::new(DOM::new()));
        let mut tree = PersistentDocument::new();
        let element = tree.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        tree.AppendChild(tree.Root(), element);
        let element_id = tree.Node(element).Id();
        let arena_handle = tree.Node(element).OwnerDocumentNode();
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            DOM::WithDocument(&owner, &mut tree, || {
                let mut live_owner = owner.borrow_mut();
                let live_tree = live_owner.GetDocumentMut();
                assert_eq!(live_tree.Node(element).OwnerDocumentNode(), arena_handle);
                live_tree.SetAttribute(
                    element,
                    DOMAttribute {
                        local_name: "class".into(),
                        value: "changed".into(),
                        ..Default::default()
                    },
                );
                panic!("script task unwound");
            });
        }));
        assert!(failure.is_err());
        assert_eq!(tree.FindNodeById(element_id), Some(element));
        assert_eq!(tree.Node(element).OwnerDocumentNode(), arena_handle);
        assert_eq!(
            tree.Node(element).FindAttribute("class").unwrap().value,
            "changed"
        );
        assert_eq!(owner.borrow().GetDocument().NodeCount(), 1);
    }
}

#[cfg(test)]
mod collection_membership_epoch_tests {
    use super::*;

    fn id(value: &str) -> DOMAttribute {
        DOMAttribute {
            local_name: "id".into(),
            value: value.into(),
            ..Default::default()
        }
    }

    #[test]
    fn tree_scope_id_index_tracks_order_renames_moves_and_shadow_scope() {
        let mut document = PersistentDocument::new();
        let root = document.Root();
        let first = document.CreateElement(DOMNamespace::kHTML, "div".into(), vec![id("same")]);
        let second = document.CreateElement(DOMNamespace::kHTML, "div".into(), vec![id("same")]);
        document.AppendChild(root, first);
        document.AppendChild(root, second);
        assert_eq!(
            document.ElementsByIdInTreeScope(root, "same"),
            vec![first, second]
        );
        assert!(document.TreeScopeContainsMultipleElementsWithId(root, "same"));

        document.InsertBefore(root, second, first);
        assert_eq!(
            document.ElementsByIdInTreeScope(root, "same"),
            vec![second, first]
        );
        document.SetAttribute(second, id("renamed"));
        assert_eq!(
            document.FindElementByIdInTreeScope(root, "same"),
            Some(first)
        );
        assert_eq!(
            document.FindElementByIdInTreeScope(root, "renamed"),
            Some(second)
        );

        document.Remove(first);
        assert_eq!(document.FindElementByIdInTreeScope(root, "same"), None);
        document.AppendChild(root, first);
        assert_eq!(
            document.FindElementByIdInTreeScope(root, "same"),
            Some(first)
        );
        document.RemoveAttributeDefault(first, "id");
        assert_eq!(document.FindElementByIdInTreeScope(root, "same"), None);

        let host = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        document.AppendChild(root, host);
        let shadow = document.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
        let shadow_child =
            document.CreateElement(DOMNamespace::kHTML, "span".into(), vec![id("renamed")]);
        document.AppendChild(shadow, shadow_child);
        assert_eq!(
            document.FindElementByIdInTreeScope(root, "renamed"),
            Some(second)
        );
        assert_eq!(
            document.FindElementByIdInTreeScope(shadow, "renamed"),
            Some(shadow_child)
        );
    }

    #[test]
    fn membership_epoch_covers_direct_tree_attributes_clone_and_whole_node_replacement() {
        let mut d = PersistentDocument::new();
        let root = d.Root();
        let before = d.CollectionMembershipKey();
        let e = d.CreateElementDefault(DOMNamespace::kHTML, "p".into());
        assert_ne!(before, d.CollectionMembershipKey());
        let before = d.CollectionMembershipKey();
        d.AppendChild(root, e);
        assert_ne!(before, d.CollectionMembershipKey());
        let attr = DOMAttribute {
            local_name: "class".into(),
            value: "one".into(),
            ..Default::default()
        };
        let before = d.CollectionMembershipKey();
        d.SetAttribute(e, attr.clone());
        assert_ne!(before, d.CollectionMembershipKey());
        let before = d.CollectionMembershipKey();
        d.SetAttribute(e, attr);
        assert_eq!(before, d.CollectionMembershipKey());
        d.RemoveAttributeDefault(e, "class");
        assert_ne!(before, d.CollectionMembershipKey());
        let replacement = d.Node(e).clone();
        let before = d.CollectionMembershipKey();
        *d.NodeMut(e) = replacement;
        assert_ne!(before, d.CollectionMembershipKey());
        d.style_state.impact = Default::default();
        let geometry = d.style_state.measurement_geometry_revision;
        let fragment = d.CreateDocumentFragment();
        let temporary = d.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        d.AppendChild(fragment, temporary);
        let text = d.AppendText(temporary, "first");
        d.AppendText(temporary, " and second");
        d.Remove(text);
        assert_eq!(d.style_state.measurement_geometry_revision, geometry);
        assert!(d.style_state.impact.IsEmpty());
        d.AppendChild(root, fragment);
        assert!(d.style_state.measurement_geometry_revision > geometry);
        assert!(d.style_state.impact.reattach);
        let before = d.CollectionMembershipKey();
        let clone = d.CloneNode(e, true);
        assert_ne!(before, d.CollectionMembershipKey());
        let before = d.CollectionMembershipKey();
        d.InsertBefore(root, clone, e);
        assert_ne!(before, d.CollectionMembershipKey());
        let before = d.CollectionMembershipKey();
        d.Remove(clone);
        assert_ne!(before, d.CollectionMembershipKey());
    }
    #[test]
    fn membership_epoch_identity_moves_with_document_swap_and_never_wraps() {
        let mut a = DOM::new();
        let mut b = PersistentDocument::new();
        a.document.collection_membership_revision = 21;
        b.collection_membership_revision = 21;
        assert_ne!(
            a.GetDocument().CollectionMembershipKey(),
            b.CollectionMembershipKey()
        );
        let ka = a.GetDocument().CollectionMembershipKey();
        let kb = b.CollectionMembershipKey();
        a.SwapDocument(&mut b);
        assert_eq!(a.GetDocument().CollectionMembershipKey(), kb);
        assert_eq!(b.CollectionMembershipKey(), ka);
        b.collection_membership_revision = u64::MAX - 1;
        assert!(b.CollectionMembershipKey().is_some());
        let root = b.Root();
        let _ = b.NodeMut(root);
        assert!(b.CollectionMembershipKey().is_none());
        let _ = b.NodeMut(root);
        assert_eq!(b.collection_membership_revision, u64::MAX);
        assert!(b.CollectionMembershipKey().is_none());
    }
}

/// Partial DOM arena storage, excluding separately allocated node payloads/styles.
#[derive(Clone, Copy, Debug, Default)]
pub struct DOMMemoryDiagnostics {
    pub arena_nodes: usize,
    pub connected_nodes: usize,
    pub detached_nodes: usize,
    pub arena_slot_capacity_bytes: usize,
}

impl PersistentDocument {
    /// The canonical per-document popup/CloseWatcher owner. Callers borrow it
    /// only for the source's individual state operations, never across events.
    pub fn PopupLifecycleState(&self) -> &crate::popup_state::PopupLifecycleState {
        &self.popup_lifecycle_state
    }
    pub fn PopupLifecycleStateMut(&mut self) -> &mut crate::popup_state::PopupLifecycleState {
        &mut self.popup_lifecycle_state
    }
}

/// MobileFriendliness_TappedBadTargets UKM optional fields (tap checker cc78-104).
/// The enclosing PersistentDocument supplies the immutable source identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DocumentTappedBadTarget {
    pub too_small: Option<bool>,
    pub close_display_edge: Option<bool>,
    pub zoomed: Option<bool>,
    pub highly_zoomed: Option<bool>,
}

#[path = "serialization_attributes.rs"]
mod serialization_attributes;

#[path = "temporal_input_shadow.rs"]
mod temporal_input_shadow;

#[path = "url_attributes.rs"]
mod url_attributes;

#[path = "html_image_element.rs"]
mod html_image_element;
pub use html_image_element::HTMLImageLayoutDisposition;

#[path = "svg_element_kind.rs"]
mod svg_element_kind;
pub use svg_element_kind::SVGElementKind;
#[path = "resize_observer_data.rs"]
mod resize_observer_data;

#[path = "intersection_observer_data.rs"]
mod intersection_observer_data;

#[path = "speculation_rules_supplement.rs"]
mod speculation_rules_supplement;
pub use speculation_rules_supplement::DocumentSpeculationRules;

#[path = "editing_font_settings.rs"]
mod editing_font_settings;

#[path = "anchor_metrics_supplement.rs"]
mod anchor_metrics_supplement;
pub use anchor_metrics_supplement::{
    DocumentAnchorMetricsOwner, ShadowIncludingAnchorMetricsCursor,
};

#[path = "anchor_viewport_supplement.rs"]
mod anchor_viewport_supplement;
pub use anchor_viewport_supplement::DocumentAnchorViewportOwner;

#[path = "static_document_security.rs"]
mod static_document_security;
pub use static_document_security::JavaScriptURLDocumentSecurity;

#[path = "vtt_node.rs"]
mod vtt_node;
pub use vtt_node::{VTTCueBackgroundSource, VTTCueBoxSource, VTTElementSource, VTTTextStyleSource};

pub(crate) use vtt_node::VTTLifecycleChange;

#[path = "navigation_api_window.rs"]
mod navigation_api_window;
pub use navigation_api_window::WindowNavigationApiOwner;

#[path = "navigation_optional_state.rs"]
mod navigation_optional_state;
pub use navigation_optional_state::{
    NavigationPhase, NavigationStateSupplement, NavigationTraverseType, SkeletonDocument,
    SkeletonLoaderSupplement,
};

pub use view_transition_participation::NavigationPreviewRequired;

#[path = "markup_parser_support.rs"]
mod markup_parser_support;

#[path = "node_traversal.rs"]
mod node_traversal;

#[path = "paint_timing_facts.rs"]
mod paint_timing_facts;

impl PersistentDocument {
    /// The embedding supplies the source InputType owner before parsing/input.
    /// Existing controls are initialized once when a pre-parsed DOM is bound.
    pub fn InstallInputValueSanitizer(
        &mut self,
        sanitizer: Rc<dyn crate::input_value::DOMInputValueSanitizer>,
    ) {
        if self
            .input_value_sanitizer
            .as_ref()
            .is_some_and(|old| Rc::ptr_eq(old, &sanitizer))
        {
            return;
        }
        self.input_value_sanitizer = Some(sanitizer);
        let controls: Vec<_> = (0..self.arena.len())
            .filter(|&i| self.InputType(i).is_some())
            .collect();
        for control in controls {
            self.InitializeInputValue(control);
        }
    }
    fn SanitizeInputValue(&self, node: usize, proposed: Rc<[u16]>) -> Rc<[u16]> {
        let (Some(kind), Some(sanitizer)) =
            (self.InputType(node), self.input_value_sanitizer.as_ref())
        else {
            return proposed;
        };
        let element = &self.arena[node];
        let attribute = |name: &str| {
            element
                .AttributeValueUTF16(name)
                .unwrap_or_else(|| Rc::from([]))
        };
        let minimum = attribute("min");
        let maximum = attribute("max");
        let value_attribute = attribute("value");
        sanitizer.SanitizeValue(crate::input_value::DOMInputValueRequest {
            kind,
            proposed,
            minimum: &minimum,
            maximum: &maximum,
            value_attribute: &value_attribute,
            step: element
                .FindAttribute("step")
                .map_or("", |a| a.value.as_str()),
            multiple: element.FindAttribute("multiple").is_some(),
        })
    }
    fn NotifyInputValueCommitted(&self, node: usize, value: &[u16]) {
        if let (Some(kind), Some(sanitizer)) =
            (self.InputType(node), self.input_value_sanitizer.as_ref())
        {
            sanitizer.DidCommitValue(
                self.NodeHandle(node),
                kind,
                value,
                self.ControlValueRevision(node),
            );
        }
    }
    fn InitializeInputValue(&mut self, node: usize) {
        let (Some(kind), Some(sanitizer)) =
            (self.InputType(node), self.input_value_sanitizer.as_ref())
        else {
            return;
        };
        if sanitizer.ValueMode(kind) != crate::input_value::DOMInputValueMode::Value {
            self.control_value_state.remove(&self.arena[node].id);
            return;
        }
        let raw = self
            .control_value_state
            .get(&self.arena[node].id)
            .map(|value| value.Owner())
            .unwrap_or_else(|| {
                self.InputValueAttributeUTF16(node)
                    .unwrap_or_else(|| Rc::from([]))
            });
        let value = self.SanitizeInputValue(node, raw);
        self.AdvanceControlValueRevision(node);
        self.control_value_state.insert(
            self.arena[node].id,
            foundation::NullableUTF16String::FromOwner(value.clone()),
        );
        self.NotifyInputValueCommitted(node, &value);
        self.SetControlNeedsValidityCheck(node);
    }
    fn InputValueAttributeChanged(&mut self, node: usize, name: &str, namespace: &str) {
        if !namespace.is_empty() {
            return;
        }
        let Some(kind) = self.InputType(node) else {
            return;
        };
        let initialize = match name {
            "value" => !self.HasDirtyControlValue(node),
            "type" => true,
            "min" | "max" | "step" => kind == DOMInputType::kRange,
            "multiple" => kind == DOMInputType::kEmail,
            _ => false,
        };
        if initialize {
            self.InitializeInputValue(node);
        }
    }
}
