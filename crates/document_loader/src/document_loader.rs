#![allow(non_snake_case)]
use dom::Document;
use html::html_parser::{
    HTMLDocumentParser, HTMLDocumentParserState, HTMLParserResult, HTMLParserStatus,
};
use html::HTMLParserHost;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{
    RequestDestination, URLLoadEvent, URLLoader, URLRequest, URLResponseHead, URLStreamOperation,
    URLWakeCallback,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentLoaderStatus {
    kIdle,
    kLoading,
    kResponseReady,
    kParsing,
    kFinished,
    kFailed,
    kCancelled,
}
#[derive(Clone, Copy, Debug)]
pub struct DocumentLoadBudget {
    pub body_bytes: usize,
    pub parser_tokens: usize,
}
impl Default for DocumentLoadBudget {
    fn default() -> Self {
        Self {
            body_bytes: 16 * 1024,
            parser_tokens: 4096,
        }
    }
}
// The existing standalone parser adapters specify a token budget only.
impl From<usize> for DocumentLoadBudget {
    fn from(parser_tokens: usize) -> Self {
        Self {
            parser_tokens,
            ..Self::default()
        }
    }
}
pub struct DocumentLoadProgress {
    pub body_finished: bool,
    pub made_progress: bool,
    pub parser: HTMLParserResult,
}
/// One main-document byte stream and its resident parser. Page owns the DOM,
/// navigation commit, JavaScript environment, lifecycle events and rendering.
/// No method waits for IO or retains a borrow of the document between calls.
pub struct DocumentLoader {
    loader: Rc<RefCell<dyn URLLoader>>,
    stream: Option<Box<dyn URLStreamOperation>>,
    response: Option<URLResponseHead>,
    requested_url: String,
    parser: Option<HTMLDocumentParserState>,
    status: DocumentLoaderStatus,
    body_finished: bool,
    needs_input: bool,
    failure: Option<(io::ErrorKind, String)>,
    wake: Option<URLWakeCallback>,
}
impl DocumentLoader {
    pub fn new(loader: Rc<RefCell<dyn URLLoader>>) -> Self {
        Self {
            loader,
            stream: None,
            response: None,
            requested_url: String::new(),
            parser: None,
            status: DocumentLoaderStatus::kIdle,
            body_finished: false,
            needs_input: true,
            failure: None,
            wake: None,
        }
    }
    pub fn Status(&self) -> DocumentLoaderStatus {
        self.status
    }
    pub fn URL(&self) -> &str {
        self.response
            .as_ref()
            .map_or("", |head| head.final_url.as_str())
    }
    pub fn SetWakeCallback(&mut self, callback: URLWakeCallback) {
        self.wake = Some(callback.clone());
        if let Some(stream) = &mut self.stream {
            stream.SetWakeCallback(callback);
        }
    }
    pub fn StartLoading(&mut self, request: &URLRequest) -> io::Result<()> {
        if self.status != DocumentLoaderStatus::kIdle {
            return Err(io::Error::other("document loading has already started"));
        }
        if request.url.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Open URL is empty",
            ));
        }
        let mut request = request.clone();
        request.destination = RequestDestination::kDocument;
        let result = self.loader.borrow_mut().LoadStream(&request);
        let mut stream = match result {
            Ok(stream) => stream,
            Err(error) => return self.Fail(error),
        };
        if let Some(wake) = &self.wake {
            stream.SetWakeCallback(wake.clone());
        }
        self.requested_url = request.url;
        self.stream = Some(stream);
        self.status = DocumentLoaderStatus::kLoading;
        Ok(())
    }
    /// Deliver headers exactly once. Body consumption starts only after Page
    /// has installed its document/environment and called BeginParsing.
    pub fn PollResponse(&mut self) -> io::Result<Option<URLResponseHead>> {
        self.CheckFailure()?;
        if self.status != DocumentLoaderStatus::kLoading {
            return Ok(None);
        }
        match self.stream.as_mut().expect("loading stream exists").Poll(1) {
            Ok(None) => Ok(None),
            Ok(Some(URLLoadEvent::Response(mut head))) => {
                if head.final_url.is_empty() {
                    head.final_url = self.requested_url.clone();
                }
                self.response = Some(head.clone());
                self.status = DocumentLoaderStatus::kResponseReady;
                Ok(Some(head))
            }
            Ok(Some(_)) => self.Fail(io::Error::new(
                io::ErrorKind::InvalidData,
                "document body precedes response headers",
            )),
            Err(error) => self.Fail(error),
        }
    }
    /// Bind one resident parser to the Page-owned document. Idle permits the
    /// existing caller-supplied HTML adapter; network loads require headers.
    pub fn BeginParsing(
        &mut self,
        document: &mut Document,
        host: &mut dyn HTMLParserHost,
    ) -> io::Result<()> {
        self.CheckFailure()?;
        if !matches!(
            self.status,
            DocumentLoaderStatus::kIdle | DocumentLoaderStatus::kResponseReady
        ) {
            return Err(io::Error::other("document is not ready to begin parsing"));
        }
        let mut parser = HTMLDocumentParserState::new(document, host);
        if let Some(head) = &self.response {
            if let Err(error) = parser.SetTextEncoding(&head.text_encoding) {
                return self.Fail(error);
            }
        }
        self.parser = Some(parser);
        self.status = DocumentLoaderStatus::kParsing;
        Ok(())
    }
    pub fn Pump(
        &mut self,
        document: &mut Document,
        host: &mut dyn HTMLParserHost,
        budget: impl Into<DocumentLoadBudget>,
    ) -> io::Result<DocumentLoadProgress> {
        let budget = budget.into();
        if budget.body_bytes == 0 || budget.parser_tokens == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "document loading budgets must be positive",
            ));
        }
        self.CheckFailure()?;
        if self.status == DocumentLoaderStatus::kFinished {
            return Ok(DocumentLoadProgress {
                body_finished: self.body_finished,
                made_progress: false,
                parser: HTMLParserResult {
                    status: HTMLParserStatus::kFinished,
                    script: None,
                },
            });
        }
        self.Parser()?;
        let mut made_progress = !self.needs_input && !self.parser.as_ref().unwrap().IsPaused();
        // Do not read ahead while the tokenizer still has work or a script is
        // blocking it. This bounds decoded parser input independently of IO.
        if self.needs_input && !self.body_finished && self.stream.is_some() {
            let mut remaining = budget.body_bytes;
            while remaining > 0 {
                let event = match self.stream.as_mut().unwrap().Poll(remaining) {
                    Ok(event) => event,
                    Err(error) => return self.Fail(error),
                };
                match event {
                    None => break,
                    Some(URLLoadEvent::Data(bytes)) => {
                        if bytes.is_empty() || bytes.len() > remaining {
                            return self.Fail(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "invalid document body chunk",
                            ));
                        }
                        remaining -= bytes.len();
                        made_progress = true;
                        if let Err(error) = self.parser.as_mut().unwrap().AppendBytes(&bytes) {
                            return self.Fail(error);
                        }
                    }
                    Some(URLLoadEvent::Finished) => {
                        if let Err(error) = self.parser.as_mut().unwrap().FinishBytes() {
                            return self.Fail(error);
                        }
                        self.body_finished = true;
                        made_progress = true;
                        self.stream.take();
                        break;
                    }
                    Some(URLLoadEvent::Response(_)) => {
                        return self.Fail(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "duplicate document response headers",
                        ))
                    }
                }
            }
        }
        let result = self
            .parser
            .as_mut()
            .unwrap()
            .Pump(document, host, budget.parser_tokens);
        self.needs_input = result.status == HTMLParserStatus::kNeedMoreInput;
        if result.status == HTMLParserStatus::kFinished {
            self.status = DocumentLoaderStatus::kFinished;
        }
        if result.status == HTMLParserStatus::kYielded
            || (self.needs_input && made_progress && self.stream.is_some())
        {
            if let Some(wake) = &self.wake {
                wake();
            }
        }
        Ok(DocumentLoadProgress {
            body_finished: self.body_finished,
            made_progress,
            parser: result,
        })
    }
    pub fn StopLoading(&mut self) {
        self.stream.take();
        self.parser.take();
        if !matches!(
            self.status,
            DocumentLoaderStatus::kFinished | DocumentLoaderStatus::kFailed
        ) {
            self.status = DocumentLoaderStatus::kCancelled;
        }
    }
    fn CheckFailure(&self) -> io::Result<()> {
        match &self.failure {
            Some((kind, message)) => Err(io::Error::new(*kind, message.clone())),
            None => Ok(()),
        }
    }
    fn Fail<T>(&mut self, error: io::Error) -> io::Result<T> {
        self.failure = Some((error.kind(), error.to_string()));
        self.stream.take();
        self.parser.take();
        self.status = DocumentLoaderStatus::kFailed;
        Err(error)
    }
    fn Parser(&mut self) -> io::Result<&mut HTMLDocumentParserState> {
        self.CheckFailure()?;
        if self.status != DocumentLoaderStatus::kParsing {
            return Err(io::Error::other("document parser is not active"));
        }
        Ok(self.parser.as_mut().expect("active parser exists"))
    }
    // Caller-supplied HTML and parser script adapters. Ordinary navigation uses
    // only StartLoading/PollResponse/BeginParsing/Pump/StopLoading above.
    pub fn Append(&mut self, text: &str) -> io::Result<()> {
        if self.stream.is_some() {
            return Err(io::Error::other("network document input is loader-owned"));
        }
        self.Parser()?.Append(text);
        if !text.is_empty() {
            self.needs_input = false;
        }
        Ok(())
    }
    pub fn FinishInput(&mut self) -> io::Result<()> {
        if self.stream.is_some() {
            return Err(io::Error::other("network document EOF is loader-owned"));
        }
        self.Parser()?.FinishInput();
        self.body_finished = true;
        self.needs_input = false;
        Ok(())
    }
    pub fn WithParser<R>(
        &mut self,
        document: &mut Document,
        host: &mut dyn HTMLParserHost,
        task: impl FnOnce(&mut HTMLDocumentParser<'_>) -> R,
    ) -> io::Result<R> {
        // EOF has been parsed, but deferred scripts may still need scoped access.
        self.CheckFailure()?;
        let parser = self
            .parser
            .as_mut()
            .ok_or_else(|| io::Error::other("document parser is not active"))?;
        let was_paused = parser.IsPaused();
        let result = parser.WithParser(document, host, task);
        if parser.IsFinished() {
            self.status = DocumentLoaderStatus::kFinished;
        }
        if was_paused && !parser.IsPaused() {
            self.needs_input = false;
            if let Some(wake) = &self.wake {
                wake();
            }
        }
        Ok(result)
    }
    pub fn InsertFromScript(&mut self, text: &str) -> io::Result<()> {
        self.Parser()?.InsertFromScript(text);
        Ok(())
    }
    pub fn ResumeAfterScript(&mut self) -> io::Result<()> {
        self.Parser()?.ResumeAfterScript();
        self.needs_input = false;
        if let Some(wake) = &self.wake {
            wake();
        }
        Ok(())
    }
}
