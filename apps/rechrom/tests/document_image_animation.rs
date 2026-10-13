use std::{sync::Arc, time::Duration};

use image_resource::{
    ContainerKey, DocumentImageDecoder, DocumentImageEffect, DocumentImageMutation,
};

const SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32">
<style>@keyframes move{from{transform:translateX(0px)}to{transform:translateX(20px)}}rect{animation:move 1s linear infinite;fill:red}</style>
<rect width="16" height="16"/></svg>"#;
const RESPONSIVE_SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32">
<style>rect{fill:red}@media (min-width:100px){rect{fill:blue}}</style>
<rect width="100%" height="100%"/></svg>"#;

#[test]
fn svg_presentation_transform_and_fill_reach_document_paint_record() {
    on_layout_thread(|| {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 220 190">
          <g transform="translate(-111.5 -66.5) scale(.35)">
            <path fill="#15919d" d="M519 324V282C519 244 551 211 590 216L849 259C883 265 910 290 910 326V545C910 584 880 609 844 602L790 594V538L830 545C849 549 861 538 861 521V354C861 337 850 327 834 325L597 283C580 280 568 290 568 306V331Z"/>
            <path fill="#062142" d="M355 388C355 347 386 320 427 327L728 376C757 381 775 403 775 432V476C760 454 745 446 717 441L441 392C422 388 409 399 409 419V575C409 591 418 602 434 605L701 648C714 651 722 645 722 631V461C750 467 775 486 775 512V651C775 692 747 716 710 709L410 660C377 654 355 630 355 595Z"/>
          </g>
        </svg>"##;
        let assembly = browser::CreateLayoutAssembly();
        let constraints = browser::CreateBrowserConstraints(220, 190);
        let mut decoder = browser::CreateSVGImageDecoderWithConstraints(&assembly, &constraints);
        let frame = decoder
            .create(
                99,
                Arc::from(svg.as_slice()),
                "image/svg+xml",
                &ContainerKey::new(220, 190, 1.0, 1.0),
            )
            .unwrap()
            .initial_frame;
        let record = frame
            .record
            .as_any()
            .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
            .unwrap();
        let paths = record
            .artifact
            .items
            .iter()
            .filter(|item| item.r#type == paint::paint_engine::DisplayItemType::kDrawPath)
            .collect::<Vec<_>>();
        assert_eq!(paths.len(), 2);
        assert!((paths[0].color.red - 0x15 as f32 / 255.0).abs() < 1e-6);
        assert!((paths[0].color.green - 0x91 as f32 / 255.0).abs() < 1e-6);
        assert!((paths[1].color.blue - 0x42 as f32 / 255.0).abs() < 1e-6);
        assert!(record.artifact.items.iter().any(|item| {
            item.r#type == paint::paint_engine::DisplayItemType::kConcat
                && (item.transform.values[0] - 0.35).abs() < 1e-9
                && (item.transform.values[12] + 111.5).abs() < 1e-9
                && (item.transform.values[13] + 66.5).abs() < 1e-9
        }));
    });
}

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
        let mut decoder = browser::CreateSVGImageDecoderWithConstraints(&assembly, &constraints);
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
        let mut decoder = browser::CreateSVGImageDecoderWithConstraints(&assembly, &constraints);
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

#[test]
fn svg_user_agent_rules_survive_repeated_isolated_document_lifecycles() {
    on_layout_thread(|| {
        // Chromium svg.css supplies the default 0 0 origin at UA priority.
        // The second group verifies that author style still wins the cascade.
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 220 190">
          <g transform="translate(10 20) scale(.35)">
            <rect width="20" height="10" fill="red"/>
          </g>
          <g style="transform-origin:10px 20px" transform="translate(70 60) scale(2)">
            <rect width="20" height="10" fill="blue"/>
          </g>
        </svg>"##;
        let assembly = browser::CreateLayoutAssembly();
        let constraints = browser::CreateBrowserConstraints(220, 190);
        for mut decoder in [
            browser::CreateSVGImageDecoder(&assembly),
            browser::CreateSVGImageDecoderWithConstraints(&assembly, &constraints),
        ] {
            let container = ContainerKey::new(220, 190, 1.0, 1.0);
            let mut created = decoder
                .create(100, Arc::from(svg.as_slice()), "image/svg+xml", &container)
                .unwrap();
            let check = |frame: &image_resource::DocumentImageFrame| {
                let record = frame
                    .record
                    .as_any()
                    .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
                    .unwrap();
                for (scale, x, y) in [(0.35, 10.0, 20.0), (2.0, 60.0, 40.0)] {
                    assert!(
                        record.artifact.items.iter().any(|item| {
                            item.r#type == paint::paint_engine::DisplayItemType::kConcat
                                && (item.transform.values[0] - scale).abs() < 1e-9
                                && (item.transform.values[5] - scale).abs() < 1e-9
                                && (item.transform.values[12] - x).abs() < 1e-9
                                && (item.transform.values[13] - y).abs() < 1e-9
                        }),
                        "expected scale {scale} and translation ({x}, {y}) at revision {}",
                        frame.revision
                    );
                }
            };
            check(&created.initial_frame);
            let original_record = created.initial_frame.record.clone();
            let mut revision = 1;
            for mutation in [
                DocumentImageMutation::SetContainer {
                    container: ContainerKey::new(330, 285, 1.0, 1.0),
                },
                DocumentImageMutation::SetPreferredColorScheme { scheme: 1 },
                DocumentImageMutation::SetPreferredColorScheme { scheme: 0 },
            ] {
                let effects = created.image.apply_mutation(mutation).unwrap();
                let next = effects
                    .iter()
                    .find_map(|effect| match effect {
                        DocumentImageEffect::FrameChanged { frame, .. } => Some(frame),
                        _ => None,
                    })
                    .expect("changed image environment publishes a new record");
                revision += 1;
                assert_eq!(next.revision, revision);
                assert!(!Arc::ptr_eq(&original_record, &next.record));
                check(next);
                check(&created.initial_frame);
                assert_eq!(created.initial_frame.revision, 1);
            }
        }
    });
}
