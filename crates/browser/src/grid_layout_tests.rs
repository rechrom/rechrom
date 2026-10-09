use layoutng_assembly::{
    fragment_tree::{FragmentKind, FragmentNode},
    internal::layout_input::{ConstraintSpace, FontFace, Size},
};
#[test]
#[cfg_attr(
    debug_assertions,
    ignore = "Compare the source NDEBUG build with cargo test --release; named areas violate a source GridSpan DCHECK"
)]
fn native_grid_layout_matches_unchanged_cpp() {
    crate::native_test_thread::run(body);
}
#[test]
fn native_grid_layout_debug_matches_unchanged_cpp() {
    crate::native_test_thread::run(debug_body);
}
fn debug_body() {
    run_matrix(true);
}
fn body() {
    run_matrix(false);
}
fn run_matrix(smoke: bool) {
    let reference = include_str!("../../../artifacts/cpp-reference/grid-native-layout-results.tsv");
    assert_eq!(reference.lines().count(), 1092);
    let input = include_str!("../../../artifacts/cpp-reference/grid-native-layout.html");
    // The first Grid is independent of later siblings. Keep its C++ input
    // unchanged while excluding the source named-area DCHECK conflict.
    let smoke_input = format!(
        "{}</body></html>",
        &input[..input.find("</section>").unwrap() + "</section>".len()]
    );
    let mut owner = html::html_parser::ParseHTML(if smoke { &smoke_input } else { input });
    owner
        .GetDocumentMut()
        .AppendStyleSheet(style::ParseCSS(include_str!(
            "../../../artifacts/cpp-reference/grid-native-layout.css"
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
    for stage in 0..if smoke { 1 } else { 7 } {
        let d = owner.GetDocumentMut();
        let attr = |d: &mut dom::Document, id: &str, name: &str, value: &str| {
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
                d,
                "g0",
                "style",
                "width:355.5px;grid-template-columns:repeat(4,minmax(25px,1fr));gap:11px",
            );
            attr(d, "i6_1", "style", "grid-column:span 3;order:-1");
            attr(d, "g15", "style", "width:330px;gap:12px");
        }
        if stage == 2 {
            attr(
                d,
                "g0",
                "style",
                "width:187px;grid-template-columns:repeat(2,1fr);direction:rtl",
            );
            attr(d, "i2_4", "style", "display:block;grid-column:-1");
            d.AppendChild(ids["g6"], ids["i6_0"]);
            attr(d, "g15", "style", "width:270px;gap:4px");
        }
        crate::style_services::ResolveLayoutStyles(&mut owner, &space);
        engine.ApplyMutation(layoutng_assembly::layout_engine::LayoutMutation::Constraints(&space));
        let root = crate::native_test_thread::BuildDOMProjection(
            &mut owner,
            &dom::UserInteractionState::default(),
            &mut engine,
        );
        let _scope = layoutng_assembly::internal::layout_pass_scope::LayoutObjectFactoryScope::new(
            &assembly.objects,
        );
        let mut grids = std::collections::BTreeMap::new();
        fn collect(
            o: &layoutng_assembly::internal::layout_object::LayoutObject,
            out: &mut std::collections::BTreeMap<
                u64,
                *const layoutng_grid::layout_grid::LayoutGrid,
            >,
        ) {
            if o.IsLayoutGrid() && !o.GetNode().is_null() {
                out.insert(
                    unsafe { &*o.GetNode() }.InputId(),
                    foundation::To::<layoutng_grid::layout_grid::LayoutGrid>(o),
                );
            }
            let mut c = o.SlowFirstChild();
            while !c.is_null() {
                let child = unsafe { &*c };
                collect(child, out);
                c = child.NextSibling();
            }
        }
        collect(unsafe { &*root }, &mut grids);
        let fragments =
            if stage < 3 {
                engine.Layout();
                let fragments = engine
                    .TakeLayoutResult()
                    .expect("successful layout has fragments");
                std::rc::Rc::try_unwrap(fragments).unwrap_or_else(|shared| (*shared).clone())
            } else {
                use layoutng_assembly::internal::{
                    block_node::BlockNode, constraint_space::FragmentationType,
                    constraint_space_builder::ConstraintSpaceBuilder,
                    layout_pass_scope::LayoutPassScope,
                };
                let _heap = foundation::LayoutHeapScope::new();
                let mut environment =
                    assembly.boundary.create_environment.unwrap()(unsafe { &mut *root }, &space);
                let parent_space = assembly.boundary.prepare_constraints.unwrap()(
                    unsafe { &*(*root).GetNode() }.InputStyle(),
                    &space,
                    false,
                );
                assembly.boundary.prepare_tree.unwrap()(
                    unsafe { &mut *root },
                    &space,
                    &parent_space,
                    environment.ReusesPreparedFonts(),
                );
                let _pass = LayoutPassScope::new(&assembly.algorithms, &assembly.objects);
                let grid = grids[&owner.GetDocument().Node(ids["g0"]).Id()].cast_mut();
                let mut builder = ConstraintSpaceBuilder::new(
                    &parent_space,
                    unsafe { &*grid }.StyleRef().GetWritingDirection(),
                    true,
                );
                let mut available = parent_space.AvailableSize().clone();
                available.inline_size = foundation::LayoutUnit::from_signed(1024);
                available.block_size = foundation::kIndefiniteSize;
                builder.SetAvailableSize(available);
                builder.SetPercentageResolutionSize(parent_space.PercentageResolutionSize());
                builder.SetFragmentationType(FragmentationType::kFragmentPage);
                builder.SetFragmentainerBlockSize(foundation::LayoutUnit::from_signed(
                    if stage == 4 { 72 } else { 40 },
                ));
                builder.SetFragmentainerOffset(foundation::LayoutUnit::default());
                builder.SetIsAtFragmentainerStart();
                builder.SetShouldPropagateChildBreakValues(true);
                let grid_space = builder.ToConstraintSpace();
                let node = BlockNode::new(grid.cast());
                let mut result = node.Layout(
                    &grid_space,
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null(),
                );
                for _ in 0..if stage >= 5 { stage - 4 } else { 0 } {
                    let physical = foundation::To::<
                        layoutng_assembly::physical_box_fragment::PhysicalBoxFragment,
                    >(unsafe { &*result }.GetPhysicalFragment());
                    let token = unsafe { &*physical }.GetBreakToken();
                    assert!(!token.is_null(), "expected continuation token");
                    result = node.Layout(&grid_space, token, std::ptr::null(), std::ptr::null());
                }
                let fragments = assembly.boundary.export_tree.unwrap()(
                    unsafe { &mut *grid.cast() },
                    unsafe { &*result },
                    &space,
                );
                environment.Commit(unsafe { &mut *root });
                fragments
            };
        for line in
            include_str!("../../../artifacts/cpp-reference/grid-native-layout-results.tsv").lines()
        {
            let (key, rest) = line.split_once('\t').unwrap();
            let (record_stage, id) = key.split_once(':').unwrap();
            if smoke && id != "g0" && !id.starts_with("i0_") {
                continue;
            }
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
            if let Some(grid) = grids.get(&node.Id()).filter(|p| {
                find(&fragments, node.Id()).is_some() && !unsafe { &***p }.LayoutData().is_null()
            }) {
                let grid = unsafe { &**grid };
                for direction in [
                    layoutng_style::style::grid_enums::GridTrackSizingDirection::kForColumns,
                    layoutng_style::style::grid_enums::GridTrackSizingDirection::kForRows,
                ] {
                    values.extend([
                        grid.AutoRepeatCountForDirection(direction) as f64,
                        grid.ExplicitGridStartForDirection(direction) as f64,
                        grid.ExplicitGridEndForDirection(direction) as f64,
                        grid.GridGap(direction).ToDouble(),
                    ]);
                    let sizes = grid.TrackSizesForComputedStyle(direction);
                    values.push(sizes.len() as f64);
                    values.extend(sizes.iter().map(|v| v.ToDouble()));
                    let positions = grid.GridTrackPositions(direction);
                    values.push(positions.len() as f64);
                    values.extend(positions.iter().map(|v| v.ToDouble()));
                }
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
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(if smoke {
            "../../artifacts/grid-debug-layout-actual.tsv"
        } else {
            "../../artifacts/grid-native-layout-actual.tsv"
        }),
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
