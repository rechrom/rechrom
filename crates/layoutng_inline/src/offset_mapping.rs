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

    use font_engine::text::native::character::Character;
    use foundation::{
        DynamicTo, HeapHashMap, HeapVector, Member, String, Traceable, UChar, Visitor,
    };
    use layoutng::internal::editing::forward::{EphemeralRange, Position};
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
        fn OffsetMappingGetFirstPosition(this: &OffsetMapping, offset: u32) -> Position;
        fn OffsetMappingGetLastPosition(this: &OffsetMapping, offset: u32) -> Position;
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
            unsafe { OffsetMappingGetFirstPosition(self, offset) }
        }
        pub fn GetLastPosition(&self, offset: u32) -> Position {
            unsafe { OffsetMappingGetLastPosition(self, offset) }
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
