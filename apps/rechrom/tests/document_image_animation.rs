use std::{sync::Arc, time::Duration};

use document_image::SVGImageDecoder;
use image_resource::{
    ContainerKey, DocumentImageDecoder, DocumentImageEffect, DocumentImageMutation,
};

const SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32">
<style>@keyframes move{from{transform:translateX(0px)}to{transform:translateX(20px)}}rect{animation:move 1s linear infinite;fill:red}</style>
<rect width="16" height="16"/></svg>"#;
const RESPONSIVE_SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32">
<style>rect{fill:red}@media (min-width:100px){rect{fill:blue}}</style>
<rect width="100%" height="100%"/></svg>"#;

fn on_layout_thread(test: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("document-image-test".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(test)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn changed_samples_publish_new_immutable_records_without_zero_time_churn() {
    on_layout_thread(|| {
        let assembly = browser::CreateLayoutAssembly();
        let constraints = browser::CreateBrowserConstraints(320, 200);
        let mut decoder = SVGImageDecoder::new_with_constraints(&assembly, &constraints);
        let container = ContainerKey::new(64, 32, 1.0, 1.0);
        let mut created = decoder
            .create(7, Arc::from(SVG), "image/svg+xml", &container)
            .unwrap();
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
            .unwrap();
        assert_eq!(next.revision, 2);
        let old_record = created
            .initial_frame
            .record
            .as_any()
            .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
            .unwrap();
        let next_record = next
            .record
            .as_any()
            .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
            .unwrap();
        assert!(old_record.artifact.items != next_record.artifact.items);
        assert_eq!(created.initial_frame.revision, 1);
        assert!(!Arc::ptr_eq(&created.initial_frame.record, &next.record));
    });
}

#[test]
fn container_configuration_versions_the_record_cache_key() {
    on_layout_thread(|| {
        let assembly = browser::CreateLayoutAssembly();
        let constraints = browser::CreateBrowserConstraints(320, 200);
        let mut decoder = SVGImageDecoder::new_with_constraints(&assembly, &constraints);
        let initial = ContainerKey::new(64, 32, 1.0, 1.0);
        let mut created = decoder
            .create(9, Arc::from(RESPONSIVE_SVG), "image/svg+xml", &initial)
            .unwrap();
        let resized = ContainerKey::new(128, 64, 1.0, 2.0);
        let effects = created
            .image
            .apply_mutation(DocumentImageMutation::SetContainer {
                container: resized.clone(),
            })
            .unwrap();
        let next = effects
            .iter()
            .find_map(|effect| match effect {
                DocumentImageEffect::FrameChanged { frame, .. } => Some(frame),
                _ => None,
            })
            .unwrap();
        assert_eq!(next.container_key, resized);
        assert_ne!(next.container_key, created.initial_frame.container_key);
        let old_record = created
            .initial_frame
            .record
            .as_any()
            .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
            .unwrap();
        let next_record = next
            .record
            .as_any()
            .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
            .unwrap();
        assert_eq!(old_record.record_size.width, 64);
        assert_eq!(next_record.record_size.width, 128);
        assert!(old_record.artifact.items != next_record.artifact.items);
    });
}
