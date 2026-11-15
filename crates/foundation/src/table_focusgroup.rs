//! Borrowed table facts used by Chromium's GridFocusgroupStructureInfo.
//! Each object identity names the same resident native owner; no table model
//! or DOM geometry is materialized at this boundary.
#[derive(Clone, Copy)]
pub enum TableFocusgroupFacts<O: Copy> {
    Table {
        columns: u32,
        first_section: Option<O>,
        last_section: Option<O>,
    },
    Section {
        previous: Option<O>,
        next: Option<O>,
        first_row: Option<O>,
        last_row: Option<O>,
    },
    Row {
        section: O,
        previous: Option<O>,
        next: Option<O>,
        first_cell: Option<O>,
        last_cell: Option<O>,
    },
    Cell {
        row: O,
        previous: Option<O>,
        column: u32,
        col_span: u32,
        row_span: u32,
    },
}
impl<O: Copy> TableFocusgroupFacts<O> {
    pub fn Map<P: Copy>(self, mut f: impl FnMut(O) -> P) -> TableFocusgroupFacts<P> {
        match self {
            Self::Table {
                columns,
                first_section,
                last_section,
            } => TableFocusgroupFacts::Table {
                columns,
                first_section: first_section.map(&mut f),
                last_section: last_section.map(&mut f),
            },
            Self::Section {
                previous,
                next,
                first_row,
                last_row,
            } => TableFocusgroupFacts::Section {
                previous: previous.map(&mut f),
                next: next.map(&mut f),
                first_row: first_row.map(&mut f),
                last_row: last_row.map(&mut f),
            },
            Self::Row {
                section,
                previous,
                next,
                first_cell,
                last_cell,
            } => TableFocusgroupFacts::Row {
                section: f(section),
                previous: previous.map(&mut f),
                next: next.map(&mut f),
                first_cell: first_cell.map(&mut f),
                last_cell: last_cell.map(&mut f),
            },
            Self::Cell {
                row,
                previous,
                column,
                col_span,
                row_span,
            } => TableFocusgroupFacts::Cell {
                row: f(row),
                previous: previous.map(&mut f),
                column,
                col_span,
                row_span,
            },
        }
    }
}
