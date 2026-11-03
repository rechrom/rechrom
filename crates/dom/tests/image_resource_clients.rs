use dom::{
    persistent_document::{DOMAttribute, DOMNamespace},
    ImageResourceMetadata, WebFeature, DOM,
};

// Chromium: core/layout/layout_image.cc:132-137 and layout_image.h:57-70.
// LayoutImageResource is a stable client wrapper whose selected content may
// change. Rechrom owns that relation at the DOM/image-loader boundary: a source
// mutation moves the existing image node between exact resource client sets,
// and stale completion must not refresh the retained LayoutImage.
#[test]
fn replaced_image_client_tracks_the_current_resource_binding() {
    let mut owner = DOM::new();
    let document = owner.GetDocumentMut();
    let image = document.CreateElement(
        DOMNamespace::kHTML,
        "img".into(),
        vec![DOMAttribute {
            local_name: "src".into(),
            value: "old.png".into(),
            ..Default::default()
        }],
    );
    document.AppendChild(document.Root(), image);
    let image_id = document.Node(image).Id();

    document.SetAttribute(
        image,
        DOMAttribute {
            local_name: "src".into(),
            value: "current.png".into(),
            ..Default::default()
        },
    );
    document.StyleStateMut().geometry_input_nodes.clear();

    document.SetImageResourceContentDPR(1, Some(2.0));
    document.SetImageResource(
        "old.png".into(),
        ImageResourceMetadata {
            id: 1,
            natural_width: 10.0,
            natural_height: 10.0,
            ..Default::default()
        },
    );
    assert!(!document
        .StyleState()
        .geometry_input_nodes
        .contains(&image_id));
    assert!(!document.IsUseCounted(WebFeature::kClientHintsContentDPR));

    document.SetImageResourceContentDPR(2, Some(4.0));
    document.SetImageResource(
        "current.png".into(),
        ImageResourceMetadata {
            id: 2,
            natural_width: 20.0,
            natural_height: 10.0,
            ..Default::default()
        },
    );
    assert!(document
        .StyleState()
        .geometry_input_nodes
        .contains(&image_id));
    assert!(document.IsUseCounted(WebFeature::kClientHintsContentDPR));
}

#[test]
fn repeated_image_resource_uses_exact_live_layout_image_clients() {
    let mut owner = DOM::new();
    let document = owner.GetDocumentMut();
    let root = document.Root();
    let make_image = |document: &mut dom::Document| {
        document.CreateElement(
            DOMNamespace::kHTML,
            "img".into(),
            vec![DOMAttribute {
                local_name: "src".into(),
                value: "animated.gif".into(),
                ..Default::default()
            }],
        )
    };
    let first = make_image(document);
    let second = make_image(document);
    document.AppendChild(root, first);
    document.AppendChild(root, second);
    document.SetImageResource(
        "animated.gif".into(),
        ImageResourceMetadata {
            id: 7,
            natural_width: 1.0,
            natural_height: 1.0,
            ..Default::default()
        },
    );
    assert!(document.ImageResourceUsedByMultipleReplacedImages(7));

    document.Remove(second);
    assert!(!document.ImageResourceUsedByMultipleReplacedImages(7));
}
