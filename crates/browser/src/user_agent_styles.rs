#![allow(non_snake_case)]

use cssom::{CSSStyleSheet, CSSUserAgentStyleSheetRole};
use dom::ParsedDocument;
use style::{ParseUserAgentCSS, ParseUserAgentCSSWithRole};

// Exact resource bytes from Chromium commit
// 6c1d401fcca5e1b0030563a90c2f2bba168e0c15. CSSDefaultStyleSheets owns
// distinct HTML, HTML-quirks, forced-colors, marker, fullscreen, SVG and
// MathML default RuleSets; all seven cross the existing
// CSSStyleSheet/StyleEngine boundary with typed
// UA-role provenance.
const HTML_USER_AGENT_CSS: &str = include_str!("../resources/user_agent/html.css");
const HTML_QUIRKS_USER_AGENT_CSS: &str = include_str!("../resources/user_agent/quirks.css");
const FORCED_COLORS_USER_AGENT_CSS: &str =
    include_str!("../resources/user_agent/forced_colors.css");
const MARKER_USER_AGENT_CSS: &str = include_str!("../resources/user_agent/marker.css");
const FULLSCREEN_USER_AGENT_CSS: &str = include_str!("../resources/user_agent/fullscreen.css");
const SVG_USER_AGENT_CSS: &str = include_str!("../resources/user_agent/svg.css");
const MATHML_USER_AGENT_CSS: &str = include_str!("../resources/user_agent/mathml.css");

// cpp: browser/user_agent_styles.h:14-25
pub struct UserAgentStyleSheets {
    sheets_: Vec<CSSStyleSheet>,
}

thread_local! {
    // Blink's StyleSheetContents and CSSStyleSheet live on the document/style
    // thread. Keep one parsed UA owner per thread and cheaply clone its Rc
    // backing into each browser assembly instead of requiring CSSOM to be Sync.
    static SHEETS: Vec<CSSStyleSheet> = vec![
        ParseUserAgentCSS(HTML_USER_AGENT_CSS),
        ParseUserAgentCSSWithRole(
            HTML_QUIRKS_USER_AGENT_CSS,
            CSSUserAgentStyleSheetRole::kHTMLQuirks,
        ),
        ParseUserAgentCSSWithRole(
            FORCED_COLORS_USER_AGENT_CSS,
            CSSUserAgentStyleSheetRole::kForcedColors,
        ),
        ParseUserAgentCSSWithRole(MARKER_USER_AGENT_CSS, CSSUserAgentStyleSheetRole::kMarker),
        ParseUserAgentCSSWithRole(
            FULLSCREEN_USER_AGENT_CSS,
            CSSUserAgentStyleSheetRole::kFullscreen,
        ),
        ParseUserAgentCSSWithRole(SVG_USER_AGENT_CSS, CSSUserAgentStyleSheetRole::kSVG),
        ParseUserAgentCSSWithRole(MATHML_USER_AGENT_CSS, CSSUserAgentStyleSheetRole::kMathML),
    ];
}

impl UserAgentStyleSheets {
    // cpp: browser/user_agent_styles.cc:46-47
    pub fn new() -> Self {
        Self {
            sheets_: SHEETS.with(Clone::clone),
        }
    }

    // cpp: browser/user_agent_styles.cc:49-52
    pub fn For(&self, _document: &ParsedDocument) -> &[CSSStyleSheet] {
        self.sheets_.as_slice()
    }
    // cpp: browser/user_agent_styles.cc:49-52
    pub fn ForDocument(&self, _document: &dom::Document) -> &[CSSStyleSheet] {
        self.sheets_.as_slice()
    }
}

impl Default for UserAgentStyleSheets {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::EDisplay;
    use layoutng_assembly::internal::layout_input::{ConstraintSpace, Size};

    #[test]
    fn cached_typed_html_quirks_svg_mathml_ua_sheets_reach_browser_computed_style() {
        crate::native_test_thread::run(body);
    }

    fn body() {
        let mut owner = html::html_parser::ParseHTML(
            "<!doctype html><html><head><title id=title>x</title></head>\
             <body id=body><section id=section></section>\
             <svg id=svg><text id=svgtext>x</text></svg>\
             <math id=math><mtable><mtr id=mathrow><mtd>x</mtd></mtr></mtable></math>\
             </body></html>",
        );
        let user_agent = UserAgentStyleSheets::new();
        assert_eq!(
            user_agent
                .ForDocument(owner.GetDocument())
                .iter()
                .map(CSSStyleSheet::UserAgentStyleSheetRole)
                .collect::<Vec<_>>(),
            vec![
                Some(CSSUserAgentStyleSheetRole::kHTML),
                Some(CSSUserAgentStyleSheetRole::kHTMLQuirks),
                Some(CSSUserAgentStyleSheetRole::kForcedColors),
                Some(CSSUserAgentStyleSheetRole::kMarker),
                Some(CSSUserAgentStyleSheetRole::kFullscreen),
                Some(CSSUserAgentStyleSheetRole::kSVG),
                Some(CSSUserAgentStyleSheetRole::kMathML),
            ]
        );
        let mut constraints = ConstraintSpace::default();
        constraints.available_size = Size {
            width: 800.0,
            height: 600.0,
        };
        let mut engine = style::StyleEngine::new(&owner);
        crate::style_services::ResolveLayoutStylesWithEngine(
            &mut engine,
            &mut owner,
            &constraints,
            style::PreferredColorScheme::kLight,
        );

        let document = owner.GetDocument();
        let find = |id: &str| {
            (0..document.NodeCount())
                .find(|&index| {
                    document
                        .Node(index)
                        .FindAttribute("id")
                        .is_some_and(|attribute| attribute.value == id)
                })
                .unwrap()
        };
        let display = |id: &str| unsafe {
            (&*document
                .ResolvedStyleFor(find(id))
                .unwrap()
                .native_style
                .Get())
                .Display()
        };
        assert_eq!(display("title"), EDisplay::kNone);
        assert_eq!(display("body"), EDisplay::kBlock);
        assert_eq!(display("section"), EDisplay::kBlock);
        assert_eq!(display("svgtext"), EDisplay::kBlock);
        assert_eq!(display("mathrow"), EDisplay::kTableRow);
    }
}
