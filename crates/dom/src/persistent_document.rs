#![allow(non_snake_case)]

use crate::error::{invalid_argument, logic_error};
use crate::ImageResourceMetadata;
use cssom::{CSSDeclaration, CSSOMAdoptionError, CSSOMEvent, CSSOMMutation, CSSStyleSheet, CSSOM};
use font_engine::text::native::bidi_paragraph::BidiParagraph;
use foundation::Persistent;
use foundation::{StringView, TextDirection};
use layoutng_assembly::internal::layout_input::{ComputedStyle, Offset};
use layoutng_style::style::computed_style::ComputedStyle as NativeComputedStyle;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

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
#[derive(Clone, Debug)]
pub struct DOMNode {
    id: u64,
    node_type: DOMNodeType,
    namespace: DOMNamespace,
    name: String,
    data: String,
    attributes: Vec<DOMAttribute>,
    owner_document_handle: Option<DOMOwnerHandle>,
    parent: Option<usize>,
    children: Vec<usize>,
}

impl DOMNode {
    // cpp: dom/document.h:45-53
    pub fn Type(&self) -> DOMNodeType {
        self.node_type
    }
    pub fn Namespace(&self) -> DOMNamespace {
        self.namespace
    }
    pub fn Id(&self) -> u64 {
        self.id
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
    pub fn OwnerDocumentNode(&self) -> Option<DOMOwnerHandle> {
        self.owner_document_handle
    }
    pub fn Parent(&self) -> Option<usize> {
        self.parent
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
                },
                Self::Image {
                    resource_id: b_id,
                    shader: b_shader,
                    natural_width: b_width,
                    natural_height: b_height,
                    resolution_scale: b_scale,
                },
            ) => {
                a_id == b_id
                    && a_shader == b_shader
                    && a_width == b_width
                    && a_height == b_height
                    && a_scale == b_scale
            }
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct PseudoElement {
    pub style: ComputedStyle,
    /// Owns the immutable Chromium style referenced by `style.native_style`.
    pub native_style: Persistent<NativeComputedStyle>,
    pub text: String,
    /// Ordered non-alt generated content. Chromium creates one anonymous
    /// layout child per ContentData node; `text` remains the concatenated view
    /// used by the current first-letter adapter.
    pub content: Vec<PseudoContent>,
    pub display_contents: bool,
    /// A generated ::before/::after whose computed display is list-item owns
    /// its own ::marker pseudo. The box keeps the public projection while its
    /// native style owns the matching cached pseudo style.
    pub marker: Option<Box<PseudoElement>>,
}

impl PartialEq for PseudoElement {
    fn eq(&self, other: &Self) -> bool {
        self.style == other.style
            && self.native_style.Get() == other.native_style.Get()
            && self.text == other.text
            && self.content == other.content
            && self.display_contents == other.display_contents
            && self.marker == other.marker
    }
}

impl Default for PseudoElement {
    fn default() -> Self {
        Self {
            style: ComputedStyle::default(),
            native_style: Persistent::default(),
            text: String::new(),
            content: Vec::new(),
            display_contents: false,
            marker: None,
        }
    }
}

impl PseudoElement {
    pub fn SetNativeStyle(&mut self, native_style: Persistent<NativeComputedStyle>) {
        self.style.native_style = native_style.Get();
        self.native_style = native_style;
    }
}

#[derive(Clone)]
pub struct ResolvedNodeStyle {
    pub style: ComputedStyle,
    /// Roots the immutable Chromium style used by the live LayoutObject tree.
    /// `style.native_style` is a borrowed pointer into this owner.
    pub native_style: Persistent<NativeComputedStyle>,
    pub generates_box: bool,
    pub own_generates_box: bool,
    pub own_display_contents: bool,
    pub display_contents: bool,
    pub custom_properties: std::sync::Arc<HashMap<String, Option<String>>>,
    /// StyleBuilder's explicit-inheritance dependency after variable resolution.
    /// None denotes an external/old style without a dependency proof.
    pub has_explicit_inheritance: Option<bool>,
    pub before: Option<PseudoElement>,
    pub after: Option<PseudoElement>,
    pub first_letter: Option<PseudoElement>,
    pub placeholder: Option<PseudoElement>,
    /// Generated list marker style owned by the originating list item. The
    /// native object is also cached on `native_style` under kPseudoIdMarker;
    /// this projection is the retained DOM-to-layout edge.
    pub marker: Option<PseudoElement>,
}

impl PartialEq for ResolvedNodeStyle {
    fn eq(&self, other: &Self) -> bool {
        self.style == other.style
            && self.native_style.Get() == other.native_style.Get()
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
            style: ComputedStyle::default(),
            native_style: Persistent::default(),
            generates_box: true,
            own_generates_box: true,
            own_display_contents: false,
            display_contents: false,
            custom_properties: Default::default(),
            has_explicit_inheritance: None,
            before: None,
            after: None,
            first_letter: None,
            placeholder: None,
            marker: None,
        }
    }
}

impl ResolvedNodeStyle {
    pub fn SetNativeStyle(&mut self, native_style: Persistent<NativeComputedStyle>) {
        self.style.native_style = native_style.Get();
        self.native_style = native_style;
    }

    /// URLs referenced by the canonical native computed style, including
    /// generated-content pseudos. Resource loading consumes this dependency
    /// view without re-parsing CSS or flattening StyleImage into DOM state.
    pub fn ReferencedStyleImageSources(&self) -> Vec<String> {
        StyleImageSources(self)
    }
}

fn NativeStyleImageSources(style: &NativeComputedStyle, output: &mut Vec<String>) {
    use layoutng_style::style::{
        content_data::ImageContentData, fill_layer::FillLayer, style_fetched_image::FetchedImage,
        style_image::StyleImage,
    };
    fn append(image: *mut StyleImage, output: &mut Vec<String>) {
        let Some(image) = (unsafe { image.as_ref() }) else {
            return;
        };
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
        if let Some(native) = unsafe { pseudo.native_style.Get().as_ref() } {
            NativeStyleImageSources(native, output);
        }
        if let Some(marker) = pseudo.marker.as_deref() {
            append_pseudo(marker, output);
        }
    }
    if let Some(native) = unsafe { style.native_style.Get().as_ref() } {
        NativeStyleImageSources(native, &mut output);
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

// cpp: dom/document.h:207-232
// Re-exported as dom::Document. The older parser's narrow tree is exposed
// separately as dom::ParsedDocument until its callers migrate.
pub struct PersistentDocument {
    arena: Vec<DOMNode>,
    arena_id: u64,
    collection_membership_revision: u64,
    root: usize,
    next_id: u64,
    compatibility: CompatibilityMode,
    // cpp: Settings::GetTextAreasAreResizable. Resize::ApplyValue consumes
    // this document setting for the UA-only -internal-textarea-auto value.
    text_areas_are_resizable: bool,
    // cpp: Settings::GetDefaultFontSize, consumed by
    // LayoutThemeFontProvider::DefaultFontSize for system-font shorthands.
    default_font_size: f32,
    document_types: HashMap<u64, DOMDocumentType>,
    template_contents: HashMap<u64, usize>,
    // cpp: element.h:1129-1132; shadow_root.h:97-110.  Shadow roots are not
    // ordinary children of their hosts.  These two maps retain the two sides
    // of Blink's Element/ShadowRoot ownership without changing Parent().
    shadow_roots: HashMap<usize, (usize, DOMShadowRootMode, DOMSlotAssignmentMode)>,
    shadow_hosts: HashMap<usize, usize>,
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
    text_control_placeholders: HashMap<usize, usize>,
    text_control_inner_editors: HashMap<usize, usize>,
    text_control_inner_editor_hosts: HashMap<usize, usize>,
    text_control_containers: HashMap<usize, usize>,
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
    image_resources: HashMap<String, ImageResourceMetadata>,
    // SVGImage/ImageResourceContent type is resource-owned metadata. Style
    // needs it to apply image-set resolution only to bitmap images, matching
    // StyleFetchedImage::ApplyImageResolution without depending on Decode.
    document_image_resource_ids: HashSet<u64>,
    responsive_image_selections: HashMap<usize, ResponsiveImageSelection>,
    // HTMLImageElement::last_auto_sizes_width_. Kept on the DOM/image-loader
    // owner because it survives source reselection but is reset by removal.
    responsive_image_auto_sizes: HashMap<usize, f64>,
    scroll_offsets: HashMap<u64, Offset>,
    resolved_styles: HashMap<u64, std::sync::Arc<ResolvedNodeStyle>>,
    // StyleFetchedImage::AddClient/RemoveClient: only resolved CSS consumers
    // need their background/mask resource IDs refreshed on image completion.
    style_image_consumers: HashMap<String, BTreeSet<usize>>,
    style_state: crate::style_state::StyleState,
    control_checked_state: HashMap<u64, bool>,
    // cpp: HTMLOptionElement::is_selected_/is_dirty_. Selectedness is live
    // state; the selected content attribute is only defaultSelected and stops
    // driving selectedness after the IDL selection has become dirty.
    option_selected_state: HashMap<u64, bool>,
    option_dirty_state: HashSet<u64>,
    // cpp: html_input_element.h is_indeterminate_. This is live IDL state,
    // independent of the checked content attribute and retained across input
    // type changes.
    control_indeterminate_state: HashMap<u64, bool>,
    control_value_state: HashMap<u64, String>,
    // cpp: HTMLFormControlElement::autofill_state_. This is live form state,
    // separate from attributes and from the suggested text rendered in the UA
    // shadow tree.
    control_autofill_state: HashMap<u64, DOMAutofillState>,
    // cpp: document.h TopLayerElements; html_dialog_element.h is_modal_;
    //      popover_data.h visibility_state_. Attribute `open` is deliberately
    // not used as modal or top-layer state.
    top_layer_elements: Vec<u64>,
    modal_dialogs: HashSet<u64>,
    open_popovers: HashSet<u64>,
    // cpp: fullscreen/fullscreen.cc FullscreenParamsMap plus Document's top
    // layer ordering. Every entry owns the fullscreen flag; the last entry is
    // the document's current fullscreen element.
    fullscreen_element_stack: Vec<u64>,
    // cpp: document.h css_target_. The fragment-navigation owner updates this
    // retained Document state; selector matching only observes it.
    css_target: Option<u64>,
    // SVGSVGElement::view_spec_. Only an isolated SVG document-image owner
    // writes this state, immediately before its Style/Layout/Paint lifecycle.
    svg_image_view_root: Option<u64>,
    svg_image_view_spec: Option<SVGImageViewSpec>,
    // cpp: svg/svg_element_rare_data.h corresponding_element_. SVG <use>
    // instance-tree construction owns this relation; selector matching only
    // observes it. Ordinary SVG elements have no corresponding element.
    corresponding_svg_elements: HashMap<u64, u64>,
}

impl PersistentDocument {
    // cpp: dom/document.cc:36-39
    fn new() -> Self {
        let mut document = Self {
            arena: Vec::new(),
            arena_id: NEXT_ARENA_ID.fetch_add(1, Ordering::Relaxed),
            collection_membership_revision: 0,
            root: 0,
            next_id: 1,
            compatibility: CompatibilityMode::kNoQuirks,
            text_areas_are_resizable: true,
            default_font_size: 16.0,
            document_types: HashMap::new(),
            template_contents: HashMap::new(),
            shadow_roots: HashMap::new(),
            shadow_hosts: HashMap::new(),
            manually_assigned_nodes: HashMap::new(),
            manually_assigned_slots: HashMap::new(),
            shadow_pseudo_ids: HashMap::new(),
            ultimate_originating_elements: HashMap::new(),
            ua_shadow_pseudo_elements: HashMap::new(),
            text_control_placeholders: HashMap::new(),
            text_control_inner_editors: HashMap::new(),
            text_control_inner_editor_hosts: HashMap::new(),
            text_control_containers: HashMap::new(),
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
            document_image_resource_ids: HashSet::new(),
            responsive_image_selections: HashMap::new(),
            responsive_image_auto_sizes: HashMap::new(),
            scroll_offsets: HashMap::new(),
            resolved_styles: HashMap::new(),
            style_image_consumers: HashMap::new(),
            style_state: Default::default(),
            control_checked_state: HashMap::new(),
            option_selected_state: HashMap::new(),
            option_dirty_state: HashSet::new(),
            control_indeterminate_state: HashMap::new(),
            control_value_state: HashMap::new(),
            control_autofill_state: HashMap::new(),
            top_layer_elements: Vec::new(),
            modal_dialogs: HashSet::new(),
            open_popovers: HashSet::new(),
            fullscreen_element_stack: Vec::new(),
            css_target: None,
            svg_image_view_root: None,
            svg_image_view_spec: None,
            corresponding_svg_elements: HashMap::new(),
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
    pub fn Root(&self) -> usize {
        self.root
    }
    // Identity adapter for C++ &Document::Root(), which is distinct from
    // Root().OwnerDocumentNode(): a document node has no owner document.
    pub fn RootHandle(&self) -> DOMOwnerHandle {
        DOMOwnerHandle {
            arena_id: self.arena_id,
            node_id: self.arena[self.root].id,
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
    pub fn GetCompatibilityMode(&self) -> CompatibilityMode {
        self.compatibility
    }

    // cpp: settings.json5 TextAreasAreResizable; Settings getter/setter.
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
    pub fn SetCompatibilityMode(&mut self, mode: CompatibilityMode) {
        self.compatibility = mode;
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
        if !self.arena[element].IsElement("select", DOMNamespace::kHTML) {
            return false;
        }
        let multiple = self.HasBooleanHTMLAttribute(element, "multiple");
        let size_attribute = self.arena[element].FindAttribute("size");
        let size = size_attribute
            .and_then(|attribute| Self::ParseHTMLNonNegativeInteger(&attribute.value))
            .unwrap_or(0);
        if multiple {
            size_attribute.is_some() && size == 1
        } else {
            size <= 1
        }
    }

    // cpp: html_select_element.cc:2155-2195;
    //      html_option_element.h:87-89,219-229. The association follows the
    // DOM ancestor chain and stops at the same related-part barriers used by
    // HTMLSelectElement::WalkAncestorsForRelatedParts.
    pub fn OptionOwnerSelect(&self, option: usize) -> Option<usize> {
        if !self.arena[option].IsElement("option", DOMNamespace::kHTML) {
            return None;
        }
        let mut ancestor = self.arena[option].Parent();
        let mut optgroup = None;
        while let Some(index) = ancestor {
            let node = &self.arena[index];
            ancestor = node.Parent();
            if node.IsElement("option", DOMNamespace::kHTML)
                || node.IsElement("hr", DOMNamespace::kHTML)
                || node.IsElement("datalist", DOMNamespace::kHTML)
            {
                return None;
            }
            if node.IsElement("optgroup", DOMNamespace::kHTML) {
                if optgroup.is_some() {
                    return None;
                }
                optgroup = Some(index);
                continue;
            }
            if node.IsElement("select", DOMNamespace::kHTML) {
                return Some(index);
            }
            if node.Type() == DOMNodeType::kDocumentFragment {
                return None;
            }
        }
        None
    }

    // cpp: html_select_element.h:289; html_select_element.cc GetOptionList().
    // Retain source tree order while using OptionOwnerSelect as the single
    // association owner for nested customizable-select markup.
    pub fn SelectOptionNodeIds(&self, select: usize) -> Vec<u64> {
        fn collect(
            document: &PersistentDocument,
            root: usize,
            select: usize,
            options: &mut Vec<u64>,
        ) {
            for &child in document.arena[root].Children() {
                if document.OptionOwnerSelect(child) == Some(select) {
                    options.push(document.arena[child].Id());
                }
                collect(document, child, select, options);
            }
        }
        let mut options = Vec::new();
        collect(self, select, select, &mut options);
        options
    }

    // cpp: select_type.cc:717-722,2008-2010. MenuListSelectType assigns the
    // first element child when it is an HTMLButtonElement; ListBoxSelectType
    // never exposes a slotted button. This query intentionally does not invent
    // the UA shadow slot whose producer is absent from the compact DOM.
    pub fn SelectSlottedButton(&self, select: usize) -> Option<usize> {
        if !self.SelectUsesMenuList(select) {
            return None;
        }
        self.arena[select]
            .Children()
            .iter()
            .copied()
            .find(|&child| self.arena[child].Type() == DOMNodeType::kElement)
            .filter(|&child| self.arena[child].IsHTMLElement("button"))
    }

    // cpp: html_input_element.cc:2064-2075. The pseudo class observes the
    // actual datalist association, not merely a non-empty `list` attribute.
    pub fn InputHasDataList(&self, element: usize) -> bool {
        let Some(input_type) = self.InputType(element) else {
            return false;
        };
        if !input_type.ShouldRespectListAttribute() {
            return false;
        }
        let Some(id) = self.arena[element]
            .FindAttribute("list")
            .map(|attribute| attribute.value.as_str())
            .filter(|id| !id.is_empty())
        else {
            return false;
        };
        self.FindElementByIdInTreeScope(element, id)
            .is_some_and(|target| self.arena[target].IsElement("datalist", DOMNamespace::kHTML))
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

    fn FindElementByIdInTreeScope(&self, element: usize, id: &str) -> Option<usize> {
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
        find(self, self.TreeScopeRoot(element), id)
    }

    fn IsLabelableElement(&self, element: usize) -> bool {
        let node = &self.arena[element];
        if node.Namespace() != DOMNamespace::kHTML {
            return false;
        }
        match node.Name() {
            "button" | "meter" | "output" | "progress" | "select" | "textarea" => true,
            "input" => self.InputType(element) != Some(DOMInputType::kHidden),
            _ => false,
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
        let node = &self.arena[element];
        if node.IsElement("option", DOMNamespace::kHTML) {
            if self.HasBooleanHTMLAttribute(element, "disabled")
                || self
                    .NearestAncestorElementNamed(element, "optgroup")
                    .is_some_and(|group| self.HasBooleanHTMLAttribute(group, "disabled"))
            {
                return true;
            }
            return self
                .NearestAncestorElementNamed(element, "select")
                .is_some_and(|select| self.IsActuallyDisabledFormControl(select));
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
        self.BumpCollectionMembershipRevision();
        self.InvalidateNodeStyle(index);
        self.MarkStyleElementTextDirty(index);
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
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
        self.arena.push(DOMNode {
            id,
            node_type,
            namespace,
            name,
            data,
            attributes,
            owner_document_handle,
            parent: None,
            children: Vec::new(),
        });
        index
    }

    // cpp: dom/document.cc:53-58
    pub fn AdoptNode(&mut self, node: usize, document_node: usize) {
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
        self.arena[node].owner_document_handle = Some(owner);
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
        let element = self.Allocate(
            DOMNodeType::kElement,
            namespace,
            name,
            String::new(),
            attributes,
        );
        if self.arena[element].IsElement("template", DOMNamespace::kHTML) {
            self.EnsureTemplateContents(element);
        }
        self.SyncPartNamesMap(element, false);
        self.EnsureTextControlUAShadowTree(element);
        self.EnsureColorUAShadowTree(element);
        self.EnsureFileUAShadowTree(element);
        self.EnsureMeterUAShadowTree(element);
        self.EnsureProgressUAShadowTree(element);
        self.RecomputeDirectionality();
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
    pub fn CreateText(&mut self, data: String) -> usize {
        self.Allocate(
            DOMNodeType::kText,
            DOMNamespace::kNone,
            "#text".into(),
            data,
            Vec::new(),
        )
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
        let original = self.arena[source].clone();
        let clone = self.Allocate(
            original.node_type,
            original.namespace,
            original.name,
            original.data,
            original.attributes,
        );
        if let Some(owner_document) = original.owner_document_handle {
            self.AdoptNodeByHandle(clone, owner_document);
        }
        self.SyncPartNamesMap(clone, false);
        if self.arena[source].IsElement("template", DOMNamespace::kHTML) {
            if let Some(contents) = self.TemplateContents(source) {
                let cloned_contents = self.EnsureTemplateContents(clone);
                if deep {
                    let children = self.arena[contents].children.clone();
                    for child in children {
                        let cloned_child = self.CloneNode(child, true);
                        self.AppendChild(cloned_contents, cloned_child);
                    }
                }
            }
        }
        self.EnsureTextControlUAShadowTree(clone);
        self.EnsureColorUAShadowTree(clone);
        self.EnsureFileUAShadowTree(clone);
        self.EnsureMeterUAShadowTree(clone);
        self.EnsureProgressUAShadowTree(clone);
        if deep {
            let children = self.arena[source].children.clone();
            for child in children {
                let cloned_child = self.CloneNode(child, true);
                self.AppendChild(clone, cloned_child);
            }
        }
        let source_id = self.arena[source].id;
        let clone_id = self.arena[clone].id;
        if let Some(&checked) = self.control_checked_state.get(&source_id) {
            self.control_checked_state.insert(clone_id, checked);
        }
        if let Some(value) = self.control_value_state.get(&source_id).cloned() {
            self.control_value_state.insert(clone_id, value);
        }
        // The UA subtree is created before live control state is copied. Bring
        // its implementation element back in sync with the cloned value.
        if self.arena[clone].IsElement("input", DOMNamespace::kHTML)
            || self.arena[clone].IsElement("textarea", DOMNamespace::kHTML)
        {
            self.SyncTextControlPlaceholderElement(clone);
            self.SyncSearchCancelButtonVisibility(clone);
            self.SyncColorSwatchAppearance(clone);
        }
        self.RecomputeDirectionality();
        clone
    }

    fn EnsureTextControlUAShadowTree(&mut self, element: usize) {
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
        );
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
        self.SyncTextControlPlaceholderElement(element);
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

    // cpp: TextControlInnerEditorElement::SetVisibility. SuggestedValue's
    // nullness owns editor visibility; an empty live value needs no opacity
    // suppression because it contributes no painted text.
    pub fn TextControlInnerEditorIsVisible(&self, editor: usize) -> bool {
        self.TextControlInnerEditorHost(editor).is_some_and(|host| {
            self.SuggestedValue(host).is_none() || self.ControlValue(host).is_empty()
        })
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
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
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
                self.style_state
                    .impact
                    .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
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
                self.style_state
                    .impact
                    .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
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
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
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
        self.template_contents
            .insert(self.arena[template_element].id, fragment);
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
        self.shadow_roots
            .insert(host, (root, mode, slot_assignment));
        self.shadow_hosts.insert(root, host);
        self.RecomputeDirectionality();
        self.InvalidateNodeStyle(host);
        self.style_state.all_dirty = true;
        root
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

    // cpp: node.cc:2121-2149.  A node's TreeScope root is the first
    // document/document-fragment root reached through ordinary Parent links;
    // the ShadowRoot -> host edge belongs only to shadow-including traversal.
    pub fn TreeScopeRoot(&self, node: usize) -> usize {
        let mut root = node;
        while let Some(parent) = self.arena[root].parent {
            root = parent;
        }
        root
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

    // cpp: flat_tree_traversal.cc:104-123,161-191. This is the element-only
    // parent used by UserActionElementTraversal. Assigned light children walk
    // through their slot, inactive light children and inactive slot fallback
    // roots have no flat-tree parent, and a shadow-root child walks to its
    // host.
    pub fn FlatTreeParentElement(&self, node: usize) -> Option<usize> {
        if let Some(slot) = self.AssignedSlot(node) {
            return Some(slot);
        }
        let parent = self.arena[node].parent?;
        if self.arena[parent].node_type == DOMNodeType::kElement {
            if self.ShadowRoot(parent).is_some() {
                return None;
            }
            if self.IsSlotSupportingAssignment(parent) && !self.AssignedNodes(parent).is_empty() {
                return None;
            }
            return Some(parent);
        }
        self.ShadowHost(parent)
    }

    pub fn IsInShadowTree(&self, node: usize) -> bool {
        self.ContainingShadowRoot(node).is_some()
    }

    // cpp: html/media/html_media_element.h:145. HTMLMediaElement is the
    // shared retained DOM owner for HTMLAudioElement and HTMLVideoElement.
    pub fn IsMediaElement(&self, node: usize) -> bool {
        self.arena[node].Namespace() == DOMNamespace::kHTML
            && matches!(self.arena[node].Name(), "audio" | "video")
    }

    // cpp: html_select_element.cc:2164-2186,2287-2294. Inputs are counted for
    // their nearest associated select. option/hr/datalist and invalid nested
    // optgroup structures terminate that association walk.
    pub fn NumDescendantInputs(&self, select: usize) -> usize {
        if !self.arena[select].IsHTMLElement("select") {
            return 0;
        }
        fn count(document: &PersistentDocument, node: usize, in_optgroup: bool) -> usize {
            let current = &document.arena[node];
            if current.Namespace() == DOMNamespace::kHTML {
                match current.Name() {
                    "select" | "option" | "hr" | "datalist" => return 0,
                    "optgroup" if in_optgroup => return 0,
                    "input" => return 1,
                    _ => {}
                }
            }
            let in_optgroup = in_optgroup || current.IsHTMLElement("optgroup");
            current
                .Children()
                .iter()
                .map(|&child| count(document, child, in_optgroup))
                .sum()
        }
        self.arena[select]
            .Children()
            .iter()
            .map(|&child| count(self, child, false))
            .sum()
    }

    // cpp: style_engine.cc:290-316,648-725. Each connected TreeScope owns a
    // separate active stylesheet collection. ShadowRoot is connected through
    // its host edge, never through Parent().
    pub fn IsConnectedToDocument(&self, node: usize) -> bool {
        let mut root = self.TreeScopeRoot(node);
        loop {
            if root == self.root {
                return true;
            }
            let Some(host) = self.ShadowHost(root) else {
                return false;
            };
            root = self.TreeScopeRoot(host);
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
        let host = self.arena[node].parent?;
        let root = self.ShadowRoot(host)?;
        if self.ShadowRootSlotAssignment(root) == Some(DOMSlotAssignmentMode::kManual) {
            let slot = self.ManuallyAssignedSlot(node)?;
            return (self.ContainingShadowRoot(slot) == Some(root)).then_some(slot);
        }
        let requested_name = self.arena[node]
            .FindAttribute("slot")
            .map_or("", |attribute| attribute.value.as_str());
        let mut stack = vec![root];
        while let Some(current) = stack.pop() {
            let candidate = &self.arena[current];
            if candidate.IsHTMLElement("slot")
                && candidate
                    .FindAttribute("name")
                    .map_or("", |attribute| attribute.value.as_str())
                    == requested_name
            {
                return Some(current);
            }
            stack.extend(candidate.children.iter().rev().copied());
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
            self.RecordNonAppendChildrenChange(changed_slot);
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

    pub fn AssignedNodes(&self, slot: usize) -> Vec<usize> {
        let Some(root) = self.ContainingShadowRoot(slot) else {
            return Vec::new();
        };
        let Some(host) = self.ShadowHost(root) else {
            return Vec::new();
        };
        match self.ShadowRootSlotAssignment(root) {
            Some(DOMSlotAssignmentMode::kManual) => self
                .ManuallyAssignedNodes(slot)
                .iter()
                .copied()
                .filter(|&node| {
                    self.arena[node].parent == Some(host)
                        && self.ManuallyAssignedSlot(node) == Some(slot)
                })
                .collect(),
            Some(DOMSlotAssignmentMode::kNamed) => self.arena[host]
                .children
                .iter()
                .copied()
                .filter(|&node| self.AssignedSlot(node) == Some(slot))
                .collect(),
            None => Vec::new(),
        }
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

    fn ComputeDirectionality(
        &self,
        element: usize,
        cache: &mut HashMap<usize, TextDirection>,
    ) -> TextDirection {
        if let Some(direction) = cache.get(&element).copied() {
            return direction;
        }
        let parent_direction = self
            .ParentOrShadowHostElement(element)
            .map(|parent| self.ComputeDirectionality(parent, cache))
            .unwrap_or(TextDirection::kLtr);
        let direction = match self.DirectionAttribute(element) {
            Some(value) if value.eq_ignore_ascii_case("ltr") => TextDirection::kLtr,
            Some(value) if value.eq_ignore_ascii_case("rtl") => TextDirection::kRtl,
            Some(_) => self
                .ResolveAutoDirectionality(element, cache)
                .unwrap_or(TextDirection::kLtr),
            None if self.arena[element].IsHTMLElement("bdi") => self
                .ResolveAutoDirectionality(element, cache)
                .unwrap_or(TextDirection::kLtr),
            None if self.arena[element].IsHTMLElement("input")
                && self.arena[element]
                    .FindAttribute("type")
                    .is_some_and(|attribute| attribute.value.eq_ignore_ascii_case("tel")) =>
            {
                TextDirection::kLtr
            }
            None => parent_direction,
        };
        cache.insert(element, direction);
        direction
    }

    fn RecomputeDirectionality(&mut self) {
        let mut next = HashMap::new();
        for element in 0..self.arena.len() {
            if self.arena[element].Type() == DOMNodeType::kElement {
                self.ComputeDirectionality(element, &mut next);
            }
        }
        let changed = next
            .iter()
            .filter_map(|(&element, &direction)| {
                (self.cached_directionality.get(&element).copied() != Some(direction))
                    .then_some(element)
            })
            .collect::<Vec<_>>();
        self.cached_directionality = next;
        for element in changed {
            self.InvalidateNodeStyle(element);
        }
    }

    pub fn CachedDirectionality(&self, element: usize) -> TextDirection {
        self.cached_directionality
            .get(&element)
            .copied()
            .unwrap_or(TextDirection::kLtr)
    }

    // cpp: dom/document.cc:153-162
    pub fn Remove(&mut self, child: usize) {
        let Some(parent) = self.arena[child].parent else {
            return;
        };
        // HTMLSelectElement::OptionRemoved runs while OwnerSelectElement is
        // still observable. Capture associations that this detach breaks;
        // removing an entire select keeps its internal option ownership.
        let mut removed_nodes = Vec::new();
        let mut pending = vec![child];
        while let Some(node) = pending.pop() {
            removed_nodes.push(node);
            pending.extend_from_slice(&self.arena[node].children);
        }
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
        self.arena[parent].children.remove(position);
        self.arena[child].parent = None;
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
            let id = self.arena[node].id;
            self.modal_dialogs.remove(&id);
            self.open_popovers.remove(&id);
            self.top_layer_elements.retain(|candidate| *candidate != id);
            self.RemoveResolvedStyle(node);
            self.responsive_image_selections.remove(&node);
            self.responsive_image_auto_sizes.remove(&node);
            detached.extend_from_slice(&self.arena[node].children);
        }
        self.RecordNonAppendChildrenChange(parent);
    }

    // cpp: dom/document.cc:164-175
    pub fn AppendChild(&mut self, parent: usize, child: usize) {
        if parent == child {
            invalid_argument("a DOM node cannot contain itself");
        }
        let mut ancestor = Some(parent);
        while let Some(index) = ancestor {
            if index == child {
                invalid_argument("DOM insertion would create a cycle");
            }
            ancestor = self.arena[index].parent;
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
        self.arena[parent].children.push(child);
        self.arena[child].parent = Some(parent);
        self.OptionSubtreeInserted(child);
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
    }

    // cpp: dom/document.cc:177-193
    pub fn InsertBefore(&mut self, parent: usize, child: usize, next_child: usize) {
        if self.arena[next_child].parent != Some(parent) {
            invalid_argument("DOM insertion point is not a child");
        }
        if child == next_child {
            return;
        }
        let mut ancestor = Some(parent);
        while let Some(index) = ancestor {
            if index == child {
                invalid_argument("DOM insertion would create a cycle");
            }
            ancestor = self.arena[index].parent;
        }
        self.Remove(child);
        let position = self.arena[parent]
            .children
            .iter()
            .position(|&index| index == next_child)
            .expect("DOM insertion point is not a child");
        self.arena[parent].children.insert(position, child);
        self.arena[child].parent = Some(parent);
        self.OptionSubtreeInserted(child);
        self.StyleSheetConnectivityChanged(child);
        self.RecordNonAppendChildrenChange(parent);
        // InsertBefore attaches the same fresh/stale-free subtree as
        // AppendChild. Record it explicitly so style invalidation resolves the
        // inserted branch and treats existing siblings through selector
        // invalidation sets instead of rescanning the whole parent subtree.
        if !self.style_state.all_dirty && self.arena[child].node_type == DOMNodeType::kElement {
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
    }

    // cpp: dom/document.cc:195-204
    pub fn AppendText(&mut self, parent: usize, data: &str) -> usize {
        if let Some(&last) = self.arena[parent].children.last() {
            if self.arena[last].node_type == DOMNodeType::kText {
                let was_empty = self.arena[last].data.is_empty();
                self.arena[last].data.push_str(data);
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
                self.arena[previous].data.push_str(data);
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
    pub fn SetAttribute(&mut self, element: usize, attribute: DOMAttribute) {
        if self.arena[element].node_type != DOMNodeType::kElement {
            invalid_argument("attributes require an element");
        }
        let existing = self.arena[element].attributes.iter().position(|candidate| {
            candidate.local_name == attribute.local_name
                && candidate.namespace_uri == attribute.namespace_uri
        });
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
                "checked" | "form" | "id" | "name" | "type"
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
        let changed = !existing.is_some_and(|i| self.arena[element].attributes[i] == attribute);
        if std::env::var_os("BROWSER_PROFILE_STYLE_INVALIDATION").is_some() {
            eprintln!("style-attribute-mutation-profile node_id={} name={:?} namespace={:?} changed={} inline_style={}",
                self.arena[element].id, attribute.local_name, attribute.namespace_uri, changed,
                attribute.local_name == "style" && attribute.namespace_uri.is_empty());
        }
        if !changed {
            return;
        }
        let sibling_sensitive_before = self.NodeMayAffectSiblingSelectors(element);
        let descendant_sensitive_before = self.NodeMayAffectDescendantSelectors(element);
        self.BumpCollectionMembershipRevision();
        let affects_metadata = !matches!(attribute.local_name.as_str(), "style" | "class" | "id");
        let selector_only = if attribute.local_name == "style" && attribute.namespace_uri.is_empty()
        {
            self.RecordStyleChange(crate::style_state::StyleChange::InlineStyle(element));
            if sibling_sensitive_before && !self.style_state.all_dirty {
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
        if affects_metadata && !selector_only {
            self.style_state
                .impact
                .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        }
        if let Some(index) = existing {
            self.arena[element].attributes[index] = attribute;
        } else {
            self.arena[element].attributes.push(attribute);
        }
        if self.arena[element].IsElement("option", DOMNamespace::kHTML) {
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
        if radio_group_may_change {
            self.InvalidateRadioInputs();
        }
        if exportparts_changed {
            self.SyncPartNamesMap(element, false);
            self.RecordStyleChange(crate::style_state::StyleChange::ExportParts(element));
        }
        if self.arena[element].IsElement("input", DOMNamespace::kHTML)
            || self.arena[element].IsElement("textarea", DOMNamespace::kHTML)
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
        if directionality_changed {
            self.RecomputeDirectionality();
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
        } else if select_size_changed {
            self.ResetSelectToDefaultSelection(element);
        }
        // Keep both sides of the mutation: removing a matching class/id is as
        // relevant as adding one, and the old state is no longer observable.
        self.RecordSiblingSensitivity(element, false);
        self.RecordDescendantSensitivity(element, descendant_sensitive_before);
    }

    // cpp: dom/document.cc:238-247
    pub fn RemoveAttribute(&mut self, element: usize, local_name: &str, namespace_uri: &str) {
        if self.arena[element].node_type != DOMNodeType::kElement {
            invalid_argument("attributes require an element");
        }
        if !self.arena[element]
            .attributes
            .iter()
            .any(|a| a.local_name == local_name && a.namespace_uri == namespace_uri)
        {
            return;
        }
        let sibling_sensitive_before = self.NodeMayAffectSiblingSelectors(element);
        let descendant_sensitive_before = self.NodeMayAffectDescendantSelectors(element);
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
        if std::env::var_os("BROWSER_PROFILE_STYLE_INVALIDATION").is_some() {
            eprintln!("style-attribute-mutation-profile node_id={} name={:?} namespace={:?} changed=true inline_style={} removed=true",
                self.arena[element].id, local_name, namespace_uri,
                local_name == "style" && namespace_uri.is_empty());
        }
        let selector_only = if local_name == "style" && namespace_uri.is_empty() {
            self.RecordStyleChange(crate::style_state::StyleChange::InlineStyle(element));
            if sibling_sensitive_before && !self.style_state.all_dirty {
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
        if !selector_only && !matches!(local_name, "style" | "class" | "id") {
            self.style_state
                .impact
                .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        }
        self.arena[element].attributes.retain(|attribute| {
            attribute.local_name != local_name || attribute.namespace_uri != namespace_uri
        });
        if default_button_may_change {
            self.InvalidateDefaultPseudoCandidates();
        }
        if self.arena[element].IsElement("option", DOMNamespace::kHTML) {
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
        } else if select_size_changed {
            self.ResetSelectToDefaultSelection(element);
        }
        if namespace_uri.is_empty() && local_name == "popover" {
            let id = self.arena[element].id;
            self.open_popovers.remove(&id);
            self.RemoveFromTopLayerIfUnused(element);
        }
        if namespace_uri.is_empty()
            && local_name == "open"
            && self.arena[element].IsHTMLElement("dialog")
        {
            let id = self.arena[element].id;
            self.modal_dialogs.remove(&id);
            self.RemoveFromTopLayerIfUnused(element);
        }
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
            && matches!(local_name, "checked" | "form" | "id" | "name" | "type")
        {
            self.InvalidateRadioInputs();
        }
        if local_name == "exportparts" && namespace_uri.is_empty() {
            self.SyncPartNamesMap(element, true);
            self.RecordStyleChange(crate::style_state::StyleChange::ExportParts(element));
        }
        if self.arena[element].IsElement("input", DOMNamespace::kHTML)
            || self.arena[element].IsElement("textarea", DOMNamespace::kHTML)
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
            self.RecomputeDirectionality();
        }
        self.RecordSiblingSensitivity(element, false);
        self.RecordDescendantSensitivity(element, descendant_sensitive_before);
    }

    // cpp: dom/document.h:115-117
    pub fn RemoveAttributeDefault(&mut self, element: usize, local_name: &str) {
        self.RemoveAttribute(element, local_name, "")
    }

    // cpp: dom/document.cc:249-265
    pub fn SetTextContent(&mut self, node: usize, data: String) {
        if matches!(
            self.arena[node].node_type,
            DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
        ) {
            if self.arena[node].data == data {
                return;
            }
            let was_empty = self.arena[node].data.is_empty();
            self.arena[node].data = data;
            if let Some(parent) = self.arena[node].parent {
                self.RecordCharacterDataChange(parent, was_empty, self.arena[node].data.is_empty());
            }
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
            // cpp: radio_button_group_scope.cc:65-75,111-139. Checking one
            // radio clears the previous checked member; either transition can
            // change :indeterminate for every member of the group.
            if checked {
                for member in self.RadioGroupMembers(node) {
                    if member != node && self.ControlChecked(member) {
                        self.control_checked_state
                            .insert(self.arena[member].id, false);
                    }
                }
            }
            self.InvalidateRadioGroup(node);
        } else {
            self.InvalidateNodeStyle(node);
        }
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
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
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
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
    pub fn IsPopover(&self, element: usize) -> bool {
        self.arena[element].Namespace() == DOMNamespace::kHTML
            && self.arena[element].FindAttribute("popover").is_some()
    }

    pub fn PopoverOpen(&self, element: usize) -> bool {
        self.IsPopover(element) && self.open_popovers.contains(&self.arena[element].id)
    }

    pub fn DialogIsModal(&self, element: usize) -> bool {
        self.arena[element].IsHTMLElement("dialog")
            && self.modal_dialogs.contains(&self.arena[element].id)
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

    fn AddToTopLayer(&mut self, element: usize) {
        let id = self.arena[element].id;
        if !self.top_layer_elements.contains(&id) {
            self.top_layer_elements.push(id);
        }
    }

    fn RemoveFromTopLayerIfUnused(&mut self, element: usize) {
        let id = self.arena[element].id;
        if !self.modal_dialogs.contains(&id)
            && !self.open_popovers.contains(&id)
            && !self.fullscreen_element_stack.contains(&id)
        {
            self.top_layer_elements.retain(|candidate| *candidate != id);
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
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
    }

    pub fn SVGImageViewSpecFor(&self, root: usize) -> Option<SVGImageViewSpec> {
        (self.svg_image_view_root == Some(self.arena[root].id))
            .then_some(self.svg_image_view_spec)
            .flatten()
    }

    pub fn CorrespondingSVGElement(&self, element: usize) -> Option<usize> {
        self.corresponding_svg_elements
            .get(&self.arena[element].id)
            .and_then(|id| self.FindNodeById(*id))
    }

    pub fn SetCorrespondingSVGElement(&mut self, element: usize, corresponding: Option<usize>) {
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
        self.modal_dialogs.insert(self.arena[element].id);
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
        self.modal_dialogs.remove(&self.arena[element].id);
        self.RemoveFromTopLayerIfUnused(element);
        self.InvalidateNodeStyle(element);
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::TREE);
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
        self.RemoveFromTopLayerIfUnused(element);
        self.InvalidateNodeStyle(element);
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::TREE);
    }

    // cpp: listed_element.cc form-owner reset algorithm. This compact owner
    // covers native listed controls; form-associated custom elements need
    // their own ElementInternals producer before they can enter this path.
    fn FormOwner(&self, control: usize) -> Option<usize> {
        if let Some(id) = self.arena[control]
            .FindAttribute("form")
            .map(|attribute| attribute.value.as_str())
            .filter(|id| !id.is_empty())
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

    fn RadioFormOwner(&self, radio: usize) -> Option<usize> {
        self.FormOwner(radio)
    }

    // cpp: submit_input_type.cc:73-75; image_input_type.cc:153-155;
    //      html_button_element.cc:287-289.
    fn CanBeSuccessfulSubmitButton(&self, element: usize) -> bool {
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
        if self.InputType(radio) != Some(DOMInputType::kRadio) {
            return Vec::new();
        }
        let name = self.arena[radio]
            .FindAttribute("name")
            .map_or("", |attribute| attribute.value.as_str());
        if name.is_empty() {
            return vec![radio];
        }
        let form = self.RadioFormOwner(radio);
        let tree_scope = self.TreeScopeRoot(radio);
        (0..self.arena.len())
            .filter(|&candidate| {
                self.InputType(candidate) == Some(DOMInputType::kRadio)
                    && self.arena[candidate]
                        .FindAttribute("name")
                        .is_some_and(|attribute| attribute.value == name)
                    && match form {
                        Some(form) => self.RadioFormOwner(candidate) == Some(form),
                        None => {
                            self.RadioFormOwner(candidate).is_none()
                                && self.TreeScopeRoot(candidate) == tree_scope
                        }
                    }
            })
            .collect()
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

    // cpp: html_option_element.cc:243-254.
    fn OptionValue(&self, option: usize) -> String {
        if let Some(value) = self.arena[option].FindAttribute("value") {
            return value.value.clone();
        }
        let mut value = String::new();
        self.AppendDescendantText(option, &mut value);
        value
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

    // cpp: html_option_element.cc:386-412.
    fn SetOptionSelectedState(&mut self, option: usize, selected: bool) -> bool {
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
            self.InvalidateNodeStyle(select);
        }
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        true
    }

    fn SelectIsMultiple(&self, select: usize) -> bool {
        self.arena[select].IsElement("select", DOMNamespace::kHTML)
            && self.arena[select].FindAttribute("multiple").is_some()
    }

    fn SelectSize(&self, select: usize) -> u32 {
        self.arena[select]
            .FindAttribute("size")
            .and_then(|attribute| Self::ParseHTMLNonNegativeInteger(&attribute.value))
            .unwrap_or(0)
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
        self.control_value_state.remove(&self.arena[select].id);
        self.InvalidateNodeStyle(select);
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
    fn OptionSubtreeInserted(&mut self, root: usize) {
        let mut options = Vec::new();
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            if self.arena[node].IsElement("option", DOMNamespace::kHTML) {
                options.push(node);
            }
            pending.extend(self.arena[node].Children().iter().rev().copied());
        }
        for option in options {
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
    pub fn ControlValue(&self, node: usize) -> String {
        // cpp: FileInputType::ValueInFilenameValueMode. This projection has
        // no FileList owner, so it can only expose the exact empty-list state;
        // the value content attribute never supplies a filename.
        if self.InputType(node) == Some(DOMInputType::kFile) {
            return String::new();
        }
        if let Some(value) = self.control_value_state.get(&self.arena[node].id) {
            return value.clone();
        }
        if self.arena[node].IsElement("input", DOMNamespace::kHTML) {
            let value = self.arena[node]
                .FindAttribute("value")
                .map_or_else(String::new, |value| value.value.clone());
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

    // cpp: dom/document.cc:328-333
    pub fn SetControlValue(&mut self, node: usize, value: String) {
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
            self.control_value_state.remove(&self.arena[node].id);
            self.SyncFileUAShadowTree(node);
            self.InvalidateNodeStyle(node);
            self.style_state
                .impact
                .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
            return;
        }
        if self.arena[node].IsElement("select", DOMNamespace::kHTML) {
            // SelectOptionByElement clears suggested_option_ before applying
            // either script selection or an autofill selection.
            self.SetSelectSuggestedOption(node, None);
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
        self.SetAutofillState(node, DOMAutofillState::kNotFilled);
        let value = if self.InputType(node) == Some(DOMInputType::kColor) {
            Self::SanitizeColorInputValue(&value)
        } else {
            value
        };
        // HTMLInputElement/HTMLTextAreaElement clear the base-class suggested
        // value before their early-return-on-equal-value check.
        let had_suggested_value = if self.arena[node].IsElement("input", DOMNamespace::kHTML)
            || self.arena[node].IsElement("textarea", DOMNamespace::kHTML)
        {
            self.suggested_value_state
                .remove(&self.arena[node].id)
                .is_some()
        } else {
            false
        };
        let previous = self.ControlValue(node);
        let changed = previous != value;
        let empty_changed = previous.is_empty() != value.is_empty();
        self.control_value_state.insert(self.arena[node].id, value);
        if !changed {
            if had_suggested_value {
                self.SyncTextControlPlaceholderElement(node);
                if let Some(editor) = self.TextControlInnerEditorElement(node) {
                    self.InvalidateNodeStyle(editor);
                }
                self.InvalidateNodeStyle(node);
                self.style_state
                    .impact
                    .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
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
        self.RecomputeDirectionality();
        // Live text values affect selector matching through :placeholder-shown.
        // Changing one nonempty value to another leaves that state (and every
        // attribute selector) unchanged. Keep the broad dependency fallback for
        // an empty/nonempty transition, including ancestors matched by :has().
        // Layout still receives every edit, even when the cascade is unchanged.
        if empty_changed || self.arena[node].IsElement("select", DOMNamespace::kHTML) {
            self.InvalidateNodeStyle(node);
        }
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
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
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
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
    pub fn TakeAllChildren(&mut self, source: usize, destination: usize) {
        let children = self.arena[source].children.clone();
        for child in children {
            self.AppendChild(destination, child);
        }
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
            .is_some_and(|node| self.arena[node].IsHTMLElement("style"))
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
        let contains_sheet = self.cssom.GetStyleSheets(None).iter().any(|sheet| {
            let mut owner = self.FindNodeById(sheet.owner_node_id);
            while let Some(node) = owner {
                if node == subtree {
                    return true;
                }
                owner = self.arena[node].parent;
            }
            false
        });
        if contains_sheet {
            self.style_state.sheet_revision += 1;
            // Let StyleEngine compare the retained active-sheet sequence.
            // Connecting a new source-order suffix can use its compiled target
            // features as an invalidation set; removal, replacement and
            // reordering still fail that proof and become a full update there.
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
        self.cssom
            .GetStyleSheets(None)
            .iter()
            .filter(move |sheet| {
                sheet.CanBeActivated(preferred_set_name)
                    && self.StyleSheetTreeScope(sheet) == Some(scope)
            })
            .chain(
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

    // cpp: dom/document.cc:366-378
    pub fn SetImageResource(&mut self, source: String, metadata: ImageResourceMetadata) {
        // Image completion changes the resource and intrinsic geometry, not
        // selector matching across the entire document. Our style adapter
        // embeds background/mask ids, so refresh only their actual consumers;
        // replaced-image metadata is synchronized by the layout tree builder.
        if let Some(consumers) = self.style_image_consumers.get(&source) {
            let consumers: Vec<_> = consumers.iter().copied().collect();
            for index in consumers {
                self.RecordStyleChange(crate::style_state::StyleChange::Resource(index));
            }
        }
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        match self.image_resources.entry(source) {
            std::collections::hash_map::Entry::Vacant(slot) => {
                slot.insert(metadata);
            }
            std::collections::hash_map::Entry::Occupied(_) => {
                invalid_argument("image source must be unique");
            }
        }
    }
    pub fn ImageResourceFor(&self, source: &str) -> Option<&ImageResourceMetadata> {
        self.image_resources.get(source)
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
        if let Some(selection) = selection {
            self.responsive_image_selections.insert(node, selection);
        } else {
            self.responsive_image_selections.remove(&node);
        }
        // Candidate changes alter the replaced resource id, intrinsic size
        // and image_device_pixel_ratio without changing selector matching.
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        true
    }
    pub fn UpdateImageResourceIntrinsicSize(
        &mut self,
        source: &str,
        metadata: ImageResourceMetadata,
    ) -> bool {
        if let Some(consumers) = self.style_image_consumers.get(source) {
            let consumers: Vec<_> = consumers.iter().copied().collect();
            for index in consumers {
                self.RecordStyleChange(crate::style_state::StyleChange::Resource(index));
            }
        }
        let Some(resource) = self.image_resources.get_mut(source) else {
            return false;
        };
        *resource = metadata;
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        true
    }

    // cpp: dom/document.h:197-199
    // cpp: dom/document.cc:351-364
    pub fn ResolvedStyleFor(&self, node: usize) -> Option<&ResolvedNodeStyle> {
        self.resolved_styles
            .get(&self.arena[node].id)
            .map(std::sync::Arc::as_ref)
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
        self.style_state.sibling_sensitive_nodes.clear();
        self.style_state.append_only_children.clear();
        self.style_state.non_append_children.clear();
        self.style_state.inserted_style_subtrees.clear();
        self.style_state.append_affected_subtrees.clear();
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
    }
    fn RecordStyleChange(&mut self, change: crate::style_state::StyleChange) {
        if !self.style_state.all_dirty {
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
        if self.style_state.all_dirty {
            return;
        }
        let sensitive = conservative || self.NodeMayAffectSiblingSelectors(node);
        if sensitive {
            self.style_state.sibling_sensitive_nodes.insert(node);
        }
    }
    fn RecordDescendantSensitivity(&mut self, node: usize, conservative: bool) {
        if self.style_state.all_dirty {
            return;
        }
        if conservative || self.NodeMayAffectDescendantSelectors(node) {
            self.style_state.descendant_sensitive_nodes.insert(node);
        }
    }
    fn MarkStyleElementTextDirty(&mut self, node: usize) {
        let mut ancestor = Some(node);
        while let Some(index) = ancestor {
            if self.arena[index].IsHTMLElement("style") {
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
        self.BumpCollectionMembershipRevision();
        self.MarkStyleElementTextDirty(parent);
        if was_empty != is_empty || self.arena[parent].IsHTMLElement("textarea") {
            self.RecordStyleChange(crate::style_state::StyleChange::Children(parent));
            if !self.style_state.all_dirty {
                self.style_state.non_append_children.insert(parent);
                self.style_state.append_only_children.remove(&parent);
            }
        }
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        self.RecomputeDirectionality();
        self.RefreshSelectSuggestedValues();
    }
    fn RecordChildrenChange(&mut self, parent: usize) {
        self.BumpCollectionMembershipRevision();
        self.MarkStyleElementTextDirty(parent);
        self.RecordStyleChange(crate::style_state::StyleChange::Children(parent));
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::TREE);
        if self.arena[parent].IsElement("fieldset", DOMNamespace::kHTML) {
            self.InvalidateFormDisabledDescendants(parent);
        }
        // cpp: id_target_observer.cc / HTMLInputElement::ListAttributeTargetChanged.
        // Inserting, removing or reordering an ID target can change which
        // datalist a list-bearing input resolves in its TreeScope. The compact
        // DOM owner does not retain Blink's per-ID observer registry, so it
        // publishes the equivalent input invalidations here.
        self.InvalidateDatalistInputs();
        self.InvalidateRadioInputs();
        self.InvalidateDefaultPseudoCandidates();
        self.InvalidateAncestorSelectContainsInput(parent);
        self.RecomputeDirectionality();
        self.RefreshSelectSuggestedValues();
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

    fn InvalidateDatalistInputs(&mut self) {
        if self.style_state.all_dirty {
            return;
        }
        let inputs = (0..self.arena.len())
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
            self.InvalidateNodeStyle(input);
        }
    }

    fn InvalidateRadioInputs(&mut self) {
        if self.style_state.all_dirty {
            return;
        }
        let radios = (0..self.arena.len())
            .filter(|&node| self.InputType(node) == Some(DOMInputType::kRadio))
            .collect::<Vec<_>>();
        for radio in radios {
            self.InvalidateNodeStyle(radio);
        }
    }

    // HTMLFormElement::FindDefaultButton is tree-order dependent. A form
    // association, submit-button type, form id, or children mutation can move
    // :default from one control to another without mutating the new control.
    fn InvalidateDefaultPseudoCandidates(&mut self) {
        if self.style_state.all_dirty {
            return;
        }
        let candidates = (0..self.arena.len())
            .filter(|&node| {
                self.arena[node].Namespace() == DOMNamespace::kHTML
                    && matches!(self.arena[node].Name(), "input" | "button")
            })
            .collect::<Vec<_>>();
        for candidate in candidates {
            self.InvalidateNodeStyle(candidate);
        }
    }
    fn RecordNonAppendChildrenChange(&mut self, parent: usize) {
        self.RecordChildrenChange(parent);
        if self.style_state.all_dirty {
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
        if self.style_state.all_dirty {
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
    fn RecordAttributeStyleChange(
        &mut self,
        node: usize,
        name: &str,
        namespace: &str,
        sibling_sensitive: bool,
    ) -> bool {
        use crate::style_state::SelectorOnlyAttribute;
        let kind = if namespace.is_empty() && self.arena[node].namespace == DOMNamespace::kHTML {
            match name {
                "data-pcr" => Some(SelectorOnlyAttribute::DataPcr),
                "src" if self.arena[node].IsHTMLElement("script") => {
                    Some(SelectorOnlyAttribute::ScriptSrc)
                }
                "nonce" if self.arena[node].IsHTMLElement("script") => {
                    Some(SelectorOnlyAttribute::ScriptNonce)
                }
                _ => None,
            }
        } else {
            None
        };
        if let Some(kind) = kind {
            self.RecordStyleChange(crate::style_state::StyleChange::Attribute(node, kind));
            if sibling_sensitive && !self.style_state.all_dirty {
                self.style_state.sibling_sensitive_nodes.insert(node);
            }
            true
        } else {
            self.RecordStyleChange(crate::style_state::StyleChange::Node(node));
            if sibling_sensitive && !self.style_state.all_dirty {
                self.style_state.sibling_sensitive_nodes.insert(node);
            }
            false
        }
    }
    pub fn InvalidateNodeStyle(&mut self, node: usize) {
        self.RecordStyleChange(crate::style_state::StyleChange::Node(node));
        self.RecordSiblingSensitivity(node, true);
        self.RecordDescendantSensitivity(node, true);
    }
    pub fn ResolvedStyleHandle(&self, node: usize) -> Option<std::sync::Arc<ResolvedNodeStyle>> {
        self.resolved_styles.get(&self.arena[node].id).cloned()
    }
    pub fn SetResolvedStyle(&mut self, node: usize, style: ResolvedNodeStyle) {
        if self.ResolvedStyleFor(node) == Some(&style) {
            return;
        }
        let new_sources = StyleImageSources(&style);
        let same_sources = self
            .ResolvedStyleFor(node)
            .is_some_and(|old| StyleImageSources(old) == new_sources);
        if !same_sources {
            self.RemoveResolvedStyle(node);
            for source in &new_sources {
                self.style_image_consumers
                    .entry(source.clone())
                    .or_default()
                    .insert(node);
            }
        }
        self.resolved_styles
            .insert(self.arena[node].id, std::sync::Arc::new(style));
    }
    fn RemoveResolvedStyle(&mut self, node: usize) {
        let Some(style) = self.resolved_styles.remove(&self.arena[node].id) else {
            return;
        };
        for source in StyleImageSources(&style) {
            if let Some(consumers) = self.style_image_consumers.get_mut(&source) {
                consumers.remove(&node);
                if consumers.is_empty() {
                    self.style_image_consumers.remove(&source);
                }
            }
        }
    }

    // cpp: dom/document.h:166-167
    // cpp: dom/document.cc:380-388
    pub fn SetScrollOffset(&mut self, node: usize, offset: Offset) {
        self.scroll_offsets.insert(self.arena[node].id, offset);
    }
    pub fn ScrollOffsetFor(&self, node: usize) -> Offset {
        self.scroll_offsets
            .get(&self.arena[node].id)
            .copied()
            .unwrap_or_default()
    }

    // cpp: dom/document.h:132-132
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
        use layoutng_assembly::internal::paint_input::{BackgroundImageLayer, PaintMaskLayer};
        fn image(source: &str) -> BackgroundImageLayer {
            BackgroundImageLayer {
                source_url: source.into(),
                ..Default::default()
            }
        }
        fn pseudo(source: &str) -> PseudoElement {
            let mut pseudo = PseudoElement::default();
            pseudo.style.paint.mask_images.push(PaintMaskLayer {
                image: image(source),
                ..Default::default()
            });
            pseudo
        }
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
        style.style.paint.background_images.push(image("shared"));
        style.style.paint.mask_images.push(PaintMaskLayer {
            image: image("shared"),
            ..Default::default()
        });
        style.before = Some(pseudo("before"));
        style.after = Some(pseudo("after"));
        style.first_letter = Some(pseudo("first-letter"));
        style.placeholder = Some(pseudo("placeholder"));
        document.SetResolvedStyle(first, style);
        let mut style = ResolvedNodeStyle::default();
        style.style.paint.background_images.push(image("shared"));
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
        replacement
            .style
            .paint
            .background_images
            .push(image("replacement"));
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
