// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/page/grid_focusgroup_structure_info.h:36-145
// Source: third_party/blink/renderer/core/page/grid_focusgroup_structure_info.cc:20-486
use super::focus_navigation::FocusNavigationNode as Node;
use foundation::table_focusgroup::TableFocusgroupFacts as Facts;

#[derive(Clone, Copy)]
pub enum NoCellFoundAtIndexBehavior {
    kReturn,
    kFindPreviousCellInRow,
    kFindNextCellInRow,
    kFindPreviousCellInColumn,
    kFindNextCellInColumn,
}
pub trait GridLayout {
    type Object: Copy + Eq;
    fn Layout(&self, node: Node) -> Option<Self::Object>;
    fn Node(&self, object: Self::Object) -> Option<Node>;
    fn Facts(&self, object: Self::Object) -> Option<Facts<Self::Object>>;
    fn Behavior(&self, node: Node) -> foundation::focusgroup_flags::FocusgroupBehavior;
    /// HTMLTableRowElement::cells()->item(index), only for actual HTML rows.
    fn HTMLRowCell(&self, row: Self::Object, index: u32) -> Option<Self::Object>;
}
#[derive(Clone, Copy)]
pub enum GridQuery {
    ColumnCount,
    PreviousCellInRow(Node),
    NextCellInRow(Node),
    FirstCellInRow(Node),
    LastCellInRow(Node),
    ColumnIndexForCell(Node),
    PreviousCellInColumn(Node),
    NextCellInColumn(Node),
    FirstCellInColumn(u32),
    LastCellInColumn(u32),
    PreviousRow(Node),
    NextRow(Node),
    FirstRow,
    LastRow,
    RowForCell(Node),
}
pub enum GridResult {
    Node(Option<Node>),
    Index(u32),
}

pub struct AutomaticGridFocusgroupStructureInfo<'a, L: GridLayout> {
    layout: &'a L,
    table: L::Object,
}
impl<'a, L: GridLayout> AutomaticGridFocusgroupStructureInfo<'a, L> {
    pub fn new(layout: &'a L, root: Node) -> Option<Self> {
        let table = layout.Layout(root)?;
        if !matches!(layout.Facts(table), Some(Facts::Table { .. })) {
            return None;
        }
        debug_assert_eq!(
            layout.Behavior(root),
            foundation::focusgroup_flags::FocusgroupBehavior::kGrid
        );
        Some(Self { layout, table })
    }
    pub fn Root(&self) -> Option<Node> {
        self.layout.Node(self.table)
    }
    pub fn Behavior(&self) -> foundation::focusgroup_flags::FocusgroupBehavior {
        self.layout
            .Behavior(self.Root().expect("grid root Element"))
    }
    pub fn ColumnCount(&self) -> u32 {
        match self.layout.Facts(self.table) {
            Some(Facts::Table { columns, .. }) => columns,
            _ => unreachable!(),
        }
    }
    fn Cell(&self, node: Node) -> Option<(L::Object, L::Object, u32, u32, u32)> {
        let object = self.layout.Layout(node)?;
        let Facts::Cell {
            row,
            column,
            col_span,
            row_span,
            ..
        } = self.layout.Facts(object)?
        else {
            return None;
        };
        Some((object, row, column, col_span, row_span))
    }
    pub fn PreviousCellInRow(&self, node: Node) -> Option<Node> {
        let (_, row, column, _, _) = self.Cell(node)?;
        self.CellAtIndexInRow(
            column.wrapping_sub(1),
            self.layout.Node(row)?,
            NoCellFoundAtIndexBehavior::kFindPreviousCellInRow,
        )
    }
    pub fn NextCellInRow(&self, node: Node) -> Option<Node> {
        let (_, row, column, span, _) = self.Cell(node)?;
        if span == 0 {
            return None;
        }
        self.CellAtIndexInRow(
            column.wrapping_add(span),
            self.layout.Node(row)?,
            NoCellFoundAtIndexBehavior::kFindNextCellInRow,
        )
    }
    pub fn FirstCellInRow(&self, row: Node) -> Option<Node> {
        self.CellAtIndexInRow(0, row, NoCellFoundAtIndexBehavior::kFindNextCellInRow)
    }
    pub fn LastCellInRow(&self, row: Node) -> Option<Node> {
        self.CellAtIndexInRow(
            self.ColumnCount().wrapping_sub(1),
            row,
            NoCellFoundAtIndexBehavior::kFindPreviousCellInRow,
        )
    }
    pub fn ColumnIndexForCell(&self, cell: Node) -> u32 {
        self.Cell(cell).map_or(0, |(_, _, column, _, _)| column)
    }
    pub fn PreviousCellInColumn(&self, node: Node) -> Option<Node> {
        let (_, row, column, _, _) = self.Cell(node)?;
        self.CellAtIndexInRow(
            column,
            self.layout.Node(self.PreviousLayoutRow(row)?)?,
            NoCellFoundAtIndexBehavior::kFindPreviousCellInColumn,
        )
    }
    pub fn NextCellInColumn(&self, node: Node) -> Option<Node> {
        let (_, mut row, column, _, span) = self.Cell(node)?;
        for _ in 0..span {
            row = self.NextLayoutRow(row)?;
        }
        self.CellAtIndexInRow(
            column,
            self.layout.Node(row)?,
            NoCellFoundAtIndexBehavior::kFindNextCellInColumn,
        )
    }
    pub fn FirstCellInColumn(&self, column: u32) -> Option<Node> {
        if column >= self.ColumnCount() {
            return None;
        }
        self.CellAtIndexInRow(
            column,
            self.FirstRow()?,
            NoCellFoundAtIndexBehavior::kFindNextCellInColumn,
        )
    }
    pub fn LastCellInColumn(&self, column: u32) -> Option<Node> {
        if column >= self.ColumnCount() {
            return None;
        }
        self.CellAtIndexInRow(
            column,
            self.LastRow()?,
            NoCellFoundAtIndexBehavior::kFindPreviousCellInColumn,
        )
    }
    pub fn PreviousRow(&self, row: Node) -> Option<Node> {
        self.layout
            .Node(self.PreviousLayoutRow(self.layout.Layout(row)?)?)
    }
    pub fn NextRow(&self, row: Node) -> Option<Node> {
        self.layout
            .Node(self.NextLayoutRow(self.layout.Layout(row)?)?)
    }
    pub fn FirstRow(&self) -> Option<Node> {
        let Facts::Table { first_section, .. } = self.layout.Facts(self.table)? else {
            return None;
        };
        let Facts::Section { first_row, .. } = self.layout.Facts(first_section?)? else {
            return None;
        };
        let mut row = first_row;
        while let Some(current) = row {
            let Facts::Row {
                first_cell, next, ..
            } = self.layout.Facts(current)?
            else {
                return None;
            };
            if first_cell.is_some() {
                return self.layout.Node(current);
            }
            row = next;
        }
        None
    }
    pub fn LastRow(&self) -> Option<Node> {
        let Facts::Table { last_section, .. } = self.layout.Facts(self.table)? else {
            return None;
        };
        let Facts::Section { last_row, .. } = self.layout.Facts(last_section?)? else {
            return None;
        };
        let mut row = last_row;
        while let Some(current) = row {
            let Facts::Row {
                first_cell,
                previous,
                ..
            } = self.layout.Facts(current)?
            else {
                return None;
            };
            if first_cell.is_some() {
                return self.layout.Node(current);
            }
            row = previous;
        }
        None
    }
    pub fn RowForCell(&self, cell: Node) -> Option<Node> {
        self.layout.Node(self.Cell(cell)?.1)
    }
    pub fn CellAtIndexInRow(
        &self,
        mut index: u32,
        row: Node,
        behavior: NoCellFoundAtIndexBehavior,
    ) -> Option<Node> {
        let mut row = self.layout.Layout(row)?;
        let Facts::Row { first_cell, .. } = self.layout.Facts(row)? else {
            return None;
        };
        first_cell?;
        let columns = self.ColumnCount();
        if index >= columns {
            return None;
        }
        let mut cell = self.TableCellAtIndexInRowRecursive(index, row, None);
        while cell.is_none() {
            match behavior {
                NoCellFoundAtIndexBehavior::kReturn => return None,
                NoCellFoundAtIndexBehavior::kFindPreviousCellInRow => {
                    assert_ne!(index, 0);
                    index -= 1;
                }
                NoCellFoundAtIndexBehavior::kFindNextCellInRow => {
                    if index >= columns {
                        return None;
                    }
                    index += 1;
                }
                NoCellFoundAtIndexBehavior::kFindPreviousCellInColumn => {
                    row = self.PreviousLayoutRow(row)?
                }
                NoCellFoundAtIndexBehavior::kFindNextCellInColumn => {
                    row = self.NextLayoutRow(row)?
                }
            }
            cell = self.TableCellAtIndexInRowRecursive(index, row, None);
        }
        self.layout.Node(cell?)
    }
    // Source: grid_focusgroup_structure_info.cc:328-382. These traverse the
    // original native sections and rows, including anonymous table owners.
    fn PreviousLayoutRow(&self, row: L::Object) -> Option<L::Object> {
        let Facts::Row {
            mut section,
            previous,
            ..
        } = self.layout.Facts(row)?
        else {
            return None;
        };
        let mut previous = previous;
        loop {
            if let Some(candidate) = previous {
                let Facts::Row {
                    first_cell,
                    previous: prior,
                    ..
                } = self.layout.Facts(candidate)?
                else {
                    return None;
                };
                if first_cell.is_some() {
                    return Some(candidate);
                }
                // Preserve the upstream continuation-row/section branch.
                let _ = prior;
            }
            let Facts::Section {
                previous: prior_section,
                ..
            } = self.layout.Facts(section)?
            else {
                return None;
            };
            section = prior_section?;
            let Facts::Section { last_row, .. } = self.layout.Facts(section)? else {
                return None;
            };
            previous = last_row;
        }
    }
    fn NextLayoutRow(&self, row: L::Object) -> Option<L::Object> {
        let Facts::Row {
            mut section, next, ..
        } = self.layout.Facts(row)?
        else {
            return None;
        };
        let mut next = next;
        loop {
            if let Some(candidate) = next {
                let Facts::Row { first_cell, .. } = self.layout.Facts(candidate)? else {
                    return None;
                };
                if first_cell.is_some() {
                    return Some(candidate);
                }
            }
            let Facts::Section {
                next: next_section, ..
            } = self.layout.Facts(section)?
            else {
                return None;
            };
            section = next_section?;
            let Facts::Section { first_row, .. } = self.layout.Facts(section)? else {
                return None;
            };
            next = first_row;
        }
    }
    fn TableCellAtIndexInRowRecursive(
        &self,
        index: u32,
        row: L::Object,
        expected_rowspan: Option<u32>,
    ) -> Option<L::Object> {
        let Facts::Row { last_cell, .. } = self.layout.Facts(row)? else {
            return None;
        };
        let mut cell = self.layout.HTMLRowCell(row, index).or(last_cell)?;
        let Facts::Cell {
            column: mut actual_index,
            ..
        } = self.layout.Facts(cell)?
        else {
            return None;
        };
        while actual_index != index {
            let Facts::Cell {
                previous, col_span, ..
            } = self.layout.Facts(cell)?
            else {
                return None;
            };
            if actual_index > index {
                if let Some(previous) = previous {
                    cell = previous;
                    let Facts::Cell { column, .. } = self.layout.Facts(cell)? else {
                        return None;
                    };
                    actual_index = column;
                    continue;
                }
            } else if col_span == 0 || actual_index.wrapping_add(col_span) > index {
                break;
            }
            cell = self.TableCellAtIndexInRowRecursive(
                index,
                self.PreviousLayoutRow(row)?,
                Some(expected_rowspan.map_or(2, |span| span + 1)),
            )?;
            let Facts::Cell { column, .. } = self.layout.Facts(cell)? else {
                return None;
            };
            actual_index = column;
            break;
        }
        if actual_index == index {
            if let Some(expected) = expected_rowspan {
                let Facts::Cell { row_span, .. } = self.layout.Facts(cell)? else {
                    return None;
                };
                if row_span == 0 || expected > row_span {
                    return None;
                }
            }
        }
        Some(cell)
    }
    pub fn Query(&self, query: GridQuery) -> GridResult {
        use GridQuery::*;
        GridResult::Node(match query {
            ColumnCount => return GridResult::Index(self.ColumnCount()),
            ColumnIndexForCell(node) => return GridResult::Index(self.ColumnIndexForCell(node)),
            PreviousCellInRow(node) => self.PreviousCellInRow(node),
            NextCellInRow(node) => self.NextCellInRow(node),
            FirstCellInRow(node) => self.FirstCellInRow(node),
            LastCellInRow(node) => self.LastCellInRow(node),
            PreviousCellInColumn(node) => self.PreviousCellInColumn(node),
            NextCellInColumn(node) => self.NextCellInColumn(node),
            FirstCellInColumn(index) => self.FirstCellInColumn(index),
            LastCellInColumn(index) => self.LastCellInColumn(index),
            PreviousRow(row) => self.PreviousRow(row),
            NextRow(row) => self.NextRow(row),
            FirstRow => self.FirstRow(),
            LastRow => self.LastRow(),
            RowForCell(cell) => self.RowForCell(cell),
        })
    }
}
