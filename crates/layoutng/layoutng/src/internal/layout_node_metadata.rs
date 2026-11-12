#![allow(non_camel_case_types)]

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use foundation::{
    gfx, AtomicString, LayoutUnit, MakeGarbageCollected, Member, ScopedCSSName,
    String as BlinkString, Visitor,
};
use layoutng_style::style::computed_style::ComputedStyle as StyleComputedStyle;
use layoutng_style::style::computed_style_constants::PseudoId;

use super::boundary::layout_boundary::ReusableLayoutEnvironment;
use super::form_control_types::FormControlType;
use super::layout_box::LayoutBox;
use super::layout_input::{
    CompatibilityMode, ComputedStyle as InputComputedStyle, ContentLockState, DocumentRole,
    ElementData, NodeConstructionData, NodeKind, PaintPathVerb, StyleResourceData, SvgLengthAdjust,
    SvgShapeGeometry, ViewportGeometry,
};
use super::layout_input_types::{ControlThemeMetrics, IntSize, ScrollbarThemeMetrics};
use super::layout_object::LayoutObject;
use super::scroll_types::ScrollOffset;
use super::svg_length_adjust_type::SVGLengthAdjustType;

#[path = "../../../layout_algo_svg/src/svg_metadata_validation.rs"]
mod layout_algo_svg_metadata_validation;
use layout_algo_svg_metadata_validation::ValidateSvgElementInput;
#[path = "../../../layout_invalidation/src/timeline_trigger_metadata.rs"]
mod layout_invalidation_timeline_trigger_metadata;
use layout_invalidation_timeline_trigger_metadata::PopulateTimelineTriggerNames;
#[path = "../../../layout_algo_list/src/list_marker_lookup.rs"]
mod layout_algo_list_marker_lookup;
#[path = "../../../layout_invalidation/src/element_metadata_input.rs"]
mod layout_invalidation_element_metadata_input;
#[path = "../../../layout_invalidation/src/metadata_append.rs"]
mod layout_invalidation_metadata_append;
#[path = "../../../layout_invalidation/src/node_input_state.rs"]
mod layout_invalidation_node_input_state;
#[path = "../../../layout_invalidation/src/node_metadata_tree.rs"]
mod layout_invalidation_node_metadata_tree;

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
    kDocumentFragmentPaintData = 11,
}

// cpp: layoutng/internal/layout_node_metadata.cc:16-49
pub struct StandaloneNodeData {
    pub editing: crate::internal::editing::forward::EditingNodeFacts,
    pub compatibility_mode: CompatibilityMode,
    pub printing: bool,
    pub should_print_backgrounds: bool,
    pub paginated: bool,
    pub fragmentainer_block_size: Option<f64>,
    pub vertical_scroll_enforced: bool,
    pub fullscreen_element_blocks_layout_view_scroll: bool,
    pub device_pixel_ratio: f64,
    pub canvas_draw_element_enabled: bool,
    pub prefer_non_composited_scrolling_enabled: bool,
    pub viewport: Option<ViewportGeometry>,
    pub media_panel_page: Option<super::layout_input::MediaPanelPageFacts>,
    pub minimum_font_physical_size: Option<f32>,
    pub scrollbar_theme: Option<ScrollbarThemeMetrics>,
    pub control_theme: ControlThemeMetrics,
    pub document_role: DocumentRole,
    pub input_id: u64,
    pub input_kind: NodeKind,
    pub input_is_style_generated: bool,
    pub input_style_resources: Arc<StyleResourceData>,
    pub input_first_line_style_resources: Option<Arc<StyleResourceData>>,
    pub debug_name: String,
    pub element_data: Option<ElementData>,
    pub reusable_layout_environment: Option<Arc<ReusableLayoutEnvironment>>,
    // Last font resolved by the constraint-space adapter. Construction may
    // temporarily install a placeholder style before that binding runs.
    bound_font: Member<font_engine::fonts::font::Font>,
}
impl StandaloneNodeData {
    #[inline(always)]
    pub fn new(input: &NodeConstructionData) -> Self {
        layout_invalidation_node_input_state::NewStandaloneNodeData(input)
    }
}

// C++ Member<T> fields retain their traced identity. The owning tree
// allocation and destruction order will be established with LayoutObjectTree.
// cpp: layoutng/internal/layout_node_metadata.h:67-164
pub struct Node {
    layout_detach_sink_: Option<std::rc::Rc<std::cell::RefCell<Vec<u64>>>>,
    input_tree_sealed_: bool,
    input_font_dirty_: bool,
    hit_test_node_owner_: Option<u64>,
    has_first_line_styles_: bool,
    first_line_style_: Member<StyleComputedStyle>,
    input_data_: Box<StandaloneNodeData>,
    global_input_owner_: Member<Node>,
    control_host_: Member<Element>,
    implicit_anchor_: Member<Element>,
    ultimate_originating_element_: Member<Element>,
    generated_scroll_marker_group_: Member<Element>,
    viewport_defining_element_: Member<Element>,
    type_: NodeType,
    is_container_: bool,
    style_: Member<StyleComputedStyle>,
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
        input: &NodeConstructionData,
        style: *const StyleComputedStyle,
        node_type: NodeType,
        container: bool,
    ) -> Result<Self, NodeMetadataError> {
        let input_data = Box::new(StandaloneNodeData::new(input));
        if style.is_null() {
            return Err(NodeMetadataError::MissingComputedStyle);
        }
        Ok(Self {
            layout_detach_sink_: None,
            input_tree_sealed_: false,
            input_font_dirty_: true,
            hit_test_node_owner_: None,
            has_first_line_styles_: false,
            first_line_style_: Member::default(),
            input_data_: input_data,
            global_input_owner_: Member::default(),
            control_host_: Member::default(),
            implicit_anchor_: Member::default(),
            ultimate_originating_element_: Member::default(),
            generated_scroll_marker_group_: Member::default(),
            viewport_defining_element_: Member::default(),
            type_: node_type,
            is_container_: container,
            style_: Member::from_ptr(style as *mut StyleComputedStyle),
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
        self.GlobalInputOwner().has_first_line_styles_
    }
    pub fn InputFirstLineStyle(&self) -> *const StyleComputedStyle {
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
        self.GlobalInputData().compatibility_mode
    }
    pub fn InputLineHeightQuirksMode(&self) -> bool {
        self.InputCompatibilityMode() != CompatibilityMode::kStandards
    }
    pub fn InputPrinting(&self) -> bool {
        self.GlobalInputData().printing
    }
    pub fn InputShouldPrintBackgrounds(&self) -> bool {
        self.GlobalInputData().should_print_backgrounds
    }
    pub fn InputPaginated(&self) -> bool {
        self.GlobalInputData().paginated
    }
    pub fn InputFragmentainerBlockSize(&self) -> Option<f64> {
        self.GlobalInputData().fragmentainer_block_size
    }
    pub fn InputVerticalScrollEnforced(&self) -> bool {
        self.GlobalInputData().vertical_scroll_enforced
    }
    pub fn InputFullscreenElementBlocksLayoutViewScroll(&self) -> bool {
        self.GlobalInputData()
            .fullscreen_element_blocks_layout_view_scroll
    }
    pub fn InputDevicePixelRatio(&self) -> f64 {
        self.GlobalInputData().device_pixel_ratio
    }
    pub fn InputElementCanvasTransformEnabled(&self) -> bool {
        self.input_data_
            .element_data
            .as_ref()
            .is_some_and(|data| data.element_canvas_transform_enabled)
    }
    pub fn InputCanvasDrawElementEnabled(&self) -> bool {
        self.GlobalInputData().canvas_draw_element_enabled
    }
    pub fn InputPreferNonCompositedScrollingEnabled(&self) -> bool {
        self.GlobalInputData()
            .prefer_non_composited_scrolling_enabled
    }
    pub fn InputMediaPanelPageFacts(&self) -> Option<super::layout_input::MediaPanelPageFacts> {
        self.GlobalInputData().media_panel_page
    }
    pub fn InputViewport(&self) -> &Option<ViewportGeometry> {
        &self.GlobalInputData().viewport
    }
    pub fn InputMinimumFontPhysicalSize(&self) -> Option<f32> {
        self.GlobalInputData().minimum_font_physical_size
    }
    pub fn InputScrollbarTheme(&self) -> &Option<ScrollbarThemeMetrics> {
        &self.GlobalInputData().scrollbar_theme
    }
    pub fn InputDocumentRole(&self) -> DocumentRole {
        self.input_data_.document_role
    }
    pub fn InputEditingFacts(&self) -> crate::internal::editing::forward::EditingNodeFacts {
        self.input_data_.editing
    }
    pub(crate) fn SetInputEditingFacts(
        &mut self,
        facts: crate::internal::editing::forward::EditingNodeFacts,
    ) {
        self.input_data_.editing = facts;
    }
    pub fn InputHitTestNodeOwner(&self) -> Option<u64> {
        self.hit_test_node_owner_
    }
    pub fn SetInputHitTestNodeOwner(&mut self, owner: Option<u64>) {
        self.hit_test_node_owner_ = owner;
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
    pub fn InputStyleResources(&self) -> &StyleResourceData {
        self.input_data_.input_style_resources.as_ref()
    }
    pub fn InputFirstLineStyleResources(&self) -> &Option<Arc<StyleResourceData>> {
        &self.input_data_.input_first_line_style_resources
    }
    pub fn InputDebugName(&self) -> &String {
        &self.input_data_.debug_name
    }
    pub fn InputIsHTMLAnchorElement(&self) -> bool {
        self.input_data_
            .element_data
            .as_ref()
            .is_some_and(|data| data.html_anchor)
    }
    pub fn InputIsHTMLCanvasElement(&self) -> bool {
        self.input_data_
            .element_data
            .as_ref()
            .is_some_and(|data| data.html_canvas)
    }
    pub fn InputIsHTMLHRElement(&self) -> bool {
        self.input_data_
            .element_data
            .as_ref()
            .is_some_and(|data| data.html_hr)
    }
    pub fn InputIsHTMLListOwner(&self) -> bool {
        self.input_data_
            .element_data
            .as_ref()
            .is_some_and(|data| data.html_list_owner)
    }
    pub fn InputOrderedListReversed(&self) -> bool {
        self.input_data_
            .element_data
            .as_ref()
            .is_some_and(|data| data.ordered_list_reversed)
    }
    pub fn InputOrderedListStart(&self) -> Option<i32> {
        self.input_data_
            .element_data
            .as_ref()
            .and_then(|data| data.ordered_list_start)
    }
    pub fn InputListItemExplicitValue(&self) -> Option<i32> {
        self.input_data_
            .element_data
            .as_ref()
            .and_then(|data| data.list_item_explicit_value)
    }
    pub fn InputIsHTMLMediaElement(&self) -> bool {
        self.input_data_
            .element_data
            .as_ref()
            .is_some_and(|data| data.html_media)
    }
    pub fn InputIsHTMLVideoElement(&self) -> bool {
        self.input_data_
            .element_data
            .as_ref()
            .is_some_and(|data| data.html_video)
    }
    /// HTMLMediaElement::UpdateLayoutObject updates the resident native input
    /// before invoking LayoutVideo::UpdateFromElement on that same object.
    pub fn SetVideoLayoutState(&mut self, state: super::layout_input::VideoLayoutState) {
        let element = self
            .input_data_
            .element_data
            .as_mut()
            .expect("video element input");
        assert!(element.html_video);
        element.video = Some(state);
        element.media_is_fullscreen = state.is_fullscreen;
    }
    pub fn InputCanvasLayoutSubtree(&self) -> bool {
        self.input_data_
            .element_data
            .as_ref()
            .is_some_and(|data| data.canvas_layout_subtree)
    }
    pub fn InputHasDrawableAttribute(&self) -> bool {
        self.input_data_
            .element_data
            .as_ref()
            .is_some_and(|data| data.drawable_attribute)
    }
    pub fn InputControlTheme(&self) -> &ControlThemeMetrics {
        &self.GlobalInputData().control_theme
    }

    // cpp: layoutng/internal/layout_node_metadata.h:105-105
    pub fn InputViewportDefiningElement(&self) -> *mut Element {
        self.GlobalInputOwner().viewport_defining_element_.Get()
    }

    // cpp: layoutng/internal/layout_node_metadata.h:115-126
    pub fn GetComputedStyle(&self) -> *const StyleComputedStyle {
        self.style_.Get()
    }
    pub fn ComputedStyleRef(&self) -> &StyleComputedStyle {
        assert!(!self.style_.Get().is_null());
        unsafe { &*self.style_.Get() }
    }
    pub fn SetComputedStyle(&mut self, style: *const StyleComputedStyle) {
        self.style_ = Member::from_ptr(style as *mut StyleComputedStyle);
    }
    // The owning layout tree updates source data without replacing node identity.
    #[inline(always)]
    pub fn UpdateInputData(&mut self, input: &NodeConstructionData) {
        layout_invalidation_node_input_state::UpdateNodeInputData(self, input)
    }

    // Scroll translation is paint state, not a font or geometry invalidation.
    pub(crate) fn ReflectInputScrollOffset(&mut self, offset: super::layout_input::Offset) {
        if let Some(data) = self.input_data_.element_data.as_mut() {
            data.scroll_offset = offset;
        }
    }

    pub(crate) fn UpdateInputScrollOffset(&mut self, offset: super::layout_input::Offset) {
        self.UpdateInputScrollOffsetWithType(
            offset,
            layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kClamping,
        );
    }

    pub(crate) fn UpdateInputScrollOffsetWithType(
        &mut self,
        offset: super::layout_input::Offset,
        scroll_type: layoutng_style::style::scroll_enums::mojom::blink::ScrollType,
    ) {
        let changed = self
            .input_data_
            .element_data
            .as_ref()
            .is_some_and(|data| data.scroll_offset != offset);
        self.ReflectInputScrollOffset(offset);
        let area = super::layout_scrollable_area::PaintLayerScrollableArea::FromNode(self);
        if !area.is_null() {
            // The resident area, not SavedLayerScrollOffset, owns live offsets.
            unsafe { &mut *area }.UpdateScrollOffset(
                ScrollOffset::new(offset.x as f32, offset.y as f32),
                scroll_type,
            );
        }
        if changed {
            let object = self.GetLayoutObject();
            if !object.is_null() {
                unsafe { &mut *object }.SetNeedsPaintPropertyUpdate();
            }
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
        layout_invalidation_metadata_append::DetachForTreeUpdate(self)
    }

    pub fn SetInputFirstLineStyle(&mut self, style: *const StyleComputedStyle) {
        self.first_line_style_ = Member::from_ptr(style as *mut StyleComputedStyle);
    }
    pub fn GetLayoutObject(&self) -> *mut LayoutObject {
        self.layout_object_.Get()
    }
    pub fn SetLayoutObject(&mut self, object: *mut LayoutObject) {
        self.layout_object_ = Member::from_ptr(object);
    }
    pub(crate) fn SetLayoutDetachSink(&mut self, sink: std::rc::Rc<std::cell::RefCell<Vec<u64>>>) {
        self.layout_detach_sink_ = Some(sink);
    }
    pub(crate) fn LayoutDetachNotification(
        &self,
        object: *const LayoutObject,
    ) -> Option<(std::rc::Rc<std::cell::RefCell<Vec<u64>>>, u64)> {
        // DestroyLeftoverChildren clears Node::layout_object_ immediately
        // before recursively destroying that same native child.
        if (!self.GetLayoutObject().is_null() && self.GetLayoutObject().cast_const() != object)
            || !self.IsElementNode()
            || self.InputIsStyleGenerated()
        {
            return None;
        }
        self.layout_detach_sink_
            .clone()
            .map(|sink| (sink, self.InputId()))
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
        self.input_data_.should_print_backgrounds = input.should_print_backgrounds;
        self.input_data_.paginated = input.fragmentainer_block_size.is_some();
        self.input_data_.fragmentainer_block_size = input.fragmentainer_block_size;
        self.input_data_.vertical_scroll_enforced = input.vertical_scroll_enforced;
        self.input_data_
            .fullscreen_element_blocks_layout_view_scroll =
            input.fullscreen_element_blocks_layout_view_scroll;
        self.input_data_.device_pixel_ratio = input.device_pixel_ratio;
        self.input_data_.media_panel_page = input.media_panel_page;
        self.input_data_.canvas_draw_element_enabled = input.canvas_draw_element_enabled;
        self.input_data_.prefer_non_composited_scrolling_enabled =
            input.prefer_non_composited_scrolling_enabled;
        // Host viewport inputs are CSS pixels. Chromium constraints and layout
        // objects use layout-zoom coordinates; convert once at this boundary,
        // independently of device scale, just like PrepareConstraints.
        self.input_data_.viewport = input.viewport.map(|mut viewport| {
            let scaled = |size: IntSize| IntSize {
                width: (size.width as f64 * input.layout_zoom_factor).round() as i32,
                height: (size.height as f64 * input.layout_zoom_factor).round() as i32,
            };
            viewport.size = scaled(viewport.size);
            viewport.root_scroller_visible_size = viewport.root_scroller_visible_size.map(scaled);
            viewport.transition_snapshot_size = viewport.transition_snapshot_size.map(scaled);
            viewport
        });
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

    // Rust accessors for the C++ friend NodeMetadataRelations. Keeping
    // the fields private to this module avoids exposing relation mutations to
    // ordinary callers while the implementation lives in boundary/.
    pub(crate) fn set_control_host(&mut self, host: *mut Element) {
        self.control_host_ = Member::from_ptr(host);
    }
    pub fn UltimateOriginatingElement(&self) -> *mut Element {
        self.ultimate_originating_element_.Get()
    }
    pub(crate) fn set_ultimate_originating_element(&mut self, element: *mut Element) {
        self.ultimate_originating_element_ = Member::from_ptr(element);
    }
    pub fn GeneratedScrollMarkerGroup(&self) -> *mut Element {
        self.generated_scroll_marker_group_.Get()
    }
    pub(crate) fn set_generated_scroll_marker_group(&mut self, group: *mut Element) {
        self.generated_scroll_marker_group_ = Member::from_ptr(group);
    }
    pub(crate) fn set_implicit_anchor(&mut self, anchor: *mut Element) {
        self.implicit_anchor_ = Member::from_ptr(anchor);
    }
    pub(crate) fn set_viewport_defining_element(&mut self, element: *mut Element) {
        self.viewport_defining_element_ = Member::from_ptr(element);
    }
    pub(crate) fn set_global_input_owner(&mut self, owner: *mut Node) {
        self.global_input_owner_ = Member::from_ptr(owner);
    }
    pub(crate) fn seal_input_tree(&mut self) {
        self.input_tree_sealed_ = true;
    }

    // Document's current LayoutView is retained by the canonical global input
    // owner; this is identity, not another view/geometry representation.
    pub(crate) fn InputLayoutView(&self) -> *mut super::layout_view::LayoutView {
        foundation::DynamicTo::<super::layout_view::LayoutView>(
            self.GlobalInputOwner().GetLayoutObject(),
        )
    }
    fn GlobalInputOwner(&self) -> &Node {
        let owner = self.global_input_owner_.Get();
        if owner.is_null() {
            self
        } else {
            unsafe { &*owner }
        }
    }

    fn GlobalInputData(&self) -> &StandaloneNodeData {
        &self.GlobalInputOwner().input_data_
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:81-83
    pub fn GetLayoutBox(&self) -> *mut LayoutBox {
        foundation::DynamicTo::<LayoutBox>(self.GetLayoutObject())
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:191-200
    pub fn IsFirstLetterPseudoElement(&self) -> bool {
        self.ComputedStyleRef().StyleType() == PseudoId::kPseudoIdFirstLetter
    }
    // cpp: layoutng/internal/layout_node_metadata.cc:179-189
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.control_host_);
        visitor.Trace(&self.implicit_anchor_);
        visitor.Trace(&self.ultimate_originating_element_);
        visitor.Trace(&self.generated_scroll_marker_group_);
        visitor.Trace(&self.viewport_defining_element_);
        visitor.Trace(&self.global_input_owner_);
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
        input: &NodeConstructionData,
        style: *const StyleComputedStyle,
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
        layout_invalidation_metadata_append::AppendChild(self, child)
    }

    pub(crate) fn InsertForTreeUpdate(&mut self, child: &mut Node, before: *mut Node) {
        layout_invalidation_metadata_append::InsertForTreeUpdate(self, child, before)
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

// cpp: third_party/blink/renderer/core/layout/layout_object.h:3309-3313
#[allow(non_snake_case)]
pub fn ContentLockBlocksChildPrePaint(node: *const Node) -> bool {
    if node.is_null() || !unsafe { &*node }.IsElementNode() {
        return false;
    }
    let element = unsafe { &*node.cast::<Element>() };
    element
        .InputContentLockState()
        .is_some_and(|state| !state.should_prepaint_children)
}

// cpp: third_party/blink/renderer/core/layout/layout_object.h:3315-3319
#[allow(non_snake_case)]
pub fn ContentLockBlocksChildPaint(node: *const Node) -> bool {
    if node.is_null() || !unsafe { &*node }.IsElementNode() {
        return false;
    }
    let element = unsafe { &*node.cast::<Element>() };
    element
        .InputContentLockState()
        .is_some_and(|state| !state.should_paint_children)
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
pub struct NodeMetadataRelations;

// cpp: layoutng/internal/layout_node_metadata.h:249-253
// The constructor dispatch belongs to boundary/node_metadata_input.cc.
pub use super::boundary::node_metadata_input::CreateElementMetadata;

// cpp: layoutng/internal/layout_node_metadata.h:266-274
// Foundation's DynamicTo adapter can delegate these C++ DowncastTraits
// predicates when the source metadata tree is connected.
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
        input: &NodeConstructionData,
        style: *const StyleComputedStyle,
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

// Element and ContainerNode also begin with Node; source downcasts must
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
    pub fn InputRegionCaptureCropId(&self) -> Option<[u64; 2]> {
        self.InputElementData()
            .as_ref()
            .and_then(|data| data.region_capture_crop_id)
    }
    pub fn InputTrackedElementSubRects(
        &self,
    ) -> &[crate::internal::layout_input::TrackedElementSubRect] {
        self.InputElementData()
            .as_ref()
            .map_or(&[], |data| data.tracked_element_sub_rects.as_slice())
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
        layout_algo_list_marker_lookup::InputListMarker(self)
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
