#![allow(non_snake_case)]

use super::border_radius::{Length, SplitTopLevel};
use super::shadow::SplitWhitespace;
use layoutng_assembly::css_color_parser::ParseCSSColor;
use layoutng_assembly::internal::layout_input::Offset;
use layoutng_assembly::internal::paint_input::{
    PaintColorStop, PaintShader, PaintShaderKind, PaintSpreadMethod,
};

// cpp: style_resolver/style_resolver.cc:2556-2599
fn ParseLinearGradientDirection(input: &str) -> Option<(Offset, Offset)> {
    let value = input.trim().to_ascii_lowercase();
    let (start, end) = match value.as_str() {
        "to bottom" | "180deg" => ((0.5, 0.0), (0.5, 1.0)),
        "to top" | "0deg" | "360deg" => ((0.5, 1.0), (0.5, 0.0)),
        "to right" | "90deg" => ((0.0, 0.5), (1.0, 0.5)),
        "to left" | "270deg" | "-90deg" => ((1.0, 0.5), (0.0, 0.5)),
        "to top right" | "to right top" => ((0.0, 1.0), (1.0, 0.0)),
        "to bottom right" | "to right bottom" => ((0.0, 0.0), (1.0, 1.0)),
        "to bottom left" | "to left bottom" => ((1.0, 0.0), (0.0, 1.0)),
        "to top left" | "to left top" => ((1.0, 1.0), (0.0, 0.0)),
        _ => return None,
    };
    Some((
        Offset {
            x: start.0,
            y: start.1,
        },
        Offset { x: end.0, y: end.1 },
    ))
}

fn AngleDegrees(input: &str) -> Option<f64> {
    let text = input.trim().to_ascii_lowercase();
    if let Some(x) = text.strip_suffix("deg") {
        return x.trim().parse().ok();
    }
    if let Some(x) = text.strip_suffix("turn") {
        return x.trim().parse::<f64>().ok().map(|v| v * 360.0);
    }
    if let Some(x) = text.strip_suffix("rad") {
        return x.trim().parse::<f64>().ok().map(f64::to_degrees);
    }
    None
}

// cpp: style_resolver/style_resolver.cc:2701-2775
fn ParseGradientColorStops(arguments: &[String], font_size: f64) -> Option<Vec<PaintColorStop>> {
    #[derive(Clone, Copy, Default)]
    struct Position {
        fraction: f64,
        length: f64,
        specified: bool,
    }
    let mut stops = Vec::new();
    let mut positions = Vec::new();
    for argument in arguments {
        let parts = SplitWhitespace(argument);
        if parts.is_empty() || parts.len() > 3 {
            return None;
        }
        let color = ParseCSSColor(&parts[0])?;
        if parts.len() == 1 {
            stops.push(PaintColorStop {
                color,
                ..Default::default()
            });
            positions.push(Position::default());
            continue;
        }
        for part in &parts[1..] {
            let mut position = Position::default();
            if let Some(percent) = part.strip_suffix('%').and_then(|v| v.parse::<f64>().ok()) {
                position.fraction = percent / 100.0;
                position.specified = true;
            } else if let Some(length) = Length(part, font_size) {
                position.length = length;
                position.specified = true;
            }
            if !position.specified || position.length < 0.0 {
                return None;
            }
            stops.push(PaintColorStop {
                color,
                ..Default::default()
            });
            positions.push(position);
        }
    }
    if stops.len() < 2 {
        return None;
    }
    if !positions[0].specified {
        positions[0].specified = true;
    }
    let last = positions.len() - 1;
    if !positions[last].specified {
        positions[last].fraction = 1.0;
        positions[last].specified = true;
    }
    let mut previous = positions[0];
    for p in &mut positions[1..] {
        if p.specified {
            if p.length == 0.0 && previous.length == 0.0 {
                p.fraction = p.fraction.max(previous.fraction);
            }
            previous = *p;
        }
    }
    let mut defined = 0;
    while defined + 1 < positions.len() {
        let next = (defined + 1..positions.len()).find(|&i| positions[i].specified)?;
        let first = positions[defined];
        let last = positions[next];
        for (i, p) in positions
            .iter_mut()
            .enumerate()
            .take(next)
            .skip(defined + 1)
        {
            let progress = (i - defined) as f64 / (next - defined) as f64;
            p.fraction = first.fraction + (last.fraction - first.fraction) * progress;
            p.length = first.length + (last.length - first.length) * progress;
            p.specified = true;
        }
        defined = next;
    }
    for (stop, position) in stops.iter_mut().zip(positions) {
        stop.offset = position.fraction;
        stop.offset_length = position.length;
    }
    Some(stops)
}

// cpp: style_resolver/style_resolver.cc:2777-2803
fn NormalizeExtendedGradientDomain(stops: &mut [PaintColorStop]) -> Option<(f64, f64)> {
    let start = 0.0_f64.min(stops.first()?.offset);
    let end = 1.0_f64.max(stops.last()?.offset);
    if start == 0.0 && end == 1.0 {
        return Some((start, end));
    }
    if stops.iter().any(|s| s.offset_length != 0.0) {
        return None;
    }
    for stop in stops {
        stop.offset = (stop.offset - start) / (end - start);
    }
    Some((start, end))
}

// cpp: style_resolver/style_resolver.cc:2608-2699
pub(crate) fn ParseLinearGradient(input: &str, font_size: f64) -> Option<PaintShader> {
    let value = input.trim();
    let lower = value.to_ascii_lowercase();
    let repeating = lower.starts_with("repeating-linear-gradient(");
    if !repeating && !lower.starts_with("linear-gradient(") {
        return None;
    }
    let open = value.find('(')?;
    let inner = value.get(open + 1..value.len().checked_sub(1)?)?;
    if !value.ends_with(')') {
        return None;
    }
    let mut arguments = SplitTopLevel(inner, b',');
    if arguments.len() < 2 {
        return None;
    }
    let mut shader = PaintShader {
        kind: PaintShaderKind::kLinearGradient,
        spread: if repeating {
            PaintSpreadMethod::kRepeat
        } else {
            PaintSpreadMethod::kPad
        },
        unit_coordinates: true,
        start: Offset { x: 0.5, y: 0.0 },
        end: Offset { x: 0.5, y: 1.0 },
        ..Default::default()
    };
    if let Some((start, end)) = ParseLinearGradientDirection(&arguments[0]) {
        shader.start = start;
        shader.end = end;
        arguments.remove(0);
    } else if let Some(angle) = AngleDegrees(&arguments[0]) {
        shader.linear_angle = Some(angle);
        arguments.remove(0);
    }
    if arguments.len() < 2 {
        return None;
    }
    shader.stops = ParseGradientColorStops(&arguments, font_size)?;
    if repeating {
        if shader.stops.iter().any(|s| s.offset_length != 0.0) {
            return None;
        }
        let first = shader.stops.first()?.offset;
        let last = shader.stops.last()?.offset;
        let period = last - first;
        if period <= 0.0 {
            return None;
        }
        if shader.linear_angle.is_some() {
            shader.linear_start_offset = first;
            shader.linear_end_offset = last;
        } else {
            let start = shader.start;
            let dx = shader.end.x - start.x;
            let dy = shader.end.y - start.y;
            shader.start = Offset {
                x: start.x + dx * first,
                y: start.y + dy * first,
            };
            shader.end = Offset {
                x: start.x + dx * last,
                y: start.y + dy * last,
            };
        }
        for stop in &mut shader.stops {
            stop.offset = (stop.offset - first) / period;
        }
    } else {
        let (first, last) = NormalizeExtendedGradientDomain(&mut shader.stops)?;
        if first != 0.0 || last != 1.0 {
            if shader.linear_angle.is_some() {
                shader.linear_start_offset = first;
                shader.linear_end_offset = last;
            } else {
                let start = shader.start;
                let dx = shader.end.x - start.x;
                let dy = shader.end.y - start.y;
                shader.start = Offset {
                    x: start.x + dx * first,
                    y: start.y + dy * first,
                };
                shader.end = Offset {
                    x: start.x + dx * last,
                    y: start.y + dy * last,
                };
            }
        }
    }
    Some(shader)
}
