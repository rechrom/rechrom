#![allow(non_camel_case_types)]

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use foundation::{
    gfx, AtomicString, LayoutUnit, MakeGarbageCollected, Member, ScopedCSSName,
    String as BlinkString, Visitor,
};
use layoutng_style::style::computed_style::ComputedStyle as NativeComputedStyle;
use layoutng_style::style::computed_style_constants::PseudoId;

use super::boundary::layout_boundary::ReusableLayoutEnvironment;
use super::form_control_types::FormControlType;
use super::layout_box::LayoutBox;
use super::layout_input::{
    CompatibilityMode, ComputedStyle as InputComputedStyle, ContentLockState, DocumentRole,
    ElementData, NativeNodeConstructionData, NodeKind, PaintPathVerb, SvgLengthAdjust,
    SvgShapeGeometry, ViewportGeometry,
};
use super::layout_input_types::{ControlThemeMetrics, IntSize, ScrollbarThemeMetrics};
use super::layout_object::LayoutObject;
use super::scroll_types::ScrollOffset;
use super::svg_length_adjust_type::SVGLengthAdjustType;

// This header mapping remains in progress while cross-file declarations and
// the owning tree builder are connected.

// cpp: layoutng/internal/layout_node_metadata.h:51-61
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementType {
    kElement,
    kMathMLElement,
    kMathMLRowElement,
    kMathMLFractionElement,
    kMathMLRadicalElement,
    kMathMLPaddedElement,
    kMathMLSpaceElement,
    kMathMLTokenElement,
    kMathMLOperatorElement,
    kMathMLScriptsElement,
    kMathMLUnderOverElement,
    kMathMLTableCellElement,
    kHTMLElement,
    kHTMLInputElement,
    kHTMLSelectElement,
    kHTMLTextAreaElement,
    kHTMLButtonElement,
    kHTMLFieldSetElement,
    kHTMLLegendElement,
    kHTMLOutputElement,
    kHTMLMarqueeElement,
    kHTMLDivElement,
    kSliderThumbElement,
    kTextControlInnerEditorElement,
    kSpinButtonElement,
    kHTMLFrameSetElement,
    kHTMLFrameElement,
    kHTMLAreaElement,
    kHTMLImageElement,
}

// cpp: layoutng/internal/layout_node_metadata.h:69-75
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    kElementNode = 1,
    kAttributeNode = 2,
    kTextNode = 3,
    kCdataSectionNode = 4,
    kProcessingInstructionNode = 7,
    kCommentNode = 8,
    kDocumentNode = 9,
    kDocumentTypeNode = 10,
    kDocumentFragmentNode = 11,
}

// cpp: layoutng/internal/layout_node_metadata.cc:16-49
pub struct StandaloneNodeData {
    pub compatibility_mode: CompatibilityMode,
    pub printing: bool,
    pub paginated: bool,
    pub fragmentainer_block_size: Option<f64>,
    pub vertical_scroll_enforced: bool,
    pub device_pixel_ratio: f64,
    pub canvas_draw_element_enabled: bool,
    pub viewport: Option<ViewportGeometry>,
    pub minimum_font_physical_size: Option<f32>,
    pub scrollbar_theme: Option<ScrollbarThemeMetrics>,
    pub control_theme: ControlThemeMetrics,
    pub document_role: DocumentRole,
    pub input_id: u64,
    pub input_kind: NodeKind,
    pub input_is_style_generated: bool,
    pub input_style: InputComputedStyle,
    pub input_first_line_style_data: Option<InputComputedStyle>,
    pub debug_name: String,
    pub element_data: Option<ElementData>,
    pub reusable_layout_environment: Option<Arc<ReusableLayoutEnvironment>>,
    // Last font resolved by the constraint-space adapter. Construction may
    // temporarily install a placeholder style before that binding runs.
    bound_font: Member<font_engine::fonts::font::Font>,
}
impl StandaloneNodeData {
    pub fn new(input: &NativeNodeConstructionData) -> Self {
        Self {
            compatibility_mode: CompatibilityMode::kStandards,
            printing: false,
            paginated: false,
            fragmentainer_block_size: None,
            vertical_scroll_enforced: false,
            device_pixel_ratio: 1.0,
            canvas_draw_element_enabled: false,
            viewport: None,
            minimum_font_physical_size: None,
            scrollbar_theme: None,
            control_theme: ControlThemeMetrics::default(),
            document_role: input
                .element
                .as_ref()
                .map_or(DocumentRole::kNone, |element| element.document_role),
            input_id: input.id,
            input_kind: input.kind,
            input_is_style_generated: input.style_generated,
            input_style: input.style.clone(),
            input_first_line_style_data: input.first_line_style.clone(),
            debug_name: input.debug_name.clone(),
            element_data: input.element.clone(),
            reusable_layout_environment: None,
            bound_font: Member::default(),
        }
    }
}

// C++ Member<T> fields retain their traced identity. The owning tree
// allocation and destruction order will be established with LayoutObjectTree.
// cpp: layoutng/internal/layout_node_metadata.h:67-164
pub struct Node {
    input_tree_sealed_: bool,
    input_font_dirty_: bool,
    has_first_line_styles_: bool,
    first_line_style_: Member<NativeComputedStyle>,
    input_data_: Box<StandaloneNodeData>,
    control_host_: Member<Element>,
    implicit_anchor_: Member<Element>,
    viewport_defining_element_: Member<Element>,
    type_: NodeType,
    is_container_: bool,
    style_: Member<NativeComputedStyle>,
    layout_object_: Member<LayoutObject>,
    parent_: Member<ContainerNode>,
    previous_: Member<Node>,
    next_: Member<Node>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeMetadataError {
    MissingComputedStyle,
}

#[allow(non_snake_case)]
impl Node {
    // cpp: layoutng/internal/layout_node_metadata.cc:85-93
    pub(crate) fn new(
        input: &NativeNodeConstructionData,
        style: *const NativeComputedStyle,
        node_type: NodeType,
        container: bool,
    ) -> Result<Self, NodeMetadataError> {
        let input_data = Box::new(StandaloneNodeData::new(input));
        if style.is_null() {
            return Err(NodeMetadataError::MissingComputedStyle);
        }
        Ok(Self {
            input_tree_sealed_: false,
            input_font_dirty_: true,
            has_first_line_styles_: false,
            first_line_style_: Member::default(),
            input_data_: input_data,
            control_host_: Member::default(),
            implicit_anchor_: Member::default(),
            viewport_defining_element_: Member::default(),
            type_: node_type,
            is_container_: container,
            style_: Member::from_ptr(style as *mut NativeComputedStyle),
            layout_object_: Member::default(),
            parent_: Member::default(),
            previous_: Member::default(),
            next_: Member::default(),
        })
    }

    // cpp: layoutng/internal/layout_node_metadata.h:79-86
    pub fn getNodeType(&self) -> NodeType {
        self.type_
    }
    pub fn IsElementNode(&self) -> bool {
        self.type_ == NodeType::kElementNode
    }
    pub fn IsTextNode(&self) -> bool {
        self.type_ == NodeType::kTextNode
    }
    pub fn IsContainerNode(&self) -> bool {
        self.is_container_
    }
    pub fn IsDocumentNode(&self) -> bool {
        self.type_ == NodeType::kDocumentNode
    }
    pub fn InputTreeIsSealed(&self) -> bool {
        self.input_tree_sealed_
    }
    pub fn InputHasFirstLineStyles(&self) -> bool {
        self.has_first_line_styles_
    }
    pub fn InputFirstLineStyle(&self) -> *const NativeComputedStyle {
        self.first_line_style_.Get()
    }

    // cpp: layoutng/internal/layout_node_metadata.h:88-89
    pub fn IsMathMLElement(&self) -> bool {
        if !self.IsElementNode() {
            return false;
        }
        let kind = unsafe { &*(self as *const Node).cast::<Element>() }.GetElementType() as u8;
        kind >= ElementType::kMathMLElement as u8
            && kind <= ElementType::kMathMLTableCellElement as u8
    }
    pub fn IsHTMLElement(&self) -> bool {
        if !self.IsElementNode() {
            return false;
        }
        let kind = unsafe { &*(self as *const Node).cast::<Element>() }.GetElementType() as u8;
        kind >= ElementType::kHTMLElement as u8 && kind <= ElementType::kHTMLImageElement as u8
    }

    // cpp: layoutng/internal/layout_node_metadata.h:112-112
    // MathMLTag is an external uint8_t enum. MathML metadata will provide
    // its override; the base Node implementation always returns false.
    pub fn HasTagName(&self, _tag: u8) -> bool {
        false
    }

    // cpp: layoutng/internal/layout_node_metadata.h:90-92
    pub fn OwnerShadowHost(&self) -> *mut Element {
        self.control_host_.Get()
    }
    pub fn IsInUserAgentShadowRoot(&self) -> bool {
        !self.control_host_.Get().is_null()
    }
    pub fn ImplicitAnchorElement(&self) -> *mut Element {
        self.implicit_anchor_.Get()
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:95-147
    pub fn InputCompatibilityMode(&self) -> CompatibilityMode {
        self.input_data_.compatibility_mode
    }
    pub fn InputLineHeightQuirksMode(&self) -> bool {
        self.InputCompatibilityMode() != CompatibilityMode::kStandards
    }
    pub fn InputPrinting(&self) -> bool {
        self.input_data_.printing
    }
    pub fn InputPaginated(&self) -> bool {
        self.input_data_.paginated
    }
    pub fn InputFragmentainerBlockSize(&self) -> Option<f64> {
        self.input_data_.fragmentainer_block_size
    }
    pub fn InputVerticalScrollEnforced(&self) -> bool {
        self.input_data_.vertical_scroll_enforced
    }
    pub fn InputDevicePixelRatio(&self) -> f64 {
        self.input_data_.device_pixel_ratio
    }
    pub fn InputCanvasDrawElementEnabled(&self) -> bool {
        self.input_data_.canvas_draw_element_enabled
    }
    pub fn InputViewport(&self) -> &Option<ViewportGeometry> {
        &self.input_data_.viewport
    }
    pub fn InputMinimumFontPhysicalSize(&self) -> Option<f32> {
        self.input_data_.minimum_font_physical_size
    }
    pub fn InputScrollbarTheme(&self) -> &Option<ScrollbarThemeMetrics> {
        &self.input_data_.scrollbar_theme
    }
    pub fn InputDocumentRole(&self) -> DocumentRole {
        self.input_data_.document_role
    }
    pub fn InputId(&self) -> u64 {
        self.input_data_.input_id
    }
    pub fn InputKind(&self) -> NodeKind {
        self.input_data_.input_kind
    }
    pub fn InputIsStyleGenerated(&self) -> bool {
        self.input_data_.input_is_style_generated
    }
    pub fn InputStyle(&self) -> &InputComputedStyle {
        &self.input_data_.input_style
    }
    pub fn InputFirstLineStyleData(&self) -> &Option<InputComputedStyle> {
        &self.input_data_.input_first_line_style_data
    }
    pub fn InputDebugName(&self) -> &String {
        &self.input_data_.debug_name
    }
    pub fn InputControlTheme(&self) -> &ControlThemeMetrics {
        &self.input_data_.control_theme
    }

    // cpp: layoutng/internal/layout_node_metadata.h:105-105
    pub fn InputViewportDefiningElement(&self) -> *mut Element {
        self.viewport_defining_element_.Get()
    }

    // cpp: layoutng/internal/layout_node_metadata.h:115-126
    pub fn GetComputedStyle(&self) -> *const NativeComputedStyle {
        self.style_.Get()
    }
    pub fn ComputedStyleRef(&self) -> &NativeComputedStyle {
        assert!(!self.style_.Get().is_null());
        unsafe { &*self.style_.Get() }
    }
    pub fn SetComputedStyle(&mut self, style: *const NativeComputedStyle) {
        self.style_ = Member::from_ptr(style as *mut NativeComputedStyle);
    }
    // The owning layout tree updates source data without replacing node identity.
    pub fn UpdateInputData(&mut self, input: &NativeNodeConstructionData) {
        assert_eq!(self.InputId(), input.id);
        assert_eq!(self.InputKind(), input.kind);
        self.input_font_dirty_ = true;
        self.input_data_.input_style = input.style.clone();
        self.input_data_.input_first_line_style_data = input.first_line_style.clone();
        self.input_data_.document_role = input
            .element
            .as_ref()
            .map_or(DocumentRole::kNone, |e| e.document_role);
        self.input_data_.element_data = input.element.clone();
        self.input_data_.debug_name = input.debug_name.clone();
        self.input_tree_sealed_ = false;
    }

    // Scroll translation is paint state, not a font or geometry invalidation.
    pub(crate) fn UpdateInputScrollOffset(&mut self, offset: super::layout_input::Offset) {
        if let Some(data) = self.input_data_.element_data.as_mut() {
            data.scroll_offset = offset;
        }
        let area = super::layout_scrollable_area::PaintLayerScrollableArea::FromNode(self);
        if !area.is_null() {
            // The resident area, not SavedLayerScrollOffset, owns live offsets.
            unsafe { &mut *area }
                .UpdateScrollOffset(ScrollOffset::new(offset.x as f32, offset.y as f32));
        }
    }

    pub(crate) fn InputBoundFont(&self) -> *mut font_engine::fonts::font::Font {
        self.input_data_.bound_font.Get()
    }
    pub(crate) fn SetInputBoundFont(&mut self, font: *mut font_engine::fonts::font::Font) {
        self.input_data_.bound_font = Member::from_ptr(font);
    }

    pub(crate) fn NeedsInputFontBinding(&self) -> bool {
        self.input_font_dirty_
    }
    pub(crate) fn ClearInputFontBinding(&mut self) {
        self.input_font_dirty_ = false;
    }

    pub(crate) fn UnsealForTreeUpdate(&mut self) {
        self.input_tree_sealed_ = false;
    }

    pub(crate) fn DetachForTreeUpdate(&mut self) {
        let parent = self.parent_.Get();
        if parent.is_null() {
            return;
        }
        let previous = self.previous_.Get();
        let next = self.next_.Get();
        unsafe {
            if previous.is_null() {
                (*parent).first_child_ = Member::from_ptr(next);
            } else {
                (*previous).next_ = Member::from_ptr(next);
            }
            if next.is_null() {
                (*parent).last_child_ = Member::from_ptr(previous);
            } else {
                (*next).previous_ = Member::from_ptr(previous);
            }
        }
        self.parent_ = Member::default();
        self.previous_ = Member::default();
        self.next_ = Member::default();
    }

    pub fn SetInputFirstLineStyle(&mut self, style: *const NativeComputedStyle) {
        self.first_line_style_ = Member::from_ptr(style as *mut NativeComputedStyle);
    }
    pub fn GetLayoutObject(&self) -> *mut LayoutObject {
        self.layout_object_.Get()
    }
    pub fn SetLayoutObject(&mut self, object: *mut LayoutObject) {
        self.layout_object_ = Member::from_ptr(object);
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:149-177
    pub fn PrepareStandaloneLayout(
        &mut self,
        input: &super::layout_input::ConstraintSpace,
        viewport_defining_element: *mut Element,
        has_first_line_styles: bool,
    ) {
        self.input_data_.compatibility_mode = input.compatibility_mode;
        self.input_data_.printing = input.printing;
        self.input_data_.paginated = input.fragmentainer_block_size.is_some();
        self.input_data_.fragmentainer_block_size = input.fragmentainer_block_size;
        self.input_data_.vertical_scroll_enforced = input.vertical_scroll_enforced;
        self.input_data_.device_pixel_ratio = input.device_pixel_ratio;
        self.input_data_.canvas_draw_element_enabled = input.canvas_draw_element_enabled;
        self.input_data_.viewport = input.viewport;
        self.has_first_line_styles_ = has_first_line_styles;
        self.input_data_.minimum_font_physical_size = input.minimum_font_physical_size;
        self.input_data_.scrollbar_theme = input.scrollbar_theme;
        self.input_data_.control_theme = input.control_theme;
        self.viewport_defining_element_ = Member::from_ptr(viewport_defining_element);
        self.input_tree_sealed_ = true;
    }
    pub fn TakeReusableLayoutEnvironment(&mut self) -> Option<Arc<ReusableLayoutEnvironment>> {
        self.input_data_.reusable_layout_environment.take()
    }
    pub fn SetReusableLayoutEnvironment(&mut self, cache: Option<Arc<ReusableLayoutEnvironment>>) {
        self.input_data_.reusable_layout_environment = cache;
    }

    // cpp: layoutng/internal/layout_node_metadata.h:137-140
    pub fn parentNode(&self) -> *mut ContainerNode {
        self.parent_.Get()
    }
    pub fn previousSibling(&self) -> *mut Node {
        self.previous_.Get()
    }
    pub fn nextSibling(&self) -> *mut Node {
        self.next_.Get()
    }

    // Rust accessors for the C++ friend NativeNodeMetadataRelations. Keeping
    // the fields private to this module avoids exposing relation mutations to
    // ordinary callers while the implementation lives in boundary/.
    pub(crate) fn set_control_host(&mut self, host: *mut Element) {
        self.control_host_ = Member::from_ptr(host);
    }
    pub(crate) fn set_implicit_anchor(&mut self, anchor: *mut Element) {
        self.implicit_anchor_ = Member::from_ptr(anchor);
    }
    pub(crate) fn set_viewport_defining_element(&mut self, element: *mut Element) {
        self.viewport_defining_element_ = Member::from_ptr(element);
    }
    pub(crate) fn seal_input_tree(&mut self) {
        self.input_tree_sealed_ = true;
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:81-83
    pub fn GetLayoutBox(&self) -> *mut LayoutBox {
        foundation::DynamicTo::<LayoutBox>(self.GetLayoutObject())
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:191-200
    pub fn IsFirstLetterPseudoElement(&self) -> bool {
        self.ComputedStyleRef().StyleType() == PseudoId::kPseudoIdFirstLetter
    }
    pub fn parentElement(&self) -> *mut Element {
        let mut parent = self.parent_.Get();
        while !parent.is_null() {
            // SAFETY: parent links point to live metadata nodes owned by the
            // native tree. Element begins with ContainerNode at offset zero.
            let node = unsafe { &(*parent).node };
            if node.IsElementNode() {
                return parent.cast::<Element>();
            }
            parent = node.parent_.Get();
        }
        std::ptr::null_mut()
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:179-189
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.control_host_);
        visitor.Trace(&self.implicit_anchor_);
        visitor.Trace(&self.viewport_defining_element_);
        visitor.Trace(&self.style_);
        visitor.Trace(&self.first_line_style_);
        visitor.Trace(&self.input_data_.bound_font);
        visitor.Trace(&self.layout_object_);
        visitor.Trace(&self.parent_);
        visitor.Trace(&self.previous_);
        visitor.Trace(&self.next_);
    }
}

// The base member occupies offset zero, preserving the C++ upcast from a
// ContainerNode pointer to a Node pointer used by the tree links.
// cpp: layoutng/internal/layout_node_metadata.h:168-184
#[repr(C)]
pub struct ContainerNode {
    pub node: Node,
    first_child_: Member<Node>,
    last_child_: Member<Node>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppendChildError {
    SealedTree,
    AlreadyAttached,
    Cycle,
}

#[allow(non_snake_case)]
impl ContainerNode {
    pub(crate) fn new(
        input: &NativeNodeConstructionData,
        style: *const NativeComputedStyle,
        node_type: NodeType,
    ) -> Result<Self, NodeMetadataError> {
        Ok(Self {
            node: Node::new(input, style, node_type, true)?,
            first_child_: Member::default(),
            last_child_: Member::default(),
        })
    }

    // cpp: layoutng/internal/layout_node_metadata.h:170-171
    pub fn firstChild(&self) -> *mut Node {
        self.first_child_.Get()
    }
    pub fn lastChild(&self) -> *mut Node {
        self.last_child_.Get()
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:202-215
    pub fn AppendChild(&mut self, child: &mut Node) -> Result<(), AppendChildError> {
        if self.node.InputTreeIsSealed() || child.InputTreeIsSealed() {
            return Err(AppendChildError::SealedTree);
        }
        if !child.parent_.Get().is_null()
            || !child.previous_.Get().is_null()
            || !child.next_.Get().is_null()
        {
            return Err(AppendChildError::AlreadyAttached);
        }
        let child_ptr = child as *mut Node;
        let mut ancestor = &mut self.node as *mut Node;
        while !ancestor.is_null() {
            if ancestor == child_ptr {
                return Err(AppendChildError::Cycle);
            }
            // SAFETY: metadata tree links are valid for the owning tree's
            // lifetime; only the root owner destroys nodes.
            ancestor = unsafe {
                let parent = (*ancestor).parent_.Get();
                if parent.is_null() {
                    std::ptr::null_mut()
                } else {
                    &mut (*parent).node
                }
            };
        }
        child.parent_ = Member::from_ptr(self);
        child.previous_ = Member::from_ptr(self.last_child_.Get());
        if !self.last_child_.Get().is_null() {
            // SAFETY: last_child_ is a linked node owned by the same tree.
            unsafe {
                (*self.last_child_.Get()).next_ = Member::from_ptr(child_ptr);
            }
        } else {
            self.first_child_ = Member::from_ptr(child_ptr);
        }
        self.last_child_ = Member::from_ptr(child_ptr);
        Ok(())
    }

    pub(crate) fn InsertForTreeUpdate(&mut self, child: &mut Node, before: *mut Node) {
        assert!(before.is_null() || unsafe { &*before }.parentNode() == self);
        child.DetachForTreeUpdate();
        let previous = if before.is_null() {
            self.last_child_.Get()
        } else {
            unsafe { &*before }.previousSibling()
        };
        child.parent_ = Member::from_ptr(self);
        child.previous_ = Member::from_ptr(previous);
        child.next_ = Member::from_ptr(before);
        unsafe {
            if previous.is_null() {
                self.first_child_ = Member::from_ptr(child);
            } else {
                (*previous).next_ = Member::from_ptr(child);
            }
            if before.is_null() {
                self.last_child_ = Member::from_ptr(child);
            } else {
                (*before).previous_ = Member::from_ptr(child);
            }
        }
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:217-221
    pub fn Trace(&self, visitor: &mut Visitor) {
        self.node.Trace(visitor);
        visitor.Trace(&self.first_child_);
        visitor.Trace(&self.last_child_);
    }
}

// cpp: layoutng/internal/layout_node_metadata.cc:223-227
#[allow(non_snake_case)]
pub fn ContentLockBlocksChildLayout(node: *const Node) -> bool {
    if node.is_null() || !unsafe { &*node }.IsElementNode() {
        return false;
    }
    let element = unsafe { &*node.cast::<Element>() };
    element
        .InputContentLockState()
        .is_some_and(|state| !state.should_layout_children)
}

// The base Node is at offset zero, as with the C++ public inheritance.
// cpp: layoutng/internal/layout_node_metadata.h:255-264
#[repr(C)]
pub struct Text {
    pub node: Node,
    data_: BlinkString,
}

// C++ public inheritance keeps each base at offset zero. Deref exposes the
// inherited Node methods without copying them into every derived type.
// cpp: layoutng/internal/layout_node_metadata.h:168-186,255-264
impl Deref for ContainerNode {
    type Target = Node;
    fn deref(&self) -> &Node {
        &self.node
    }
}
impl DerefMut for ContainerNode {
    fn deref_mut(&mut self) -> &mut Node {
        &mut self.node
    }
}
impl Deref for Text {
    type Target = Node;
    fn deref(&self) -> &Node {
        &self.node
    }
}
impl DerefMut for Text {
    fn deref_mut(&mut self) -> &mut Node {
        &mut self.node
    }
}
const _: () = assert!(std::mem::offset_of!(ContainerNode, node) == 0);
const _: () = assert!(std::mem::offset_of!(Text, node) == 0);

// cpp: layoutng/internal/layout_node_metadata.h:277-288
// The two static relation methods are defined by boundary/node_metadata_input.cc.
pub struct NativeNodeMetadataRelations;

// cpp: layoutng/internal/layout_node_metadata.h:249-253
// The constructor dispatch belongs to boundary/node_metadata_input.cc.
pub use super::boundary::node_metadata_input::CreateNativeElementMetadata;

// cpp: layoutng/internal/layout_node_metadata.h:266-274
// Foundation's DynamicTo adapter can delegate these C++ DowncastTraits
// predicates when the native metadata tree is connected.
#[allow(non_snake_case)]
pub trait NodeDowncast {
    fn AllowFrom(node: &Node) -> bool;
}
impl NodeDowncast for ContainerNode {
    fn AllowFrom(node: &Node) -> bool {
        node.IsContainerNode()
    }
}
impl NodeDowncast for Element {
    fn AllowFrom(node: &Node) -> bool {
        node.IsElementNode()
    }
}
impl NodeDowncast for Text {
    fn AllowFrom(node: &Node) -> bool {
        node.IsTextNode()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextMetadataError {
    Node(NodeMetadataError),
    InvalidTextInput,
}

#[allow(non_snake_case)]
impl Text {
    // cpp: layoutng/internal/layout_node_metadata.cc:495-499
    pub fn new(
        input: &NativeNodeConstructionData,
        style: *const NativeComputedStyle,
        data: BlinkString,
    ) -> Result<Self, TextMetadataError> {
        let node =
            Node::new(input, style, NodeType::kTextNode, false).map_err(TextMetadataError::Node)?;
        if input.kind != NodeKind::kText || input.element.is_some() {
            return Err(TextMetadataError::InvalidTextInput);
        }
        Ok(Self { node, data_: data })
    }

    // cpp: layoutng/internal/layout_node_metadata.h:259-260
    pub fn data(&self) -> &BlinkString {
        &self.data_
    }
    pub(crate) fn SetDataForTreeUpdate(&mut self, data: BlinkString) {
        self.data_ = data;
    }

    pub fn length(&self) -> u32 {
        self.data_.length() as u32
    }
}

// Element and ContainerNode also begin with Node; native downcasts must
// validate NodeType and ElementType before converting a raw base pointer.
// cpp: layoutng/internal/layout_node_metadata.h:186-247
#[repr(C)]
pub struct Element {
    pub container: ContainerNode,
    element_type_: ElementType,
    is_form_control_element_: bool,
    remembered_inline_size_: Option<LayoutUnit>,
    remembered_block_size_: Option<LayoutUnit>,
    used_canvas_transform_: Option<gfx::Transform>,
    saved_layer_scroll_offset_: ScrollOffset,
    timeline_trigger_names_: Vec<Member<ScopedCSSName>>,
    anchor_scroll_data_: Member<super::anchor_position_scroll_data::AnchorPositionScrollData>,
    out_of_flow_data_: Member<super::css::out_of_flow_data::OutOfFlowData>,
    active_transform_animations_: u8,
    may_be_implicit_anchor_: bool,
}

// cpp: layoutng/internal/layout_node_metadata.h:186-247
impl Deref for Element {
    type Target = ContainerNode;
    fn deref(&self) -> &ContainerNode {
        &self.container
    }
}
impl DerefMut for Element {
    fn deref_mut(&mut self) -> &mut ContainerNode {
        &mut self.container
    }
}
const _: () = assert!(std::mem::offset_of!(Element, container) == 0);

// cpp: layoutng/internal/layout_node_metadata.h:267-271
impl foundation::DowncastFrom<Node> for ContainerNode {
    fn AllowFrom(node: &Node) -> bool {
        node.IsContainerNode()
    }
}

impl foundation::DowncastFrom<Node> for Element {
    fn AllowFrom(node: &Node) -> bool {
        node.IsElementNode()
    }
}

impl foundation::DowncastFrom<Node> for Text {
    fn AllowFrom(node: &Node) -> bool {
        node.IsTextNode()
    }
}

// cpp: layoutng/internal/layout_node_metadata.h:230-230
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransformAnimation {
    kTransform,
    kTranslate,
    kRotate,
    kScale,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementMetadataError {
    Node(NodeMetadataError),
    InvalidInput(&'static str),
}

#[allow(non_snake_case)]
impl Element {
    // cpp: layoutng/internal/layout_node_metadata.cc:229-399
    pub fn new(
        input: &NativeNodeConstructionData,
        style: *const NativeComputedStyle,
    ) -> Result<Self, ElementMetadataError> {
        let container = ContainerNode::new(input, style, NodeType::kElementNode)
            .map_err(ElementMetadataError::Node)?;
        let mut element = Self {
            container,
            element_type_: ElementType::kElement,
            is_form_control_element_: false,
            remembered_inline_size_: None,
            remembered_block_size_: None,
            used_canvas_transform_: None,
            saved_layer_scroll_offset_: ScrollOffset::default(),
            timeline_trigger_names_: Vec::new(),
            anchor_scroll_data_: Member::default(),
            out_of_flow_data_: Member::default(),
            active_transform_animations_: 0,
            may_be_implicit_anchor_: false,
        };
        validate_element_input_prefix(input).map_err(ElementMetadataError::InvalidInput)?;
        prepare_element_dimensions(&mut element, input)
            .map_err(ElementMetadataError::InvalidInput)?;
        element.InputSvgLengthAdjust();
        if let Some(style) = input.style.extended.as_ref() {
            for value in &style.timeline_trigger_names {
                let Some(name) = value else {
                    element.timeline_trigger_names_.push(Member::default());
                    continue;
                };
                if name.is_empty() {
                    return Err(ElementMetadataError::InvalidInput(
                        "Timeline trigger requires a nonempty UTF-8 name",
                    ));
                }
                let units: Vec<u16> = name.encode_utf16().collect();
                let atom = AtomicString::from_utf16(&units);
                let scoped_name = MakeGarbageCollected(ScopedCSSName::new(&atom, std::ptr::null()));
                element
                    .timeline_trigger_names_
                    .push(Member::from_ptr(scoped_name));
            }
        }
        Ok(element)
    }

    pub(crate) fn UpdateElementInput(&mut self, input: &NativeNodeConstructionData) {
        validate_element_input_prefix(input).unwrap_or_else(|error| panic!("{error:?}"));
        self.node.UpdateInputData(input);
        self.remembered_inline_size_ = None;
        self.remembered_block_size_ = None;
        self.used_canvas_transform_ = None;
        prepare_element_dimensions(self, input).unwrap_or_else(|error| panic!("{error:?}"));
        self.InputSvgLengthAdjust();
        let control = foundation::DynamicTo::<super::form_node_metadata::HTMLFormControlElement>(
            self as *mut Element,
        );
        if !control.is_null() {
            unsafe { &mut *control }.autofill_state_ = input
                .element
                .as_ref()
                .map_or(super::form_control_types::AutofillState::kNotFilled, |e| {
                    e.autofill_state
                });
        }
    }

    // Derived metadata constructors select their C++ virtual identity through
    // these tags after the base Element constructor completes.
    pub(crate) fn set_runtime_identity(&mut self, kind: ElementType, form_control: bool) {
        self.element_type_ = kind;
        self.is_form_control_element_ = form_control;
    }

    // cpp: layoutng/internal/layout_node_metadata.h:189-192
    pub fn GetElementType(&self) -> ElementType {
        self.element_type_
    }
    pub fn IsFormControlElement(&self) -> bool {
        self.is_form_control_element_
    }
    pub fn GetUsedCanvasTransform(&self) -> Option<&gfx::Transform> {
        self.used_canvas_transform_.as_ref()
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:401-461
    pub fn InputElementData(&self) -> &Option<ElementData> {
        &self.container.node.input_data_.element_data
    }
    pub fn InputSvgLengthAdjust(&self) -> SVGLengthAdjustType {
        let value = self
            .InputElementData()
            .as_ref()
            .map_or(SvgLengthAdjust::kSpacing, |data| data.svg_length_adjust);
        match value {
            SvgLengthAdjust::kUnknown => SVGLengthAdjustType::kSVGLengthAdjustUnknown,
            SvgLengthAdjust::kSpacing => SVGLengthAdjustType::kSVGLengthAdjustSpacing,
            SvgLengthAdjust::kSpacingAndGlyphs => {
                SVGLengthAdjustType::kSVGLengthAdjustSpacingAndGlyphs
            }
        }
    }
    pub fn InputRootScrollerVisibleSize(&self) -> Option<IntSize> {
        self.InputElementData()
            .as_ref()
            .and_then(|data| data.root_scroller_visible_size)
    }
    pub fn HasNoWrapAttributeForLayout(&self) -> bool {
        self.InputElementData()
            .as_ref()
            .is_some_and(|data| data.nowrap_attribute)
    }
    pub fn InputIsTextControlPlaceholder(&self) -> bool {
        self.InputElementData()
            .as_ref()
            .is_some_and(|data| data.text_control_placeholder)
    }
    pub fn InputIsTextControlContainer(&self) -> bool {
        self.InputElementData()
            .as_ref()
            .is_some_and(|data| data.text_control_container)
    }
    pub fn InputSupportsBaseAppearance(&self) -> bool {
        self.InputElementData()
            .as_ref()
            .is_some_and(|data| data.supports_base_appearance)
    }
    pub fn InputIsViewportDefining(&self) -> bool {
        self.InputElementData()
            .as_ref()
            .is_some_and(|data| data.viewport_defining)
    }
    pub fn InputIsRootEditable(&self) -> bool {
        self.InputElementData()
            .as_ref()
            .is_some_and(|data| data.root_editable)
    }
    pub fn InputContentLockState(&self) -> Option<&ContentLockState> {
        self.InputElementData()
            .as_ref()
            .and_then(|data| data.content_lock.as_ref())
    }
    pub fn InputContentLock(&self) -> Option<bool> {
        self.InputContentLockState().map(|state| state.is_locked)
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:463-473
    pub fn InputListMarker(&self) -> *mut Element {
        let mut child = self.container.firstChild();
        while !child.is_null() {
            // SAFETY: child links are owned by the native metadata tree and
            // list-marker nodes have element identity.
            let child_ref = unsafe { &*child };
            if child_ref.InputKind() == NodeKind::kListMarker {
                assert!(child_ref.IsElementNode());
                return child.cast::<Element>();
            }
            child = child_ref.nextSibling();
        }
        std::ptr::null_mut()
    }
    pub fn InputListMarkerLayoutObject(&self) -> *mut LayoutObject {
        let marker = self.InputListMarker();
        if marker.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &(*marker).container.node }.GetLayoutObject()
        }
    }

    // cpp: layoutng/internal/layout_node_metadata.h:200-220
    pub fn SavedLayerScrollOffset(&self) -> ScrollOffset {
        self.saved_layer_scroll_offset_
    }
    pub fn SetSavedLayerScrollOffset(&mut self, offset: ScrollOffset) {
        self.saved_layer_scroll_offset_ = offset;
    }
    pub fn LastRememberedInlineSize(&self) -> Option<LayoutUnit> {
        self.remembered_inline_size_.clone()
    }
    pub fn LastRememberedBlockSize(&self) -> Option<LayoutUnit> {
        self.remembered_block_size_.clone()
    }
    pub fn InputMayBeImplicitAnchor(&self) -> bool {
        self.may_be_implicit_anchor_
    }
    pub(crate) fn set_may_be_implicit_anchor(&mut self, value: bool) {
        self.may_be_implicit_anchor_ = value;
    }
    pub fn InputTimelineTriggerNames(&self) -> &[Member<ScopedCSSName>] {
        &self.timeline_trigger_names_
    }

    // cpp: layoutng/internal/layout_node_metadata.h:223-227
    pub fn GetAnchorPositionScrollData(
        &self,
    ) -> *mut super::anchor_position_scroll_data::AnchorPositionScrollData {
        self.anchor_scroll_data_.Get()
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.cc:32-36
    pub fn EnsureAnchorPositionScrollData(
        &mut self,
    ) -> &mut super::anchor_position_scroll_data::AnchorPositionScrollData {
        if self.anchor_scroll_data_.Get().is_null() {
            let ptr = MakeGarbageCollected(
                super::anchor_position_scroll_data::AnchorPositionScrollData::new(self),
            );
            self.anchor_scroll_data_ = Member::from_ptr(ptr);
        }
        unsafe { &mut *self.anchor_scroll_data_.Get() }
    }
    pub fn RemoveAnchorPositionScrollData(&mut self) {
        self.anchor_scroll_data_ = Member::default();
    }
    pub fn GetOutOfFlowData(&self) -> *mut super::css::out_of_flow_data::OutOfFlowData {
        self.out_of_flow_data_.Get()
    }

    // cpp: layoutng_out_of_flow/out_of_flow_element_data.cc:11-15
    pub fn EnsureOutOfFlowData(&mut self) -> &mut super::css::out_of_flow_data::OutOfFlowData {
        if self.out_of_flow_data_.Get().is_null() {
            self.out_of_flow_data_ = Member::from_ptr(MakeGarbageCollected(
                super::css::out_of_flow_data::OutOfFlowData::default(),
            ));
        }
        unsafe { &mut *self.out_of_flow_data_.Get() }
    }

    // Keep the narrow write interface used by shared-assembly providers.
    pub fn SetOutOfFlowDataForAssembly(
        &mut self,
        data: *mut super::css::out_of_flow_data::OutOfFlowData,
    ) {
        self.out_of_flow_data_ = Member::from_ptr(data);
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:482-493
    pub fn HasActiveTransformAnimation(&self, property: TransformAnimation) -> bool {
        let index = property as u8;
        assert!(index < 4);
        self.active_transform_animations_ & (1 << index) != 0
    }
    pub fn SetActiveTransformAnimation(&mut self, property: TransformAnimation, active: bool) {
        let index = property as u8;
        assert!(index < 4);
        let mask = 1 << index;
        if active {
            self.active_transform_animations_ |= mask;
        } else {
            self.active_transform_animations_ &= !mask;
        }
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:475-480
    pub fn Trace(&self, visitor: &mut Visitor) {
        self.container.Trace(visitor);
        visitor.Trace(&self.anchor_scroll_data_);
        visitor.Trace(&self.out_of_flow_data_);
        visitor.Trace(&self.timeline_trigger_names_);
    }
}

// C++ member definitions live in other source files. Static receiver
// parameters let providers in later crates implement the interface under
// Rust's orphan rules while preserving the Element identity and GC handle.
#[allow(non_snake_case)]
pub trait OutOfFlowDataProvider {
    // cpp: layoutng/internal/layout_node_metadata.h:227-227
    fn EnsureOutOfFlowData(
        element: &mut Element,
    ) -> *mut super::css::out_of_flow_data::OutOfFlowData;
}

// The pure first stage of Element construction. The owning constructor must
// call this after constructing ContainerNode, as in the C++ initializer list.
// cpp: layoutng/internal/layout_node_metadata.cc:231-327
fn validate_element_input_prefix(input: &NativeNodeConstructionData) -> Result<(), &'static str> {
    let data = input.element.as_ref();
    if input.kind == NodeKind::kText || !input.text.is_empty() {
        return Err("Element metadata requires an element input; text needs a Text node");
    }
    if input.kind == NodeKind::kSvgTextPath
        && !data.is_some_and(|data| data.svg_text_path.is_some())
    {
        return Err("An SVG textPath node requires resolved path geometry");
    }
    if let Some(data) = data {
        if let Some(shape) = data.svg_shape.as_ref() {
            if input.kind != NodeKind::kSvgShape {
                return Err("SVG shape geometry requires an SVG shape node");
            }
            let finite = |value: f64| value.is_finite() && value.abs() <= f32::MAX as f64;
            if !finite(shape.bounds_offset.x)
                || !finite(shape.bounds_offset.y)
                || !finite(shape.bounds_width)
                || !finite(shape.bounds_height)
                || shape.bounds_width < 0.0
                || shape.bounds_height < 0.0
            {
                return Err("SVG shape bounds must be finite and nonnegative");
            }
            if shape.geometry == SvgShapeGeometry::kPath
                && (shape.path.is_empty() || shape.path[0].verb != PaintPathVerb::kMoveTo)
            {
                return Err("SVG path must start with move-to");
            }
            for command in &shape.path {
                for point in [command.control1, command.control2, command.point] {
                    if !finite(point.x) || !finite(point.y) {
                        return Err("SVG path coordinates must be finite native values");
                    }
                }
            }
            if let Some(transform) = shape.local_transform.as_ref() {
                for value in transform.values {
                    if !finite(value) {
                        return Err("SVG shape transform must be finite native values");
                    }
                }
            }
        } else if input.kind == NodeKind::kSvgShape {
            return Err("An SVG shape node requires resolved geometry");
        }
        if let Some(path) = data.svg_text_path.as_ref() {
            if input.kind != NodeKind::kSvgTextPath {
                return Err("SVG textPath geometry requires an SVG textPath node");
            }
            if !path.start_offset.is_finite() {
                return Err("SVG textPath offset must be finite");
            }
            for point in &path.points {
                if !point.x.is_finite()
                    || !point.y.is_finite()
                    || point.x.abs() > f32::MAX as f64
                    || point.y.abs() > f32::MAX as f64
                {
                    return Err("SVG textPath points must be finite native coordinates");
                }
            }
        }
        if data.supports_base_appearance && input.kind != NodeKind::kFormControl {
            return Err("Base-appearance capability requires a form control");
        }
        if data.select_uses_menu_list.is_some()
            && (input.kind != NodeKind::kFormControl
                || (data.form_control_type != Some(FormControlType::kSelectOne)
                    && data.form_control_type != Some(FormControlType::kSelectMultiple)))
        {
            return Err("Select presentation requires a select form control");
        }
        if let Some(sizing) = data.text_area_sizing.as_ref() {
            if input.kind != NodeKind::kFormControl
                || data.form_control_type != Some(FormControlType::kTextArea)
                || sizing.rows == 0
                || sizing.columns == 0
            {
                return Err("Textarea sizing requires a textarea and positive rows/columns");
            }
        }
        if data.text_field_sizing.is_some()
            && (input.kind != NodeKind::kFormControl
                || data.form_control_type.is_none()
                || data.form_control_type.is_some_and(|kind| {
                    (kind as i32) < FormControlType::kInputButton as i32
                        || (kind as i32) > FormControlType::kInputWeek as i32
                }))
        {
            return Err("Text-field sizing requires an input control");
        }
        if data.text_control_inner_editor
            || data.text_control_container
            || data.text_control_spin_button
            || data.text_control_placeholder
        {
            let part_count = data.text_control_inner_editor as u32
                + data.text_control_container as u32
                + data.text_control_spin_button as u32
                + data.text_control_placeholder as u32;
            if input.kind != NodeKind::kBox
                || part_count > 1
                || data.form_control_type.is_some()
                || data.file_upload_button
                || data.image_map_area
                || data.html_image
                || data.first_letter_pseudo
            {
                return Err("Text control part has conflicting node identity");
            }
        }
        if data
            .content_lock
            .is_some_and(|lock| !lock.is_locked && !lock.should_layout_children)
        {
            return Err("An unlocked content-lock context must allow child layout");
        }
    }
    Ok(())
}

// The second construction stage validates and converts each native value in
// source order. Rust String already enforces the UTF-8 validity that C++
// checks after accepting arbitrary std::string bytes for the file label.
// cpp: layoutng/internal/layout_node_metadata.cc:328-385
fn prepare_element_dimensions(
    element: &mut Element,
    input: &NativeNodeConstructionData,
) -> Result<(), &'static str> {
    let Some(data) = input.element.as_ref() else {
        return Ok(());
    };
    if data.file_no_file_label.is_some()
        && (input.kind != NodeKind::kFormControl
            || data.form_control_type != Some(FormControlType::kInputFile))
    {
        return Err("No-file-selected label requires an input-file control");
    }
    let remembered_size = |size: Option<f64>| -> Result<Option<LayoutUnit>, &'static str> {
        let Some(size) = size else {
            return Ok(None);
        };
        if !size.is_finite() || size < 0.0 || size > LayoutUnit::Max().ToDouble() {
            return Err("Remembered intrinsic size must be finite, nonnegative and representable");
        }
        Ok(Some(LayoutUnit::from_f64(size)))
    };
    element.remembered_inline_size_ = remembered_size(data.remembered_inline_size)?;
    element.remembered_block_size_ = remembered_size(data.remembered_block_size)?;
    if let Some(transform) = data.used_canvas_transform.as_ref() {
        for value in transform.values {
            if !value.is_finite() {
                return Err("Used Canvas transform coefficients must be finite");
            }
        }
        element.used_canvas_transform_ = Some(gfx::Transform::ColMajor(&transform.values));
    }
    if data
        .root_scroller_visible_size
        .is_some_and(|size| size.width < 0 || size.height < 0)
    {
        return Err("Root scroller visible size must be nonnegative");
    }
    let valid_natural_dimension = |value: Option<f64>| {
        value.map_or(true, |value| {
            value.is_finite() && value >= 0.0 && value <= LayoutUnit::Max().ToDouble()
        })
    };
    if !valid_natural_dimension(data.natural_width)
        || !valid_natural_dimension(data.natural_height)
        || data
            .natural_aspect_ratio
            .is_some_and(|ratio| !ratio.is_finite() || ratio <= 0.0)
        || !data.image_device_pixel_ratio.is_finite()
        || data.image_device_pixel_ratio <= 0.0
    {
        return Err("Replaced natural dimensions must be finite and nonnegative");
    }
    let has_replaced_data = data.natural_width.is_some()
        || data.natural_height.is_some()
        || data.natural_aspect_ratio.is_some()
        || data.image_device_pixel_ratio != 1.0
        || data.replaced_respects_css_overflow;
    if has_replaced_data && input.kind != NodeKind::kReplaced && input.kind != NodeKind::kSvgRoot {
        return Err("Replaced natural dimensions require a replaced node");
    }
    if data.image_device_pixel_ratio != 1.0 && !data.html_image {
        return Err("Image density correction requires HTML image identity");
    }
    let offset = data.scroll_offset;
    let valid_offset = |value: f64| value.is_finite() && value.abs() <= f32::MAX as f64;
    if !valid_offset(offset.x) || !valid_offset(offset.y) {
        return Err("Scroll offset must be finite and representable as a native float");
    }
    element.saved_layer_scroll_offset_ = ScrollOffset::new(offset.x as f32, offset.y as f32);
    Ok(())
}
