//! Bounded, diagnostic-only observations from the native HTML tokenizer.
//!
//! This is a lexical scan, not a tree builder or a resource admission policy.
//! It owns no document, host, callbacks, URL resolver, clock, or network access.
//! The caller must discard it when the authoritative parser's input can diverge
//! (for example, on script resumption or document.write).
#![allow(non_snake_case)]

use crate::html_tag_names::{html_names::HTMLTag, LookupHtmlTag};
use crate::parser::html_parser_options::HTMLParserOptions;
use crate::parser::html_token::{Attribute, HTMLToken, TokenType};
use crate::parser::html_tokenizer::HTMLTokenizer;
use crate::parser::segmented_string::SegmentedString;
use crate::text_decoder::TextDecoder;
use foundation::BlinkString;
use std::{io, pin::Pin};

const MAX_INPUT_BYTES: usize = 1024 * 1024;
const MAX_APPEND_BYTES: usize = 16 * 1024;
const MAX_SRC_BYTES: usize = 2048;

/// An image spelling observed in source, never permission to fetch it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticCandidate {
    /// Native-tokenizer attribute value, including entity decoding, without
    /// trimming or URL resolution. Values over 2048 UTF-8 bytes are omitted.
    pub raw_src: String,
    /// Cumulative decoded source UTF-16 units consumed through this token.
    /// This is neither a byte offset nor an offset in normalized token text.
    pub consumed_utf16: usize,
    /// Lexical base/referrer/CSP boundaries, not an accepted policy version.
    pub policy_epoch: usize,
    pub has_base: bool,
    pub in_picture: bool,
}

#[derive(Debug, Default)]
pub struct DiagnosticScanResult {
    pub observations: Vec<DiagnosticCandidate>,
    /// Cumulative decoded source consumption, excluding the synthetic EOF.
    pub consumed_utf16: usize,
    /// Cumulative source bytes fed to the decoder, including any BOM.
    pub bytes_fed: usize,
    /// The native tokenizer has emitted EOF, not merely received final bytes.
    pub finished: bool,
    /// More scanning may run without additional bytes. False after the native
    /// tokenizer asks for more input, even if an incomplete entity remains.
    pub work_pending: bool,
}

/// A separate decoder/tokenizer lifetime for a bounded diagnostic read-ahead.
/// Errors disable this diagnostic at the caller; they must not fail a document.
pub struct DiagnosticHTMLScanner {
    decoder: TextDecoder,
    tokenizer: Pin<Box<HTMLTokenizer>>,
    input: SegmentedString,
    context: LexicalContext,
    bytes_fed: usize,
    decoded_utf16: usize,
    input_finished: bool,
    finished: bool,
    work_pending: bool,
}

#[derive(Default)]
struct LexicalContext {
    template_depth: usize,
    picture_depth: usize,
    policy_epoch: usize,
    has_base: bool,
}

impl DiagnosticHTMLScanner {
    pub fn new(encoding: &str, scripting: bool) -> io::Result<Self> {
        Ok(Self {
            decoder: TextDecoder::new(encoding)?,
            tokenizer: HTMLTokenizer::new(HTMLParserOptions {
                scripting_flag: scripting,
                ..HTMLParserOptions::default()
            }),
            input: SegmentedString::default(),
            context: LexicalContext::default(),
            bytes_fed: 0,
            decoded_utf16: 0,
            input_finished: false,
            finished: false,
            work_pending: false,
        })
    }

    /// Append at most 16 KiB per call and 1 MiB over this scanner's lifetime.
    /// The incremental decoder retains incomplete code points between calls.
    pub fn AppendBytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        if self.input_finished {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot append after diagnostic input is finished",
            ));
        }
        if bytes.len() > MAX_APPEND_BYTES || bytes.len() > MAX_INPUT_BYTES - self.bytes_fed {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "diagnostic HTML input budget exceeded",
            ));
        }
        // Count bytes before decoding: even a failed decode can retain bytes,
        // so an ignored error must not let repeated appends bypass the cap.
        self.bytes_fed += bytes.len();
        let decoded = self.decoder.append(bytes, false)?;
        self.AppendDecoded(&decoded);
        Ok(())
    }

    pub fn FinishBytes(&mut self) -> io::Result<()> {
        if self.input_finished {
            return Ok(());
        }
        let decoded = self.decoder.append(&[], true)?;
        self.AppendDecoded(&decoded);
        // Match the authoritative parser's SegmentedString EOF protocol.
        self.input
            .Append(&SegmentedString::from_string(&BlinkString::from_utf16(&[
                0,
            ])));
        self.input.Close();
        self.input_finished = true;
        self.work_pending = true;
        Ok(())
    }

    fn AppendDecoded(&mut self, decoded: &str) {
        if decoded.is_empty() {
            return;
        }
        let text = BlinkString::FromUtf8(decoded.as_bytes());
        self.decoded_utf16 += text.length() as usize;
        self.input.Append(&SegmentedString::from_string(&text));
        self.work_pending = true;
    }

    /// Process at most `token_budget` emitted native tokens. A zero budget is
    /// an observational no-op. One token can span all retained input: this is
    /// a token fairness limit, not a character or time preemption boundary.
    pub fn Scan(&mut self, token_budget: usize) -> DiagnosticScanResult {
        let mut observations = Vec::new();
        for _ in 0..token_budget {
            if self.finished || !self.work_pending {
                break;
            }
            let Some(token) = self.tokenizer.as_mut().NextToken(&mut self.input) else {
                self.work_pending = false;
                break;
            };
            let consumed_utf16 =
                (self.input.NumberOfCharactersConsumed() as usize).min(self.decoded_utf16);
            let eof = token.GetType() == TokenType::kEndOfFile;
            let tag = match token.GetType() {
                TokenType::kStartTag | TokenType::kEndTag => {
                    let tag = LookupHtmlTag(token.GetName().as_slice());
                    self.context
                        .Observe(token, tag, consumed_utf16, &mut observations);
                    (token.GetType() == TokenType::kStartTag).then_some(tag)
                }
                _ => None,
            };
            // Reuse native script, RAWTEXT, RCDATA, plaintext, and conditional
            // noscript transitions, including inside ignored templates.
            if let Some(tag) = tag {
                self.tokenizer.as_mut().UpdateStateForTag(tag);
            }
            self.tokenizer.as_mut().ClearToken();
            if eof {
                self.finished = true;
                self.work_pending = false;
            }
        }
        DiagnosticScanResult {
            observations,
            consumed_utf16: (self.input.NumberOfCharactersConsumed() as usize)
                .min(self.decoded_utf16),
            bytes_fed: self.bytes_fed,
            finished: self.finished,
            work_pending: self.work_pending,
        }
    }
}

impl LexicalContext {
    fn Observe(
        &mut self,
        token: &HTMLToken,
        tag: HTMLTag,
        consumed_utf16: usize,
        observations: &mut Vec<DiagnosticCandidate>,
    ) {
        let start = token.GetType() == TokenType::kStartTag;
        if tag == HTMLTag::kTemplate {
            if start {
                self.template_depth += 1;
            } else {
                self.template_depth = self.template_depth.saturating_sub(1);
            }
            return;
        }
        if self.template_depth != 0 {
            return;
        }
        if tag == HTMLTag::kPicture {
            if start {
                self.picture_depth += 1;
            } else {
                self.picture_depth = self.picture_depth.saturating_sub(1);
            }
            return;
        }
        if !start {
            return;
        }
        match tag {
            HTMLTag::kBase if !self.has_base && attribute(token, "href").is_some() => {
                self.has_base = true;
                self.policy_epoch += 1;
            }
            HTMLTag::kMeta
                if attribute_equals(token, "name", "referrer")
                    || attribute_equals(token, "http-equiv", "content-security-policy") =>
            {
                self.policy_epoch += 1;
            }
            HTMLTag::kImg => {
                if let Some(src) = attribute(token, "src") {
                    let value = src.Value();
                    if value.length() as usize > MAX_SRC_BYTES {
                        return;
                    }
                    let raw_src = value.Utf8();
                    if raw_src.len() <= MAX_SRC_BYTES {
                        observations.push(DiagnosticCandidate {
                            raw_src,
                            consumed_utf16,
                            policy_epoch: self.policy_epoch,
                            has_base: self.has_base,
                            in_picture: self.picture_depth != 0,
                        });
                    }
                }
            }
            _ => {}
        }
    }
}

fn attribute<'a>(token: &'a HTMLToken, name: &str) -> Option<&'a Attribute> {
    // The tokenizer lowercases attribute names. The first duplicate wins.
    token.Attributes().iter().find(|attribute| {
        attribute
            .NameBuffer()
            .as_slice()
            .iter()
            .copied()
            .eq(name.bytes().map(u16::from))
    })
}

fn attribute_equals(token: &HTMLToken, name: &str, value: &str) -> bool {
    attribute(token, name)
        .is_some_and(|attribute| attribute.Value().Utf8().eq_ignore_ascii_case(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drain(scanner: &mut DiagnosticHTMLScanner) -> DiagnosticScanResult {
        let mut observations = Vec::new();
        loop {
            let mut result = scanner.Scan(3);
            observations.append(&mut result.observations);
            if !result.work_pending {
                result.observations = observations;
                return result;
            }
        }
    }

    fn scan(source: &str, scripting: bool) -> DiagnosticScanResult {
        let mut scanner = DiagnosticHTMLScanner::new("utf-8", scripting).unwrap();
        scanner.AppendBytes(source.as_bytes()).unwrap();
        scanner.FinishBytes().unwrap();
        drain(&mut scanner)
    }

    #[test]
    fn split_utf8_and_attributes_match_whole_input_without_a_host() {
        let source = "\u{feff}\r\n😀<img SRC='你好&amp;😀.png' src='duplicate'><img src=tail>";
        let expected = scan(source, true);
        assert_eq!(expected.observations[0].raw_src, "你好&😀.png");
        assert_eq!(expected.observations[1].raw_src, "tail");
        for split in 0..=source.len() {
            let mut scanner = DiagnosticHTMLScanner::new("utf8", true).unwrap();
            scanner.AppendBytes(&source.as_bytes()[..split]).unwrap();
            let mut observations = drain(&mut scanner).observations;
            scanner.AppendBytes(&source.as_bytes()[split..]).unwrap();
            scanner.FinishBytes().unwrap();
            let result = drain(&mut scanner);
            observations.extend(result.observations);
            assert_eq!(observations, expected.observations, "split={split}");
            assert_eq!(result.bytes_fed, source.len());
            assert_eq!(result.consumed_utf16, source.encode_utf16().count() - 1);
            assert!(result.finished);
        }
        let mut scanner = DiagnosticHTMLScanner::new("utf-8", true).unwrap();
        let mut observations = Vec::new();
        for byte in source.as_bytes() {
            scanner.AppendBytes(std::slice::from_ref(byte)).unwrap();
            observations.extend(drain(&mut scanner).observations);
        }
        scanner.FinishBytes().unwrap();
        observations.extend(drain(&mut scanner).observations);
        assert_eq!(observations, expected.observations);
    }

    #[test]
    fn incomplete_attribute_is_not_observed_or_repeated() {
        let mut scanner = DiagnosticHTMLScanner::new("utf-8", true).unwrap();
        scanner.AppendBytes(b"<img src='first&am").unwrap();
        let before = scanner.Scan(8);
        assert!(before.observations.is_empty());
        assert!(!before.work_pending);
        assert!(!before.finished);
        assert!(scanner.Scan(8).observations.is_empty());
        scanner.AppendBytes(b"p;second'>").unwrap();
        let result = scanner.Scan(8);
        assert_eq!(result.observations[0].raw_src, "first&second");
        assert_eq!(result.observations[0].consumed_utf16, 28);
        scanner.FinishBytes().unwrap();
        let result = drain(&mut scanner);
        assert!(result.observations.is_empty());
        assert!(result.finished);
        assert_eq!(result.consumed_utf16, 28);
    }

    #[test]
    fn native_states_and_templates_exclude_lexical_false_images() {
        let source = concat!(
            "<!-- <img src=comment> -->",
            "<script>let x = '<img src=script>'; let y = '</template>';</script>",
            "<style>p{content:'<img src=style>'}</style>",
            "<title><img src=title></title><textarea><img src=textarea></textarea>",
            "<template><img src=template><template><img src=nested></template>",
            "<script>let x = '</template><img src=script-template>';</script></template>",
            "<noscript><img src=noscript></noscript><img src=visible>"
        );
        assert_eq!(
            scan(source, true)
                .observations
                .iter()
                .map(|o| o.raw_src.as_str())
                .collect::<Vec<_>>(),
            ["visible"]
        );
        assert_eq!(
            scan(source, false)
                .observations
                .iter()
                .map(|o| o.raw_src.as_str())
                .collect::<Vec<_>>(),
            ["noscript", "visible"]
        );
        let expected = scan(source, true).observations;
        for chunk_size in [1, 2, 7, 31] {
            let mut scanner = DiagnosticHTMLScanner::new("utf-8", true).unwrap();
            let mut observations = Vec::new();
            for chunk in source.as_bytes().chunks(chunk_size) {
                scanner.AppendBytes(chunk).unwrap();
                observations.extend(drain(&mut scanner).observations);
            }
            scanner.FinishBytes().unwrap();
            observations.extend(drain(&mut scanner).observations);
            assert_eq!(observations, expected, "chunk_size={chunk_size}");
        }
    }

    #[test]
    fn base_meta_and_picture_only_annotate_the_lexical_context() {
        let result = scan(
            concat!(
                "<img src=before><base target=_blank><img src=no-href>",
                "<base href='https://example.invalid/base/'><img src=relative>",
                "<base href='https://ignored.invalid/'><img src=still-relative>",
                "<meta NAME=REFERRER content=no-referrer><img src=referrer>",
                "<meta HTTP-EQUIV=Content-Security-Policy content=\"img-src 'none'\"><img src=csp>",
                "<picture><template><base href=x><meta name=referrer><picture></template>",
                "<img src=in-picture></picture><img src=after-picture>"
            ),
            true,
        );
        let contexts: Vec<_> = result
            .observations
            .iter()
            .map(|o| (o.raw_src.as_str(), o.policy_epoch, o.has_base, o.in_picture))
            .collect();
        assert_eq!(
            contexts,
            [
                ("before", 0, false, false),
                ("no-href", 0, false, false),
                ("relative", 1, true, false),
                ("still-relative", 1, true, false),
                ("referrer", 2, true, false),
                ("csp", 3, true, false),
                ("in-picture", 3, true, true),
                ("after-picture", 3, true, false),
            ]
        );
    }

    #[test]
    fn token_budget_and_eof_keep_cumulative_source_offsets() {
        let source = "😀<img src=one><img src=two>";
        let mut scanner = DiagnosticHTMLScanner::new("utf-8", true).unwrap();
        scanner.AppendBytes(source.as_bytes()).unwrap();
        scanner.FinishBytes().unwrap();
        let untouched = scanner.Scan(0);
        assert_eq!(untouched.consumed_utf16, 0);
        assert_eq!(untouched.bytes_fed, source.len());
        assert!(untouched.work_pending);
        let text = scanner.Scan(1);
        assert!(text.observations.is_empty());
        assert_eq!(text.consumed_utf16, 2);
        let first = scanner.Scan(1);
        assert_eq!(first.observations[0].raw_src, "one");
        assert_eq!(first.observations[0].consumed_utf16, 15);
        let second = scanner.Scan(1);
        assert_eq!(second.observations[0].raw_src, "two");
        assert!(!second.finished);
        let eof = scanner.Scan(1);
        assert!(eof.finished);
        assert!(!eof.work_pending);
        assert_eq!(eof.consumed_utf16, source.encode_utf16().count());
        scanner.FinishBytes().unwrap();
        assert!(scanner.Scan(1).observations.is_empty());
        assert!(scanner.AppendBytes(b"ignored").is_err());
    }

    #[test]
    fn incomplete_utf8_at_eof_and_unsupported_encodings_are_diagnostic_errors() {
        assert!(DiagnosticHTMLScanner::new("unsupported", true).is_err());
        let mut scanner = DiagnosticHTMLScanner::new("utf-8", true).unwrap();
        scanner.AppendBytes(&[0xf0, 0x9f]).unwrap();
        let result = scanner.Scan(8);
        assert_eq!(result.bytes_fed, 2);
        assert_eq!(result.consumed_utf16, 0);
        assert!(!result.work_pending);
        assert!(scanner.FinishBytes().is_err());
    }

    #[test]
    fn windows1252_uses_the_shared_incremental_decoder() {
        let mut scanner = DiagnosticHTMLScanner::new("windows-1252", true).unwrap();
        scanner.AppendBytes(b"<img src='\x80.png'>").unwrap();
        scanner.FinishBytes().unwrap();
        assert_eq!(drain(&mut scanner).observations[0].raw_src, "€.png");
    }

    #[test]
    fn diagnostic_input_and_attribute_storage_have_explicit_caps() {
        let mut scanner = DiagnosticHTMLScanner::new("utf-8", true).unwrap();
        assert!(scanner
            .AppendBytes(&vec![b' '; MAX_APPEND_BYTES + 1])
            .is_err());
        assert_eq!(scanner.Scan(0).bytes_fed, 0);
        let chunk = vec![b' '; MAX_APPEND_BYTES];
        for _ in 0..MAX_INPUT_BYTES / MAX_APPEND_BYTES {
            scanner.AppendBytes(&chunk).unwrap();
        }
        assert!(scanner.AppendBytes(b" ").is_err());
        assert_eq!(scanner.Scan(0).bytes_fed, MAX_INPUT_BYTES);
        let allowed = "é".repeat(MAX_SRC_BYTES / 2);
        let too_long = format!("{allowed}é");
        let result = scan(
            &format!("<img src='{allowed}'><img src='{too_long}'><img src=tail>"),
            true,
        );
        assert_eq!(result.observations.len(), 2);
        assert_eq!(result.observations[0].raw_src, allowed);
        assert_eq!(result.observations[1].raw_src, "tail");
    }
}
