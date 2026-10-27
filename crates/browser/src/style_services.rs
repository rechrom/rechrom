#![allow(non_snake_case)]
use crate::user_agent_styles::UserAgentStyleSheets;
use dom::{Document, UserInteractionState, DOM};
use layoutng_assembly::internal::layout_input::ConstraintSpace;
use std::{cell::RefCell, rc::Rc};
use style::media_queries::{media_values_cached::MediaValuesCachedData, scripting::Scripting};
use webapi::dom_bindings::DOMBindingsHost;

struct StyleFontMetricsState {
    engine: font_engine::FontEngineHandle,
    fonts: font_engine::FontFaceCatalog,
    backend: Option<std::sync::Arc<dyn font_engine::FontBackendFactory>>,
    queries: Option<font_engine::FontQueryCollector>,
    metrics_pending: bool,
}

pub(crate) struct StyleFontMetricsResolver {
    state: RefCell<StyleFontMetricsState>,
}

impl StyleFontMetricsResolver {
    fn Engine(constraints: &ConstraintSpace) -> font_engine::FontEngineHandle {
        constraints.font_engine.clone().unwrap_or_else(|| {
            let mut engine = font_engine::FontEngine::FromFaces(
                &constraints.fonts,
                constraints.font_backend_factory.clone(),
            );
            engine.SetQueryCollector(constraints.font_queries.clone());
            font_engine::FontEngineHandle::new(engine)
        })
    }

    pub(crate) fn new(constraints: &ConstraintSpace) -> Self {
        let engine = Self::Engine(constraints);
        Self {
            state: RefCell::new(StyleFontMetricsState {
                engine,
                fonts: constraints.fonts.clone(),
                backend: constraints.font_backend_factory.clone(),
                queries: constraints.font_queries.clone(),
                metrics_pending: false,
            }),
        }
    }

    /// Blink's document StyleEngine retains FontCache/Font objects across
    /// incremental style updates. Rebuild only when the immutable catalog or
    /// host service identity changes; ordinary DOM/style invalidations keep
    /// parsed faces, native backends and resolved FontDescription entries.
    pub(crate) fn Update(&self, constraints: &ConstraintSpace) {
        fn same_arc<T: ?Sized>(
            a: &Option<std::sync::Arc<T>>,
            b: &Option<std::sync::Arc<T>>,
        ) -> bool {
            match (a, b) {
                (Some(a), Some(b)) => std::sync::Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
        }
        let mut state = self.state.borrow_mut();
        let same_engine = constraints
            .font_engine
            .as_ref()
            .is_some_and(|engine| engine.ptr_eq(&state.engine));
        let same_queries = match (&state.queries, &constraints.font_queries) {
            (Some(a), Some(b)) => a.ptr_eq(b),
            (None, None) => true,
            _ => false,
        };
        if (same_engine || state.fonts == constraints.fonts)
            && same_arc(&state.backend, &constraints.font_backend_factory)
            && same_queries
        {
            return;
        }
        let engine = Self::Engine(constraints);
        *state = StyleFontMetricsState {
            engine,
            fonts: constraints.fonts.clone(),
            backend: constraints.font_backend_factory.clone(),
            queries: constraints.font_queries.clone(),
            metrics_pending: false,
        };
    }

    /// A pending metrics lookup means Style consumed a provisional CSS
    /// font-relative metric and must be recalculated after the host admits the
    /// face. A WillUse-only miss can proceed directly to Layout.
    pub(crate) fn TakeMetricsPending(&self) -> bool {
        std::mem::take(&mut self.state.borrow_mut().metrics_pending)
    }

    fn QueueSystemQueries(&self, pending: &font_engine::FontPending) {
        let state = self.state.borrow();
        let Some(queries) = &state.queries else {
            return;
        };
        for request in &pending.system_queries {
            queries.Push(request.query.clone());
        }
    }
}

impl style::resolver::production_style_builder::FontMetricsResolver for StyleFontMetricsResolver {
    fn WillUseFont(&self, description: &font_engine::FontDescription) {
        let engine = self.state.borrow().engine.clone();
        let result = engine.with_mut(|engine| {
            let version = engine.ContextVersion();
            engine.GetMetrics(description, version)
        });
        if let font_engine::FontCompute::Pending(pending) = result {
            self.QueueSystemQueries(&pending);
        }
    }

    fn Metrics(
        &self,
        description: &font_engine::FontDescription,
    ) -> Option<font_engine::FontMetrics> {
        let state = self.state.borrow();
        let engine = state.engine.clone();
        drop(state);
        let result = engine.with_mut(|engine| {
            let version = engine.ContextVersion();
            engine.GetMetrics(description, version)
        });
        match result {
            font_engine::FontCompute::Ready(metrics) => Some(metrics),
            font_engine::FontCompute::Pending(pending) => {
                self.QueueSystemQueries(&pending);
                self.state.borrow_mut().metrics_pending = true;
                None
            }
            font_engine::FontCompute::Error(_) => None,
        }
    }
}

pub(crate) fn MediaValuesForLayout(
    constraints: &ConstraintSpace,
    preferred_color_scheme: style::PreferredColorScheme,
    forced_colors: style::media_queries::forced_colors::ForcedColors,
) -> MediaValuesCachedData {
    let width = constraints.available_size.width;
    let height = constraints.available_size.height;
    MediaValuesCachedData {
        viewport_width: width,
        viewport_height: height,
        small_viewport_width: width,
        small_viewport_height: height,
        large_viewport_width: width,
        large_viewport_height: height,
        dynamic_viewport_width: width,
        dynamic_viewport_height: height,
        device_width: width.round() as i32,
        device_height: height.round() as i32,
        // LocalFrame::DevicePixelRatio is inspector/host DSF multiplied by
        // LayoutZoomFactor. Media resolution and image selection consume this
        // effective value once from the shared immutable environment.
        device_pixel_ratio: (constraints.device_pixel_ratio
            * constraints.layout_zoom_factor.max(f64::MIN_POSITIVE))
            as f32,
        layout_zoom_factor: constraints.layout_zoom_factor as f32,
        em_size: 16.0,
        media_type: foundation::String::from(if constraints.printing {
            "print"
        } else {
            "screen"
        }),
        preferred_color_scheme,
        forced_colors,
        scripting: Scripting::kEnabled,
        ..Default::default()
    }
}

pub(crate) fn ResponsiveMediaMatches(
    query: &str,
    values: &style::media_queries::MediaValuesCached,
) -> bool {
    if query.trim().is_empty() {
        return true;
    }
    let set = style::media_queries::media_query_backend::ParseMediaQuerySet(query);
    style::media_queries::media_query_backend::ForMediaValues(values).Eval(&set)
}

fn SupportedResponsiveImageType(value: Option<&str>) -> bool {
    // ContentType(type).GetType(): MIME parameters do not participate in the
    // supported-image registry lookup.
    let Some(value) = value
        .map(|value| value.split_once(';').map_or(value, |(mime, _)| mime))
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return true;
    };
    matches!(
        value.to_ascii_lowercase().as_str(),
        "image/avif"
            | "image/bmp"
            | "image/gif"
            | "image/jpeg"
            | "image/jpg"
            | "image/pjpeg"
            | "image/png"
            | "image/apng"
            | "image/svg+xml"
            | "image/vnd.microsoft.icon"
            | "image/webp"
            | "image/x-icon"
            | "image/x-png"
            | "image/x-xbitmap"
    )
}

// cpp: html_image_element.cc SelectSourceURL/ImageSourceURL and
// html_srcset_parser.cc BestFitSourceForImageAttributes. Browser is the
// composition root: HTML owns source-list parsing, Style owns CSS media and
// length evaluation, and ResourceFetcher receives only the selected URL/DPR.
pub(crate) fn SelectResponsiveImage(
    document: &Document,
    index: usize,
    constraints: &ConstraintSpace,
    preferred_color_scheme: style::PreferredColorScheme,
    forced_colors: style::media_queries::forced_colors::ForcedColors,
    is_loaded: &dyn Fn(&str) -> bool,
) -> Option<dom::persistent_document::ResponsiveImageSelection> {
    let image = document.Node(index);
    let attribute = |name| {
        image
            .FindAttribute(name)
            .map(|attribute| attribute.value.clone())
    };
    let allows_auto_sizes = document.AllowsResponsiveImageAutoSizes(index);
    let mut input = document_loader::ResponsiveImageInput {
        src: attribute("src"),
        srcset: attribute("srcset"),
        sizes: attribute("sizes"),
        allows_auto_sizes,
        auto_size: allows_auto_sizes
            .then(|| document.ResponsiveImageAutoSize(index))
            .flatten(),
        lazy: attribute("loading").is_some_and(|value| value.eq_ignore_ascii_case("lazy")),
        ..Default::default()
    };
    if let Some(parent) = image
        .Parent()
        .filter(|parent| document.Node(*parent).IsHTMLElement("picture"))
    {
        for &child in document.Node(parent).Children() {
            if child == index {
                break;
            }
            let source = document.Node(child);
            if !source.IsHTMLElement("source") {
                continue;
            }
            let attribute = |name| {
                source
                    .FindAttribute(name)
                    .map(|attribute| attribute.value.clone())
            };
            input
                .picture_sources
                .push(document_loader::ResponsiveImageSourceInput {
                    srcset: attribute("srcset"),
                    sizes: attribute("sizes"),
                    media: attribute("media"),
                    image_type: attribute("type"),
                });
        }
    }
    SelectResponsiveImageFromAttributes(
        &input,
        constraints,
        preferred_color_scheme,
        forced_colors,
        is_loaded,
    )
}

/// The native srcset/sizes/media path is shared by HTMLImageElement and the
/// preload scanner, so speculation does not pick a different density/source.
/// Loading policy is handled by the request owner; this entry only selects.
pub(crate) fn SelectResponsiveImageFromAttributes(
    input: &document_loader::ResponsiveImageInput,
    constraints: &ConstraintSpace,
    preferred_color_scheme: style::PreferredColorScheme,
    forced_colors: style::media_queries::forced_colors::ForcedColors,
    is_loaded: &dyn Fn(&str) -> bool,
) -> Option<dom::persistent_document::ResponsiveImageSelection> {
    use style::media_queries::MediaValues;

    let media = MediaValuesForLayout(constraints, preferred_color_scheme, forced_colors);
    let values = style::media_queries::MediaValuesCached::new(&media);
    let fallback = input.src.as_deref();
    let image_sizes = input.sizes.as_deref();
    let auto_size = input.allows_auto_sizes.then_some(input.auto_size).flatten();
    let source_size = |sizes: Option<&str>| {
        html::parser::sizes_attribute_parser::ParseSizesAttribute(
            sizes,
            values.Width().unwrap_or(media.viewport_width),
            auto_size,
            |component| {
                use html::parser::sizes_attribute_parser::SizesComponent;
                match style::responsive_image::EvaluateSizesComponent(component, &values) {
                    style::responsive_image::SizesEvaluation::Auto => SizesComponent::Auto,
                    style::responsive_image::SizesEvaluation::Length(length) => {
                        SizesComponent::Length(length)
                    }
                    style::responsive_image::SizesEvaluation::Invalid => SizesComponent::Invalid,
                    style::responsive_image::SizesEvaluation::FatalInvalid => {
                        SizesComponent::FatalInvalid
                    }
                }
            },
        )
    };
    for source in &input.picture_sources {
        if !ResponsiveMediaMatches(source.media.as_deref().unwrap_or(""), &values)
            || !SupportedResponsiveImageType(source.image_type.as_deref())
        {
            continue;
        }
        let Some(srcset) = source.srcset.as_deref().filter(|value| !value.is_empty()) else {
            continue;
        };
        let sizes = source.sizes.as_deref();
        let sizes = if sizes.is_none() && input.allows_auto_sizes {
            Some("auto")
        } else {
            sizes
        };
        if let Some(candidate) =
            html::parser::html_srcset_parser::BestFitSourceForImageAttributesWithCache(
                values.DevicePixelRatio() as f64,
                source_size(sizes),
                None,
                Some(srcset),
                is_loaded,
            )
        {
            return Some(dom::persistent_document::ResponsiveImageSelection {
                source: candidate.source,
                density: Some(candidate.density),
            });
        }
    }
    let Some(srcset) = input.srcset.as_deref() else {
        return fallback.filter(|source| !source.is_empty()).map(|source| {
            dom::persistent_document::ResponsiveImageSelection {
                source: source.to_owned(),
                density: None,
            }
        });
    };
    let candidate = html::parser::html_srcset_parser::BestFitSourceForImageAttributesWithCache(
        values.DevicePixelRatio() as f64,
        source_size(image_sizes),
        fallback,
        Some(srcset),
        is_loaded,
    )?;
    Some(dom::persistent_document::ResponsiveImageSelection {
        source: candidate.source,
        density: Some(candidate.density),
    })
}

// cpp: browser/browser.cc:641-648,932-939
// The callback adapter measures on demand. Page's mutation-invalidated cache
// will own these same inputs; no cached snapshot is used by this adapter.
pub fn CreateLayoutBindingsHost(
    document: Rc<RefCell<DOM>>,
    constraints: Rc<RefCell<ConstraintSpace>>,
    interaction: Rc<RefCell<UserInteractionState>>,
) -> DOMBindingsHost {
    let resolve_document = document.clone();
    let resolve_constraints = constraints.clone();
    let engine = Rc::new(RefCell::new(crate::LayoutEngine::new(
        &crate::CreateLayoutAssembly(),
    )));
    let measure = Rc::new(move || {
        let constraints = constraints.borrow();
        let mut document = document.borrow_mut();
        ResolveLayoutStyles(&mut document, &constraints);
        let mut layout = engine.borrow_mut();
        crate::persistent_layout::LayoutDocumentWithEngineAtStage(
            &mut layout,
            &mut document,
            &interaction.borrow(),
            &constraints,
            layoutng_assembly::layout_boundary_support::LayoutUpdateKind::Layout,
        );
        drop(layout);
        engine.clone()
    });
    let geometry = measure.clone();
    DOMBindingsHost {
        update_style: Some(Box::new(move |id| {
            let mut document = resolve_document.borrow_mut();
            let constraints = resolve_constraints.borrow();
            let mut engine = style::StyleEngine::new(&document);
            ResolveLayoutStylesWithEngine(
                &mut engine,
                &mut document,
                &constraints,
                style::PreferredColorScheme::kLight,
            );
            if let Some(node) = document.GetDocument().FindNodeById(id) {
                EnsureComputedStyleForNode(
                    &mut engine,
                    &mut document,
                    &constraints,
                    style::PreferredColorScheme::kLight,
                    style::media_queries::forced_colors::ForcedColors::kNone,
                    node,
                );
            }
        })),
        read_metric: Some(Box::new(move |id, name| {
            let layout = measure();
            let layout = layout.borrow();
            let metric = layoutng_assembly::layout_engine::LayoutMetric::try_from(name).ok()?;
            layout
                .Output()?
                .Node(id)
                .map_or(Some(0.), |node| node.Metric(metric))
        })),
        read_geometry: Some(Box::new(move |id| {
            let layout = geometry();
            let layout = layout.borrow();
            layout
                .Output()
                .and_then(|output| output.Node(id))
                .map_or_else(Vec::new, |node| {
                    node.ClientRects()
                        .into_iter()
                        .map(|r| paint::paint_engine::PaintRect {
                            x: r.x,
                            y: r.y,
                            width: r.width,
                            height: r.height,
                        })
                        .collect()
                })
        })),
        ..Default::default()
    }
}

// cpp: browser/browser.cc:1748-1758
pub(crate) fn ResolveLayoutStyles(owner: &mut DOM, constraints: &ConstraintSpace) {
    let mut engine = style::StyleEngine::new(owner);
    ResolveLayoutStylesWithEngine(
        &mut engine,
        owner,
        constraints,
        style::PreferredColorScheme::kLight,
    );
}

pub(crate) fn ResolveLayoutStylesWithEngine(
    engine: &mut style::StyleEngine,
    owner: &mut DOM,
    constraints: &ConstraintSpace,
    preferred_color_scheme: style::PreferredColorScheme,
) {
    ResolveLayoutStylesWithEngineAndForcedColors(
        engine,
        owner,
        constraints,
        preferred_color_scheme,
        style::media_queries::forced_colors::ForcedColors::kNone,
    )
}

pub(crate) fn ResolveLayoutStylesWithEngineAndForcedColors(
    engine: &mut style::StyleEngine,
    owner: &mut DOM,
    constraints: &ConstraintSpace,
    preferred_color_scheme: style::PreferredColorScheme,
    forced_colors: style::media_queries::forced_colors::ForcedColors,
) {
    let font_metrics = Rc::new(StyleFontMetricsResolver::new(constraints));
    ResolveLayoutStylesWithEngineAndForcedColorsUsingFontMetrics(
        engine,
        owner,
        constraints,
        preferred_color_scheme,
        forced_colors,
        font_metrics,
    );
}

pub(crate) fn ResolveLayoutStylesWithEngineAndForcedColorsUsingFontMetrics(
    engine: &mut style::StyleEngine,
    owner: &mut DOM,
    constraints: &ConstraintSpace,
    preferred_color_scheme: style::PreferredColorScheme,
    forced_colors: style::media_queries::forced_colors::ForcedColors,
    font_metrics: Rc<StyleFontMetricsResolver>,
) {
    // Blink style resolution allocates ComputedStyle and its rare-data on the
    // document's cppgc heap before Layout begins. Keep that owner-thread heap
    // boundary in the browser assembly layer; StyleEngine itself remains
    // independent of the host event loop and thread placement.
    let mut heap_scope = foundation::LayoutHeapScope::new();
    let user_agent = UserAgentStyleSheets::new();
    let environment = MediaValuesForLayout(constraints, preferred_color_scheme, forced_colors);
    let sheets = user_agent.ForDocument(owner.GetDocument());
    font_metrics.Update(constraints);
    engine.SetFontMetricsResolver(font_metrics);
    engine
        .Update(owner, &environment, sheets)
        .unwrap_or_else(|error| panic!("StyleEngine update failed: {error}"));
    heap_scope.DeferCollection();
}

pub(crate) fn EnsureComputedStyleForNode(
    engine: &mut style::StyleEngine,
    owner: &mut DOM,
    constraints: &ConstraintSpace,
    preferred_color_scheme: style::PreferredColorScheme,
    forced_colors: style::media_queries::forced_colors::ForcedColors,
    node: usize,
) {
    // Element::EnsureComputedStyle returns an existing style before preparing
    // the resolver/selector environment. The normal lifecycle already ran.
    if owner.GetDocument().ResolvedStyleFor(node).is_some() {
        return;
    }
    let mut heap_scope = foundation::LayoutHeapScope::new();
    let user_agent = UserAgentStyleSheets::new();
    let environment = MediaValuesForLayout(constraints, preferred_color_scheme, forced_colors);
    let sheets = user_agent.ForDocument(owner.GetDocument());
    engine
        .EnsureComputedStyle(owner, &environment, sheets, node)
        .unwrap_or_else(|error| panic!("computed style ensure failed: {error}"));
    heap_scope.DeferCollection();
}

/// The browser joins DOM identity, neutral native layout sizes and the
/// document's StyleEngine. Layout itself does not import StyleEngine.
pub(crate) fn PublishContainerSizes(
    engine: &mut style::StyleEngine,
    owner: &mut DOM,
    sizes: &[layoutng_assembly::layout_engine::ContainerQuerySizeSnapshot],
) -> bool {
    let sizes = sizes
        .iter()
        .filter_map(|size| {
            owner
                .GetDocument()
                .FindNodeById(size.node_id)
                .map(|node| (node, size.width, size.height))
        })
        .collect::<Vec<_>>();
    engine
        .SetContainerSizes(owner, sizes)
        .unwrap_or_else(|error| panic!("container size publication failed: {error}"))
}

// cpp: browser/browser.cc:641-641,1748-1758
// Supply source computed-style resolution on the same DOM arena used by the
// script runtime. Page's dirty cache and measurement/render lifecycle are
// separate services; this callback computes current styles on demand.
pub fn CreateStyleBindingsHost(
    document: Rc<RefCell<DOM>>,
    environment: Rc<RefCell<MediaValuesCachedData>>,
) -> DOMBindingsHost {
    let user_agent = UserAgentStyleSheets::new();
    let engine = RefCell::new(style::StyleEngine::new(&document.borrow()));
    DOMBindingsHost {
        update_style: Some(Box::new(move |id| {
            let mut heap_scope = foundation::LayoutHeapScope::new();
            let mut owner = document.borrow_mut();
            let environment = environment.borrow();
            let sheets = user_agent.ForDocument(owner.GetDocument());
            engine
                .borrow_mut()
                .Update(&mut owner, &environment, sheets)
                .unwrap_or_else(|error| panic!("StyleEngine update failed: {error}"));
            if let Some(node) = owner.GetDocument().FindNodeById(id) {
                engine
                    .borrow_mut()
                    .EnsureComputedStyle(&mut owner, &environment, sheets, node)
                    .unwrap_or_else(|error| panic!("computed style ensure failed: {error}"));
            }
            heap_scope.DeferCollection();
        })),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom_mutation::ApplyDOMTreeMutation;
    use javascript::{
        javascript_runtime::JavaScriptRuntime, quickjs_javascript_runtime::QuickJsJavaScriptRuntime,
    };
    use webapi::dom_bindings::DOMJavaScriptBindings;

    #[test]
    fn responsive_picture_uses_css_media_sizes_and_effective_dpr() {
        let owner = html::html_parser::ParseHTML(
            "<picture><source media='(min-width:600px)' sizes='50vw' \
             srcset='bad.png 1.x, wide-1.png 400w, wide-2.png 800w'>\
             <img src='fallback.png' sizes='100vw' \
             srcset='narrow-1.png 400w, narrow-2.png 800w'></picture>",
        );
        let document = owner.GetDocument();
        let image = (0..document.NodeCount())
            .find(|&node| document.Node(node).IsHTMLElement("img"))
            .unwrap();
        let mut constraints = ConstraintSpace::default();
        constraints.available_size.width = 800.0;
        constraints.available_size.height = 600.0;
        constraints.device_pixel_ratio = 1.0;
        constraints.layout_zoom_factor = 2.0;
        assert_eq!(
            SelectResponsiveImage(
                document,
                image,
                &constraints,
                style::PreferredColorScheme::kLight,
                style::media_queries::forced_colors::ForcedColors::kNone,
                &|_| false,
            )
            .unwrap()
            .source,
            "wide-2.png"
        );
    }

    #[test]
    fn preload_picture_selection_matches_dom_and_loaded_candidate_policy() {
        let owner = html::html_parser::ParseHTML(
            "<picture><source type='image/unsupported' srcset='unsupported.png 2x'>\
             <source media='(max-width:600px)' srcset='small.png 2x'>\
             <source type='image/png; codecs=example' sizes='50vw' \
             srcset='one.png 400w, two.png 800w, cached.png 1600w'>\
             <img src='fallback.png' srcset='fallback-2.png 2x'>\
             <source srcset='after-image.png 4x'></picture>",
        );
        let document = owner.GetDocument();
        let image = (0..document.NodeCount())
            .find(|&node| document.Node(node).IsHTMLElement("img"))
            .unwrap();
        let input = document_loader::ResponsiveImageInput {
            src: Some("fallback.png".into()),
            srcset: Some("fallback-2.png 2x".into()),
            picture_sources: vec![
                document_loader::ResponsiveImageSourceInput {
                    image_type: Some("image/unsupported".into()),
                    srcset: Some("unsupported.png 2x".into()),
                    ..Default::default()
                },
                document_loader::ResponsiveImageSourceInput {
                    media: Some("(max-width:600px)".into()),
                    srcset: Some("small.png 2x".into()),
                    ..Default::default()
                },
                document_loader::ResponsiveImageSourceInput {
                    image_type: Some("image/png; codecs=example".into()),
                    sizes: Some("50vw".into()),
                    srcset: Some("one.png 400w, two.png 800w, cached.png 1600w".into()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let mut constraints = ConstraintSpace::default();
        constraints.available_size.width = 800.0;
        constraints.available_size.height = 600.0;
        constraints.device_pixel_ratio = 1.0;
        constraints.layout_zoom_factor = 2.0;
        for (loaded, expected_source, expected_density) in
            [(false, "two.png", 2.0), (true, "cached.png", 4.0)]
        {
            let cache = |source: &str| loaded && source == "cached.png";
            let speculative = SelectResponsiveImageFromAttributes(
                &input,
                &constraints,
                style::PreferredColorScheme::kLight,
                style::media_queries::forced_colors::ForcedColors::kNone,
                &cache,
            )
            .unwrap();
            let authoritative = SelectResponsiveImage(
                document,
                image,
                &constraints,
                style::PreferredColorScheme::kLight,
                style::media_queries::forced_colors::ForcedColors::kNone,
                &cache,
            )
            .unwrap();
            assert_eq!(speculative.source, expected_source);
            assert_eq!(speculative.density, Some(expected_density));
            assert_eq!(authoritative.source, speculative.source);
            assert_eq!(authoritative.density, speculative.density);
        }
    }

    #[test]
    fn javascript_computed_style_reads_real_cascade_and_live_mutations() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML("<html><body><style>.old{font-size:20px;padding-left:3px}.new{font-size:30px;padding-left:7px}input:placeholder-shown{font-weight:600}input:not(:placeholder-shown){font-weight:800}</style><div id=box class=old></div><input id=search placeholder=search></body></html>")));
        let sheet=style::ParseCSS(".old{font-size:20px;padding-left:3px}.new{font-size:30px;padding-left:7px}input:placeholder-shown{font-weight:600}input:not(:placeholder-shown){font-weight:800}");
        document
            .borrow_mut()
            .GetDocumentMut()
            .AppendStyleSheet(sheet);
        let environment = Rc::new(RefCell::new(MediaValuesCachedData {
            viewport_width: 1024.0,
            viewport_height: 768.0,
            small_viewport_width: 1024.0,
            small_viewport_height: 768.0,
            large_viewport_width: 1024.0,
            large_viewport_height: 768.0,
            dynamic_viewport_width: 1024.0,
            dynamic_viewport_height: 768.0,
            device_width: 1024,
            device_height: 768,
            em_size: 16.0,
            media_type: foundation::String::from("screen"),
            scripting: Scripting::kEnabled,
            ..Default::default()
        }));
        let host = CreateStyleBindingsHost(document.clone(), environment);
        let mutated = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::WithHost(
            document,
            Box::new(move |mutation| ApplyDOMTreeMutation(&mut mutated.borrow_mut(), mutation)),
            host,
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings);
        assert!(runtime
            .Evaluate(&realm, webapi::DOMBootstrapSource(), "dom-webidl")
            .Succeeded());
        let result=runtime.Evaluate(&realm,r#"
            function check(v,m){if(!v)throw Error(m)}
            var box=document.getElementById('box'), input=document.getElementById('search');
            var style=getComputedStyle(box);
            check(style.fontSize==='20px','resolved font');check(style.paddingLeft==='3px','resolved padding');
            box.className='new';check(style.fontSize==='30px','live class');check(style.paddingLeft==='7px','live padding');
            check(getComputedStyle(input).fontWeight==='600','placeholder selector');input.value='typed';check(getComputedStyle(input).fontWeight==='800','current control state');
            box.style.fontSize='40px';check(style.fontSize==='40px','inline style mutation');
        "#,"live-styles");
        assert!(result.Succeeded(), "{:?}", result.exception);
    }
}

#[cfg(test)]
mod measurement_tests {
    use super::*;
    use crate::dom_mutation::ApplyDOMTreeMutation;
    use javascript::{
        javascript_runtime::JavaScriptRuntime, quickjs_javascript_runtime::QuickJsJavaScriptRuntime,
    };
    use webapi::dom_bindings::DOMJavaScriptBindings;
    #[test]
    fn live_javascript_metrics_and_client_rects_match_cpp_after_mutations() {
        crate::native_test_thread::run(live_measurement_body);
    }
    fn live_measurement_body() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(include_str!(
            "../../../artifacts/cpp-reference/persistent-measurement.html"
        ))));
        document
            .borrow_mut()
            .GetDocumentMut()
            .AppendStyleSheet(style::ParseCSS(include_str!(
                "../../../artifacts/cpp-reference/persistent-measurement.css"
            )));
        assert_eq!(
            include_str!("../../../artifacts/cpp-reference/persistent-measurement-results.tsv")
                .lines()
                .count(),
            36,
            "frozen C++ evidence must contain all 12 nodes in all 3 stages"
        );
        let mut space = ConstraintSpace::default();
        space
            .fonts
            .push(layoutng_assembly::internal::layout_input::FontFace {
                family: "sans-serif".into(),
                bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
                    .as_slice()
                    .into(),
                ..Default::default()
            });
        space.viewport = Some(
            layoutng_assembly::internal::layout_input::ViewportGeometry {
                size: layoutng_assembly::internal::layout_input_types::IntSize {
                    width: 1024,
                    height: 768,
                },
                overscroll_type:
                    layoutng_assembly::internal::layout_input::OverscrollType::kTransform,
                ..Default::default()
            },
        );
        space.scrollbar_theme = Some(
            layoutng_assembly::internal::layout_input_types::ScrollbarThemeMetrics {
                auto_thickness: 15,
                thin_thickness: 7,
                minimum_thumb_length: 24,
                has_buttons: true,
                uses_overlay_scrollbars: true,
                ..Default::default()
            },
        );
        space.available_size = layoutng_assembly::internal::layout_input::Size {
            width: 1024.0,
            height: 768.0,
        };
        let constraints = Rc::new(RefCell::new(space));
        let host = CreateLayoutBindingsHost(
            document.clone(),
            constraints,
            Rc::new(RefCell::new(UserInteractionState::default())),
        );
        let mutated = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::WithHost(
            document,
            Box::new(move |mutation| ApplyDOMTreeMutation(&mut mutated.borrow_mut(), mutation)),
            host,
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings);
        assert!(runtime
            .Evaluate(&realm, webapi::DOMBootstrapSource(), "dom-webidl")
            .Succeeded());
        for stage in 0..3 {
            let mutate=match stage {1=>"document.getElementById('box').className='wide'",2=>"document.getElementById('box').style.cssText='width:55.5px;height:40.75px';document.getElementById('box').appendChild(document.getElementById('move'))",_=>""};
            let result = runtime.Evaluate(&realm, mutate, "measurement-mutation");
            assert!(result.Succeeded(), "{:?}", result.exception);
            for line in
                include_str!("../../../artifacts/cpp-reference/persistent-measurement-results.tsv")
                    .lines()
            {
                let (key, rest) = line.split_once('\t').unwrap();
                let (s, id) = key.split_once(':').unwrap();
                if s.parse::<usize>().unwrap() != stage {
                    continue;
                }
                let expected = rest
                    .split('\t')
                    .map(|v| v.parse::<f64>().unwrap())
                    .collect::<Vec<_>>();
                let values = expected
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                let script = format!(
                    r#"(function() {{
                    const node=document.getElementById('{id}'), style=getComputedStyle(node);
                    const actual=[node.offsetWidth,node.offsetHeight,node.clientWidth,node.clientHeight,parseFloat(style.width),parseFloat(style.height),node.clientLeft,node.clientTop,node.scrollWidth,node.scrollHeight];
                    const rects=node.getClientRects(); actual.push(rects.length);for(const r of rects)actual.push(r.x,r.y,r.width,r.height);
                    const expected=[{values}];if(actual.length!==expected.length)throw Error('rect count '+actual+' != '+expected);
                    for(let i=0;i<actual.length;i++)if(actual[i]!==expected[i])throw Error('stage {stage} id {id} field '+i+' '+actual[i]+' != '+expected[i]);
                    const bounds=node.getBoundingClientRect();if(rects.length===1&&(bounds.x!==rects[0].x||bounds.y!==rects[0].y||bounds.width!==rects[0].width||bounds.height!==rects[0].height))throw Error('bounding rect');
                }})()"#
                );
                let result = runtime.Evaluate(&realm, &script, "cpp-native-measurement");
                assert!(result.Succeeded(), "{:?}", result.exception);
            }
        }
    }
}
