//! Negative proof for three selector-only HTML attributes. A simple attr(name)
//! records that name. Unknown functions/attribute grammar and escaped tokens
//! retain all dependency bits. Custom-property values use the same proof.
#![allow(non_snake_case)]
use cssom::compiled_rules::SelectorOnlyAttribute;

pub(crate) fn DeclarationAttributeDependencies(text: &str) -> u8 {
    const ALL: u8 = 7;
    let bytes = text.as_bytes();
    let mut result = 0;
    let mut i = 0;
    let mut depth = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' | b'|' => return ALL,
            b'\'' | b'"' => {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != quote {
                    if bytes[i] == b'\\' {
                        return ALL;
                    }
                    i += 1;
                }
                if i == bytes.len() {
                    return ALL;
                }
                i += 1;
                continue;
            }
            b if b.is_ascii_alphabetic() || matches!(b, b'_' | b'-') => {
                let start = i;
                while bytes
                    .get(i)
                    .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
                {
                    i += 1;
                }
                if bytes.get(i) != Some(&b'(') {
                    continue;
                }
                let name = text[start..i].to_ascii_lowercase();
                if name == "attr" {
                    let Some(close) = text[i + 1..].find(')').map(|offset| i + 1 + offset) else {
                        return ALL;
                    };
                    let attribute = text[i + 1..close].trim();
                    if attribute.is_empty()
                        || !attribute
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
                    {
                        return ALL;
                    }
                    for kind in SelectorOnlyAttribute::ALL {
                        if attribute.eq_ignore_ascii_case(kind.name()) {
                            result |= kind.mask();
                        }
                    }
                    i = close + 1;
                    continue;
                }
                // Recognized CSS value functions do not themselves read DOM
                // attributes. Continue into their arguments, including var()
                // fallbacks; unknown functions retain the full fallback.
                if ![
                    "var",
                    "env",
                    "calc",
                    "min",
                    "max",
                    "clamp",
                    "rgb",
                    "rgba",
                    "hsl",
                    "hsla",
                    "url",
                    "format",
                    "local",
                    "linear-gradient",
                    "radial-gradient",
                    "conic-gradient",
                    "repeating-linear-gradient",
                    "repeating-radial-gradient",
                    "repeating-conic-gradient",
                    "translate",
                    "translatex",
                    "translatey",
                    "translatez",
                    "translate3d",
                    "scale",
                    "scalex",
                    "scaley",
                    "scalez",
                    "scale3d",
                    "rotate",
                    "rotatex",
                    "rotatey",
                    "rotatez",
                    "rotate3d",
                    "skew",
                    "skewx",
                    "skewy",
                    "matrix",
                    "matrix3d",
                    "perspective",
                    "cubic-bezier",
                    "steps",
                    "invert",
                    "contrast",
                    "blur",
                    "alpha",
                    "grayscale",
                    "brightness",
                    "opacity",
                    "saturate",
                    "sepia",
                    "hue-rotate",
                    "drop-shadow",
                    "rect",
                    "inset",
                    "circle",
                    "ellipse",
                    "polygon",
                    "path",
                    "minmax",
                    "repeat",
                    "fit-content",
                ]
                .contains(&name.as_str())
                {
                    return ALL;
                }
                depth += 1;
                i += 1;
                continue;
            }
            b'(' => return ALL,
            b')' => {
                if depth == 0 {
                    return ALL;
                }
                depth -= 1;
            }
            b if !b.is_ascii() => return ALL,
            _ => {}
        }
        i += 1;
    }
    if depth != 0 {
        ALL
    } else {
        result
    }
}
