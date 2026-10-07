use crate::{native_test_thread, table_size_distribution::*};
use foundation::LayoutUnit;
use layoutng_assembly::internal::table_layout_algorithm_types::{
    CellBlockConstraint, CellInlineConstraint, ColspanCell, Column, Columns, Row, RowspanCell,
    Section,
};
use layoutng_geometry::geometry::{box_strut::BoxStrut, logical_size::LogicalSize};

struct Fields<'a> {
    fields: Vec<&'a str>,
    at: usize,
}
impl<'a> Fields<'a> {
    fn next(&mut self) -> &'a str {
        let s = self.fields[self.at];
        self.at += 1;
        s
    }
    fn integer(&mut self) -> i32 {
        self.next().parse().unwrap()
    }
    fn unit(&mut self) -> LayoutUnit {
        LayoutUnit::FromRawValue(self.integer())
    }
    fn optional(&mut self) -> Option<LayoutUnit> {
        let s = self.next();
        if s == "none" {
            None
        } else {
            Some(LayoutUnit::FromRawValue(s.parse().unwrap()))
        }
    }
    fn percent(&mut self) -> Option<f32> {
        let s = self.next();
        if s == "none" {
            None
        } else {
            Some(s.parse().unwrap())
        }
    }
    fn flag(&mut self) -> bool {
        self.integer() != 0
    }
}
#[test]
fn native_table_size_distribution_matches_unchanged_cpp() {
    native_test_thread::run(distribution_body);
}
fn distribution_body() {
    let inputs = include_str!("../../../artifacts/cpp-reference/table-distribution-input.tsv");
    let expected = include_str!("../../../artifacts/cpp-reference/table-distribution-results.tsv");
    assert_eq!(inputs.lines().count(), 32000);
    assert_eq!(expected.lines().count(), 32000);
    for (index, (input, result)) in inputs.lines().zip(expected.lines()).enumerate() {
        let mut f = Fields {
            fields: input.split('\t').collect(),
            at: 0,
        };
        let mode = f.next();
        let mut values: Vec<Option<f64>> = vec![Some(index as f64)];
        if mode == "columns" || mode == "colspan" || mode == "grid" || mode == "merge" {
            let fixed = f.flag();
            let size = f.unit();
            let spacing = f.unit();
            let n = f.integer();
            let mut columns = Columns { data: Vec::new() };
            for _ in 0..n {
                let lo = f.optional();
                let hi = f.optional();
                let p = f.percent();
                let bp = f.unit();
                let constrained = f.flag();
                let collapsed = f.flag();
                let mergeable = f.flag();
                columns.data.push(Column::new(
                    lo,
                    hi,
                    p,
                    bp,
                    constrained,
                    collapsed,
                    fixed,
                    mergeable,
                ));
            }
            if mode == "colspan" || mode == "merge" {
                if mode == "merge" {
                    let count = f.integer();
                    let mut cells = Vec::new();
                    for _ in 0..count {
                        cells.push(if f.flag() {
                            Some(CellInlineConstraint {
                                min_inline_size: f.unit(),
                                max_inline_size: f.unit(),
                                percent: f.percent(),
                                percent_border_padding: f.unit(),
                                is_constrained: f.flag(),
                            })
                        } else {
                            None
                        });
                    }
                    let count = f.integer();
                    let mut spans = Vec::new();
                    for _ in 0..count {
                        let start = f.integer() as u32;
                        let span = f.integer() as u32;
                        let constraint = CellInlineConstraint {
                            min_inline_size: f.unit(),
                            max_inline_size: f.unit(),
                            percent: f.percent(),
                            percent_border_padding: f.unit(),
                            is_constrained: f.flag(),
                        };
                        spans.push(ColspanCell::new(&constraint, start, span));
                    }
                    ApplyCellConstraintsToColumnConstraints(
                        &cells,
                        spacing,
                        fixed,
                        &mut spans,
                        &mut columns,
                    );
                } else {
                    let start = f.integer() as u32;
                    let span = f.integer() as u32;
                    let constraint = CellInlineConstraint {
                        min_inline_size: f.unit(),
                        max_inline_size: f.unit(),
                        percent: f.percent(),
                        percent_border_padding: f.unit(),
                        is_constrained: f.flag(),
                    };
                    DistributeColspanCellsToColumns(
                        &vec![ColspanCell::new(&constraint, start, span)],
                        spacing,
                        fixed,
                        &mut columns,
                    );
                }
                for c in columns.data {
                    values.extend([
                        c.min_inline_size.map(|v| v.RawValue() as f64),
                        c.max_inline_size.map(|v| v.RawValue() as f64),
                        c.percent.map(|v| v as f64),
                        Some(c.percent_border_padding.RawValue() as f64),
                        Some(c.is_constrained as u8 as f64),
                        Some(c.is_collapsed as u8 as f64),
                        Some(c.is_table_fixed as u8 as f64),
                        Some(c.is_mergeable as u8 as f64),
                    ]);
                }
            } else if mode == "grid" {
                let allow = f.flag();
                let sizes =
                    ComputeGridInlineMinMaxWithAllowance(&columns, spacing, fixed, || allow);
                values.extend([
                    Some(sizes.min_size.RawValue() as f64),
                    Some(sizes.max_size.RawValue() as f64),
                ]);
            } else {
                values.extend(
                    SynchronizeAssignableTableInlineSizeAndColumns(size, fixed, &columns)
                        .into_iter()
                        .map(|v| Some(v.RawValue() as f64)),
                );
            }
        } else {
            let n = f.integer();
            let size = f.unit();
            let spacing = f.unit();
            let percentage = f.unit();
            let rowspan = f.flag();
            let mut rows = Vec::new();
            for _ in 0..n {
                rows.push(Row {
                    block_size: f.unit(),
                    start_cell_index: 0,
                    cell_count: 0,
                    baseline: None,
                    percent: f.percent(),
                    is_constrained: f.flag(),
                    has_rowspan_start: f.flag(),
                    is_collapsed: f.flag(),
                });
            }
            if mode == "rows" {
                let start = f.integer() as u32;
                let count = f.integer() as u32;
                if rowspan {
                    DistributeRowspanCellToRows(
                        &RowspanCell::new(start, count, size),
                        spacing,
                        &mut rows,
                    );
                } else {
                    DistributeSectionFixedBlockSizeToRows(
                        start, count, size, spacing, percentage, &mut rows,
                    );
                }
                values.extend(rows.iter().map(|r| Some(r.block_size.RawValue() as f64)));
            } else if mode == "sections" {
                let m = f.integer();
                let mut sections = Vec::new();
                for _ in 0..m {
                    sections.push(Section {
                        start_row: f.integer() as u32,
                        row_count: f.integer() as u32,
                        block_size: f.unit(),
                        percent: f.percent(),
                        is_constrained: f.flag(),
                        is_tbody: f.flag(),
                        needs_redistribution: f.flag(),
                    });
                }
                DistributeTableBlockSizeToSections(spacing, size, &mut sections, &mut rows);
                for s in sections {
                    values.extend([
                        Some(s.block_size.RawValue() as f64),
                        Some(s.needs_redistribution as u8 as f64),
                    ]);
                }
                values.extend(rows.iter().map(|r| Some(r.block_size.RawValue() as f64)));
            } else {
                let start = f.integer() as u32;
                let span = f.integer() as u32;
                let constrained = f.flag();
                let specified = f.flag();
                let cell = CellBlockConstraint::new(
                    size,
                    BoxStrut::default(),
                    0,
                    span,
                    constrained,
                    false,
                );
                let data = ComputeCellBlockSize(
                    &cell,
                    &rows,
                    start,
                    &LogicalSize::new(LayoutUnit::default(), spacing),
                    specified,
                );
                values.extend([
                    Some(data.block_size.RawValue() as f64),
                    Some(data.is_initial_block_size_indefinite as u8 as f64),
                ]);
            }
        }
        assert_eq!(f.at, f.fields.len(), "input {index}");
        let reference: Vec<Option<f64>> = result
            .split('\t')
            .map(|s| {
                if s == "none" {
                    None
                } else {
                    Some(s.parse().unwrap())
                }
            })
            .collect();
        assert_eq!(values, reference, "C++ case {index}: {input}");
    }
}
