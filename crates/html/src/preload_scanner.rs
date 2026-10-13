//! Source-aligned speculative HTML token scanning. The scanner consumes the
//! authoritative decoder's output and emits owned resource descriptions. URL
//! resolution, responsive selection, policy checks and requests belong to the
//! document resource owner; a candidate is never permission to fetch.
#![allow(non_snake_case)]

use crate::html_tag_names::{html_names::HTMLTag, LookupHtmlTag};
use crate::parser::html_parser_options::HTMLParserOptions;
use crate::parser::html_token::{HTMLToken, TokenType};
use crate::parser::html_tokenizer::HTMLTokenizer;
use crate::parser::segmented_string::SegmentedString;
use foundation::BlinkString;
use std::pin::Pin;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreloadKind {
    Image,
    StyleSheet,
    Script,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreloadCandidate {
    pub kind: PreloadKind,
    /// Native decoded attributes, with the first duplicate winning.
    pub attributes: Vec<(String, String)>,
    /// Ordered direct-child <source> attributes of the current <picture>.
    pub picture_sources: Vec<Vec<(String, String)>>,
    /// Earlier <base href> spellings in document order. The resource owner
    /// resolves them against the document URL, applies base-uri policy, and
    /// chooses the first valid non-data/non-javascript URL. Keeping invalid
    /// spellings here avoids incorrectly making an invalid first base win.
    pub base_hrefs: Vec<String>,
    /// Last valid meta referrer policy, normalized including legacy keywords.
    pub referrer_policy: Option<String>,
    /// End offset in this scanner's decoded UTF-16 input, excluding EOF.
    pub source_utf16_end: usize,
    pub from_insertion: bool,
}

#[derive(Debug, Default)]
pub struct PreloadScanResult {
    pub candidates: Vec<PreloadCandidate>,
    pub work_pending: bool,
    pub finished: bool,
    pub stopped_for_csp: bool,
}

/// Corresponds to HTMLPreloadScanner + TokenPreloadScanner, with request
/// admission projected into the resource owner across the package boundary.
pub struct HTMLPreloadScanner {
    tokenizer: Pin<Box<HTMLTokenizer>>,
    input: SegmentedString,
    decoded_utf16: usize,
    work_pending: bool,
    input_finished: bool,
    finished: bool,
    stopped_for_csp: bool,
    from_insertion: bool,
    context: TokenPreloadContext,
}

#[derive(Default)]
struct TokenPreloadContext {
    template_count: usize,
    dsd_count: usize,
    in_picture: bool,
    picture_sources: Vec<Vec<(String, String)>>,
    base_hrefs: Vec<String>,
    referrer_policy: Option<String>,
}

impl HTMLPreloadScanner {
    pub fn new(scripting: bool, from_insertion: bool) -> Self {
        Self {
            tokenizer: HTMLTokenizer::new(HTMLParserOptions {
                scripting_flag: scripting,
                ..HTMLParserOptions::default()
            }),
            input: SegmentedString::default(),
            decoded_utf16: 0,
            work_pending: false,
            input_finished: false,
            finished: false,
            stopped_for_csp: false,
            from_insertion,
            context: TokenPreloadContext::default(),
        }
    }

    /// Accept the same decoded segment as the parser; no independent decoder
    /// or stream read is allowed here.
    pub fn AppendDecoded(&mut self, text: &BlinkString) {
        if self.input_finished || self.stopped_for_csp || text.empty() {
            return;
        }
        self.decoded_utf16 += text.length() as usize;
        self.input.Append(&SegmentedString::from_string(text));
        self.work_pending = true;
    }

    pub fn FinishInput(&mut self) {
        if self.input_finished || self.stopped_for_csp {
            return;
        }
        self.input
            .Append(&SegmentedString::from_string(&BlinkString::from_utf16(&[
                0,
            ])));
        self.input.Close();
        self.input_finished = true;
        self.work_pending = true;
    }

    pub fn Scan(&mut self, token_budget: usize) -> PreloadScanResult {
        let mut candidates = Vec::new();
        for _ in 0..token_budget {
            if !self.work_pending || self.finished || self.stopped_for_csp {
                break;
            }
            let Some(token) = self.tokenizer.as_mut().NextToken(&mut self.input) else {
                self.work_pending = false;
                break;
            };
            let eof = token.GetType() == TokenType::kEndOfFile;
            let mut start_tag = None;
            let mut csp = false;
            if matches!(token.GetType(), TokenType::kStartTag | TokenType::kEndTag) {
                let tag = LookupHtmlTag(token.GetName().as_slice());
                csp = self.context.Observe(
                    token,
                    tag,
                    (self.input.NumberOfCharactersConsumed() as usize).min(self.decoded_utf16),
                    self.from_insertion,
                    &mut candidates,
                );
                if token.GetType() == TokenType::kStartTag {
                    start_tag = Some(tag);
                }
            }
            // HTMLPreloadScanner::Scan updates native tokenizer state even
            // within ignored templates: strings in scripts are not markup.
            if let Some(tag) = start_tag {
                self.tokenizer.as_mut().UpdateStateForTag(tag);
            }
            self.tokenizer.as_mut().ClearToken();
            if csp {
                // Chromium's legacy AllowPreloadingWithCSPMetaTag=false branch
                // clears source and resets the tokenizer. Until the owner can
                // publish an applied CSP checkpoint, never restart this scan.
                self.input.Clear();
                self.tokenizer.as_mut().Reset();
                self.stopped_for_csp = true;
                self.work_pending = false;
            } else if eof {
                self.finished = true;
                self.work_pending = false;
            }
        }
        PreloadScanResult {
            candidates,
            work_pending: self.work_pending,
            finished: self.finished,
            stopped_for_csp: self.stopped_for_csp,
        }
    }
}

impl TokenPreloadContext {
    // cpp: html_preload_scanner.cc TokenPreloadScanner::Scan
    fn Observe(
        &mut self,
        token: &HTMLToken,
        tag: HTMLTag,
        source_utf16_end: usize,
        from_insertion: bool,
        candidates: &mut Vec<PreloadCandidate>,
    ) -> bool {
        let start = token.GetType() == TokenType::kStartTag;
        if tag == HTMLTag::kTemplate {
            if start {
                let dsd = attribute(token, "shadowrootmode").is_some_and(|s| {
                    s.eq_ignore_ascii_case("open") || s.eq_ignore_ascii_case("closed")
                });
                if dsd && self.template_count == 0 {
                    self.dsd_count += 1;
                } else {
                    self.template_count += 1;
                }
            } else if self.template_count != 0 {
                self.template_count -= 1;
            } else {
                self.dsd_count = self.dsd_count.saturating_sub(1);
            }
            // DSD start tags also end the current direct-picture-child scope.
            if self.template_count == 0 && start {
                self.in_picture = false;
                self.picture_sources.clear();
            }
            return false;
        }
        if self.template_count != 0 {
            return false;
        }
        if !start {
            if tag == HTMLTag::kPicture {
                self.in_picture = false;
                self.picture_sources.clear();
            }
            return false;
        }
        if tag == HTMLTag::kBase && self.dsd_count == 0 {
            if let Some(href) = attribute(token, "href") {
                self.base_hrefs.push(href);
            }
            return false;
        }
        if tag == HTMLTag::kMeta && self.dsd_count == 0 {
            if let Some(equiv) = attribute(token, "http-equiv") {
                return equiv.eq_ignore_ascii_case("content-security-policy");
            }
            if attribute(token, "name").is_some_and(|s| s.eq_ignore_ascii_case("referrer")) {
                if let Some(policy) =
                    attribute(token, "content").and_then(|s| NormalizeMetaReferrer(&s))
                {
                    self.referrer_policy = Some(policy.to_owned());
                }
            }
            return false;
        }
        if tag == HTMLTag::kPicture {
            self.in_picture = true;
            self.picture_sources.clear();
            return false;
        }
        // Source stops picture matching at any intervening non-void start tag;
        // it is deliberately a lexical scanner, not another tree builder.
        if !is_void(tag) {
            self.in_picture = false;
            self.picture_sources.clear();
        }
        if tag == HTMLTag::kSource && self.in_picture {
            self.picture_sources.push(attributes(token));
            return false;
        }
        let kind = match tag {
            HTMLTag::kImg => Some(PreloadKind::Image),
            HTMLTag::kScript => Some(PreloadKind::Script),
            HTMLTag::kLink => attribute(token, "rel").and_then(|s| {
                s.split_ascii_whitespace()
                    .any(|rel| rel.eq_ignore_ascii_case("stylesheet"))
                    .then_some(PreloadKind::StyleSheet)
            }),
            _ => None,
        };
        if let Some(kind) = kind {
            candidates.push(PreloadCandidate {
                kind,
                attributes: attributes(token),
                picture_sources: if self.in_picture && kind == PreloadKind::Image {
                    self.picture_sources.clone()
                } else {
                    Vec::new()
                },
                base_hrefs: self.base_hrefs.clone(),
                referrer_policy: self.referrer_policy.clone(),
                source_utf16_end,
                from_insertion,
            });
        }
        false
    }
}

fn attribute(token: &HTMLToken, name: &str) -> Option<String> {
    token
        .Attributes()
        .iter()
        .find(|attribute| {
            attribute
                .NameBuffer()
                .as_slice()
                .iter()
                .copied()
                .eq(name.bytes().map(u16::from))
        })
        .map(|attribute| attribute.Value().Utf8())
}

fn attributes(token: &HTMLToken) -> Vec<(String, String)> {
    let mut result = Vec::new();
    for attr in token.Attributes() {
        let name = String::from_utf16_lossy(attr.NameBuffer().as_slice());
        if !result.iter().any(|(seen, _)| seen == &name) {
            result.push((name, attr.Value().Utf8()));
        }
    }
    result
}

pub fn NormalizeMetaReferrer(value: &str) -> Option<&'static str> {
    match value.to_ascii_lowercase().as_str() {
        "no-referrer" | "never" | "none" => Some("no-referrer"),
        "no-referrer-when-downgrade" => Some("no-referrer-when-downgrade"),
        "default" => Some("strict-origin-when-cross-origin"),
        "origin" => Some("origin"),
        "origin-when-cross-origin" | "origin-when-crossorigin" => Some("origin-when-cross-origin"),
        "same-origin" => Some("same-origin"),
        "strict-origin" => Some("strict-origin"),
        "strict-origin-when-cross-origin" => Some("strict-origin-when-cross-origin"),
        "unsafe-url" | "always" => Some("unsafe-url"),
        _ => None,
    }
}

fn is_void(tag: HTMLTag) -> bool {
    matches!(
        tag,
        HTMLTag::kArea
            | HTMLTag::kBase
            | HTMLTag::kBr
            | HTMLTag::kCol
            | HTMLTag::kEmbed
            | HTMLTag::kHr
            | HTMLTag::kImg
            | HTMLTag::kInput
            | HTMLTag::kLink
            | HTMLTag::kMeta
            | HTMLTag::kSource
            | HTMLTag::kTrack
            | HTMLTag::kWbr
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drain(scanner: &mut HTMLPreloadScanner) -> PreloadScanResult {
        let mut candidates = Vec::new();
        loop {
            let mut result = scanner.Scan(3);
            candidates.append(&mut result.candidates);
            if !result.work_pending {
                result.candidates = candidates;
                return result;
            }
        }
    }
    fn scan(text: &str, scripting: bool) -> PreloadScanResult {
        let mut scanner = HTMLPreloadScanner::new(scripting, false);
        scanner.AppendDecoded(&BlinkString::FromUtf8(text.as_bytes()));
        scanner.FinishInput();
        drain(&mut scanner)
    }
    fn src(candidate: &PreloadCandidate) -> Option<&str> {
        candidate
            .attributes
            .iter()
            .find(|(name, _)| name == "src")
            .map(|(_, s)| s.as_str())
    }

    #[test]
    fn split_decoded_segments_preserve_native_entities_attributes_and_offsets() {
        let source = "😀<img SRC='你好&amp;.png' src=duplicate><link rel=stylesheet href=a><script src=b>let x='<img src=false>';</script><img src=tail>";
        let expected = scan(source, true);
        assert_eq!(src(&expected.candidates[0]), Some("你好&.png"));
        assert_eq!(
            expected.candidates[0]
                .attributes
                .iter()
                .filter(|(n, _)| n == "src")
                .count(),
            1
        );
        assert_eq!(expected.candidates.len(), 4);
        for split in source
            .char_indices()
            .map(|(i, _)| i)
            .chain(std::iter::once(source.len()))
        {
            let mut scanner = HTMLPreloadScanner::new(true, false);
            scanner.AppendDecoded(&BlinkString::FromUtf8(source[..split].as_bytes()));
            let mut candidates = drain(&mut scanner).candidates;
            scanner.AppendDecoded(&BlinkString::FromUtf8(source[split..].as_bytes()));
            scanner.FinishInput();
            let result = drain(&mut scanner);
            candidates.extend(result.candidates);
            assert_eq!(candidates, expected.candidates, "split={split}");
            assert!(result.finished);
        }
    }

    #[test]
    fn rawtext_regular_templates_dsd_and_noscript_follow_source_branches() {
        let source = concat!(
            "<!-- <img src=comment> --><style>p{content:'<img src=style>'}</style>",
            "<title><img src=title></title><textarea><img src=textarea></textarea>",
            "<script>let x='<img src=script>';</script>",
            "<template><img src=template><template shadowrootmode=open><img src=nested></template></template>",
            "<template shadowrootmode=closed><base href=ignored><meta name=referrer content=never>",
            "<meta http-equiv=content-security-policy><img src=shadow></template>",
            "<noscript><img src=noscript></noscript><img src=visible>"
        );
        let result = scan(source, true);
        let images: Vec<_> = result
            .candidates
            .iter()
            .filter(|c| c.kind == PreloadKind::Image)
            .collect();
        assert_eq!(
            images.iter().filter_map(|c| src(c)).collect::<Vec<_>>(),
            ["shadow", "visible"]
        );
        assert!(!result.stopped_for_csp);
        assert!(images
            .iter()
            .all(|c| c.base_hrefs.is_empty() && c.referrer_policy.is_none()));
        assert!(scan(source, false)
            .candidates
            .iter()
            .any(|c| src(c) == Some("noscript")));
    }

    #[test]
    fn picture_source_scope_ends_at_intervening_non_void_start_tags() {
        let result = scan(
            concat!(
                "<picture><source srcset=first media=screen><br><source srcset=second>",
                "<img src=fallback><div><img src=nested></div></picture>",
                "<picture><source srcset=third><img src=direct></picture><img src=outside>"
            ),
            true,
        );
        assert_eq!(result.candidates[0].picture_sources.len(), 2);
        assert!(result.candidates[1].picture_sources.is_empty());
        assert_eq!(result.candidates[2].picture_sources.len(), 1);
        assert!(result.candidates[3].picture_sources.is_empty());
    }

    #[test]
    fn policy_spelling_preserves_first_valid_base_and_stops_at_csp() {
        let mut scanner = HTMLPreloadScanner::new(true, false);
        scanner.AppendDecoded(&BlinkString::from(concat!(
            "<base href='javascript:invalid'><base href='../assets/'><img src=before>",
            "<meta name=referrer content=never><meta name=referrer content=invalid><img src=private>",
            "<meta http-equiv=content-security-policy content=\"img-src 'none'\"><img src=blocked>"
        )));
        let result = drain(&mut scanner);
        assert_eq!(result.candidates.len(), 2);
        assert_eq!(
            result.candidates[0].base_hrefs,
            ["javascript:invalid", "../assets/"]
        );
        assert_eq!(
            result.candidates[1].referrer_policy.as_deref(),
            Some("no-referrer")
        );
        assert!(result.stopped_for_csp);
        scanner.AppendDecoded(&BlinkString::from("<img src=also-blocked>"));
        scanner.FinishInput();
        assert!(drain(&mut scanner).candidates.is_empty());
    }
}
