// C++: layoutng_inline/offset_mapping_builder.h/.cc.
// The scope stores raw builder identity so callers may append while the
// C++-style RAII guard is alive; Drop restores the saved annotation and offset.
#![allow(non_snake_case)]

use foundation::{DynamicTo, HeapVector, MakeGarbageCollected, Member, String};
use layoutng::internal::layout_node_metadata::Node;
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_text::LayoutText;

use crate::layout_text_fragment::LayoutTextFragment;
use crate::offset_mapping::{
    OffsetMapping, OffsetMappingUnit, OffsetMappingUnitType, RangeMap, UnitVector,
};

// cpp: layoutng_inline/offset_mapping_builder.cc:17-26
fn GetAssociatedStartOffset(layout_object: *const LayoutObject) -> u32 {
    let fragment = DynamicTo::<LayoutTextFragment>(layout_object);
    if fragment.is_null() || unsafe { &*fragment }.AssociatedTextNode().is_null() {
        return 0;
    }
    unsafe { &*fragment }.Start()
}

// cpp: layoutng_inline/offset_mapping_builder.h:60-74
pub struct SourceNodeScope {
    builder_: *mut OffsetMappingBuilder,
    old_layout_object_: *const LayoutObject,
    old_offset_: u32,
}

impl SourceNodeScope {
    // cpp: layoutng_inline/offset_mapping_builder.cc:32-47
    pub fn new(builder: &mut OffsetMappingBuilder, node: *const LayoutObject) -> Self {
        let old_layout_object = builder.current_layout_object_;
        let old_offset = builder.current_offset_;
        builder.current_layout_object_ = node;
        builder.current_offset_ = GetAssociatedStartOffset(node);
        builder.has_open_unit_ = false;
        if !node.is_null() {
            debug_assert!(!builder.has_nonnull_node_scope_);
            builder.has_nonnull_node_scope_ = true;
        }
        Self {
            builder_: builder,
            old_layout_object_: old_layout_object,
            old_offset_: old_offset,
        }
    }
}

impl Drop for SourceNodeScope {
    // cpp: layoutng_inline/offset_mapping_builder.cc:49-55
    fn drop(&mut self) {
        let builder = unsafe { &mut *self.builder_ };
        builder.has_open_unit_ = false;
        if !builder.current_layout_object_.is_null() {
            builder.has_nonnull_node_scope_ = false;
        }
        // C++ AutoReset members run in reverse declaration order.
        builder.current_offset_ = self.old_offset_;
        builder.current_layout_object_ = self.old_layout_object_;
    }
}

// cpp: layoutng_inline/offset_mapping_builder.h:76-159
pub struct OffsetMappingBuilder {
    current_layout_object_: *const LayoutObject,
    current_offset_: u32,
    has_open_unit_: bool,
    has_nonnull_node_scope_: bool,
    destination_length_: u32,
    mapping_units_: UnitVector,
    unit_ranges_: RangeMap,
    destination_string_: String,
}

impl Default for OffsetMappingBuilder {
    // cpp: layoutng_inline/offset_mapping_builder.cc:30-30
    fn default() -> Self {
        Self {
            current_layout_object_: std::ptr::null(),
            current_offset_: 0,
            has_open_unit_: false,
            has_nonnull_node_scope_: false,
            destination_length_: 0,
            mapping_units_: UnitVector::default(),
            unit_ranges_: RangeMap::default(),
            destination_string_: String::new(),
        }
    }
}

impl OffsetMappingBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    // cpp: layoutng_inline/offset_mapping_builder.cc:57-60
    pub fn ReserveCapacity(&mut self, capacity: u32) {
        self.unit_ranges_.ReserveCapacityForSize(capacity);
        self.mapping_units_
            .reserve((f64::from(capacity) * 1.5) as usize);
    }

    // cpp: layoutng_inline/offset_mapping_builder.cc:62-87
    pub fn AppendIdentityMapping(&mut self, length: u32) {
        debug_assert!(length > 0);
        let dom_start = self.current_offset_;
        let dom_end = dom_start + length;
        let text_content_start = self.destination_length_;
        let text_content_end = text_content_start + length;
        self.current_offset_ += length;
        self.destination_length_ += length;
        if self.current_layout_object_.is_null() {
            return;
        }
        if self.has_open_unit_
            && self.mapping_units_.last().unwrap().GetType() == OffsetMappingUnitType::kIdentity
        {
            let last = self.mapping_units_.last_mut().unwrap();
            debug_assert!(std::ptr::eq(
                last.GetLayoutObject(),
                self.current_layout_object_
            ));
            debug_assert_eq!(last.DOMEnd(), dom_start);
            last.dom_end_ += length;
            last.text_content_end_ += length;
            return;
        }
        self.mapping_units_.push_back(OffsetMappingUnit::new(
            OffsetMappingUnitType::kIdentity,
            unsafe { &*self.current_layout_object_ },
            dom_start,
            dom_end,
            text_content_start,
            text_content_end,
        ));
        self.has_open_unit_ = true;
    }

    // cpp: layoutng_inline/offset_mapping_builder.cc:89-93
    pub fn RevertIdentityMapping1(&mut self) {
        assert!(self.current_layout_object_.is_null());
        self.current_offset_ -= 1;
        self.destination_length_ -= 1;
    }

    // cpp: layoutng_inline/offset_mapping_builder.cc:95-117
    pub fn AppendCollapsedMapping(&mut self, length: u32) {
        debug_assert!(length > 0);
        let dom_start = self.current_offset_;
        let dom_end = dom_start + length;
        let text_content_start = self.destination_length_;
        let text_content_end = text_content_start;
        self.current_offset_ += length;
        if self.current_layout_object_.is_null() {
            return;
        }
        if self.has_open_unit_ && self.mapping_units_.last().unwrap().IsCollapsed() {
            let last = self.mapping_units_.last_mut().unwrap();
            debug_assert!(std::ptr::eq(
                last.GetLayoutObject(),
                self.current_layout_object_
            ));
            debug_assert_eq!(last.DOMEnd(), dom_start);
            last.dom_end_ += length;
            return;
        }
        self.mapping_units_.push_back(OffsetMappingUnit::new(
            OffsetMappingUnitType::kCollapsed,
            unsafe { &*self.current_layout_object_ },
            dom_start,
            dom_end,
            text_content_start,
            text_content_end,
        ));
        self.has_open_unit_ = true;
    }

    // cpp: layoutng_inline/offset_mapping_builder.cc:119-140
    pub fn AppendVariableMapping(&mut self, dom_length: u32, text_content_length: u32) {
        debug_assert!(dom_length > 0);
        debug_assert!(text_content_length > 0);
        let dom_start = self.current_offset_;
        let dom_end = dom_start + dom_length;
        let text_content_start = self.destination_length_;
        let text_content_end = text_content_start + text_content_length;
        self.current_offset_ += dom_length;
        self.destination_length_ += text_content_length;
        if self.current_layout_object_.is_null() {
            return;
        }
        // Variable units cannot be merged.
        self.mapping_units_.push_back(OffsetMappingUnit::new(
            OffsetMappingUnitType::kVariable,
            unsafe { &*self.current_layout_object_ },
            dom_start,
            dom_end,
            text_content_start,
            text_content_end,
        ));
        self.has_open_unit_ = false;
    }

    // cpp: layoutng_inline/offset_mapping_builder.cc:142-205
    pub fn CollapseTrailingSpace(&mut self, space_offset: u32) {
        debug_assert!(space_offset < self.destination_length_);
        self.destination_length_ -= 1;

        let mut container_index = None;
        for index in (0..self.mapping_units_.len()).rev() {
            let unit = &mut self.mapping_units_[index];
            if unit.TextContentStart() > space_offset {
                unit.text_content_start_ -= 1;
                unit.text_content_end_ -= 1;
                continue;
            }
            container_index = Some(index);
            break;
        }

        let Some(position) = container_index else {
            return;
        };
        let container_unit = &self.mapping_units_[position];
        if container_unit.TextContentEnd() <= space_offset {
            return;
        }

        debug_assert_eq!(OffsetMappingUnitType::kIdentity, container_unit.GetType());
        let layout_object = container_unit.GetLayoutObject();
        let mut dom_offset = container_unit.DOMStart();
        let mut text_content_offset = container_unit.TextContentStart();
        let offset_to_collapse = space_offset - text_content_offset;
        let mut new_units: HeapVector<OffsetMappingUnit, 3> = HeapVector::new();
        if offset_to_collapse != 0 {
            new_units.push_back(OffsetMappingUnit::new(
                OffsetMappingUnitType::kIdentity,
                layout_object,
                dom_offset,
                dom_offset + offset_to_collapse,
                text_content_offset,
                text_content_offset + offset_to_collapse,
            ));
            dom_offset += offset_to_collapse;
            text_content_offset += offset_to_collapse;
        }
        new_units.push_back(OffsetMappingUnit::new(
            OffsetMappingUnitType::kCollapsed,
            layout_object,
            dom_offset,
            dom_offset + 1,
            text_content_offset,
            text_content_offset,
        ));
        dom_offset += 1;
        if dom_offset < container_unit.DOMEnd() {
            new_units.push_back(OffsetMappingUnit::new(
                OffsetMappingUnitType::kIdentity,
                layout_object,
                dom_offset,
                container_unit.DOMEnd(),
                text_content_offset,
                container_unit.TextContentEnd() - 1,
            ));
        }

        // The C++ erase/insert sequence first replaces the containing unit,
        // then merges its right and left boundaries independently.
        self.mapping_units_.remove(position);
        let new_unit_end = position + new_units.len();
        self.mapping_units_
            .splice(position..position, new_units.into_iter());
        while new_unit_end != 0 && new_unit_end < self.mapping_units_.len() {
            let next = self.mapping_units_[new_unit_end].clone();
            if !self.mapping_units_[new_unit_end - 1].Concatenate(&next) {
                break;
            }
            self.mapping_units_.remove(new_unit_end);
        }
        while position != 0 && position < self.mapping_units_.len() {
            let next = self.mapping_units_[position].clone();
            if !self.mapping_units_[position - 1].Concatenate(&next) {
                break;
            }
            self.mapping_units_.remove(position);
        }
    }

    // cpp: layoutng_inline/offset_mapping_builder.cc:207-240
    pub fn RestoreTrailingCollapsibleSpace(&mut self, layout_text: &LayoutText, offset: u32) {
        self.destination_length_ += 1;
        let layout_object = layout_text as *const LayoutText as *const LayoutObject;
        for index in (0..self.mapping_units_.len()).rev() {
            let unit = &mut self.mapping_units_[index];
            if unit.text_content_end_ < offset {
                unreachable!("collapsed mapping at offset must exist");
            }
            if unit.text_content_start_ != offset
                || unit.text_content_end_ != offset
                || !std::ptr::eq(unit.GetLayoutObject(), layout_object)
            {
                unit.text_content_start_ += 1;
                unit.text_content_end_ += 1;
                continue;
            }
            debug_assert!(unit.IsCollapsed());
            let original_dom_end = unit.dom_end_;
            unit.type_ = OffsetMappingUnitType::kIdentity;
            unit.dom_end_ = unit.dom_start_ + 1;
            unit.text_content_end_ = unit.text_content_start_ + 1;
            if original_dom_end - unit.dom_start_ == 1 {
                return;
            }
            let remaining = OffsetMappingUnit::new(
                OffsetMappingUnitType::kCollapsed,
                unsafe { &*layout_object },
                unit.dom_end_,
                original_dom_end,
                unit.text_content_end_,
                unit.text_content_end_,
            );
            self.mapping_units_.insert(index + 1, remaining);
            return;
        }
        unreachable!("collapsed mapping at offset must exist");
    }

    // cpp: layoutng_inline/offset_mapping_builder.cc:242-252
    pub fn SetDestinationString(&mut self, string: &String) -> bool {
        debug_assert_eq!(self.destination_length_, string.length());
        if self.destination_length_ != string.length() {
            return false;
        }
        self.destination_string_ = string.clone();
        true
    }

    // cpp: layoutng_inline/offset_mapping_builder.cc:254-279
    pub fn Build(&mut self) -> *mut OffsetMapping {
        let mut range_start = 0;
        while range_start < self.mapping_units_.len() {
            let layout_object = self.mapping_units_[range_start].GetLayoutObject();
            let mut range_end = range_start + 1;
            let node = self.mapping_units_[range_start].AssociatedNode();
            if !node.is_null() {
                while range_end < self.mapping_units_.len()
                    && self.mapping_units_[range_end].AssociatedNode() == node
                {
                    range_end += 1;
                }
                let key = Member::from_ptr(node as *mut Node);
                debug_assert!(!self.unit_ranges_.Contains(&key));
                self.unit_ranges_
                    .insert(key, (range_start as u32, range_end as u32));
            } else {
                while range_end < self.mapping_units_.len()
                    && std::ptr::eq(
                        self.mapping_units_[range_end].GetLayoutObject(),
                        layout_object,
                    )
                {
                    range_end += 1;
                }
            }
            range_start = range_end;
        }
        MakeGarbageCollected(OffsetMapping::new(
            std::mem::take(&mut self.mapping_units_),
            std::mem::take(&mut self.unit_ranges_),
            self.destination_string_.clone(),
        ))
    }
}

impl Drop for OffsetMappingBuilder {
    // cpp: layoutng_inline/offset_mapping_builder.h:79-82
    fn drop(&mut self) {
        self.mapping_units_.clear();
        self.unit_ranges_.clear();
    }
}
