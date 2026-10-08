#![allow(non_snake_case)]
use crate::user_agent_styles::UserAgentStyleSheets;
use dom::style_resolver::StyleEnvironment;
use dom::UserInteractionState;
use dom::DOM;
use layoutng_assembly::{
    fragment_tree::{FragmentKind, FragmentNode},
    internal::layout_input::ConstraintSpace,
};
use std::{cell::RefCell, rc::Rc};
use webapi::dom_bindings::DOMBindingsHost;

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
    ResolveLayoutStylesWithColorScheme(owner, constraints, Default::default());
}

pub(crate) fn ResolveLayoutStylesWithColorScheme(
    owner: &mut DOM,
    constraints: &ConstraintSpace,
    preferred_color_scheme: dom::style_resolver::PreferredColorScheme,
) {
    let user_agent = UserAgentStyleSheets::new();
    let environment = StyleEnvironment {
        viewport_width: Some(constraints.available_size.width),
        viewport_height: Some(constraints.available_size.height),
        resolution_dppx: Some(1.0),
        preferred_color_scheme,
        ..Default::default()
    };
    owner.ResolveStyles(&environment, user_agent.ForDocument(owner.GetDocument()));
}

// cpp: browser/browser.cc:641-641,1748-1758
// Supply source computed-style resolution on the same DOM arena used by the
// script runtime. Page's dirty cache and measurement/render lifecycle are
// separate services; this callback computes current styles on demand.
pub fn CreateStyleBindingsHost(
    document: Rc<RefCell<DOM>>,
    environment: Rc<RefCell<StyleEnvironment>>,
) -> DOMBindingsHost {
    let user_agent = UserAgentStyleSheets::new();
    DOMBindingsHost {
        update_style: Some(Box::new(move || {
            let mut owner = document.borrow_mut();
            let environment = *environment.borrow();
            let sheets = user_agent.ForDocument(owner.GetDocument());
            dom::style_resolver::ResolveComputedStyles(&mut owner, &environment, sheets);
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
    fn javascript_computed_style_reads_real_cascade_and_live_mutations() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML("<html><body><style>.old{font-size:20px;padding-left:3px}.new{font-size:30px;padding-left:7px}input:placeholder-shown{font-weight:600}input:not(:placeholder-shown){font-weight:800}</style><div id=box class=old></div><input id=search placeholder=search></body></html>")));
        let sheet=cssom::ParseCSS(".old{font-size:20px;padding-left:3px}.new{font-size:30px;padding-left:7px}input:placeholder-shown{font-weight:600}input:not(:placeholder-shown){font-weight:800}");
        document
            .borrow_mut()
            .GetDocumentMut()
            .AppendStyleSheet(sheet);
        let environment = Rc::new(RefCell::new(StyleEnvironment {
            viewport_width: Some(1024.0),
            viewport_height: Some(768.0),
            resolution_dppx: Some(1.0),
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
            .AppendStyleSheet(cssom::ParseCSS(include_str!(
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
                    "../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
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
