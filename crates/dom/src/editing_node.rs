//! Canonical DOM/HTML owners of editing endpoint eligibility.
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
#![allow(non_snake_case)]

use super::{DOMNamespace, DOMNode, DOMNodeType, PersistentDocument};
use std::rc::Rc;

/// The arena's equivalent of the owning C++ dynamic class. Only allocation
/// selects a class from a name; editing queries dispatch on retained identity.
#[derive(Clone, Debug)]
pub(super) enum DOMNodeClass {
    Node,
    Document,
    DocumentFragment,
    Text,
    CDATASection,
    Element,
    ScriptElement,
    StyleElement,
    HTMLAnchorElement {
        link_relations: u32,
    },
    HTMLAreaElement {
        link_relations: u32,
        data: Box<super::html_image_map::HTMLAreaElementData>,
    },
    HTMLMapElement(Box<super::html_image_map::HTMLMapElementData>),
    SVGTextElement,
    SVGForeignObjectElement,
    HTMLMarqueeElement,
    SVGAElement {
        link_relations: u32,
    },
    MathMLAnchorElement {
        link_relations: u32,
    },
    HTMLTableElement,
    HTMLDetailsElement(Box<super::html_details_owner::HTMLDetailsElementData>),
    HTMLTableRowElement(Box<super::table_row_cells::HTMLTableRowElement>),
    HTMLTableCellElement,
    HTMLDialogElement,
    HTMLButtonElement,
    HTMLFieldSetElement,
    HTMLFormElement,
    HTMLInputElement,
    HTMLHtmlElement,
    HTMLBodyElement,
    HTMLFrameSetElement,
    HTMLDivElement,
    VTTElement(Rc<dyn super::vtt_node::VTTElementSource>),
    VTTCueBox(Rc<dyn super::vtt_node::VTTCueBoxSource>),
    VTTCueBackgroundBox(Rc<dyn super::vtt_node::VTTCueBackgroundSource>),
    HTMLSpanElement,
    HTMLLegendElement,
    HTMLNoScriptElement,
    HTMLTemplateElement,
    HTMLSelectUAPart {
        select: usize,
        kind: super::html_select_owner::DOMSelectUAPartKind,
        appearance_state: Option<bool>,
    },
    HTMLOptGroupElement,
    HTMLDataListElement(Box<super::html_data_list_owner::HTMLDataListElementData>),
    HTMLOptionElement {
        was_element_inserted: bool,
        multi_select_focused: bool,
        is_filtered: bool,
        nearest_ancestor_datalist: Option<usize>,
        label_container: Option<usize>,
    },
    HTMLSelectElement(Box<super::html_select_owner::HTMLSelectElementData>),
    HTMLSelectedContentElement(Box<super::html_select_owner::HTMLSelectedContentElementData>),
    HTMLTextAreaElement,
    HTMLSourceElement(Box<super::html_source_element::HTMLSourceElementData>),
    HTMLTrackElement,
    HTMLMediaElement {
        is_video: bool,
    },
    MediaControls(super::media_layout_children::MediaControls),
    MediaControlPanel(super::media_layout_children::MediaControlPanel),
    MediaControlDiv {
        controls: super::DOMOwnerHandle,
        kind: super::media_layout_children::MediaControlDivClass,
    },
    MediaControlInput {
        controls: super::DOMOwnerHandle,
        kind: super::media_layout_children::MediaControlInputClass,
    },
    TextTrackContainer(super::media_layout_children::TextTrackContainer),
    MediaRemotingInterstitial(super::media_layout_children::MediaRemotingInterstitial),
    PictureInPictureInterstitial(super::media_layout_children::PictureInPictureInterstitial),
    HTMLBRElement,
    HTMLImageElement,
    HTMLAltTextContainerElement,
    HTMLAltTextImageElement,
    HTMLCanvasElement,
    HTMLHRElement,
    HTMLFrameElementBase,
    HTMLPlugInElement,
    HTMLProgressElement,
    HTMLMeterElement,
    HTMLOutputElement(Box<HTMLOutputElement>),
    HTMLObjectElement(HTMLObjectElement),
}

#[derive(Clone, Debug)]
pub(super) struct HTMLOutputElement {
    is_default_value_mode: bool,
    default_value: std::rc::Rc<[u16]>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct HTMLObjectElement {
    use_fallback_content: bool,
}

impl HTMLObjectElement {
    // html_plugin_element.cc:910-912; html_object_element.cc:422-424.
    pub(super) fn UseFallbackContent(&self) -> bool {
        self.use_fallback_content
    }
}

impl DOMNodeClass {
    pub(super) fn IsHTMLInputElement(&self) -> bool {
        matches!(
            self,
            Self::HTMLInputElement | Self::MediaControlInput { .. }
        )
    }
    pub(super) fn AnchorLinkRelations(&self) -> Option<u32> {
        match self {
            Self::HTMLAnchorElement { link_relations }
            | Self::HTMLAreaElement { link_relations, .. }
            | Self::SVGAElement { link_relations }
            | Self::MathMLAnchorElement { link_relations } => Some(*link_relations),
            _ => None,
        }
    }
    pub(super) fn AnchorLinkRelationsMut(&mut self) -> Option<&mut u32> {
        match self {
            Self::HTMLAnchorElement { link_relations }
            | Self::HTMLAreaElement { link_relations, .. }
            | Self::SVGAElement { link_relations }
            | Self::MathMLAnchorElement { link_relations } => Some(link_relations),
            _ => None,
        }
    }
    pub(super) fn Allocate(kind: DOMNodeType, namespace: DOMNamespace, name: &str) -> Self {
        match kind {
            DOMNodeType::kDocument => Self::Document,
            DOMNodeType::kDocumentFragment => Self::DocumentFragment,
            DOMNodeType::kText => Self::Text,
            DOMNodeType::kElement if namespace == DOMNamespace::kHTML => match name {
                "script" => Self::ScriptElement,
                "style" => Self::StyleElement,
                "a" => Self::HTMLAnchorElement { link_relations: 0 },
                "area" => Self::HTMLAreaElement {
                    link_relations: 0,
                    data: Box::default(),
                },
                "map" => Self::HTMLMapElement(Box::default()),
                "dialog" => Self::HTMLDialogElement,
                "button" => Self::HTMLButtonElement,
                "fieldset" => Self::HTMLFieldSetElement,
                "form" => Self::HTMLFormElement,
                "input" => Self::HTMLInputElement,
                "html" => Self::HTMLHtmlElement,
                "body" => Self::HTMLBodyElement,
                "frameset" => Self::HTMLFrameSetElement,
                "div" => Self::HTMLDivElement,
                "span" => Self::HTMLSpanElement,
                "legend" => Self::HTMLLegendElement,
                "noscript" => Self::HTMLNoScriptElement,
                "template" => Self::HTMLTemplateElement,
                "optgroup" => Self::HTMLOptGroupElement,
                "datalist" => Self::HTMLDataListElement(Box::default()),
                "option" => Self::HTMLOptionElement {
                    was_element_inserted: false,
                    multi_select_focused: false,
                    is_filtered: false,
                    nearest_ancestor_datalist: None,
                    label_container: None,
                },
                "select" => Self::HTMLSelectElement(Box::default()),
                "selectedcontent" => Self::HTMLSelectedContentElement(Box::default()),
                "textarea" => Self::HTMLTextAreaElement,
                "source" => Self::HTMLSourceElement(Box::default()),
                "track" => Self::HTMLTrackElement,
                "audio" => Self::HTMLMediaElement { is_video: false },
                "video" => Self::HTMLMediaElement { is_video: true },
                "br" => Self::HTMLBRElement,
                "img" => Self::HTMLImageElement,
                "canvas" => Self::HTMLCanvasElement,
                "hr" => Self::HTMLHRElement,
                "frame" | "iframe" => Self::HTMLFrameElementBase,
                "embed" => Self::HTMLPlugInElement,
                "progress" => Self::HTMLProgressElement,
                "meter" => Self::HTMLMeterElement,
                "table" => Self::HTMLTableElement,
                "marquee" => Self::HTMLMarqueeElement,
                "details" => Self::HTMLDetailsElement(Box::default()),
                "tr" => Self::HTMLTableRowElement(Box::default()),
                "td" | "th" => Self::HTMLTableCellElement,
                // html_output_element.cc:39-43; html_object_element.cc:50-54.
                "output" => Self::HTMLOutputElement(Box::new(HTMLOutputElement {
                    is_default_value_mode: true,
                    default_value: std::rc::Rc::from([]),
                })),
                "object" => Self::HTMLObjectElement(HTMLObjectElement::default()),
                _ => Self::Element,
            },
            DOMNodeType::kElement if namespace == DOMNamespace::kSVG && name == "style" => {
                Self::StyleElement
            }
            DOMNodeType::kElement if namespace == DOMNamespace::kSVG && name == "script" => {
                Self::ScriptElement
            }
            DOMNodeType::kElement if namespace == DOMNamespace::kSVG && name == "text" => {
                Self::SVGTextElement
            }
            DOMNodeType::kElement if namespace == DOMNamespace::kSVG && name == "foreignObject" => {
                Self::SVGForeignObjectElement
            }
            DOMNodeType::kElement if namespace == DOMNamespace::kSVG && name == "a" => {
                Self::SVGAElement { link_relations: 0 }
            }
            DOMNodeType::kElement if namespace == DOMNamespace::kMathML && name == "a" => {
                Self::MathMLAnchorElement { link_relations: 0 }
            }
            DOMNodeType::kElement => Self::Element,
            _ => Self::Node,
        }
    }

    fn CanContainRangeEndPoint(&self, node: &DOMNode, locked_ancestor: bool) -> bool {
        match self {
            // node.h:594; text.h:104; element.h:1715;
            // document.h:422; document_fragment.h:57.
            Self::Node => false,
            Self::Document
            | Self::DocumentFragment
            | Self::Text
            | Self::CDATASection
            | Self::Element
            | Self::ScriptElement
            | Self::StyleElement
            | Self::HTMLAnchorElement { .. }
            | Self::HTMLAreaElement { .. }
            | Self::HTMLMapElement(_)
            | Self::SVGAElement { .. }
            | Self::SVGTextElement
            | Self::SVGForeignObjectElement
            | Self::HTMLMarqueeElement
            | Self::MathMLAnchorElement { .. }
            | Self::HTMLTableElement
            | Self::HTMLSelectedContentElement(_)
            | Self::HTMLDetailsElement(_)
            | Self::HTMLTableRowElement(_)
            | Self::HTMLTableCellElement
            | Self::HTMLOptGroupElement
            | Self::HTMLHtmlElement
            | Self::HTMLBodyElement
            | Self::HTMLFrameSetElement
            | Self::HTMLDivElement
            // VTTElement inherits Element; cue/background boxes inherit
            // HTMLDivElement. None overrides Element::CanContainRangeEndPoint.
            | Self::VTTElement(_)
            | Self::VTTCueBox(_)
            | Self::VTTCueBackgroundBox(_)
            | Self::HTMLSpanElement
            | Self::HTMLLegendElement
            | Self::HTMLNoScriptElement
            | Self::HTMLTemplateElement
            | Self::HTMLSourceElement(_)
            | Self::HTMLTrackElement
            | Self::HTMLSelectUAPart { .. }
            | Self::HTMLDataListElement(_)
            | Self::HTMLOptionElement { .. }
            | Self::HTMLAltTextContainerElement
            | Self::HTMLMediaElement { .. }
            | Self::MediaControls(_)
            | Self::MediaControlPanel(_)
            | Self::MediaControlDiv { .. }
            | Self::TextTrackContainer(_)
            | Self::MediaRemotingInterstitial(_)
            | Self::PictureInPictureInterstitial(_)
            | Self::HTMLCanvasElement
            | Self::HTMLButtonElement
            | Self::HTMLDialogElement
            | Self::HTMLFieldSetElement
            | Self::HTMLFormElement => true,
            // html_form_control_element_with_state.h:39;
            // html_br_element.h:42; html_image_element.h:130;
            // html_frame_element_base.h:37; html_plugin_element.h:155;
            // html_progress_element.h:54.
            Self::HTMLInputElement
            | Self::MediaControlInput { .. }
            | Self::HTMLSelectElement(_)
            | Self::HTMLTextAreaElement
            | Self::HTMLBRElement
            | Self::HTMLImageElement
            | Self::HTMLAltTextImageElement
            | Self::HTMLFrameElementBase
            | Self::HTMLPlugInElement
            | Self::HTMLProgressElement => false,
            // html_hr_element.h:42.
            Self::HTMLHRElement => !node.children.is_empty(),
            // html_meter_element.cc:241-250. Style remains owned by Element.
            Self::HTMLMeterElement => {
                !locked_ancestor
                    && node
                        .resolved_style
                        .style
                        .as_ref()
                        .and_then(|style| style.ComputedStyle())
                        .is_some_and(|style| !style.HasEffectiveAppearance())
            }
            // html_output_element.h:59-61; html_object_element.h:83.
            Self::HTMLOutputElement(owner) => owner.is_default_value_mode,
            Self::HTMLObjectElement(owner) => owner.UseFallbackContent(),
        }
    }
}

impl PersistentDocument {
    /// Node::IsLink source flag projection for canonical element subclasses.
    /// Source: anchor_element_utils.cc:100-104; html_image_element.cc:341-342;
    /// mathml_anchor_element.cc:40-44; svg_a_element.cc:90-99. Empty attributes
    /// are present links; a stylesheet HTMLLinkElement is not an anchor owner.
    pub fn IsLinkForEditing(&self, node: usize) -> bool {
        let node = &self.arena[node];
        let plain = |name| {
            node.attributes
                .iter()
                .any(|attribute| attribute.local_name == name && attribute.namespace_uri.is_empty())
        };
        match node.class {
            DOMNodeClass::HTMLAnchorElement { .. }
            | DOMNodeClass::HTMLAreaElement { .. }
            | DOMNodeClass::MathMLAnchorElement { .. } => plain("href"),
            DOMNodeClass::HTMLImageElement | DOMNodeClass::HTMLAltTextImageElement => {
                plain("usemap")
            }
            DOMNodeClass::SVGAElement { .. } => {
                plain("href")
                    || node.attributes.iter().any(|attribute| {
                        attribute.local_name == "href"
                            && attribute.namespace_uri == "http://www.w3.org/1999/xlink"
                    })
            }
            _ => false,
        }
    }
    pub fn NativeArrowKeyClass(
        &self,
        node: usize,
    ) -> foundation::focusgroup_flags::NativeArrowKeyClass {
        use foundation::focusgroup_flags::NativeArrowKeyClass as C;
        let owner = &self.arena[node];
        match owner.class {
            DOMNodeClass::HTMLInputElement | DOMNodeClass::MediaControlInput { .. } => C::Input,
            DOMNodeClass::HTMLSelectElement(_) => C::Select,
            DOMNodeClass::HTMLTextAreaElement => C::TextArea,
            DOMNodeClass::HTMLFrameElementBase
            | DOMNodeClass::HTMLPlugInElement
            | DOMNodeClass::HTMLObjectElement(_) => C::Frame,
            DOMNodeClass::HTMLMediaElement { .. } => C::Media,
            _ if owner.node_type != DOMNodeType::kElement => C::Node,
            _ if owner.namespace == DOMNamespace::kHTML => C::HTMLElement,
            _ => C::Element,
        }
    }
    pub fn CanContainRangeEndPoint(&self, node: usize) -> bool {
        let owner = &self.arena[node];
        owner.class.CanContainRangeEndPoint(
            owner,
            matches!(owner.class, DOMNodeClass::HTMLMeterElement)
                && self.LockedAncestorPreventingPaintForEditing(node),
        )
    }

    /// core/editing/editing_utilities.cc:224-287, with stable feature defaults.
    pub fn IsEditableForEditing(&self, node: usize) -> bool {
        if !self.IsActive() {
            return false;
        }
        let mut ancestor = Some(node);
        while let Some(index) = ancestor {
            let owner = &self.arena[index];
            ancestor = owner.parent;
            if owner.node_type != DOMNodeType::kDocument
                && !(owner.node_type == DOMNodeType::kElement
                    && owner.namespace == DOMNamespace::kHTML)
            {
                continue;
            }
            if owner
                .parent
                .is_some_and(|parent| self.ShadowRoot(parent).is_some())
                && self.AssignedSlot(index).is_none()
            {
                continue;
            }
            if let Some(style) = owner
                .resolved_style
                .style
                .as_ref()
                .and_then(|style| style.ComputedStyle())
            {
                return style.UsedUserModify() != foundation::EUserModify::kReadOnly;
            }
            if owner.FindAttribute("inert").is_some() {
                return false;
            }
        }
        false
    }

    // core/editing/editing_utilities.cc:1887-1908. DOM children and parents,
    // not flat-tree children, define the empty noneditable island branch.
    pub fn EditingIgnoresContent(&self, node: usize) -> bool {
        !self.CanContainRangeEndPoint(node)
            || (self.arena[node].children.is_empty()
                && !self.IsEditableForEditing(node)
                && self.arena[node]
                    .parent
                    .is_some_and(|parent| self.IsEditableForEditing(parent)))
    }

    // core/editing/editing_strategy.cc:61-81.
    pub fn LastOffsetForEditing(&self, node: usize) -> usize {
        let owner = &self.arena[node];
        if matches!(
            owner.node_type,
            DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
        ) {
            return self.CharacterDataUTF16(node).len();
        }
        if !owner.children.is_empty() {
            return owner.children.len();
        }
        usize::from(self.EditingIgnoresContent(node))
    }

    pub fn EditingNodeFacts(&self, node: usize) -> layoutng_assembly::EditingNodeFacts {
        let owner = &self.arena[node];
        let is_editable = self.IsEditableForEditing(node);
        let ignores_content = !self.CanContainRangeEndPoint(node)
            || (owner.children.is_empty()
                && !is_editable
                && owner
                    .parent
                    .is_some_and(|parent| self.IsEditableForEditing(parent)));
        let last_offset = if matches!(
            owner.node_type,
            DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
        ) {
            self.CharacterDataUTF16(node).len()
        } else if !owner.children.is_empty() {
            owner.children.len()
        } else {
            usize::from(ignores_content)
        };
        layoutng_assembly::EditingNodeFacts {
            is_html_table: self.Node(node).IsHTMLElement("table"),
            has_focus_within: self.HasElementFocusWithin(node),
            is_editable,
            ignores_content,
            last_offset,
            is_text_control: owner.IsHTMLElement("textarea")
                || self
                    .InputType(node)
                    .is_some_and(super::DOMInputType::IsTextFieldInputType),
            has_user_agent_shadow_root: self.ShadowRoot(node).is_some_and(|root| {
                self.ShadowRootMode(root) == Some(super::DOMShadowRootMode::kUserAgent)
            }),
            user_agent_shadow_host_id: self
                .ContainingShadowRoot(node)
                .filter(|&root| {
                    self.ShadowRootMode(root) == Some(super::DOMShadowRootMode::kUserAgent)
                })
                .and_then(|root| self.ShadowHost(root))
                .map(|host| self.Node(host).Id()),
            tree_scope_id: self.Node(self.TreeScopeRoot(node)).Id(),
        }
    }

    /// Source editing_utilities.cc:1260-1262 also requires a LayoutObject.
    /// The caller checks native object residency; resolved style isn't proof.
    /// Original DOM dynamic types queried by VisibleUnits; no style/geometry is
    /// inferred from these retained allocation-time identities.
    pub fn IsSVGTextElementForEditing(&self, node: usize) -> bool {
        matches!(self.arena[node].class, DOMNodeClass::SVGTextElement)
    }
    pub fn IsSVGForeignObjectElementForEditing(&self, node: usize) -> bool {
        matches!(
            self.arena[node].class,
            DOMNodeClass::SVGForeignObjectElement
        )
    }
    pub fn IsHTMLMarqueeElementForEditing(&self, node: usize) -> bool {
        matches!(self.arena[node].class, DOMNodeClass::HTMLMarqueeElement)
    }
    pub fn IsHTMLTableElementForEditing(&self, node: usize) -> bool {
        matches!(self.arena[node].class, DOMNodeClass::HTMLTableElement)
    }

    // display_lock_utilities.cc:98-126,193-202,789-793. Exclusive flat-tree
    // ancestors only; an element's own lock blocks its children, not itself.
    pub fn LockedAncestorPreventingPaintForEditing(&self, node: usize) -> bool {
        if !self.IsConnectedToDocument(node)
            || self.ShadowHost(node).is_some()
            || self.DisplayLockLockedCount() == 0
        {
            return false;
        }
        let mut ancestor = self.FlatTreeParent(node);
        while let Some(index) = ancestor {
            if self.arena[index]
                .display_lock_context
                .borrow()
                .as_ref()
                .is_some_and(|context| !context.ShouldPaintChildren())
            {
                return true;
            }
            ancestor = self.FlatTreeParent(index);
        }
        false
    }

    pub fn OutputValueUTF16(&self, element: usize) -> Rc<[u16]> {
        assert!(matches!(
            self.arena[element].class,
            DOMNodeClass::HTMLOutputElement(_)
        ));
        self.DescendantTextUTF16(element)
    }
    pub fn OutputValue(&self, element: usize) -> String {
        String::from_utf16_lossy(&self.OutputValueUTF16(element))
    }
    // html_output_element.cc:98-109. Mode changes even on an equal value.
    pub fn SetOutputValueUTF16(&mut self, element: usize, value: &[u16]) {
        let old_value = self.OutputValueUTF16(element);
        let DOMNodeClass::HTMLOutputElement(owner) = &mut self.arena[element].class else {
            unreachable!()
        };
        if owner.is_default_value_mode {
            owner.default_value = old_value.clone();
        }
        owner.is_default_value_mode = false;
        self.RecordEditingNodeFactsChange(element);
        if value != old_value.as_ref() {
            self.SetTextContentUTF16(element, value);
        }
    }
    pub fn SetOutputValue(&mut self, element: usize, value: String) {
        self.SetOutputValueUTF16(element, &value.encode_utf16().collect::<Vec<_>>());
    }
    // html_output_element.cc:111-123.
    pub fn OutputDefaultValueUTF16(&self, element: usize) -> Rc<[u16]> {
        let DOMNodeClass::HTMLOutputElement(owner) = &self.arena[element].class else {
            panic!("output owner required")
        };
        if owner.is_default_value_mode {
            self.OutputValueUTF16(element)
        } else {
            owner.default_value.clone()
        }
    }
    pub fn OutputDefaultValue(&self, element: usize) -> String {
        String::from_utf16_lossy(&self.OutputDefaultValueUTF16(element))
    }
    pub fn SetOutputDefaultValueUTF16(&mut self, element: usize, value: &[u16]) {
        let DOMNodeClass::HTMLOutputElement(owner) = &mut self.arena[element].class else {
            panic!("output owner required")
        };
        if owner.default_value.as_ref() == value {
            return;
        }
        owner.default_value = Rc::from(value);
        if owner.is_default_value_mode {
            self.SetTextContentUTF16(element, value);
        }
    }
    pub fn SetOutputDefaultValue(&mut self, element: usize, value: String) {
        self.SetOutputDefaultValueUTF16(element, &value.encode_utf16().collect::<Vec<_>>());
    }
    // html_output_element.cc:89-92.
    pub fn ResetOutput(&mut self, element: usize) {
        let value = self.OutputDefaultValueUTF16(element);
        self.SetTextContentUTF16(element, &value);
        let DOMNodeClass::HTMLOutputElement(owner) = &mut self.arena[element].class else {
            unreachable!()
        };
        owner.is_default_value_mode = true;
        self.RecordEditingNodeFactsChange(element);
    }
    /// Required state completion from HTMLObjectElement::RenderFallbackContent
    /// (html_object_element.cc:298-337). Its production caller must first own
    /// error dispatch, MIME/image retry and content-frame disconnection. The
    /// UpdatePluginInternal/RequestObject chain (html_object_element.cc:192-244;
    /// html_plugin_element.cc:695-751) selects fallback; child presence alone
    /// never selects it. That producer is not yet implemented in this arena.
    pub fn RenderObjectFallbackContent(&mut self, element: usize) {
        let DOMNodeClass::HTMLObjectElement(owner) = &mut self.arena[element].class else {
            panic!("object owner required")
        };
        if owner.UseFallbackContent() {
            return;
        }
        if !self.IsConnectedToDocument(element) {
            return;
        }
        let DOMNodeClass::HTMLObjectElement(owner) = &mut self.arena[element].class else {
            unreachable!()
        };
        owner.use_fallback_content = true;
        self.RecordEditingNodeFactsChange(element);
        self.RecordLayoutInputChange(element);
    }

    pub(super) fn RecordEditingNodeFactsChange(&mut self, node: usize) {
        self.style_state
            .editing_input_nodes
            .insert(self.arena[node].id);
    }
    pub(super) fn RecordEditingStyleChange(&mut self, node: usize) {
        self.RecordEditingNodeFactsChange(node);
        // Text has no ComputedStyle of its own. Empty child eligibility also
        // consults its DOM parent's editability after the parent's recalc.
        for &child in &self.arena[node].children {
            self.style_state
                .editing_input_nodes
                .insert(self.arena[child].id);
        }
    }
}

impl PersistentDocument {
    /// Node::IsStyledElement, node.h:536-538.
    pub fn IsStyledElementForEditing(&self, node: usize) -> bool {
        self.arena[node].node_type == DOMNodeType::kElement
            && matches!(
                self.arena[node].namespace,
                DOMNamespace::kHTML | DOMNamespace::kSVG | DOMNamespace::kMathML
            )
    }
    /// NodeTraversal::PreviousPostOrder, node_traversal.cc:180-204.
    pub fn PreviousPostOrderForEditing(&self, node: usize) -> Option<usize> {
        if let Some(child) = self.arena[node].children.last().copied() {
            return Some(child);
        }
        let mut current = Some(node);
        while let Some(node) = current {
            if let Some(previous) = self.arena[node].previous_sibling {
                return Some(previous);
            }
            current = self.arena[node].parent;
        }
        None
    }
}

impl PersistentDocument {
    /// NodeTraversal::PreviousAbsoluteSibling, node_traversal.cc:140-150.
    pub fn PreviousAbsoluteSiblingForEditing(&self, node: usize) -> Option<usize> {
        let mut current = Some(node);
        while let Some(node) = current {
            if let Some(previous) = self.arena[node].previous_sibling {
                return Some(previous);
            }
            current = self.arena[node].parent;
        }
        None
    }
}

impl PersistentDocument {
    /// Element::tagName, element.h:409-418 / HTMLElement::nodeName. The
    /// source compares the serialized qualified name atom, not namespaces.
    pub fn ElementTagNameForEditing(&self, node: usize) -> foundation::AtomicString {
        let q = &self.arena[node].tag_qname;
        let name = if q.Prefix().IsNull() || q.Prefix().empty() {
            q.LocalName().clone()
        } else {
            let mut value = q.Prefix().utf16_units().unwrap_or_default().to_vec();
            value.push(58);
            value.extend_from_slice(q.LocalName().utf16_units().unwrap_or_default());
            foundation::AtomicString::from_utf16(&value)
        };
        if self.DocumentType(self.root) == super::DOMDocumentType::kHTML
            && self.arena[node].namespace == DOMNamespace::kHTML
        {
            name.ToAsciiUpper()
        } else {
            name
        }
    }
}

impl PersistentDocument {
    /// Document::IsInWebAppScope, document.cc:9800-9809. The normalized source
    /// URL is owned by this Document; only the live Settings scope is borrowed.
    pub fn IsInWebAppScope(&self, scope: &foundation::String) -> bool {
        let units = scope.Span16().unwrap_or_default();
        if units.is_empty() {
            return false;
        }
        self.document_url
            .encode_utf16()
            .take(units.len())
            .eq(units.iter().copied())
    }
}
