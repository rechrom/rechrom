use foundation::AtomicString;

// cpp: layoutng_style/style/grid_position.h:41-46
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum GridPositionType {
    kAutoPosition,
    kExplicitPosition,
    kSpanPosition,
    kNamedGridAreaPosition,
}

// cpp: layoutng_style/style/grid_position.h:48-116
#[derive(Clone, PartialEq, Eq)]
pub struct GridPosition {
    type_: GridPositionType,
    integer_position_: i32,
    named_grid_line_: AtomicString,
}

#[allow(non_snake_case)]
impl GridPosition {
    // cpp: layoutng_style/style/grid_position.h:54
    pub fn IsPositive(&self) -> bool {
        self.IntegerPosition() > 0
    }

    // cpp: layoutng_style/style/grid_position.h:56-59
    pub fn GetType(&self) -> GridPositionType {
        self.type_
    }
    pub fn IsAuto(&self) -> bool {
        self.type_ == GridPositionType::kAutoPosition
    }
    pub fn IsSpan(&self) -> bool {
        self.type_ == GridPositionType::kSpanPosition
    }
    pub fn IsNamedGridArea(&self) -> bool {
        self.type_ == GridPositionType::kNamedGridAreaPosition
    }

    // cpp: layoutng_style/style/grid_position.h:61-65
    pub fn SetExplicitPosition(&mut self, position: i32, named_grid_line: &AtomicString) {
        self.type_ = GridPositionType::kExplicitPosition;
        self.integer_position_ = position;
        self.named_grid_line_ = named_grid_line.clone();
    }

    // cpp: layoutng_style/style/grid_position.h:67-70
    pub fn SetAutoPosition(&mut self) {
        self.type_ = GridPositionType::kAutoPosition;
        self.integer_position_ = 0;
    }

    // cpp: layoutng_style/style/grid_position.h:72-79
    pub fn SetSpanPosition(&mut self, position: i32, named_grid_line: &AtomicString) {
        self.type_ = GridPositionType::kSpanPosition;
        self.integer_position_ = position;
        self.named_grid_line_ = named_grid_line.clone();
    }

    // cpp: layoutng_style/style/grid_position.h:81-84
    pub fn SetNamedGridArea(&mut self, named_grid_area: &AtomicString) {
        self.type_ = GridPositionType::kNamedGridAreaPosition;
        self.named_grid_line_ = named_grid_area.clone();
    }

    // cpp: layoutng_style/style/grid_position.h:86-89
    pub fn IntegerPosition(&self) -> i32 {
        debug_assert_eq!(self.GetType(), GridPositionType::kExplicitPosition);
        self.integer_position_
    }

    // cpp: layoutng_style/style/grid_position.h:91-95
    pub fn NamedGridLine(&self) -> AtomicString {
        debug_assert!(
            self.GetType() == GridPositionType::kExplicitPosition
                || self.GetType() == GridPositionType::kSpanPosition
                || self.GetType() == GridPositionType::kNamedGridAreaPosition
        );
        self.named_grid_line_.clone()
    }

    // cpp: layoutng_style/style/grid_position.h:97-100
    pub fn SpanPosition(&self) -> i32 {
        debug_assert_eq!(self.GetType(), GridPositionType::kSpanPosition);
        self.integer_position_
    }

    // cpp: layoutng_style/style/grid_position.h:108-110
    pub fn ShouldBeResolvedAgainstOppositePosition(&self) -> bool {
        self.IsAuto() || self.IsSpan()
    }
}

// cpp: layoutng_style/style/grid_position.h:52
impl Default for GridPosition {
    fn default() -> Self {
        Self {
            type_: GridPositionType::kAutoPosition,
            integer_position_: 0,
            named_grid_line_: AtomicString::default(),
        }
    }
}
