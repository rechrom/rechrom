// C++: layoutng_inline/offset_mapping.h/.cc. Defined methods are translated;
// position APIs lacking definitions in this checkout retain typed providers.
#![allow(non_snake_case, non_camel_case_types)]

// The block-only host needs only type identity until the mutual LayoutNG /
// fragment-tree / inline assembly is installed.
#[cfg(not(feature = "translation_in_progress"))]
pub enum OffsetMapping {}

#[cfg(feature = "translation_in_progress")]
mod translated {
    use std::cell::Cell;

    use font_engine::text::implementation::character::Character;
    use foundation::{
        DynamicTo, HeapHashMap, HeapVector, Member, String, Traceable, UChar, Visitor,
    };
    use layoutng::internal::editing::forward::{EphemeralRange, Position, PositionAnchorType};
    use layoutng::internal::inline_node::InlineNode;
    use layoutng::internal::layout_block_flow::LayoutBlockFlow;
    use layoutng::internal::layout_node_metadata::Node;
    use layoutng::internal::layout_object::LayoutObject;
    use layoutng::internal::layout_text::LayoutText;

    use crate::layout_text_fragment::LayoutTextFragment;

    // cpp: layoutng_inline/offset_mapping.h:22-22
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum OffsetMappingUnitType {
        kIdentity,
        kCollapsed,
        kVariable,
    }

    // cpp: layoutng_inline/offset_mapping.h:42-101
    #[derive(Clone)]
    pub struct OffsetMappingUnit {
        pub(crate) type_: OffsetMappingUnitType,
        pub(crate) layout_object_: Member<LayoutObject>,
        pub(crate) dom_start_: u32,
        pub(crate) dom_end_: u32,
        pub(crate) text_content_start_: u32,
        pub(crate) text_content_end_: u32,
    }

    impl OffsetMappingUnit {
        // cpp: layoutng_inline/offset_mapping.cc:35-50
        pub fn new(
            type_: OffsetMappingUnitType,
            layout_object: &LayoutObject,
            dom_start: u32,
            dom_end: u32,
            text_content_start: u32,
            text_content_end: u32,
        ) -> Self {
            let unit = Self {
                type_,
                layout_object_: Member::from_ptr(layout_object as *const _ as *mut _),
                dom_start_: dom_start,
                dom_end_: dom_end,
                text_content_start_: text_content_start,
                text_content_end_: text_content_end,
            };
            unit.AssertValid();
            unit
        }

        // cpp: layoutng_inline/offset_mapping.cc:52-69
        pub fn AssertValid(&self) {
            debug_assert!(self.dom_start_ <= self.dom_end_);
            debug_assert!(self.text_content_start_ <= self.text_content_end_);
            let object = self.GetLayoutObject();
            if object.IsText()
                && !unsafe { &*(object as *const _ as *const LayoutText) }.IsWordBreak()
            {
                let layout_text = unsafe { &*(object as *const _ as *const LayoutText) };
                let text_start = if self.AssociatedNode().is_null() {
                    0
                } else {
                    let fragment = DynamicTo::<LayoutTextFragment>(self.layout_object_.Get());
                    if fragment.is_null() {
                        layout_text.TextStartOffset()
                    } else {
                        unsafe { &*fragment }.TextStartOffset()
                    }
                };
                debug_assert!(self.dom_end_ >= text_start);
            } else {
                debug_assert_eq!(self.dom_start_, 0);
                debug_assert_eq!(self.dom_end_, 1);
            }
        }

        // cpp: layoutng_inline/offset_mapping.cc:71-76
        pub fn AssociatedNode(&self) -> *mut Node {
            let object = self.layout_object_.Get();
            let fragment = DynamicTo::<LayoutTextFragment>(object);
            if !fragment.is_null() {
                return unsafe { &*fragment }.AssociatedTextNode().cast::<Node>();
            }
            unsafe { &*object }.GetNode()
        }

        // cpp: layoutng_inline/offset_mapping.cc:78-82
        pub fn GetOwner(&self) -> &Node {
            let node = self.AssociatedNode();
            assert!(!node.is_null());
            unsafe { &*node }
        }

        /// Source: core/layout/inline/offset_mapping.cc:24-43. Keep neighbor
        /// anchors for atomic inline objects; text offsets are DOM UTF-16.
        pub fn PositionForDOMOffset(&self, offset: u32) -> Position {
            let Some(node) = (unsafe { self.AssociatedNode().as_ref() }) else {
                return Position::default();
            };
            if node.IsTextNode() {
                let text = DynamicTo::<LayoutText>(node.GetLayoutObject());
                let Some(text) = (unsafe { text.as_ref() }) else {
                    return Position::default();
                };
                return Position::new(
                    node.InputId(),
                    offset.min(text.OriginalTextLength()) as usize,
                );
            }
            debug_assert!(offset <= 1);
            Position::WithAnchor(
                node.InputId(),
                0,
                if offset == 0 {
                    PositionAnchorType::BeforeAnchor
                } else {
                    PositionAnchorType::AfterAnchor
                },
            )
        }

        // cpp: layoutng_inline/offset_mapping.h:59-78
        pub fn GetType(&self) -> OffsetMappingUnitType {
            self.type_
        }
        pub fn IsCollapsed(&self) -> bool {
            self.type_ == OffsetMappingUnitType::kCollapsed
        }
        pub fn GetLayoutObject(&self) -> &LayoutObject {
            unsafe { &*self.layout_object_.Get() }
        }
        pub fn DOMStart(&self) -> u32 {
            self.dom_start_
        }
        pub fn DOMEnd(&self) -> u32 {
            self.dom_end_
        }
        pub fn TextContentStart(&self) -> u32 {
            self.text_content_start_
        }
        pub fn TextContentEnd(&self) -> u32 {
            self.text_content_end_
        }

        // cpp: layoutng_inline/offset_mapping.cc:84-104
        pub fn Concatenate(&mut self, other: &Self) -> bool {
            if self.layout_object_ != other.layout_object_ {
                return false;
            }
            if self.type_ != other.type_ {
                return false;
            }
            if self.dom_end_ != other.dom_start_ {
                return false;
            }
            if self.text_content_end_ != other.text_content_start_ {
                return false;
            }
            let fragment = DynamicTo::<LayoutTextFragment>(self.layout_object_.Get());
            if !fragment.is_null() {
                let fragment = unsafe { &*fragment };
                if fragment.IsRemainingTextLayoutObject()
                    && other.dom_start_ == fragment.TextStartOffset()
                {
                    return false;
                }
            }
            self.dom_end_ = other.dom_end_;
            self.text_content_end_ = other.text_content_end_;
            true
        }

        // cpp: layoutng_inline/offset_mapping.cc:106-123
        pub fn ConvertDOMOffsetToTextContent(&self, offset: u32) -> u32 {
            debug_assert!(offset >= self.dom_start_ && offset <= self.dom_end_);
            if offset == self.dom_start_ {
                return self.text_content_start_;
            }
            if offset == self.dom_end_ {
                return self.text_content_end_;
            }
            if self.text_content_start_ == self.text_content_end_ {
                return self.text_content_start_;
            }
            let text_content_offset = offset - self.dom_start_ + self.text_content_start_;
            text_content_offset.min(self.text_content_end_)
        }

        // cpp: layoutng_inline/offset_mapping.cc:125-138
        pub fn ConvertTextContentToFirstDOMOffset(&self, offset: u32) -> u32 {
            debug_assert!(offset >= self.text_content_start_ && offset <= self.text_content_end_);
            if self.text_content_start_ == self.text_content_end_ {
                return self.dom_start_;
            }
            if self.type_ == OffsetMappingUnitType::kIdentity {
                return self.dom_start_ + offset - self.text_content_start_;
            }
            if offset < self.text_content_end_ {
                self.dom_start_
            } else {
                self.dom_end_
            }
        }

        // cpp: layoutng_inline/offset_mapping.cc:140-150
        pub fn ConvertTextContentToLastDOMOffset(&self, offset: u32) -> u32 {
            debug_assert!(offset >= self.text_content_start_ && offset <= self.text_content_end_);
            if self.text_content_start_ == self.text_content_end_ {
                return self.dom_end_;
            }
            self.ConvertTextContentToFirstDOMOffset(offset)
        }
    }

    impl Traceable for OffsetMappingUnit {
        // cpp: layoutng_inline/offset_mapping.cc:344-346
        fn Trace(&self, visitor: &mut Visitor<'_>) {
            visitor.Trace(&self.layout_object_);
        }
    }

    // cpp: layoutng_inline/offset_mapping.h:112-125,271-283
    pub type UnitVector = HeapVector<OffsetMappingUnit>;
    pub type RangeMap = HeapHashMap<Member<Node>, (u32, u32)>;
    pub struct OffsetMapping {
        pub(crate) units_: UnitVector,
        pub(crate) ranges_: RangeMap,
        pub(crate) text_: String,
    }

    // These declared position APIs have no C++ definitions in this checkout.
    // Their typed providers must be supplied by the editing/inline assembly.
    unsafe extern "Rust" {
        // cpp: layoutng_inline/offset_mapping.h:146-162,177-235,239-243,287-287
        fn OffsetMappingAcceptsPosition(position: &Position) -> bool;
        fn OffsetMappingGetForPosition(position: &Position) -> *const OffsetMapping;
        fn OffsetMappingForceGetFor(position: &Position) -> *const OffsetMapping;
        fn OffsetMappingGetInlineFormattingContextOfPosition(
            position: &Position,
        ) -> *mut LayoutBlockFlow;
        fn OffsetMappingGetMappingUnitForPosition(
            this: &OffsetMapping,
            position: &Position,
        ) -> *const OffsetMappingUnit;
        fn OffsetMappingGetMappingUnitsForDOMRange(
            this: &OffsetMapping,
            range: &EphemeralRange,
        ) -> UnitVector;
        fn OffsetMappingGetTextContentOffset(
            this: &OffsetMapping,
            position: &Position,
        ) -> Option<u32>;
        fn OffsetMappingStartOfNextNonCollapsedContent(
            this: &OffsetMapping,
            position: &Position,
        ) -> Position;
        fn OffsetMappingEndOfLastNonCollapsedContent(
            this: &OffsetMapping,
            position: &Position,
        ) -> Position;
        fn OffsetMappingIsBeforeNonCollapsedContent(
            this: &OffsetMapping,
            position: &Position,
        ) -> bool;
        fn OffsetMappingIsAfterNonCollapsedContent(
            this: &OffsetMapping,
            position: &Position,
        ) -> bool;
        fn OffsetMappingGetCharacterBefore(
            this: &OffsetMapping,
            position: &Position,
        ) -> Option<UChar>;
        fn NGInlineFormattingContextOf(position: &Position) -> *mut LayoutBlockFlow;
    }

    impl OffsetMapping {
        // cpp: layoutng_inline/offset_mapping.cc:182-201
        pub fn new(units: UnitVector, ranges: RangeMap, text: String) -> Self {
            for unit in units.iter() {
                debug_assert!(unit.TextContentStart() <= text.length());
                debug_assert!(unit.TextContentEnd() <= text.length());
                unit.AssertValid();
            }
            for (_, &(first, last)) in ranges.iter() {
                debug_assert!((first as usize) < units.len());
                debug_assert!((last as usize) <= units.len());
            }
            Self {
                units_: units,
                ranges_: ranges,
                text_: text,
            }
        }

        // cpp: layoutng_inline/offset_mapping.h:127-129
        pub fn GetUnits(&self) -> &UnitVector {
            &self.units_
        }
        pub fn GetRanges(&self) -> &RangeMap {
            &self.ranges_
        }
        pub fn GetText(&self) -> &String {
            &self.text_
        }

        // cpp: layoutng_inline/offset_mapping.h:146-162
        pub fn AcceptsPosition(position: &Position) -> bool {
            unsafe { OffsetMappingAcceptsPosition(position) }
        }
        pub fn GetForPosition(position: &Position) -> *const Self {
            unsafe { OffsetMappingGetForPosition(position) }
        }
        pub fn ForceGetFor(position: &Position) -> *const Self {
            unsafe { OffsetMappingForceGetFor(position) }
        }

        // cpp: layoutng_inline/offset_mapping.cc:160-168
        pub fn GetForLayoutObject(layout_object: *const LayoutObject) -> *const Self {
            if layout_object.is_null() {
                return std::ptr::null();
            }
            let context = unsafe { &*layout_object }.FragmentItemsContainer();
            if context.is_null() {
                return std::ptr::null();
            }
            InlineNode::GetOffsetMapping(context)
        }

        // cpp: layoutng_inline/offset_mapping.cc:170-180
        pub fn GetInlineFormattingContextOfObject(object: &LayoutObject) -> *mut LayoutBlockFlow {
            let mut runner = object.Parent();
            while !runner.is_null() {
                let block_flow = DynamicTo::<LayoutBlockFlow>(runner);
                if !block_flow.is_null() {
                    return block_flow;
                }
                runner = unsafe { &*runner }.Parent();
            }
            std::ptr::null_mut()
        }
        // cpp: layoutng_inline/offset_mapping.h:177-181
        pub fn GetInlineFormattingContextOfPosition(position: &Position) -> *mut LayoutBlockFlow {
            unsafe { OffsetMappingGetInlineFormattingContextOfPosition(position) }
        }

        // cpp: layoutng_inline/offset_mapping.h:185-199
        pub fn GetMappingUnitForPosition(&self, position: &Position) -> *const OffsetMappingUnit {
            unsafe { OffsetMappingGetMappingUnitForPosition(self, position) }
        }
        pub fn GetMappingUnitsForDOMRange(&self, range: &EphemeralRange) -> UnitVector {
            unsafe { OffsetMappingGetMappingUnitsForDOMRange(self, range) }
        }

        // cpp: layoutng_inline/offset_mapping.cc:207-215
        pub fn GetMappingUnitsForNode(&self, node: &Node) -> &[OffsetMappingUnit] {
            let key = Member::from_ptr(node as *const _ as *mut Node);
            let Some(&(first, last)) = self.ranges_.get(&key) else {
                return &[];
            };
            &self.units_[first as usize..last as usize]
        }

        // cpp: layoutng_inline/offset_mapping.cc:217-231
        pub fn GetMappingUnitsForLayoutObject(
            &self,
            object: &LayoutObject,
        ) -> &[OffsetMappingUnit] {
            let begin = self
                .units_
                .iter()
                .position(|unit| std::ptr::eq(unit.GetLayoutObject(), object))
                .expect("layout object must have associated mapping");
            let end = self.units_[begin + 1..]
                .iter()
                .position(|unit| !std::ptr::eq(unit.GetLayoutObject(), object))
                .map_or(self.units_.len(), |relative| begin + 1 + relative);
            debug_assert!(begin < end);
            &self.units_[begin..end]
        }

        // cpp: layoutng_inline/offset_mapping.h:211-258
        /// Source: core/layout/inline/offset_mapping.cc:47-61,278-299,405-413.
        /// The resident caller supplies the borrowed anchor owner, avoiding a
        /// second node index or a global lookup from neutral boundary values.
        fn DOMOffsetForNode(node: &Node, position: &Position) -> Option<u32> {
            if position.node_id != Some(node.InputId()) {
                return None;
            }
            match (node.IsTextNode(), position.anchor) {
                (_, PositionAnchorType::BeforeAnchor) => Some(0),
                (true, PositionAnchorType::OffsetInAnchor) => u32::try_from(position.offset).ok(),
                (true, PositionAnchorType::AfterAnchor) => {
                    let text = DynamicTo::<LayoutText>(node.GetLayoutObject());
                    Some(unsafe { text.as_ref() }?.OriginalTextLength())
                }
                (false, PositionAnchorType::AfterAnchor) => Some(1),
                _ => None,
            }
        }
        /// Source offset_mapping.cc:70-79,201-218. The resident caller lends
        /// the canonical node so no global DOM lookup crosses this boundary.
        pub fn GetInlineFormattingContextForNode(
            node: &Node,
            position: &Position,
        ) -> *mut LayoutBlockFlow {
            if position.node_id != Some(node.InputId()) {
                return std::ptr::null_mut();
            }
            if !node.IsTextNode() {
                if !matches!(
                    position.anchor,
                    PositionAnchorType::BeforeAnchor | PositionAnchorType::AfterAnchor
                ) {
                    return std::ptr::null_mut();
                }
                let Some(object) = (unsafe { node.GetLayoutObject().as_ref() }) else {
                    return std::ptr::null_mut();
                };
                if !object.IsText() && !object.IsAtomicInline() {
                    return std::ptr::null_mut();
                }
            }
            let Some(offset) = Self::DOMOffsetForNode(node, position)
                .and_then(|offset| i32::try_from(offset).ok())
            else {
                return std::ptr::null_mut();
            };
            let object = crate::internal::layout_object::AssociatedLayoutObjectOf(
                node,
                offset,
                crate::internal::layout_object::LayoutObjectSide::kRemainingTextIfOnBoundary,
            );
            let Some(object) = (unsafe { object.as_ref() }) else {
                return std::ptr::null_mut();
            };
            Self::GetInlineFormattingContextOfObject(object)
        }
        fn MappingIndex(units: &[OffsetMappingUnit], offset: u32) -> Option<usize> {
            let end = units.partition_point(|unit| unit.DOMStart() <= offset);
            let index = end.checked_sub(1)?;
            (offset <= units[index].DOMEnd()).then_some(index)
        }
        pub fn GetTextContentOffsetForNode(&self, node: &Node, position: &Position) -> Option<u32> {
            let offset = Self::DOMOffsetForNode(node, position)?;
            let units = self.GetMappingUnitsForNode(node);
            let index = Self::MappingIndex(units, offset)?;
            Some(units[index].ConvertDOMOffsetToTextContent(offset))
        }
        /// Source: core/layout/inline/offset_mapping.cc:415-438,440-461.
        pub fn NonCollapsedBoundaryForNode(
            &self,
            node: &Node,
            position: &Position,
            forward: bool,
        ) -> Position {
            let Some(offset) = Self::DOMOffsetForNode(node, position) else {
                return Position::default();
            };
            let units = self.GetMappingUnitsForNode(node);
            let Some(index) = Self::MappingIndex(units, offset) else {
                return Position::default();
            };
            if forward {
                for unit in &units[index..] {
                    if unit.DOMEnd() > offset && !unit.IsCollapsed() {
                        return unit.PositionForDOMOffset(offset.max(unit.DOMStart()));
                    }
                }
            } else {
                for unit in units[..=index].iter().rev() {
                    if unit.DOMStart() < offset && !unit.IsCollapsed() {
                        return unit.PositionForDOMOffset(offset.min(unit.DOMEnd()));
                    }
                }
            }
            Position::default()
        }
        /// Source: core/layout/inline/offset_mapping.cc:463-482.
        pub fn IsAdjacentToNonCollapsedContentForNode(
            &self,
            node: &Node,
            position: &Position,
            before: bool,
        ) -> bool {
            let Some(offset) = Self::DOMOffsetForNode(node, position) else {
                return false;
            };
            let search = if before {
                offset
            } else {
                let Some(previous) = offset.checked_sub(1) else {
                    return false;
                };
                previous
            };
            let units = self.GetMappingUnitsForNode(node);
            let Some(index) = Self::MappingIndex(units, search) else {
                return false;
            };
            let unit = &units[index];
            !unit.IsCollapsed()
                && if before {
                    offset < unit.DOMEnd()
                } else {
                    offset > unit.DOMStart()
                }
        }
        pub fn GetTextContentOffset(&self, position: &Position) -> Option<u32> {
            unsafe { OffsetMappingGetTextContentOffset(self, position) }
        }
        pub fn StartOfNextNonCollapsedContent(&self, position: &Position) -> Position {
            unsafe { OffsetMappingStartOfNextNonCollapsedContent(self, position) }
        }
        pub fn EndOfLastNonCollapsedContent(&self, position: &Position) -> Position {
            unsafe { OffsetMappingEndOfLastNonCollapsedContent(self, position) }
        }
        pub fn IsBeforeNonCollapsedContent(&self, position: &Position) -> bool {
            unsafe { OffsetMappingIsBeforeNonCollapsedContent(self, position) }
        }
        pub fn IsAfterNonCollapsedContent(&self, position: &Position) -> bool {
            unsafe { OffsetMappingIsAfterNonCollapsedContent(self, position) }
        }
        pub fn GetCharacterBefore(&self, position: &Position) -> Option<UChar> {
            unsafe { OffsetMappingGetCharacterBefore(self, position) }
        }
        pub fn GetFirstPosition(&self, offset: u32) -> Position {
            // Source: core/layout/inline/offset_mapping.cc:492-513. Generated
            // content has no source node and must not become a DOM anchor.
            let index = self
                .units_
                .partition_point(|unit| unit.TextContentEnd() < offset);
            for unit in &self.units_[index..] {
                if unit.TextContentStart() > offset {
                    break;
                }
                if !unit.AssociatedNode().is_null() {
                    return unit
                        .PositionForDOMOffset(unit.ConvertTextContentToFirstDOMOffset(offset));
                }
            }
            Position::default()
        }
        pub fn GetLastPosition(&self, offset: u32) -> Position {
            // Source: core/layout/inline/offset_mapping.cc:560-578.
            let end = self
                .units_
                .partition_point(|unit| unit.TextContentStart() <= offset);
            for unit in self.units_[..end].iter().rev() {
                if unit.TextContentEnd() < offset {
                    break;
                }
                if !unit.AssociatedNode().is_null() {
                    return unit
                        .PositionForDOMOffset(unit.ConvertTextContentToLastDOMOffset(offset));
                }
            }
            Position::default()
        }

        // cpp: layoutng_inline/offset_mapping.cc:233-254
        pub fn GetMappingUnitsForTextContentOffsetRange(
            &self,
            start: u32,
            end: u32,
        ) -> &[OffsetMappingUnit] {
            debug_assert!(start <= end);
            if self
                .units_
                .first()
                .expect("source requires nonempty units")
                .TextContentStart()
                >= end
                || self
                    .units_
                    .last()
                    .expect("source requires nonempty units")
                    .TextContentEnd()
                    <= start
            {
                return &[];
            }
            let result_begin = self
                .units_
                .partition_point(|unit| unit.TextContentEnd() <= start);
            if result_begin == self.units_.len()
                || self.units_[result_begin].TextContentStart() >= end
            {
                return &[];
            }
            let result_end = self
                .units_
                .partition_point(|unit| unit.TextContentStart() < end);
            &self.units_[result_begin..result_end]
        }

        // cpp: layoutng_inline/offset_mapping.cc:270-297
        pub fn GetFirstMappingUnit(&self, offset: u32) -> Option<&OffsetMappingUnit> {
            if self.units_.is_empty() || self.units_[0].TextContentStart() > offset {
                return None;
            }
            let result = self
                .units_
                .partition_point(|unit| unit.TextContentEnd() < offset);
            if result == self.units_.len() {
                return None;
            }
            let next = result + 1;
            if next < self.units_.len() && self.units_[next].TextContentStart() == offset {
                return Some(&self.units_[next]);
            }
            Some(&self.units_[result])
        }

        // cpp: layoutng_inline/offset_mapping.cc:299-314
        pub fn GetLastMappingUnit(&self, offset: u32) -> Option<&OffsetMappingUnit> {
            if self.units_.is_empty() || self.units_[0].TextContentStart() > offset {
                return None;
            }
            let result = self
                .units_
                .partition_point(|unit| unit.TextContentStart() <= offset);
            assert!(result > 0);
            let unit = &self.units_[result - 1];
            if unit.TextContentEnd() < offset {
                return None;
            }
            Some(unit)
        }

        // cpp: layoutng_inline/offset_mapping.cc:318-327
        pub fn HasBidiControlCharactersOnly(&self, start: u32, end: u32) -> bool {
            debug_assert!(start <= end && end <= self.text_.length());
            let units = self.text_.Span16().unwrap_or_default();
            for &character in &units[start as usize..end as usize] {
                if !Character::IsBidiControl(character as i32) {
                    return false;
                }
            }
            true
        }
    }

    impl Traceable for OffsetMapping {
        // cpp: layoutng_inline/offset_mapping.h:266-269
        fn Trace(&self, visitor: &mut Visitor<'_>) {
            visitor.Trace(&self.units_);
            // The value of each RangeMap entry is a pair of scalar indices;
            // visiting each key is equivalent to tracing the C++ map.
            for (node, _) in self.ranges_.iter() {
                visitor.Trace(node);
            }
        }
    }

    // cpp: layoutng_inline/offset_mapping.h:132-144
    pub struct LayoutObjectConverter<'a> {
        units_: &'a [OffsetMappingUnit],
        last_unit_: Cell<usize>,
        last_offset_: Cell<u32>,
    }

    impl<'a> LayoutObjectConverter<'a> {
        pub fn new(offset_mapping: &'a OffsetMapping, object: &LayoutObject) -> Self {
            Self {
                units_: offset_mapping.GetMappingUnitsForLayoutObject(object),
                last_unit_: Cell::new(0),
                last_offset_: Cell::new(0),
            }
        }
        // cpp: layoutng_inline/offset_mapping.cc:329-342
        pub fn TextContentOffset(&self, offset: u32) -> u32 {
            let mut index = if offset >= self.last_offset_.get() {
                self.last_unit_.get()
            } else {
                0
            };
            if offset >= self.units_[index].DOMEnd() {
                index = (index..self.units_.len())
                    .find(|&i| {
                        self.units_[i].DOMStart() <= offset && offset < self.units_[i].DOMEnd()
                    })
                    .expect("DOM offset must have mapping unit");
            }
            self.last_unit_.set(index);
            self.last_offset_.set(offset);
            self.units_[index].ConvertDOMOffsetToTextContent(offset)
        }
    }

    // cpp: layoutng_inline/offset_mapping.h:287-292
    pub fn NGInlineFormattingContextOfPosition(position: &Position) -> *mut LayoutBlockFlow {
        unsafe { NGInlineFormattingContextOf(position) }
    }
    pub const OFFSET_MAPPING_UNIT_CAN_CLEAR_UNUSED_SLOTS_WITH_MEMSET: bool = true;
    pub const OFFSET_MAPPING_UNIT_CAN_TRACE_CONCURRENTLY: bool = true;
}

#[cfg(feature = "translation_in_progress")]
pub use translated::*;
