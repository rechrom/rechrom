use crate::{
    layout_table::LayoutTable, layout_table_cell::LayoutTableCell,
    layout_table_column::LayoutTableColumn, layout_table_row::LayoutTableRow,
    layout_table_section::LayoutTableSection,
};
use foundation::To;
use layoutng_assembly::internal::{
    layout_box::LayoutBox, layout_object::LayoutObject,
    layout_object_factory_set::LayoutObjectFactorySet, layout_pass_scope::LayoutObjectFactoryScope,
    table_layout_algorithm_types::TableGroupedChildren, table_node::TableNode,
};
use layoutng_assembly::{layout_assembly::LayoutAssembly, layout_engine::LayoutEngine};
use layoutng_replaced as _;
use std::collections::BTreeMap;

#[test]
fn native_table_objects_match_unchanged_cpp_tree_and_mutations() {
    crate::native_test_thread::run(native_objects_body);
}
fn native_objects_body() {
    let mut owner = html::html_parser::ParseHTML(include_str!(
        "../../../artifacts/cpp-reference/table-native-tree.html"
    ));
    owner
        .GetDocumentMut()
        .AppendStyleSheet(style::ParseCSS(include_str!(
            "../../../artifacts/cpp-reference/table-native-tree.css"
        )));
    let mut style_engine = style::StyleEngine::new(&owner);
    style_engine
        .Update(
            &mut owner,
            &style::media_queries::media_values_cached::MediaValuesCachedData {
                viewport_width: 1024.0,
                viewport_height: 768.0,
                small_viewport_width: 1024.0,
                small_viewport_height: 768.0,
                large_viewport_width: 1024.0,
                large_viewport_height: 768.0,
                dynamic_viewport_width: 1024.0,
                dynamic_viewport_height: 768.0,
                device_pixel_ratio: 1.0,
                em_size: 16.0,
                ..Default::default()
            },
            &[],
        )
        .expect("style update");
    let ids: BTreeMap<_, _> = (0..owner.GetDocument().NodeCount())
        .filter_map(|i| {
            let n = owner.GetDocument().Node(i);
            n.FindAttribute("id").map(|a| (n.Id(), a.value.clone()))
        })
        .collect();
    let mut factories = LayoutObjectFactorySet::default();
    crate::assembly::InstallTableObjects(&mut factories);
    let assembly = LayoutAssembly {
        objects: factories,
        ..LayoutAssembly::default()
    };
    let mut engine = LayoutEngine::new(&assembly);
    let root = BuildDOMProjection(
        &mut owner,
        &dom::UserInteractionState::default(),
        &mut engine,
    );
    let _scope = LayoutObjectFactoryScope::new(&factories);
    let mut objects = BTreeMap::new();
    fn gather(
        o: &LayoutObject,
        ids: &BTreeMap<u64, String>,
        objects: &mut BTreeMap<String, *mut LayoutObject>,
    ) {
        if let Some(n) = unsafe { o.GetNode().as_ref() } {
            if let Some(id) = ids.get(&n.InputId()) {
                objects.insert(id.clone(), (o as *const LayoutObject).cast_mut());
            }
        }
        let mut c = o.SlowFirstChild();
        while !c.is_null() {
            let child = unsafe { &*c };
            gather(child, ids, objects);
            c = child.NextSibling();
        }
    }
    gather(unsafe { &*root }, &ids, &mut objects);
    fn dump(
        o: &LayoutObject,
        path: &str,
        stage: usize,
        ids: &BTreeMap<u64, String>,
        out: &mut String,
    ) {
        use std::fmt::Write;
        write!(
            out,
            "{stage}:{path}\t{}\t{}\t{}",
            o.GetName(),
            o.IsAnonymous() as u8,
            o.StyleRef().Display() as i32
        )
        .unwrap();
        for b in [
            o.IsBox(),
            o.IsLayoutBlock(),
            o.IsLayoutBlockFlow(),
            o.IsTable(),
            o.IsTableSection(),
            o.IsTableRow(),
            o.IsTableCell(),
            o.IsLayoutTableCol(),
            o.IsTableCaption(),
            o.IsEligibleForPaintOrLayoutContainment(),
            o.IsEligibleForSizeContainment(),
            o.RespectsCSSOverflow(),
            o.VisualRectRespectsVisibility(),
        ] {
            write!(out, "\t{}", b as u8).unwrap();
        }
        let n = unsafe { o.GetNode().as_ref() };
        write!(
            out,
            "\t{}\t{}",
            n.map_or("-", |n| n.InputDebugName()),
            n.map_or("-", |n| ids.get(&n.InputId()).map_or("", String::as_str))
        )
        .unwrap();
        if o.IsBox() {
            write!(
                out,
                "\t{}",
                unsafe { &*To::<LayoutBox>(o) }.CreatesNewFormattingContext() as u8
            )
            .unwrap();
        } else {
            out.push_str("\t-");
        }
        if o.IsTableCell() {
            let c = unsafe { &*To::<LayoutTableCell>(o) };
            write!(
                out,
                "\t{}\t{}\t{}\t{}",
                c.ColSpan(),
                c.ComputedRowSpan(),
                c.ResolvedRowSpan(),
                c.RowIndex()
            )
            .unwrap();
        } else {
            out.push_str("\t-\t-\t-\t-");
        }
        if o.IsTableRow() {
            write!(out, "\t{}", unsafe { &*To::<LayoutTableRow>(o) }.RowIndex()).unwrap();
        } else {
            out.push_str("\t-");
        }
        if o.IsTableSection() {
            write!(
                out,
                "\t{}",
                unsafe { &*To::<LayoutTableSection>(o) }.NumRows()
            )
            .unwrap();
        } else {
            out.push_str("\t-");
        }
        if o.IsLayoutTableCol() {
            write!(out, "\t{}", unsafe { &*To::<LayoutTableColumn>(o) }.Span()).unwrap();
        } else {
            out.push_str("\t-");
        }
        if o.IsTable() {
            let node = TableNode::new((o as *const LayoutObject).cast_mut().cast());
            let borders = unsafe { &*node.GetTableBorders() };
            let s = borders.TableBorder();
            for v in [s.inline_start, s.inline_end, s.block_start, s.block_end] {
                write!(out, "\t{}", v.RawValue()).unwrap();
            }
            write!(out, "\t{}\t{}", borders.EdgesPerRow(), borders.EdgeCount()).unwrap();
            for i in 0..borders.EdgeCount() {
                let i = i as u32;
                write!(
                    out,
                    "\t{}\t{}\t{}\t{}",
                    borders.BorderWidth(i).RawValue(),
                    borders.BorderStyle(i) as i32,
                    borders.BoxOrder(i),
                    borders.CanPaint(i) as u8
                )
                .unwrap();
            }
            let grouped = TableGroupedChildren::new(&node);
            let mut it = grouped.begin();
            while !it.Equals(&grouped.end()) {
                let section = it.Dereference();
                let n = unsafe { &*section.GetLayoutBox() }.GetNode();
                let n = unsafe { n.as_ref() };
                write!(
                    out,
                    "\t{}\t{}",
                    n.map_or("-", |n| n.InputDebugName()),
                    n.map_or("-", |n| ids.get(&n.InputId()).map_or("", String::as_str))
                )
                .unwrap();
                it.Increment();
            }
        }
        out.push('\n');
        let mut c = o.SlowFirstChild();
        let mut i = 0;
        while !c.is_null() {
            let child = unsafe { &*c };
            dump(child, &format!("{path}.{i}"), stage, ids, out);
            i += 1;
            c = child.NextSibling();
        }
    }
    let mut actual = String::new();
    let mut inserted: *mut LayoutTableCell = std::ptr::null_mut();
    for stage in 0..3 {
        if stage == 1 {
            let t = unsafe { &mut *To::<LayoutTable>(objects["css-table"]) };
            inserted = LayoutTableCell::CreateAnonymousWithParent(t);
            t.AddChild(inserted.cast(), objects["directblock"]);
        }
        if stage == 2 {
            unsafe { &mut *(*inserted).Parent() }.RemoveChild(inserted.cast());
            unsafe { &mut *inserted }.Destroy();
        }
        dump(unsafe { &*root }, "0", stage, &ids, &mut actual);
    }
    let expected = include_str!("../../../artifacts/cpp-reference/table-native-tree-results.tsv");
    if actual != expected {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../artifacts");
        std::fs::write(root.join("table-native-tree-actual.tsv"), &actual).unwrap();
    }
    assert_eq!(
        actual.lines().count(),
        expected.lines().count(),
        "native table tree record count"
    );
    for (a, e) in actual.lines().zip(expected.lines()) {
        assert_eq!(
            a,
            e,
            "native table record {}",
            e.split('\t').next().unwrap()
        );
    }
}

/// Fixture wiring lives in the consuming host, never in production DOM.
pub(crate) fn BuildDOMProjection(
    owner: &mut dom::DOM,
    interaction: &dom::UserInteractionState,
    engine: &mut layoutng_assembly::layout_engine::LayoutEngine,
) -> *mut layoutng_assembly::internal::layout_object::LayoutObject {
    owner.EmitLayoutMutations(interaction, |mutation| {
        engine.ApplyMutation(mutation);
    });
    engine.GetLayoutTree().expect("projected tree").Root() as *const _ as *mut _
}
