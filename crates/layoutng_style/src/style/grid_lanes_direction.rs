// cpp: layoutng_style/style/grid_lanes_direction.h:12-20
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum GridLanesOrientation {
    #[default]
    kNormal,
    kRow,
    kColumn,
}

// cpp: layoutng_style/style/grid_lanes_direction.h:22-30
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GridLanesDirection {
    pub orientation: GridLanesOrientation,
    pub is_fill_reverse: bool,
    pub is_track_reverse: bool,
}

impl GridLanesDirection {
    pub fn new(
        orientation: GridLanesOrientation,
        is_fill_reverse: bool,
        is_track_reverse: bool,
    ) -> Self {
        let result = Self {
            orientation,
            is_fill_reverse,
            is_track_reverse,
        };
        // cpp: layoutng_style/style/grid_lanes_direction.h:31-36
        if orientation == GridLanesOrientation::kNormal {
            assert!(!is_fill_reverse);
            assert!(!is_track_reverse);
        }
        result
    }
}

// cpp: layoutng_style/style/grid_lanes_direction.h:38-44
// Both equality and inequality are provided by the derived PartialEq.
