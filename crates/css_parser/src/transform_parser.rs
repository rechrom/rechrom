#![allow(non_snake_case)]

use crate::length_percentage_parser::ParseLengthPercentage;
use layoutng::internal::layout_input::{Offset, TransformMatrix};
use layoutng::internal::paint_input::{
    PaintTransform, PaintTransformOperation, PaintTransformOperationKind,
};

// cpp: css_parser/transform_parser.cc:16-34
fn Trim(input: &str) -> String {
    input
        .trim_matches(|c: char| c.is_ascii_whitespace())
        .to_owned()
}
fn Lower(input: &str) -> String {
    input.to_ascii_lowercase()
}

// cpp: css_parser/transform_parser.cc:36-49
fn Multiply(left: &TransformMatrix, right: &TransformMatrix) -> TransformMatrix {
    let mut output = TransformMatrix::default();
    for column in 0..4 {
        for row in 0..4 {
            let mut value = 0.0;
            for index in 0..4 {
                value += left.values[index * 4 + row] * right.values[column * 4 + index];
            }
            output.values[column * 4 + row] = value;
        }
    }
    output
}

// cpp: css_parser/transform_parser.cc:51-81
fn Translation(x: f64, y: f64) -> TransformMatrix {
    let mut output = TransformMatrix::default();
    output.values[12] = x;
    output.values[13] = y;
    output
}
fn Scale(x: f64, y: f64) -> TransformMatrix {
    let mut output = TransformMatrix::default();
    output.values[0] = x;
    output.values[5] = y;
    output
}
fn Rotation(radians: f64) -> TransformMatrix {
    let mut output = TransformMatrix::default();
    let cosine = radians.cos();
    let sine = radians.sin();
    output.values[0] = cosine;
    output.values[1] = sine;
    output.values[4] = -sine;
    output.values[5] = cosine;
    output
}
fn Skew(x_radians: f64, y_radians: f64) -> TransformMatrix {
    let mut output = TransformMatrix::default();
    output.values[4] = x_radians.tan();
    output.values[1] = y_radians.tan();
    output
}

// cpp: css_parser/transform_parser.cc:83-107
fn Arguments(input: &str) -> Vec<String> {
    let bytes = input.as_bytes();
    let mut output = Vec::new();
    let mut begin = 0;
    let mut depth = 0_u32;
    for index in 0..=bytes.len() {
        let value = bytes.get(index).copied().unwrap_or(b',');
        if value == b'(' {
            depth += 1;
            continue;
        }
        if value == b')' {
            if depth == 0 {
                return Vec::new();
            }
            depth -= 1;
            continue;
        }
        if depth != 0 || (value != b',' && !value.is_ascii_whitespace()) {
            continue;
        }
        if index > begin {
            output.push(Trim(&input[begin..index]));
        }
        begin = index + 1;
    }
    if depth != 0 {
        return Vec::new();
    }
    output.retain(|value| !value.is_empty());
    output
}

// cpp: css_parser/transform_parser.cc:109-117
fn Number(input: &str) -> Option<f64> {
    let storage = Trim(input);
    if storage.is_empty() {
        return None;
    }
    let mut bytes = storage.as_bytes().to_vec();
    bytes.push(0);
    let begin = bytes.as_ptr().cast::<std::ffi::c_char>();
    let mut end = std::ptr::null_mut();
    unsafe extern "C" {
        fn strtod(begin: *const std::ffi::c_char, end: *mut *mut std::ffi::c_char) -> f64;
    }
    let value = unsafe { strtod(begin, &mut end) };
    let consumed = unsafe { end.offset_from(begin) };
    if consumed == 0 || consumed as usize != storage.len() || !value.is_finite() {
        return None;
    }
    Some(value)
}

// cpp: css_parser/transform_parser.cc:119-126
fn Length(input: &str, svg: bool) -> Option<f64> {
    let mut storage = Lower(&Trim(input));
    let pixels = storage.ends_with("px");
    if pixels {
        storage.truncate(storage.len() - 2);
    }
    let value = Number(&storage)?;
    if !svg && !pixels && value != 0.0 {
        return None;
    }
    Some(value)
}

// cpp: css_parser/transform_parser.cc:128-131
#[derive(Clone, Copy, Default)]
struct LengthPercentage {
    pixels: f64,
    percentage: f64,
}

// cpp: css_parser/transform_parser.cc:133-163
fn CSSLengthPercentage(input: &str, font_size: f64) -> Option<LengthPercentage> {
    let mut storage = Lower(&Trim(input));
    if let Some(calculated) = ParseLengthPercentage(&storage, font_size) {
        return Some(LengthPercentage {
            pixels: calculated.pixels,
            percentage: calculated.percentage,
        });
    }
    let mut scale = 1.0;
    let mut percentage = false;
    if storage.ends_with('%') {
        storage.truncate(storage.len() - 1);
        percentage = true;
    } else if storage.ends_with("rem") {
        storage.truncate(storage.len() - 3);
        scale = 16.0;
    } else if storage.ends_with("em") {
        storage.truncate(storage.len() - 2);
        scale = font_size;
    } else if storage.ends_with("px") {
        storage.truncate(storage.len() - 2);
    } else {
        let zero = Number(&storage)?;
        if zero != 0.0 {
            return None;
        }
        return Some(LengthPercentage::default());
    }
    let value = Number(&storage)?;
    if percentage {
        Some(LengthPercentage {
            percentage: value,
            ..LengthPercentage::default()
        })
    } else {
        Some(LengthPercentage {
            pixels: value * scale,
            ..LengthPercentage::default()
        })
    }
}

fn CSSLength(input: &str, font_size: f64) -> Option<f64> {
    let value = CSSLengthPercentage(input, font_size)?;
    (value.percentage == 0.0).then_some(value.pixels)
}

// cpp: css_parser/transform_parser.cc:165-186
fn Angle(input: &str, svg: bool) -> Option<f64> {
    let mut storage = Lower(&Trim(input));
    let mut multiplier = std::f64::consts::PI / 180.0;
    if storage.ends_with("grad") {
        storage.truncate(storage.len() - 4);
        multiplier = std::f64::consts::PI / 200.0;
    } else if storage.ends_with("deg") {
        storage.truncate(storage.len() - 3);
    } else if storage.ends_with("rad") {
        storage.truncate(storage.len() - 3);
        multiplier = 1.0;
    } else if storage.ends_with("turn") {
        storage.truncate(storage.len() - 4);
        multiplier = 2.0 * std::f64::consts::PI;
    } else if !svg {
        let zero = Number(&storage)?;
        if zero != 0.0 {
            return None;
        }
        return Some(0.0);
    }
    Number(&storage).map(|value| value * multiplier)
}

// cpp: css_parser/transform_parser.cc:188-262
fn Function(name: &str, args: &[String], svg: bool) -> Option<TransformMatrix> {
    let function = Lower(name);
    if function == "matrix" && args.len() == 6 {
        let mut values = Vec::new();
        for argument in args {
            values.push(Number(argument)?);
        }
        let mut output = TransformMatrix::default();
        output.values[0] = values[0];
        output.values[1] = values[1];
        output.values[4] = values[2];
        output.values[5] = values[3];
        output.values[12] = values[4];
        output.values[13] = values[5];
        return Some(output);
    }
    if function == "matrix3d" && args.len() == 16 {
        let mut output = TransformMatrix::default();
        for (index, argument) in args.iter().enumerate() {
            output.values[index] = Number(argument)?;
        }
        return Some(output);
    }
    if (function == "translate" && matches!(args.len(), 1 | 2))
        || ((function == "translatex" || function == "translatey") && args.len() == 1)
    {
        let first = Length(&args[0], svg)?;
        let second = if args.len() == 2 {
            Length(&args[1], svg)?
        } else {
            0.0
        };
        return Some(if function == "translatey" {
            Translation(0.0, first)
        } else if function == "translatex" {
            Translation(first, 0.0)
        } else {
            Translation(first, second)
        });
    }
    if (function == "scale" && matches!(args.len(), 1 | 2))
        || ((function == "scalex" || function == "scaley") && args.len() == 1)
    {
        let first = Number(&args[0])?;
        let second = if args.len() == 2 {
            Number(&args[1])?
        } else {
            first
        };
        return Some(if function == "scaley" {
            Scale(1.0, first)
        } else if function == "scalex" {
            Scale(first, 1.0)
        } else {
            Scale(first, second)
        });
    }
    if (function == "rotate" || function == "rotatez")
        && (args.len() == 1 || (svg && args.len() == 3))
    {
        let angle = Angle(&args[0], svg)?;
        let rotation = Rotation(angle);
        if args.len() == 1 {
            return Some(rotation);
        }
        let x = Number(&args[1])?;
        let y = Number(&args[2])?;
        return Some(Multiply(
            &Multiply(&Translation(x, y), &rotation),
            &Translation(-x, -y),
        ));
    }
    if (function == "skewx" || function == "skewy") && args.len() == 1 {
        let angle = Angle(&args[0], svg)?;
        return Some(if function == "skewx" {
            Skew(angle, 0.0)
        } else {
            Skew(0.0, angle)
        });
    }
    if function == "skew" && matches!(args.len(), 1 | 2) {
        let x = Angle(&args[0], svg)?;
        let y = if args.len() == 2 {
            Angle(&args[1], svg)?
        } else {
            0.0
        };
        return Some(Skew(x, y));
    }
    None
}

// cpp: css_parser/transform_parser.cc:264-293
fn CSSFunction(name: &str, args: &[String], font_size: f64) -> Option<PaintTransformOperation> {
    let function = Lower(name);
    if (function == "translate" && matches!(args.len(), 1 | 2))
        || ((function == "translatex" || function == "translatey") && args.len() == 1)
    {
        let first = CSSLengthPercentage(&args[0], font_size)?;
        let second = if args.len() == 2 {
            CSSLengthPercentage(&args[1], font_size)?
        } else {
            LengthPercentage::default()
        };
        let x = if function == "translatey" {
            LengthPercentage::default()
        } else {
            first
        };
        let y = if function == "translatey" {
            first
        } else if function == "translatex" {
            LengthPercentage::default()
        } else {
            second
        };
        return Some(PaintTransformOperation {
            kind: PaintTransformOperationKind::kTranslate,
            pixels: Offset {
                x: x.pixels,
                y: y.pixels,
            },
            percentages: Offset {
                x: x.percentage,
                y: y.percentage,
            },
            ..PaintTransformOperation::default()
        });
    }
    if function == "translatez" && args.len() == 1 {
        return Some(PaintTransformOperation {
            kind: PaintTransformOperationKind::kTranslate3D,
            z: CSSLength(&args[0], font_size)?,
            ..PaintTransformOperation::default()
        });
    }
    if function == "translate3d" && args.len() == 3 {
        let x = CSSLengthPercentage(&args[0], font_size)?;
        let y = CSSLengthPercentage(&args[1], font_size)?;
        return Some(PaintTransformOperation {
            kind: PaintTransformOperationKind::kTranslate3D,
            pixels: Offset {
                x: x.pixels,
                y: y.pixels,
            },
            percentages: Offset {
                x: x.percentage,
                y: y.percentage,
            },
            z: CSSLength(&args[2], font_size)?,
            ..PaintTransformOperation::default()
        });
    }
    let matrix = Function(name, args, false)?;
    Some(PaintTransformOperation {
        kind: PaintTransformOperationKind::kMatrix,
        matrix,
        ..PaintTransformOperation::default()
    })
}

// cpp: css_parser/transform_parser.cc:295-305
fn FunctionClose(input: &str, argument_begin: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut depth = 1;
    for cursor in argument_begin..bytes.len() {
        if bytes[cursor] == b'(' {
            depth += 1;
        } else if bytes[cursor] == b')' {
            depth -= 1;
            if depth == 0 {
                return Some(cursor);
            }
        }
    }
    None
}

// cpp: css_parser/transform_parser.cc:307-344
fn Parse(input: &str, svg: bool) -> Option<TransformMatrix> {
    let storage = Trim(input);
    if Lower(&storage) == "none" {
        return Some(TransformMatrix::default());
    }
    let bytes = storage.as_bytes();
    let mut output = TransformMatrix::default();
    let mut cursor = 0;
    let mut parsed = false;
    while cursor < bytes.len() {
        while cursor < bytes.len()
            && (bytes[cursor].is_ascii_whitespace() || (svg && bytes[cursor] == b','))
        {
            cursor += 1;
        }
        if cursor == bytes.len() {
            break;
        }
        let name_begin = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_alphanumeric() {
            cursor += 1;
        }
        let name = &storage[name_begin..cursor];
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if name.is_empty() || cursor == bytes.len() || bytes[cursor] != b'(' {
            return None;
        }
        cursor += 1;
        let argument_begin = cursor;
        let close = FunctionClose(&storage, cursor)?;
        let function = Function(name, &Arguments(&storage[argument_begin..close]), svg)?;
        output = Multiply(&output, &function);
        parsed = true;
        cursor = close + 1;
    }
    parsed.then_some(output)
}

// cpp: css_parser/transform_parser.cc:346-391
fn ParseCSS(input: &str, font_size: f64) -> Option<PaintTransform> {
    let storage = Trim(input);
    if Lower(&storage) == "none" {
        return Some(PaintTransform::default());
    }
    let bytes = storage.as_bytes();
    let mut output = PaintTransform::default();
    let mut cursor = 0;
    let mut parsed = false;
    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor == bytes.len() {
            break;
        }
        let name_begin = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_alphanumeric() {
            cursor += 1;
        }
        let name = &storage[name_begin..cursor];
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if name.is_empty() || cursor == bytes.len() || bytes[cursor] != b'(' {
            return None;
        }
        cursor += 1;
        let argument_begin = cursor;
        let close = FunctionClose(&storage, cursor)?;
        let operation = CSSFunction(name, &Arguments(&storage[argument_begin..close]), font_size)?;
        if operation.kind == PaintTransformOperationKind::kMatrix
            && output
                .operations
                .last()
                .is_some_and(|last| last.kind == PaintTransformOperationKind::kMatrix)
        {
            let last = output
                .operations
                .last_mut()
                .expect("checked last operation");
            last.matrix = Multiply(&last.matrix, &operation.matrix);
        } else {
            output.operations.push(operation);
        }
        parsed = true;
        cursor = close + 1;
    }
    parsed.then_some(output)
}

// cpp: css_parser/transform_parser.h:10-14
// cpp: css_parser/transform_parser.cc:395-398
pub fn ParseCSSTransform(value: &str, font_size: f64) -> Option<PaintTransform> {
    ParseCSS(value, font_size)
}

// cpp: css_parser/transform_parser.h:16-18
// cpp: css_parser/transform_parser.cc:400-402
pub fn ParseSVGTransform(value: &str) -> Option<TransformMatrix> {
    Parse(value, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_translate3d_preserves_box_relative_components() {
        let parsed = ParseCSSTransform("translate3d(-50%, 2em, 3px)", 12.0).unwrap();
        assert_eq!(parsed.operations.len(), 1);
        let operation = parsed.operations[0];
        assert_eq!(operation.kind, PaintTransformOperationKind::kTranslate3D);
        assert_eq!(operation.percentages.x, -50.0);
        assert_eq!(operation.pixels.y, 24.0);
        assert_eq!(operation.z, 3.0);
    }
}
