//! ResourcePreloader admission projected across HTML's package boundary.
//! Speculative requests have no DOM, decoder, events or load-blocker lifetime.
#![allow(non_snake_case)]
use document_loader::{
    ResolveUrl, ResourceFetcher, ResourceLoader, ResponsiveImageInput, ResponsiveImageSourceInput,
};
use html::preload_scanner::{PreloadCandidate, PreloadKind};
use resource::ResourceEngine;
use url_loader::{ReferrerPolicy, RequestDestination, URLRequest};

#[derive(Default)]
pub(crate) struct Preloads {
    pending: Vec<ResourceLoader>,
}

fn Attr<'a>(attributes: &'a [(String, String)], name: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}
fn Trim(value: &str) -> &str {
    value.trim_matches(|c: char| matches!(c, ' ' | '\t'..='\r'))
}

/// HTMLBaseElement::Process: use the first valid non-data/non-javascript base.
/// CSP documents are gated before speculative input reaches this owner.
pub(crate) fn AcceptedBase(document_url: &str, spelling: &str) -> Option<String> {
    let resolved = ResolveUrl(document_url, Trim(spelling)).ok()?;
    let parsed = url::Url::parse(&resolved).ok()?;
    (!matches!(parsed.scheme(), "data" | "javascript")).then_some(resolved)
}

impl Preloads {
    pub(crate) fn Poll(&mut self) {
        self.pending.retain_mut(|load| !load.Poll());
    }
    pub(crate) fn Start(
        &mut self,
        candidate: &PreloadCandidate,
        engine: &ResourceEngine,
        page: Option<&ResourceFetcher>,
        document_url: &str,
        current_base: &str,
        base_seen: bool,
        scripting: bool,
        supports_modules: bool,
    ) {
        let attributes = &candidate.attributes;
        // These fetch modes do not yet have a typed owner in this embedding.
        // Let the authoritative consumer discover them; never reuse bytes from
        // an incompatible speculative credentials/CORS or integrity request.
        if Attr(attributes, "crossorigin").is_some() || Attr(attributes, "integrity").is_some() {
            return;
        }
        let predicted_base = if base_seen {
            None
        } else {
            candidate
                .base_hrefs
                .iter()
                .find_map(|href| AcceptedBase(document_url, href))
        };
        let base = predicted_base.as_deref().unwrap_or(current_base);
        let source = match candidate.kind {
            PreloadKind::Image => {
                if Attr(attributes, "loading").is_some_and(|s| s.eq_ignore_ascii_case("lazy")) {
                    return;
                }
                let Some(page) = page else {
                    return;
                };
                let input = ResponsiveImageInput {
                    src: Attr(attributes, "src").map(str::to_owned),
                    srcset: Attr(attributes, "srcset").map(str::to_owned),
                    sizes: Attr(attributes, "sizes").map(str::to_owned),
                    picture_sources: candidate
                        .picture_sources
                        .iter()
                        .map(|source| ResponsiveImageSourceInput {
                            srcset: Attr(source, "srcset").map(str::to_owned),
                            sizes: Attr(source, "sizes").map(str::to_owned),
                            media: Attr(source, "media").map(str::to_owned),
                            image_type: Attr(source, "type").map(str::to_owned),
                        })
                        .collect(),
                    ..Default::default()
                };
                let Some(selection) = page.SelectPreloadImage(&input, base) else {
                    return;
                };
                selection.source
            }
            PreloadKind::StyleSheet => {
                let disposition = html::link_style::ClassifyStyleSheetLink(
                    Attr(attributes, "rel").unwrap_or(""),
                    Attr(attributes, "type").unwrap_or(""),
                    Attr(attributes, "disabled").is_some(),
                );
                if !disposition.is_style_sheet || disposition.disabled || disposition.alternate {
                    return;
                }
                let media = Attr(attributes, "media").unwrap_or("");
                if !Trim(media).is_empty()
                    && !page.is_some_and(|page| page.PreloadMediaMatches(media))
                {
                    return;
                }
                Attr(attributes, "href").unwrap_or("").to_owned()
            }
            PreloadKind::Script => {
                if !scripting {
                    return;
                }
                let kind = Trim(Attr(attributes, "type").unwrap_or("")).to_ascii_lowercase();
                let module = kind == "module";
                if (module && !supports_modules)
                    || (!module && Attr(attributes, "nomodule").is_some() && supports_modules)
                    || (!module
                        && !matches!(
                            kind.as_str(),
                            "" | "text/javascript"
                                | "application/javascript"
                                | "text/ecmascript"
                                | "application/ecmascript"
                        ))
                {
                    return;
                }
                // Module fetch identity also includes its credentials mode;
                // leave it with ModuleResources until that typed contract exists.
                if module {
                    return;
                }
                Attr(attributes, "src").unwrap_or("").to_owned()
            }
        };
        let source = Trim(&source);
        if source.is_empty() || source.starts_with('#') {
            return;
        }
        let Ok(url) = ResolveUrl(base, source) else {
            return;
        };
        let Ok(parsed) = url::Url::parse(&url) else {
            return;
        };
        if !matches!(parsed.scheme(), "http" | "https") {
            return;
        }
        let policy = Attr(attributes, "referrerpolicy")
            .and_then(ReferrerPolicy::FromString)
            .or_else(|| {
                candidate
                    .referrer_policy
                    .as_deref()
                    .and_then(ReferrerPolicy::FromString)
            })
            .unwrap_or_else(|| engine.ReferrerPolicy());
        let destination = match candidate.kind {
            PreloadKind::Image => RequestDestination::kImage,
            PreloadKind::StyleSheet => RequestDestination::kStyleSheet,
            PreloadKind::Script => RequestDestination::kScript,
        };
        browser_tracing::instant_with_text(
            "loading",
            "PreloadRequest",
            &url,
            &[
                ("destination", destination as u8 as f64),
                ("source_utf16_end", candidate.source_utf16_end as f64),
                ("from_insertion", candidate.from_insertion as u8 as f64),
            ],
        );
        self.pending.push(engine.Start(URLRequest {
            url,
            destination,
            defer: if candidate.kind == PreloadKind::Script
                && (Attr(attributes, "async").is_some() || Attr(attributes, "defer").is_some())
            {
                url_loader::RequestDefer::Lazy
            } else {
                url_loader::RequestDefer::None
            },
            referrer: document_url.to_owned(),
            referrer_policy: Some(policy),
            ..Default::default()
        }));
    }
}
