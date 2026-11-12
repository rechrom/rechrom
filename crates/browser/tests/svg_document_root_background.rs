//! SVG document images keep their root background on the SVG element.

#[path = "../src/test_thread.rs"]
mod test_thread;

use std::sync::Arc;

use image_resource::{ContainerKey, DocumentImageDecoder};

const ROOT_BACKGROUND_SVG: &[u8] = br#"
<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32"
     style="background:rgb(255, 0, 0)">
  <rect x="16" y="8" width="16" height="16" fill="blue"/>
</svg>
"#;

#[test]
fn svg_document_root_background_is_not_transferred_to_the_html_canvas() {
    test_thread::run(|| {
        let assembly = browser::CreateLayoutAssembly();
        let mut decoder = browser::CreateSVGImageDecoder(&assembly);
        let container = ContainerKey::new(64, 32, 1.0, 1.0);
        let created = decoder
            .create(
                29,
                Arc::from(ROOT_BACKGROUND_SVG),
                "image/svg+xml",
                &container,
            )
            .unwrap();
        let record = paint::paint_engine::downcast_document_paint_record::<
            paint::paint_engine::DocumentPaintArtifactRecord,
        >(&created.initial_frame.record)
        .unwrap();

        assert!(record.artifact.display_items.iter().all(|item| item
            .id
            .r#type
            .TypeAsDebugString()
            != "DocumentBackground"));
        assert!(record.artifact.display_items.iter().any(|item| item
            .id
            .r#type
            .TypeAsDebugString()
            == "BoxDecorationBackground"));
    });
}
