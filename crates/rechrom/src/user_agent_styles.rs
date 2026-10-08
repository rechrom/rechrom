#![allow(non_snake_case)]

use cssom::css_style_sheet::{CSSStyleSheet, ParseCSS};
use dom::ParsedDocument;

// cpp: browser/user_agent_styles.cc:10-42
const HTML_USER_AGENT_CSS: &str = r#"
  /* Keep the core rules in the same order and with the same values as
     Blink's html.css.  These are observable even on the oldest unstyled
     documents, so omitting them changes both typography and block geometry. */
  body {
    display: block;
    margin: 8px;
  }
  div {
    display: block;
  }
  article, aside, footer, header, hgroup, main, nav, search, section {
    display: block;
  }
  p {
    display: block;
    margin-block-start: 1__qem;
    margin-block-end: 1__qem;
    margin-inline-start: 0;
    margin-inline-end: 0;
  }
  h1 {
    display: block;
    font-size: 2em;
    margin-block-start: 0.67__qem;
    margin-block-end: 0.67em;
    margin-inline-start: 0;
    margin-inline-end: 0;
    font-weight: bold;
  }
  h2 {
    display: block;
    font-size: 1.5em;
    margin-block-start: 0.83__qem;
    margin-block-end: 0.83em;
    margin-inline-start: 0;
    margin-inline-end: 0;
    font-weight: bold;
  }
  h3 {
    display: block;
    font-size: 1.17em;
    margin-block-start: 1__qem;
    margin-block-end: 1em;
    margin-inline-start: 0;
    margin-inline-end: 0;
    font-weight: bold;
  }
  h4 {
    display: block;
    margin-block-start: 1.33__qem;
    margin-block-end: 1.33em;
    margin-inline-start: 0;
    margin-inline-end: 0;
    font-weight: bold;
  }
  h5 {
    display: block;
    font-size: 0.83em;
    margin-block-start: 1.67__qem;
    margin-block-end: 1.67em;
    margin-inline-start: 0;
    margin-inline-end: 0;
    font-weight: bold;
  }
  h6 {
    display: block;
    font-size: 0.67em;
    margin-block-start: 2.33__qem;
    margin-block-end: 2.33em;
    margin-inline-start: 0;
    margin-inline-end: 0;
    font-weight: bold;
  }
  dl {
    display: block;
    margin-block-start: 1__qem;
    margin-block-end: 1em;
  }
  dt { display: block; }
  dd {
    display: block;
    margin-inline-start: 40px;
  }
  ul, menu, dir {
    display: block;
    list-style-type: disc;
    margin-block-start: 1__qem;
    margin-block-end: 1em;
    padding-inline-start: 40px;
  }
  ol {
    display: block;
    list-style-type: decimal;
    margin-block-start: 1__qem;
    margin-block-end: 1em;
    padding-inline-start: 40px;
  }
  li { display: list-item; }
  blockquote {
    display: block;
    margin-block-start: 1__qem;
    margin-block-end: 1em;
    margin-inline-start: 40px;
    margin-inline-end: 40px;
  }
  /* Blink html.css a:-webkit-any-link defaults. HTML links use href
     presence here because the selector engine exposes that state directly. */
  a[href] {
    color: #0000ee;
    text-decoration: underline;
  }
  a[href], area[href] {
    cursor: pointer;
  }
  u, ins {
    text-decoration: underline;
  }
  abbr[title], acronym[title] {
    text-decoration: dotted underline;
  }
  tt, code, kbd, samp {
    font-family: monospace;
  }
  pre, xmp, plaintext, listing {
    display: block;
    font-family: monospace;
    white-space: pre;
    margin-block-start: 1__qem;
    margin-block-end: 1__qem;
    margin-inline-start: 0;
    margin-inline-end: 0;
  }
  table {
    border-spacing: 2px;
  }
  center {
    display: block;
    text-align: -webkit-center;
  }
  button,
  input:not([type="file" i], [type="image" i], [type="checkbox" i],
            [type="radio" i]) {
    -internal-align-content-block: center;
  }
  input {
    padding: 1px 2px;
    border: 2px inset #767676;
    background-color: #ffffff;
  }
  input[type="radio" i], input[type="checkbox" i] {
    padding: initial;
    background-color: initial;
    border: initial;
  }
  input[type="button" i], input[type="submit" i], input[type="reset" i],
  button {
    box-sizing: border-box;
    text-align: center;
    padding: 1px 6px;
    border: 2px outset #767676;
    background-color: #efefef;
    color: #101010;
  }
"#;

// cpp: browser/user_agent_styles.h:14-25
pub struct UserAgentStyleSheets {
    sheets_: &'static [CSSStyleSheet],
}

impl UserAgentStyleSheets {
    // cpp: browser/user_agent_styles.cc:46-47
    pub fn new() -> Self {
        static SHEETS: std::sync::OnceLock<Vec<CSSStyleSheet>> = std::sync::OnceLock::new();
        Self {
            sheets_: SHEETS.get_or_init(|| vec![ParseCSS(HTML_USER_AGENT_CSS)]),
        }
    }

    // cpp: browser/user_agent_styles.cc:49-52
    pub fn For(&self, _document: &ParsedDocument) -> &[CSSStyleSheet] {
        &self.sheets_
    }
    // cpp: browser/user_agent_styles.cc:49-52
    pub fn ForDocument(&self, _document: &dom::Document) -> &[CSSStyleSheet] {
        &self.sheets_
    }
}

impl Default for UserAgentStyleSheets {
    fn default() -> Self {
        Self::new()
    }
}
