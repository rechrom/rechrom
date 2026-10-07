//! SkAAClip's count/alpha rows and quickContains admission for an actual A8
//! coverage product. Coverage rasterization and edge blending are unchanged.
//! The CPU receiver supplies dense A8; encoding it once adapts SkAAClip's
//! Builder/run representation, rather than caching a guessed opaque rectangle.

struct Row {
    // SkAAClip::YOffset::fY: inclusive last row sharing these encoded runs.
    last_y: usize,
    offset: usize,
    end: usize,
}

pub(crate) struct AAClipRuns {
    width: usize,
    height: usize,
    rows: Vec<Row>,
    data: Vec<u8>,
}

// SkAAClip::findX / expandToRuns: borrow the existing count/alpha product,
// clipping its first and final count to the actual horizontal write span.
pub(crate) struct RowRuns<'a> {
    data: &'a [u8],
    count: usize,
    remaining: usize,
    x: usize,
}
impl Iterator for RowRuns<'_> {
    type Item = (usize, usize, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let count = self.count.min(self.remaining);
        let result = (self.x, count, self.data[1]);
        self.x += count;
        self.remaining -= count;
        self.data = &self.data[2..];
        self.count = self.data.first().copied().unwrap_or(0) as usize;
        Some(result)
    }
}

impl AAClipRuns {
    pub(crate) fn from_alpha(pixels: &[u8], width: usize) -> Self {
        let height = if width == 0 { 0 } else { pixels.len() / width };
        let mut result = Self {
            width,
            height,
            rows: Vec::new(),
            data: Vec::new(),
        };
        if width == 0 || pixels.len() % width != 0 {
            return result;
        }
        let mut encoded = Vec::new();
        for (y, row) in pixels.chunks_exact(width).enumerate() {
            encoded.clear();
            let mut x = 0;
            while x < width {
                let alpha = row[x];
                let start = x;
                x += 1;
                // SkAAClip::Builder::appendRun splits runs at 255 because
                // count and coverage are each one byte (SkAAClip.cpp).
                while x < width && x - start < 255 && row[x] == alpha {
                    x += 1;
                }
                encoded.extend_from_slice(&[(x - start) as u8, alpha]);
            }
            if let Some(last) = result.rows.last_mut() {
                if result.data[last.offset..last.end] == encoded {
                    last.last_y = y;
                    continue;
                }
            }
            let offset = result.data.len();
            result.data.extend_from_slice(&encoded);
            result.rows.push(Row {
                last_y: y,
                offset,
                end: result.data.len(),
            });
        }
        result
    }

    pub(crate) fn bytes(&self) -> usize {
        self.data
            .len()
            .saturating_add(self.rows.len().saturating_mul(std::mem::size_of::<Row>()))
    }

    /// Actual horizontal coverage runs, with offsets relative to `left`.
    /// SkAAClipBlitter::blitH (SkAAClip.cpp:1670) expands these runs for the
    /// underlying AA blitter; callers can dispatch FF/zero/partial separately.
    pub(crate) fn row_runs(&self, y: usize, left: usize, right: usize) -> RowRuns<'_> {
        let empty = || RowRuns {
            data: &[],
            count: 0,
            remaining: 0,
            x: 0,
        };
        if y >= self.height || left >= right || right > self.width {
            return empty();
        }
        let Some(row) = self.rows.iter().find(|row| row.last_y >= y) else {
            return empty();
        };
        let mut data = &self.data[row.offset..row.end];
        let mut x = left;
        while !data.is_empty() && x >= data[0] as usize {
            x -= data[0] as usize;
            data = &data[2..];
        }
        if data.is_empty() {
            return empty();
        }
        RowRuns {
            data,
            count: data[0] as usize - x,
            remaining: right - left,
            x: 0,
        }
    }

    /// SkAAClip.cpp:1597-1627, including its conservative lastY check. The
    /// rectangle is in this A8 product's local integer coordinates. Identical
    /// encoded rows must cover the full query and every crossed run must be FF.
    pub(crate) fn quick_contains(
        &self,
        left: usize,
        top: usize,
        right: usize,
        bottom: usize,
    ) -> bool {
        if self.rows.is_empty()
            || left >= right
            || top >= bottom
            || right > self.width
            || bottom > self.height
        {
            return false;
        }
        let Some(row) = self.rows.iter().find(|row| row.last_y >= top) else {
            return false;
        };
        if row.last_y < bottom {
            return false;
        }
        let data = &self.data[row.offset..row.end];
        let mut at = 0;
        let mut x = left;
        // SkAAClip::findX leaves the remaining count in the starting run.
        while at < data.len() && x >= data[at] as usize {
            x -= data[at] as usize;
            at += 2;
        }
        if at >= data.len() {
            return false;
        }
        let mut count = data[at] as usize - x;
        let mut remaining = right - left;
        while data[at + 1] == 255 {
            if count >= remaining {
                return true;
            }
            remaining -= count;
            at += 2;
            if at >= data.len() {
                return false;
            }
            count = data[at] as usize;
        }
        false
    }
}

#[cfg(test)]
#[test]
fn quick_contains_requires_real_full_runs_and_repeated_rows() {
    let width = 520;
    let mut pixels = vec![255; width * 7];
    for row in pixels.chunks_exact_mut(width) {
        row[0] = 0;
        row[1] = 128;
        row[518] = 64;
        row[519] = 0;
    }
    pixels[2] = 192;
    pixels[6 * width + 2] = 192;
    let clip = AAClipRuns::from_alpha(&pixels, width);
    assert_eq!(
        clip.row_runs(1, 250, 518).collect::<Vec<_>>(),
        vec![(0, 7, 255), (7, 255, 255), (262, 6, 255)],
        "findX clips the first count without shifting the sampling origin"
    );
    assert_eq!(
        clip.row_runs(1, 0, 4).collect::<Vec<_>>(),
        vec![(0, 1, 0), (1, 1, 128), (2, 2, 255)],
        "preserve zero/partial/FF runs and clip the final count"
    );
    assert!(
        clip.quick_contains(2, 1, 518, 5),
        "cross consecutive FF runs longer than 255"
    );
    assert!(
        !clip.quick_contains(1, 1, 518, 5),
        "left AA edge is not full coverage"
    );
    assert!(
        !clip.quick_contains(2, 1, 519, 5),
        "right AA edge is not full coverage"
    );
    assert!(
        !clip.quick_contains(2, 0, 518, 1),
        "different top row cannot cover the rectangle"
    );
    assert!(
        !clip.quick_contains(2, 1, 518, 6),
        "preserve SkAAClip's conservative lastY rule"
    );
    assert!(
        !clip.quick_contains(2, 1, 521, 5),
        "outside product bounds is never contained"
    );
}
