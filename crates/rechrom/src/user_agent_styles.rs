#![allow(non_snake_case)]

use cssom::css_style_sheet::{CSSStyleSheet, ParseCSS};
use dom::ParsedDocument;

// cpp: browser/user_agent_styles.cc:10-42
const HTML_USER_AGENT_CSS: &str = r#"
  /* Blink html.css: a:-webkit-any-link and area defaults. HTML links
     use href presence; keep this in UA CSS so author cursor rules win. */
  a[href], area[href] {
    cursor: pointer;
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
