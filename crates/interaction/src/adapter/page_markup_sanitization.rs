// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! serialization.cc:783–949. A private inert Document owns the staging tree;
//! the embedding lends only its Style/Layout lifecycle, never another Page.
use super::page_selection_markup::{FragmentError, NativeMarkupDocument, RetainedFragment};
use crate::ownership::InteractionDocument;
use dom::{persistent_document::DOMNamespace, Document, DOM};
use html::{markup_fragment, parser::parser_content_policy::ParserContentPolicy};
use interaction_default::core::editing::serializers::{
    markup_accumulator,
    markup_formatter::ResolveUrls,
    serialization::{self, MarkupSanitizationContext},
};

/// The embedding assembles the ordinary engines for this one staging Document.
/// A session is discarded after the stabilization loop, including error exits.
pub trait MarkupSanitizationHost {
    fn CreateLifecycle(&self, document: &DOM) -> Box<dyn MarkupSanitizationLifecycle>;
}
pub trait MarkupSanitizationLifecycle {
    /// UpdateStyleAndLayout(kEditing), then styled CreateMarkup with annotate
    /// and for_markup_sanitization. The body belongs to `document` throughout.
    fn SerializeBodyWithInlineStyles(
        &mut self,
        document: &mut DOM,
        body: usize,
    ) -> Result<Vec<u16>, FragmentError>;
    fn Shutdown(&mut self, document: &mut DOM);
}

/// CreateStagingDocumentForMarkupSanitization. This Document starts at an opaque
/// about:blank origin and never inherits the live target's URL/security context.
/// Fragment tokenization uses the enabled scripting flag separately from the
/// disabled execution setting, as the source nonordinary frame does.
fn CreateStagingDocumentForMarkupSanitization() -> DOM {
    let mut document =
        html::html_parser::ParseHTML("<!doctype html><html><head></head><body></body></html>");
    let native = document.GetDocumentMut();
    native.SetDocumentURL("about:blank".into());
    native.SetIsForMarkupSanitization();
    native.SetScriptingEnabled(false);
    native.BindDOMWindow();
    native.SetLifecycleActive(true);
    document
}
struct StagingDocument {
    document: DOM,
    lifecycle: Box<dyn MarkupSanitizationLifecycle>,
}
impl Drop for StagingDocument {
    fn drop(&mut self) {
        self.lifecycle.Shutdown(&mut self.document);
        self.document.GetDocumentMut().UnbindDOMWindow();
        self.document.GetDocument().SetLifecycleActive(false);
    }
}
fn ContainsStyleElements(document: &Document, node: usize) -> bool {
    document.Node(node).IsStyleElement()
        || document
            .Node(node)
            .Children()
            .iter()
            .any(|child| ContainsStyleElements(document, *child))
}
/// SVGURLReferenceResolver::IsLocal plus serialization.cc:835–852. The return
/// value intentionally reports any <use>, including a retained local reference.
fn StripSvgUseNonLocalHrefs(document: &mut Document, node: usize) -> bool {
    let element = document.Node(node);
    if element.Namespace() == DOMNamespace::kSVG && element.Name() == "use" {
        let href = element
            .Attributes()
            .iter()
            .find(|attribute| attribute.local_name == "href" && attribute.namespace_uri.is_empty())
            .or_else(|| {
                element.Attributes().iter().find(|attribute| {
                    attribute.local_name == "href"
                        && attribute.namespace_uri == "http://www.w3.org/1999/xlink"
                })
            })
            .map_or("", |attribute| attribute.value.as_str());
        let absolute = document.CompleteURLValue(href);
        let absolute = absolute.AsStr();
        let local = href.starts_with('#')
            || absolute.split('#').next() == document.DocumentURL().split('#').next();
        if !local
            || absolute
                .split(':')
                .next()
                .is_some_and(|scheme| scheme.eq_ignore_ascii_case("data"))
        {
            document.Remove(node);
        }
        return true;
    }
    // Only the sibling indices are retained across removal; no copied tree.
    let mut child = document.Node(node).Children().first().copied();
    let mut found = false;
    while let Some(current) = child {
        let siblings = document.Node(node).Children();
        let position = siblings
            .iter()
            .position(|index| *index == current)
            .expect("current child");
        child = siblings.get(position + 1).copied();
        found |= StripSvgUseNonLocalHrefs(document, current);
    }
    found
}
impl MarkupSanitizationContext for StagingDocument {
    type Fragment = usize;
    type Error = FragmentError;
    fn ParseWithContext(
        &mut self,
        markup: &[u16],
        start: usize,
        end: usize,
    ) -> Result<usize, FragmentError> {
        markup_fragment::CreateFragmentFromMarkupWithContext(
            self.document.GetDocumentMut(),
            markup,
            start,
            end,
            "",
            ParserContentPolicy::DisallowScriptingAndPluginContent,
            true,
        )
        .map_err(|_| FragmentError::InvalidOffsets)?
        .ok_or(FragmentError::ParseFailed)
    }
    fn ContainsStyleElements(&self, fragment: &usize) -> bool {
        ContainsStyleElements(self.document.GetDocument(), *fragment)
    }
    fn StripSvgUseNonLocalHrefs(&mut self, fragment: &usize) -> bool {
        StripSvgUseNonLocalHrefs(self.document.GetDocumentMut(), *fragment)
    }
    fn Serialize(
        &mut self,
        fragment: &usize,
        inline_styles: bool,
    ) -> Result<Vec<u16>, FragmentError> {
        if !inline_styles {
            return Ok(markup_accumulator::CreateMarkup(
                &NativeMarkupDocument(self.document.GetDocument()),
                *fragment,
                false,
                ResolveUrls::None,
            ));
        }
        let body = self
            .document
            .GetDocument()
            .BodyElement()
            .ok_or(FragmentError::ParseFailed)?;
        self.document.GetDocumentMut().AppendChild(body, *fragment);
        let result = self
            .lifecycle
            .SerializeBodyWithInlineStyles(&mut self.document, body);
        // Source removes body children after each styled serialization, even
        // when this embedding cannot represent a required CSS resource value.
        while let Some(child) = self
            .document
            .GetDocument()
            .Node(body)
            .Children()
            .first()
            .copied()
        {
            self.document.GetDocumentMut().Remove(child);
        }
        result
    }
    fn ParseAndSerializeFinal(
        &mut self,
        markup: &[u16],
        base_url: &str,
    ) -> Result<Vec<u16>, FragmentError> {
        let fragment = markup_fragment::CreateFragmentFromMarkup(
            self.document.GetDocumentMut(),
            markup,
            base_url,
            ParserContentPolicy::DisallowScriptingAndPluginContent,
            true,
        );
        Ok(markup_accumulator::CreateMarkup(
            &NativeMarkupDocument(self.document.GetDocument()),
            fragment,
            false,
            ResolveUrls::None,
        ))
    }
}

pub fn CreateStrictlyProcessedMarkupWithContext(
    host: &dyn MarkupSanitizationHost,
    markup: &[u16],
    start: usize,
    end: usize,
    base_url: &str,
) -> Result<Option<Vec<u16>>, FragmentError> {
    if markup.is_empty() {
        return Ok(None);
    }
    let document = CreateStagingDocumentForMarkupSanitization();
    let lifecycle = host.CreateLifecycle(&document);
    let mut staging = StagingDocument {
        document,
        lifecycle,
    };
    serialization::CreateStrictlyProcessedMarkupWithContext(
        &mut staging,
        markup,
        start,
        end,
        base_url,
    )
}
/// Source creates the final native fragment in the live target Document only
/// after stabilization. The staging arena and every compiled style stay private.
pub fn CreateStrictlyProcessedFragmentFromMarkupWithContext(
    host: &dyn MarkupSanitizationHost,
    target: &InteractionDocument,
    markup: &[u16],
    start: usize,
    end: usize,
    base_url: &str,
) -> Result<Option<RetainedFragment>, FragmentError> {
    let identity = {
        let owner = target.borrow();
        let document = owner.GetDocument();
        if !document.IsActive() {
            return Err(FragmentError::DocumentChanged);
        }
        document.RootHandle().Identity()
    };
    let Some(markup) = CreateStrictlyProcessedMarkupWithContext(host, markup, start, end, "")?
    else {
        return Ok(None);
    };
    let mut owner = target.borrow_mut();
    let document = owner.GetDocumentMut();
    if !document.IsActive() || document.RootHandle().Identity() != identity {
        return Err(FragmentError::DocumentChanged);
    }
    let scripting_flag = document.ScriptingEnabled();
    let fragment = markup_fragment::CreateFragmentFromMarkup(
        document,
        &markup,
        base_url,
        ParserContentPolicy::DisallowScriptingAndPluginContent,
        scripting_flag,
    );
    Ok(Some(RetainedFragment {
        root: document.Node(fragment).Id(),
        document: target.clone(),
        identity,
    }))
}
