//! Independent scalar metadata candidate. Not production/compiled/tested.
//! Input rows must come from the real RLE receiver before trim, not from
//! equal-byte runs of the final dense mask or an unproven sparse-start union.
//! Coverage remains in the existing Mask owner; this owns only its encoding.
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Bounds(pub [i32; 4]);
impl Bounds {
    fn valid(self) -> bool {
        self.0[0] < self.0[2] && self.0[1] < self.0[3]
    }
    fn width(self) -> i32 {
        self.0[2] - self.0[0]
    }
    fn intersection(self, other: Self) -> Option<Self> {
        let b = Self([
            self.0[0].max(other.0[0]),
            self.0[1].max(other.0[1]),
            self.0[2].min(other.0[2]),
            self.0[3].min(other.0[3]),
        ]);
        b.valid().then_some(b)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Run {
    pub count: u8,
    pub alpha: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    /// Inclusive Y endpoint: absolute in finish() input, relative to bounds.top
    /// in completed Encoding storage (native RunHead sharing/translation).
    pub last_y: i32,
    pub runs: Vec<Run>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Encoding {
    pub bounds: Bounds,
    pub rows: Arc<Vec<Row>>,
}
fn append(runs: &mut Vec<Run>, alpha: u8, mut count: i32) {
    // Native Builder::AppendRun preserves equal adjacent alpha segments.
    while count > 0 {
        let n = count.min(255) as u8;
        runs.push(Run { count: n, alpha });
        count -= i32::from(n);
    }
}
fn zero(row: &Row) -> bool {
    row.runs.iter().all(|r| r.alpha == 0)
}
fn edge_zeros(row: &Row) -> (i32, i32) {
    let left = row
        .runs
        .iter()
        .take_while(|r| r.alpha == 0)
        .map(|r| i32::from(r.count))
        .sum();
    let right = row
        .runs
        .iter()
        .rev()
        .take_while(|r| r.alpha == 0)
        .map(|r| i32::from(r.count))
        .sum();
    (left, right)
}
fn trim_runs(runs: &mut Vec<Run>, mut left: i32, mut right: i32) {
    let mut removed = 0;
    while left > 0 {
        let r = &mut runs[removed];
        assert_eq!(r.alpha, 0);
        let count = i32::from(r.count);
        if count > left {
            r.count = (count - left) as u8;
            break;
        }
        left -= count;
        removed += 1;
    }
    runs.drain(..removed);
    while right > 0 {
        let r = runs.last_mut().unwrap();
        assert_eq!(r.alpha, 0);
        let count = i32::from(r.count);
        if count > right {
            r.count = (count - right) as u8;
            break;
        }
        right -= count;
        runs.pop();
    }
}

impl Encoding {
    /// Receiver supplies complete zero-gap-padded Builder rows in ascending Y.
    /// Each existing row corresponds to one raw emitted Y band. This validates
    /// lengths, merges only encoded equality before trim, and mirrors finish.
    /// It deliberately has no "from_dense" / "from_sparse_union" constructor.
    pub(crate) fn finish(bounds: Bounds, raw: Vec<Row>) -> Option<Self> {
        if !bounds.valid() || raw.is_empty() {
            return None;
        }
        let mut rows: Vec<Row> = Vec::new();
        let mut previous = bounds.0[1] - 1;
        for row in raw {
            assert!(row.last_y > previous && row.last_y < bounds.0[3]);
            assert!(row.runs.iter().all(|r| r.count > 0));
            assert_eq!(
                row.runs.iter().map(|r| i32::from(r.count)).sum::<i32>(),
                bounds.width()
            );
            previous = row.last_y;
            if let Some(last) = rows.last_mut().filter(|last| last.runs == row.runs) {
                last.last_y = row.last_y;
            } else {
                rows.push(row);
            }
        }
        // Builder may stop before its initial bottom; trimBounds first sets
        // bottom to its actual final Y row, then trims zero rows/columns.
        let mut bounds = bounds;
        bounds.0[3] = rows.last().unwrap().last_y + 1;
        let first = rows.iter().position(|row| !zero(row))?;
        let last = rows.iter().rposition(|row| !zero(row)).unwrap();
        if first > 0 {
            bounds.0[1] = rows[first - 1].last_y + 1;
        }
        bounds.0[3] = rows[last].last_y + 1;
        rows.drain(last + 1..);
        rows.drain(..first);
        let left = rows.iter().map(|r| edge_zeros(r).0).min().unwrap();
        let right = rows.iter().map(|r| edge_zeros(r).1).min().unwrap();
        for row in &mut rows {
            trim_runs(&mut row.runs, left, right);
        }
        bounds.0[0] += left;
        bounds.0[2] -= right;
        // Do not merge Y again after trimLeftRight: upstream explicitly leaves
        // duplicate X/Y cleanup as TODO. Lost pre-trim Y boundaries are not
        // recoverable from final dense pixels or surviving sparse starts.
        for row in &mut rows {
            row.last_y -= bounds.0[1];
        }
        Some(Self {
            bounds,
            rows: Arc::new(rows),
        })
    }
    /// Original RunHead::AllocRect: one relative inclusive Y endpoint and
    /// opaque AppendRun counts anchored at the actual rectangle's left edge.
    pub(crate) fn set_rect(bounds: Bounds) -> Option<Self> {
        if !bounds.valid() {
            return None;
        }
        let mut runs = Vec::new();
        append(&mut runs, 255, bounds.width());
        Some(Self {
            bounds,
            rows: Arc::new(vec![Row {
                last_y: bounds.0[3] - bounds.0[1] - 1,
                runs,
            }]),
        })
    }
    /// Genuine canonical SkRegion::Iterator rectangles, in its actual order.
    /// No dense-derived AA factory: caller must supply native-equivalent BW
    /// region bands. setRegion does not call Builder flushRow or trimBounds.
    pub(crate) fn set_region(bounds: Bounds, rects: &[Bounds]) -> Option<Self> {
        if !bounds.valid() || rects.is_empty() {
            return None;
        }
        let mut rows: Vec<Row> = Vec::new();
        let mut previous_right = 0;
        let mut previous_bot = 0;
        for &rect in rects {
            assert!(rect.valid() && rect.intersection(bounds) == Some(rect));
            let bot = rect.0[3] - bounds.0[1];
            assert!(bot >= previous_bot);
            if bot > previous_bot {
                if let Some(row) = rows.last_mut() {
                    append(&mut row.runs, 0, bounds.width() - previous_right);
                }
                let top = rect.0[1] - bounds.0[1];
                if top > previous_bot {
                    let mut runs = Vec::new();
                    append(&mut runs, 0, bounds.width());
                    rows.push(Row {
                        last_y: top - 1,
                        runs,
                    });
                }
                rows.push(Row {
                    last_y: bot - 1,
                    runs: Vec::new(),
                });
                previous_right = 0;
                previous_bot = bot;
            }
            let x = rect.0[0] - bounds.0[0];
            assert!(x >= previous_right);
            let row = rows.last_mut().unwrap();
            append(&mut row.runs, 0, x - previous_right);
            append(&mut row.runs, 255, rect.width());
            previous_right = x + rect.width();
        }
        append(
            &mut rows.last_mut().unwrap().runs,
            0,
            bounds.width() - previous_right,
        );
        Some(Self {
            bounds,
            rows: Arc::new(rows),
        })
    }
    /// Source SkAAClip::op(IRect,Intersect) identity/setRect/encoded-op order.
    /// Original class conversion remains the Canvas caller's responsibility.
    pub(crate) fn intersect_rect(&self, rect: Bounds) -> Option<Self> {
        let pixel_bounds = self.bounds.intersection(rect)?;
        if pixel_bounds == self.bounds {
            return Some(self.clone());
        }
        if self.quick_contains(pixel_bounds) {
            return Self::set_rect(pixel_bounds);
        }
        self.intersect(&Self::set_rect(rect)?)
    }
    /// Native translate changes bounds only and retains the same RunHead.
    /// Checked arithmetic declines an unrepresentable device coordinate.
    pub(crate) fn translated(&self, dx: i32, dy: i32) -> Option<Self> {
        Some(Self {
            bounds: Bounds([
                self.bounds.0[0].checked_add(dx)?,
                self.bounds.0[1].checked_add(dy)?,
                self.bounds.0[2].checked_add(dx)?,
                self.bounds.0[3].checked_add(dy)?,
            ]),
            rows: self.rows.clone(),
        })
    }
    pub(crate) fn quick_contains(&self, query: Bounds) -> bool {
        if !query.valid() || query.intersection(self.bounds) != Some(query) {
            return false;
        }
        let row = &self.rows[self
            .rows
            .partition_point(|r| r.last_y < query.0[1] - self.bounds.0[1])];
        if row.last_y < query.0[3] - self.bounds.0[1] {
            return false;
        }
        let mut x = self.bounds.0[0];
        for run in &row.runs {
            let right = x + i32::from(run.count);
            if right > query.0[0] && x < query.0[2] && run.alpha != 255 {
                return false;
            }
            if right >= query.0[2] {
                return true;
            }
            x = right;
        }
        false
    }
    pub(crate) fn original_is_rect(&self) -> bool {
        // Source SkAAClip::isRect compares relative row Y with absolute bottom.
        // Preserve it; "opaque dense bbox" is not a proof of BW conversion.
        self.rows.len() == 1
            && self.rows[0].last_y == self.bounds.0[3] - 1
            && self.rows[0].runs.iter().all(|r| r.alpha == 255)
    }
    /// SkAAClip::op(Difference) retains A's initial bounds and streams the
    /// exact existing X/Y run boundaries through Builder::operateY/operateX.
    pub(crate) fn difference(&self, other: &Self) -> Option<Self> {
        if self.bounds.intersection(other.bounds).is_none() {
            return Some(self.clone());
        }
        let bounds = self.bounds;
        let mut rows = Vec::new();
        let mut y = bounds.0[1];
        while y < bounds.0[3] {
            let a = &self.rows[self
                .rows
                .partition_point(|r| r.last_y < y - self.bounds.0[1])];
            let b = if y >= other.bounds.0[1] && y < other.bounds.0[3] {
                Some(
                    &other.rows[other
                        .rows
                        .partition_point(|r| r.last_y < y - other.bounds.0[1])],
                )
            } else {
                None
            };
            let b_end = b.map_or_else(
                || {
                    if y < other.bounds.0[1] {
                        other.bounds.0[1] - 1
                    } else {
                        bounds.0[3] - 1
                    }
                },
                |row| row.last_y + other.bounds.0[1],
            );
            let end = (a.last_y + self.bounds.0[1])
                .min(b_end)
                .min(bounds.0[3] - 1);
            let mut ai = 0;
            let mut ax = self.bounds.0[0];
            let mut bi = 0;
            let mut bx = other.bounds.0[0];
            let mut x = bounds.0[0];
            let mut runs = Vec::new();
            while x < bounds.0[2] {
                while ax + i32::from(a.runs[ai].count) <= x {
                    ax += i32::from(a.runs[ai].count);
                    ai += 1;
                }
                let a_right = ax + i32::from(a.runs[ai].count);
                let (b_alpha, b_right) = if let Some(b) = b {
                    if x < other.bounds.0[0] {
                        (0, other.bounds.0[0])
                    } else if x >= other.bounds.0[2] {
                        (0, bounds.0[2])
                    } else {
                        while bx + i32::from(b.runs[bi].count) <= x {
                            bx += i32::from(b.runs[bi].count);
                            bi += 1;
                        }
                        (b.runs[bi].alpha, bx + i32::from(b.runs[bi].count))
                    }
                } else {
                    (0, bounds.0[2])
                };
                let right = a_right.min(b_right).min(bounds.0[2]);
                let product = u32::from(a.runs[ai].alpha) * u32::from(255 - b_alpha) + 128;
                append(
                    &mut runs,
                    ((product + (product >> 8)) >> 8) as u8,
                    right - x,
                );
                x = right;
            }
            rows.push(Row { last_y: end, runs });
            y = end + 1;
        }
        Self::finish(bounds, rows)
    }
    pub(crate) fn difference_rect(&self, rect: Bounds) -> Option<Self> {
        let Some(overlap) = self.bounds.intersection(rect) else {
            return Some(self.clone());
        };
        if overlap == self.bounds {
            return None;
        }
        self.difference(&Self::set_rect(rect)?)
    }
    pub(crate) fn intersect(&self, other: &Self) -> Option<Self> {
        let bounds = self.bounds.intersection(other.bounds)?;
        let mut y = bounds.0[1];
        let mut rows = Vec::new();
        while y < bounds.0[3] {
            let a = &self.rows[self
                .rows
                .partition_point(|r| r.last_y < y - self.bounds.0[1])];
            let b = &other.rows[other
                .rows
                .partition_point(|r| r.last_y < y - other.bounds.0[1])];
            let end = (a.last_y + self.bounds.0[1])
                .min(b.last_y + other.bounds.0[1])
                .min(bounds.0[3] - 1);
            let mut ai = 0;
            let mut ax = self.bounds.0[0];
            let mut bi = 0;
            let mut bx = other.bounds.0[0];
            let mut x = bounds.0[0];
            let mut runs = Vec::new();
            while x < bounds.0[2] {
                while ax + i32::from(a.runs[ai].count) <= x {
                    ax += i32::from(a.runs[ai].count);
                    ai += 1;
                }
                while bx + i32::from(b.runs[bi].count) <= x {
                    bx += i32::from(b.runs[bi].count);
                    bi += 1;
                }
                let right = (ax + i32::from(a.runs[ai].count))
                    .min(bx + i32::from(b.runs[bi].count))
                    .min(bounds.0[2]);
                let product = u32::from(a.runs[ai].alpha) * u32::from(b.runs[bi].alpha) + 128;
                let alpha = ((product + (product >> 8)) >> 8) as u8;
                append(&mut runs, alpha, right - x);
                x = right;
            }
            rows.push(Row { last_y: end, runs });
            y = end + 1;
        }
        Self::finish(bounds, rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(last_y: i32, entries: &[(u8, u8)]) -> Row {
        Row {
            last_y,
            runs: entries
                .iter()
                .map(|&(count, alpha)| Run { count, alpha })
                .collect(),
        }
    }
    fn expand(e: &Encoding) -> Vec<u8> {
        let mut out = Vec::new();
        let mut top = e.bounds.0[1];
        for row in e.rows.iter() {
            for _ in top..=row.last_y + e.bounds.0[1] {
                for r in &row.runs {
                    out.extend(std::iter::repeat_n(r.alpha, r.count as usize));
                }
            }
            top = row.last_y + e.bounds.0[1] + 1;
        }
        out
    }
    #[test]
    fn equal_dense_segmentations_do_not_erase_distinct_y_state() {
        let a = Encoding::finish(
            Bounds([0, 0, 7, 6]),
            vec![row(5, &[(2, 255), (3, 0), (2, 255)])],
        )
        .unwrap();
        let b = Encoding::finish(
            Bounds([0, 0, 7, 6]),
            vec![
                row(2, &[(2, 255), (3, 0), (2, 255)]),
                row(5, &[(2, 255), (1, 0), (2, 0), (2, 255)]),
            ],
        )
        .unwrap();
        assert_eq!(expand(&a), expand(&b));
        assert!(a.quick_contains(Bounds([0, 0, 1, 4])));
        assert!(!b.quick_contains(Bounds([0, 0, 1, 4])));
        assert!(a.quick_contains(Bounds([0, 0, 1, 5])));
        assert!(!a.quick_contains(Bounds([0, 0, 1, 6])));
    }
    #[test]
    fn trimming_never_remerges_preexisting_y_boundary() {
        let e = Encoding::finish(
            Bounds([0, 0, 5, 6]),
            vec![
                row(2, &[(1, 0), (1, 0), (3, 255)]),
                row(5, &[(2, 0), (3, 255)]),
            ],
        )
        .unwrap();
        assert_eq!(e.bounds, Bounds([2, 0, 5, 6]));
        assert_eq!(e.rows.len(), 2);
        assert_eq!(e.rows[0].runs, e.rows[1].runs);
        assert!(!e.quick_contains(Bounds([2, 0, 3, 4])));
    }
    #[test]
    fn opaque_rect_keeps_source_origin_sensitive_classification() {
        let top0 = Encoding::finish(Bounds([2, 0, 5, 6]), vec![row(5, &[(3, 255)])]).unwrap();
        let top2 = Encoding::finish(Bounds([2, 2, 5, 6]), vec![row(5, &[(3, 255)])]).unwrap();
        assert!(top0.original_is_rect());
        assert!(!top2.original_is_rect());
    }
    #[test]
    fn saved_and_translated_encoding_share_immutable_relative_rows() {
        let original = Encoding::finish(Bounds([2, 4, 8, 10]), vec![row(9, &[(6, 255)])]).unwrap();
        let saved = original.clone();
        let moved = original.translated(10, 20).unwrap();
        assert!(Arc::ptr_eq(&original.rows, &saved.rows));
        assert!(Arc::ptr_eq(&original.rows, &moved.rows));
        assert_eq!(original.rows[0].last_y, 5);
        assert!(original.quick_contains(Bounds([3, 5, 6, 9])));
        assert!(moved.quick_contains(Bounds([13, 25, 16, 29])));
        assert!(!moved.quick_contains(Bounds([13, 25, 16, 30])));
        drop(original);
        drop(saved);
        assert!(moved.quick_contains(Bounds([13, 25, 16, 29])));
    }
    #[test]
    fn contained_integer_rect_keeps_owner_and_original_y_boundaries() {
        let e = Encoding::finish(
            Bounds([2, 3, 7, 9]),
            vec![
                row(5, &[(2, 255), (1, 0), (2, 255)]),
                row(8, &[(2, 255), (1, 0), (2, 255)]),
            ],
        )
        .unwrap();
        let retained = e.intersect_rect(Bounds([0, 0, 20, 20])).unwrap();
        assert!(Arc::ptr_eq(&e.rows, &retained.rows));
        let rectangle = e.intersect_rect(Bounds([2, 3, 4, 5])).unwrap();
        assert_eq!(rectangle.bounds, Bounds([2, 3, 4, 5]));
        assert_eq!(rectangle.rows.as_slice(), &[row(1, &[(2, 255)])]);
        assert!(!Arc::ptr_eq(&e.rows, &rectangle.rows));
    }
    #[test]
    fn real_region_bands_keep_255_partition_and_explicit_y_hole() {
        let e = Encoding::set_region(
            Bounds([4, 2, 524, 10]),
            &[
                Bounds([4, 2, 264, 4]),
                Bounds([300, 2, 524, 4]),
                Bounds([4, 7, 264, 10]),
                Bounds([300, 7, 524, 10]),
            ],
        )
        .unwrap();
        assert_eq!(
            e.rows.iter().map(|r| r.last_y).collect::<Vec<_>>(),
            [1, 4, 7]
        );
        assert_eq!(
            e.rows[0]
                .runs
                .iter()
                .map(|r| (r.count, r.alpha))
                .collect::<Vec<_>>(),
            [(255, 255), (5, 255), (36, 0), (224, 255)]
        );
        assert_eq!(
            e.rows[1]
                .runs
                .iter()
                .map(|r| (r.count, r.alpha))
                .collect::<Vec<_>>(),
            [(255, 0), (255, 0), (10, 0)]
        );
        assert!(e.quick_contains(Bounds([5, 2, 20, 3])));
        assert!(!e.quick_contains(Bounds([5, 2, 20, 4])));
    }
}
