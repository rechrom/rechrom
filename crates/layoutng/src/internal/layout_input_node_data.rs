#![allow(non_snake_case)]

use foundation::{AtomicString, DynamicTo, EDisplay, LayoutUnit, UnsupportedLayout, Visitor};
use layoutng_style::style::computed_style::ComputedStyle;

use super::form_control_types::FormControlType;
use super::form_node_metadata::HTMLInputElement;
use super::layout_box::LayoutBox;
use super::layout_input_node::LayoutInputNode;
use super::layout_node_metadata::{Element, ElementType, Node};
use super::layout_object::LayoutObject;
use super::layout_pass_scope::LayoutPassScope;

impl LayoutInputNode {
    // Rust access to the pending C++ `box_` Member's existing GetLayoutBox entry.
    fn box_ref(&self) -> &LayoutBox {
        unsafe { &*self.GetLayoutBox() }
    }

    // cpp: layoutng/internal/layout_input_node_data.cc:15
    pub fn IsBlockFlow(&self) -> bool {
        self.IsBlock() && self.box_ref().IsLayoutBlockFlow()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:17
    pub fn IsBlockInInline(&self) -> bool {
        self.box_ref().IsBlockInInline()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:19
    pub fn IsCustom(&self) -> bool {
        self.IsBlock() && self.box_ref().IsLayoutCustom()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:21
    pub fn IsColumnSpanAll(&self) -> bool {
        self.IsBlock() && self.box_ref().IsColumnSpanAll()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:23
    pub fn IsFloating(&self) -> bool {
        self.IsBlock() && self.box_ref().IsFloating()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:25-27
    pub fn IsOutOfFlowPositioned(&self) -> bool {
        self.IsBlock() && self.box_ref().IsOutOfFlowPositioned()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:29
    pub fn IsReplaced(&self) -> bool {
        self.box_ref().IsLayoutReplaced()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:31-33
    pub fn IsAbsoluteContainer(&self) -> bool {
        self.box_ref().CanContainAbsolutePositionObjects()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:35-37
    pub fn IsFixedContainer(&self) -> bool {
        self.box_ref().CanContainFixedPositionObjects()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:39
    pub fn IsBody(&self) -> bool {
        self.IsBlock() && self.box_ref().IsBody()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:41
    pub fn IsView(&self) -> bool {
        self.IsBlock() && self.box_ref().IsLayoutView()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:43
    pub fn IsDocumentElement(&self) -> bool {
        self.box_ref().IsDocumentElement()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:45
    pub fn IsFlexItem(&self) -> bool {
        self.IsBlock() && self.box_ref().IsFlexItem()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:47
    pub fn IsFlexibleBox(&self) -> bool {
        self.IsBlock() && self.box_ref().IsFlexibleBox()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:49
    pub fn IsGrid(&self) -> bool {
        self.IsBlock() && self.box_ref().IsLayoutGrid()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:51
    pub fn IsGridLanes(&self) -> bool {
        self.IsBlock() && self.box_ref().IsLayoutGridLanes()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:53
    pub fn IsListItem(&self) -> bool {
        self.IsBlock() && self.box_ref().IsLayoutListItem()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:55-57
    pub fn IsListMarker(&self) -> bool {
        self.IsBlock() && self.box_ref().IsLayoutOutsideListMarker()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:59-65
    pub fn ListMarkerOccupiesWholeLine(&self) -> bool {
        debug_assert!(self.IsListMarker());
        let algorithms = LayoutPassScope::Algorithms();
        if algorithms.is_null() {
            std::panic::panic_any(UnsupportedLayout::new(
                "list layout module is not installed",
            ));
        }
        let callback = unsafe { &*algorithms }
            .list_support
            .marker_occupies_whole_line
            .unwrap_or_else(|| {
                std::panic::panic_any(UnsupportedLayout::new(
                    "list layout module is not installed",
                ))
            });
        callback(unsafe { &*self.GetLayoutBox().cast::<LayoutObject>() })
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:67-69
    pub fn IsButtonOrInputButton(&self) -> bool {
        self.IsBlock() && self.box_ref().IsButtonOrInputButton()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:71
    pub fn IsFieldsetContainer(&self) -> bool {
        self.IsBlock() && self.box_ref().IsFieldset()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:73
    pub fn IsInitialLetterBox(&self) -> bool {
        self.box_ref().IsInitialLetterBox()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:75
    pub fn IsMedia(&self) -> bool {
        self.box_ref().IsMedia()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:77
    pub fn IsSemiReplaced(&self) -> bool {
        self.IsBlock() && self.box_ref().IsSemiReplaced()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:79-81
    pub fn IsRenderedLegend(&self) -> bool {
        self.IsBlock() && self.box_ref().IsRenderedLegend()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:83
    pub fn IsTable(&self) -> bool {
        self.IsBlock() && self.box_ref().IsTable()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:85
    pub fn IsTextCombine(&self) -> bool {
        self.box_ref().IsLayoutTextCombine()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:87
    pub fn IsTableCaption(&self) -> bool {
        self.IsBlock() && self.box_ref().IsTableCaption()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:89
    pub fn IsTableSection(&self) -> bool {
        self.IsBlock() && self.box_ref().IsTableSection()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:91
    pub fn IsTableRow(&self) -> bool {
        self.IsBlock() && self.box_ref().IsTableRow()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:93
    pub fn IsTableCell(&self) -> bool {
        self.IsBlock() && self.box_ref().IsTableCell()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:95-97
    pub fn IsTableCol(&self) -> bool {
        self.Style().Display() == EDisplay::kTableColumn
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:99-101
    pub fn IsTableColgroup(&self) -> bool {
        self.Style().Display() == EDisplay::kTableColumnGroup
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:103
    pub fn IsTextArea(&self) -> bool {
        self.box_ref().IsTextArea()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:105
    pub fn IsTextControl(&self) -> bool {
        self.box_ref().IsTextControl()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:107
    pub fn IsTextField(&self) -> bool {
        self.box_ref().IsTextField()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:109-113
    pub fn IsSlider(&self) -> bool {
        let input = DynamicTo::<HTMLInputElement>(self.box_ref().GetNode());
        !input.is_null() && unsafe { &*input }.FormControlType() == FormControlType::kInputRange
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:115-121
    pub fn IsSliderThumb(&self) -> bool {
        let node = self.GetDOMNode();
        let element = if node.is_null() {
            std::ptr::null_mut()
        } else {
            DynamicTo::<Element>(node)
        };
        self.IsBlock()
            && !element.is_null()
            && unsafe { &*element }.GetElementType() == ElementType::kSliderThumbElement
            && unsafe { &*node }.IsInUserAgentShadowRoot()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:123
    pub fn IsMathRoot(&self) -> bool {
        self.box_ref().IsMathMLRoot()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:125
    pub fn IsMathML(&self) -> bool {
        self.box_ref().IsMathML()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:127
    pub fn IsAnonymous(&self) -> bool {
        self.box_ref().IsAnonymous()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:129
    pub fn IsAnonymousBlockFlow(&self) -> bool {
        self.box_ref().IsAnonymousBlockFlow()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:131-134
    pub fn IsQuirkyContainer(&self) -> bool {
        self.box_ref().InQuirksModeForLayout()
            && (self.box_ref().IsBody() || self.box_ref().IsTableCell())
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:136-138
    pub fn IsHorizontalWritingMode(&self) -> bool {
        self.box_ref().IsHorizontalWritingMode()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:140-142
    pub fn IsHorizontalTypographicMode(&self) -> bool {
        self.box_ref().IsHorizontalTypographicMode()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:144-149
    pub fn IsMonolithic(&self) -> bool {
        if self.IsInline() {
            return true;
        }
        self.box_ref().IsMonolithic()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:151-153
    pub fn PageName(&self) -> AtomicString {
        if self.IsBlock() {
            self.Style().Page().clone()
        } else {
            AtomicString::default()
        }
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:155-157
    pub fn IsScrollContainer(&self) -> bool {
        self.IsBlock() && self.box_ref().IsScrollContainer()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:159-161
    pub fn CreatesNewFormattingContext(&self) -> bool {
        self.IsBlock() && self.box_ref().CreatesNewFormattingContext()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:163
    pub fn GetDOMNode(&self) -> *mut Node {
        self.box_ref().GetNode()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:165
    pub fn EnclosingDOMNode(&self) -> *mut Node {
        self.box_ref().EnclosingNode()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:167
    pub fn Style(&self) -> &ComputedStyle {
        self.box_ref().StyleRef()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:169-171
    pub fn ShouldApplySizeContainment(&self) -> bool {
        self.box_ref().ShouldApplySizeContainment()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:173-175
    pub fn ShouldApplyInlineSizeContainment(&self) -> bool {
        self.box_ref().ShouldApplyInlineSizeContainment()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:177-179
    pub fn ShouldApplyBlockSizeContainment(&self) -> bool {
        self.box_ref().ShouldApplyBlockSizeContainment()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:181-183
    pub fn CanMatchSizeContainerQueries(&self) -> bool {
        self.box_ref().CanMatchSizeContainerQueries()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:185-187
    pub fn OverrideIntrinsicContentInlineSize(&self) -> LayoutUnit {
        self.box_ref().OverrideIntrinsicContentInlineSize()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:189-191
    pub fn OverrideIntrinsicContentBlockSize(&self) -> LayoutUnit {
        self.box_ref().OverrideIntrinsicContentBlockSize()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:193-195
    pub fn DefaultIntrinsicContentInlineSize(&self) -> LayoutUnit {
        self.box_ref().DefaultIntrinsicContentInlineSize()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:197-200
    pub fn DefaultIntrinsicContentBlockSize(&self, children_have_geometry: bool) -> LayoutUnit {
        self.box_ref()
            .DefaultIntrinsicContentBlockSize(children_have_geometry)
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:202-204
    pub fn ChildLayoutBlockedByDisplayLock(&self) -> bool {
        self.box_ref().ChildLayoutBlockedByDisplayLock()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:206-208
    pub fn CanTraversePhysicalFragments(&self) -> bool {
        self.box_ref().CanTraversePhysicalFragments()
    }
    // cpp: layoutng/internal/layout_input_node_data.cc:210
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.box_);
    }
}
