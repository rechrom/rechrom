//! Shared owning SVG view parser. Chromium SVGViewSpec.cc:45-72,80-88,
//! 147-266; SVGSVGElement.cc:723-750. Image and frame navigation use this
//! single parser and the original typed immutable view specification.
#![allow(non_snake_case)]
use dom::persistent_document::{
    DOMNamespace, DOMNode, SVGImagePreserveAspectRatio, SVGImageViewSpec,
};
use std::{ffi::c_char, io};
unsafe extern "C" {
    fn strtod(input: *const c_char, end: *mut *mut c_char) -> f64;
}
pub(crate) fn ParseViewBoxValue(value: &str) -> Option<[f64; 4]> {
    let values = ParseSVGNumbers(value)?;
    if values.len() != 4 || values[2] < 0.0 || values[3] < 0.0 {
        return None;
    }
    Some([values[0], values[1], values[2], values[3]])
}

pub(crate) fn ParsePreserveAspectRatio(value: &str) -> Option<SVGImagePreserveAspectRatio> {
    let parts = value.split_ascii_whitespace().collect::<Vec<_>>();
    let mut cursor = usize::from(parts.first() == Some(&"defer"));
    let align = *parts.get(cursor)?;
    cursor += 1;
    let mut result = SVGImagePreserveAspectRatio::default();
    if align == "none" {
        result.preserve_none = true;
    } else {
        let bytes = align.as_bytes();
        if bytes.len() != 8 || bytes[0] != b'x' || bytes[4] != b'Y' {
            return None;
        }
        result.align_x = match &bytes[1..4] {
            b"Min" => -1,
            b"Mid" => 0,
            b"Max" => 1,
            _ => return None,
        };
        result.align_y = match &bytes[5..8] {
            b"Min" => -1,
            b"Mid" => 0,
            b"Max" => 1,
            _ => return None,
        };
    }
    if let Some(meet_or_slice) = parts.get(cursor) {
        result.slice = match *meet_or_slice {
            "meet" => false,
            "slice" => true,
            _ => return None,
        };
        cursor += 1;
    }
    (cursor == parts.len()).then_some(result)
}

fn ParseSVGNumbers(value: &str) -> Option<Vec<f64>> {
    let mut bytes = value.as_bytes().to_vec();
    bytes.push(0);
    let mut cursor = bytes.as_ptr().cast::<c_char>();
    let mut values = Vec::new();
    loop {
        while unsafe { *cursor } != 0
            && ((unsafe { *cursor } as u8).is_ascii_whitespace()
                || unsafe { *cursor } as u8 == b',')
        {
            cursor = unsafe { cursor.add(1) };
        }
        if unsafe { *cursor } == 0 {
            return Some(values);
        }
        let mut end = std::ptr::null_mut();
        let number = unsafe { strtod(cursor, &mut end) };
        if end == cursor.cast_mut() || !number.is_finite() {
            return None;
        }
        values.push(number);
        cursor = end;
    }
}

fn MultiplyAffine(left: [f64; 6], right: [f64; 6]) -> [f64; 6] {
    [
        left[0] * right[0] + left[2] * right[1],
        left[1] * right[0] + left[3] * right[1],
        left[0] * right[2] + left[2] * right[3],
        left[1] * right[2] + left[3] * right[3],
        left[0] * right[4] + left[2] * right[5] + left[4],
        left[1] * right[4] + left[3] * right[5] + left[5],
    ]
}

fn ParseViewTransform(value: &str) -> io::Result<Option<[f64; 6]>> {
    let mut cursor = 0;
    let mut transform = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let bytes = value.as_bytes();
    while cursor < bytes.len() {
        while cursor < bytes.len() && (bytes[cursor].is_ascii_whitespace() || bytes[cursor] == b',')
        {
            cursor += 1;
        }
        if cursor == bytes.len() {
            break;
        }
        let name_start = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_alphabetic() {
            cursor += 1;
        }
        let name = &value[name_start..cursor];
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if name.is_empty() || bytes.get(cursor) != Some(&b'(') {
            return Ok(None);
        }
        cursor += 1;
        let arguments_start = cursor;
        while cursor < bytes.len() && bytes[cursor] != b')' {
            cursor += 1;
        }
        if cursor == bytes.len() {
            return Ok(None);
        }
        let arguments = ParseSVGNumbers(&value[arguments_start..cursor]);
        cursor += 1;
        let Some(arguments) = arguments else {
            return Ok(None);
        };
        let operation = match name {
            "matrix" if arguments.len() == 6 => arguments.try_into().unwrap(),
            "translate" if (1..=2).contains(&arguments.len()) => [
                1.0,
                0.0,
                0.0,
                1.0,
                arguments[0],
                arguments.get(1).copied().unwrap_or(0.0),
            ],
            "scale" if (1..=2).contains(&arguments.len()) => [
                arguments[0],
                0.0,
                0.0,
                arguments.get(1).copied().unwrap_or(arguments[0]),
                0.0,
                0.0,
            ],
            "rotate" if arguments.len() == 1 || arguments.len() == 3 => {
                let angle = arguments[0].to_radians();
                let rotation = [
                    angle.cos(),
                    angle.sin(),
                    -angle.sin(),
                    angle.cos(),
                    0.0,
                    0.0,
                ];
                if arguments.len() == 1 {
                    rotation
                } else {
                    let to_origin = [1.0, 0.0, 0.0, 1.0, -arguments[1], -arguments[2]];
                    let from_origin = [1.0, 0.0, 0.0, 1.0, arguments[1], arguments[2]];
                    MultiplyAffine(from_origin, MultiplyAffine(rotation, to_origin))
                }
            }
            "skewX" if arguments.len() == 1 => {
                [1.0, 0.0, arguments[0].to_radians().tan(), 1.0, 0.0, 0.0]
            }
            "skewY" if arguments.len() == 1 => {
                [1.0, arguments[0].to_radians().tan(), 0.0, 1.0, 0.0, 0.0]
            }
            known
                if matches!(
                    known,
                    "matrix" | "translate" | "scale" | "rotate" | "skewX" | "skewY"
                ) =>
            {
                return Ok(None);
            }
            _ => return Ok(None),
        };
        transform = MultiplyAffine(transform, operation);
    }
    Ok(Some(transform))
}

fn ParseSVGViewSpecification(fragment: &str) -> io::Result<Option<SVGImageViewSpec>> {
    let Some(body) = fragment.strip_prefix("svgView(") else {
        return Ok(None);
    };
    let mut result = SVGImageViewSpec::default();
    let mut cursor = 0;
    while cursor < body.len() && body.as_bytes()[cursor] != b')' {
        let name_start = cursor;
        while cursor < body.len() && body.as_bytes()[cursor].is_ascii_alphabetic() {
            cursor += 1;
        }
        let name = &body[name_start..cursor];
        if body.as_bytes().get(cursor) != Some(&b'(') {
            return Ok(None);
        }
        cursor += 1;
        let value_start = cursor;
        let mut depth = 1usize;
        while cursor < body.len() && depth != 0 {
            match body.as_bytes()[cursor] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            cursor += 1;
        }
        if depth != 0 {
            return Ok(None);
        }
        let value = &body[value_start..cursor - 1];
        match name {
            "viewBox" => {
                let Some(view_box) = ParseViewBoxValue(value) else {
                    return Ok(None);
                };
                result.view_box = Some(view_box);
            }
            "preserveAspectRatio" => {
                let Some(preserve) = ParsePreserveAspectRatio(value) else {
                    return Ok(None);
                };
                result.preserve_aspect_ratio = Some(preserve);
            }
            "transform" => {
                let Some(transform) = ParseViewTransform(value)? else {
                    return Ok(None);
                };
                result.transform = Some(transform);
            }
            // SVGViewSpec parses but deliberately ignores viewTarget.
            "viewTarget" => {}
            "zoomAndPan" if matches!(value, "magnify" | "disable") => {
                result.zoom_and_pan = Some(value == "magnify");
            }
            "zoomAndPan" => return Ok(None),
            _ => return Ok(None),
        }
        if body.as_bytes().get(cursor) == Some(&b';') {
            cursor += 1;
        }
    }
    Ok((body.as_bytes().get(cursor) == Some(&b')')).then_some(result))
}

/// SVGSVGElement::ParseViewSpec: an invalid svgView fragment may still name
/// an original SVG view element. DOM retains only this resolved typed result.
pub fn ParseViewSpec(
    fragment: &str,
    anchor: Option<&DOMNode>,
) -> io::Result<Option<SVGImageViewSpec>> {
    if fragment.starts_with("svgView(") {
        if let Some(spec) = ParseSVGViewSpecification(fragment)? {
            return Ok(Some(spec));
        }
    }
    let Some(view) = anchor.filter(|node| node.IsElement("view", DOMNamespace::kSVG)) else {
        return Ok(None);
    };
    Ok(Some(SVGImageViewSpec {
        view_box: view
            .FindAttribute("viewBox")
            .and_then(|value| ParseViewBoxValue(&value.value)),
        preserve_aspect_ratio: view
            .FindAttribute("preserveAspectRatio")
            .and_then(|value| ParsePreserveAspectRatio(&value.value)),
        zoom_and_pan: view.FindAttribute("zoomAndPan").and_then(|value| {
            if value.value.starts_with("magnify") {
                Some(true)
            } else if value.value.starts_with("disable") {
                Some(false)
            } else {
                None
            }
        }),
        transform: None,
    }))
}
