use layoutng_assembly::{
    fragment_tree::{FragmentKind, FragmentNode},
    internal::layout_input::{ConstraintSpace, FontFace, Size},
};
#[test]
fn native_table_layout_matches_unchanged_cpp() {
    crate::native_test_thread::run(body);
}
fn body() {
    let reference =
        include_str!("../../../artifacts/cpp-reference/table-native-layout-results.tsv");
    assert_eq!(reference.lines().count(), 630);
    for stage in 0..3 {
        assert_eq!(
            reference
                .lines()
                .filter(|line| line.starts_with(&format!("{stage}:")))
                .count(),
            210
        );
    }
    let mut owner = html::html_parser::ParseHTML(include_str!(
        "../../../artifacts/cpp-reference/table-native-layout.html"
    ));
    owner
        .GetDocumentMut()
        .AppendStyleSheet(style::ParseCSS(include_str!(
            "../../../artifacts/cpp-reference/table-native-layout.css"
        )));
    let mut space = ConstraintSpace::default();
    space.available_size = Size {
        width: 1024.0,
        height: 768.0,
    };
    space.fonts.push(FontFace {
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
            overscroll_type: layoutng_assembly::internal::layout_input::OverscrollType::kTransform,
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

    fn find(f: &FragmentNode, id: u64) -> Option<&FragmentNode> {
        if f.node_id == id && f.kind == FragmentKind::kBox {
            return Some(f);
        }
        f.children.iter().find_map(|c| find(c, id))
    }
    let mut actual = String::new();
    let mut mismatches = Vec::new();
    let ids: std::collections::BTreeMap<_, _> = (0..owner.GetDocument().NodeCount())
        .filter_map(|i| {
            owner
                .GetDocument()
                .Node(i)
                .FindAttribute("id")
                .map(|a| (a.value.clone(), i))
        })
        .collect();
    let assembly = crate::CreateLayoutAssembly();
    let mut engine = crate::LayoutEngine::new(&assembly);
    for stage in 0..3 {
        let d = owner.GetDocumentMut();
        let mut attr = |id: &str, name: &str, value: &str| {
            d.SetAttribute(
                ids[id],
                dom::persistent_document::DOMAttribute {
                    local_name: name.into(),
                    value: value.into(),
                    ..Default::default()
                },
            );
        };
        if stage == 1 {
            attr(
                "t0",
                "style",
                "width:355.5px;height:215px;table-layout:fixed;border-collapse:collapse",
            );
            attr("hc0", "colspan", "3");
            attr("a0", "rowspan", "1");
            attr("c1", "style", "width:57%;visibility:collapse");
        }
        if stage == 2 {
            attr(
                "t0",
                "style",
                "width:187px;height:auto;table-layout:auto;direction:rtl",
            );
            attr("r0", "style", "visibility:collapse");
            attr("cap0", "style", "caption-side:bottom;margin:9px");
            attr("e0", "style", "height:90px;vertical-align:middle");
            attr("c1", "style", "width:15%");
            d.AppendChild(ids["r1"], ids["f0"]);
        }
        crate::style_services::ResolveLayoutStyles(&mut owner, &space);
        engine.ApplyMutation(layoutng_assembly::layout_engine::LayoutMutation::Constraints(&space));
        let root = crate::native_test_thread::BuildDOMProjection(
            &mut owner,
            &dom::UserInteractionState::default(),
            &mut engine,
        );
        engine.Layout();
        let fragments = engine
            .GetLayoutResult()
            .expect("successful layout has fragments");
        let _scope = layoutng_assembly::internal::layout_pass_scope::LayoutObjectFactoryScope::new(
            &assembly.objects,
        );
        let mut columns = std::collections::BTreeMap::new();
        fn collect(
            o: &layoutng_assembly::internal::layout_object::LayoutObject,
            out: &mut std::collections::BTreeMap<
                u64,
                *const layoutng_assembly::internal::layout_box::LayoutBox,
            >,
        ) {
            if o.IsLayoutTableCol() && !o.GetNode().is_null() {
                out.insert(
                    unsafe { &*o.GetNode() }.InputId(),
                    foundation::To::<layoutng_assembly::internal::layout_box::LayoutBox>(o),
                );
            }
            let mut c = o.SlowFirstChild();
            while !c.is_null() {
                let child = unsafe { &*c };
                collect(child, out);
                c = child.NextSibling();
            }
        }
        collect(unsafe { &*root }, &mut columns);
        for line in
            include_str!("../../../artifacts/cpp-reference/table-native-layout-results.tsv").lines()
        {
            let (key, rest) = line.split_once('\t').unwrap();
            let (record_stage, id) = key.split_once(':').unwrap();
            if record_stage.parse::<usize>().unwrap() != stage {
                continue;
            }
            let document = owner.GetDocument();
            let node = (0..document.NodeCount())
                .map(|i| document.Node(i))
                .find(|n| n.FindAttribute("id").is_some_and(|a| a.value == id))
                .unwrap();
            let mut values = if let Some(f) = find(&fragments, node.Id()) {
                let p = &f.paint;
                vec![
                    f.size.width.round(),
                    f.size.height.round(),
                    (f.content_size.width + p.padding.left + p.padding.right).round(),
                    (f.content_size.height + p.padding.top + p.padding.bottom).round(),
                    f.content_size.width,
                    f.content_size.height,
                    p.border.left.round(),
                    p.border.top.round(),
                    p.scroll_size.width.round(),
                    p.scroll_size.height.round(),
                ]
            } else {
                vec![0.0; 10]
            };
            let rects = paint::paint_engine::FragmentClientRects(&fragments, node.Id());
            values.push(rects.len() as f64);
            for r in rects {
                values.extend([r.x, r.y, r.width, r.height]);
            }
            if let Some(column) = columns.get(&node.Id()) {
                let column = unsafe { &**column };
                let size = column.StitchedSize();
                let location = column.PhysicalLocation();
                values.extend([
                    size.width.ToDouble(),
                    size.height.ToDouble(),
                    location.left.ToDouble(),
                    location.top.ToDouble(),
                ]);
            }
            use std::fmt::Write;
            write!(&mut actual, "{key}").unwrap();
            for v in &values {
                write!(&mut actual, "\t{v}").unwrap();
            }
            actual.push('\n');
            let expected: Vec<f64> = rest.split('\t').map(|s| s.parse().unwrap()).collect();
            if values != expected {
                mismatches.push(format!("{key}: actual {values:?}; expected {expected:?}"));
            }
        }
    }
    std::fs::write(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../artifacts/table-native-layout-actual.tsv"),
        actual,
    )
    .unwrap();
    assert!(
        mismatches.is_empty(),
        "{} mismatches\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}
