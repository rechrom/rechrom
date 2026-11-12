//! Source FloatRoundedRect prerequisites; geometry/float_rounded_rect.h:70-227,
//! geometry/float_rounded_rect.cc:87-332. Corner order is TL, TR, BR, BL.
use foundation::{
    gfx::{OutsetsF, PointF, QuadF, RectF, SizeF, Vector2dF},
    gfx_quad_geometry::QuadFHitTestExt,
};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Radii(pub [SizeF; 4]);
impl Radii {
    pub fn IsZero(&self) -> bool {
        self.0.iter().all(SizeF::IsZero)
    }
    pub fn Scale(&mut self, factor: f32) {
        if factor == 1. {
            return;
        }
        for r in &mut self.0 {
            r.ScaleUniform(factor);
            if r.width() == 0. || r.height() == 0. {
                *r = SizeF::default();
            }
        }
    }
    pub fn SetMinimumRadius(&mut self, minimum: f32) {
        for r in &mut self.0 {
            *r = SizeF::new(minimum.max(r.width()), minimum.max(r.height()));
        }
    }
    pub fn UniformRadius(&self) -> Option<f32> {
        let r = self.0[0];
        (r.width() == r.height() && self.0.iter().all(|v| *v == r)).then_some(r.width())
    }
    pub fn Outset(&mut self, outsets: OutsetsF) {
        let deltas = [
            (outsets.left(), outsets.top()),
            (outsets.right(), outsets.top()),
            (outsets.right(), outsets.bottom()),
            (outsets.left(), outsets.bottom()),
        ];
        for (r, (x, y)) in self.0.iter_mut().zip(deltas) {
            if r.width() > 0. {
                r.set_width(r.width() + x);
            }
            if r.height() > 0. {
                r.set_height(r.height() + y);
            }
        }
    }
    pub fn OutsetWithCornerCorrection(&mut self, outsets: OutsetsF, size: SizeF) {
        let deltas = [
            (outsets.left(), outsets.top()),
            (outsets.right(), outsets.top()),
            (outsets.right(), outsets.bottom()),
            (outsets.left(), outsets.bottom()),
        ];
        for (r, (x, y)) in self.0.iter_mut().zip(deltas) {
            if r.IsZero() || (x == 0. && y == 0.) {
                continue;
            }
            let coverage = 2. * (r.width() / size.width()).min(r.height() / size.height());
            fn adjusted(radius: f32, outset: f32, coverage: f32) -> f32 {
                if radius > outset || coverage > 1. {
                    return radius + outset;
                }
                let ratio = radius / outset;
                (radius as f64
                    + outset as f64
                        * (1. - ((1. - ratio) as f64).powi(3) * (1. - (coverage as f64).powi(3))))
                    as f32
            }
            *r = SizeF::new(
                adjusted(r.width(), x, coverage),
                adjusted(r.height(), y, coverage),
            );
        }
    }
    pub fn OutsetForShapeMargin(&mut self, outset: f32) {
        for r in &mut self.0 {
            *r += SizeF::new(outset, outset);
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FloatRoundedRect {
    pub rect: RectF,
    pub radii: Radii,
}
impl FloatRoundedRect {
    pub fn new(rect: RectF, radii: Radii) -> Self {
        Self { rect, radii }
    }
    pub fn IsRounded(&self) -> bool {
        !self.radii.IsZero()
    }
    pub fn IsEmpty(&self) -> bool {
        self.rect.IsEmpty()
    }
    pub fn Move(&mut self, v: Vector2dF) {
        self.rect.Offset(v.x(), v.y());
    }
    fn outset_rect(&mut self, o: OutsetsF) {
        self.rect = RectF::new(
            PointF::new(self.rect.x() - o.left(), self.rect.y() - o.top()),
            SizeF::new(
                self.rect.width() + o.left() + o.right(),
                self.rect.height() + o.top() + o.bottom(),
            ),
        );
    }
    pub fn Outset(&mut self, o: OutsetsF) {
        self.outset_rect(o);
        self.radii.Outset(o);
    }
    pub fn Inset(&mut self, o: OutsetsF) {
        self.Outset(OutsetsF::new(-o.top(), -o.right(), -o.bottom(), -o.left()));
    }
    pub fn OutsetWithCornerCorrection(&mut self, o: OutsetsF) {
        if o == OutsetsF::default() {
            return;
        }
        let size = self.rect.size();
        self.outset_rect(o);
        if !size.IsEmpty() {
            self.radii.OutsetWithCornerCorrection(o, size);
        }
    }
    pub fn OutsetForShapeMargin(&mut self, outset: f32) {
        debug_assert!(outset >= 0.);
        if outset != 0. {
            self.outset_rect(OutsetsF::new(outset, outset, outset, outset));
            self.radii.OutsetForShapeMargin(outset);
        }
    }
    pub fn CornerRect(&self, index: usize) -> RectF {
        let r = self.radii.0[index];
        let x = if index == 0 || index == 3 {
            self.rect.x()
        } else {
            self.rect.right() - r.width()
        };
        let y = if index < 2 {
            self.rect.y()
        } else {
            self.rect.bottom() - r.height()
        };
        RectF::new(PointF::new(x, y), r)
    }
    pub fn ConstrainRadii(&mut self) {
        let r = self.radii.0;
        let mut factor = 1_f32;
        let horizontal = (r[0].width() + r[1].width()).max(r[3].width() + r[2].width());
        if horizontal > self.rect.width() {
            factor = (self.rect.width() / horizontal).min(factor);
        }
        let vertical = (r[0].height() + r[3].height()).max(r[1].height() + r[2].height());
        if vertical > self.rect.height() {
            factor = (self.rect.height() / vertical).min(factor);
        }
        debug_assert!(factor <= 1.);
        self.radii.Scale(factor);
        debug_assert!(self.IsRenderable());
    }
    pub fn IsRenderable(&self) -> bool {
        let r = self.radii.0;
        let (w, h) = (self.rect.width() * 1.0001, self.rect.height() * 1.0001);
        r[0].width() + r[1].width() <= w
            && r[3].width() + r[2].width() <= w
            && r[0].height() + r[3].height() <= h
            && r[1].height() + r[2].height() <= h
    }
    pub fn IntersectsQuad(&self, quad: &QuadF) -> bool {
        if !quad.IntersectsRect(&self.rect) {
            return false;
        }
        let (min, max) = quad.Extents();
        for index in [0, 1, 3, 2] {
            if self.radii.0[index].IsEmpty() {
                continue;
            }
            let corner = self.CornerRect(index);
            let (axes, center) = match index {
                0 => (
                    min.y() <= corner.bottom() && min.x() <= corner.right(),
                    PointF::new(corner.right(), corner.bottom()),
                ),
                1 => (
                    min.y() <= corner.bottom() && max.x() >= corner.x(),
                    PointF::new(corner.x(), corner.bottom()),
                ),
                3 => (
                    max.y() >= corner.y() && min.x() <= corner.right(),
                    PointF::new(corner.right(), corner.y()),
                ),
                _ => (
                    max.y() >= corner.y() && max.x() >= corner.x(),
                    corner.origin(),
                ),
            };
            if axes
                && quad.IntersectsRectPartial(&corner)
                && !quad.IntersectsEllipse(center, corner.size())
            {
                return false;
            }
        }
        true
    }
}
