use std::{sync::Arc, time::Duration};

use document_image::SVGImageDecoder;
use image_resource::{
    ContainerKey, DocumentImageDecoder, DocumentImageEffect, DocumentImageMutation,
};

const ANIMATED_SVG: &[u8] = br#"
<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32">
  <style>
    @keyframes move { from { transform: translateX(0px); } to { transform: translateX(20px); } }
    rect { animation: move 1s linear infinite; fill: rgb(255, 0, 0); }
  </style>
  <rect width="16" height="16"/>
</svg>
"#;

#[test]
fn document_image_timeline_publishes_immutable_changed_records_only() {
    crate::native_test_thread::run(|| {
        let assembly = crate::CreateLayoutAssembly();
        let mut decoder = SVGImageDecoder::new(&assembly);
        let container = ContainerKey::new(64, 32, 1.0, 1.0);
        let mut created = decoder
            .create(17, Arc::from(ANIMATED_SVG), "image/svg+xml", &container)
            .unwrap();
        assert_eq!(created.initial_frame.resource_id, 17);
        assert_eq!(created.initial_frame.revision, 1);
        assert!(created.image.has_active_animation());

        let unchanged = created
            .image
            .apply_mutation(DocumentImageMutation::AdvanceTimeline {
                frame_time: Duration::ZERO,
                begin_frame_sequence: 1,
            })
            .unwrap();
        assert!(!unchanged
            .iter()
            .any(|effect| matches!(effect, DocumentImageEffect::FrameChanged { .. })));

        let changed = created
            .image
            .apply_mutation(DocumentImageMutation::AdvanceTimeline {
                frame_time: Duration::from_millis(500),
                begin_frame_sequence: 2,
            })
            .unwrap();
        let next = changed
            .iter()
            .find_map(|effect| match effect {
                DocumentImageEffect::FrameChanged { frame, .. } => Some(frame.clone()),
                _ => None,
            })
            .expect("observable animation change publishes one frame");
        assert_eq!(next.revision, 2);
        assert!(!Arc::ptr_eq(&created.initial_frame.record, &next.record));
        let old = created
            .initial_frame
            .record
            .as_any()
            .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
            .unwrap();
        let new = next
            .record
            .as_any()
            .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
            .unwrap();
        assert!(!Arc::ptr_eq(&old.artifact, &new.artifact));
        assert_eq!(
            created.initial_frame.revision, 1,
            "old snapshot stays immutable"
        );
    });
}

#[test]
fn document_image_container_is_part_of_the_published_snapshot() {
    crate::native_test_thread::run(|| {
        let assembly = crate::CreateLayoutAssembly();
        let mut decoder = SVGImageDecoder::new(&assembly);
        let original = ContainerKey::new(64, 32, 1.0, 1.0);
        let mut created = decoder
            .create(23, Arc::from(ANIMATED_SVG), "image/svg+xml", &original)
            .unwrap();
        let resized = ContainerKey::new(128, 64, 1.0, 2.0);
        let effects = created
            .image
            .apply_mutation(DocumentImageMutation::SetContainer {
                container: resized.clone(),
            })
            .unwrap();
        let frame = effects
            .iter()
            .find_map(|effect| match effect {
                DocumentImageEffect::FrameChanged { frame, .. } => Some(frame),
                _ => None,
            })
            .unwrap();
        assert_eq!(frame.container_key, resized);
        assert_ne!(frame.container_key, created.initial_frame.container_key);
    });
}
