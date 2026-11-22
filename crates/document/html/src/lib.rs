// Chromium identifiers keep their source spelling.
#![allow(non_snake_case, non_upper_case_globals)]

pub(crate) mod text_decoder;

pub mod diagnostic_scanner;
pub mod html_parser;
pub mod html_parser_host;
pub(crate) mod html_tag_names;
pub mod link_style;
pub mod parser;
pub mod preload_scanner;
pub use html_parser_host::{HTMLParserHost, ParserElementEvent, ParserElementPhase};

pub mod document_parser;
pub mod xml_document_parser;

pub mod markup_fragment;
