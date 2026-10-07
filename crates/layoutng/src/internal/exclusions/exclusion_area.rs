use foundation::{EFloat, GCedHeapVector, LayoutUnit, MakeGarbageCollected, Member, Visitor};
use layoutng_geometry::geometry::bfc_offset::BfcDelta;
use layoutng_geometry::geometry::bfc_rect::BfcRect;
use layoutng_geometry::geometry::box_strut::BoxStrut;

use crate::internal::layout_box::LayoutBox;

// cpp: layoutng/internal/exclusions/exclusion_area.h:19-35
pub struct ExclusionShapeData {
    pub layout_box: Member<LayoutBox>,
    pub margins: BoxStrut,
    pub shape_insets: BoxStrut,
}

#[allow(non_snake_case)]
impl ExclusionShapeData {
    // cpp: layoutng/internal/exclusions/exclusion_area.h:21-24
    pub fn new(layout_box: *const LayoutBox, margins: &BoxStrut, shape_insets: &BoxStrut) -> Self {
        Self {
            layout_box: Member::from_ptr(layout_box as *mut LayoutBox),
            margins: *margins,
            shape_insets: *shape_insets,
        }
    }

    // cpp: layoutng/internal/exclusions/exclusion_area.cc:12-14
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.layout_box);
    }
}

impl Clone for ExclusionShapeData {
    // cpp: layoutng/internal/exclusions/exclusion_area.h:25-28
    fn clone(&self) -> Self {
        Self {
            layout_box: Member::from_ptr(self.layout_box.Get()),
            margins: self.margins,
            shape_insets: self.shape_insets,
        }
    }
}

// cpp: layoutng/internal/exclusions/exclusion_area.h:40-43
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ExclusionAreaKind {
    kFloat,
    kInitialLetterBox,
}

// cpp: layoutng/internal/exclusions/exclusion_area.h:37-89
pub struct ExclusionArea {
    pub rect: BfcRect,
    pub r#type: EFloat,
    pub kind: ExclusionAreaKind,
    pub is_past_other_exclusions: bool,
    pub shape_data: Member<ExclusionShapeData>,
}

#[allow(non_snake_case)]
impl ExclusionArea {
    // cpp: layoutng/internal/exclusions/exclusion_area.h:45-49
    pub fn new(
        rect: &BfcRect,
        r#type: EFloat,
        kind: ExclusionAreaKind,
        shape_data: *const ExclusionShapeData,
    ) -> Self {
        Self {
            rect: *rect,
            r#type,
            kind,
            is_past_other_exclusions: false,
            shape_data: Member::from_ptr(shape_data as *mut ExclusionShapeData),
        }
    }

    // cpp: layoutng/internal/exclusions/exclusion_area.h:51-56
    pub fn Create(
        rect: &BfcRect,
        r#type: EFloat,
        shape_data: *const ExclusionShapeData,
    ) -> *const Self {
        MakeGarbageCollected(Self::new(
            rect,
            r#type,
            ExclusionAreaKind::kFloat,
            shape_data,
        ))
    }

    pub fn CreateWithoutShape(rect: &BfcRect, r#type: EFloat) -> *const Self {
        Self::Create(rect, r#type, std::ptr::null())
    }

    // cpp: layoutng/internal/exclusions/exclusion_area.h:58-62
    pub fn CreateForInitialLetterBox(rect: &BfcRect, r#type: EFloat) -> *const Self {
        MakeGarbageCollected(Self::new(
            rect,
            r#type,
            ExclusionAreaKind::kInitialLetterBox,
            std::ptr::null(),
        ))
    }

    // cpp: layoutng/internal/exclusions/exclusion_area.h:64-76
    pub fn CopyWithOffset(&self, offset_delta: &BfcDelta) -> *const Self {
        if offset_delta.line_offset_delta == LayoutUnit::default()
            && offset_delta.block_offset_delta == LayoutUnit::default()
        {
            return self;
        }

        let mut new_rect = self.rect;
        new_rect.start_offset += *offset_delta;
        new_rect.end_offset += *offset_delta;

        let shape_data = self.shape_data.Get();
        let copied_shape_data = if shape_data.is_null() {
            std::ptr::null_mut()
        } else {
            MakeGarbageCollected(unsafe { &*shape_data }.clone())
        };
        MakeGarbageCollected(Self::new(
            &new_rect,
            self.r#type,
            self.kind,
            copied_shape_data,
        ))
    }

    // cpp: layoutng/internal/exclusions/exclusion_area.h:78-80
    pub fn IsForInitialLetterBox(&self) -> bool {
        self.kind == ExclusionAreaKind::kInitialLetterBox
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.shape_data);
    }
}

// cpp: layoutng/internal/exclusions/exclusion_area.h:88-88
// cpp: layoutng/internal/exclusions/exclusion_area.cc:16-19
impl PartialEq for ExclusionArea {
    fn eq(&self, other: &Self) -> bool {
        self.r#type == other.r#type
            && self.kind == other.kind
            && self.rect == other.rect
            && self.shape_data.Get() == other.shape_data.Get()
    }
}

// cpp: layoutng/internal/exclusions/exclusion_area.h:91-92
pub type ExclusionAreaPtrArray = Vec<Member<ExclusionArea>>;
pub type GCedExclusionAreaPtrArray = GCedHeapVector<Member<ExclusionArea>>;

// cpp: layoutng/internal/exclusions/exclusion_area.h:94-95
// Both ostream insertions are declared but have no definitions in the source tree.
