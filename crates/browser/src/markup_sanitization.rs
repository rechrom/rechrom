// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Private Style/Layout assembly for serialization.cc's nonordinary staging
//! document. This lifecycle has a fixed viewport and no resource/script loader,
//! Paint, Compositor, render target, scheduler, or Page presentation owner.
use dom::{Document, DOM};
use interaction::adapter::{
    page_markup_sanitization::{MarkupSanitizationHost, MarkupSanitizationLifecycle},
    page_selection_markup::{FragmentError, MarkupStyleQueries, SerializeMarkupBody},
};
use layoutng_assembly::{internal::layout_input::ConstraintSpace, layout_engine::LayoutEngine};
use std::{cell::RefCell, rc::Rc};

pub(crate) struct BrowserMarkupSanitizationHost;
impl MarkupSanitizationHost for BrowserMarkupSanitizationHost {
    fn CreateLifecycle(&self, document: &DOM) -> Box<dyn MarkupSanitizationLifecycle> {
        // Source LocalFrameView uses 800x600 to avoid exposing host viewport
        // dimensions to unsanitized selectors and viewport-dependent CSS.
        let mut constraints = crate::CreateBrowserConstraints(800, 600);
        let mut fonts = font_engine::FontEngine::new(
            font_engine::FontContextVersion::default(),
            constraints.font_backend_factory.clone(),
        );
        fonts
            .ApplyMutation(font_engine::FontMutation::SetFaces(
                constraints.fonts.iter().cloned().collect(),
            ))
            .expect("staging font catalog");
        // This private document admits no URL/resource effects. Its already
        // decoded emergency face supplies ordinary shaping without I/O.
        constraints.font_queries = None;
        constraints.font_engine = Some(font_engine::FontEngineHandle::new(fonts));
        let font_metrics = Rc::new(crate::style_services::StyleFontMetricsResolver::new(
            &constraints,
        ));
        let mut style = style::StyleEngine::new(document);
        style.SetFontMetricsResolver(font_metrics);
        Box::new(StagingLifecycle {
            style: RefCell::new(style),
            layout: LayoutEngine::new(&crate::CreateLayoutAssembly()),
            constraints,
            user_agent: crate::user_agent_styles::UserAgentStyleSheets::new(),
        })
    }
}
struct StagingLifecycle {
    style: RefCell<style::StyleEngine>,
    layout: LayoutEngine,
    constraints: ConstraintSpace,
    user_agent: crate::user_agent_styles::UserAgentStyleSheets,
}
struct StagingStyleQueries<'a> {
    style: &'a RefCell<style::StyleEngine>,
    document: &'a Document,
}
impl MarkupStyleQueries for StagingStyleQueries<'_> {
    fn Query(
        &self,
        operation: cssom::editing_style::SerializationStyleOperation,
    ) -> Result<cssom::CSSInlineStyle, cssom::editing_style::EditingStyleError> {
        style::editing_style::QuerySerializationEditingStyle(
            &mut self.style.borrow_mut(),
            self.document,
            operation,
            None,
        )
    }
}
impl MarkupSanitizationLifecycle for StagingLifecycle {
    fn SerializeBodyWithInlineStyles(
        &mut self,
        document: &mut DOM,
        body: usize,
    ) -> Result<Vec<u16>, FragmentError> {
        let mut heap = foundation::LayoutHeapScope::new();
        heap.DeferCollection();
        document.GetDocumentMut().UpdateSVGUseInstances();
        let mut media = crate::style_services::MediaValuesForLayout(
            &self.constraints,
            style::PreferredColorScheme::kLight,
            style::media_queries::forced_colors::ForcedColors::kNone,
        );
        media.scripting = style::media_queries::scripting::Scripting::kNone;
        self.style
            .borrow_mut()
            .Update(
                document,
                &media,
                self.user_agent.ForDocument(document.GetDocument()),
            )
            .map_err(|_| FragmentError::RequiresStyleSanitization)?;
        crate::persistent_layout::LayoutDocumentWithEngineAtStage(
            &mut self.layout,
            document,
            &dom::UserInteractionState::default(),
            &self.constraints,
            layoutng_assembly::layout_boundary_support::LayoutUpdateKind::Layout,
        );
        let output = self.layout.Output().ok_or(FragmentError::ParseFailed)?;
        let styles = StagingStyleQueries {
            style: &self.style,
            document: document.GetDocument(),
        };
        SerializeMarkupBody(document.GetDocument(), output, &styles, body)
            .map_err(|error| match error {
                interaction::default::core::editing::serializers::styled_markup_serializer::SerializationError::UnsupportedResource => FragmentError::UnsupportedResource,
                _ => FragmentError::RequiresStyleSanitization,
            })
    }
    fn Shutdown(&mut self, document: &mut DOM) {
        // Document::Shutdown detaches Layout before detaching the Style owner.
        let mut heap = foundation::LayoutHeapScope::new();
        heap.DeferCollection();
        crate::persistent_layout::DetachDocumentLayoutTreeWithEngine(&mut self.layout, document);
        self.style.borrow_mut().DidDetach();
    }
}
