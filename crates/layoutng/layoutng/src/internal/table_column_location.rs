use foundation::LayoutUnit;

// cpp: layoutng/internal/table_column_location.h:12-17
#[derive(Clone, Copy, PartialEq)]
pub struct TableColumnLocation {
    pub offset: LayoutUnit,
    pub size: LayoutUnit,
    pub is_collapsed: bool,
}
