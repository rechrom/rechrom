//! Host-side completion of pure font cache misses, before layout publication.
#[cfg(test)]
use font_engine::FontEngine;
use font_engine::{
    FontContextVersion, FontEngineHandle, FontMutation, SystemFontProvider, SystemFontResponse,
};
use layoutng_assembly::internal::layout_input::ConstraintSpace;

pub(crate) struct SystemFontSession {
    engine: FontEngineHandle,
    provider: Box<dyn SystemFontProvider>,
}
impl SystemFontSession {
    #[cfg(test)]
    pub(crate) fn with_provider(provider: Box<dyn SystemFontProvider>) -> Self {
        Self::with_engine(
            FontEngineHandle::new(FontEngine::new(FontContextVersion::default(), None)),
            provider,
        )
    }
    pub(crate) fn with_engine(
        engine: FontEngineHandle,
        provider: Box<dyn SystemFontProvider>,
    ) -> Self {
        Self { engine, provider }
    }
    pub(crate) fn SetProvider(
        &mut self,
        mut provider: Box<dyn SystemFontProvider>,
        constraints: &mut ConstraintSpace,
    ) {
        self.EnvironmentChanged(constraints);
        provider.SetContextVersion(self.engine.with_mut(|engine| engine.ContextVersion()));
        self.provider = provider;
    }
    pub(crate) fn EnvironmentChanged(&mut self, constraints: &mut ConstraintSpace) {
        let version = FontContextVersion(
            self.engine
                .with_mut(|engine| engine.ContextVersion())
                .0
                .checked_add(1)
                .expect("font context version exhausted"),
        );
        self.provider.SetContextVersion(version);
        self.engine
            .with_mut(|engine| engine.ApplyMutation(FontMutation::EnvironmentChanged(version)))
            .expect("font environment mutation");
        constraints.fonts = self.engine.with_mut(|engine| engine.ResolvedFaces());
        if let Some(collector) = &constraints.font_queries {
            collector.Clear();
        }
    }
    /// Returns true only when new immutable faces were admitted. A answered
    /// negative query never drives another layout; cached replies never perform
    /// platform I/O twice. No Page callback or paint runs in this transaction.
    pub(crate) fn CompletePass(&mut self, constraints: &mut ConstraintSpace) -> bool {
        let Some(collector) = &constraints.font_queries else {
            return false;
        };
        let previous_revision = collector.Revision();
        self.engine
            .with_mut(|engine| engine.SetQueryCollector(Some(collector.clone())));
        let queries = collector.Drain();
        // An ordinary layout pass has no font-host work. Keep the exact shared
        // catalog generation already in ConstraintSpace and avoid rebuilding
        // or comparing any face metadata on the hot lifecycle path.
        if queries.is_empty() {
            return false;
        }
        for query in queries {
            // Discovery and exact-open are the two protocol phases. Stop
            // after open so a real shaping retry, not this host, decides whether
            // the next contextual fallback candidate is needed.
            loop {
                let pending = self
                    .engine
                    .with_mut(|engine| engine.PrepareSystemQuery(query.clone()));
                if pending.system_queries.is_empty() {
                    break;
                }
                let opened = pending
                    .system_queries
                    .iter()
                    .any(|r| matches!(r.query, font_engine::SystemFontQuery::OpenFace { .. }));
                let mut admitted = false;
                for request in pending.system_queries {
                    let answer = self.provider.Resolve(&request);
                    admitted |= matches!(&answer, font_engine::SystemFontAnswer::Face(face) if !face.bytes.is_empty());
                    self.engine
                        .with_mut(|engine| {
                            engine.ApplyMutation(FontMutation::SystemFontResponse(
                                SystemFontResponse { request, answer },
                            ))
                        })
                        .expect("host font response must match its pending request and version");
                }
                if opened && admitted {
                    break;
                }
            }
        }
        // FontFaceId in shaped/paint output is the index in FontEngine's
        // canonical catalog. Publish that exact order instead of incrementally
        // appending faces in a host-owned order.
        let resolved = self.engine.with_mut(|engine| engine.ResolvedFaces());
        let changed = constraints.fonts != resolved;
        constraints.fonts = resolved;
        changed || collector.Revision() != previous_revision
    }
}

/// The provisional result stays private to LayoutEngine. Font-sensitive caches
/// are invalidated through the existing constraints/document mutation gates;
/// only the final pass can reach measurement, container sizes, Paint or clients.
pub(crate) fn LayoutWithSystemFonts(
    layout: &mut layoutng_assembly::layout_engine::LayoutEngine,
    document: &mut dom::DOM,
    interaction: &dom::UserInteractionState,
    constraints: &mut ConstraintSpace,
    fonts: &mut SystemFontSession,
) {
    LayoutWithSystemFontsAtStage(
        layout,
        document,
        interaction,
        constraints,
        fonts,
        layoutng_assembly::layout_boundary_support::LayoutUpdateKind::Paint,
    );
}

pub(crate) fn LayoutWithSystemFontsAtStage(
    layout: &mut layoutng_assembly::layout_engine::LayoutEngine,
    document: &mut dom::DOM,
    interaction: &dom::UserInteractionState,
    constraints: &mut ConstraintSpace,
    fonts: &mut SystemFontSession,
    purpose: layoutng_assembly::layout_boundary_support::LayoutUpdateKind,
) {
    loop {
        super::persistent_layout::LayoutDocumentWithEngineAtStage(
            layout,
            document,
            interaction,
            constraints,
            purpose,
        );
        if !fonts.CompletePass(constraints) {
            break;
        }
        layout.ReleaseLayoutResult();
        // Chromium's font invalidation dirties text layout while retaining the
        // attached LayoutObject tree. The provider/catalog changed underneath
        // the same typed style inputs, so only geometry needs another pass.
        document.InvalidateLayoutGeometry();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use font_engine::{
        FontFace, FontFallbackRole, FontQueryCollector, SystemFontAnswer, SystemFontFace,
        SystemFontFaceId, SystemFontQuery, SystemFontRequest,
    };
    use std::{cell::RefCell, rc::Rc};
    struct Provider {
        calls: Rc<RefCell<Vec<SystemFontQuery>>>,
    }
    impl SystemFontProvider for Provider {
        fn Resolve(&mut self, request: &SystemFontRequest) -> SystemFontAnswer {
            self.calls.borrow_mut().push(request.query.clone());
            match &request.query {
                SystemFontQuery::MatchFamily { family, .. } if family == "RequestedFont" => {
                    SystemFontAnswer::Faces(vec![SystemFontFace {
                        id: SystemFontFaceId("test-exact".into()),
                        face: FontFace {
                            family: family.clone(),
                            ..Default::default()
                        },
                    }])
                }
                SystemFontQuery::OpenFace { .. } => SystemFontAnswer::Face(FontFace {
                    family: "Real Native Name".into(),
                    bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
                        .as_slice()
                        .into(),
                    ..Default::default()
                }),
                _ => SystemFontAnswer::Unavailable,
            }
        }
    }
    #[test]
    fn query_completion_preserves_alias_and_negative_answers_terminate() {
        crate::native_test_thread::run(|| {
            let calls = Rc::new(RefCell::new(vec![]));
            let mut session = SystemFontSession::with_provider(Box::new(Provider {
                calls: calls.clone(),
            }));
            let mut constraints = ConstraintSpace::default();
            let collector = FontQueryCollector::default();
            constraints.font_queries = Some(collector.clone());
            let query = SystemFontQuery::MatchFamily {
                family: "RequestedFont".into(),
                weight: 400.,
                italic: false,
                locale: "en".into(),
            };
            collector.Push(query.clone());
            assert!(session.CompletePass(&mut constraints));
            assert_eq!(calls.borrow().len(), 2, "metadata then exact byte open");
            assert_eq!(constraints.fonts[0].family, "RequestedFont");
            assert!(!constraints.fonts[0].bytes.is_empty());
            collector.Push(query);
            assert!(!session.CompletePass(&mut constraints));
            assert_eq!(calls.borrow().len(), 2, "known answers do not perform I/O");
            let missing = SystemFontQuery::MatchUniqueName {
                name: "no such face".into(),
            };
            collector.Push(missing.clone());
            assert!(!session.CompletePass(&mut constraints));
            collector.Push(missing);
            assert!(!session.CompletePass(&mut constraints));
            assert_eq!(
                calls.borrow().len(),
                3,
                "negative reply is terminal and cached"
            );
        });
    }
    #[test]
    fn layout_transaction_retries_before_exposing_final_font_resources() {
        crate::native_test_thread::run(|| {
            let calls = Rc::new(RefCell::new(vec![]));
            let mut fonts = SystemFontSession::with_provider(Box::new(Provider {
                calls: calls.clone(),
            }));
            let mut constraints = crate::CreateBrowserConstraints(320, 120);
            constraints.fonts[0].fallback_role = FontFallbackRole::kLastResort;
            let mut document = html::html_parser::ParseHTML(
                "<style>div{font:16px RequestedFont}</style><div>office</div>",
            );
            document.GetDocumentMut().AppendStyleSheet(style::ParseCSS(
                "div {font-family:RequestedFont;font-size:16px}",
            ));
            crate::style_services::ResolveLayoutStyles(&mut document, &constraints);
            let mut layout =
                layoutng_assembly::layout_engine::LayoutEngine::new(&crate::CreateLayoutAssembly());
            LayoutWithSystemFonts(
                &mut layout,
                &mut document,
                &dom::UserInteractionState::default(),
                &mut constraints,
                &mut fonts,
            );
            assert!(calls
                .borrow()
                .iter()
                .any(|q| matches!(q, SystemFontQuery::OpenFace { .. })));
            assert!(constraints
                .fonts
                .iter()
                .any(|f| f.family == "RequestedFont" && !f.bytes.is_empty()));
            let final_result = layout
                .TakeLayoutResult()
                .expect("only settled layout returned");
            fn selected(node: &layoutng_assembly::fragment_tree::FragmentNode) -> bool {
                node.paint
                    .glyph_runs
                    .iter()
                    .any(|run| run.font_face_index > 0)
                    || node.children.iter().any(selected)
            }
            assert!(
                selected(&final_result),
                "final glyphs select admitted exact face, not emergency .notdef"
            );
            let before = calls.borrow().len();
            LayoutWithSystemFonts(
                &mut layout,
                &mut document,
                &dom::UserInteractionState::default(),
                &mut constraints,
                &mut fonts,
            );
            assert_eq!(before, calls.borrow().len(), "stable pass is terminal");
        });
    }
}
