use std::{collections::HashSet, rc::Rc};

use layoutng_assembly::fragment_tree::FragmentNode;
use layoutng_assembly::layout_engine::LayoutEngine;
use paint::paint_engine::PaintArtifact;

/// Complete the persistent paint cycle while the engine still roots its
/// native client owners. Exported addresses are only compared to live IDs.
pub(super) fn CommitPaint(
    layout: &mut LayoutEngine,
    caret: &layoutng_assembly::caret::FrameCaret,
    fragments: &mut Rc<FragmentNode>,
    artifact: &mut PaintArtifact,
) {
    let used_clients: HashSet<_> = artifact
        .display_items
        .iter()
        .map(|item| item.id.client_id)
        .chain(artifact.chunks.iter().map(|chunk| chunk.id.client_id))
        .collect();
    let caret_id = caret.display_item_client().Id();
    if used_clients.contains(&caret_id) {
        caret.ValidateForCommittedPaint(caret_id);
    }
    layout.CommitPaintClients(used_clients, fragments);
    // cpp: platform/graphics/paint/paint_controller.cc:141-143
    for chunk in &mut artifact.chunks {
        chunk.client_is_just_created = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_validates_only_used_rooted_clients_and_refreshes_next_export() {
        crate::native_test_thread::run(|| {
            let mut document = html::html_parser::ParseHTML(
                "<html><body><div id=a style='width:20px;height:20px;background:red'></div><div id=b style='width:20px;height:20px;background:blue'></div></body></html>",
            );
            dom::style_resolver::ResolveComputedStyles(&mut document, &Default::default(), &[]);
            let constraints = crate::CreateBrowserConstraints(100, 100);
            let interaction = dom::UserInteractionState::default();
            let mut layout = LayoutEngine::new(&crate::CreateLayoutAssembly());
            let mut fragments = Rc::new(crate::LayoutPersistentDocument(
                &mut layout,
                &mut document,
                &interaction,
                &constraints,
            ));
            let node_id = |name: &str| {
                (0..document.GetDocument().NodeCount())
                    .find_map(|index| {
                        let node = document.GetDocument().Node(index);
                        node.FindAttribute("id")
                            .filter(|attribute| attribute.value == name)
                            .map(|_| node.Id())
                    })
                    .unwrap()
            };
            fn optional(node: &FragmentNode, id: u64) -> Option<&FragmentNode> {
                if node.node_id == id {
                    return Some(node);
                }
                node.children.iter().find_map(|child| optional(child, id))
            }
            fn find(node: &FragmentNode, id: u64) -> &FragmentNode {
                optional(node, id).unwrap()
            }
            let a = node_id("a");
            let b = node_id("b");
            let client_id = find(&fragments, a).paint.display_item_client_id;
            assert_ne!(client_id, 0);
            assert!(
                find(&fragments, a)
                    .paint
                    .display_item_client_is_just_created
            );
            assert!(
                find(&fragments, b)
                    .paint
                    .display_item_client_is_just_created
            );
            let mut artifact = PaintArtifact::default();
            for id in [client_id, u64::MAX] {
                artifact.chunks.push(paint::paint_engine::PaintChunk {
                    id: paint::paint_engine::PaintChunkId {
                        client_id: id,
                        ..Default::default()
                    },
                    is_cacheable: true,
                    client_is_just_created: true,
                    ..Default::default()
                });
            }
            let caret = layoutng_assembly::caret::FrameCaret::default();
            CommitPaint(&mut layout, &caret, &mut fragments, &mut artifact);
            assert!(artifact
                .chunks
                .iter()
                .all(|chunk| !chunk.client_is_just_created));
            assert!(
                !find(&fragments, a)
                    .paint
                    .display_item_client_is_just_created
            );
            assert!(
                find(&fragments, b)
                    .paint
                    .display_item_client_is_just_created
            );
            assert!(caret.display_item_client().IsJustCreated());
            let shared = fragments.clone();
            artifact
                .display_items
                .push(paint::paint_engine::RecordedDisplayItem {
                    kind: paint::paint_engine::RecordedDisplayItemKind::Drawing,
                    scroll_translation: None,
                    id: paint::paint_engine::PaintChunkId {
                        client_id: caret.display_item_client().Id(),
                        ..Default::default()
                    },
                    visual_rect: Default::default(),
                    visual_rect_is_accurate: false,
                    draws_content: false,
                    raster_effect_outset: paint::paint_engine::RasterEffectOutset::kNone,
                    record_begin: 0,
                    record_end: 0,
                });
            CommitPaint(&mut layout, &caret, &mut fragments, &mut artifact);
            assert!(!caret.display_item_client().IsJustCreated());
            assert!(
                Rc::ptr_eq(&shared, &fragments),
                "already valid shared geometry stays shared"
            );
            let next = crate::LayoutPersistentDocument(
                &mut layout,
                &mut document,
                &interaction,
                &constraints,
            );
            assert_eq!(find(&next, a).paint.display_item_client_id, client_id);
            assert!(!find(&next, a).paint.display_item_client_is_just_created);
            assert!(find(&next, b).paint.display_item_client_is_just_created);
        });
    }
}
