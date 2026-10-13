#![allow(non_snake_case)]

use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb};
use std::ffi::c_char;

unsafe extern "C" {
    fn strtod(input: *const c_char, end: *mut *mut c_char) -> f64;
    fn isalpha(c: std::ffi::c_int) -> std::ffi::c_int;
    fn isspace(c: std::ffi::c_int) -> std::ffi::c_int;
    fn isdigit(c: std::ffi::c_int) -> std::ffi::c_int;
    fn islower(c: std::ffi::c_int) -> std::ffi::c_int;
    fn toupper(c: std::ffi::c_int) -> std::ffi::c_int;
}

fn IsAlpha(byte: u8) -> bool {
    unsafe { isalpha(byte as std::ffi::c_int) != 0 }
}
fn IsSpace(byte: u8) -> bool {
    unsafe { isspace(byte as std::ffi::c_int) != 0 }
}
fn IsDigit(byte: u8) -> bool {
    unsafe { isdigit(byte as std::ffi::c_int) != 0 }
}
fn IsLower(byte: u8) -> bool {
    unsafe { islower(byte as std::ffi::c_int) != 0 }
}
fn ToUpper(byte: u8) -> u8 {
    unsafe { toupper(byte as std::ffi::c_int) as u8 }
}

// cpp: dom/svg_path_parser.h:14-21
#[derive(Clone, Debug)]
pub struct ParsedSVGPath {
    pub commands: Vec<PaintPathCommand>,
    pub flattened_points: Vec<Offset>,
    pub bounds_offset: Offset,
    pub bounds_width: f64,
    pub bounds_height: f64,
    pub single_subpath: bool,
}

impl Default for ParsedSVGPath {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            flattened_points: Vec::new(),
            bounds_offset: Offset::default(),
            bounds_width: 0.0,
            bounds_height: 0.0,
            single_subpath: true,
        }
    }
}

// cpp: dom/svg_path_parser.cc:14-18
fn Add(a: Offset, b: Offset) -> Offset {
    Offset {
        x: a.x + b.x,
        y: a.y + b.y,
    }
}
fn Subtract(a: Offset, b: Offset) -> Offset {
    Offset {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}
fn Scale(point: Offset, scale: f64) -> Offset {
    Offset {
        x: point.x * scale,
        y: point.y * scale,
    }
}

// cpp: dom/svg_path_parser.cc:20-28
fn DistanceToLine(point: Offset, start: Offset, end: Offset) -> f64 {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length = dx.hypot(dy);
    if length == 0.0 {
        return (point.x - start.x).hypot(point.y - start.y);
    }
    (dy * point.x - dx * point.y + end.x * start.y - end.y * start.x).abs() / length
}

// cpp: dom/svg_path_parser.cc:30-46
fn FlattenQuadratic(
    start: Offset,
    control: Offset,
    end: Offset,
    tolerance: f64,
    depth: u32,
    output: &mut Vec<Offset>,
) {
    if depth >= 12 || DistanceToLine(control, start, end) <= tolerance {
        output.push(end);
        return;
    }
    let start_control = Scale(Add(start, control), 0.5);
    let control_end = Scale(Add(control, end), 0.5);
    let middle = Scale(Add(start_control, control_end), 0.5);
    FlattenQuadratic(start, start_control, middle, tolerance, depth + 1, output);
    FlattenQuadratic(middle, control_end, end, tolerance, depth + 1, output);
}

// cpp: dom/svg_path_parser.cc:48-69
fn FlattenCubic(
    start: Offset,
    control1: Offset,
    control2: Offset,
    end: Offset,
    tolerance: f64,
    depth: u32,
    output: &mut Vec<Offset>,
) {
    if depth >= 12 {
        output.push(end);
        return;
    }
    let first_distance = DistanceToLine(control1, start, end);
    let second_distance = DistanceToLine(control2, start, end);
    let maximum_distance = if first_distance < second_distance {
        second_distance
    } else {
        first_distance
    };
    if maximum_distance <= tolerance {
        output.push(end);
        return;
    }
    let a = Scale(Add(start, control1), 0.5);
    let b = Scale(Add(control1, control2), 0.5);
    let c = Scale(Add(control2, end), 0.5);
    let d = Scale(Add(a, b), 0.5);
    let e = Scale(Add(b, c), 0.5);
    let middle = Scale(Add(d, e), 0.5);
    FlattenCubic(start, a, d, middle, tolerance, depth + 1, output);
    FlattenCubic(middle, e, c, end, tolerance, depth + 1, output);
}

fn ConicPoint(start: Offset, control: Offset, end: Offset, weight: f64, t: f64) -> Offset {
    let u = 1.0 - t;
    let denominator = u * u + 2.0 * weight * u * t + t * t;
    Offset {
        x: (u * u * start.x + 2.0 * weight * u * t * control.x + t * t * end.x) / denominator,
        y: (u * u * start.y + 2.0 * weight * u * t * control.y + t * t * end.y) / denominator,
    }
}

fn FlattenConicRange(
    start: Offset,
    control: Offset,
    end: Offset,
    weight: f64,
    t0: f64,
    p0: Offset,
    t1: f64,
    p1: Offset,
    tolerance: f64,
    depth: u32,
    output: &mut Vec<Offset>,
) {
    let tm = (t0 + t1) * 0.5;
    let middle = ConicPoint(start, control, end, weight, tm);
    if depth >= 12 || DistanceToLine(middle, p0, p1) <= tolerance {
        output.push(p1);
        return;
    }
    FlattenConicRange(
        start,
        control,
        end,
        weight,
        t0,
        p0,
        tm,
        middle,
        tolerance,
        depth + 1,
        output,
    );
    FlattenConicRange(
        start,
        control,
        end,
        weight,
        tm,
        middle,
        t1,
        p1,
        tolerance,
        depth + 1,
        output,
    );
}

// cpp: dom/svg_path_parser.cc:71-76
// cpp: dom/svg_path_parser.cc:396-407
struct Parser {
    storage: Vec<u8>,
    cursor: usize,
    tolerance: f64,
    result: ParsedSVGPath,
    current: Offset,
    subpath_start: Offset,
    previous_cubic_control: Offset,
    previous_quadratic_control: Offset,
    have_subpath: bool,
    previous_command: u8,
}

impl Parser {
    fn new(input: &[u8], tolerance: f64) -> Self {
        let mut storage = input.to_vec();
        storage.push(0);
        Self {
            storage,
            cursor: 0,
            tolerance,
            result: ParsedSVGPath::default(),
            current: Offset::default(),
            subpath_start: Offset::default(),
            previous_cubic_control: Offset::default(),
            previous_quadratic_control: Offset::default(),
            have_subpath: false,
            previous_command: 0,
        }
    }

    fn end(&self) -> usize {
        self.storage.len() - 1
    }

    // cpp: dom/svg_path_parser.cc:77-106
    fn Parse(mut self) -> Option<ParsedSVGPath> {
        let mut command = 0u8;
        loop {
            self.SkipSeparators();
            if self.cursor == self.end() {
                break;
            }
            if IsAlpha(self.storage[self.cursor]) {
                command = self.storage[self.cursor];
                self.cursor += 1;
            } else if command == 0 {
                return None;
            }
            let upper = ToUpper(command);
            if upper == b'Z' {
                if !self.have_subpath {
                    return None;
                }
                self.AddClose();
                command = 0;
                continue;
            }
            loop {
                self.ParseArguments(command)?;
                if upper == b'M' {
                    // Every pair after a moveto is an implicit lineto and
                    // retains the original absolute/relative mode. Once the
                    // command has become `l`/`L`, keep that case for every
                    // remaining pair instead of turning a third relative pair
                    // into an absolute line.
                    command = if IsLower(command) { b'l' } else { b'L' };
                }
                if !self.HasNumber() {
                    break;
                }
            }
        }
        if self.result.commands.is_empty() {
            return None;
        }
        self.ComputeBounds();
        Some(self.result)
    }

    // cpp: dom/svg_path_parser.cc:109-121
    fn SkipSeparators(&mut self) {
        while self.cursor < self.end()
            && (IsSpace(self.storage[self.cursor]) || self.storage[self.cursor] == b',')
        {
            self.cursor += 1;
        }
    }
    fn HasNumber(&mut self) -> bool {
        self.SkipSeparators();
        self.cursor < self.end()
            && (matches!(self.storage[self.cursor], b'+' | b'-' | b'.')
                || IsDigit(self.storage[self.cursor]))
    }

    // cpp: dom/svg_path_parser.cc:123-146
    fn Number(&mut self) -> Option<f64> {
        self.SkipSeparators();
        if self.cursor == self.end() {
            return None;
        }
        let begin = unsafe { self.storage.as_ptr().add(self.cursor).cast::<c_char>() };
        let mut parsed_end = std::ptr::null_mut();
        let value = unsafe { strtod(begin, &mut parsed_end) };
        if parsed_end == begin.cast_mut() || !value.is_finite() {
            return None;
        }
        self.cursor += unsafe { parsed_end.offset_from(begin) } as usize;
        Some(value)
    }
    fn Flag(&mut self) -> Option<bool> {
        self.SkipSeparators();
        if self.cursor == self.end() || !matches!(self.storage[self.cursor], b'0' | b'1') {
            return None;
        }
        let value = self.storage[self.cursor] == b'1';
        self.cursor += 1;
        Some(value)
    }
    fn Point(&mut self, relative: bool) -> Option<Offset> {
        let x = self.Number();
        let y = self.Number();
        let (Some(x), Some(y)) = (x, y) else {
            return None;
        };
        let point = Offset { x, y };
        Some(if relative {
            Add(self.current, point)
        } else {
            point
        })
    }

    // cpp: dom/svg_path_parser.cc:148-209
    fn ResetSmoothControls(&mut self, command: u8) {
        self.previous_command = command;
    }
    fn AddMove(&mut self, point: Offset) {
        // Blink's SVGPathBuilder resolves relative coordinates before forwarding
        // to SkPathBuilder::moveTo. Consecutive moves replace the last point;
        // the abandoned move has no segment and must not reach either the
        // retained commands or the flattened text-path/bounds projection.
        // A move following a close or any drawing verb starts another contour.
        if self
            .result
            .commands
            .last()
            .is_some_and(|last| last.verb == PaintPathVerb::kMoveTo)
        {
            self.result.commands.last_mut().unwrap().point = point;
            *self.result.flattened_points.last_mut().unwrap() = point;
        } else {
            if self.have_subpath {
                self.result.single_subpath = false;
            }
            self.result.commands.push(PaintPathCommand {
                verb: PaintPathVerb::kMoveTo,
                point,
                ..PaintPathCommand::default()
            });
            self.result.flattened_points.push(point);
        }
        self.current = point;
        self.subpath_start = point;
        self.have_subpath = true;
        self.ResetSmoothControls(b'M');
    }
    fn AddLine(&mut self, point: Offset) {
        self.result.commands.push(PaintPathCommand {
            verb: PaintPathVerb::kLineTo,
            point,
            ..PaintPathCommand::default()
        });
        if self.result.flattened_points.is_empty() {
            self.result.flattened_points.push(self.current);
        }
        self.result.flattened_points.push(point);
        self.current = point;
        self.ResetSmoothControls(b'L');
    }
    fn AddQuadratic(&mut self, control: Offset, point: Offset, command: u8) {
        self.result.commands.push(PaintPathCommand {
            verb: PaintPathVerb::kQuadraticTo,
            control1: control,
            point,
            ..PaintPathCommand::default()
        });
        if self.result.flattened_points.is_empty() {
            self.result.flattened_points.push(self.current);
        }
        FlattenQuadratic(
            self.current,
            control,
            point,
            self.tolerance,
            0,
            &mut self.result.flattened_points,
        );
        self.current = point;
        self.previous_quadratic_control = control;
        self.ResetSmoothControls(command);
    }
    fn AddCubic(&mut self, control1: Offset, control2: Offset, point: Offset, command: u8) {
        self.result.commands.push(PaintPathCommand {
            verb: PaintPathVerb::kCubicTo,
            control1,
            control2,
            point,
            ..PaintPathCommand::default()
        });
        if self.result.flattened_points.is_empty() {
            self.result.flattened_points.push(self.current);
        }
        FlattenCubic(
            self.current,
            control1,
            control2,
            point,
            self.tolerance,
            0,
            &mut self.result.flattened_points,
        );
        self.current = point;
        self.previous_cubic_control = control2;
        self.ResetSmoothControls(command);
    }
    fn AddConic(&mut self, control: Offset, point: Offset, weight: f64, command: u8) {
        self.result.commands.push(PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: control,
            point,
            conic_weight: weight,
            ..PaintPathCommand::default()
        });
        if self.result.flattened_points.is_empty() {
            self.result.flattened_points.push(self.current);
        }
        FlattenConicRange(
            self.current,
            control,
            point,
            weight,
            0.0,
            self.current,
            1.0,
            point,
            self.tolerance,
            0,
            &mut self.result.flattened_points,
        );
        self.current = point;
        self.ResetSmoothControls(command);
    }
    fn AddClose(&mut self) {
        self.result.commands.push(PaintPathCommand {
            verb: PaintPathVerb::kClose,
            ..PaintPathCommand::default()
        });
        if self.current != self.subpath_start {
            self.result.flattened_points.push(self.subpath_start);
        }
        self.current = self.subpath_start;
        self.ResetSmoothControls(b'Z');
    }

    // cpp: platform/geometry/path_builder.cc:185-197
    // cpp: skia/src/core/SkPathBuilder.cpp:568-694
    fn AddArc(
        &mut self,
        rx: f64,
        ry: f64,
        rotation: f64,
        large_arc: bool,
        sweep: bool,
        point: Offset,
    ) {
        // Blink sends SVG arcs to SkPathBuilder::arcTo. Keep all arc
        // construction in SkScalar precision and retain its rational conics;
        // a generic <=90-degree cubic approximation changes both bounds and
        // edge coverage for ordinary SVG icons.
        let source = (self.current.x as f32, self.current.y as f32);
        let target = (point.x as f32, point.y as f32);
        let target_offset = Offset {
            x: f64::from(target.0),
            y: f64::from(target.1),
        };
        let mut rx = (rx as f32).abs();
        let mut ry = (ry as f32).abs();
        if rx == 0.0 || ry == 0.0 {
            self.AddLine(target_offset);
            return;
        }
        if source == target {
            // SkPathBuilder emits the zero-length line for continuity.
            self.AddLine(target_offset);
            return;
        }

        let angle = rotation as f32 * std::f32::consts::PI / 180.0;
        let (sin_angle, cos_angle) = angle.sin_cos();
        let midpoint = ((source.0 - target.0) * 0.5, (source.1 - target.1) * 0.5);
        let transformed_midpoint = (
            cos_angle * midpoint.0 + sin_angle * midpoint.1,
            -sin_angle * midpoint.0 + cos_angle * midpoint.1,
        );
        let mut square_rx = rx * rx;
        let mut square_ry = ry * ry;
        let square_x = transformed_midpoint.0 * transformed_midpoint.0;
        let square_y = transformed_midpoint.1 * transformed_midpoint.1;
        let mut radii_scale = square_x / square_rx + square_y / square_ry;
        if radii_scale > 1.0 {
            radii_scale = radii_scale.sqrt();
            rx *= radii_scale;
            ry *= radii_scale;
            square_rx = rx * rx;
            square_ry = ry * ry;
        }
        debug_assert!(square_rx.is_finite() && square_ry.is_finite());

        // Skia builds a scale matrix from the reciprocals and then maps the
        // points through it. Multiplying by a rounded reciprocal is not
        // bit-equivalent to dividing every coordinate here; the difference is
        // visible on antialiased SVG edges. Preserve SkMatrix's operation
        // order so the retained path is identical to Chromium's.
        let inverse_rx = 1.0_f32 / rx;
        let inverse_ry = 1.0_f32 / ry;
        let normalize = |point: (f32, f32)| {
            let rotated = (
                cos_angle * point.0 + sin_angle * point.1,
                -sin_angle * point.0 + cos_angle * point.1,
            );
            (rotated.0 * inverse_rx, rotated.1 * inverse_ry)
        };
        let mut unit_start = normalize(source);
        let mut unit_end = normalize(target);
        let mut delta = (unit_end.0 - unit_start.0, unit_end.1 - unit_start.1);
        let distance = delta.0 * delta.0 + delta.1 * delta.1;
        let mut center_scale = (0.0_f32.max(1.0 / distance - 0.25)).sqrt();
        let counter_clockwise = !sweep;
        if counter_clockwise != large_arc {
            center_scale = -center_scale;
        }
        delta.0 *= center_scale;
        delta.1 *= center_scale;
        let center = (
            (unit_start.0 + unit_end.0) * 0.5 - delta.1,
            (unit_start.1 + unit_end.1) * 0.5 + delta.0,
        );
        unit_start.0 -= center.0;
        unit_start.1 -= center.1;
        unit_end.0 -= center.0;
        unit_end.1 -= center.1;
        let theta1 = unit_start.1.atan2(unit_start.0);
        let theta2 = unit_end.1.atan2(unit_end.0);
        let mut theta_arc = theta2 - theta1;
        if theta_arc < 0.0 && sweep {
            theta_arc += std::f32::consts::TAU;
        } else if theta_arc > 0.0 && !sweep {
            theta_arc -= std::f32::consts::TAU;
        }
        if theta_arc.abs() < std::f32::consts::PI / 1_000_000.0 {
            self.AddLine(target_offset);
            return;
        }
        let segments = (theta_arc.abs() / (std::f32::consts::TAU / 3.0)).ceil() as u32;
        let theta_width = theta_arc / segments as f32;
        let tangent = (0.5 * theta_width).tan();
        if !tangent.is_finite() {
            self.ResetSmoothControls(b'A');
            return;
        }
        let weight = (0.5 + theta_width.cos() * 0.5).sqrt();
        let expect_integers = (std::f32::consts::FRAC_PI_2 - theta_width.abs()).abs()
            <= 1.0 / 4096.0
            && rx == rx.floor()
            && ry == ry.floor()
            && target.0 == target.0.floor()
            && target.1 == target.1.floor();
        let denormalize = |point: (f32, f32)| {
            let scaled = (point.0 * rx, point.1 * ry);
            (
                cos_angle * scaled.0 - sin_angle * scaled.1,
                sin_angle * scaled.0 + cos_angle * scaled.1,
            )
        };
        let snap_trig = |value: f32| {
            if value.abs() <= 1.0 / 65_536.0 {
                0.0
            } else {
                value
            }
        };
        let mut start_theta = theta1;
        for index in 0..segments {
            let end_theta = start_theta + theta_width;
            let sin_end = snap_trig(end_theta.sin());
            let cos_end = snap_trig(end_theta.cos());
            let endpoint_unit = (cos_end + center.0, sin_end + center.1);
            let control_unit = (
                endpoint_unit.0 + tangent * sin_end,
                endpoint_unit.1 - tangent * cos_end,
            );
            let mut control = denormalize(control_unit);
            let mut endpoint = denormalize(endpoint_unit);
            if expect_integers {
                control = (control.0.round(), control.1.round());
                endpoint = (endpoint.0.round(), endpoint.1.round());
            }
            let end = if index + 1 == segments {
                target_offset
            } else {
                Offset {
                    x: f64::from(endpoint.0),
                    y: f64::from(endpoint.1),
                }
            };
            self.AddConic(
                Offset {
                    x: f64::from(control.0),
                    y: f64::from(control.1),
                },
                end,
                f64::from(weight),
                b'A',
            );
            start_theta = end_theta;
        }
        self.ResetSmoothControls(b'A');
    }

    // cpp: dom/svg_path_parser.cc:295-377
    fn ParseArguments(&mut self, command: u8) -> Option<()> {
        let relative = IsLower(command);
        let upper = ToUpper(command);
        if upper != b'M' && !self.have_subpath {
            return None;
        }
        match upper {
            b'M' => {
                let point = self.Point(relative)?;
                self.AddMove(point);
            }
            b'L' => {
                let point = self.Point(relative)?;
                self.AddLine(point);
            }
            b'H' => {
                let x = self.Number()?;
                self.AddLine(Offset {
                    x: if relative { self.current.x + x } else { x },
                    y: self.current.y,
                });
            }
            b'V' => {
                let y = self.Number()?;
                self.AddLine(Offset {
                    x: self.current.x,
                    y: if relative { self.current.y + y } else { y },
                });
            }
            b'C' => {
                let c1 = self.Point(relative);
                let c2 = self.Point(relative);
                let point = self.Point(relative);
                let (Some(c1), Some(c2), Some(point)) = (c1, c2, point) else {
                    return None;
                };
                self.AddCubic(c1, c2, point, b'C');
            }
            b'S' => {
                let c2 = self.Point(relative);
                let point = self.Point(relative);
                let (Some(c2), Some(point)) = (c2, point) else {
                    return None;
                };
                let c1 = if matches!(self.previous_command, b'C' | b'S') {
                    Subtract(Scale(self.current, 2.0), self.previous_cubic_control)
                } else {
                    self.current
                };
                self.AddCubic(c1, c2, point, b'S');
            }
            b'Q' => {
                let control = self.Point(relative);
                let point = self.Point(relative);
                let (Some(control), Some(point)) = (control, point) else {
                    return None;
                };
                self.AddQuadratic(control, point, b'Q');
            }
            b'T' => {
                let point = self.Point(relative)?;
                let control = if matches!(self.previous_command, b'Q' | b'T') {
                    Subtract(Scale(self.current, 2.0), self.previous_quadratic_control)
                } else {
                    self.current
                };
                self.AddQuadratic(control, point, b'T');
            }
            b'A' => {
                let rx = self.Number();
                let ry = self.Number();
                let rotation = self.Number();
                let large = self.Flag();
                let sweep = self.Flag();
                let point = self.Point(relative);
                let (Some(rx), Some(ry), Some(rotation), Some(large), Some(sweep), Some(point)) =
                    (rx, ry, rotation, large, sweep, point)
                else {
                    return None;
                };
                if rx < 0.0 || ry < 0.0 {
                    return None;
                }
                self.AddArc(rx, ry, rotation, large, sweep, point);
            }
            _ => return None,
        }
        Some(())
    }

    // cpp: dom/svg_path_parser.cc:379-394
    fn ComputeBounds(&mut self) {
        let Some(first) = self.result.flattened_points.first() else {
            return;
        };
        let mut left = first.x;
        let mut right = left;
        let mut top = first.y;
        let mut bottom = top;
        for point in &self.result.flattened_points {
            if point.x < left {
                left = point.x;
            }
            if right < point.x {
                right = point.x;
            }
            if point.y < top {
                top = point.y;
            }
            if bottom < point.y {
                bottom = point.y;
            }
        }
        self.result.bounds_offset = Offset { x: left, y: top };
        self.result.bounds_width = right - left;
        self.result.bounds_height = bottom - top;
    }
}

// cpp: dom/svg_path_parser.h:23-24
// cpp: dom/svg_path_parser.cc:411-415
// C++ string_view accepts arbitrary bytes, so the source-level entry uses &[u8].
pub fn ParseSVGPathBytes(path: &[u8], tolerance: f64) -> Option<ParsedSVGPath> {
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return None;
    }
    Parser::new(path, tolerance).Parse()
}

pub fn ParseSVGPath(path: &str, tolerance: f64) -> Option<ParsedSVGPath> {
    ParseSVGPathBytes(path.as_bytes(), tolerance)
}

// cpp: dom/svg_path_parser.h:23-24
// Rust has no default parameters; this preserves the one-argument C++ call.
pub fn ParseSVGPathDefault(path: &str) -> Option<ParsedSVGPath> {
    ParseSVGPath(path, 0.01)
}
