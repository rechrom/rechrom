#![allow(non_snake_case)]

use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb};
use std::ffi::c_char;

unsafe extern "C" {
    fn strtod(input: *const c_char, end: *mut *mut c_char) -> f64;
    fn remainder(x: f64, y: f64) -> f64;
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
                    command = if command == b'm' { b'l' } else { b'L' };
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
        if self.have_subpath {
            self.result.single_subpath = false;
        }
        self.result.commands.push(PaintPathCommand {
            verb: PaintPathVerb::kMoveTo,
            point,
            ..PaintPathCommand::default()
        });
        self.result.flattened_points.push(point);
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

    // cpp: dom/svg_path_parser.cc:211-293
    fn AddArc(
        &mut self,
        mut rx: f64,
        mut ry: f64,
        rotation: f64,
        large_arc: bool,
        sweep: bool,
        point: Offset,
    ) {
        rx = rx.abs();
        ry = ry.abs();
        if rx == 0.0 || ry == 0.0 || point == self.current {
            if point != self.current {
                self.AddLine(point);
            } else {
                self.ResetSmoothControls(b'A');
            }
            return;
        }
        let phi = unsafe { remainder(rotation, 360.0) } * std::f64::consts::PI / 180.0;
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();
        let dx = (self.current.x - point.x) / 2.0;
        let dy = (self.current.y - point.y) / 2.0;
        let x1 = cos_phi * dx + sin_phi * dy;
        let y1 = -sin_phi * dx + cos_phi * dy;
        let lambda = x1 * x1 / (rx * rx) + y1 * y1 / (ry * ry);
        if lambda > 1.0 {
            let scale = lambda.sqrt();
            rx *= scale;
            ry *= scale;
        }
        let rx2 = rx * rx;
        let ry2 = ry * ry;
        // The optimized C++ arm64 expression contracts these subtractions.
        // Preserve its rounding near diameter endpoints: changing the tiny
        // center offset can change ceil(sweep_angle / (pi / 2)) and the path.
        let numerator = 0.0_f64.max((-ry2 * x1).mul_add(x1, (-rx2 * y1).mul_add(y1, rx2 * ry2)));
        let denominator = rx2 * y1 * y1 + ry2 * x1 * x1;
        let sign = if large_arc == sweep { -1.0 } else { 1.0 };
        let coefficient = if denominator == 0.0 {
            0.0
        } else {
            sign * (numerator / denominator).sqrt()
        };
        let cx1 = coefficient * (rx * y1 / ry);
        let cy1 = coefficient * (-ry * x1 / rx);
        let center_x = cos_phi * cx1 - sin_phi * cy1 + (self.current.x + point.x) / 2.0;
        let center_y = sin_phi * cx1 + cos_phi * cy1 + (self.current.y + point.y) / 2.0;
        let angle =
            |ux: f64, uy: f64, vx: f64, vy: f64| (ux * vy - uy * vx).atan2(ux * vx + uy * vy);
        let ux = (x1 - cx1) / rx;
        let uy = (y1 - cy1) / ry;
        let vx = (-x1 - cx1) / rx;
        let vy = (-y1 - cy1) / ry;
        let start_angle = angle(1.0, 0.0, ux, uy);
        let mut sweep_angle = angle(ux, uy, vx, vy);
        let tau = 2.0 * std::f64::consts::PI;
        if !sweep && sweep_angle > 0.0 {
            sweep_angle -= tau;
        }
        if sweep && sweep_angle < 0.0 {
            sweep_angle += tau;
        }
        let segments = 1u32.max((sweep_angle.abs() / (std::f64::consts::PI / 2.0)).ceil() as u32);
        let step = sweep_angle / segments as f64;
        let ellipse_point = |theta: f64| Offset {
            x: center_x + rx * cos_phi * theta.cos() - ry * sin_phi * theta.sin(),
            y: center_y + rx * sin_phi * theta.cos() + ry * cos_phi * theta.sin(),
        };
        let ellipse_derivative = |theta: f64| Offset {
            x: -rx * cos_phi * theta.sin() - ry * sin_phi * theta.cos(),
            y: -rx * sin_phi * theta.sin() + ry * cos_phi * theta.cos(),
        };
        for index in 0..segments {
            let a0 = start_angle + step * index as f64;
            let a1 = a0 + step;
            let alpha = 4.0 / 3.0 * (step / 4.0).tan();
            let p0 = ellipse_point(a0);
            let mut p1 = ellipse_point(a1);
            if index + 1 == segments {
                p1 = point;
            }
            let control1 = Add(p0, Scale(ellipse_derivative(a0), alpha));
            let control2 = Subtract(p1, Scale(ellipse_derivative(a1), alpha));
            self.AddCubic(control1, control2, p1, b'A');
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
