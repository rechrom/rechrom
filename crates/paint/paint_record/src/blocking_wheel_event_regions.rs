//! Read-only input projection of the committed PaintChunk hit-test records.
//!
//! ObjectPainter records these rectangles after PrePaint propagated blocking
//! handler state. Mapping uses the same retained property owners as painting;
//! it never reads Layout or reconstructs listener descendant geometry.
//! Source: paint_chunks_to_cc_layer.cc:1386-1392 (UpdateWheelEventRegion),
//! chunk_to_layer_mapper.cc:25-68,70-99 (property mapping and clipping).

use paint_model::{
    geometry_mapper::{LocalToAncestorClipRect, SourceToDestinationRect},
    paint_property_tree::PropertyTreeState,
    PaintRect,
};

use crate::PaintRecording;

impl PaintRecording {
    /// Borrow the committed blocking-wheel rectangles in CSS client space.
    /// The boolean records attachment to the root scrolling transform, not the
    /// positioning of the listener target. `None` means viewport property input
    /// is unavailable; an empty iterator is coherent, empty hit-test geometry.
    pub fn CommittedBlockingWheelEventRegions(
        &self,
    ) -> Option<impl Iterator<Item = (PaintRect, bool)> + '_> {
        let viewport = self.compositor_viewport.as_ref()?;
        let root = PropertyTreeState::Root();
        let zoom = self
            .resources
            .as_ref()
            .map_or(1.0, |r| r.layout_zoom_factor);
        Some(self.chunks.iter().flat_map(move |chunk| {
            let mut transform = Some(chunk.properties.transform.as_ref());
            let mut root_scrolling = false;
            while let Some(node) = transform {
                if viewport
                    .layout_scroll_translation
                    .as_ref()
                    .is_some_and(|root| root.lifecycle.same_node(&node.lifecycle))
                {
                    root_scrolling = true;
                    break;
                }
                transform = node.parent.as_deref();
            }
            // Chromium maps hit-test regions into the scrolling layer before
            // applying its viewport clip. Keep that stationary outer clip out
            // of a region which the compositor will translate on impl scroll;
            // otherwise an offscreen committed region could never scroll in.
            let clip_ancestor = if root_scrolling {
                PropertyTreeState {
                    clip: viewport
                        .layout_scroll_translation
                        .as_ref()
                        .and_then(|node| node.scroll.as_ref())
                        .and_then(|scroll| scroll.overflow_clip.clone())
                        .unwrap_or_else(|| viewport.viewport_clip.clone()),
                    ..root.clone()
                }
            } else {
                root.clone()
            };
            let clip = LocalToAncestorClipRect(&chunk.properties, &clip_ancestor);
            let destination = root.transform.clone();
            chunk
                .compositor
                .as_deref()
                .and_then(|metadata| metadata.hit_test_data.as_deref())
                .into_iter()
                .flat_map(|data| data.wheel_event_rects.iter())
                .map(move |rect| {
                    // A singular projection has no hittable area, matching the
                    // GeometryMapper clip branch. Nonfinite values remain
                    // visible to the input adapter's conservative fallback.
                    let mut rect =
                        SourceToDestinationRect(&chunk.properties.transform, &destination, *rect)
                            .unwrap_or_default();
                    if let Some(clip) = clip {
                        rect = rect.intersection(clip);
                    }
                    rect.x /= zoom;
                    rect.y /= zoom;
                    rect.width /= zoom;
                    rect.height /= zoom;
                    (rect, root_scrolling)
                })
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use layoutng_assembly::fragment_tree::PaintResources;
    use paint_chunk::PaintChunk;
    use paint_model::{
        geometry_mapper::TranslationTransform, paint_property_tree::PaintPropertyNodeLifecycle,
        PaintCompositorViewport, PaintCompositorViewportMode,
    };

    #[test]
    fn committed_regions_preserve_native_mapping_clips_and_root_attachment() {
        let root = PropertyTreeState::Root();
        let mut viewport_clip = root.clip.as_ref().clone();
        viewport_clip.lifecycle = PaintPropertyNodeLifecycle::default();
        viewport_clip.parent = Some(root.clip.clone());
        viewport_clip.rect = Some(PaintRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        });
        let viewport_clip = Arc::new(viewport_clip.with_layout_clips());
        let mut scroll = root.transform.as_ref().clone();
        scroll.lifecycle = PaintPropertyNodeLifecycle::default();
        scroll.parent = Some(root.transform.clone());
        scroll.matrix = TranslationTransform(0.0, -20.0);
        let mut scroll_node = scroll.scroll.as_ref().unwrap().as_ref().clone();
        scroll_node.overflow_clip = Some(viewport_clip.clone());
        scroll.scroll = Some(Arc::new(scroll_node));
        let scroll = Arc::new(scroll);
        let mut clip = root.clip.as_ref().clone();
        clip.lifecycle = PaintPropertyNodeLifecycle::default();
        clip.parent = Some(viewport_clip.clone());
        clip.local_transform_space = scroll.clone();
        clip.rect = Some(PaintRect {
            x: 0.0,
            y: 0.0,
            width: 20.0,
            height: 20.0,
        });
        let clip = Arc::new(clip.with_layout_clips());
        let mut scrolling = PaintChunk::default();
        scrolling.properties.transform = scroll.clone();
        scrolling.properties.clip = clip;
        scrolling
            .EnsureHitTestData()
            .wheel_event_rects
            .push(PaintRect {
                x: 10.0,
                y: 10.0,
                width: 30.0,
                height: 30.0,
            });
        let mut fixed = PaintChunk::default();
        fixed.properties.clip = viewport_clip.clone();
        fixed.EnsureHitTestData().wheel_event_rects.push(PaintRect {
            x: 2.0,
            y: 4.0,
            width: 6.0,
            height: 8.0,
        });
        let artifact = PaintRecording {
            compositor_viewport: Some(PaintCompositorViewport {
                mode: PaintCompositorViewportMode::SingleLayoutViewport,
                viewport_clip,
                overscroll_elasticity_transform: root.transform.clone(),
                page_scale: root.transform.clone(),
                visual_scroll_translation: root.transform.clone(),
                layout_scroll_translation: Some(scroll),
                scroll_translation_nodes: vec![],
            }),
            chunks: vec![scrolling, fixed].into(),
            resources: Some(Arc::new(PaintResources {
                layout_zoom_factor: 2.0,
                ..Default::default()
            })),
            ..Default::default()
        };
        let regions: Vec<_> = artifact
            .CommittedBlockingWheelEventRegions()
            .unwrap()
            .collect();
        assert_eq!(
            regions,
            vec![
                (
                    PaintRect {
                        x: 5.0,
                        y: -5.0,
                        width: 5.0,
                        height: 5.0
                    },
                    true
                ),
                (
                    PaintRect {
                        x: 1.0,
                        y: 2.0,
                        width: 3.0,
                        height: 4.0
                    },
                    false
                ),
            ]
        );
        assert!(PaintRecording::default()
            .CommittedBlockingWheelEventRegions()
            .is_none());
    }
}
