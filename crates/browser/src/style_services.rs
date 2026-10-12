#![allow(non_snake_case)]
use crate::user_agent_styles::UserAgentStyleSheets;
use dom::{Document, UserInteractionState, DOM};
use layoutng_assembly::{
    fragment_tree::{FragmentKind, FragmentNode},
    internal::layout_input::ConstraintSpace,
};
use std::{cell::RefCell, rc::Rc};
use style::media_queries::{media_values_cached::MediaValuesCachedData, scripting::Scripting};
use webapi::dom_bindings::DOMBindingsHost;

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
        media_type: foundation::String::from("screen"),
        preferred_color_scheme,
        forced_colors,
        scripting: Scripting::kEnabled,
        ..Default::default()
    }
}

fn ResponsiveMediaMatches(query: &str, values: &style::media_queries::MediaValuesCached) -> bool {
    if query.trim().is_empty() {
        return true;
    }
    let set = style::media_queries::media_query_backend::ParseMediaQuerySet(query);
    style::media_queries::media_query_evaluator::MediaQueryEvaluator::<
        style::media_queries::media_query_backend::MediaQueryValueBackend<'_>,
    >::ForMediaValues(values)
    .Eval(&set)
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
    use style::media_queries::MediaValues;

    let media = MediaValuesForLayout(constraints, preferred_color_scheme, forced_colors);
    let values = style::media_queries::MediaValuesCached::new(&media);
    let image = document.Node(index);
    let fallback = image
        .FindAttribute("src")
        .map(|attribute| attribute.value.as_str());
    let image_sizes = image
        .FindAttribute("sizes")
        .map(|attribute| attribute.value.as_str());
    let allows_auto_sizes = document.AllowsResponsiveImageAutoSizes(index);
    // SizesAttributeParser::EffectiveSize uses the last rendered content-box
    // width after absolute-zoom adjustment. The first pre-layout selection has
    // no stored width and therefore evaluates the explicit fallback/100vw.
    let auto_size = allows_auto_sizes
        .then(|| document.ResponsiveImageAutoSize(index))
        .flatten();
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
            if !ResponsiveMediaMatches(
                source
                    .FindAttribute("media")
                    .map_or("", |attribute| &attribute.value),
                &values,
            ) || !SupportedResponsiveImageType(
                source
                    .FindAttribute("type")
                    .map(|attribute| attribute.value.as_str()),
            ) {
                continue;
            }
            let Some(srcset) = source
                .FindAttribute("srcset")
                .filter(|attribute| !attribute.value.is_empty())
            else {
                continue;
            };
            let sizes = source
                .FindAttribute("sizes")
                .map(|attribute| attribute.value.as_str());
            let sizes = if sizes.is_none() && allows_auto_sizes {
                Some("auto")
            } else {
                sizes
            };
            if let Some(candidate) =
                html::parser::html_srcset_parser::BestFitSourceForImageAttributesWithCache(
                    values.DevicePixelRatio() as f64,
                    source_size(sizes),
                    None,
                    Some(&srcset.value),
                    is_loaded,
                )
            {
                return Some(dom::persistent_document::ResponsiveImageSelection {
                    source: candidate.source,
                    density: Some(candidate.density),
                });
            }
        }
    }
    let Some(srcset) = image.FindAttribute("srcset") else {
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
        Some(&srcset.value),
        is_loaded,
    )?;
    Some(dom::persistent_document::ResponsiveImageSelection {
        source: candidate.source,
        density: Some(candidate.density),
    })
}

// cpp: browser/browser.cc:649-671
pub(crate) fn FragmentMetric(root: &FragmentNode, id: u64, name: &str) -> f64 {
    fn find(node: &FragmentNode, id: u64) -> Option<&FragmentNode> {
        if node.node_id == id && node.kind == FragmentKind::kBox {
            return Some(node);
        }
        node.children.iter().find_map(|child| find(child, id))
    }
    let Some(fragment) = find(root, id) else {
        return 0.0;
    };
    let p = &fragment.paint;
    // CSSOM View integer dimensions exclude CSS zoom; client rectangles
    // retain the zoomed physical geometry exported by paint.
    // Blink's zoom adjustment uses float division before integer rounding.
    // Keeping the quotient in f64 would round 90 / float(0.8) to 112 instead
    // of the native float quotient 112.5, whose integer result is 113.
    let unzoom = |value: f64| {
        if p.effective_zoom == 1.0 {
            value
        } else {
            (value as f32 / p.effective_zoom) as f64
        }
    };
    match name {
        "offsetWidth" => unzoom(fragment.size.width).round(),
        "offsetHeight" => unzoom(fragment.size.height).round(),
        // Fragment offsets are relative to their containing fragment. This is
        // the same coordinate space CSSOM View exposes for offsetLeft/Top in
        // the common unfragmented offset-parent case.
        "offsetLeft" => unzoom(fragment.offset.x).round(),
        "offsetTop" => unzoom(fragment.offset.y).round(),
        "clientWidth" => {
            unzoom(fragment.content_size.width + p.padding.left + p.padding.right).round()
        }
        "clientHeight" => {
            unzoom(fragment.content_size.height + p.padding.top + p.padding.bottom).round()
        }
        "contentWidth" => unzoom(fragment.content_size.width),
        "contentHeight" => unzoom(fragment.content_size.height),
        "clientLeft" => unzoom(p.border.left).round(),
        "clientTop" => unzoom(p.border.top).round(),
        "scrollWidth" => unzoom(p.scroll_size.width).round(),
        "scrollHeight" => unzoom(p.scroll_size.height).round(),
        // Internal CSSOM View adapter metrics. ScrollableArea's range is not
        // necessarily 0..(scroll-size - client-size): RTL and vertical writing
        // modes can put the logical start at a negative physical offset.
        "minimumScrollLeft" => p
            .scroll_container
            .map_or(0.0, |scroll| unzoom(scroll.minimum_scroll_offset.x)),
        "maximumScrollLeft" => p.scroll_container.map_or_else(
            || unzoom((p.scroll_size.width - fragment.size.width).max(0.0)),
            |scroll| unzoom(scroll.maximum_scroll_offset.x),
        ),
        "minimumScrollTop" => p
            .scroll_container
            .map_or(0.0, |scroll| unzoom(scroll.minimum_scroll_offset.y)),
        "maximumScrollTop" => p.scroll_container.map_or_else(
            || unzoom((p.scroll_size.height - fragment.size.height).max(0.0)),
            |scroll| unzoom(scroll.maximum_scroll_offset.y),
        ),
        _ => 0.0,
    }
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
    let engine = RefCell::new(crate::LayoutEngine::new(&crate::CreateLayoutAssembly()));
    let measure = Rc::new(move || {
        let constraints = constraints.borrow();
        let mut document = document.borrow_mut();
        ResolveLayoutStyles(&mut document, &constraints);
        let mut engine = engine.borrow_mut();
        crate::persistent_layout::LayoutDocumentWithEngine(
            &mut engine,
            &mut document,
            &interaction.borrow(),
            &constraints,
        );
        engine
            .GetLayoutResult()
            .expect("successful layout has fragments")
            .clone()
    });
    let geometry = measure.clone();
    DOMBindingsHost {
        update_style: Some(Box::new(move || {
            ResolveLayoutStyles(
                &mut resolve_document.borrow_mut(),
                &resolve_constraints.borrow(),
            )
        })),
        read_metric: Some(Box::new(move |id, name| {
            FragmentMetric(&measure(), id, name)
        })),
        read_geometry: Some(Box::new(move |id| {
            paint::paint_engine::FragmentClientRects(&geometry(), id)
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
    // Blink style resolution allocates ComputedStyle and its rare-data on the
    // document's cppgc heap before Layout begins. Keep that owner-thread heap
    // boundary in the browser assembly layer; StyleEngine itself remains
    // independent of the host event loop and thread placement.
    let mut heap_scope = foundation::LayoutHeapScope::new();
    let user_agent = UserAgentStyleSheets::new();
    let environment = MediaValuesForLayout(constraints, preferred_color_scheme, forced_colors);
    let sheets = user_agent.ForDocument(owner.GetDocument());
    engine
        .Update(owner, &environment, sheets)
        .unwrap_or_else(|error| panic!("StyleEngine update failed: {error}"));
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
        update_style: Some(Box::new(move || {
            let mut heap_scope = foundation::LayoutHeapScope::new();
            let mut owner = document.borrow_mut();
            let environment = environment.borrow();
            let sheets = user_agent.ForDocument(owner.GetDocument());
            engine
                .borrow_mut()
                .Update(&mut owner, &environment, sheets)
                .unwrap_or_else(|error| panic!("StyleEngine update failed: {error}"));
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
                bytes: include_bytes!(
                    "../../../assets/fonts/Roboto-Regular.ttf"
                )
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
