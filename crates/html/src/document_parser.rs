//! The original ScriptableDocumentParser boundary shared by HTML and XML.
//! Document::CreateParser, document.cc:3653-3685. Each variant owns one parser
//! and borrows the same original arena only during pump or synchronous script.
#![allow(non_snake_case)]
use crate::{
    html_parser::{HTMLDocumentParser, HTMLDocumentParserState, HTMLParserResult},
    preload_scanner::PreloadScanResult,
    xml_document_parser::XMLDocumentParserState,
    HTMLParserHost,
};
use dom::Document;
use std::{io, time::Duration};

pub enum DocumentParserState {
    HTML(HTMLDocumentParserState),
    XML(XMLDocumentParserState),
}
pub enum DocumentParser<'p, 'd> {
    HTML(&'p mut HTMLDocumentParser<'d>),
    XML(&'p mut XMLDocumentParserState, &'p mut Document),
}
impl DocumentParser<'_, '_> {
    pub fn WithDocument<R>(&mut self, task: impl FnOnce(&mut Document) -> R) -> R {
        match self {
            Self::HTML(parser) => parser.WithDocument(task),
            Self::XML(_, document) => task(document),
        }
    }
    pub fn WithPausedDocument<R>(&mut self, task: impl FnOnce(&mut Document) -> R) -> R {
        match self {
            Self::HTML(parser) => parser.WithPausedDocument(task),
            Self::XML(parser, document) => {
                assert!(parser.IsPaused());
                let _script_scope = parser.Lifecycle().EnterExecutingScript();
                task(document)
            }
        }
    }
    pub fn ResumeAfterScript(&mut self) {
        match self {
            Self::HTML(parser) => parser.ResumeAfterScript(),
            Self::XML(parser, _) => parser.ResumeAfterScript(),
        }
    }
    pub fn InsertFromScript(&mut self, text: &str) {
        match self {
            Self::HTML(parser) => parser.InsertFromScript(text),
            Self::XML(_, _) => std::panic::panic_any(dom::error::DOMException {
                kind: dom::error::DOMExceptionKind::InvalidStateError,
                message: "document.write is unavailable on XML documents",
            }),
        }
    }
}
impl DocumentParserState {
    pub fn NewHTML(document: &mut Document, host: &mut dyn HTMLParserHost) -> Self {
        Self::HTML(HTMLDocumentParserState::new(document, host))
    }
    pub fn NewXML(document: &Document) -> Self {
        Self::XML(XMLDocumentParserState::new(document))
    }
    pub fn Lifecycle(
        &self,
    ) -> std::rc::Rc<dom::document_parser_lifecycle::DocumentParserLifecycle> {
        match self {
            Self::HTML(p) => p.Lifecycle(),
            Self::XML(p) => p.Lifecycle(),
        }
    }
    pub fn Detach(&mut self) {
        match self {
            Self::HTML(p) => p.Detach(),
            Self::XML(p) => p.Detach(),
        }
    }
    pub fn StopParsing(&mut self) {
        match self {
            Self::HTML(p) => p.StopParsing(),
            Self::XML(p) => p.StopParsing(),
        }
    }
    pub fn SetTextEncoding(&mut self, label: &str) -> io::Result<()> {
        match self {
            Self::HTML(p) => p.SetTextEncoding(label),
            Self::XML(p) => p.SetTextEncoding(label),
        }
    }
    pub fn AppendBytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        match self {
            Self::HTML(p) => p.AppendBytes(bytes),
            Self::XML(p) => p.AppendBytes(bytes),
        }
    }
    pub fn FinishBytes(&mut self) -> io::Result<()> {
        match self {
            Self::HTML(p) => p.FinishBytes(),
            Self::XML(p) => p.FinishBytes(),
        }
    }
    pub fn Append(&mut self, text: &str) {
        match self {
            Self::HTML(p) => p.Append(text),
            Self::XML(p) => p.Append(text),
        }
    }
    pub fn FinishInput(&mut self) {
        match self {
            Self::HTML(p) => p.FinishInput(),
            Self::XML(p) => p.FinishInput(),
        }
    }
    pub fn CommitDecodedEncodingData(&self, document: &mut Document) {
        match self {
            Self::HTML(p) => p.CommitDecodedEncodingData(document),
            Self::XML(p) => p.CommitDecodedEncodingData(document),
        }
    }
    pub fn BufferedInputCharacters(&self) -> usize {
        match self {
            Self::HTML(p) => p.BufferedInputCharacters(),
            Self::XML(p) => p.BufferedInputCharacters(),
        }
    }
    pub fn IsPaused(&self) -> bool {
        match self {
            Self::HTML(p) => p.IsPaused(),
            Self::XML(p) => p.IsPaused(),
        }
    }
    pub fn IsFinished(&self) -> bool {
        match self {
            Self::HTML(p) => p.IsFinished(),
            Self::XML(p) => p.IsFinished(),
        }
    }
    pub fn ResumeAfterScript(&mut self) {
        match self {
            Self::HTML(p) => p.ResumeAfterScript(),
            Self::XML(p) => p.ResumeAfterScript(),
        }
    }
    pub fn InsertFromScript(&mut self, text: &str) -> io::Result<()> {
        match self {
            Self::HTML(p) => {
                p.InsertFromScript(text);
                Ok(())
            }
            Self::XML(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "document.write is unavailable on XML documents",
            )),
        }
    }
    pub fn EnablePreloadScanning(&mut self, scripting: bool) {
        if let Self::HTML(p) = self {
            p.EnablePreloadScanning(scripting)
        }
    }
    pub fn DisablePreloadScanning(&mut self) {
        if let Self::HTML(p) = self {
            p.DisablePreloadScanning()
        }
    }
    pub fn ScanPreloads(&mut self, budget: usize) -> PreloadScanResult {
        match self {
            Self::HTML(p) => p.ScanPreloads(budget),
            Self::XML(_) => PreloadScanResult::default(),
        }
    }
    pub fn PumpWithTimeBudget(
        &mut self,
        document: &mut Document,
        host: &mut dyn HTMLParserHost,
        budget: usize,
        time: Duration,
    ) -> io::Result<HTMLParserResult> {
        match self {
            Self::HTML(p) => Ok(p.PumpWithTimeBudget(document, host, budget, time)),
            Self::XML(p) => p.PumpWithTimeBudget(document, host, budget, time),
        }
    }
    pub fn WithParser<R>(
        &mut self,
        document: &mut Document,
        host: &mut dyn HTMLParserHost,
        task: impl FnOnce(&mut DocumentParser<'_, '_>) -> R,
    ) -> R {
        match self {
            Self::HTML(p) => p.WithParser(document, host, |parser| {
                task(&mut DocumentParser::HTML(parser))
            }),
            Self::XML(p) => task(&mut DocumentParser::XML(p, document)),
        }
    }
}
