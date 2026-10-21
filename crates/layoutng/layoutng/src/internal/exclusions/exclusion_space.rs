#![allow(non_snake_case)]

use std::cell::RefCell;

use foundation::{
    EClear, EFloat, HeapVector, LayoutUnit, MakeGarbageCollected, Member, TextDirection, Vector,
    Visitor,
};
use layoutng_geometry::geometry::bfc_offset::{BfcDelta, BfcOffset};
use layoutng_geometry::geometry::bfc_rect::BfcRect;

use super::exclusion_area::{ExclusionArea, GCedExclusionAreaPtrArray};
use super::layout_opportunity::LayoutOpportunity;
use super::shape_exclusions::ShapeExclusions;

// The source's inline capacity of one is an allocation hint.
// cpp: layoutng/internal/exclusions/exclusion_space.h:24-24
pub type LayoutOpportunityVector = HeapVector<LayoutOpportunity>;

// cpp: layoutng/internal/exclusions/exclusion_space.cc:17-63
fn InsertClosedArea(area: ClosedArea, areas: &mut HeapVector<ClosedArea>) {
    if areas.is_empty() {
        areas.push(area);
        return;
    }
    for index in (0..areas.len()).rev() {
        let other = &areas[index];
        if other.opportunity.rect.BlockStartOffset() <= area.opportunity.rect.BlockStartOffset() {
            if other.opportunity.rect.BlockStartOffset() == area.opportunity.rect.BlockStartOffset()
            {
                debug_assert!(
                    other.opportunity.rect.BlockSize() <= area.opportunity.rect.BlockSize()
                );
                debug_assert!(
                    other.opportunity.rect.InlineSize() >= area.opportunity.rect.InlineSize()
                );
            }
            areas.insert(index + 1, area);
            return;
        }
    }
    debug_assert_eq!(area.opportunity.rect.BlockStartOffset(), LayoutUnit::Min());
    areas.insert(0, area);
}

// cpp: layoutng/internal/exclusions/exclusion_space.cc:65-100
fn HasSolidEdges(
    edges: &Vector<ShelfEdge>,
    block_start: LayoutUnit,
    block_end: LayoutUnit,
) -> bool {
    edges.is_empty()
        || edges
            .iter()
            .any(|edge| edge.block_end > block_start && edge.block_start < block_end)
}

fn CollectSolidEdges(edges: &mut Vector<ShelfEdge>, block_offset: LayoutUnit) -> Vector<ShelfEdge> {
    let mut out_edges = std::mem::take(edges);
    out_edges.retain(|edge| edge.block_end > block_offset);
    out_edges
}

// cpp: layoutng/internal/exclusions/exclusion_space.cc:102-160
fn Intersects(
    opportunity: &LayoutOpportunity,
    offset: &BfcOffset,
    inline_size: LayoutUnit,
) -> bool {
    opportunity.rect.LineEndOffset() >= offset.line_offset
        && opportunity.rect.LineStartOffset() <= offset.line_offset + inline_size
        && opportunity.rect.BlockEndOffset() > offset.block_offset
}

fn CreateLayoutOpportunityFromArea(
    other: &LayoutOpportunity,
    offset: &BfcOffset,
    inline_size: LayoutUnit,
) -> LayoutOpportunity {
    debug_assert!(Intersects(other, offset, inline_size));
    let start = BfcOffset::new(
        other.rect.LineStartOffset().max(offset.line_offset),
        other.rect.BlockStartOffset().max(offset.block_offset),
    );
    let end = BfcOffset::new(
        other
            .rect
            .LineEndOffset()
            .min(offset.line_offset + inline_size),
        other.rect.BlockEndOffset(),
    );
    let shapes = other.shape_exclusions.Get();
    let copied_shapes = if shapes.is_null() {
        std::ptr::null()
    } else {
        MakeGarbageCollected(unsafe { &*shapes }.clone()) as *const ShapeExclusions
    };
    LayoutOpportunity::new(&BfcRect::new(start, end), copied_shapes)
}

fn CreateLayoutOpportunityFromShelf(
    shelf: &Shelf,
    offset: &BfcOffset,
    inline_size: LayoutUnit,
    direction: TextDirection,
) -> LayoutOpportunity {
    let mut line_left = shelf.line_left.max(offset.line_offset);
    let mut line_right = shelf.line_right.min(offset.line_offset + inline_size);
    if direction == TextDirection::kLtr {
        line_right = line_right.max(line_left);
    } else {
        line_left = line_left.min(line_right);
    }
    let start = BfcOffset::new(line_left, shelf.block_offset.max(offset.block_offset));
    let end = BfcOffset::new(line_right, LayoutUnit::Max());
    let shapes = shelf.shape_exclusions.Get();
    let copied_shapes = if shelf.has_shape_exclusions && !shapes.is_null() {
        MakeGarbageCollected(unsafe { &*shapes }.clone()) as *const ShapeExclusions
    } else {
        std::ptr::null()
    };
    LayoutOpportunity::new(&BfcRect::new(start, end), copied_shapes)
}

// cpp: layoutng/internal/exclusions/exclusion_space.h:271-278
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShelfEdge {
    pub block_start: LayoutUnit,
    pub block_end: LayoutUnit,
}

impl ShelfEdge {
    pub fn new(block_start: LayoutUnit, block_end: LayoutUnit) -> Self {
        Self {
            block_start,
            block_end,
        }
    }
}

// cpp: layoutng/internal/exclusions/exclusion_space.h:310-353
pub struct Shelf {
    pub block_offset: LayoutUnit,
    pub line_left: LayoutUnit,
    pub line_right: LayoutUnit,
    pub line_left_edges: Vector<ShelfEdge>,
    pub line_right_edges: Vector<ShelfEdge>,
    pub shape_exclusions: Member<ShapeExclusions>,
    pub has_shape_exclusions: bool,
}

impl Shelf {
    pub fn new(block_offset: LayoutUnit, track_shape_exclusions: bool) -> Self {
        Self {
            block_offset,
            line_left: LayoutUnit::Min(),
            line_right: LayoutUnit::Max(),
            line_left_edges: Vector::default(),
            line_right_edges: Vector::default(),
            shape_exclusions: if track_shape_exclusions {
                Member::from_ptr(MakeGarbageCollected(ShapeExclusions::default()))
            } else {
                Member::default()
            },
            has_shape_exclusions: false,
        }
    }

    // The copy constructor duplicates shape bookkeeping, rather than sharing it.
    pub fn copy_from(other: &Self) -> Self {
        let shapes = other.shape_exclusions.Get();
        Self {
            block_offset: other.block_offset,
            line_left: other.line_left,
            line_right: other.line_right,
            line_left_edges: other.line_left_edges.clone(),
            line_right_edges: other.line_right_edges.clone(),
            shape_exclusions: if shapes.is_null() {
                Member::default()
            } else {
                Member::from_ptr(MakeGarbageCollected(unsafe { &*shapes }.clone()))
            },
            has_shape_exclusions: other.has_shape_exclusions,
        }
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.shape_exclusions);
    }
}

// cpp: layoutng/internal/exclusions/exclusion_space.h:385-403
pub struct ClosedArea {
    pub opportunity: LayoutOpportunity,
    pub line_left_edges: Vector<ShelfEdge>,
    pub line_right_edges: Vector<ShelfEdge>,
}

impl ClosedArea {
    pub fn new(
        opportunity: LayoutOpportunity,
        line_left_edges: &Vector<ShelfEdge>,
        line_right_edges: &Vector<ShelfEdge>,
    ) -> Self {
        Self {
            opportunity,
            line_left_edges: line_left_edges.clone(),
            line_right_edges: line_right_edges.clone(),
        }
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.opportunity);
    }
}

// cpp: layoutng/internal/exclusions/exclusion_space.h:471-533
pub struct DerivedGeometry {
    pub shelves_: HeapVector<Shelf>,
    pub areas_: HeapVector<ClosedArea>,
    pub block_offset_limit_: LayoutUnit,
    pub track_shape_exclusions_: bool,
}

impl DerivedGeometry {
    // cpp: layoutng/internal/exclusions/exclusion_space.cc:227-235
    pub fn new(block_offset_limit: LayoutUnit, track_shape_exclusions: bool) -> Self {
        let mut shelves = HeapVector::default();
        shelves.push(Shelf::new(LayoutUnit::Min(), track_shape_exclusions));
        Self {
            shelves_: shelves,
            areas_: HeapVector::default(),
            block_offset_limit_: block_offset_limit,
            track_shape_exclusions_: track_shape_exclusions,
        }
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.cc:336-631
    pub fn Add(&mut self, exclusion: &ExclusionArea) {
        debug_assert!(exclusion.rect.BlockStartOffset() >= self.block_offset_limit_);
        if exclusion.rect.LineEndOffset() <= exclusion.rect.LineStartOffset() {
            return;
        }

        let exclusion_end = exclusion.rect.BlockEndOffset();
        let mut index = 0usize;
        let mut inserted = false;
        while index < self.shelves_.len() {
            let is_between_shelves = exclusion_end >= self.shelves_[index].block_offset
                && (index + 1 >= self.shelves_.len()
                    || exclusion_end < self.shelves_[index + 1].block_offset);
            let mut shelf_copy =
                is_between_shelves.then(|| Shelf::copy_from(&self.shelves_[index]));
            let is_below = exclusion.rect.BlockStartOffset() > self.shelves_[index].block_offset;

            if is_below {
                let shelf = &self.shelves_[index];
                let block_start = exclusion.rect.BlockStartOffset();
                let has_solid_edges =
                    HasSolidEdges(&shelf.line_left_edges, shelf.block_offset, block_start)
                        && HasSolidEdges(&shelf.line_right_edges, shelf.block_offset, block_start);
                let is_overlapping = if exclusion.r#type == EFloat::kLeft {
                    exclusion.rect.LineStartOffset() <= shelf.line_right
                        && exclusion.rect.LineEndOffset() > shelf.line_left
                } else {
                    debug_assert_eq!(exclusion.r#type, EFloat::kRight);
                    exclusion.rect.LineStartOffset() < shelf.line_right
                        && exclusion.rect.LineEndOffset() >= shelf.line_left
                };
                if has_solid_edges && is_overlapping {
                    let rect = BfcRect::new(
                        BfcOffset::new(shelf.line_left, shelf.block_offset),
                        BfcOffset::new(shelf.line_right, block_start),
                    );
                    let shapes = shelf.shape_exclusions.Get();
                    let copied_shapes = if shelf.has_shape_exclusions && !shapes.is_null() {
                        MakeGarbageCollected(unsafe { &*shapes }.clone()) as *const ShapeExclusions
                    } else {
                        std::ptr::null()
                    };
                    InsertClosedArea(
                        ClosedArea::new(
                            LayoutOpportunity::new(&rect, copied_shapes),
                            &shelf.line_left_edges,
                            &shelf.line_right_edges,
                        ),
                        &mut self.areas_,
                    );
                }
            }

            let is_intersecting = !is_below && exclusion_end > self.shelves_[index].block_offset;
            let mut removed = false;
            if is_below || is_intersecting {
                let previous_bounds = if index > 0 {
                    Some((
                        self.shelves_[index - 1].line_left,
                        self.shelves_[index - 1].line_right,
                    ))
                } else {
                    None
                };
                let shelf = &mut self.shelves_[index];
                if exclusion.r#type == EFloat::kLeft {
                    if exclusion.rect.LineEndOffset() >= shelf.line_left {
                        if exclusion.rect.LineEndOffset() > shelf.line_left {
                            shelf.line_left_edges.clear();
                        }
                        shelf.line_left = exclusion.rect.LineEndOffset();
                        shelf.line_left_edges.push(ShelfEdge::new(
                            exclusion.rect.BlockStartOffset(),
                            exclusion.rect.BlockEndOffset(),
                        ));
                    }
                    let shapes = shelf.shape_exclusions.Get();
                    if !shapes.is_null() {
                        unsafe { &mut *shapes }
                            .line_left_shapes
                            .push(Member::from_ptr(
                                exclusion as *const ExclusionArea as *mut _,
                            ));
                    }
                } else {
                    debug_assert_eq!(exclusion.r#type, EFloat::kRight);
                    if exclusion.rect.LineStartOffset() <= shelf.line_right {
                        if exclusion.rect.LineStartOffset() < shelf.line_right {
                            shelf.line_right_edges.clear();
                        }
                        shelf.line_right = exclusion.rect.LineStartOffset();
                        shelf.line_right_edges.push(ShelfEdge::new(
                            exclusion.rect.BlockStartOffset(),
                            exclusion.rect.BlockEndOffset(),
                        ));
                    }
                    let shapes = shelf.shape_exclusions.Get();
                    if !shapes.is_null() {
                        unsafe { &mut *shapes }
                            .line_right_shapes
                            .push(Member::from_ptr(
                                exclusion as *const ExclusionArea as *mut _,
                            ));
                    }
                }
                if !exclusion.shape_data.Get().is_null() {
                    shelf.has_shape_exclusions = true;
                }
                let is_closed_off = shelf.line_left > shelf.line_right;
                let is_same_as_previous = previous_bounds.is_some_and(|(left, right)| {
                    shelf.line_left == left && shelf.line_right == right
                });
                if is_closed_off || is_same_as_previous {
                    self.shelves_.remove(index);
                    removed = true;
                }
            }

            if is_between_shelves {
                debug_assert!(!inserted);
                inserted = true;
                let mut old_shelf = shelf_copy.take().expect("between shelves requires copy");
                if exclusion_end != old_shelf.block_offset {
                    let mut new_shelf = Shelf::new(exclusion_end, self.track_shape_exclusions_);
                    new_shelf.line_left_edges =
                        CollectSolidEdges(&mut old_shelf.line_left_edges, exclusion_end);
                    new_shelf.line_right_edges =
                        CollectSolidEdges(&mut old_shelf.line_right_edges, exclusion_end);
                    new_shelf.shape_exclusions = Member::from_ptr(old_shelf.shape_exclusions.Get());
                    new_shelf.has_shape_exclusions = old_shelf.has_shape_exclusions;
                    new_shelf.line_left = if new_shelf.line_left_edges.is_empty() {
                        LayoutUnit::Min()
                    } else {
                        old_shelf.line_left
                    };
                    new_shelf.line_right = if new_shelf.line_right_edges.is_empty() {
                        LayoutUnit::Max()
                    } else {
                        old_shelf.line_right
                    };
                    self.shelves_
                        .insert(if removed { index } else { index + 1 }, new_shelf);
                }
                break;
            }
            if !removed {
                index += 1;
            }
        }
        debug_assert!(inserted);
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.cc:633-684
    pub fn FindLayoutOpportunity(
        &self,
        offset: &BfcOffset,
        available_inline_size: LayoutUnit,
        direction: TextDirection,
        minimum_inline_size: LayoutUnit,
    ) -> LayoutOpportunity {
        debug_assert!(offset.block_offset >= self.block_offset_limit_);
        let mut result = LayoutOpportunity::default();
        self.IterateAllLayoutOpportunities(
            offset,
            available_inline_size,
            direction,
            |opportunity| {
                if opportunity.rect.InlineSize() >= minimum_inline_size
                    || (opportunity.rect.InlineSize() == available_inline_size
                        && opportunity.rect.LineStartOffset() == offset.line_offset)
                {
                    result = opportunity;
                    true
                } else {
                    false
                }
            },
        );
        result
    }

    pub fn AllLayoutOpportunities(
        &self,
        offset: &BfcOffset,
        available_inline_size: LayoutUnit,
        direction: TextDirection,
    ) -> LayoutOpportunityVector {
        debug_assert!(offset.block_offset >= self.block_offset_limit_);
        let mut opportunities = LayoutOpportunityVector::default();
        self.IterateAllLayoutOpportunities(
            offset,
            available_inline_size,
            direction,
            |opportunity| {
                opportunities.push(opportunity);
                false
            },
        );
        opportunities
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.cc:686-752
    pub fn IterateAllLayoutOpportunities<F>(
        &self,
        offset: &BfcOffset,
        available_inline_size: LayoutUnit,
        direction: TextDirection,
        mut callback: F,
    ) where
        F: FnMut(LayoutOpportunity) -> bool,
    {
        let mut shelf_index = 0usize;
        let mut area_index = 0usize;
        while shelf_index < self.shelves_.len() || area_index < self.areas_.len() {
            debug_assert!(shelf_index < self.shelves_.len());
            let shelf = &self.shelves_[shelf_index];
            if area_index < self.areas_.len() {
                let area = &self.areas_[area_index];
                if !Intersects(&area.opportunity, offset, available_inline_size) {
                    area_index += 1;
                    continue;
                }
                let block_start = area
                    .opportunity
                    .rect
                    .BlockStartOffset()
                    .max(offset.block_offset);
                if block_start <= shelf.block_offset.max(offset.block_offset) {
                    let block_end = area.opportunity.rect.BlockEndOffset();
                    if HasSolidEdges(&area.line_left_edges, block_start, block_end)
                        && HasSolidEdges(&area.line_right_edges, block_start, block_end)
                        && callback(CreateLayoutOpportunityFromArea(
                            &area.opportunity,
                            offset,
                            available_inline_size,
                        ))
                    {
                        return;
                    }
                    area_index += 1;
                    continue;
                }
            }
            if HasSolidEdges(
                &shelf.line_left_edges,
                offset.block_offset,
                LayoutUnit::Max(),
            ) && HasSolidEdges(
                &shelf.line_right_edges,
                offset.block_offset,
                LayoutUnit::Max(),
            ) && callback(CreateLayoutOpportunityFromShelf(
                shelf,
                offset,
                available_inline_size,
                direction,
            )) {
                return;
            }
            shelf_index += 1;
        }
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.shelves_);
        visitor.Trace(&self.areas_);
    }
}

// cpp: layoutng/internal/exclusions/exclusion_space.h:29-39,416-541
pub struct ExclusionSpaceInternal {
    exclusions_: Member<GCedExclusionAreaPtrArray>,
    num_exclusions_: u32,
    left_clear_offset_: LayoutUnit,
    right_clear_offset_: LayoutUnit,
    last_float_block_start_: LayoutUnit,
    initial_letter_left_clear_offset_: LayoutUnit,
    initial_letter_right_clear_offset_: LayoutUnit,
    track_shape_exclusions_: bool,
    has_break_before_left_float_: bool,
    has_break_before_right_float_: bool,
    has_break_inside_left_float_: bool,
    has_break_inside_right_float_: bool,
    derived_geometry_: RefCell<Member<DerivedGeometry>>,
}

impl Default for ExclusionSpaceInternal {
    // cpp: layoutng/internal/exclusions/exclusion_space.cc:163-169
    fn default() -> Self {
        Self {
            exclusions_: Member::from_ptr(MakeGarbageCollected(
                GCedExclusionAreaPtrArray::default(),
            )),
            num_exclusions_: 0,
            left_clear_offset_: LayoutUnit::Min(),
            right_clear_offset_: LayoutUnit::Min(),
            last_float_block_start_: LayoutUnit::Min(),
            initial_letter_left_clear_offset_: LayoutUnit::Min(),
            initial_letter_right_clear_offset_: LayoutUnit::Min(),
            track_shape_exclusions_: false,
            has_break_before_left_float_: false,
            has_break_before_right_float_: false,
            has_break_inside_left_float_: false,
            has_break_inside_right_float_: false,
            derived_geometry_: RefCell::new(Member::default()),
        }
    }
}

impl Clone for ExclusionSpaceInternal {
    // cpp: layoutng/internal/exclusions/exclusion_space.cc:171-191
    fn clone(&self) -> Self {
        Self {
            exclusions_: Member::from_ptr(self.exclusions_.Get()),
            num_exclusions_: self.num_exclusions_,
            left_clear_offset_: self.left_clear_offset_,
            right_clear_offset_: self.right_clear_offset_,
            last_float_block_start_: self.last_float_block_start_,
            initial_letter_left_clear_offset_: self.initial_letter_left_clear_offset_,
            initial_letter_right_clear_offset_: self.initial_letter_right_clear_offset_,
            track_shape_exclusions_: self.track_shape_exclusions_,
            has_break_before_left_float_: self.has_break_before_left_float_,
            has_break_before_right_float_: self.has_break_before_right_float_,
            has_break_inside_left_float_: self.has_break_inside_left_float_,
            has_break_inside_right_float_: self.has_break_inside_right_float_,
            derived_geometry_: RefCell::new(self.derived_geometry_.replace(Member::default())),
        }
    }
}

impl ExclusionSpaceInternal {
    // cpp: layoutng/internal/exclusions/exclusion_space.cc:753-806
    pub fn GetDerivedGeometry(&self, mut block_offset_limit: LayoutUnit) -> &DerivedGeometry {
        let mut geometry = self.derived_geometry_.borrow().Get();
        if !geometry.is_null() && block_offset_limit < unsafe { &*geometry }.block_offset_limit_ {
            *self.derived_geometry_.borrow_mut() = Member::default();
            geometry = std::ptr::null_mut();
        }
        if geometry.is_null() {
            let exclusions = unsafe { &*self.exclusions_.Get() };
            debug_assert!(self.num_exclusions_ as usize <= exclusions.len());
            debug_assert!(self.num_exclusions_ > 0);
            let count = self.num_exclusions_ as usize;
            let mut index = 0usize;
            while index < count
                && unsafe { &*exclusions[index].Get() }.rect.BlockStartOffset() < block_offset_limit
            {
                index += 1;
            }
            if index == 0 {
                block_offset_limit = LayoutUnit::Min();
            } else {
                index -= 1;
                while index > 0 && !unsafe { &*exclusions[index].Get() }.is_past_other_exclusions {
                    index -= 1;
                }
                block_offset_limit = unsafe { &*exclusions[index].Get() }.rect.BlockStartOffset();
            }
            geometry = MakeGarbageCollected(DerivedGeometry::new(
                block_offset_limit,
                self.track_shape_exclusions_,
            ));
            for member in exclusions.iter().take(count).skip(index) {
                unsafe { &mut *geometry }.Add(unsafe { &*member.Get() });
            }
            *self.derived_geometry_.borrow_mut() = Member::from_ptr(geometry);
        }
        unsafe { &*geometry }
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:48-91
    pub fn FindLayoutOpportunity(
        &self,
        offset: &BfcOffset,
        available_inline_size: LayoutUnit,
        direction: TextDirection,
        minimum_inline_size: LayoutUnit,
    ) -> LayoutOpportunity {
        let max_clear_offset = self
            .left_clear_offset_
            .max(self.right_clear_offset_)
            .max(self.initial_letter_left_clear_offset_)
            .max(self.initial_letter_right_clear_offset_);
        if offset.block_offset >= max_clear_offset {
            let end = BfcOffset::new(
                offset.line_offset + available_inline_size.ClampNegativeToZero(),
                LayoutUnit::Max(),
            );
            return LayoutOpportunity::without_shapes(&BfcRect::new(*offset, end));
        }
        self.GetDerivedGeometry(offset.block_offset)
            .FindLayoutOpportunity(
                offset,
                available_inline_size,
                direction,
                minimum_inline_size,
            )
    }

    pub fn AllLayoutOpportunities(
        &self,
        offset: &BfcOffset,
        available_inline_size: LayoutUnit,
        direction: TextDirection,
    ) -> LayoutOpportunityVector {
        let max_clear_offset = self
            .left_clear_offset_
            .max(self.right_clear_offset_)
            .max(self.initial_letter_left_clear_offset_)
            .max(self.initial_letter_right_clear_offset_);
        if offset.block_offset >= max_clear_offset {
            let end = BfcOffset::new(
                offset.line_offset + available_inline_size.ClampNegativeToZero(),
                LayoutUnit::Max(),
            );
            let mut opportunities = LayoutOpportunityVector::default();
            opportunities.push(LayoutOpportunity::without_shapes(&BfcRect::new(
                *offset, end,
            )));
            return opportunities;
        }
        self.GetDerivedGeometry(offset.block_offset)
            .AllLayoutOpportunities(offset, available_inline_size, direction)
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.cc:237-334
    pub fn Add(&mut self, exclusion: *const ExclusionArea) {
        let source = unsafe { &*exclusion };
        let existing = unsafe { &*self.exclusions_.Get() };
        debug_assert!(self.num_exclusions_ as usize <= existing.len());

        let mut already_exists = false;
        if (self.num_exclusions_ as usize) < existing.len() {
            if source == unsafe { &*existing[self.num_exclusions_ as usize].Get() } {
                already_exists = true;
            } else {
                let mut copied = GCedExclusionAreaPtrArray::default();
                for member in existing.iter().take(self.num_exclusions_ as usize) {
                    copied.push(Member::from_ptr(member.Get()));
                }
                self.exclusions_ = Member::from_ptr(MakeGarbageCollected(copied));
            }
        }

        if !self.track_shape_exclusions_ && !source.shape_data.Get().is_null() {
            self.track_shape_exclusions_ = true;
            *self.derived_geometry_.borrow_mut() = Member::default();
        }

        let block_start = source.rect.BlockStartOffset();
        unsafe { &mut *(exclusion as *mut ExclusionArea) }.is_past_other_exclusions = block_start
            >= self
                .left_clear_offset_
                .max(self.right_clear_offset_)
                .max(self.initial_letter_left_clear_offset_)
                .max(self.initial_letter_right_clear_offset_);

        let source = unsafe { &*exclusion };
        let clear_offset = source.rect.BlockEndOffset();
        if source.IsForInitialLetterBox() {
            if source.r#type == EFloat::kLeft {
                self.initial_letter_left_clear_offset_ =
                    self.initial_letter_left_clear_offset_.max(clear_offset);
            } else if source.r#type == EFloat::kRight {
                self.initial_letter_right_clear_offset_ =
                    self.initial_letter_right_clear_offset_.max(clear_offset);
            }

            if !already_exists {
                let existing = unsafe { &*self.exclusions_.Get() };
                let mut copied = GCedExclusionAreaPtrArray::default();
                let mut inserted = false;
                for member in existing.iter().take(self.num_exclusions_ as usize) {
                    if !inserted && block_start < unsafe { &*member.Get() }.rect.BlockStartOffset()
                    {
                        copied.push(Member::from_ptr(exclusion as *mut ExclusionArea));
                        inserted = true;
                    }
                    copied.push(Member::from_ptr(member.Get()));
                }
                if !inserted {
                    copied.push(Member::from_ptr(exclusion as *mut ExclusionArea));
                }
                self.exclusions_ = Member::from_ptr(MakeGarbageCollected(copied));
            }
            self.num_exclusions_ += 1;

            let exclusions = unsafe { &*self.exclusions_.Get() };
            if exclusions[self.num_exclusions_ as usize - 1].Get() != exclusion as *mut _ {
                *self.derived_geometry_.borrow_mut() = Member::default();
            }
            let geometry = self.derived_geometry_.borrow().Get();
            if !geometry.is_null() {
                unsafe { &mut *geometry }.Add(source);
            }
            return;
        }

        self.last_float_block_start_ = self.last_float_block_start_.max(block_start);
        if source.r#type == EFloat::kLeft {
            self.left_clear_offset_ = self.left_clear_offset_.max(clear_offset);
        } else if source.r#type == EFloat::kRight {
            self.right_clear_offset_ = self.right_clear_offset_.max(clear_offset);
        }

        let geometry = self.derived_geometry_.borrow().Get();
        if !geometry.is_null() {
            unsafe { &mut *geometry }.Add(source);
        }
        if !already_exists {
            unsafe { &mut *self.exclusions_.Get() }
                .push(Member::from_ptr(exclusion as *mut ExclusionArea));
        }
        self.num_exclusions_ += 1;
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.cc:193-216
    pub fn AssignFrom(&mut self, other: &Self) {
        self.CopyFrom(other);
        self.MoveDerivedGeometry(other);
    }

    pub fn CopyFrom(&mut self, other: &Self) {
        self.exclusions_ = Member::from_ptr(other.exclusions_.Get());
        self.num_exclusions_ = other.num_exclusions_;
        self.left_clear_offset_ = other.left_clear_offset_;
        self.right_clear_offset_ = other.right_clear_offset_;
        self.last_float_block_start_ = other.last_float_block_start_;
        self.initial_letter_left_clear_offset_ = other.initial_letter_left_clear_offset_;
        self.initial_letter_right_clear_offset_ = other.initial_letter_right_clear_offset_;
        self.track_shape_exclusions_ = other.track_shape_exclusions_;
        self.has_break_before_left_float_ = other.has_break_before_left_float_;
        self.has_break_before_right_float_ = other.has_break_before_right_float_;
        self.has_break_inside_left_float_ = other.has_break_inside_left_float_;
        self.has_break_inside_right_float_ = other.has_break_inside_right_float_;
        *self.derived_geometry_.borrow_mut() = Member::default();
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:41-44
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.exclusions_);
        visitor.Trace(&*self.derived_geometry_.borrow());
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:93-106
    pub fn ClearanceOffset(&self, clear_type: EClear) -> LayoutUnit {
        match clear_type {
            EClear::kNone => LayoutUnit::Min(),
            EClear::kLeft => self.left_clear_offset_,
            EClear::kRight => self.right_clear_offset_,
            EClear::kBoth => self.left_clear_offset_.max(self.right_clear_offset_),
            _ => unreachable!("invalid clear type"),
        }
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:108-111
    pub fn ClearanceOffsetIncludingInitialLetter(&self, clear_type: EClear) -> LayoutUnit {
        self.ClearanceOffset(clear_type)
            .max(self.InitialLetterClearanceOffset(EClear::kBoth))
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:113-127
    pub fn InitialLetterClearanceOffset(&self, clear_type: EClear) -> LayoutUnit {
        match clear_type {
            EClear::kNone => LayoutUnit::Min(),
            EClear::kLeft => self.initial_letter_left_clear_offset_,
            EClear::kRight => self.initial_letter_right_clear_offset_,
            EClear::kBoth => self
                .initial_letter_left_clear_offset_
                .max(self.initial_letter_right_clear_offset_),
            _ => unreachable!("invalid clear type"),
        }
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:129-134
    pub fn InitialLetterClearanceOffsetForFloat(&self, float_type: EFloat) -> LayoutUnit {
        if float_type == EFloat::kLeft {
            self.initial_letter_left_clear_offset_
        } else {
            debug_assert_eq!(float_type, EFloat::kRight);
            self.initial_letter_right_clear_offset_
        }
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:136-160
    pub fn SetHasBreakBeforeFloat(&mut self, float_type: EFloat) {
        match float_type {
            EFloat::kLeft => self.has_break_before_left_float_ = true,
            EFloat::kRight => self.has_break_before_right_float_ = true,
            _ => unreachable!("invalid float type"),
        }
    }

    pub fn SetHasBreakInsideFloat(&mut self, float_type: EFloat) {
        match float_type {
            EFloat::kLeft => self.has_break_inside_left_float_ = true,
            EFloat::kRight => self.has_break_inside_right_float_ = true,
            _ => unreachable!("invalid float type"),
        }
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:162-194
    pub fn NeedsClearancePastFragmentainer(&self, clear_type: EClear) -> bool {
        match clear_type {
            EClear::kNone => false,
            EClear::kLeft => self.has_break_inside_left_float_ || self.has_break_before_left_float_,
            EClear::kRight => {
                self.has_break_inside_right_float_ || self.has_break_before_right_float_
            }
            EClear::kBoth => {
                self.has_break_inside_left_float_
                    || self.has_break_before_left_float_
                    || self.has_break_inside_right_float_
                    || self.has_break_before_right_float_
            }
            _ => unreachable!("invalid clear type"),
        }
    }

    pub fn NeedsBreakBeforeFloat(&self, clear_type: EClear) -> bool {
        self.has_break_before_left_float_
            || self.has_break_before_right_float_
            || self.NeedsClearancePastFragmentainer(clear_type)
    }

    pub fn HasFragmentainerBreak(&self) -> bool {
        self.NeedsClearancePastFragmentainer(EClear::kBoth)
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:196-198
    pub fn LastFloatBlockStart(&self) -> LayoutUnit {
        self.last_float_block_start_
    }

    pub fn IsEmpty(&self) -> bool {
        self.num_exclusions_ == 0
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:202-207
    pub fn PreInitialize(&mut self, other: &Self) {
        let exclusions = unsafe { &*self.exclusions_.Get() };
        let other_exclusions = unsafe { &*other.exclusions_.Get() };
        debug_assert!(exclusions.is_empty());
        debug_assert!(!other_exclusions.is_empty());
        self.exclusions_ = Member::from_ptr(other.exclusions_.Get());
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:210-231
    pub fn MoveAndUpdateDerivedGeometry(&mut self, other: &Self) {
        if other.derived_geometry_.borrow().Get().is_null() {
            return;
        }
        self.MoveDerivedGeometry(other);
        for index in other.num_exclusions_..self.num_exclusions_ {
            let exclusions = unsafe { &*self.exclusions_.Get() };
            let exclusion = unsafe { &*exclusions[index as usize].Get() };
            if !self.track_shape_exclusions_ && !exclusion.shape_data.Get().is_null() {
                self.track_shape_exclusions_ = true;
                *self.derived_geometry_.borrow_mut() = Member::default();
                return;
            }
            let geometry = self.derived_geometry_.borrow().Get();
            unsafe { &mut *geometry }.Add(exclusion);
        }
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:234-241
    pub fn MoveDerivedGeometry(&mut self, other: &Self) {
        let mut other_geometry = other.derived_geometry_.borrow_mut();
        if other_geometry.Get().is_null() {
            return;
        }
        self.track_shape_exclusions_ = other.track_shape_exclusions_;
        *self.derived_geometry_.borrow_mut() = Member::from_ptr(other_geometry.Get());
        *other_geometry = Member::default();
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:244-253
    pub fn MergeExclusionSpaces(
        &mut self,
        offset_delta: &BfcDelta,
        previous_output: &Self,
        previous_input: *const Self,
    ) {
        let begin = if previous_input.is_null() {
            0
        } else {
            unsafe { &*previous_input }.num_exclusions_
        };
        let previous_exclusions = unsafe { &*previous_output.exclusions_.Get() };
        for index in begin..previous_output.num_exclusions_ {
            let exclusion = unsafe { &*previous_exclusions[index as usize].Get() };
            self.Add(exclusion.CopyWithOffset(offset_delta));
        }
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:257-268
    #[cfg(debug_assertions)]
    pub fn CheckSameForSimplifiedLayout(&self, other: &Self) {
        debug_assert_eq!(self.num_exclusions_, other.num_exclusions_);
        let exclusions = unsafe { &*self.exclusions_.Get() };
        let other_exclusions = unsafe { &*other.exclusions_.Get() };
        for index in 0..self.num_exclusions_ as usize {
            let exclusion = unsafe { &*exclusions[index].Get() };
            let other_exclusion = unsafe { &*other_exclusions[index].Get() };
            debug_assert_eq!(exclusion.rect, other_exclusion.rect);
            debug_assert_eq!(exclusion.r#type, other_exclusion.r#type);
            debug_assert_eq!(
                !exclusion.shape_data.Get().is_null(),
                !other_exclusion.shape_data.Get().is_null()
            );
        }
    }
}

// cpp: layoutng/internal/exclusions/exclusion_space.h:255
// cpp: layoutng/internal/exclusions/exclusion_space.cc:808-818
impl PartialEq for ExclusionSpaceInternal {
    fn eq(&self, other: &Self) -> bool {
        if self.num_exclusions_ == 0 && other.num_exclusions_ == 0 {
            return true;
        }
        self.num_exclusions_ == other.num_exclusions_
            && self.exclusions_.Get() == other.exclusions_.Get()
            && self.has_break_before_left_float_ == other.has_break_before_left_float_
            && self.has_break_before_right_float_ == other.has_break_before_right_float_
            && self.has_break_inside_left_float_ == other.has_break_inside_left_float_
            && self.has_break_inside_right_float_ == other.has_break_inside_right_float_
    }
}

impl Eq for ExclusionSpaceInternal {}

// cpp: layoutng/internal/exclusions/exclusion_space.h:547-785
#[derive(Default)]
pub struct ExclusionSpace {
    exclusion_space_: RefCell<Member<ExclusionSpaceInternal>>,
}

impl Clone for ExclusionSpace {
    // cpp: layoutng/internal/exclusions/exclusion_space.h:553-569
    fn clone(&self) -> Self {
        let internal = self.exclusion_space_.borrow().Get();
        if internal.is_null() {
            Self::default()
        } else {
            Self {
                exclusion_space_: RefCell::new(Member::from_ptr(MakeGarbageCollected(
                    unsafe { &*internal }.clone(),
                ))),
            }
        }
    }
}

impl ExclusionSpace {
    fn internal_ptr(&self) -> *mut ExclusionSpaceInternal {
        self.exclusion_space_.borrow().Get()
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.cc:218-225
    pub fn CopyFrom(&mut self, other: &Self) {
        let other_internal = other.internal_ptr();
        if other_internal.is_null() {
            *self.exclusion_space_.borrow_mut() = Member::default();
            return;
        }
        let mut internal = ExclusionSpaceInternal::default();
        internal.CopyFrom(unsafe { &*other_internal });
        *self.exclusion_space_.borrow_mut() = Member::from_ptr(MakeGarbageCollected(internal));
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:575-581
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&*self.exclusion_space_.borrow());
    }

    pub fn Add(&mut self, exclusion: *const ExclusionArea) {
        let mut internal = self.internal_ptr();
        if internal.is_null() {
            internal = MakeGarbageCollected(ExclusionSpaceInternal::default());
            *self.exclusion_space_.borrow_mut() = Member::from_ptr(internal);
        }
        unsafe { &mut *internal }.Add(exclusion);
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:583-609
    pub fn SetHasBreakBeforeFloat(&mut self, float_type: EFloat) {
        let internal = self.internal_ptr();
        debug_assert!(!internal.is_null());
        unsafe { &mut *internal }.SetHasBreakBeforeFloat(float_type);
    }

    pub fn SetHasBreakInsideFloat(&mut self, float_type: EFloat) {
        let internal = self.internal_ptr();
        debug_assert!(!internal.is_null());
        unsafe { &mut *internal }.SetHasBreakInsideFloat(float_type);
    }

    pub fn NeedsClearancePastFragmentainer(&self, clear_type: EClear) -> bool {
        let internal = self.internal_ptr();
        !internal.is_null() && unsafe { &*internal }.NeedsClearancePastFragmentainer(clear_type)
    }

    pub fn NeedsBreakBeforeFloat(&self, clear_type: EClear) -> bool {
        let internal = self.internal_ptr();
        !internal.is_null() && unsafe { &*internal }.NeedsBreakBeforeFloat(clear_type)
    }

    pub fn HasFragmentainerBreak(&self) -> bool {
        let internal = self.internal_ptr();
        !internal.is_null() && unsafe { &*internal }.HasFragmentainerBreak()
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:611-644
    pub fn FindLayoutOpportunity(
        &self,
        offset: &BfcOffset,
        available_inline_size: LayoutUnit,
        direction: TextDirection,
        minimum_inline_size: LayoutUnit,
    ) -> LayoutOpportunity {
        let internal = self.internal_ptr();
        if internal.is_null() {
            let end = BfcOffset::new(
                offset.line_offset + available_inline_size.ClampNegativeToZero(),
                LayoutUnit::Max(),
            );
            return LayoutOpportunity::without_shapes(&BfcRect::new(*offset, end));
        }
        unsafe { &*internal }.FindLayoutOpportunity(
            offset,
            available_inline_size,
            direction,
            minimum_inline_size,
        )
    }

    // C++ supplies a zero minimum-inline-size default argument.
    // cpp: layoutng/internal/exclusions/exclusion_space.h:619-619
    pub fn FindLayoutOpportunityDefault(
        &self,
        offset: &BfcOffset,
        available_inline_size: LayoutUnit,
        direction: TextDirection,
    ) -> LayoutOpportunity {
        self.FindLayoutOpportunity(
            offset,
            available_inline_size,
            direction,
            LayoutUnit::default(),
        )
    }

    pub fn AllLayoutOpportunities(
        &self,
        offset: &BfcOffset,
        available_inline_size: LayoutUnit,
        direction: TextDirection,
    ) -> LayoutOpportunityVector {
        let internal = self.internal_ptr();
        if internal.is_null() {
            let end = BfcOffset::new(
                offset.line_offset + available_inline_size.ClampNegativeToZero(),
                LayoutUnit::Max(),
            );
            let mut opportunities = LayoutOpportunityVector::default();
            opportunities.push(LayoutOpportunity::without_shapes(&BfcRect::new(
                *offset, end,
            )));
            return opportunities;
        }
        unsafe { &*internal }.AllLayoutOpportunities(offset, available_inline_size, direction)
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:646-681
    pub fn ClearanceOffset(&self, clear_type: EClear) -> LayoutUnit {
        let internal = self.internal_ptr();
        if internal.is_null() {
            LayoutUnit::Min()
        } else {
            unsafe { &*internal }.ClearanceOffset(clear_type)
        }
    }

    pub fn ClearanceOffsetIncludingInitialLetter(&self, clear_type: EClear) -> LayoutUnit {
        let internal = self.internal_ptr();
        if internal.is_null() {
            LayoutUnit::Min()
        } else {
            unsafe { &*internal }.ClearanceOffsetIncludingInitialLetter(clear_type)
        }
    }

    pub fn InitialLetterClearanceOffset(&self, clear_type: EClear) -> LayoutUnit {
        let internal = self.internal_ptr();
        if internal.is_null() {
            LayoutUnit::Min()
        } else {
            unsafe { &*internal }.InitialLetterClearanceOffset(clear_type)
        }
    }

    pub fn InitialLetterClearanceOffsetForFloat(&self, float_type: EFloat) -> LayoutUnit {
        let internal = self.internal_ptr();
        if internal.is_null() {
            LayoutUnit::Min()
        } else {
            unsafe { &*internal }.InitialLetterClearanceOffsetForFloat(float_type)
        }
    }

    pub fn LastFloatBlockStart(&self) -> LayoutUnit {
        let internal = self.internal_ptr();
        if internal.is_null() {
            LayoutUnit::Min()
        } else {
            unsafe { &*internal }.LastFloatBlockStart()
        }
    }

    pub fn IsEmpty(&self) -> bool {
        let internal = self.internal_ptr();
        internal.is_null() || unsafe { &*internal }.IsEmpty()
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:683-714
    pub fn PreInitialize(&self, other: &Self) {
        if !self.internal_ptr().is_null() {
            return;
        }
        let other_internal = other.internal_ptr();
        if other_internal.is_null() {
            return;
        }
        let mut internal = ExclusionSpaceInternal::default();
        internal.PreInitialize(unsafe { &*other_internal });
        *self.exclusion_space_.borrow_mut() = Member::from_ptr(MakeGarbageCollected(internal));
    }

    pub fn MoveAndUpdateDerivedGeometry(&self, other: &Self) {
        let internal = self.internal_ptr();
        let other_internal = other.internal_ptr();
        if internal.is_null() || other_internal.is_null() {
            return;
        }
        unsafe { &mut *internal }.MoveAndUpdateDerivedGeometry(unsafe { &*other_internal });
    }

    pub fn MoveDerivedGeometry(&self, other: &Self) {
        debug_assert!(self == other);
        let internal = self.internal_ptr();
        let other_internal = other.internal_ptr();
        if internal.is_null() || other_internal.is_null() {
            return;
        }
        unsafe { &mut *internal }.MoveDerivedGeometry(unsafe { &*other_internal });
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:716-766
    pub fn MergeExclusionSpaces(
        old_output: &Self,
        old_input: &Self,
        new_input: &Self,
        offset_delta: &BfcDelta,
    ) -> Self {
        let new_output = new_input.clone();
        let old_output_internal = old_output.internal_ptr();
        if old_output_internal.is_null() || old_input == old_output {
            return new_output;
        }
        let mut new_output_internal = new_output.internal_ptr();
        if new_output_internal.is_null() {
            new_output_internal = MakeGarbageCollected(ExclusionSpaceInternal::default());
            *new_output.exclusion_space_.borrow_mut() = Member::from_ptr(new_output_internal);
        }
        unsafe { &mut *new_output_internal }.MergeExclusionSpaces(
            offset_delta,
            unsafe { &*old_output_internal },
            old_input.internal_ptr(),
        );
        new_output
    }

    // cpp: layoutng/internal/exclusions/exclusion_space.h:776-782
    #[cfg(debug_assertions)]
    pub fn CheckSameForSimplifiedLayout(&self, other: &Self) {
        let lhs = self.internal_ptr();
        let rhs = other.internal_ptr();
        debug_assert_eq!(lhs.is_null(), rhs.is_null());
        if !lhs.is_null() {
            unsafe { &*lhs }.CheckSameForSimplifiedLayout(unsafe { &*rhs });
        }
    }
}

// cpp: layoutng/internal/exclusions/exclusion_space.h:768-774
impl PartialEq for ExclusionSpace {
    fn eq(&self, other: &Self) -> bool {
        let lhs = self.internal_ptr();
        let rhs = other.internal_ptr();
        if lhs == rhs {
            true
        } else if lhs.is_null() || rhs.is_null() {
            false
        } else {
            (unsafe { &*lhs }) == (unsafe { &*rhs })
        }
    }
}

impl Eq for ExclusionSpace {}
