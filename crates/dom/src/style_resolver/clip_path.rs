#![allow(non_snake_case)]

use super::border_radius::{Length, SplitTopLevel};
use super::number::Number;
use super::selector::SourceSpace;
use super::shadow::SplitWhitespace;
use css_parser::length_percentage_parser::ParseLengthPercentage;
use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb};
use layoutng_assembly::internal::paint_input::ClipPathPaint;

// cpp: style_resolver/style_resolver.cc:905-909
fn Percentage(text: &str) -> Option<f64> {
    Number(
        text.trim_matches(|c: char| c.source_space())
            .strip_suffix('%')?,
    )
}

// cpp: style_resolver/style_resolver.cc:2369-2493
pub(crate) fn ParseClipPath(input: &str, font_size: f64) -> Option<ClipPathPaint> {
    let value = input.trim_matches(|c: char| c.source_space());
    let lower = value.to_ascii_lowercase();
    if !value.ends_with(')') {
        return None;
    }
    if lower.starts_with("inset(") {
        let parts = SplitWhitespace(&value[6..value.len() - 1]);
        if parts.is_empty() || parts.len() > 4 {
            return None;
        }
        let mut components = Vec::with_capacity(parts.len());
        for part in parts {
            if part.eq_ignore_ascii_case("round") {
                return None;
            }
            components.push(
                if let Some(calculated) = ParseLengthPercentage(&part, font_size) {
                    (calculated.pixels, calculated.percentage / 100.0)
                } else if let Some(percentage) = Percentage(&part) {
                    (0.0, percentage / 100.0)
                } else {
                    (Length(&part, font_size)?, 0.0)
                },
            );
        }
        let top = components[0];
        let right = *components.get(1).unwrap_or(&top);
        let bottom = *components.get(2).unwrap_or(&top);
        let left = *components.get(3).unwrap_or(&right);
        let x = [
            left,
            (-right.0, 1.0 - right.1),
            (-right.0, 1.0 - right.1),
            left,
        ];
        let y = [
            top,
            top,
            (-bottom.0, 1.0 - bottom.1),
            (-bottom.0, 1.0 - bottom.1),
        ];
        let mut result = ClipPathPaint::default();
        for index in 0..4 {
            let verb = if index == 0 {
                PaintPathVerb::kMoveTo
            } else {
                PaintPathVerb::kLineTo
            };
            result.commands.push(PaintPathCommand {
                verb,
                point: Offset {
                    x: x[index].0,
                    y: y[index].0,
                },
                ..Default::default()
            });
            result.percentage_commands.push(PaintPathCommand {
                verb,
                point: Offset {
                    x: x[index].1,
                    y: y[index].1,
                },
                ..Default::default()
            });
        }
        result.commands.push(PaintPathCommand {
            verb: PaintPathVerb::kClose,
            ..Default::default()
        });
        result.percentage_commands.push(PaintPathCommand {
            verb: PaintPathVerb::kClose,
            ..Default::default()
        });
        return Some(result);
    }
    if lower.starts_with("polygon(") {
        let mut arguments = SplitTopLevel(&value[8..value.len() - 1], b',');
        let rule = arguments.first()?.to_ascii_lowercase();
        let even_odd = rule == "evenodd";
        if even_odd || rule == "nonzero" {
            arguments.remove(0);
        }
        if arguments.len() < 3 {
            return None;
        }
        let component = |text: &str| -> Option<(f64, f64)> {
            if let Some(percentage) = Percentage(text) {
                Some((0.0, percentage / 100.0))
            } else {
                Some((Length(text, font_size)?, 0.0))
            }
        };
        let mut result = ClipPathPaint {
            even_odd,
            ..Default::default()
        };
        for (index, argument) in arguments.iter().enumerate() {
            let parts = SplitWhitespace(argument);
            if parts.len() != 2 {
                return None;
            }
            let x = component(&parts[0])?;
            let y = component(&parts[1])?;
            let verb = if index == 0 {
                PaintPathVerb::kMoveTo
            } else {
                PaintPathVerb::kLineTo
            };
            result.commands.push(PaintPathCommand {
                verb,
                point: Offset { x: x.0, y: y.0 },
                ..Default::default()
            });
            result.percentage_commands.push(PaintPathCommand {
                verb,
                point: Offset { x: x.1, y: y.1 },
                ..Default::default()
            });
        }
        result.commands.push(PaintPathCommand {
            verb: PaintPathVerb::kClose,
            ..Default::default()
        });
        result.percentage_commands.push(PaintPathCommand {
            verb: PaintPathVerb::kClose,
            ..Default::default()
        });
        return Some(result);
    }
    if !lower.starts_with("path(") {
        return None;
    }
    let arguments = SplitTopLevel(&value[5..value.len() - 1], b',');
    if arguments.is_empty() || arguments.len() > 2 {
        return None;
    }
    let mut even_odd = false;
    let path_text = if arguments.len() == 2 {
        let rule = arguments[0].to_ascii_lowercase();
        if rule != "evenodd" && rule != "nonzero" {
            return None;
        }
        even_odd = rule == "evenodd";
        &arguments[1]
    } else {
        &arguments[0]
    };
    let bytes = path_text.as_bytes();
    if bytes.len() < 2
        || !matches!(
            (bytes[0], bytes[bytes.len() - 1]),
            (b'\'', b'\'') | (b'"', b'"')
        )
    {
        return None;
    }
    let path = crate::svg_path_parser::ParseSVGPathDefault(&path_text[1..path_text.len() - 1])?;
    if path.commands.is_empty() {
        return None;
    }
    Some(ClipPathPaint {
        commands: path.commands,
        even_odd,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_clip_path_forms_and_invalid_cascade() {
        // Source style_resolver_test.cc:2031-2054, including mixed calc percentages.
        let path = ParseClipPath("inset(10px 20% calc(5px + 10%) 4px)", 16.0).unwrap();
        assert_eq!(path.commands.len(), 5);
        assert_eq!(path.commands[0].point, Offset { x: 4.0, y: 10.0 });
        assert_eq!(path.commands[2].point, Offset { x: -0.0, y: -5.0 });
        assert_eq!(path.percentage_commands[1].point, Offset { x: 0.8, y: 0.0 });
        assert_eq!(path.percentage_commands[2].point, Offset { x: 0.8, y: 0.9 });
        let polygon = ParseClipPath("POLYGON(evenodd, 50% 0, 100% 38%, 0 38%)", 16.0).unwrap();
        assert!(polygon.even_odd);
        assert_eq!(polygon.percentage_commands[0].point.x, 0.5);
        let mut style = layoutng_assembly::internal::layout_input::ComputedStyle::default();
        crate::style_resolver::apply(
            &mut style,
            "clip-path",
            "path(evenodd, 'M0 0 H20 V20 H0 Z M5 5 V15 H15 V5 Z')",
            (100.0, 100.0),
        );
        let original = style.paint.clip_path.clone().unwrap();
        assert!(original.even_odd);
        assert_eq!(original.commands[1].point.x, 20.0);
        for invalid in [
            "path('L0 0')",
            "inset(0 round 2px)",
            "polygon(0 0, 1px 0)",
            "circle(50%)",
            "inset(NaN%)",
        ] {
            crate::style_resolver::apply(&mut style, "clip-path", invalid, (100.0, 100.0));
            assert_eq!(style.paint.clip_path.as_ref(), Some(&original), "{invalid}");
        }
        crate::style_resolver::apply(&mut style, "clip-path", "NONE", (100.0, 100.0));
        assert!(style.paint.clip_path.is_none());
    }
}
