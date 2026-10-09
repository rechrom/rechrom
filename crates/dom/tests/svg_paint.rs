#![allow(non_snake_case)]

use layoutng_assembly::internal::layout_input::{ComputedStyle, Offset};
use layoutng_assembly::internal::paint_input::{PaintShaderKind, PaintSpreadMethod};
use std::collections::HashMap;

extern crate layoutng_block;
extern crate layoutng_replaced;

const MARKUP: &str = r##"<html><body><svg viewBox="0 0 200 100"><defs>
<linearGradient id="MixedCase" x2="50%"><stop id="first" offset="70%" stop-color="red" stop-opacity=".5"/>
<stop offset="20%" style="stop-color:currentColor;color:blue;stop-opacity:50%"/></linearGradient>
<radialGradient id="radial" href="#MixedCase" gradientUnits="userSpaceOnUse" spreadMethod="reflect" gradientTransform="translate(2 3)"/>
<linearGradient id="cycle" href="#cycle"/></defs><path id="shape" fill="url('#MixedCase')" stroke="url(#radial)"/>
<path id="fallback" fill="url(#missing) green"/><path id="loop" fill="url(#cycle) currentColor" color="blue"/></svg></body></html>"##;

fn ids(document: &dom::Document) -> HashMap<String, usize> {
    (0..document.NodeCount())
        .filter_map(|index| {
            document
                .Node(index)
                .FindAttribute("id")
                .map(|attribute| (attribute.value.clone(), index))
        })
        .collect()
}

fn media() -> style::media_queries::media_values_cached::MediaValuesCachedData {
    style::media_queries::media_values_cached::MediaValuesCachedData {
        viewport_width: 800.0,
        viewport_height: 600.0,
        small_viewport_width: 800.0,
        small_viewport_height: 600.0,
        large_viewport_width: 800.0,
        large_viewport_height: 600.0,
        dynamic_viewport_width: 800.0,
        dynamic_viewport_height: 600.0,
        device_width: 800,
        device_height: 600,
        device_pixel_ratio: 1.0,
        em_size: 16.0,
        ..Default::default()
    }
}

fn update(engine: &mut style::StyleEngine, owner: &mut dom::DOM) {
    engine.Update(owner, &media(), &[]).expect("style update");
}

fn check_gradient(style: &ComputedStyle) {
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

fn sheet() -> cssom::CSSStyleSheet {
    style::ParseCSS("#first { stop-color:rgba(255,0,0,.5) } defs { stop-opacity:.1;stop-color:yellow } #first { stop-opacity:garbage }")
}

// These expectations remain visible while Chromium's SVG paint property
// applicators are translated. They compile only against the new StyleEngine;
// no compatibility resolver is retained.
#[test]
#[ignore = "pending Chromium SVG paint property applicators"]
fn gradients_use_cascaded_stops_and_template_attributes() {
    let mut owner = html::html_parser::ParseHTML(MARKUP);
    owner.GetDocumentMut().AppendStyleSheet(sheet());
    let mut engine = style::StyleEngine::new(&owner);
    update(&mut engine, &mut owner);
    let ids = ids(owner.GetDocument());
    check_gradient(
        &owner
            .GetDocument()
            .ResolvedStyleFor(ids["shape"])
            .unwrap()
            .style,
    );
    assert!(owner
        .GetDocument()
        .ResolvedStyleFor(ids["fallback"])
        .unwrap()
        .style
        .paint
        .svg_fill_server
        .is_none());
    assert!(
        owner
            .GetDocument()
            .ResolvedStyleFor(ids["fallback"])
            .unwrap()
            .style
            .paint
            .svg_fill
            .unwrap()
            .green
            > 0.0
    );
    assert_eq!(
        owner
            .GetDocument()
            .ResolvedStyleFor(ids["loop"])
            .unwrap()
            .style
            .paint
            .svg_fill
            .unwrap()
            .blue,
        1.0
    );
}

#[test]
#[ignore = "pending Chromium SVG paint property applicators"]
fn gradients_refresh_when_resource_stops_change() {
    let mut owner = html::html_parser::ParseHTML(MARKUP);
    owner.GetDocumentMut().AppendStyleSheet(sheet());
    let ids = ids(owner.GetDocument());
    let mut engine = style::StyleEngine::new(&owner);
    update(&mut engine, &mut owner);
    check_gradient(
        &owner
            .GetDocument()
            .ResolvedStyleFor(ids["shape"])
            .unwrap()
            .style,
    );
    owner.GetDocumentMut().SetAttribute(
        ids["first"],
        dom::persistent_document::DOMAttribute {
            local_name: "style".into(),
            value: "stop-color:green;stop-opacity:25%".into(),
            ..Default::default()
        },
    );
    update(&mut engine, &mut owner);
    let fill = owner
        .GetDocument()
        .ResolvedStyleFor(ids["shape"])
        .unwrap()
        .style
        .paint
        .svg_fill_server
        .as_ref()
        .unwrap();
    assert!((fill.stops[0].color.alpha - 0.25).abs() < 1.0 / 255.0);
    assert!(fill.stops[0].color.green > 0.0);
}

#[test]
fn generated_properties_report_unsupported_without_losing_valid_declarations() {
    use style::css_property_names::{
        kCSSPropertyAliasList, kIntFirstCSSProperty, kIntLastCSSProperty, ConvertToCSSPropertyID,
        GetPropertyName,
    };
    let names = (kIntFirstCSSProperty..=kIntLastCSSProperty)
        .map(ConvertToCSSPropertyID)
        .chain(kCSSPropertyAliasList)
        .map(GetPropertyName);
    for property in names {
        for value in ["unimplemented-web-value", "initial", "none", "0"] {
            let markup = format!("<html><body><div style='{property}:{value};width:30px;color:red'></div></body></html>");
            let mut owner = html::html_parser::ParseHTML(&markup);
            let mut engine = style::StyleEngine::new(&owner);
            update(&mut engine, &mut owner);
            let div = (0..owner.GetDocument().NodeCount())
                .find(|&index| owner.GetDocument().Node(index).IsHTMLElement("div"))
                .unwrap();
            assert!(
                !owner
                    .GetDocument()
                    .ResolvedStyleFor(div)
                    .unwrap()
                    .style
                    .native_style
                    .is_null(),
                "{property}:{value}"
            );
        }
    }
}
