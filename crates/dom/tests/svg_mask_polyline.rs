use dom::persistent_document::{DOMAttribute, DOMNamespace};
use layoutng_assembly::internal::layout_input::{Offset, PaintPathVerb, SvgShapeGeometry};
use layoutng_style::style::computed_style::ComputedStyle;

#[test]
fn mask_polyline_uses_the_open_layout_shape_path() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = dom::DOM::new();
    let document = owner.GetDocumentMut();
    let polyline = document.CreateElement(
        DOMNamespace::kSVG,
        "polyline".into(),
        vec![DOMAttribute {
            local_name: "points".into(),
            value: "4,6 12,9 7,15".into(),
            ..Default::default()
        }],
    );
    let root = document.Root();
    document.AppendChild(root, polyline);
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let shape = dom::layout_mapping::SVGMaskShape(document, polyline, initial)
        .expect("supported mask geometry")
        .expect("polyline layout shape");

    assert_eq!(shape.geometry, SvgShapeGeometry::kPath);
    assert_eq!(shape.bounds_offset, Offset { x: 4.0, y: 6.0 });
    assert_eq!(shape.bounds_width, 8.0);
    assert_eq!(shape.bounds_height, 9.0);
    assert_eq!(
        shape
            .path
            .iter()
            .map(|command| command.verb)
            .collect::<Vec<_>>(),
        vec![
            PaintPathVerb::kMoveTo,
            PaintPathVerb::kLineTo,
            PaintPathVerb::kLineTo,
        ]
    );
}
