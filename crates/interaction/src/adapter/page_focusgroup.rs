//! Live DOM/Layout leaves for the translated focusgroup controller utilities.
//! Native virtual dispatch follows the canonical DOM allocation class; no
//! focusgroup attributes are reparsed and no document/geometry tree is copied.

use dom::{DOMInputType, Document};
use foundation::focusgroup_flags::{FocusgroupData, FocusgroupFlags as Flags, NativeArrowKeyClass};
use interaction_default::core::{
    dom::element::{self, ElementNativeArrowKeyState},
    html::{
        forms::{html_input_element, html_select_element, html_text_area_element},
        html_element::{self, HTMLElementNativeArrowKeyState},
    },
    page::{
        focus_navigation::FocusNavigationNode as Node, focusgroup_controller_utils::FocusgroupTree,
    },
};

use super::{FocusNavigationLayout, FocusNavigationPolicy};
use crate::adapter::page_element_focusability::PageElementFocusability;

pub(super) struct PageFocusgroup<'a> {
    document: &'a Document,
    layout: &'a dyn FocusNavigationLayout,
    focused: Option<u64>,
    policy: FocusNavigationPolicy,
    focusability: PageElementFocusability<'a>,
}

impl<'a> PageFocusgroup<'a> {
    pub(super) fn new(
        document: &'a Document,
        layout: &'a dyn FocusNavigationLayout,
        focused: Option<u64>,
        policy: FocusNavigationPolicy,
    ) -> Self {
        Self {
            document,
            layout,
            focused,
            policy,
            focusability: PageElementFocusability::new(document, layout, focused),
        }
    }
    fn Index(&self, node: Node) -> usize {
        self.document
            .FindNodeById(node.0)
            .expect("focusgroup node belongs to the live Document")
    }
    fn Identity(&self, index: usize) -> Node {
        Node(self.document.Node(index).Id())
    }
    fn NativeAxes(&self, index: usize) -> Flags {
        let base = NativeArrowKeyElement { owner: self, index };
        match self.document.NativeArrowKeyClass(index) {
            NativeArrowKeyClass::Node => Flags::kNone,
            NativeArrowKeyClass::Element => element::NativeArrowKeyAxes(&base),
            NativeArrowKeyClass::HTMLElement => html_element::NativeArrowKeyAxes(&base),
            NativeArrowKeyClass::Input => {
                let input_type = self
                    .document
                    .InputType(index)
                    .expect("HTMLInputElement owns its InputType");
                html_input_element::NativeArrowKeyAxes(
                    input_type.IsTextFieldInputType(),
                    crate::adapter::page_input_type::SourceType(input_type).IsSteppable(),
                    input_type == DOMInputType::kRadio,
                    &base,
                )
            }
            NativeArrowKeyClass::Select => html_select_element::NativeArrowKeyAxes(),
            NativeArrowKeyClass::TextArea => html_text_area_element::NativeArrowKeyAxes(),
            // Source: html_frame_owner_element.cc:413-417.
            NativeArrowKeyClass::Frame => Flags::kInline | Flags::kBlock,
            // Source: html_media_element.cc:3344-3369,4810-4817. The current
            // Document has no Settings owner (the source optional Settings
            // check therefore has no value); use actual media state below.
            NativeArrowKeyClass::Media => {
                let show_controls = self.document.IsActive()
                    && self
                        .document
                        .MediaElementState(index)
                        .user_wants_controls_visible
                        .get()
                        .unwrap_or_else(|| {
                            self.document
                                .Node(index)
                                .FindAttribute("controls")
                                .is_some()
                                || self.document.FullscreenElement() == Some(index)
                                || !self.document.ScriptingEnabled()
                        });
                if show_controls {
                    Flags::kInline | Flags::kBlock
                } else {
                    html_element::NativeArrowKeyAxes(&base)
                }
            }
        }
    }
}

impl FocusgroupTree for PageFocusgroup<'_> {
    fn Enabled(&self) -> bool {
        self.document.FocusgroupFeatures().enabled
    }
    fn ParentElement(&self, node: Node) -> Option<Node> {
        self.document
            .FlatTreeParentElement(self.Index(node))
            .map(|node| self.Identity(node))
    }
    fn NextElement(&self, node: Node, skip_subtree: bool) -> Option<Node> {
        self.document
            .FlatTreeNextElement(self.Index(node), skip_subtree)
            .map(|node| self.Identity(node))
    }
    fn PreviousElement(&self, node: Node, skip_subtree: bool) -> Option<Node> {
        self.document
            .FlatTreePreviousElement(self.Index(node), skip_subtree)
            .map(|node| self.Identity(node))
    }
    fn Data(&self, node: Node) -> FocusgroupData {
        self.document.FocusgroupData(self.Index(node))
    }
    fn IsInTopLayer(&self, node: Node) -> bool {
        self.document.IsInTopLayer(self.Index(node))
    }
    fn IsFocusable(&self, node: Node) -> bool {
        self.focusability.IsFocusable(self.Index(node))
    }
    fn IsKeyboardFocusable(&self, node: Node) -> bool {
        self.focusability.IsKeyboardFocusable(
            self.Index(node),
            self.policy.tabs_to_links,
            self.policy.spatial_navigation_enabled,
        )
    }
    fn FocusedElement(&self) -> Option<Node> {
        self.focused
            .and_then(|id| self.document.FindNodeById(id))
            .map(|node| self.Identity(node))
    }
    fn LastFocused(&self, owner: Node) -> Option<Node> {
        self.document
            .FocusgroupLastFocused(self.Index(owner))
            .and_then(|id| self.document.FindNodeById(id))
            .map(|node| self.Identity(node))
    }
    fn IsFocusgroupStart(&self, node: Node) -> bool {
        self.document
            .Node(self.Index(node))
            .FindAttribute("focusgroupstart")
            .is_some()
    }
    fn NativeArrowKeyAxes(&self, node: Node) -> Flags {
        self.NativeAxes(self.Index(node))
    }
    fn ReadingFlowContainer(&self, node: Node) -> Option<Node> {
        // Source: focus_controller.cc:1763-1782, called with true.
        if self
            .layout
            .Facts(node.0)
            .is_some_and(|facts| facts.is_reading_flow_container)
        {
            return Some(node);
        }
        let parent = self.document.LayoutTreeBuilderParent(self.Index(node))?;
        self.layout
            .Facts(self.document.Node(parent).Id())
            .is_some_and(|facts| facts.is_reading_flow_container)
            .then(|| self.Identity(parent))
    }
    fn ReadingFlowElements(&self, node: Node) -> Vec<Node> {
        self.layout
            .ReadingFlowNodes(node.0)
            .into_iter()
            .filter_map(|id| {
                let index = self.document.FindNodeById(id)?;
                (self.document.Node(index).Type() == dom::DOMNodeType::kElement
                    && self.document.PseudoElementFacts(index).is_none())
                .then_some(Node(id))
            })
            .collect()
    }
}

struct NativeArrowKeyElement<'a, 'b> {
    owner: &'a PageFocusgroup<'b>,
    index: usize,
}
impl ElementNativeArrowKeyState for NativeArrowKeyElement<'_, '_> {
    fn IsFocusable(&self) -> bool {
        self.owner.focusability.IsFocusable(self.index)
    }
    fn ScrollsOverflowX(&self) -> bool {
        self.owner
            .document
            .ResolvedStyleFor(self.index)
            .and_then(|style| style.ComputedStyle())
            .is_some_and(|style| style.ScrollsOverflowX())
    }
    fn ScrollsOverflowY(&self) -> bool {
        self.owner
            .document
            .ResolvedStyleFor(self.index)
            .and_then(|style| style.ComputedStyle())
            .is_some_and(|style| style.ScrollsOverflowY())
    }
}
impl HTMLElementNativeArrowKeyState for NativeArrowKeyElement<'_, '_> {
    fn IsContentEditableForBinding(&self) -> bool {
        self.owner.document.IsContentEditableForBinding(self.index)
    }
    fn ElementNativeArrowKeyAxes(&self) -> Flags {
        element::NativeArrowKeyAxes(self)
    }
}

use crate::adapter::{event::EventTargetResolver, page_event_dispatcher::EventDispatcher};
use interaction_default::core::page::{
    focusgroup_controller::{self, FocusgroupContext},
    focusgroup_controller_utils::{DirectionalFocusgroupTree, FocusgroupDirection},
    grid_focusgroup_structure_info::{
        AutomaticGridFocusgroupStructureInfo, GridLayout, GridQuery, GridResult,
    },
};
use interaction_dispatch::core::events::keyboard_event::KeyboardEventRef;
use layoutng_assembly::layout_engine::{LayoutOutput, TableFocusgroupCursor};

struct BorrowedGrid<'a> {
    document: &'a Document,
    output: LayoutOutput<'a>,
}
impl<'a> GridLayout for BorrowedGrid<'a> {
    type Object = TableFocusgroupCursor<'a>;
    fn Layout(&self, node: Node) -> Option<Self::Object> {
        self.output.Node(node.0)?.TableFocusgroupCursor()
    }
    fn Node(&self, object: Self::Object) -> Option<Node> {
        object.NodeId().map(Node)
    }
    fn Facts(
        &self,
        object: Self::Object,
    ) -> Option<foundation::table_focusgroup::TableFocusgroupFacts<Self::Object>> {
        object.Facts()
    }
    fn Behavior(&self, node: Node) -> foundation::focusgroup_flags::FocusgroupBehavior {
        self.document
            .FocusgroupData(
                self.document
                    .FindNodeById(node.0)
                    .expect("grid root retains DOM owner"),
            )
            .behavior
    }
    fn HTMLRowCell(&self, row: Self::Object, index: u32) -> Option<Self::Object> {
        let row = self.document.FindNodeById(row.NodeId()?)?;
        let cell = self.document.TableRowCellAtIndex(row, index)?;
        self.Layout(Node(self.document.Node(cell).Id()))
    }
}
/// The host borrows its resident layout for this one source helper operation.
/// Anonymous sections/rows retain native cursor identities inside the call;
/// only canonical DOM node IDs cross back to the interaction owner.
pub fn QueryGridFocusgroup(
    document: &Document,
    output: LayoutOutput<'_>,
    root: u64,
    query: GridQuery,
) -> Option<GridResult> {
    let layout = BorrowedGrid { document, output };
    Some(AutomaticGridFocusgroupStructureInfo::new(&layout, Node(root))?.Query(query))
}

struct KeyboardFocusgroup<'a, 'listener, 'frame> {
    navigation: &'a super::PageFocusNavigation,
    dispatcher: &'a EventDispatcher<'listener, 'frame>,
}
impl KeyboardFocusgroup<'_, '_, '_> {
    fn Read<T>(&self, read: impl FnOnce(&PageFocusgroup<'_>) -> T) -> T {
        let owner = &self.navigation.owner;
        let document = owner.document.borrow();
        read(&PageFocusgroup::new(
            document.GetDocument(),
            owner.layout.as_ref(),
            owner.state.borrow().focused_node_id,
            owner.special.KeyboardPolicy(),
        ))
    }
}
impl FocusgroupTree for KeyboardFocusgroup<'_, '_, '_> {
    fn Enabled(&self) -> bool {
        self.Read(|tree| tree.Enabled())
    }
    fn ParentElement(&self, node: Node) -> Option<Node> {
        self.Read(|tree| tree.ParentElement(node))
    }
    fn NextElement(&self, node: Node, skip: bool) -> Option<Node> {
        self.Read(|tree| tree.NextElement(node, skip))
    }
    fn PreviousElement(&self, node: Node, skip: bool) -> Option<Node> {
        self.Read(|tree| tree.PreviousElement(node, skip))
    }
    fn Data(&self, node: Node) -> FocusgroupData {
        self.Read(|tree| tree.Data(node))
    }
    fn IsInTopLayer(&self, node: Node) -> bool {
        self.Read(|tree| tree.IsInTopLayer(node))
    }
    fn IsFocusable(&self, node: Node) -> bool {
        self.Read(|tree| tree.IsFocusable(node))
    }
    fn IsKeyboardFocusable(&self, node: Node) -> bool {
        self.Read(|tree| tree.IsKeyboardFocusable(node))
    }
    fn FocusedElement(&self) -> Option<Node> {
        self.Read(|tree| tree.FocusedElement())
    }
    fn LastFocused(&self, node: Node) -> Option<Node> {
        self.Read(|tree| tree.LastFocused(node))
    }
    fn IsFocusgroupStart(&self, node: Node) -> bool {
        self.Read(|tree| tree.IsFocusgroupStart(node))
    }
    fn NativeArrowKeyAxes(&self, node: Node) -> Flags {
        self.Read(|tree| tree.NativeArrowKeyAxes(node))
    }
    fn ReadingFlowContainer(&self, node: Node) -> Option<Node> {
        self.Read(|tree| tree.ReadingFlowContainer(node))
    }
    fn ReadingFlowElements(&self, node: Node) -> Vec<Node> {
        self.Read(|tree| tree.ReadingFlowElements(node))
    }
}
impl DirectionalFocusgroupTree for KeyboardFocusgroup<'_, '_, '_> {
    fn V2Enabled(&self) -> bool {
        self.navigation
            .owner
            .document
            .borrow()
            .GetDocument()
            .FocusgroupFeatures()
            .v2
    }
    fn WritingDirection(&self, node: Node) -> Option<foundation::WritingDirectionMode> {
        let owner = self.navigation.owner.document.borrow();
        let document = owner.GetDocument();
        Some(
            document
                .ResolvedStyleFor(document.FindNodeById(node.0)?)?
                .ComputedStyle()?
                .GetWritingDirection(),
        )
    }
    fn IsTable(&self, node: Node) -> bool {
        self.navigation
            .owner
            .layout
            .Facts(node.0)
            .is_some_and(|facts| facts.is_table)
    }
    fn IsTableCell(&self, node: Node) -> bool {
        self.navigation
            .owner
            .layout
            .Facts(node.0)
            .is_some_and(|facts| facts.is_table_cell)
    }
}
impl FocusgroupContext for KeyboardFocusgroup<'_, '_, '_> {
    fn CaretBrowsingEnabled(&self) -> bool {
        self.dispatcher
            .input_frame
            .as_ref()
            .expect("keyboard LocalFrame input owner")
            .caret_browsing_enabled
            .get()
    }
    fn HasEditContext(&self, node: Node) -> bool {
        self.FocusedElement() == Some(node)
            && self
                .dispatcher
                .input_method_state
                .as_ref()
                .is_some_and(|state| state.GetActiveEditContext().is_some())
    }
    fn IsEditable(&self, node: Node) -> bool {
        let owner = self.navigation.owner.document.borrow();
        let document = owner.GetDocument();
        document
            .FindNodeById(node.0)
            .is_some_and(|index| document.IsEditableForEditing(index))
    }
    fn UpdateStyleAndLayoutTreeForElement(&self, _: Node) {
        self.navigation.owner.layout.UpdateStyleAndLayoutForFocus();
    }
    fn UpdateAllLifecyclePhasesExceptPaint(&self, _: Node) {
        self.navigation.owner.layout.UpdateStyleAndLayoutForFocus();
    }
    fn Grid(&self, root: Node, query: GridQuery) -> GridResult {
        self.navigation
            .owner
            .layout
            .GridFocusgroup(root.0, query)
            .expect("prepared grid focusgroup native LayoutTable owner")
    }
    fn Focus(&self, node: Node, direction: FocusgroupDirection) {
        use interaction_default::core::dom::{
            element::SelectionBehaviorOnFocus, focus_params::FocusParams,
        };
        let Some(target) = self.navigation.owner.targets.ResolveTarget(node.0) else {
            return;
        };
        self.dispatcher.FocusKeyboardTarget(&target, &FocusParams {
            selection_behavior: SelectionBehaviorOnFocus::kReset,
            r#type: if interaction_default::core::page::focusgroup_controller_utils::IsDirectionForward(direction) { interaction_default::public::mojom::input::focus_type::FocusType::kForward } else { interaction_default::public::mojom::input::focus_type::FocusType::kBackward },
            ..Default::default()
        });
    }
}
pub(super) fn HandleKeyboardEvent(
    navigation: &super::PageFocusNavigation,
    dispatcher: &EventDispatcher<'_, '_>,
    event: &KeyboardEventRef,
) -> bool {
    focusgroup_controller::HandleKeyboardEvent(
        &KeyboardFocusgroup {
            navigation,
            dispatcher,
        },
        event,
    )
}
