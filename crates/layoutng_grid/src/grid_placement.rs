use crate::{grid_data::GridPlacementData, grid_line_resolver::GridLineResolver};
use foundation::doubly_linked_list::{DoublyLinkedList, DoublyLinkedListNode};
use foundation::{HashMap, IntWithZeroKeyHashTraits};
use layoutng_assembly::internal::{
    grid_item::GridItems, grid_track_collection::GridLayoutTrackCollection,
};
use layoutng_style::style::{
    computed_style::ComputedStyle,
    grid_area::{GridArea, GridSpan},
    grid_enums::GridTrackSizingDirection::{self, *},
};

// cpp: layoutng_grid/grid_placement.cc:15-36
#[derive(Clone, Copy, PartialEq, Eq)]
enum AutoPlacementType {
    kNotNeeded,
    kMajor,
    kMinor,
    kBoth,
}
fn AutoPlacement(position: &GridArea, major: GridTrackSizingDirection) -> AutoPlacementType {
    let minor = if major == kForColumns {
        kForRows
    } else {
        kForColumns
    };
    let major_span = position.Span(major);
    let minor_span = position.Span(minor);
    assert!(!major_span.IsUntranslatedDefinite() && !minor_span.IsUntranslatedDefinite());
    if minor_span.IsIndefinite() && major_span.IsIndefinite() {
        return AutoPlacementType::kBoth;
    }
    if minor_span.IsIndefinite() {
        return AutoPlacementType::kMinor;
    }
    if major_span.IsIndefinite() {
        return AutoPlacementType::kMajor;
    }
    AutoPlacementType::kNotNeeded
}
// cpp: layoutng_grid/grid_placement.h:24-24
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PackingBehavior {
    kSparse,
    kDense,
}
// cpp: layoutng_grid/grid_placement.h:42-42
#[derive(Clone, Copy, PartialEq, Eq)]
enum CursorMovementBehavior {
    kAuto,
    kForceMajorLine,
    kForceMinorLine,
}
// cpp: layoutng_grid/grid_placement.h:44-50
// cpp: layoutng_grid/grid_placement.cc:423-437
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct GridPosition {
    major_line: u32,
    minor_line: u32,
}
// cpp: layoutng_grid/grid_placement.h:52-82
struct PlacedGridItem {
    start_: GridPosition,
    end_: GridPosition,
    next_: *mut PlacedGridItem,
    prev_: *mut PlacedGridItem,
}
impl PlacedGridItem {
    // cpp: layoutng_grid/grid_placement.cc:439-454
    fn new(
        position: &GridArea,
        major: GridTrackSizingDirection,
        minor: GridTrackSizingDirection,
    ) -> Self {
        Self {
            start_: GridPosition {
                major_line: position.StartLine(major),
                minor_line: position.StartLine(minor),
            },
            end_: GridPosition {
                major_line: position.EndLine(major),
                minor_line: position.EndLine(minor),
            },
            next_: std::ptr::null_mut(),
            prev_: std::ptr::null_mut(),
        }
    }
    fn EndOnPreviousMajorLine(&self) -> GridPosition {
        debug_assert!(self.end_.major_line > 0);
        GridPosition {
            major_line: self.end_.major_line - 1,
            minor_line: self.end_.minor_line,
        }
    }
}
impl DoublyLinkedListNode for PlacedGridItem {
    fn SetPrev(&mut self, value: *mut Self) {
        self.prev_ = value;
    }
    fn SetNext(&mut self, value: *mut Self) {
        self.next_ = value;
    }
    fn Prev(&self) -> *mut Self {
        self.prev_
    }
    fn Next(&self) -> *mut Self {
        self.next_
    }
}

// cpp: layoutng_grid/grid_placement.h:84-125
struct AutoPlacementCursor {
    items_overlapping_major_line_: Vec<*mut PlacedGridItem>,
    should_move_to_next_item_major_end_line_: bool,
    next_placed_item_: *mut PlacedGridItem,
    current_position_: GridPosition,
}
impl AutoPlacementCursor {
    fn new(first: *mut PlacedGridItem) -> Self {
        Self {
            items_overlapping_major_line_: Vec::new(),
            should_move_to_next_item_major_end_line_: true,
            next_placed_item_: first,
            current_position_: GridPosition::default(),
        }
    }
    // Source libc++ comparator: rhs->End() < lhs->End(). Preserve vector heap
    // layout as the source scans that vector, rather than scanning a sorted copy.
    fn Compare(lhs: *mut PlacedGridItem, rhs: *mut PlacedGridItem) -> bool {
        unsafe { (*rhs).end_ < (*lhs).end_ }
    }
    fn SiftUp(&mut self, mut index: usize) {
        let value = self.items_overlapping_major_line_[index];
        while index > 0 {
            let parent = (index - 1) / 2;
            if !Self::Compare(self.items_overlapping_major_line_[parent], value) {
                break;
            }
            self.items_overlapping_major_line_[index] = self.items_overlapping_major_line_[parent];
            index = parent;
        }
        self.items_overlapping_major_line_[index] = value;
    }
    fn PushHeap(&mut self, item: *mut PlacedGridItem) {
        self.items_overlapping_major_line_.push(item);
        self.SiftUp(self.items_overlapping_major_line_.len() - 1);
    }
    fn PopHeap(&mut self) {
        // libc++ __pop_heap + __floyd_sift_down: equal children prefer left.
        let length = self.items_overlapping_major_line_.len();
        if length > 1 {
            let top = self.items_overlapping_major_line_[0];
            let mut hole = 0;
            loop {
                let mut child = 2 * hole + 1;
                if child + 1 < length
                    && Self::Compare(
                        self.items_overlapping_major_line_[child],
                        self.items_overlapping_major_line_[child + 1],
                    )
                {
                    child += 1;
                }
                self.items_overlapping_major_line_[hole] =
                    self.items_overlapping_major_line_[child];
                hole = child;
                if hole > (length - 2) / 2 {
                    break;
                }
            }
            if hole == length - 1 {
                self.items_overlapping_major_line_[hole] = top;
            } else {
                self.items_overlapping_major_line_[hole] =
                    self.items_overlapping_major_line_[length - 1];
                self.items_overlapping_major_line_[length - 1] = top;
                self.SiftUp(hole);
            }
        }
        self.items_overlapping_major_line_.pop();
    }
    // cpp: layoutng_grid/grid_placement.cc:456-552
    fn MoveCursorToFitGridSpan(
        &mut self,
        major_size: u32,
        minor_size: u32,
        minor_max_end: u32,
        movement: CursorMovementBehavior,
    ) {
        debug_assert!(minor_size <= minor_max_end);
        let allow_minor = movement != CursorMovementBehavior::kForceMinorLine;
        let minor_max_start = if movement == CursorMovementBehavior::kForceMajorLine {
            minor_max_end
        } else {
            minor_max_end - minor_size
        };
        if self.current_position_.minor_line > minor_max_start {
            self.MoveToNextMajorLine(allow_minor);
        }
        loop {
            self.UpdateItemsOverlappingMajorLine();
            let mut next_minor = self.current_position_.minor_line;
            for &item in &self.items_overlapping_major_line_ {
                let item = unsafe { &*item };
                let minor_end = next_minor + minor_size;
                let item_end = item.end_.minor_line;
                if next_minor < item_end && item.start_.minor_line < minor_end {
                    next_minor = item_end;
                    if self.NeedsToMoveToNextMajorLine(next_minor, minor_max_start, allow_minor) {
                        break;
                    }
                }
            }
            if !self.DoesCurrentPositionFitGridSpan(next_minor, minor_max_start, allow_minor) {
                continue;
            }
            let mut upcoming = self.next_placed_item_;
            while !upcoming.is_null() {
                let item = unsafe { &*upcoming };
                let major_end = self.current_position_.major_line + major_size;
                let minor_end = next_minor + minor_size;
                let item_end = item.end_.minor_line;
                if next_minor < item_end
                    && self.current_position_.major_line < item.end_.major_line
                    && item.start_.major_line < major_end
                    && item.start_.minor_line < minor_end
                {
                    next_minor = item_end;
                    if self.NeedsToMoveToNextMajorLine(next_minor, minor_max_start, allow_minor) {
                        break;
                    }
                }
                upcoming = item.Next();
            }
            if self.DoesCurrentPositionFitGridSpan(next_minor, minor_max_start, allow_minor) {
                break;
            }
        }
    }
    // The two source lambdas remain same-order calls; explicit arguments avoid
    // conflicting captures of mutable self in Rust.
    fn NeedsToMoveToNextMajorLine(&self, next: u32, maximum: u32, allow_minor: bool) -> bool {
        next > maximum || (!allow_minor && next != self.current_position_.minor_line)
    }
    fn DoesCurrentPositionFitGridSpan(
        &mut self,
        next: u32,
        maximum: u32,
        allow_minor: bool,
    ) -> bool {
        if self.NeedsToMoveToNextMajorLine(next, maximum, allow_minor) {
            self.MoveToNextMajorLine(allow_minor);
        } else {
            if self.current_position_.minor_line == next {
                return true;
            }
            debug_assert!(self.current_position_.minor_line < next);
            self.MoveToMinorLine(next);
        }
        false
    }
    // cpp: layoutng_grid/grid_placement.cc:554-602
    fn UpdateItemsOverlappingMajorLine(&mut self) {
        debug_assert!(
            (1..self.items_overlapping_major_line_.len()).all(|i| !Self::Compare(
                self.items_overlapping_major_line_[(i - 1) / 2],
                self.items_overlapping_major_line_[i]
            ))
        );
        while !self.items_overlapping_major_line_.is_empty() {
            let last = unsafe { &*self.items_overlapping_major_line_[0] }.EndOnPreviousMajorLine();
            if self.current_position_ < last {
                break;
            }
            if self.current_position_.major_line == last.major_line {
                self.should_move_to_next_item_major_end_line_ = false;
            }
            self.PopHeap();
        }
        while !self.next_placed_item_.is_null()
            && unsafe { &*self.next_placed_item_ }.start_ <= self.current_position_
        {
            let item = self.next_placed_item_;
            let last = unsafe { &*item }.EndOnPreviousMajorLine();
            if self.current_position_.major_line <= last.major_line {
                self.should_move_to_next_item_major_end_line_ = false;
            }
            if self.current_position_ < last {
                self.PushHeap(item);
            }
            self.next_placed_item_ = unsafe { &*item }.Next();
        }
    }
    // cpp: layoutng_grid/grid_placement.cc:604-609
    fn MoveToMajorLine(&mut self, line: u32) {
        debug_assert!(self.current_position_.major_line <= line);
        self.current_position_.major_line = line;
    }
    // cpp: layoutng_grid/grid_placement.cc:611-619
    fn MoveToMinorLine(&mut self, line: u32) {
        if line < self.current_position_.minor_line {
            self.current_position_.major_line += 1;
        }
        self.current_position_.minor_line = line;
    }
    // cpp: layoutng_grid/grid_placement.cc:621-638
    fn MoveToNextMajorLine(&mut self, allow_minor: bool) {
        self.current_position_.major_line += 1;
        if self.should_move_to_next_item_major_end_line_
            && !self.items_overlapping_major_line_.is_empty()
        {
            let end = unsafe { &*self.items_overlapping_major_line_[0] }
                .end_
                .major_line;
            debug_assert!(end >= self.current_position_.major_line);
            self.current_position_.major_line = end;
        }
        if allow_minor {
            self.current_position_.minor_line = 0;
        }
        self.should_move_to_next_item_major_end_line_ = true;
    }
    // cpp: layoutng_grid/grid_placement.cc:640-658
    fn InsertPlacedItemAtCurrentPosition(&mut self, item: *mut PlacedGridItem) {
        if !self.next_placed_item_.is_null() {
            debug_assert_eq!(unsafe { &*self.next_placed_item_ }.Prev(), item);
            debug_assert!(unsafe { &*item }.start_ < unsafe { &*self.next_placed_item_ }.start_);
        }
        debug_assert_eq!(unsafe { &*item }.Next(), self.next_placed_item_);
        self.next_placed_item_ = item;
        self.MoveToMinorLine(unsafe { &*item }.end_.minor_line);
        self.UpdateItemsOverlappingMajorLine();
    }
}
// cpp: layoutng_grid/grid_placement.h:127-134
#[derive(Default)]
struct PlacedGridItemsList {
    item_vector: Vec<Box<PlacedGridItem>>,
    ordered_list: DoublyLinkedList<PlacedGridItem>,
    needs_to_sort_item_vector: bool,
}
impl PlacedGridItemsList {
    // cpp: layoutng_grid/grid_placement.cc:660-677
    fn AppendCurrentItemsToOrderedList(&mut self) {
        debug_assert!(self.ordered_list.empty());
        if self.needs_to_sort_item_vector {
            self.item_vector
                .sort_unstable_by(|a, b| a.start_.cmp(&b.start_));
        }
        debug_assert!(self
            .item_vector
            .windows(2)
            .all(|p| p[0].start_ <= p[1].start_));
        for item in &mut self.item_vector {
            unsafe {
                self.ordered_list.Append(&mut **item);
            }
        }
    }
}
// cpp: layoutng_grid/grid_placement.h:19-175
pub struct GridPlacement {
    placement_data_: GridPlacementData,
    packing_behavior_: PackingBehavior,
    major_direction_: GridTrackSizingDirection,
    minor_direction_: GridTrackSizingDirection,
    minor_max_end_line_: u32,
}
impl GridPlacement {
    // cpp: layoutng_grid/grid_placement.cc:40-52
    pub fn new(style: &ComputedStyle, resolver: &GridLineResolver) -> Self {
        Self {
            placement_data_: GridPlacementData::new(resolver),
            packing_behavior_: if style.IsGridAutoFlowAlgorithmSparse() {
                PackingBehavior::kSparse
            } else {
                PackingBehavior::kDense
            },
            major_direction_: if style.IsGridAutoFlowDirectionRow() {
                kForRows
            } else {
                kForColumns
            },
            minor_direction_: if style.IsGridAutoFlowDirectionRow() {
                kForColumns
            } else {
                kForRows
            },
            minor_max_end_line_: 0,
        }
    }
    // cpp: layoutng_grid/grid_placement.cc:55-123
    // Consuming self represents the source's one-call contract and moved result.
    pub fn RunAutoPlacementAlgorithm(mut self, items: &GridItems) -> GridPlacementData {
        let mut placed = PlacedGridItemsList::default();
        let mut locked = Vec::new();
        let mut unlocked = Vec::new();
        if !self.PlaceNonAutoGridItems(items, &mut placed, &mut locked, &mut unlocked) {
            self.ClampGridItemsToFitSubgridArea(kForColumns);
            self.ClampGridItemsToFitSubgridArea(kForRows);
            return self.placement_data_;
        }
        placed.AppendCurrentItemsToOrderedList();
        self.PlaceGridItemsLockedToMajorAxis(&locked, &mut placed);
        self.ClampGridItemsToFitSubgridArea(kForColumns);
        self.ClampGridItemsToFitSubgridArea(kForRows);
        let dense = !self.HasSparsePacking();
        let mut cursor = AutoPlacementCursor::new(placed.ordered_list.Head());
        for position in unlocked {
            let position = unsafe { &mut *position };
            match AutoPlacement(position, self.major_direction_) {
                AutoPlacementType::kBoth => {
                    self.PlaceAutoBothAxisGridItem(position, &mut placed, &mut cursor)
                }
                AutoPlacementType::kMajor => {
                    self.PlaceAutoMajorAxisGridItem(position, &mut placed, &mut cursor)
                }
                _ => unreachable!("non-auto/major-locked item should already be placed"),
            }
            if dense {
                cursor = AutoPlacementCursor::new(placed.ordered_list.Head());
            }
        }
        self.ClampGridItemsToFitSubgridArea(kForColumns);
        self.ClampGridItemsToFitSubgridArea(kForRows);
        self.placement_data_
    }
    // cpp: layoutng_grid/grid_placement.cc:125-224
    fn PlaceNonAutoGridItems(
        &mut self,
        items: &GridItems,
        placed: &mut PlacedGridItemsList,
        locked: &mut Vec<*mut GridArea>,
        unlocked: &mut Vec<*mut GridArea>,
    ) -> bool {
        self.placement_data_
            .grid_item_positions
            .reserve(items.Size() as usize);
        self.placement_data_.column_start_offset = 0;
        self.placement_data_.row_start_offset = 0;
        let mut it = items.begin_const();
        let end = items.end_const();
        while it.NotEqual(&end) {
            let style = it.Current().node.Style();
            let resolver = &self.placement_data_.line_resolver;
            let mut position = GridArea::default();
            position.columns = resolver.ResolveGridPositionsFromStyle(style, kForColumns);
            debug_assert!(!position.columns.IsTranslatedDefinite());
            position.rows = resolver.ResolveGridPositionsFromStyle(style, kForRows);
            debug_assert!(!position.rows.IsTranslatedDefinite());
            if position.columns.IsUntranslatedDefinite() {
                self.placement_data_.column_start_offset =
                    (self.placement_data_.column_start_offset as i32)
                        .max(-position.columns.UntranslatedStartLine()) as u32;
            }
            if position.rows.IsUntranslatedDefinite() {
                self.placement_data_.row_start_offset =
                    (self.placement_data_.row_start_offset as i32)
                        .max(-position.rows.UntranslatedStartLine()) as u32;
            }
            self.placement_data_.grid_item_positions.push(position);
            it.Advance();
        }
        self.minor_max_end_line_ = self.IntrinsicEndLine(self.minor_direction_);
        let subgrid = !self
            .placement_data_
            .HasStandaloneAxis(self.minor_direction_);
        placed.needs_to_sort_item_vector = false;
        placed.item_vector.reserve(items.Size() as usize);
        for position in &mut self.placement_data_.grid_item_positions {
            let mut major = *position.Span(self.major_direction_);
            let mut minor = *position.Span(self.minor_direction_);
            let indefinite_major = major.IsIndefinite();
            let indefinite_minor = minor.IsIndefinite();
            if !indefinite_major {
                major.Translate(if self.major_direction_ == kForColumns {
                    self.placement_data_.column_start_offset
                } else {
                    self.placement_data_.row_start_offset
                });
                position.SetSpan(&major, self.major_direction_);
            }
            if !indefinite_minor {
                minor.Translate(if self.minor_direction_ == kForColumns {
                    self.placement_data_.column_start_offset
                } else {
                    self.placement_data_.row_start_offset
                });
                position.SetSpan(&minor, self.minor_direction_);
            }
            self.minor_max_end_line_ = self.minor_max_end_line_.max(if indefinite_minor {
                minor.IndefiniteSpanSize()
            } else {
                minor.EndLine()
            });
            if !indefinite_major && !indefinite_minor {
                let item = Box::new(PlacedGridItem::new(
                    position,
                    self.major_direction_,
                    self.minor_direction_,
                ));
                placed.needs_to_sort_item_vector |= placed
                    .item_vector
                    .last()
                    .is_some_and(|last| item.start_ < last.start_);
                placed.item_vector.push(item);
            } else if indefinite_major {
                unlocked.push(position);
            } else {
                locked.push(position);
            }
        }
        if subgrid {
            self.ClampMinorMaxToSubgridArea();
        }
        !unlocked.is_empty() || !locked.is_empty()
    }
    // cpp: layoutng_grid/grid_placement.cc:226-287
    fn PlaceGridItemsLockedToMajorAxis(
        &mut self,
        positions: &[*mut GridArea],
        placed: &mut PlacedGridItemsList,
    ) {
        let sparse = self.HasSparsePacking();
        let subgrid = !self
            .placement_data_
            .HasStandaloneAxis(self.minor_direction_);
        let mut minor_cursors: HashMap<u32, u32, IntWithZeroKeyHashTraits<u32>> =
            HashMap::default();
        for &position in positions {
            let position = unsafe { &mut *position };
            debug_assert!(
                AutoPlacement(position, self.major_direction_) == AutoPlacementType::kMinor
            );
            let minor_size = position
                .Span(self.minor_direction_)
                .IndefiniteSpanSize()
                .min(self.minor_max_end_line_);
            let start = position.StartLine(self.major_direction_);
            let mut cursor = AutoPlacementCursor::new(placed.ordered_list.Head());
            cursor.MoveToMajorLine(start);
            if sparse {
                if let Some(&line) = minor_cursors.get(&start) {
                    cursor.MoveToMinorLine(line);
                }
            }
            cursor.MoveCursorToFitGridSpan(
                position.SpanSize(self.major_direction_),
                minor_size,
                self.minor_max_end_line_,
                CursorMovementBehavior::kForceMajorLine,
            );
            let end = cursor.current_position_.minor_line + minor_size;
            if sparse {
                minor_cursors.Set(start, end);
            }
            self.minor_max_end_line_ = self.minor_max_end_line_.max(end);
            if subgrid {
                self.ClampMinorMaxToSubgridArea();
            }
            position.SetSpan(
                &GridSpan::TranslatedDefiniteGridSpan(cursor.current_position_.minor_line, end),
                self.minor_direction_,
            );
            self.PlaceGridItemAtCursor(position, placed, &mut cursor);
        }
    }
    // cpp: layoutng_grid/grid_placement.cc:289-310
    fn PlaceAutoMajorAxisGridItem(
        &self,
        position: &mut GridArea,
        placed: &mut PlacedGridItemsList,
        cursor: &mut AutoPlacementCursor,
    ) {
        let size = position.Span(self.major_direction_).IndefiniteSpanSize();
        cursor.MoveToMinorLine(position.StartLine(self.minor_direction_));
        cursor.MoveCursorToFitGridSpan(
            size,
            position.SpanSize(self.minor_direction_),
            self.minor_max_end_line_,
            CursorMovementBehavior::kForceMinorLine,
        );
        position.SetSpan(
            &GridSpan::TranslatedDefiniteGridSpan(
                cursor.current_position_.major_line,
                cursor.current_position_.major_line + size,
            ),
            self.major_direction_,
        );
        self.PlaceGridItemAtCursor(position, placed, cursor);
    }
    // cpp: layoutng_grid/grid_placement.cc:312-341
    fn PlaceAutoBothAxisGridItem(
        &self,
        position: &mut GridArea,
        placed: &mut PlacedGridItemsList,
        cursor: &mut AutoPlacementCursor,
    ) {
        let major = position.Span(self.major_direction_).IndefiniteSpanSize();
        let minor = position
            .Span(self.minor_direction_)
            .IndefiniteSpanSize()
            .min(self.minor_max_end_line_);
        cursor.MoveCursorToFitGridSpan(
            major,
            minor,
            self.minor_max_end_line_,
            CursorMovementBehavior::kAuto,
        );
        position.SetSpan(
            &GridSpan::TranslatedDefiniteGridSpan(
                cursor.current_position_.major_line,
                cursor.current_position_.major_line + major,
            ),
            self.major_direction_,
        );
        position.SetSpan(
            &GridSpan::TranslatedDefiniteGridSpan(
                cursor.current_position_.minor_line,
                cursor.current_position_.minor_line + minor,
            ),
            self.minor_direction_,
        );
        self.PlaceGridItemAtCursor(position, placed, cursor);
    }
    // cpp: layoutng_grid/grid_placement.cc:343-359
    fn PlaceGridItemAtCursor(
        &self,
        position: &GridArea,
        placed: &mut PlacedGridItemsList,
        cursor: &mut AutoPlacementCursor,
    ) {
        let mut item = Box::new(PlacedGridItem::new(
            position,
            self.major_direction_,
            self.minor_direction_,
        ));
        let pointer = &mut *item as *mut _;
        let next = cursor.next_placed_item_;
        let point = if next.is_null() {
            placed.ordered_list.Tail()
        } else {
            unsafe { &*next }.Prev()
        };
        unsafe {
            placed.ordered_list.InsertAfter(pointer, point);
        }
        cursor.InsertPlacedItemAtCurrentPosition(pointer);
        placed.item_vector.push(item);
    }
    // cpp: layoutng_grid/grid_placement.cc:361-402
    fn ClampGridItemsToFitSubgridArea(&mut self, direction: GridTrackSizingDirection) {
        let size = self.placement_data_.SubgridSpanSize(direction);
        if size == u32::MAX {
            return;
        }
        debug_assert!(size > 0);
        let offset = self.placement_data_.StartOffset(direction) as i32;
        for position in &mut self.placement_data_.grid_item_positions {
            if !position.Span(direction).IsTranslatedDefinite() {
                continue;
            }
            let start = position.StartLine(direction).wrapping_sub(offset as u32) as i32;
            let end = position.EndLine(direction).wrapping_sub(offset as u32) as i32;
            position.SetSpan(
                &GridSpan::TranslatedDefiniteGridSpan(
                    start.clamp(0, (size - 1) as i32) as u32,
                    end.clamp(1, size as i32) as u32,
                ),
                direction,
            );
        }
        if direction == kForColumns {
            self.placement_data_.column_start_offset = 0;
        } else {
            self.placement_data_.row_start_offset = 0;
        }
    }
    // cpp: layoutng_grid/grid_placement.cc:404-414
    fn ClampMinorMaxToSubgridArea(&mut self) {
        debug_assert!(!self
            .placement_data_
            .HasStandaloneAxis(self.minor_direction_));
        let maximum = self.IntrinsicEndLine(self.minor_direction_);
        debug_assert!(self.minor_max_end_line_ >= maximum);
        if self.minor_max_end_line_ > maximum {
            self.minor_max_end_line_ = maximum;
        }
    }
    // cpp: layoutng_grid/grid_placement.cc:416-427
    fn HasSparsePacking(&self) -> bool {
        self.packing_behavior_ == PackingBehavior::kSparse
    }
    fn IntrinsicEndLine(&self, direction: GridTrackSizingDirection) -> u32 {
        self.placement_data_.StartOffset(direction)
            + self.placement_data_.ExplicitGridTrackCount(direction)
    }
    // cpp: layoutng_grid/grid_placement.cc:680-706
    pub fn ResolveOutOfFlowItemGridLines(
        tracks: &GridLayoutTrackCollection,
        resolver: &GridLineResolver,
        _grid_style: &ComputedStyle,
        style: &ComputedStyle,
        offset: u32,
        start: &mut u32,
        end: &mut u32,
    ) {
        *start = u32::MAX;
        *end = u32::MAX;
        let direction = tracks.Direction();
        let columns = direction == kForColumns;
        let span = resolver.ResolveGridPositionsFromStyle(style, direction);
        if span.IsIndefinite() {
            return;
        }
        let span_start = (span.UntranslatedStartLine() as u32).wrapping_add(offset) as i32;
        let span_end = (span.UntranslatedEndLine() as u32).wrapping_add(offset) as i32;
        if span_start >= 0
            && !(if columns {
                style.GridColumnStart()
            } else {
                style.GridRowStart()
            })
            .IsAuto()
            && tracks.IsGridLineWithinImplicitGrid(span_start as u32)
        {
            *start = span_start as u32;
        }
        if span_end >= 0
            && !(if columns {
                style.GridColumnEnd()
            } else {
                style.GridRowEnd()
            })
            .IsAuto()
            && tracks.IsGridLineWithinImplicitGrid(span_end as u32)
        {
            *end = span_end as u32;
        }
    }
}
