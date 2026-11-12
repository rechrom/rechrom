//! Source core/paint/cull_rect_updater.cc:232-277,437-541,667-697.
//! A synchronous scope over the retained FragmentData owners. Only the cull
//! values changed by special painting are saved; no source/tree/style copy.
use crate::cull_rect_updater::{CullRectUpdater, ShouldUseInfiniteCullRect};
use crate::pre_paint_tree_walk::{PaintTreeNode, PrePaintTreeWalk};
use layoutng_assembly::internal::layout_input::Position;
use paint_model::{
    paint_property_tree::{ClipPaintPropertyNode, PropertyTreeState},
    PaintRect,
};
use std::sync::Arc;

struct OriginalCullRects {
    node: *mut PaintTreeNode<'static>,
    self_cull: Option<PaintRect>,
    contents_cull: Option<PaintRect>,
}
#[derive(Clone)]
struct ContainerCull {
    state: PropertyTreeState,
    cull: Option<PaintRect>,
    subtree_infinite: bool,
    fragmented: bool,
}
#[derive(Clone)]
struct Context {
    current: ContainerCull,
    absolute: ContainerCull,
    fixed: ContainerCull,
}

struct OverriddenCullRectScope {
    original: Vec<OriginalCullRects>,
}
impl Drop for OverriddenCullRectScope {
    fn drop(&mut self) {
        for saved in self.original.iter().rev() {
            // WithSpecialPaint borrows PrePaint exclusively. Its resident boxes
            // and source lease remain alive until the scope restores all values.
            let node = unsafe { &mut *saved.node };
            node.self_cull_rect = saved.self_cull;
            node.cull_rect = saved.contents_cull;
        }
    }
}
impl OverriddenCullRectScope {
    fn Override(
        &mut self,
        node: &mut PaintTreeNode<'static>,
        context: &Context,
        root: &PropertyTreeState,
        view_cull: Option<PaintRect>,
        view_contents_clip: &Arc<ClipPaintPropertyNode>,
        starting: bool,
    ) {
        let fragment = node.fragment.as_deref().expect("retained FragmentData");
        if fragment.paint.IsFragmentLessBox() || fragment.paint.svg_hidden_container {
            return;
        }
        // UpdateRecursively, source.cc:305-310. Containing-block contexts
        // remain distinct from the logical/stacking traversal parent.
        let mut context = context.clone();
        if !starting {
            match fragment.paint.position {
                Position::kAbsolute => context.current = context.absolute.clone(),
                Position::kFixed => context.current = context.fixed.clone(),
                _ => {}
            }
        }
        let facts = node.cull_decision_inputs;
        let infinite = ShouldUseInfiniteCullRect(
            fragment,
            context.current.subtree_infinite,
            facts.has_filter_that_moves_pixels,
            facts.has_perspective,
            facts.has_active_transform_animation,
            facts.has_non_composited_transform,
            node.previous_paint_result.get(),
        );
        let self_cull = if infinite.use_infinite {
            None
        } else if starting {
            context.current.cull
        } else {
            // source.cc:417-421,497-509: a fragmented containing layer has
            // no single parent FragmentData; map an infinite cull from root.
            let (mut input, mut parent) = if context.current.fragmented {
                (None, root)
            } else {
                (context.current.cull, &context.current.state)
            };
            let mut local_state = node.properties.nodes.clone();
            // source.cc:454-486: a viewport-fixed paint-offset translation
            // uses the viewport clip, ignoring movable non-container clips.
            if fragment.paint.position == Position::kFixed
                && node
                    .paint_offset_translation
                    .as_ref()
                    .and_then(|translation| translation.parent.as_ref())
                    .is_some_and(|parent| Arc::ptr_eq(parent, &root.transform))
            {
                local_state.clip = view_contents_clip.clone();
                parent = root;
                input = view_cull;
            }

            CullRectUpdater::ComputeFragmentCullRect(input, None, root, parent, &local_state, 0.0).0
        };
        let contents = if infinite.use_infinite {
            None
        } else {
            CullRectUpdater::ComputeFragmentContentsCullRect(
                self_cull,
                None,
                root,
                &node.properties.nodes,
                &node.contents_properties.nodes,
                0.0,
            )
            .0
        };
        if node.self_cull_rect != self_cull || node.cull_rect != contents {
            self.original.push(OriginalCullRects {
                node: node as *mut _,
                self_cull: node.self_cull_rect,
                contents_cull: node.cull_rect,
            });
            node.self_cull_rect = self_cull;
            node.cull_rect = contents;
        }
        // source.cc:349-361,378-382. Starting layer can contain every
        // descendant for this override, independent of ordinary CSS ownership.
        context.current = ContainerCull {
            state: node.contents_properties.nodes.clone(),
            cull: contents,
            subtree_infinite: infinite.subtree_should_use_infinite,
            fragmented: node.paint_layer_is_fragmented,
        };
        if starting || node.can_contain_absolute_position_objects {
            context.absolute = context.current.clone();
        }
        if starting || node.can_contain_fixed_position_objects {
            context.fixed = context.current.clone();
        }
        if node.child_paint_blocked_by_display_lock {
            return;
        }
        for child in &mut node.children {
            self.Override(child, &context, root, view_cull, view_contents_clip, false);
        }
    }
}
impl PrePaintTreeWalk {
    /// One source PaintLayer's first FragmentData, through the existing client
    /// directory. Zero requests the retained LocalFrameView paint root.
    /// `input_rect` is in this layer's LocalBorderBoxProperties transform space.
    /// The callback cannot retain a borrowed tree node beyond this scope.
    pub fn WithSpecialPaint<T>(
        &mut self,
        layer_client_id: u64,
        mut input_rect: PaintRect,
        offset_input_by_paint_offset: bool,
        query: impl for<'n> FnOnce(&'n PaintTreeNode<'static>, &'n PaintTreeNode<'static>) -> T,
    ) -> Option<T> {
        let root = self.retained_tree.as_deref_mut()? as *mut PaintTreeNode<'static>;
        let layer = if layer_client_id == 0 {
            root
        } else {
            *self
                .client_nodes
                .get(&layer_client_id)?
                .iter()
                .find(|&&pointer| {
                    let node = unsafe { &*pointer };
                    node.is_paint_layer
                        && node.fragment.as_deref().is_some_and(|fragment| {
                            fragment.paint.paint_layer_client_id == layer_client_id
                        })
                })?
        };
        let root_state = self.viewport_properties_for_special_paint();
        let view_cull = self.viewport_cull_for_special_paint();
        let view_contents_clip = self.viewport_contents_clip_for_special_paint();
        let mut scope = OverriddenCullRectScope {
            original: Vec::new(),
        };
        let node = unsafe { &mut *layer };
        if offset_input_by_paint_offset {
            input_rect.x += node.paint_offset.x;
            input_rect.y += node.paint_offset.y;
        }
        // Source gfx::ToEnclosingRect, including gfx's clamped int geometry.
        let rect = foundation::gfx::ToEnclosingRect(foundation::gfx::RectF::new(
            foundation::gfx::PointF::new(input_rect.x as f32, input_rect.y as f32),
            foundation::gfx::SizeF::new(input_rect.width as f32, input_rect.height as f32),
        ));
        let input_rect = PaintRect {
            x: rect.x() as f64,
            y: rect.y() as f64,
            width: rect.width() as f64,
            height: rect.height() as f64,
        };

        let current = ContainerCull {
            state: root_state.clone(),
            cull: Some(input_rect),
            subtree_infinite: false,
            fragmented: false,
        };
        let context = Context {
            current: current.clone(),
            absolute: current.clone(),
            fixed: current,
        };
        scope.Override(
            node,
            &context,
            &root_state,
            if layer_client_id == 0 {
                Some(input_rect)
            } else {
                view_cull
            },
            &view_contents_clip,
            true,
        );
        // Both references are read-only while the guard has overridden culls.
        // Restoring starts only after the callback and its temporary borrows end.
        let result = query(unsafe { &*layer }, unsafe { &*root });
        drop(scope);
        Some(result)
    }
}
