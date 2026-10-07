use image_decoder::{
    document_image_decoder::DocumentImageDecoder, image_decoder::ImageDecodeInput,
    svg_image_decoder::SVGImageDecoder,
};
#[test]
fn source_svg_image_pipeline_matches_unchanged_cpp_rgba() {
    crate::native_test_thread::run(body);
}
fn body() {
    let assembly = crate::CreateLayoutAssembly();
    let mut decoder = SVGImageDecoder::new(&assembly);
    let mut space = layoutng_assembly::internal::layout_input::ConstraintSpace::default();
    space.available_size = layoutng_assembly::internal::layout_input::Size {
        width: 1024.0,
        height: 768.0,
    };
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
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../artifacts/cpp-reference/svg-images");
    let reference = include_str!("../../../artifacts/cpp-reference/svg-images/results.tsv");
    assert_eq!(reference.lines().count(), 19);
    let mut failures = Vec::new();
    for line in reference.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        let name = fields[0];
        let bytes = std::fs::read(root.join(format!("{name}.svg"))).unwrap();
        let input = ImageDecodeInput {
            bytes: &bytes,
            mime_type: "image/svg+xml",
        };
        assert_eq!(
            decoder.CanDecode(&input),
            fields[1] == "1",
            "{name} capability"
        );
        match decoder.Decode(&input, &space) {
            Ok(img) => {
                assert_ne!(fields[2], "error", "{name} expected failure");
                assert_eq!(
                    (img.width, img.height, img.rgba8.len()),
                    (
                        fields[2].parse().unwrap(),
                        fields[3].parse().unwrap(),
                        fields[4].parse().unwrap()
                    ),
                    "{name} dimensions"
                );
                let expected = std::fs::read(root.join(format!("{name}.rgba"))).unwrap();
                assert_eq!(img.rgba8.len(), expected.len());
                let mut doc = html::html_parser::ParseHTMLBytes(&bytes);
                dom::style_resolver::AddStyleSheet(&mut doc,cssom::css_style_sheet::ParseCSS("html,body{margin:0;padding:0;overflow:hidden;background:transparent;}html,body{width:100%;height:100%;}svg{display:block;}"));
                dom::style_resolver::ResolveComputedStyles(
                    &mut doc,
                    &dom::style_resolver::StyleEnvironment {
                        viewport_width: Some(f64::from(img.width)),
                        viewport_height: Some(f64::from(img.height)),
                        resolution_dppx: Some(1.0),
                        ..Default::default()
                    },
                    &[],
                );
                let mut cs = space.clone();
                cs.available_size = layoutng_assembly::internal::layout_input::Size {
                    width: f64::from(img.width),
                    height: f64::from(img.height),
                };
                cs.viewport = Some(
                    layoutng_assembly::internal::layout_input::ViewportGeometry {
                        size: layoutng_assembly::internal::layout_input_types::IntSize {
                            width: img.width as i32,
                            height: img.height as i32,
                        },
                        ..Default::default()
                    },
                );
                let mut engine = crate::LayoutEngine::new(&assembly);
                engine.ApplyMutation(
                    layoutng_assembly::layout_engine::LayoutMutation::Constraints(&cs),
                );
                crate::native_test_thread::BuildDOMProjection(
                    &mut doc,
                    &dom::UserInteractionState::default(),
                    &mut engine,
                );
                engine.Layout();
                let fragments = engine
                    .GetLayoutResult()
                    .expect("successful layout has fragments");
                let items = paint::paint_engine::Paint(&fragments);
                let mut dump = String::new();
                use std::fmt::Write;
                for i in items.items.iter() {
                    write!(
                        dump,
                        "{}\t{}\t{}\t{}\t{}\t{}",
                        i.r#type as u8,
                        i.rect.x,
                        i.rect.y,
                        i.rect.width,
                        i.rect.height,
                        i.stroke_width
                    )
                    .unwrap();
                    for v in i.transform.values {
                        write!(dump, "\t{v}").unwrap();
                    }
                    dump.push('\n');
                    for p in &i.path {
                        writeln!(
                            dump,
                            "path\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                            p.verb as u8,
                            p.point.x,
                            p.point.y,
                            p.control1.x,
                            p.control1.y,
                            p.control2.x,
                            p.control2.y,
                            p.conic_weight
                        )
                        .unwrap();
                    }
                }
                std::fs::write(root.join(format!("{name}-rust-items.tsv")), dump).unwrap();
                let different = img
                    .rgba8
                    .chunks_exact(4)
                    .zip(expected.chunks_exact(4))
                    .filter(|(a, b)| a != b)
                    .count();
                if different != 0 {
                    std::fs::write(root.join(format!("{name}-rust.rgba")), &img.rgba8).unwrap();
                    failures.push(format!(
                        "{name}: {different} / {} different pixels",
                        expected.len() / 4
                    ));
                }
            }
            Err(e) => {
                assert_eq!(fields[2], "error", "{name}: {e}");
                assert_eq!(e.to_string(), fields[3], "{name} error");
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
