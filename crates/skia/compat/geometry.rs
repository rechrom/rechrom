//! Local command/paint/path adapters. Their algorithms have explicit reference
//! records; these DTO conversions are not official Skia namespaces or APIs.
use crate::compat::commands::*;
use crate::raster::{Paint, Path, PathBuilder, PathSegment, Point, Rect};
use ttf_parser::OutlineBuilder;
pub(crate) fn rect(value: PaintRect) -> Option<Rect> {
    Rect::from_xywh(
        value.x as f32,
        value.y as f32,
        value.width as f32,
        value.height as f32,
    )
}

pub(crate) fn rect_commands(bounds: PaintRect) -> Option<Vec<PaintPathCommand>> {
    use crate::compat::commands::Offset;
    let l = bounds.x as f32;
    let t = bounds.y as f32;
    let r = l + bounds.width as f32;
    let b = t + bounds.height as f32;
    if ![l, t, r, b].into_iter().all(f32::is_finite) {
        return None;
    }
    let mut commands: Vec<_> = [(l, t), (r, t), (r, b), (l, b)]
        .into_iter()
        .enumerate()
        .map(|(i, (x, y))| PaintPathCommand {
            verb: if i == 0 {
                PaintPathVerb::kMoveTo
            } else {
                PaintPathVerb::kLineTo
            },
            point: Offset {
                x: f64::from(x),
                y: f64::from(y),
            },
            ..Default::default()
        })
        .collect();
    commands.push(PaintPathCommand {
        verb: PaintPathVerb::kClose,
        ..Default::default()
    });
    Some(commands)
}

pub(crate) fn oval_commands(bounds: PaintRect) -> Option<Vec<PaintPathCommand>> {
    use crate::compat::commands::Offset;
    let r = rect(bounds)?;
    let x = (r.left() + r.right()) * 0.5;
    let y = (r.top() + r.bottom()) * 0.5;
    let p = |x: f32, y: f32| Offset {
        x: f64::from(x),
        y: f64::from(y),
    };
    let mut commands = vec![PaintPathCommand {
        verb: PaintPathVerb::kMoveTo,
        point: p(r.right(), y),
        ..Default::default()
    }];
    for (control1, point) in [
        (p(r.right(), r.bottom()), p(x, r.bottom())),
        (p(r.left(), r.bottom()), p(r.left(), y)),
        (p(r.left(), r.top()), p(x, r.top())),
        (p(r.right(), r.top()), p(r.right(), y)),
    ] {
        commands.push(PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1,
            point,
            conic_weight: f64::from(std::f32::consts::FRAC_1_SQRT_2),
            ..Default::default()
        });
    }
    commands.push(PaintPathCommand {
        verb: PaintPathVerb::kClose,
        ..Default::default()
    });
    Some(commands)
}

pub(crate) fn rounded_rect_commands(
    bounds: PaintRect,
    radii: PaintCornerRadii,
    start: usize,
) -> Option<Vec<PaintPathCommand>> {
    use crate::compat::commands::Offset;
    let b = rect(bounds)?;
    let mut r = [
        radii.top_left,
        radii.top_right,
        radii.bottom_right,
        radii.bottom_left,
    ]
    .map(|r| (r.x.max(0.0) as f32, r.y.max(0.0) as f32));
    for p in &mut r {
        if p.0 == 0.0 || p.1 == 0.0 {
            *p = (0.0, 0.0);
        }
    }
    let mut scale = 1.0f32;
    for (extent, sum) in [
        (b.width(), r[0].0 + r[1].0),
        (b.width(), r[3].0 + r[2].0),
        (b.height(), r[0].1 + r[3].1),
        (b.height(), r[1].1 + r[2].1),
    ] {
        if sum > extent {
            scale = scale.min(extent / sum);
        }
    }
    for p in &mut r {
        p.0 *= scale;
        p.1 *= scale;
    }
    let points = [
        (b.left() + r[0].0, b.top()),
        (b.right() - r[1].0, b.top()),
        (b.right(), b.top() + r[1].1),
        (b.right(), b.bottom() - r[2].1),
        (b.right() - r[2].0, b.bottom()),
        (b.left() + r[3].0, b.bottom()),
        (b.left(), b.bottom() - r[3].1),
        (b.left(), b.top() + r[0].1),
    ];
    let controls = [
        (b.right(), b.top()),
        (b.right(), b.bottom()),
        (b.left(), b.bottom()),
        (b.left(), b.top()),
    ];
    let p = |p: (f32, f32)| Offset {
        x: p.0 as f64,
        y: p.1 as f64,
    };
    let mut commands = vec![PaintPathCommand {
        verb: PaintPathVerb::kMoveTo,
        point: p(points[start]),
        ..Default::default()
    }];
    for j in 0..4 {
        let corner = (start / 2 + j) % 4;
        let k = corner * 2;
        commands.push(PaintPathCommand {
            verb: PaintPathVerb::kLineTo,
            point: p(points[k + 1]),
            ..Default::default()
        });
        commands.push(PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: p(controls[corner]),
            point: p(points[(k + 2) % 8]),
            conic_weight: std::f32::consts::FRAC_1_SQRT_2 as f64,
            ..Default::default()
        });
    }
    commands.push(PaintPathCommand {
        verb: PaintPathVerb::kClose,
        ..Default::default()
    });
    Some(commands)
}

pub(crate) fn path_from_commands(commands: &[PaintPathCommand]) -> Option<Path> {
    let mut builder = PathBuilder::new();
    let mut current = (0.0f32, 0.0f32);
    let mut start = current;
    for command in commands {
        let point = command.point;
        match command.verb {
            PaintPathVerb::kMoveTo => builder.move_to(point.x as f32, point.y as f32),
            PaintPathVerb::kLineTo => builder.line_to(point.x as f32, point.y as f32),
            PaintPathVerb::kQuadraticTo => builder.quad_to(
                command.control1.x as f32,
                command.control1.y as f32,
                point.x as f32,
                point.y as f32,
            ),
            PaintPathVerb::kCubicTo => builder.cubic_to(
                command.control1.x as f32,
                command.control1.y as f32,
                command.control2.x as f32,
                command.control2.y as f32,
                point.x as f32,
                point.y as f32,
            ),
            PaintPathVerb::kConicTo => {
                if !(command.conic_weight as f32).is_finite() || command.conic_weight < 0.0 {
                    return None;
                }
                for q in crate::src::core::SkGeometry::conic_to_quads(
                    [
                        current,
                        (command.control1.x as f32, command.control1.y as f32),
                        (point.x as f32, point.y as f32),
                    ],
                    command.conic_weight as f32,
                ) {
                    builder.quad_to(q[1].0, q[1].1, q[2].0, q[2].1);
                }
            }
            PaintPathVerb::kClose => builder.close(),
        }
        current = if command.verb == PaintPathVerb::kClose {
            start
        } else {
            (point.x as f32, point.y as f32)
        };
        if command.verb == PaintPathVerb::kMoveTo {
            start = current;
        }
    }
    builder.finish()
}

pub(crate) fn commands_from_path(path: &Path) -> Vec<PaintPathCommand> {
    use crate::raster::PathSegment;
    let offset = |p: Point| crate::compat::commands::Offset {
        x: f64::from(p.x),
        y: f64::from(p.y),
    };
    path.segments()
        .map(|segment| {
            let mut c = PaintPathCommand::default();
            match segment {
                PathSegment::MoveTo(p) => {
                    c.verb = PaintPathVerb::kMoveTo;
                    c.point = offset(p);
                }
                PathSegment::LineTo(p) => {
                    c.verb = PaintPathVerb::kLineTo;
                    c.point = offset(p);
                }
                PathSegment::QuadTo(a, p) => {
                    c.verb = PaintPathVerb::kQuadraticTo;
                    c.control1 = offset(a);
                    c.point = offset(p);
                }
                PathSegment::CubicTo(a, b, p) => {
                    c.verb = PaintPathVerb::kCubicTo;
                    c.control1 = offset(a);
                    c.control2 = offset(b);
                    c.point = offset(p);
                }
                PathSegment::Close => c.verb = PaintPathVerb::kClose,
            }
            c
        })
        .collect()
}

pub(crate) fn color(value: Color) -> crate::raster::Color {
    crate::raster::Color::from_rgba(
        value.red.clamp(0.0, 1.0),
        value.green.clamp(0.0, 1.0),
        value.blue.clamp(0.0, 1.0),
        value.alpha.clamp(0.0, 1.0),
    )
    .expect("paint color is finite")
}

pub(crate) fn solid_paint(value: Color, antialias: bool) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color(color(value));
    paint.anti_alias = antialias;
    paint
}

pub(crate) fn skia_mul_div_255_round(a: u8, b: u8) -> u8 {
    let product = u32::from(a) * u32::from(b) + 128;
    ((product + (product >> 8)) >> 8) as u8
}

pub(crate) fn skia_shadow_premultiplied_color(color: Color) -> [u8; 4] {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    let alpha = channel(color.alpha);
    [
        skia_mul_div_255_round(channel(color.red), alpha),
        skia_mul_div_255_round(channel(color.green), alpha),
        skia_mul_div_255_round(channel(color.blue), alpha),
        alpha,
    ]
}

pub(crate) fn rounded_rect_path(bounds: PaintRect, radii: PaintCornerRadii) -> Option<Path> {
    let bounds = rect(bounds)?;
    let x = bounds.x();
    let y = bounds.y();
    let w = bounds.width();
    let h = bounds.height();
    let mut r = [
        (
            radii.top_left.x.max(0.0) as f32,
            radii.top_left.y.max(0.0) as f32,
        ),
        (
            radii.top_right.x.max(0.0) as f32,
            radii.top_right.y.max(0.0) as f32,
        ),
        (
            radii.bottom_right.x.max(0.0) as f32,
            radii.bottom_right.y.max(0.0) as f32,
        ),
        (
            radii.bottom_left.x.max(0.0) as f32,
            radii.bottom_left.y.max(0.0) as f32,
        ),
    ];
    for (rx, ry) in &mut r {
        if *rx == 0.0 || *ry == 0.0 {
            *rx = 0.0;
            *ry = 0.0;
        }
    }
    let mut scale = 1.0_f32;
    for (extent, total) in [
        (w, r[0].0 + r[1].0),
        (w, r[3].0 + r[2].0),
        (h, r[0].1 + r[3].1),
        (h, r[1].1 + r[2].1),
    ] {
        if total > extent {
            scale = scale.min(extent / total);
        }
    }
    for (rx, ry) in &mut r {
        *rx *= scale;
        *ry *= scale;
    }
    if r.iter().all(|&(rx, ry)| rx == 0.0 || ry == 0.0) {
        return Some(PathBuilder::from_rect(bounds));
    }
    const K: f32 = 0.552_284_8;
    let mut path = PathBuilder::new();
    path.move_to(x + r[0].0, y);
    path.line_to(x + w - r[1].0, y);
    path.cubic_to(
        x + w - r[1].0 * (1.0 - K),
        y,
        x + w,
        y + r[1].1 * (1.0 - K),
        x + w,
        y + r[1].1,
    );
    path.line_to(x + w, y + h - r[2].1);
    path.cubic_to(
        x + w,
        y + h - r[2].1 * (1.0 - K),
        x + w - r[2].0 * (1.0 - K),
        y + h,
        x + w - r[2].0,
        y + h,
    );
    path.line_to(x + r[3].0, y + h);
    path.cubic_to(
        x + r[3].0 * (1.0 - K),
        y + h,
        x,
        y + h - r[3].1 * (1.0 - K),
        x,
        y + h - r[3].1,
    );
    path.line_to(x, y + r[0].1);
    path.cubic_to(
        x,
        y + r[0].1 * (1.0 - K),
        x + r[0].0 * (1.0 - K),
        y,
        x + r[0].0,
        y,
    );
    path.close();
    path.finish()
}

pub(crate) struct GlyphPathBuilder {
    pub(crate) path: PathBuilder,
    pub(crate) origin_x: f32,
    pub(crate) origin_y: f32,
    pub(crate) scale: f32,
    pub(crate) synthetic_italic: bool,
}

impl GlyphPathBuilder {
    pub(crate) fn new(origin_x: f32, origin_y: f32, scale: f32, synthetic_italic: bool) -> Self {
        Self {
            path: PathBuilder::new(),
            origin_x,
            origin_y,
            scale,
            synthetic_italic,
        }
    }

    fn point(&self, x: f32, y: f32) -> (f32, f32) {
        let italic_shift = if self.synthetic_italic { 0.25 * y } else { 0.0 };
        (
            self.origin_x + (x + italic_shift) * self.scale,
            self.origin_y - y * self.scale,
        )
    }
}

impl OutlineBuilder for GlyphPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.point(x, y);
        self.path.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.point(x, y);
        self.path.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (x1, y1) = self.point(x1, y1);
        let (x, y) = self.point(x, y);
        self.path.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (x1, y1) = self.point(x1, y1);
        let (x2, y2) = self.point(x2, y2);
        let (x, y) = self.point(x, y);
        self.path.cubic_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.path.close();
    }
}
