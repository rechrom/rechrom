//! HTML `sizes` list orchestration.
//!
//! CSS token interpretation is supplied by the style owner.  This module owns
//! the HTML source-size list order, `auto` fallback, balanced comma recovery,
//! and the 100vw default used by Blink's `SizesAttributeParser`.
#![allow(non_snake_case)]

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SizesComponent {
    Auto,
    Length(f64),
    Invalid,
    FatalInvalid,
}

fn components(attribute: &str) -> Vec<&str> {
    let bytes = attribute.as_bytes();
    let mut output = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    let mut comment = false;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if comment {
            if byte == b'*' && bytes.get(index + 1) == Some(&b'/') {
                comment = false;
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
            }
            index += 1;
            continue;
        }
        if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
            comment = true;
            index += 2;
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                output.push(attribute[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    output.push(attribute[start..].trim());
    output
}

// cpp: sizes_attribute_parser.cc Parse/EffectiveSize.
pub fn ParseSizesAttribute(
    attribute: Option<&str>,
    viewport_width: f64,
    auto_size: Option<f64>,
    mut evaluate: impl FnMut(&str) -> SizesComponent,
) -> f64 {
    let Some(attribute) = attribute else {
        return viewport_width;
    };
    for component in components(attribute) {
        match evaluate(component) {
            SizesComponent::Auto => {
                if let Some(width) = auto_size.filter(|width| width.is_finite() && *width >= 0.0) {
                    return width;
                }
            }
            SizesComponent::Length(length) if length.is_finite() && length >= 0.0 => return length,
            SizesComponent::Length(_) | SizesComponent::Invalid => {}
            SizesComponent::FatalInvalid => return viewport_width,
        }
    }
    viewport_width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commas_inside_css_functions_do_not_split_the_component() {
        let mut seen = Vec::new();
        let result = ParseSizesAttribute(
            Some("(width > 20px) calc(min(40vw, 300px)), 100vw"),
            800.0,
            None,
            |component| {
                seen.push(component.to_owned());
                SizesComponent::Invalid
            },
        );
        assert_eq!(result, 800.0);
        assert_eq!(seen, ["(width > 20px) calc(min(40vw, 300px))", "100vw"]);
    }
}
