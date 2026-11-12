#![allow(non_snake_case)]

use cssom::{CSSStyleSheet, CSSUserAgentStyleSheetRole};
use std::{cell::OnceCell, rc::Rc};
use style::{ParseUserAgentCSS, ParseUserAgentCSSWithRole};

// Exact resource bytes from Chromium commit
// 6c1d401fcca5e1b0030563a90c2f2bba168e0c15. CSSDefaultStyleSheets owns
// distinct HTML, HTML-quirks, forced-colors, marker, overscroll, fullscreen, SVG and
// MathML default RuleSets; all eight cross the existing
// CSSStyleSheet/StyleEngine boundary with typed
// UA-role provenance.
const HTML_USER_AGENT_CSS: &str = include_str!("../resources/user_agent/html.css");
const HTML_QUIRKS_USER_AGENT_CSS: &str = include_str!("../resources/user_agent/quirks.css");
const FORCED_COLORS_USER_AGENT_CSS: &str =
    include_str!("../resources/user_agent/forced_colors.css");
const OVERSCROLL_USER_AGENT_CSS: &str = include_str!("../resources/user_agent/overscroll.css");
const MARKER_USER_AGENT_CSS: &str = include_str!("../resources/user_agent/marker.css");
const FULLSCREEN_USER_AGENT_CSS: &str = include_str!("../resources/user_agent/fullscreen.css");
const SVG_USER_AGENT_CSS: &str = include_str!("../resources/user_agent/svg.css");
const MATHML_USER_AGENT_CSS: &str = include_str!("../resources/user_agent/mathml.css");

// Source: modules/media_controls/media_controls_resource_loader.cc:17-22,83-101.
const MEDIA_CONTROLS_CSS: &str = include_str!("../resources/user_agent/media_controls.css");
const MEDIA_CONTROLS_ANDROID_CSS: &str =
    include_str!("../resources/user_agent/media_controls_android.css");
const MEDIA_INTERSTITIALS_CSS: &str =
    include_str!("../resources/user_agent/media_interstitials.css");
struct MediaControlsResourceLoader;
impl MediaControlsResourceLoader {
    fn GetUAStyleSheet(&self) -> CSSStyleSheet {
        let mut source = String::from(MEDIA_CONTROLS_CSS);
        if cfg!(target_os = "android")
            || foundation::RuntimeEnabledFeatures::MobileLayoutThemeEnabled()
        {
            source.push_str(MEDIA_CONTROLS_ANDROID_CSS);
        }
        source.push_str(MEDIA_INTERSTITIALS_CSS);
        ParseUserAgentCSSWithRole(&source, CSSUserAgentStyleSheetRole::kMediaControls)
    }
}

// cpp: browser/user_agent_styles.h:14-25
pub struct UserAgentStyleSheets {
    sheets_: Rc<Vec<Rc<CSSStyleSheet>>>,
    media_sheets_: OnceCell<Rc<Vec<Rc<CSSStyleSheet>>>>,
}

thread_local! {
    static MEDIA_LOADER: OnceCell<MediaControlsResourceLoader> = const { OnceCell::new() };
    static MEDIA_SHEETS: OnceCell<Rc<Vec<Rc<CSSStyleSheet>>>> = const { OnceCell::new() };
    // Blink's StyleSheetContents and CSSStyleSheet live on the document/style
    // thread. Keep one parsed UA owner per thread and cheaply clone its Rc
    // backing into each browser assembly instead of requiring CSSOM to be Sync.
    static SHEETS: Rc<Vec<Rc<CSSStyleSheet>>> = Rc::new(vec![
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
        ParseUserAgentCSSWithRole(OVERSCROLL_USER_AGENT_CSS, CSSUserAgentStyleSheetRole::kOverscroll),
        ParseUserAgentCSSWithRole(
            FULLSCREEN_USER_AGENT_CSS,
            CSSUserAgentStyleSheetRole::kFullscreen,
        ),
        ParseUserAgentCSSWithRole(SVG_USER_AGENT_CSS, CSSUserAgentStyleSheetRole::kSVG),
        ParseUserAgentCSSWithRole(MATHML_USER_AGENT_CSS, CSSUserAgentStyleSheetRole::kMathML),
    ].into_iter().map(Rc::new).collect());
}

impl UserAgentStyleSheets {
    // Source: MediaControlsResourceLoader::InjectMediaControlsUAStyleSheet,
    // cc:95-104. Installation is distinct from lazy parse on the first media.
    pub(crate) fn InjectMediaControlsUAStyleSheet() {
        MEDIA_LOADER.with(|loader| {
            loader.get_or_init(|| MediaControlsResourceLoader);
        });
    }

    // cpp: browser/user_agent_styles.cc:46-47
    pub fn new() -> Self {
        // CSSDefaultStyleSheets::EnsureDefaultStyleSheetsForElement selects
        // this modules resource for every video/audio, independently of the
        // controls attribute or a MediaControls widget. The same sheet owns
        // caption container and ::cue defaults (mediaControls.css:1322-1419).
        // Admit the product's loader capability here; parsing remains lazy at
        // the first real media element, below.
        Self::InjectMediaControlsUAStyleSheet();
        Self {
            sheets_: SHEETS.with(Clone::clone),
            media_sheets_: OnceCell::new(),
        }
    }

    pub(crate) fn Sheets(&self) -> &[Rc<CSSStyleSheet>] {
        &self.sheets_
    }

    // cpp: browser/user_agent_styles.cc:49-52
    pub fn ForDocument(&self, document: &dom::Document) -> &[Rc<CSSStyleSheet>] {
        if !document.HasMediaElementsForStyle() {
            return self.sheets_.as_slice();
        }
        self.media_sheets_
            .get_or_init(|| {
                MEDIA_SHEETS.with(|owner| {
                    owner
                        .get_or_init(|| {
                            let mut sheets = self.sheets_.as_ref().clone();
                            sheets.push(Rc::new(MEDIA_LOADER.with(|loader| {
                                loader
                                    .get()
                                    .expect("installed modules UA loader")
                                    .GetUAStyleSheet()
                            })));
                            Rc::new(sheets)
                        })
                        .clone()
                })
            })
            .as_slice()
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
        crate::test_thread::run(body);
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
            user_agent.Sheets().as_ptr(),
            UserAgentStyleSheets::new().Sheets().as_ptr()
        );
        assert_eq!(
            user_agent
                .ForDocument(owner.GetDocument())
                .iter()
                .map(|sheet| sheet.UserAgentStyleSheetRole())
                .collect::<Vec<_>>(),
            vec![
                Some(CSSUserAgentStyleSheetRole::kHTML),
                Some(CSSUserAgentStyleSheetRole::kHTMLQuirks),
                Some(CSSUserAgentStyleSheetRole::kForcedColors),
                Some(CSSUserAgentStyleSheetRole::kMarker),
                Some(CSSUserAgentStyleSheetRole::kOverscroll),
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
            (&*document.ResolvedStyleFor(find(id)).unwrap().style.Get()).Display()
        };
        assert_eq!(display("title"), EDisplay::kNone);
        assert_eq!(display("body"), EDisplay::kBlock);
        assert_eq!(display("section"), EDisplay::kBlock);
        assert_eq!(display("svgtext"), EDisplay::kBlock);
        assert_eq!(display("mathrow"), EDisplay::kTableRow);
    }
}
