#![allow(non_snake_case)]

// The parser reports attributes, while Open owns fetching. Keep the HTML link
// classification here so every parser adapter supplies the same Blink
// semantics to the resource boundary.
// cpp: html/link_style.cc:StyleSheetTypeIsSupported,
//      LinkStyle::LoadStylesheetIfNeeded.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StyleSheetLinkDisposition {
    pub is_style_sheet: bool,
    pub alternate: bool,
    pub disabled: bool,
}

fn Tokens(value: &str) -> impl Iterator<Item = &str> {
    value.split(|character: char| matches!(character, ' ' | '\t'..='\r'))
}

fn StyleSheetTypeIsSupported(value: &str) -> bool {
    let mime = value.split(';').next().unwrap_or("").trim();
    mime.is_empty() || mime.eq_ignore_ascii_case("text/css")
}

pub fn ClassifyStyleSheetLink(
    rel: &str,
    type_value: &str,
    disabled: bool,
) -> StyleSheetLinkDisposition {
    let mut style_sheet = false;
    let mut alternate = false;
    for token in Tokens(rel) {
        style_sheet |= token.eq_ignore_ascii_case("stylesheet");
        alternate |= token.eq_ignore_ascii_case("alternate");
    }
    StyleSheetLinkDisposition {
        is_style_sheet: style_sheet && StyleSheetTypeIsSupported(type_value),
        alternate,
        disabled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_persistent_alternate_disabled_and_mime_links() {
        assert_eq!(
            ClassifyStyleSheetLink("stylesheet", "", false),
            StyleSheetLinkDisposition {
                is_style_sheet: true,
                alternate: false,
                disabled: false,
            }
        );
        assert!(ClassifyStyleSheetLink("alternate StyleSheet", "text/css", false).alternate);
        assert!(ClassifyStyleSheetLink("stylesheet", "text/css; charset=utf-8", true).disabled);
        assert!(!ClassifyStyleSheetLink("stylesheet", "text/plain", false).is_style_sheet);
        assert!(!ClassifyStyleSheetLink("preload", "text/css", false).is_style_sheet);
    }
}
