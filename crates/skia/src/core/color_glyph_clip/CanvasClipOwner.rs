//! Private source candidate: complete encoded clip ownership, never dense AA inference.
//! Existing Mask and SkRasterClip remain their current owners. MCRec would
//! own this additional state; save() clones immutable owners, layers reset it.
#[path = "MetadataDispatch.rs"]
pub(crate) mod dispatch;
pub(crate) use dispatch::{Bounds, Encoding, MetadataDispatch};
use std::sync::Arc;

fn valid(b: Bounds) -> bool {
    b.0[0] < b.0[2] && b.0[1] < b.0[3]
}
fn intersection(a: Bounds, b: Bounds) -> Option<Bounds> {
    let c = Bounds([
        a.0[0].max(b.0[0]),
        a.0[1].max(b.0[1]),
        a.0[2].min(b.0[2]),
        a.0[3].min(b.0[3]),
    ]);
    valid(c).then_some(c)
}

/// Canonical opaque SkRegion bands, originating only in the native-equivalent
/// BW scan/construction. This is not an AA coverage-to-encoding constructor.
#[derive(Clone, Debug)]
pub(crate) struct BwRegion {
    bounds: Option<Bounds>,
    rects: Arc<Vec<Bounds>>,
}
impl BwRegion {
    pub(crate) fn from_bw_rects(rects: &[Bounds]) -> Self {
        // Exact SkRegion empty/one-rectangle cases. Only BW region storage:
        // no AA run/y seam is inferred or coalesced here.
        if rects.is_empty() || (rects.len() == 1 && !valid(rects[0])) {
            return Self {
                bounds: None,
                rects: Arc::new(Vec::new()),
            };
        }
        if rects.len() == 1 {
            return Self {
                bounds: Some(rects[0]),
                rects: Arc::new(vec![rects[0]]),
            };
        }
        let mut ys: Vec<_> = rects
            .iter()
            .filter(|b| valid(**b))
            .flat_map(|b| [b.0[1], b.0[3]])
            .collect();
        ys.sort_unstable();
        ys.dedup();
        let mut bands: Vec<(i32, i32, Vec<(i32, i32)>)> = Vec::new();
        for y in ys.windows(2) {
            let mut xs: Vec<_> = rects
                .iter()
                .filter(|b| valid(**b) && b.0[1] <= y[0] && b.0[3] >= y[1])
                .map(|b| (b.0[0], b.0[2]))
                .collect();
            xs.sort_unstable();
            let mut merged: Vec<(i32, i32)> = Vec::new();
            for x in xs {
                if let Some(last) = merged.last_mut().filter(|p| p.1 >= x.0) {
                    last.1 = last.1.max(x.1);
                } else {
                    merged.push(x);
                }
            }
            if merged.is_empty() {
                continue;
            }
            if let Some(last) = bands.last_mut().filter(|p| p.1 == y[0] && p.2 == merged) {
                last.1 = y[1];
            } else {
                bands.push((y[0], y[1], merged));
            }
        }
        let rects: Vec<_> = bands
            .into_iter()
            .flat_map(|(top, bot, xs)| {
                xs.into_iter()
                    .map(move |(left, right)| Bounds([left, top, right, bot]))
            })
            .collect();
        let bounds = rects.first().map(|first| {
            rects.iter().fold(*first, |a, b| {
                Bounds([
                    a.0[0].min(b.0[0]),
                    a.0[1].min(b.0[1]),
                    a.0[2].max(b.0[2]),
                    a.0[3].max(b.0[3]),
                ])
            })
        });
        Self {
            bounds,
            rects: Arc::new(rects),
        }
    }
    fn aa(&self) -> Option<Encoding> {
        Encoding::set_region(self.bounds?, &self.rects)
    }
    fn clipped_rect(&self, rect: Bounds) -> Self {
        let Some(bounds) = self.bounds else {
            return self.clone();
        };
        let Some(clipped) = intersection(bounds, rect) else {
            return Self::from_bw_rects(&[]);
        };
        if clipped == bounds {
            return self.clone();
        }
        if self.rects.len() == 1 {
            return Self::from_bw_rects(&[clipped]);
        }
        let rects: Vec<_> = self
            .rects
            .iter()
            .filter_map(|r| intersection(*r, rect))
            .collect();
        Self::from_bw_rects(&rects)
    }
    fn intersect(&self, other: &Self) -> Self {
        if other.rects.len() == 1 {
            return self.clipped_rect(other.rects[0]);
        }
        let rects: Vec<_> = self
            .rects
            .iter()
            .flat_map(|a| other.rects.iter().filter_map(move |b| intersection(*a, *b)))
            .collect();
        Self::from_bw_rects(&rects)
    }
    fn difference(&self, other: &Self) -> Self {
        let mut rects = (*self.rects).clone();
        for &cut in other.rects.iter() {
            let mut output = Vec::new();
            for a in rects {
                let Some(i) = intersection(a, cut) else {
                    output.push(a);
                    continue;
                };
                for r in [
                    Bounds([a.0[0], a.0[1], a.0[2], i.0[1]]),
                    Bounds([a.0[0], i.0[3], a.0[2], a.0[3]]),
                    Bounds([a.0[0], i.0[1], i.0[0], i.0[3]]),
                    Bounds([i.0[2], i.0[1], a.0[2], i.0[3]]),
                ] {
                    if valid(r) {
                        output.push(r);
                    }
                }
            }
            rects = output;
        }
        Self::from_bw_rects(&rects)
    }
    fn translated(&self, dx: i32, dy: i32) -> Option<Self> {
        let rects: Option<Vec<_>> = self
            .rects
            .iter()
            .map(|r| {
                Some(Bounds([
                    r.0[0].checked_add(dx)?,
                    r.0[1].checked_add(dy)?,
                    r.0[2].checked_add(dx)?,
                    r.0[3].checked_add(dy)?,
                ]))
            })
            .collect();
        Some(Self::from_bw_rects(&rects?))
    }
}

#[derive(Clone, Debug)]
pub(crate) enum CanvasClipOwner {
    Bw(BwRegion),
    // Empty AA is retained as AA except an explicit setEmpty; original
    // RasterClip constructor/updateCache do not always normalize its class.
    Aa(Option<Encoding>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReceiverKind {
    BwRegion,
    AaClip,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CompositeMode {
    SpriteN32,
    LegacyImageN32,
    ImageF16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FloatRect(pub [f32; 4]);
impl FloatRect {
    fn round(self) -> Bounds {
        Bounds(self.0.map(|x| (x + 0.5).floor() as i32))
    }
    fn round_out(self) -> Bounds {
        Bounds([
            self.0[0].floor() as i32,
            self.0[1].floor() as i32,
            self.0[2].ceil() as i32,
            self.0[3].ceil() as i32,
        ])
    }
    fn contains(self, b: Bounds) -> bool {
        self.0[0] <= b.0[0] as f32
            && self.0[1] <= b.0[1] as f32
            && self.0[2] >= b.0[2] as f32
            && self.0[3] >= b.0[3] as f32
    }
    fn nearly_integral(self) -> bool {
        self.0.iter().all(|&x| {
            let shifted = x + 0.125;
            shifted - shifted.floor() < 0.25
        })
    }
}

impl CanvasClipOwner {
    pub(crate) fn device(bounds: Bounds) -> Self {
        Self::Bw(BwRegion::from_bw_rects(&[bounds]))
    }
    pub(crate) fn set_empty(&mut self) {
        *self = Self::Bw(BwRegion::from_bw_rects(&[]));
    }
    pub(crate) fn is_bw(&self) -> bool {
        matches!(self, Self::Bw(_))
    }
    pub(crate) fn path_receiver_kind(&self, do_aa: bool) -> ReceiverKind {
        if do_aa || (self.is_rect() && !self.is_bw()) {
            ReceiverKind::AaClip
        } else {
            ReceiverKind::BwRegion
        }
    }
    pub(crate) fn bounds(&self) -> Option<Bounds> {
        match self {
            Self::Bw(r) => r.bounds,
            Self::Aa(e) => e.as_ref().map(|e| e.bounds),
        }
    }
    fn is_rect(&self) -> bool {
        match self {
            Self::Bw(r) => r.rects.len() == 1,
            Self::Aa(e) => e.as_ref().is_some_and(Encoding::original_is_rect),
        }
    }
    fn aa(&self) -> Option<Encoding> {
        match self {
            Self::Bw(r) => r.aa(),
            Self::Aa(e) => e.clone(),
        }
    }
    fn update_cache(&mut self) {
        if let Self::Aa(Some(e)) = self {
            if e.original_is_rect() {
                *self = Self::device(e.bounds);
            }
        }
    }
    pub(crate) fn intersect_rect(&mut self, rect: Bounds) {
        match self {
            Self::Bw(r) => *r = r.clipped_rect(rect),
            Self::Aa(e) => *e = e.as_ref().and_then(|e| e.intersect_rect(rect)),
        }
        self.update_cache();
    }
    pub(crate) fn intersect_region(&mut self, region: &BwRegion) {
        match self {
            Self::Bw(r) => *r = r.intersect(region),
            Self::Aa(e) => *e = e.as_ref().and_then(|a| a.intersect(&region.aa()?)),
        }
        self.update_cache();
    }
    /// Already mapped finite scale/translate rectangle, following native
    /// SkRasterClip::op(Rect) then SkAAClip::op(Rect), including the BW-only
    /// nearly_integral gate. Other matrices use intersect_path instead.
    /// `scan` emits the real AA rect receiver at these exact pixel bounds.
    pub(crate) fn intersect_float_rect(
        &mut self,
        rect: FloatRect,
        mut do_aa: bool,
        scan: impl FnOnce(FloatRect, Bounds) -> Option<Encoding>,
    ) {
        if matches!(self, Self::Bw(_)) && do_aa && rect.nearly_integral() {
            do_aa = false;
        }
        if !do_aa {
            self.intersect_rect(rect.round());
            return;
        }
        let Some(bounds) = self.bounds() else {
            return;
        };
        // convertToAA suppresses the normal isRect optimization here.
        if let Self::Bw(r) = self {
            *self = Self::Aa(r.aa());
        }
        let Some(pixel_bounds) = intersection(bounds, rect.round_out()) else {
            *self = Self::Aa(None);
            self.update_cache();
            return;
        };
        if rect.contains(bounds) {
            self.update_cache();
            return;
        }
        let current = self.aa().expect("nonempty native AA conversion");
        if current.quick_contains(pixel_bounds) {
            *self = Self::Aa(scan(rect, pixel_bounds));
        } else {
            let operand = scan(rect, pixel_bounds);
            *self = Self::Aa(operand.and_then(|other| current.intersect(&other)));
        }
        self.update_cache();
    }
    /// Scan once at actual current native bounds with its real selected
    /// receiver. A BW path intersecting an existing AA rect must still emit
    /// Builder::Blitter BW events, not substitute setRegion metadata.
    /// `scan` must preserve inverse fill/snug bounds/branch primitives and
    /// return exactly the requested receiver's completed ownership.
    pub(crate) fn intersect_path(
        &mut self,
        do_aa: bool,
        scan: impl FnOnce(Bounds, ReceiverKind) -> CanvasClipOwner,
    ) {
        let Some(bounds) = self.bounds() else {
            return;
        };
        let rectangular = self.is_rect();
        let aa_receiver = do_aa || (rectangular && matches!(self, Self::Aa(_)));
        let receiver = if aa_receiver {
            ReceiverKind::AaClip
        } else {
            ReceiverKind::BwRegion
        };
        let operand = scan(bounds, receiver);
        debug_assert_eq!(matches!(operand, Self::Aa(_)), aa_receiver);
        if rectangular {
            *self = operand;
        } else if let (Self::Bw(a), Self::Bw(b)) = (&mut *self, &operand) {
            *a = a.intersect(b);
        } else {
            let encoded = self.aa().and_then(|a| a.intersect(&operand.aa()?));
            *self = Self::Aa(encoded);
        }
        self.update_cache();
    }
    fn difference_operand(&mut self, operand: CanvasClipOwner) {
        if let (Self::Bw(a), Self::Bw(b)) = (&mut *self, &operand) {
            *a = a.difference(b);
        } else {
            let encoded = self
                .aa()
                .and_then(|a| operand.aa().map_or(Some(a.clone()), |b| a.difference(&b)));
            *self = Self::Aa(encoded);
        }
        self.update_cache();
    }
    /// Difference never uses the rectangular-current-clip setPath shortcut.
    /// Its new SkRasterClip operand is BW iff doAA is false, independently
    /// of the current owner's class, before the ordinary region/AA op.
    pub(crate) fn difference_path(
        &mut self,
        do_aa: bool,
        scan: impl FnOnce(Bounds, ReceiverKind) -> CanvasClipOwner,
    ) {
        let Some(bounds) = self.bounds() else {
            return;
        };
        let kind = if do_aa {
            ReceiverKind::AaClip
        } else {
            ReceiverKind::BwRegion
        };
        self.difference_operand(scan(bounds, kind));
    }
    pub(crate) fn difference_float_rect(
        &mut self,
        rect: FloatRect,
        mut do_aa: bool,
        scan: impl FnOnce(FloatRect, Bounds) -> Option<Encoding>,
    ) {
        if self.is_bw() && do_aa && rect.nearly_integral() {
            do_aa = false;
        }
        if !do_aa {
            let rounded = rect.round();
            match self {
                Self::Bw(r) => *r = r.difference(&BwRegion::from_bw_rects(&[rounded])),
                Self::Aa(e) => *e = e.as_ref().and_then(|a| a.difference_rect(rounded)),
            }
            self.update_cache();
            return;
        }
        let Some(bounds) = self.bounds() else {
            return;
        };
        if let Self::Bw(r) = self {
            *self = Self::Aa(r.aa());
        }
        if intersection(bounds, rect.round_out()).is_none() {
            self.update_cache();
            return;
        }
        if rect.contains(bounds) {
            *self = Self::Aa(None);
            self.update_cache();
            return;
        }
        // Difference's rect setPath clips to the OLD bounds, not pixelBounds.
        let operand = Self::Aa(scan(rect, bounds));
        self.difference_operand(operand);
    }
    pub(crate) fn translated(&self, dx: i32, dy: i32) -> Option<Self> {
        if self.bounds().is_none() {
            return Some(Self::Bw(BwRegion::from_bw_rects(&[])));
        }
        if dx == 0 && dy == 0 {
            return Some(self.clone());
        }
        match self {
            Self::Bw(r) => Some(Self::Bw(r.translated(dx, dy)?)),
            Self::Aa(None) => Some(self.clone()),
            Self::Aa(Some(e)) => {
                let mut out = Self::Aa(Some(e.translated(dx, dy)?));
                // Native translate calls updateCache after changing origin;
                // relative-Y isRect quirk can change its class at top0.
                out.update_cache();
                Some(out)
            }
        }
    }
    pub(crate) fn row_run_starts(&self, width: usize) -> Option<Vec<usize>> {
        let Self::Aa(Some(e)) = self else {
            return None;
        };
        let mut output = Vec::new();
        let mut top = e.bounds.0[1];
        for row in e.rows.iter() {
            let bottom = row.last_y + e.bounds.0[1];
            for y in top..=bottom {
                let mut x = e.bounds.0[0];
                for run in &row.runs {
                    output.push(y as usize * width + x as usize);
                    x += i32::from(run.count);
                }
            }
            top = bottom + 1;
        }
        Some(output)
    }
    pub(crate) fn composite_mode(&self, f16: bool, glyph: Bounds) -> CompositeMode {
        if f16 {
            return CompositeMode::ImageF16;
        }
        match self {
            Self::Bw(_) => CompositeMode::SpriteN32,
            Self::Aa(e) if e.as_ref().is_some_and(|e| e.quick_contains(glyph)) => {
                CompositeMode::SpriteN32
            }
            Self::Aa(_) => CompositeMode::LegacyImageN32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dispatch::{Row, Run};
    fn aa(bounds: Bounds) -> CanvasClipOwner {
        CanvasClipOwner::Aa(Encoding::set_rect(bounds))
    }
    #[test]
    fn narrow_bw_fast_paths_match_general_band_builder() {
        let cases = [
            vec![],
            vec![Bounds([1, 2, 7, 8])],
            vec![Bounds([1, 2, 1, 8])],
            vec![Bounds([-4, -3, 2, 4]), Bounds([5, -3, 9, 4])],
            vec![
                Bounds([0, 0, 8, 3]),
                Bounds([0, 3, 8, 7]),
                Bounds([2, 2, 10, 5]),
            ],
            vec![
                Bounds([0, 0, 2, 2]),
                Bounds([4, 3, 6, 5]),
                Bounds([0, 5, 8, 7]),
            ],
        ];
        let clips = [
            Bounds([-20, -20, 20, 20]),
            Bounds([1, 1, 6, 6]),
            Bounds([0, 0, 0, 5]),
            Bounds([30, 30, 40, 40]),
            Bounds([-4, -3, 2, 4]),
        ];
        for rects in cases {
            // Duplicating input forces the unchanged general union builder.
            let twice: Vec<_> = rects.iter().copied().chain(rects.iter().copied()).collect();
            let region = BwRegion::from_bw_rects(&rects);
            let general = BwRegion::from_bw_rects(&twice);
            assert_eq!(region.bounds, general.bounds);
            assert_eq!(region.rects, general.rects);
            assert_eq!(region.aa(), general.aa());
            for clip in clips {
                let raw: Vec<_> = region
                    .rects
                    .iter()
                    .filter_map(|r| intersection(*r, clip))
                    .collect();
                let twice: Vec<_> = raw.iter().copied().chain(raw.iter().copied()).collect();
                let expected = BwRegion::from_bw_rects(&twice);
                let actual = region.clipped_rect(clip);
                assert_eq!(actual.bounds, expected.bounds);
                assert_eq!(actual.rects, expected.rects);
                assert_eq!(actual.aa(), expected.aa());
                if intersection(region.bounds.unwrap_or(Bounds([0, 0, 0, 0])), clip)
                    == region.bounds
                    && region.bounds.is_some()
                {
                    assert!(Arc::ptr_eq(&region.rects, &actual.rects));
                }
            }
        }
    }
    #[test]
    fn save_restore_and_translation_preserve_shared_y_runs() {
        let original = aa(Bounds([2, 4, 8, 10]));
        let mut saved = original.clone();
        saved.intersect_rect(Bounds([3, 4, 8, 10]));
        let translated = original.translated(10, 20).unwrap();
        let (CanvasClipOwner::Aa(Some(a)), CanvasClipOwner::Aa(Some(b))) = (&original, &translated)
        else {
            panic!()
        };
        assert!(Arc::ptr_eq(&a.rows, &b.rows));
        assert_eq!(original.bounds(), Some(Bounds([2, 4, 8, 10])));
        assert_eq!(saved.bounds(), Some(Bounds([3, 4, 8, 10])));
    }
    #[test]
    fn direct_path_rectangle_shortcut_replaces_not_intersects_encoding() {
        let mut clip = CanvasClipOwner::device(Bounds([0, 0, 10, 10]));
        clip.intersect_path(true, |bounds, kind| {
            assert_eq!(bounds, Bounds([0, 0, 10, 10]));
            assert_eq!(kind, ReceiverKind::AaClip);
            aa(Bounds([2, 2, 8, 8]))
        });
        assert!(matches!(clip, CanvasClipOwner::Aa(_)));
        assert_eq!(
            clip.composite_mode(false, Bounds([3, 3, 4, 4])),
            CompositeMode::SpriteN32
        );
    }
    #[test]
    fn original_top_origin_class_conversion_is_not_dense_rect_test() {
        let mut top0 = aa(Bounds([2, 0, 8, 6]));
        top0.update_cache();
        let mut top2 = aa(Bounds([2, 2, 8, 8]));
        top2.update_cache();
        assert!(matches!(top0, CanvasClipOwner::Bw(_)));
        assert!(matches!(top2, CanvasClipOwner::Aa(_)));
        assert!(matches!(
            top2.translated(0, -2).unwrap(),
            CanvasClipOwner::Bw(_)
        ));
    }
    #[test]
    fn completed_compound_encoding_never_guesses_from_equal_pixels() {
        let b = Bounds([0, 0, 7, 6]);
        let make = |last_y, runs: &[(u8, u8)]| Row {
            last_y,
            runs: runs
                .iter()
                .map(|&(count, alpha)| Run { count, alpha })
                .collect(),
        };
        let a = CanvasClipOwner::Aa(Encoding::finish(
            b,
            vec![make(5, &[(2, 255), (3, 0), (2, 255)])],
        ));
        let c = CanvasClipOwner::Aa(Encoding::finish(
            b,
            vec![
                make(2, &[(2, 255), (3, 0), (2, 255)]),
                make(5, &[(2, 255), (1, 0), (2, 0), (2, 255)]),
            ],
        ));
        let glyph = Bounds([0, 0, 1, 4]);
        assert_eq!(a.composite_mode(false, glyph), CompositeMode::SpriteN32);
        assert_eq!(
            c.composite_mode(false, glyph),
            CompositeMode::LegacyImageN32
        );
    }
    #[test]
    fn nonrect_bw_region_converts_using_canonical_region_not_path_rows() {
        let r = BwRegion::from_bw_rects(&[Bounds([0, 0, 2, 6]), Bounds([5, 0, 7, 6])]);
        let mut c = CanvasClipOwner::Bw(r);
        c.intersect_path(true, |_, kind| {
            assert_eq!(kind, ReceiverKind::AaClip);
            aa(Bounds([0, 0, 7, 6]))
        });
        assert!(matches!(c, CanvasClipOwner::Aa(_)));
        assert_eq!(
            c.composite_mode(false, Bounds([0, 0, 1, 5])),
            CompositeMode::SpriteN32
        );
        assert_eq!(
            c.composite_mode(false, Bounds([0, 0, 1, 6])),
            CompositeMode::LegacyImageN32
        );
    }
    #[test]
    fn child_layer_reset_does_not_mutate_saved_aa_owner() {
        let parent = aa(Bounds([2, 2, 8, 8]));
        let mut child = CanvasClipOwner::device(Bounds([0, 0, 4, 4]));
        child.set_empty();
        assert!(parent.bounds().is_some());
        assert!(child.bounds().is_none());
    }
    #[test]
    fn near_integral_float_rect_stays_bw_without_aa_receiver() {
        let mut clip = CanvasClipOwner::device(Bounds([0, 0, 10, 10]));
        clip.intersect_float_rect(FloatRect([2.0625, 2.0625, 7.9375, 7.9375]), true, |_, _| {
            panic!("native BW gate skips AA scan")
        });
        assert!(matches!(clip, CanvasClipOwner::Bw(_)));
        assert_eq!(clip.bounds(), Some(Bounds([2, 2, 8, 8])));
    }
    #[test]
    fn aa_float_containment_keeps_original_encoded_owner() {
        let mut clip = aa(Bounds([2, 2, 8, 8]));
        let CanvasClipOwner::Aa(Some(before)) = clip.clone() else {
            panic!()
        };
        clip.intersect_float_rect(FloatRect([1.5, 1.5, 8.5, 8.5]), true, |_, _| {
            panic!("original identity skips rect scan")
        });
        let CanvasClipOwner::Aa(Some(after)) = clip else {
            panic!()
        };
        assert!(Arc::ptr_eq(&before.rows, &after.rows));
    }
    #[test]
    fn fractional_rect_quick_contains_selects_path_replace_bounds() {
        let mut clip = aa(Bounds([0, 2, 10, 10]));
        clip.intersect_float_rect(FloatRect([2.25, 3.25, 7.75, 7.75]), true, |rect, bounds| {
            assert_eq!(rect, FloatRect([2.25, 3.25, 7.75, 7.75]));
            assert_eq!(bounds, Bounds([2, 3, 8, 8]));
            Encoding::finish(
                bounds,
                vec![Row {
                    last_y: 7,
                    runs: vec![Run {
                        count: 6,
                        alpha: 200,
                    }],
                }],
            )
        });
        let CanvasClipOwner::Aa(Some(e)) = clip else {
            panic!()
        };
        assert_eq!(e.bounds, Bounds([2, 3, 8, 8]));
        assert_eq!(
            e.rows[0].runs,
            vec![Run {
                count: 6,
                alpha: 200
            }]
        );
    }
}

impl CanvasClipOwner {
    // Every actual original Arc/Vec/nested run capacity is charged, including
    // unused rows. No data() or encoded/dense normalization, no temporary Vec.
    pub(crate) fn clip_product_owned_heap_bytes(&self) -> Option<usize> {
        use std::mem::size_of;
        match self {
            Self::Bw(region) => (2 * size_of::<usize>())
                .checked_add(size_of::<Vec<Bounds>>())?
                .checked_add(region.rects.capacity().checked_mul(size_of::<Bounds>())?),
            Self::Aa(None) => Some(0),
            Self::Aa(Some(e)) => {
                let mut bytes = (2 * size_of::<usize>())
                    .checked_add(size_of::<Vec<dispatch::Row>>())?
                    .checked_add(e.rows.capacity().checked_mul(size_of::<dispatch::Row>())?)?;
                for row in e.rows.iter() {
                    bytes = bytes.checked_add(
                        row.runs
                            .capacity()
                            .checked_mul(size_of::<dispatch::Run>())?,
                    )?;
                }
                Some(bytes)
            }
        }
    }
}
