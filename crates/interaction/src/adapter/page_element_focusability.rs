//! Borrowed Element focus virtuals shared by mouse and sequential navigation.
//!
//! Chromium: core/dom/element.cc:942-1033,8838-8903,8932-9017;
//! html_element.cc:4640-4664; html_form_control_element.cc:349-360;
//! html_anchor_element.cc:98-138; html_summary_element.cc:175-186;
//! html_frame_element_base.h:72-75. The caller completes the synchronous focus
//! lifecycle barrier before borrowing this view. No DOM or Layout owner is copied.

use dom::{DOMFormControlClass, DOMInputType, DOMNamespace, DOMNodeType, Document};
use interaction_default::core::dom::element::{self, ElementMouseFocusability, FocusableState};
use layoutng_style::style::computed_style_constants::PseudoId;

/// Scalar queries over the current resident LayoutObject. A missing object is
/// distinct from an object which is not scrollable or is not a canvas.
pub trait PageElementFocusLayout {
    fn HasLayoutObject(&self, node_id: u64) -> bool;
    fn IsUserScrollable(&self, node_id: u64) -> bool;
    fn IsCanvasLayoutObject(&self, node_id: u64) -> bool;
}

/// A synchronous view of the live DOM and the current focus identity. Callers
/// release this view before dispatching a listener or updating the lifecycle.
pub struct PageElementFocusability<'a> {
    document: &'a Document,
    layout: &'a dyn PageElementFocusLayout,
    focused_node_id: Option<u64>,
}

impl<'a> PageElementFocusability<'a> {
    pub fn new(
        document: &'a Document,
        layout: &'a dyn PageElementFocusLayout,
        focused_node_id: Option<u64>,
    ) -> Self {
        Self {
            document,
            layout,
            focused_node_id,
        }
    }

    pub fn IsFocusable(&self, index: usize) -> bool {
        self.IsFocusableState(index) != FocusableState::kNotFocusable
    }

    pub fn IsMouseFocusable(&self, index: usize) -> bool {
        self.IsElement(index) && element::IsMouseFocusable(&ElementView { owner: self, index })
    }

    /// TabsToLinks is the actual embedder policy, not a navigation default.
    /// The descendant walk uses the same policy as the candidate itself.
    pub fn IsKeyboardFocusable(
        &self,
        index: usize,
        tabs_to_links: bool,
        spatial_navigation_enabled: bool,
    ) -> bool {
        if !self.IsKeyboardFocusableWithoutRadio(index, tabs_to_links, spatial_navigation_enabled) {
            return false;
        }
        if self.document.InputType(index) != Some(DOMInputType::kRadio)
            || spatial_navigation_enabled
            || self.document.ControlChecked(index)
        {
            return true;
        }
        // RadioInputType::IsKeyboardFocusableSlow, source.cc:195-247. DOM
        // owns group membership, checkedness and the retained last-focus slot.
        // Do not infer membership from name alone or rebuild a radio group.
        if self
            .document
            .RadioGroupNodeIds(index)
            .into_iter()
            .any(|id| {
                self.document.FindNodeById(id).is_some_and(|other| {
                    self.document.ControlChecked(other)
                        && self.IsKeyboardFocusable(
                            other,
                            tabs_to_links,
                            spatial_navigation_enabled,
                        )
                })
            })
        {
            return false;
        }
        if let Some(scope) = self.document.RadioGroupScopeFor(index) {
            let name = self
                .document
                .Node(index)
                .FindAttribute("name")
                .map_or("", |attribute| attribute.value.as_str());
            if self
                .document
                .RadioLastFocusedButtonForGroup(scope, name)
                .is_some_and(|id| id != self.document.Node(index).Id())
            {
                return false;
            }
        }
        true
    }

    fn IsKeyboardFocusableWithoutRadio(
        &self,
        index: usize,
        tabs_to_links: bool,
        spatial_navigation_enabled: bool,
    ) -> bool {
        let state = self.IsFocusableState(index);
        if state == FocusableState::kNotFocusable {
            return false;
        }
        let explicit = self.document.IntegralAttribute(index, "tabindex");
        if self.IsLink(index) && !tabs_to_links {
            // HTML anchors with an independent Element focus reason retain
            // keyboard focusability; SVG anchors always obey TabsToLinks.
            if self.document.Node(index).Namespace() == DOMNamespace::kSVG
                || self.BaseSupportsFocus(index) == FocusableState::kNotFocusable
            {
                return false;
            }
        }
        if let Some(tabindex) = explicit {
            return tabindex >= 0;
        }
        if matches!(
            self.document.FormControlClass(index),
            Some(
                DOMFormControlClass::Input
                    | DOMFormControlClass::TextArea
                    | DOMFormControlClass::Select
                    | DOMFormControlClass::Button
                    | DOMFormControlClass::Fieldset
                    | DOMFormControlClass::Output
            )
        ) {
            // HTMLFormControlElement::IsKeyboardFocusableSlow also applies to
            // fieldset/output, whose default tabindex is -1.
            return self.DefaultTabIndex(index) >= 0;
        }
        if state != FocusableState::kKeyboardFocusableScroller {
            return true;
        }
        // Keep a keyboard scroller focused when its descendants change.
        if self.focused_node_id == Some(self.document.Node(index).Id()) {
            return true;
        }
        !self.ContainsKeyboardFocusableElements(index, tabs_to_links, spatial_navigation_enabled)
    }

    pub fn IsScrollControlPseudoElement(&self, index: usize) -> bool {
        self.document
            .PseudoElementFacts(index)
            .is_some_and(|facts| {
                matches!(
                    facts.pseudo_id,
                    PseudoId::kPseudoIdScrollMarker
                        | PseudoId::kPseudoIdScrollButtonBlockStart
                        | PseudoId::kPseudoIdScrollButtonInlineStart
                        | PseudoId::kPseudoIdScrollButtonInlineEnd
                        | PseudoId::kPseudoIdScrollButtonBlockEnd
                )
            })
    }

    fn IsElement(&self, index: usize) -> bool {
        self.document.Node(index).Type() == DOMNodeType::kElement
    }

    fn IsFocusableState(&self, index: usize) -> FocusableState {
        if !self.IsElement(index) {
            return FocusableState::kNotFocusable;
        }
        element::IsFocusableState(&ElementView { owner: self, index })
    }

    fn IsLink(&self, index: usize) -> bool {
        let node = self.document.Node(index);
        ((node.IsHTMLElement("a") || node.IsHTMLElement("area"))
            && node.FindAttribute("href").is_some())
            || (node.IsElement("a", DOMNamespace::kSVG)
                && (node.FindAttribute("href").is_some()
                    || node.FindAttribute("xlink:href").is_some()))
    }

    fn IsRootEditableElement(&self, index: usize) -> bool {
        foundation::editable::IsRootEditableElement(
            &dom::persistent_document::DOMEditableFacts(self.document),
            self.document.Node(index).Id(),
            foundation::editable::EditableFeatures::default(),
        )
    }

    fn BaseSupportsFocus(&self, index: usize) -> FocusableState {
        if self.document.ShadowRootDelegatesFocus(index) {
            return FocusableState::kNotFocusable;
        }
        if self.document.IntegralAttribute(index, "tabindex").is_some()
            || self.IsRootEditableElement(index)
            || self.IsScrollControlPseudoElement(index)
        {
            return FocusableState::kFocusable;
        }
        if self.layout.IsUserScrollable(self.document.Node(index).Id()) {
            FocusableState::kKeyboardFocusableScroller
        } else {
            FocusableState::kNotFocusable
        }
    }

    fn SupportsFocus(&self, index: usize) -> FocusableState {
        // TextControlInnerEditorElement::SupportsFocus is an explicit virtual
        // override returning kNotFocusable.  The editor remains the selection
        // and caret owner, while mouse focus climbs the flat tree to the input
        // or textarea host.
        if self.document.TextControlInnerEditorHost(index).is_some() {
            return FocusableState::kNotFocusable;
        }
        if self.document.IsDisabledFormControl(index)
            || self.document.InputType(index) == Some(DOMInputType::kHidden)
        {
            return FocusableState::kNotFocusable;
        }
        let node = self.document.Node(index);
        let form_control = matches!(
            self.document.FormControlClass(index),
            Some(
                DOMFormControlClass::Input
                    | DOMFormControlClass::TextArea
                    | DOMFormControlClass::Select
                    | DOMFormControlClass::Button
            )
        );
        if form_control
            || (self.IsLink(index) && !self.document.IsEditableForEditing(index))
            || node.IsHTMLElement("iframe")
            || node.IsHTMLElement("frame")
            // HTMLPlugInElement::SupportsFocus is unconditional; the resident
            // attachment/style query still gates its focusability.
            || node.IsHTMLElement("object")
            || node.IsHTMLElement("embed")
            || (node.IsHTMLElement("summary") && self.document.IsDetailsMainSummary(index))
            || ((node.IsHTMLElement("audio") || node.IsHTMLElement("video"))
                && node.FindAttribute("controls").is_some())
        {
            FocusableState::kFocusable
        } else {
            self.BaseSupportsFocus(index)
        }
    }

    fn DefaultTabIndex(&self, index: usize) -> i32 {
        let node = self.document.Node(index);
        if matches!(
            self.document.FormControlClass(index),
            Some(
                DOMFormControlClass::Input
                    | DOMFormControlClass::TextArea
                    | DOMFormControlClass::Select
                    | DOMFormControlClass::Button
            )
        ) || node.IsHTMLElement("a")
            || node.IsHTMLElement("area")
            || node.IsHTMLElement("iframe")
            || node.IsHTMLElement("frame")
            || node.IsHTMLElement("object")
            || node.IsHTMLElement("audio")
            || node.IsHTMLElement("video")
            || (node.IsHTMLElement("summary") && self.document.IsDetailsMainSummary(index))
        {
            0
        } else {
            -1
        }
    }

    fn IsFocusableStyle(&self, index: usize) -> bool {
        // DisplayLockUtilities::IsInUnlockedOrActivatableSubtree uses exclusive
        // flat ancestors. The lock element itself can still receive focus.
        if self.document.DisplayLockLockedCount() != 0 {
            let mut ancestor = self.document.FlatTreeParentElement(index);
            while let Some(node) = ancestor {
                if self.document.DisplayLockIsLocked(node)
                    && !self.document.DisplayLockIsActivatable(
                        node,
                        dom::persistent_document::DisplayLockActivationReason::UserFocus,
                    )
                {
                    return false;
                }
                ancestor = self.document.FlatTreeParentElement(node);
            }
        }
        let Some(style) = self
            .document
            .ResolvedStyleFor(index)
            .and_then(|s| s.ComputedStyle())
        else {
            return false;
        };
        if !style.IsFocusable() {
            return false;
        }
        if self.layout.HasLayoutObject(self.document.Node(index).Id()) {
            return true;
        }
        if style.Display() == foundation::EDisplay::kContents
            && foundation::RuntimeEnabledFeatures::DisplayContentsFocusableEnabled()
        {
            return true;
        }
        if self.document.IsCanvasOrInCanvasSubtree(index) {
            let mut element = Some(index);
            while let Some(current) = element {
                if self.document.Node(current).IsHTMLElement("canvas") {
                    return self
                        .layout
                        .IsCanvasLayoutObject(self.document.Node(current).Id())
                        && self
                            .document
                            .ResolvedStyleFor(current)
                            .and_then(|style| style.ComputedStyle())
                            .is_some_and(|style| style.IsFocusable());
                }
                element = self.document.FlatTreeParentElement(current);
            }
        }
        false
    }

    fn ContainsKeyboardFocusableElements(
        &self,
        index: usize,
        tabs_to_links: bool,
        spatial_navigation_enabled: bool,
    ) -> bool {
        // FlatTreeTraversal::Next(node, stay_within) visits one live child at a
        // time. Do not replace this with an author-tree scan or copied subtree.
        let mut current = self.document.FlatTreeChildren(index).first().copied();
        while let Some(node) = current {
            if self.IsKeyboardFocusable(node, tabs_to_links, spatial_navigation_enabled) {
                return true;
            }
            if let Some(child) = self.document.FlatTreeChildren(node).first().copied() {
                current = Some(child);
                continue;
            }
            let mut ancestor = node;
            current = loop {
                if ancestor == index {
                    break None;
                }
                if let Some(sibling) = self.document.FlatTreeNextSibling(ancestor) {
                    break Some(sibling);
                }
                let Some(parent) = self.document.FlatTreeParent(ancestor) else {
                    break None;
                };
                ancestor = parent;
            };
        }
        false
    }
}

struct ElementView<'a, 'b> {
    owner: &'a PageElementFocusability<'b>,
    index: usize,
}
impl ElementMouseFocusability for ElementView<'_, '_> {
    fn IsConnected(&self) -> bool {
        self.owner.document.IsConnectedToDocument(self.index)
    }
    fn IsFocusableStyle(&self) -> bool {
        self.owner.IsFocusableStyle(self.index)
    }
    fn SupportsFocus(&self) -> FocusableState {
        self.owner.SupportsFocus(self.index)
    }
    fn HasExplicitTabIndex(&self) -> bool {
        self.owner
            .document
            .IntegralAttribute(self.index, "tabindex")
            .is_some()
    }
    fn DefaultTabIndex(&self) -> i32 {
        self.owner.DefaultTabIndex(self.index)
    }
}
