// cpp: html/parser/html_parser_options.h:12-20
// C++ DISALLOW_NEW prevents heap allocation. Rust has no equivalent type-level
// placement restriction; the four policy values and their defaults are kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HTMLParserOptions {
    pub scripting_flag: bool,
    pub track_attributes_ranges: bool,
    pub truncated_markup_declaration: bool,
    pub processing_instructions: bool,
}

impl Default for HTMLParserOptions {
    fn default() -> Self {
        Self {
            scripting_flag: false,
            track_attributes_ranges: false,
            truncated_markup_declaration: true,
            processing_instructions: false,
        }
    }
}
