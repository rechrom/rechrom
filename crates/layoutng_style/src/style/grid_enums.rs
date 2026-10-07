// cpp: layoutng_style/style/grid_enums.h:10-15
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum GridPositionSide {
    kColumnStartSide,
    kColumnEndSide,
    kRowStartSide,
    kRowEndSide,
}

// cpp: layoutng_style/style/grid_enums.h:17
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum GridTrackSizingDirection {
    kForColumns,
    kForRows,
}
