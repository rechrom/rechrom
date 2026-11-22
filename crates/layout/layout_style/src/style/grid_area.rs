use foundation::{HashInts, HashMap, String};

use super::grid_enums::GridTrackSizingDirection;

// cpp: layoutng_style/style/grid_area.h:45
pub const K_GRID_MAX_TRACKS: i32 = 10_000_000;

// cpp: layoutng_style/style/grid_area.h:181-182
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GridSpanType {
    Indefinite,
    TranslatedDefinite,
    UntranslatedDefinite,
}

// cpp: layoutng_style/style/grid_area.h:47-202
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridSpan {
    start_line_: i32,
    end_line_: i32,
    type_: GridSpanType,
}

#[allow(non_snake_case)]
impl GridSpan {
    // cpp: layoutng_style/style/grid_area.h:54-56
    pub fn UntranslatedDefiniteGridSpan(start_line: i32, end_line: i32) -> Self {
        Self::new(start_line, end_line, GridSpanType::UntranslatedDefinite)
    }

    // cpp: layoutng_style/style/grid_area.h:58-61
    pub fn TranslatedDefiniteGridSpan(start_line: u32, end_line: u32) -> Self {
        Self::new(
            start_line as i32,
            end_line as i32,
            GridSpanType::TranslatedDefinite,
        )
    }

    // cpp: layoutng_style/style/grid_area.h:63-65
    pub fn IndefiniteGridSpan(span_size: u32) -> Self {
        Self::new(0, span_size as i32, GridSpanType::Indefinite)
    }
    pub fn IndefiniteGridSpanDefault() -> Self {
        Self::IndefiniteGridSpan(1)
    }

    // cpp: layoutng_style/style/grid_area.h:72-76
    pub fn less_than(&self, other: &Self) -> bool {
        debug_assert!(self.IsTranslatedDefinite());
        self.start_line_ < other.start_line_
            || (self.start_line_ == other.start_line_ && self.end_line_ < other.end_line_)
    }

    // cpp: layoutng_style/style/grid_area.h:78-81
    pub fn less_equal(&self, other: &Self) -> bool {
        debug_assert!(self.IsTranslatedDefinite());
        self.less_than(other) || self == other
    }

    // cpp: layoutng_style/style/grid_area.h:83-88
    pub fn increment(&mut self) -> &mut Self {
        debug_assert!(self.IsTranslatedDefinite());
        self.start_line_ = self.start_line_.wrapping_add(1);
        self.end_line_ = self.end_line_.wrapping_add(1);
        self
    }

    // cpp: layoutng_style/style/grid_area.h:90-96
    pub fn Contains(&self, line: u32) -> bool {
        debug_assert!(self.IsTranslatedDefinite());
        debug_assert!(self.start_line_ >= 0);
        debug_assert!(self.start_line_ < self.end_line_);
        line >= self.start_line_ as u32 && line <= self.end_line_ as u32
    }

    // cpp: layoutng_style/style/grid_area.h:98-103
    pub fn GetHash(&self) -> u32 {
        HashInts(
            self.start_line_ as u32,
            (if self.IsIndefinite() {
                self.end_line_.wrapping_neg()
            } else {
                self.end_line_
            }) as u32,
        )
    }

    // cpp: layoutng_style/style/grid_area.h:105-114
    pub fn Intersects(&self, span: Self) -> bool {
        debug_assert!(self.IsTranslatedDefinite());
        debug_assert!(span.IsTranslatedDefinite());
        debug_assert!(self.start_line_ >= 0);
        debug_assert!(self.start_line_ < self.end_line_);
        debug_assert!(span.start_line_ >= 0);
        debug_assert!(span.start_line_ < span.end_line_);
        self.start_line_ < span.end_line_ && self.end_line_ >= span.start_line_
    }

    // cpp: layoutng_style/style/grid_area.h:116-119
    pub fn IntegerSpan(&self) -> u32 {
        debug_assert!(self.IsTranslatedDefinite());
        self.SpanSize()
    }

    // cpp: layoutng_style/style/grid_area.h:121-126
    pub fn IndefiniteSpanSize(&self) -> u32 {
        debug_assert!(self.IsIndefinite());
        debug_assert_eq!(self.start_line_, 0);
        debug_assert!(self.end_line_ > 0);
        self.end_line_ as u32
    }

    // cpp: layoutng_style/style/grid_area.h:128-131
    pub fn SpanSize(&self) -> u32 {
        debug_assert!(self.start_line_ < self.end_line_);
        self.end_line_.wrapping_sub(self.start_line_) as u32
    }

    // cpp: layoutng_style/style/grid_area.h:133-141
    pub fn UntranslatedStartLine(&self) -> i32 {
        debug_assert_eq!(self.type_, GridSpanType::UntranslatedDefinite);
        self.start_line_
    }
    pub fn UntranslatedEndLine(&self) -> i32 {
        debug_assert_eq!(self.type_, GridSpanType::UntranslatedDefinite);
        self.end_line_
    }

    // cpp: layoutng_style/style/grid_area.h:143-153
    pub fn StartLine(&self) -> u32 {
        debug_assert!(self.IsTranslatedDefinite());
        debug_assert!(self.start_line_ >= 0);
        self.start_line_ as u32
    }
    pub fn EndLine(&self) -> u32 {
        debug_assert!(self.IsTranslatedDefinite());
        debug_assert!(self.end_line_ > 0);
        self.end_line_ as u32
    }

    // cpp: layoutng_style/style/grid_area.h:155-157
    pub fn IsUntranslatedDefinite(&self) -> bool {
        self.type_ == GridSpanType::UntranslatedDefinite
    }
    pub fn IsTranslatedDefinite(&self) -> bool {
        self.type_ == GridSpanType::TranslatedDefinite
    }
    pub fn IsIndefinite(&self) -> bool {
        self.type_ == GridSpanType::Indefinite
    }

    // cpp: layoutng_style/style/grid_area.h:159-163
    pub fn Translate(&mut self, offset: u32) {
        debug_assert!(!self.IsIndefinite());
        *self = Self::new(
            (self.start_line_ as u32).wrapping_add(offset) as i32,
            (self.end_line_ as u32).wrapping_add(offset) as i32,
            GridSpanType::TranslatedDefinite,
        );
    }

    // cpp: layoutng_style/style/grid_area.h:165-168
    pub fn SetStart(&mut self, start_line: i32) {
        debug_assert!(!self.IsIndefinite());
        *self = Self::new(start_line, self.end_line_, GridSpanType::TranslatedDefinite);
    }

    // cpp: layoutng_style/style/grid_area.h:170-173
    pub fn SetEnd(&mut self, end_line: i32) {
        debug_assert!(!self.IsIndefinite());
        *self = Self::new(self.start_line_, end_line, GridSpanType::TranslatedDefinite);
    }

    // cpp: layoutng_style/style/grid_area.h:175-179
    pub fn Intersect(&mut self, start_line: i32, end_line: i32) {
        debug_assert!(!self.IsIndefinite());
        *self = Self::new(
            self.start_line_.max(start_line),
            self.end_line_.min(end_line),
            GridSpanType::TranslatedDefinite,
        );
    }

    // cpp: layoutng_style/style/grid_area.h:184-201
    fn new(start_line: i32, end_line: i32, type_: GridSpanType) -> Self {
        let mut span = Self {
            start_line_: 0,
            end_line_: 0,
            type_,
        };
        if type_ == GridSpanType::Indefinite {
            debug_assert_eq!(start_line, 0);
            span.end_line_ = end_line.clamp(1, K_GRID_MAX_TRACKS);
        } else {
            span.start_line_ = start_line.clamp(-K_GRID_MAX_TRACKS, K_GRID_MAX_TRACKS - 1);
            span.end_line_ = end_line.clamp(span.start_line_ + 1, K_GRID_MAX_TRACKS);
            if type_ == GridSpanType::TranslatedDefinite {
                debug_assert!(span.start_line_ >= 0);
            }
        }
        span
    }
}

// cpp: layoutng_style/style/grid_area.h:204-257
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridArea {
    pub columns: GridSpan,
    pub rows: GridSpan,
}

#[allow(non_snake_case)]
impl GridArea {
    // cpp: layoutng_style/style/grid_area.h:214
    pub fn new(rows: &GridSpan, columns: &GridSpan) -> Self {
        Self {
            columns: *columns,
            rows: *rows,
        }
    }

    // cpp: layoutng_style/style/grid_area.h:216-218
    pub fn Span(&self, track_direction: GridTrackSizingDirection) -> &GridSpan {
        if track_direction == GridTrackSizingDirection::kForColumns {
            &self.columns
        } else {
            &self.rows
        }
    }

    // cpp: layoutng_style/style/grid_area.h:220-226
    pub fn SetSpan(&mut self, span: &GridSpan, track_direction: GridTrackSizingDirection) {
        if track_direction == GridTrackSizingDirection::kForColumns {
            self.columns = *span;
        } else {
            self.rows = *span;
        }
    }

    // cpp: layoutng_style/style/grid_area.h:228-235
    pub fn MaybeTranslateSpan(
        &mut self,
        start_offset: u32,
        track_direction: GridTrackSizingDirection,
    ) -> &GridSpan {
        let span = if track_direction == GridTrackSizingDirection::kForColumns {
            &mut self.columns
        } else {
            &mut self.rows
        };
        if span.IsUntranslatedDefinite() {
            span.Translate(start_offset);
        }
        span
    }

    // cpp: layoutng_style/style/grid_area.h:237-247
    pub fn StartLine(&self, track_direction: GridTrackSizingDirection) -> u32 {
        self.Span(track_direction).StartLine()
    }
    pub fn EndLine(&self, track_direction: GridTrackSizingDirection) -> u32 {
        self.Span(track_direction).EndLine()
    }
    pub fn SpanSize(&self, track_direction: GridTrackSizingDirection) -> u32 {
        self.Span(track_direction).IntegerSpan()
    }

    // cpp: layoutng_style/style/grid_area.h:249
    pub fn Transpose(&mut self) {
        std::mem::swap(&mut self.columns, &mut self.rows);
    }
}

// cpp: layoutng_style/style/grid_area.h:209-212
impl Default for GridArea {
    fn default() -> Self {
        Self {
            columns: GridSpan::IndefiniteGridSpanDefault(),
            rows: GridSpan::IndefiniteGridSpanDefault(),
        }
    }
}

// cpp: layoutng_style/style/grid_area.h:259
pub type NamedGridAreaMap = HashMap<String, GridArea>;
