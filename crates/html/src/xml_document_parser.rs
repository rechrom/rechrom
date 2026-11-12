//! Chromium XMLDocumentParserRs and xml_ffi event adapter.
//! Fixed revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15:
//! core/xml/parser/xml_ffi.rs:23-64,88-220,260-270;
//! xml_document_parser_rs.cc:168-188,253-295,389-710,751-803.
//! The reader owns incremental input; the stack contains original DOM indices.
#![allow(non_snake_case)]
use crate::{
    html_parser::{HTMLParserResult, HTMLParserStatus, HTMLSourcePosition, ParserScript},
    HTMLParserHost, ParserElementPhase,
};
use dom::{
    persistent_document::{DOMAttribute, DOMDocumentType, DOMNamespace, DOMOwnerHandle, DOM},
    Document,
};
use foundation::{AtomicString, QualifiedName};
use std::{
    cell::Cell,
    io::{self, Cursor},
    rc::Rc,
    time::{Duration, Instant},
};
#[path = "xml_entities.rs"]
mod entities;
use xml::{
    common::Position,
    namespace::Namespace,
    reader::{ErrorKind, EventReader, XmlEvent},
    Encoding, ParserConfig,
};

pub struct XMLDocumentParserState {
    lifecycle: Rc<dom::document_parser_lifecycle::DocumentParserLifecycle>,
    document: DOMOwnerHandle,
    reader: EventReader<Cursor<Vec<u8>>>,
    namespaces: Vec<Namespace>,
    stack: Vec<usize>,
    decoder: Option<crate::text_decoder::TextDecoder>,
    text: String,
    leaf_text_node: Option<usize>,
    script_start_position: Option<HTMLSourcePosition>,
    input_finished: bool,
    finished: bool,
    saw_end_document: bool,
    waiting_for_script: Option<ParserScript>,
    failure: Option<(io::ErrorKind, String)>,
    consumed: usize,
    // XmlReadState::last_event_position (xml_ffi.rs:29,94,217-223).
    // The input reader's current token position is a different source value.
    last_event_line: Rc<Cell<i32>>,
}
impl XMLDocumentParserState {
    pub fn new(document: &Document) -> Self {
        let config = ParserConfig::new()
            .override_encoding(Some(Encoding::Utf8))
            .ignore_invalid_encoding_declarations(true)
            .ignore_comments(false)
            .ignore_end_of_stream(true)
            .allow_multiple_root_elements(false);
        let lifecycle =
            dom::document_parser_lifecycle::DocumentParserLifecycle::new(document.RootHandle());
        document.InstallDocumentParser(lifecycle.clone());
        let last_event_line = Rc::new(Cell::new(-1));
        lifecycle.BindSourceLine(foundation::parser_source_line::ParserSourceLine::Borrow(
            &last_event_line,
        ));
        Self {
            lifecycle,
            document: document.RootHandle(),
            reader: EventReader::new_with_config(Cursor::new(Vec::new()), config),
            namespaces: vec![],
            stack: vec![document.Root()],
            decoder: None,
            text: String::new(),
            leaf_text_node: None,
            script_start_position: None,
            input_finished: false,
            finished: false,
            saw_end_document: false,
            waiting_for_script: None,
            failure: None,
            consumed: 0,
            last_event_line,
        }
    }
    pub fn SetTextEncoding(&mut self, label: &str) -> io::Result<()> {
        if self.decoder.is_some() {
            return Err(io::Error::other("XML decoder already initialized"));
        }
        self.decoder = Some(crate::text_decoder::TextDecoder::new(label)?);
        Ok(())
    }
    pub fn CommitDecodedEncodingData(&self, document: &mut Document) {
        assert_eq!(document.RootHandle(), self.document);
        if let Some(decoder) = &self.decoder {
            document.SetDocumentEncoding(decoder.Encoding());
        }
    }
    pub fn AppendBytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        if self.decoder.is_none() {
            self.SetTextEncoding("utf-8")?;
        }
        let text = self.decoder.as_mut().unwrap().append(bytes, false)?;
        self.Append(&text);
        Ok(())
    }
    pub fn Append(&mut self, text: &str) {
        if self.lifecycle.IsStopped() {
            return;
        }
        assert!(!self.input_finished, "XML input after EOF");
        self.reader
            .source_mut()
            .get_mut()
            .extend_from_slice(text.as_bytes());
    }
    pub fn FinishBytes(&mut self) -> io::Result<()> {
        if let Some(decoder) = &mut self.decoder {
            let tail = decoder.append(&[], true)?;
            self.Append(&tail);
        }
        self.FinishInput();
        Ok(())
    }
    pub fn FinishInput(&mut self) {
        if self.lifecycle.IsStopped() {
            return;
        }
        self.input_finished = true;
    }
    pub fn BufferedInputCharacters(&self) -> usize {
        self.reader
            .source()
            .get_ref()
            .len()
            .saturating_sub(self.consumed)
    }
    pub fn IsPaused(&self) -> bool {
        !self.lifecycle.IsStopped() && self.lifecycle.IsWaitingForScripts()
    }
    pub fn IsFinished(&self) -> bool {
        self.lifecycle.IsStopped() || self.finished
    }
    pub fn ResumeAfterScript(&mut self) {
        if self.lifecycle.IsStopped() {
            return;
        }
        assert!(
            self.waiting_for_script.take().is_some(),
            "XML parser is not paused"
        );
        self.lifecycle.SetWaitingForScripts(false);
    }
    pub fn Lifecycle(&self) -> Rc<dom::document_parser_lifecycle::DocumentParserLifecycle> {
        self.lifecycle.clone()
    }
    pub fn StopParsing(&mut self) {
        self.lifecycle.StopParsing();
        self.waiting_for_script = None;
        self.lifecycle.SetWaitingForScripts(false);
    }
    pub fn Detach(&mut self) {
        self.lifecycle.Detach();
        self.waiting_for_script = None;
        self.lifecycle.SetWaitingForScripts(false);
        self.stack.clear();
        self.namespaces.clear();
    }
    fn FlushText(&mut self, document: &mut Document) {
        if let Some(node) = self.leaf_text_node.take() {
            document.SetTextContent(node, std::mem::take(&mut self.text));
        }
    }
    pub fn PumpWithTimeBudget(
        &mut self,
        document: &mut Document,
        host: &mut dyn HTMLParserHost,
        tokens: usize,
        budget: Duration,
    ) -> io::Result<HTMLParserResult> {
        assert_eq!(
            document.RootHandle(),
            self.document,
            "XML parser belongs to original arena"
        );
        self.CommitDecodedEncodingData(document);
        if let Some((kind, message)) = &self.failure {
            return Err(io::Error::new(*kind, message.clone()));
        }
        let result = self.ProcessEvents(document, host, tokens, budget);
        if let Err(error) = &result {
            self.failure = Some((error.kind(), error.to_string()));
        }
        result
    }
    fn ProcessEvents(
        &mut self,
        document: &mut Document,
        host: &mut dyn HTMLParserHost,
        tokens: usize,
        budget: Duration,
    ) -> io::Result<HTMLParserResult> {
        if self.lifecycle.IsStopped() {
            return Ok(HTMLParserResult {
                status: HTMLParserStatus::kFinished,
                script: None,
            });
        }
        if let Some(script) = self.waiting_for_script {
            return Ok(HTMLParserResult {
                status: HTMLParserStatus::kWaitingForScript,
                script: Some(script),
            });
        }
        if self.finished {
            return Ok(HTMLParserResult {
                status: HTMLParserStatus::kFinished,
                script: None,
            });
        }
        // The XML reader may report EndDocument at a temporary byte boundary.
        // Retain that fact until the loader's actual EOF (source Finish).
        if self.saw_end_document && self.input_finished {
            self.finished = true;
            if self.lifecycle.IsParsing() {
                self.lifecycle.PrepareToStopParsing();
            }
            return Ok(HTMLParserResult {
                status: HTMLParserStatus::kFinished,
                script: None,
            });
        }
        let start = Instant::now();
        for _ in 0..tokens {
            if self.lifecycle.IsStopped() {
                return Ok(HTMLParserResult {
                    status: HTMLParserStatus::kFinished,
                    script: None,
                });
            }
            if self.saw_end_document {
                return Ok(HTMLParserResult {
                    status: HTMLParserStatus::kNeedMoreInput,
                    script: None,
                });
            }
            let event = match self.reader.next() {
                Ok(event) => event,
                Err(error) => {
                    let resumable = matches!(error.kind(),ErrorKind::Syntax(message) if message.starts_with("Unexpected end of stream"));
                    self.consumed = self.reader.source().position() as usize;
                    if resumable && !self.input_finished {
                        return Ok(HTMLParserResult {
                            status: HTMLParserStatus::kNeedMoreInput,
                            script: None,
                        });
                    }
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("XML parse error: {error}"),
                    ));
                }
            };
            self.consumed = self.reader.source().position() as usize;
            let pos = self.reader.position();
            self.last_event_line.set(pos.row as i32);
            match event {
                XmlEvent::StartDocument {
                    version,
                    encoding,
                    standalone,
                } => {
                    // xml_ffi.rs:96-138: the parser intentionally inspects the
                    // original prologue because StartDocument supplies default
                    // values even when no XML declaration was present.
                    let buffer = self.reader.source().get_ref();
                    let declaration = buffer.get(pos.column as usize..).and_then(|tail| {
                        tail.starts_with(b"<?xml ")
                            .then(|| {
                                tail.iter()
                                    .position(|c| *c == b'>')
                                    .map(|end| &tail[..=end])
                            })
                            .flatten()
                    });
                    let encoding = declaration
                        .filter(|bytes| bytes.windows(8).any(|window| window == b"encoding"))
                        .map_or("", |_| encoding.as_str());
                    document.SetXMLDeclaration(
                        version.as_str(),
                        encoding,
                        standalone,
                        standalone.is_some() || declaration.is_some(),
                    );
                }
                XmlEvent::StartElement {
                    name,
                    attributes,
                    namespace,
                } => {
                    self.FlushText(document);
                    if self.stack.len() > 5000 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "XML tree depth exceeds 5000",
                        ));
                    }
                    let mut attrs = Vec::new();
                    for (prefix, uri) in namespace.iter() {
                        if self
                            .namespaces
                            .last()
                            .and_then(|old| old.get(prefix))
                            .is_some_and(|old| old == uri)
                            || (self.namespaces.is_empty() && prefix.is_empty() && uri.is_empty())
                        {
                            continue;
                        }
                        attrs.push(DOMAttribute {
                            prefix: if prefix.is_empty() {
                                String::new()
                            } else {
                                "xmlns".into()
                            },
                            local_name: if prefix.is_empty() {
                                "xmlns".into()
                            } else {
                                prefix.into()
                            },
                            namespace_uri: "http://www.w3.org/2000/xmlns/".into(),
                            value: uri.into(),
                        });
                    }
                    for attribute in attributes {
                        attrs.push(DOMAttribute {
                            prefix: attribute.name.prefix.unwrap_or_default(),
                            local_name: attribute.name.local_name,
                            namespace_uri: attribute.name.namespace.unwrap_or_default(),
                            value: attribute.value,
                        });
                    }
                    let uri = name.namespace.as_deref().unwrap_or("");
                    let ns = match uri {
                        "http://www.w3.org/2000/svg" => DOMNamespace::kSVG,
                        "http://www.w3.org/1999/xhtml" => DOMNamespace::kHTML,
                        "http://www.w3.org/1998/Math/MathML" => DOMNamespace::kMathML,
                        _ => DOMNamespace::kNone,
                    };
                    let tag = QualifiedName::new(
                        name.prefix
                            .as_deref()
                            .map(AtomicString::from_str)
                            .unwrap_or_default(),
                        AtomicString::from_str(&name.local_name),
                        if uri.is_empty() {
                            AtomicString::default()
                        } else {
                            AtomicString::from_str(uri)
                        },
                    );
                    let element = document.CreateElementWithQualifiedName(ns, tag, attrs);
                    if document.Node(element).IsHTMLElement("script")
                        || document
                            .Node(element)
                            .IsElement("script", DOMNamespace::kSVG)
                    {
                        self.script_start_position = Some(HTMLSourcePosition {
                            line: pos.row as u32 + 1,
                            column: pos.column as u32 + 1,
                        });
                    }
                    document.AppendChild(*self.stack.last().unwrap(), element);
                    host.HandleParserElementInDocument(
                        document,
                        element,
                        ParserElementPhase::kInserted,
                    );
                    if self.lifecycle.IsStopped() {
                        return Ok(HTMLParserResult {
                            status: HTMLParserStatus::kFinished,
                            script: None,
                        });
                    }
                    self.stack.push(element);
                    self.namespaces.push(namespace);
                }
                XmlEvent::EndElement { .. } => {
                    self.FlushText(document);
                    let element = self.stack.pop().expect("XML stack");
                    self.namespaces.pop();
                    host.HandleParserElementInDocument(
                        document,
                        element,
                        ParserElementPhase::kChildrenFinished,
                    );
                    if self.lifecycle.IsStopped() {
                        return Ok(HTMLParserResult {
                            status: HTMLParserStatus::kFinished,
                            script: None,
                        });
                    }
                    let node = document.Node(element);
                    if document.ScriptingEnabled()
                        && (node.IsHTMLElement("script")
                            || node.IsElement("script", DOMNamespace::kSVG))
                        && document.IsConnectedToDocument(element)
                    {
                        let script = ParserScript {
                            node_id: node.Id(),
                            start_position: self
                                .script_start_position
                                .take()
                                .expect("script start position"),
                        };
                        self.waiting_for_script = Some(script);
                        self.lifecycle.SetWaitingForScripts(true);
                        return Ok(HTMLParserResult {
                            status: HTMLParserStatus::kWaitingForScript,
                            script: Some(script),
                        });
                    }
                }
                XmlEvent::Characters(text) | XmlEvent::Whitespace(text) => {
                    if self.leaf_text_node.is_none() {
                        let node = document.CreateText(String::new());
                        document.AppendChild(*self.stack.last().unwrap(), node);
                        self.leaf_text_node = Some(node);
                    }
                    self.text.push_str(&text)
                }
                XmlEvent::CData(text) => {
                    self.FlushText(document);
                    let node = document.CreateCDATASection(text);
                    document.AppendChild(*self.stack.last().unwrap(), node);
                }
                XmlEvent::Comment(text) => {
                    self.FlushText(document);
                    let node = document.CreateComment(text);
                    document.AppendChild(*self.stack.last().unwrap(), node);
                }
                XmlEvent::ProcessingInstruction { name, data } => {
                    self.FlushText(document);
                    if name == "xml-stylesheet" {
                        return Err(io::Error::new(
                            io::ErrorKind::Unsupported,
                            "XML stylesheet processing-instruction resource owner",
                        ));
                    }
                    let node = document.CreateProcessingInstruction(
                        name,
                        data.unwrap_or_default().trim_start().into(),
                    );
                    document.AppendChild(*self.stack.last().unwrap(), node);
                }
                XmlEvent::Doctype { .. } => {
                    self.FlushText(document);
                    let ids = self.reader.doctype_ids().expect("XML doctype IDs");
                    let (name, public_id, system_id) = (
                        ids.name().to_owned(),
                        ids.public_id().unwrap_or("").to_owned(),
                        ids.system_id().unwrap_or("").to_owned(),
                    );
                    // xhtml_subset.cc:13-27 and xml_ffi.rs:224-233. Only
                    // the source's declared XHTML subset admits HTML entities.
                    if matches!(
                        public_id.as_str(),
                        "-//W3C//DTD XHTML 1.0 Transitional//EN"
                            | "-//W3C//DTD XHTML 1.1//EN"
                            | "-//W3C//DTD XHTML 1.0 Strict//EN"
                            | "-//W3C//DTD XHTML 1.0 Frameset//EN"
                            | "-//W3C//DTD XHTML Basic 1.0//EN"
                            | "-//W3C//DTD XHTML 1.1 plus MathML 2.0//EN"
                            | "-//W3C//DTD XHTML 1.1 plus MathML 2.0 plus SVG 1.1//EN"
                            | "-//W3C//DTD MathML 2.0//EN"
                            | "-//WAPFORUM//DTD XHTML Mobile 1.0//EN"
                            | "-//WAPFORUM//DTD XHTML Mobile 1.1//EN"
                            | "-//WAPFORUM//DTD XHTML Mobile 1.2//EN"
                    ) {
                        for map in [
                            entities::LAT1_MAP,
                            entities::SYMBOL_MAP,
                            entities::SPECIAL_MAP,
                            entities::HTML5_MAP,
                        ] {
                            self.reader
                                .add_entities(map.iter().copied())
                                .expect("source built-in entities remain mutable at doctype");
                        }
                    }
                    let node = document.CreateDocumentType(name, public_id, system_id);
                    document.AppendChild(document.Root(), node);
                }
                XmlEvent::EndDocument => {
                    self.FlushText(document);
                    self.saw_end_document = true;
                    if self.input_finished {
                        self.finished = true;
                        if self.lifecycle.IsParsing() {
                            self.lifecycle.PrepareToStopParsing();
                        }
                        return Ok(HTMLParserResult {
                            status: HTMLParserStatus::kFinished,
                            script: None,
                        });
                    }
                }
            }
            if start.elapsed() >= budget {
                break;
            }
        }
        Ok(HTMLParserResult {
            status: HTMLParserStatus::kYielded,
            script: None,
        })
    }
}

/// SVGImage's source XMLDocument constructor/parser, over the same original
/// arena. No HTML wrapper or HTML tokenizer participates.
pub fn ParseSVGDocumentBytes(bytes: &[u8]) -> io::Result<DOM> {
    let mut owner = DOM::new();
    let document = owner.GetDocumentMut();
    document.SetDocumentType(document.Root(), DOMDocumentType::kSVG);
    document.SetScriptingEnabled(false);
    let mut parser = XMLDocumentParserState::new(document);
    parser.AppendBytes(bytes)?;
    parser.FinishBytes()?;
    struct Inert;
    impl HTMLParserHost for Inert {
        fn HandleParserElement(&mut self, _: crate::ParserElementEvent<'_>) {}
    }
    let mut host = Inert;
    while !parser.IsFinished() {
        parser.PumpWithTimeBudget(document, &mut host, usize::MAX, Duration::MAX)?;
    }
    Ok(owner)
}
