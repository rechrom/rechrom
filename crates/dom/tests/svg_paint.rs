#![allow(non_snake_case)]
use layoutng_assembly::internal::layout_input::{ComputedStyle, Offset};
use layoutng_assembly::internal::paint_input::{PaintShaderKind, PaintSpreadMethod};
extern crate layoutng_block;
extern crate layoutng_replaced;
use std::collections::HashMap;
fn Ids(document: &dom::Document, _root: usize) -> HashMap<String, usize> {
    (0..document.NodeCount())
        .filter_map(|i| {
            document
                .Node(i)
                .FindAttribute("id")
                .map(|a| (a.value.clone(), i))
        })
        .collect()
}
use dom::style_resolver::{ResolveComputedStyles, ResolveCssom, StyleEnvironment};

const MARKUP: &str = r##"<html><body><svg viewBox="0 0 200 100"><defs>
      <linearGradient id="MixedCase" x2="50%"><stop id="first" offset="70%" stop-color="red" stop-opacity=".5"/>
      <stop offset="20%" style="stop-color:currentColor;color:blue;stop-opacity:50%"/></linearGradient>
      <radialGradient id="radial" href="#MixedCase" gradientUnits="userSpaceOnUse" spreadMethod="reflect" gradientTransform="translate(2 3)"/>
      <linearGradient id="cycle" href="#cycle"/>
      </defs><path id="shape" fill="url('#MixedCase')" stroke="url(#radial)"/>
      <path id="fallback" fill="url(#missing) green"/>
      <path id="loop" fill="url(#cycle) currentColor" color="blue"/></svg></body></html>"##;

fn Check(style: &ComputedStyle) {
    let fill = style.paint.svg_fill_server.as_ref().unwrap();
    assert_eq!(fill.kind, PaintShaderKind::kLinearGradient);
    assert_eq!(fill.end.x, 0.5);
    assert_eq!(fill.stops.len(), 2);
    assert_eq!(fill.stops[0].offset, 0.7);
    assert_eq!(fill.stops[1].offset, 0.7);
    assert!((fill.stops[0].color.alpha - 0.25).abs() < 1.0 / 255.0);
    assert_eq!(fill.stops[1].color.alpha, 0.5);
    assert_eq!(fill.stops[1].color.blue, 1.0);
    let stroke = style.paint.svg_stroke_server.as_ref().unwrap();
    assert_eq!(stroke.kind, PaintShaderKind::kRadialGradient);
    assert_eq!(stroke.spread, PaintSpreadMethod::kReflect);
    assert!(!stroke.unit_coordinates);
    assert_eq!(stroke.center, Offset { x: 100.0, y: 50.0 });
    assert_eq!(stroke.transform.values[12], 2.0);
}
fn Sheet() -> cssom::CSSStyleSheet {
    cssom::ParseCSS("#first { stop-color:rgba(255,0,0,.5) } defs { stop-opacity:.1;stop-color:yellow } #first { stop-opacity:garbage }")
}
#[test]
fn static_gradients_use_cascaded_stops_and_template_attributes() {
    let document = html::Parse(MARKUP);
    let styles = ResolveCssom(&document, &[Sheet()], 800.0, 600.0);
    let shape = document
        .elements
        .iter()
        .position(|e| e.id.as_deref() == Some("shape"))
        .unwrap();
    Check(&styles.styles[shape]);
    let fallback = document
        .elements
        .iter()
        .position(|e| e.id.as_deref() == Some("fallback"))
        .unwrap();
    assert!(styles.styles[fallback].paint.svg_fill_server.is_none());
    assert!(styles.styles[fallback].paint.svg_fill.unwrap().green > 0.0);
    let cycle = document
        .elements
        .iter()
        .position(|e| e.id.as_deref() == Some("loop"))
        .unwrap();
    assert!(styles.styles[cycle].paint.svg_fill_server.is_none());
    assert_eq!(styles.styles[cycle].paint.svg_fill.unwrap().blue, 1.0);
}
#[test]
fn persistent_gradients_refresh_when_resource_stops_change() {
    let mut owner = html::html_parser::ParseHTML(MARKUP);
    owner.GetDocumentMut().AppendStyleSheet(Sheet());
    let environment = StyleEnvironment::default();
    ResolveComputedStyles(&mut owner, &environment, &[]);
    let document = owner.GetDocumentMut();
    let ids = Ids(document, document.Root());
    Check(&document.ResolvedStyleFor(ids["shape"]).unwrap().style);
    document.SetAttribute(
        ids["first"],
        dom::persistent_document::DOMAttribute {
            local_name: "style".into(),
            value: "stop-color:green;stop-opacity:25%".into(),
            ..Default::default()
        },
    );
    dom::style_resolver::ResolveDocumentStyles(document, &environment, &[]);
    let fill = document
        .ResolvedStyleFor(ids["shape"])
        .unwrap()
        .style
        .paint
        .svg_fill_server
        .as_ref()
        .unwrap();
    assert!((fill.stops[0].color.alpha - 0.25).abs() < 1.0 / 255.0);
    assert!(fill.stops[0].color.green > 0.0);
    assert!(document.StyleState().impact.paint);
}
#[test]
fn stop_properties_are_non_inherited_and_support_css_wide_keywords() {
    let document = html::Parse("<html><svg><defs style='stop-color:red;stop-opacity:.2'><linearGradient style='stop-color:red;stop-opacity:.2'><stop id='default'/><stop id='inherited' style='stop-color:inherit;stop-opacity:inherit'/><stop id='unset' style='stop-color:unset;stop-opacity:unset'/></linearGradient></defs></svg></html>");
    let styles = ResolveCssom(
        &document,
        &[cssom::ParseCSS(
            "linearGradient {stop-color:red;stop-opacity:.2}",
        )],
        800.0,
        600.0,
    );
    for (i, node) in document
        .elements
        .iter()
        .enumerate()
        .filter(|(_, n)| n.tag == "stop")
    {
        let paint = &styles.styles[i].paint;
        if node.id.as_deref() == Some("inherited") {
            assert_eq!(paint.svg_stop_opacity, 0.2);
            assert_eq!(paint.svg_stop_color.red, 1.0);
        } else {
            assert_eq!(paint.svg_stop_opacity, 1.0);
            assert_eq!(paint.svg_stop_color.red, 0.0);
        }
    }
}
#[test]
fn source_inventory_properties_never_panic_on_web_declarations() {
    let inventory = include_str!("../src/style_resolver/source_supported_properties.rs");
    let names = inventory
        .split('"')
        .filter(|p| !p.contains(char::is_whitespace) && !p.is_empty());
    for property in names {
        for value in ["unimplemented-web-value", "initial", "none", "0"] {
            let markup = format!("<html><body><div style='{property}:{value};width:30px;color:red'></div></body></html>");
            let document = html::Parse(&markup);
            let styles = ResolveCssom(&document, &[], 800.0, 600.0);
            let index = document
                .elements
                .iter()
                .position(|n| n.tag == "div")
                .unwrap();
            assert_eq!(styles.styles[index].width, Some(30.0), "{property}:{value}");
        }
    }
    let document = html::Parse("<html><body><div style='width:20px; fill-opacity:.5; clip-rule:evenodd; color:red; paint-order:stroke fill; width:30px'></div></body></html>");
    let styles = ResolveCssom(&document, &[], 800.0, 600.0);
    let i = document
        .elements
        .iter()
        .position(|e| e.tag == "div")
        .unwrap();
    assert_eq!(styles.styles[i].width, Some(30.0));
    assert_eq!(styles.styles[i].paint.color.red, 1.0);
}
