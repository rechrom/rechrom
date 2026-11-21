//! Rust layout-output invalidation observer. Production revisions consume the
//! source LayoutObject dirty flags and add fragment-local geometry/resource
//! changes at the owned-output boundary. Every fragment is certified from those
//! dirty bits (no value-comparison fallback). The resulting
//! object/descendant distinction feeds PrePaintTreeWalk (upstream
//! core/paint/pre_paint_tree_walk.cc:418-436).

use std::sync::Arc;

use super::{FragmentKind, FragmentPaintData, PaintProperties, PaintResources};
use super::ResidentBoxGeometry as _;
use foundation::{
    TrustedIdentityHashMap as HashMap,
};
pub use layout_fragment_boundary::revision::PrePaintRevisionStats;

/// Version an in-place source scroll/sticky/control change on an already
/// certified layout output. The layout boundary must call this on every
/// changed node and its complete ancestor path. Zero remains unobserved;
/// this operation cannot certify external or previously unobserved inputs.
pub fn MarkScrollPropertyChange(node: &mut FragmentPaintData, own_changed: bool) {
    layout_fragment_boundary::revision::mark_scroll_property_change(node, own_changed);
}

#[derive(Default)]
pub struct PrePaintRevisionTracker {
    // Owner-token liveness moves between passes; each fragment's previous
    // certification lives on its persistent slot.
    owners: HashMap<u64, LiveOwner>,
    spare_owners: HashMap<u64, LiveOwner>,
    last_stats: PrePaintRevisionStats,
    production_update: Option<ProductionUpdate>,
    /// Advanced by invalidate(); a slot certification of another epoch is stale.
    epoch: u64,
}

struct ProductionUpdate {
    trace: layout_fragment_boundary::revision::RevisionUpdateTrace,
    generation: u64,
    previous_owners: HashMap<u64, LiveOwner>,
    next_owners: HashMap<u64, LiveOwner>,
    resource_matches: HashMap<(usize, usize, u8), bool>,
    /// The document paint resources the observed fragments paint with.
    document_resources: Option<Arc<PaintResources>>,
    stats: PrePaintRevisionStats,
}

struct LiveOwner {
    // DisplayItemClient::Id is a process-unique opaque token. Retaining the
    // token across two observations proves the same client lifetime; a new
    // client never reuses an earlier token even if its allocation address is
    // recycled.
    retained: bool,
}

impl PrePaintRevisionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn invalidate(&mut self) {
        self.owners.clear();
        self.spare_owners.clear();
        self.last_stats = PrePaintRevisionStats::default();
        self.production_update = None;
        self.epoch += 1;
    }

    /// Begin a production source walk. Layout calls ObserveProductionNode
    /// directly while following the canonical fragment cursor, so revision
    /// certification does not allocate a flat copy of the fragment topology.
    pub fn BeginProductionUpdate(&mut self) {
        assert!(self.production_update.is_none());
        let mut next_owners = std::mem::take(&mut self.spare_owners);
        debug_assert!(next_owners.is_empty());
        next_owners.reserve(self.owners.len());
        self.production_update = Some(ProductionUpdate {
            trace: layout_fragment_boundary::revision::update_span(),
            generation: layout_fragment_boundary::revision::next_revision(),
            previous_owners: std::mem::take(&mut self.owners),
            next_owners,
            resource_matches: HashMap::default(),
            document_resources: None,
            stats: PrePaintRevisionStats::default(),
        });
    }

    /// The document paint resources of the begun production walk (Chromium's
    /// Document / LocalFrameView state; every fragment shares them).
    pub fn SetProductionDocumentResources(&mut self, resources: Option<Arc<PaintResources>>) {
        self.production_update
            .as_mut()
            .expect("production revision update has not begun")
            .document_resources = resources;
    }

    pub fn ObserveProductionNode(
        &mut self,
        node: &mut FragmentPaintData,
        descendant_changed: bool,
    ) -> bool {
        assert_ne!(node.fragment_instance_id, 0);
        let update = self
            .production_update
            .as_mut()
            .expect("production revision update has not begun");
        // Production fragments carry their previous certification on their
        // persistent slot (Chromium keeps such state on FragmentData), so no
        // keyed map is moved between passes. This is `observe`'s production
        // branch: identity is the stable instance, geometry and paint are
        // proven by the source dirty bits.
        let epoch = self.epoch + 1;
        let generation = update.generation;
        let stats = &mut update.stats;
        let old = (node.certified.epoch == epoch).then(|| node.certified.clone());
        stats.nodes += 1;
        let invalidation = node.pre_paint_invalidation;
        stats.own_invalidations += invalidation.own() as usize;
        stats.descendant_invalidations += invalidation.descendants() as usize;
        stats.needs_paint_property_update += invalidation.needs_paint_property_update as usize;
        stats.descendant_needs_paint_property_update +=
            invalidation.descendant_needs_paint_property_update as usize;
        stats.should_check_for_paint_invalidation +=
            invalidation.should_check_for_paint_invalidation as usize;
        stats.should_check_layout_for_paint_invalidation +=
            invalidation.should_check_layout_for_paint_invalidation as usize;
        stats.descendant_should_check_layout_for_paint_invalidation +=
            invalidation.descendant_should_check_layout_for_paint_invalidation as usize;
        stats.subtree_should_check_for_paint_invalidation +=
            invalidation.subtree_should_check_for_paint_invalidation as usize;
        if old.is_none() {
            stats.new_nodes += 1;
        }
        let owners_live =
            retain_owners(node, &mut update.previous_owners, &mut update.next_owners);
        let current_owner_ids = owner_ids(node);
        node.pre_paint_owner_retained =
            owners_live && old.as_ref().is_some_and(|old| old.owner_ids == current_owner_ids);
        let (same_geometry, same_paint, same_resources) = old
            .as_ref()
            .map(|old| {
                let resources = resources_equal(
                    &old.resources,
                    &update.document_resources,
                    &mut update.resource_matches,
                    old.resource_usage | paint_resource_usage(node),
                );
                // Own geometry is proven by the object's dirty bits; its
                // position and size in the container are not (a parent's
                // relayout moves a cache-hit child without dirtying it).
                let geometry =
                    !invalidation.own() && old.offset == node.offset && old.size == node.size;
                (geometry, !invalidation.own(), resources)
            })
            .unwrap_or((false, false, false));
        if old.is_some() {
            stats.geometry_changes += (!same_geometry) as usize;
            stats.paint_changes += (!same_paint) as usize;
            stats.resource_changes += (!same_resources) as usize;
        }
        let marked_self = old.as_ref().is_some_and(|old| {
            node.pre_paint_revision != 0 && node.pre_paint_revision != old.revision
        });
        let marked_subtree = old.as_ref().is_some_and(|old| {
            node.pre_paint_subtree_revision != 0
                && node.pre_paint_subtree_revision != old.subtree_revision
        });
        let self_same = old.is_some()
            && !marked_self
            && same_geometry
            && same_paint
            && same_resources
            && (node.pre_paint_owner_retained || !just_created(node));
        let children_same = old.is_some()
            && !marked_subtree
            && !invalidation.descendants()
            && !descendant_changed;
        stats.child_changes += (!children_same) as usize;
        node.pre_paint_revision = if self_same {
            old.as_ref().unwrap().revision
        } else {
            generation
        };
        node.pre_paint_subtree_revision = if self_same && children_same {
            old.as_ref().unwrap().subtree_revision
        } else {
            generation
        };
        let (resources, resource_usage) = match old {
            Some(old) if self_same => (old.resources, old.resource_usage),
            _ => (update.document_resources.clone(), paint_resource_usage(node)),
        };
        node.certified = layout_fragment_boundary::CertifiedPrePaintInput {
            epoch,
            revision: node.pre_paint_revision,
            subtree_revision: node.pre_paint_subtree_revision,
            owner_ids: current_owner_ids,
            resources,
            resource_usage,
            offset: node.offset,
            size: node.size,
        };
        !self_same || !children_same
    }

    /// ObserveProductionNode for a slot population replayed unchanged (same
    /// immutable owner, clean dirty bits, same payload): when its previous
    /// certification still holds - same epoch, unmarked revisions, same
    /// geometry and resources - only the owner liveness bookkeeping and the
    /// stats change. Anything else takes the full observation.
    pub fn ReplayProductionNode(&mut self, node: &mut FragmentPaintData) -> bool {
        let epoch = self.epoch + 1;
        let update = self
            .production_update
            .as_mut()
            .expect("production revision update has not begun");
        let holds = node.certified.epoch == epoch
            && node.pre_paint_revision == node.certified.revision
            && node.pre_paint_subtree_revision == node.certified.subtree_revision
            && node.pre_paint_revision != 0
            && node.pre_paint_subtree_revision != 0
            && !node.pre_paint_invalidation.own()
            && !node.pre_paint_invalidation.descendants()
            && node.certified.offset == node.offset
            && node.certified.size == node.size
            && resources_equal(
                &node.certified.resources,
                &update.document_resources,
                &mut update.resource_matches,
                node.certified.resource_usage,
            );
        if !holds {
            return self.ObserveProductionNode(node, false);
        }
        // Owner liveness is idempotent per id within a pass, so a fallback
        // after this registration observes the same liveness.
        let owners_live =
            retain_owners(node, &mut update.previous_owners, &mut update.next_owners);
        let retained = owners_live && node.certified.owner_ids == owner_ids(node);
        if !retained && just_created(node) {
            return self.ObserveProductionNode(node, false);
        }
        update.stats.nodes += 1;
        node.pre_paint_owner_retained = retained;
        false
    }

    /// A selection commit changes only native invalidation bits. Certify the
    /// already indexed selected-client/ancestor sidecar in place; do not walk
    /// fragments again or rebuild geometry/resource/owner observations.
    pub(crate) fn ObserveSelectionInvalidation(
        &mut self,
        node: &mut FragmentPaintData,
        generation: u64,
    ) {
        assert!(self.production_update.is_none());
        // Production certifications live on the slot (ObserveProductionNode).
        if node.certified.epoch != self.epoch + 1 {
            return;
        }
        if node.pre_paint_invalidation.own() {
            node.pre_paint_revision = generation;
            node.certified.revision = generation;
        }
        node.pre_paint_subtree_revision = generation;
        node.certified.subtree_revision = generation;
    }

    pub fn FinishProductionUpdate(&mut self) {
        let mut update = self
            .production_update
            .take()
            .expect("production revision update has not begun");
        self.owners = update.next_owners;
        update.previous_owners.clear();
        self.spare_owners = update.previous_owners;
        self.last_stats = update.stats;
        update.trace.finish(update.stats);
    }

    pub fn last_stats(&self) -> PrePaintRevisionStats {
        self.last_stats
    }

}

const RESOURCE_VIEWPORT: u8 = 1 << 0;
const RESOURCE_FONTS: u8 = 1 << 1;
const RESOURCE_IMAGES: u8 = 1 << 2;

fn paint_resource_usage(fragment: &FragmentPaintData) -> u8 {
    let paint = &fragment.paint;
    // Only box fragments with a source paint nine-piece images (the former
    // ExportNinePieceImages admission).
    let nine_piece_images = fragment.kind == FragmentKind::kBox && paint.HasSource();
    // One style-source resolution; the fixed-attachment check is
    // ComputedStyle::HasFixedAttachmentBackgroundImage rather than a full
    // projection of every background layer.
    let (fixed_background, style_images) = paint.style.ResourceUsageFacts(nine_piece_images);
    // Viewport-relative painters, which also read the base background facts.
    let viewport = paint.IsLayoutView()
        || paint.canvas_background.is_some()
        || paint.is_page_container
        || fixed_background;
    let images = paint.replaced_is_image
        || paint.VideoFrameResourceId().is_some()
        || fragment
            .ViewReplacedContent()
            .is_some_and(|content| content.is_canvas)
        || style_images
        || fragment.ResidentBoxFragment().is_some_and(|box_fragment| box_fragment.IsTable())
        || paint.SvgShape().is_some()
        || fragment.ViewSvgText().is_some();
    u8::from(viewport) * RESOURCE_VIEWPORT
        | u8::from(!paint.glyph_runs.is_empty()) * RESOURCE_FONTS
        | u8::from(images) * RESOURCE_IMAGES
}

fn owner_ids(fragment: &FragmentPaintData) -> [u64; 7] {
    let paint = &fragment.paint;
    let mut ids = [
        paint.DisplayItemClientId(),
        paint.SelectionClientId(),
        paint.PaintLayerClientId(),
        0,
        0,
        0,
        0,
    ];
    if let Some(scroll) = fragment.ViewScrollContainer() {
        ids[3] = scroll.scrolling_background_client_id;
    }
    if let Some(bars) = fragment.ViewScrollbars() {
        ids[4] = bars.corner_client_id;
        ids[5] = bars
            .horizontal
            .as_ref()
            .map_or(0, |axis| axis.display_item_client_id);
        ids[6] = bars
            .vertical
            .as_ref()
            .map_or(0, |axis| axis.display_item_client_id);
    }
    ids
}

fn retain_owners(
    fragment: &FragmentPaintData,
    previous: &mut HashMap<u64, LiveOwner>,
    next: &mut HashMap<u64, LiveOwner>,
) -> bool {
    let mut observed = false;
    let mut retained = true;
    for id in owner_ids(fragment).into_iter().filter(|id| *id != 0) {
        observed = true;
        // Evaluate every owner even after a rejection so all clients are
        // registered for the next pass, once per id across every fragment.
        retained &= retain_owner(id, previous, next);
    }
    observed && retained
}

fn retain_owner(
    id: u64,
    previous: &mut HashMap<u64, LiveOwner>,
    next: &mut HashMap<u64, LiveOwner>,
) -> bool {
    if let Some(owner) = next.get(&id) {
        return owner.retained;
    }
    let retained = previous.remove(&id).is_some();
    next.insert(id, LiveOwner { retained });
    retained
}

fn just_created(fragment: &FragmentPaintData) -> bool {
    let paint = &fragment.paint;
    paint.DisplayItemClientState().is_just_created
        || paint.SelectionClientState().is_just_created
        || paint.PaintLayerClientState().is_just_created
        || fragment.ViewScrollContainer().is_some_and(|scroll| {
            paint
                .ScrollClientState(
                    scroll.scrolling_background_client_id,
                    scroll.scrolling_background_client_invalidation_reason,
                    scroll.scrolling_background_client_is_cacheable,
                    scroll.scrolling_background_client_is_just_created.get(),
                )
                .is_just_created
        })
        || fragment.ViewScrollbars().is_some_and(|bars| {
            paint
                .ScrollClientState(
                    bars.corner_client_id,
                    bars.corner_client_invalidation_reason,
                    bars.corner_client_is_cacheable,
                    bars.corner_client_is_just_created.get(),
                )
                .is_just_created
                || [&bars.horizontal, &bars.vertical]
                    .into_iter()
                    .flatten()
                    .any(|axis| {
                        paint
                            .ScrollClientState(
                                axis.display_item_client_id,
                                axis.display_item_client_invalidation_reason,
                                axis.display_item_client_is_cacheable,
                                axis.display_item_client_is_just_created.get(),
                            )
                            .is_just_created
                    })
        })
}

fn resources_equal(
    a: &Option<Arc<PaintResources>>,
    b: &Option<Arc<PaintResources>>,
    matches: &mut HashMap<(usize, usize, u8), bool>,
    usage: u8,
) -> bool {
    let (a, b) = match (a, b) {
        (None, None) => return true,
        (Some(a), Some(b)) => (a, b),
        _ => return false,
    };
    if Arc::ptr_eq(a, b) {
        return true;
    }
    let key = (Arc::as_ptr(a) as usize, Arc::as_ptr(b) as usize, usage);
    *matches.entry(key).or_insert_with(|| {
        // These are immutable resource-provider identities, not the catalog
        // allocation rebuilt by every export. New storage conservatively dirties.
        let PaintResources {
            fonts,
            images,
            canvases,
            device_pixel_ratio,
            preferred_color_scheme,
            layout_zoom_factor,
            base_background_color,
            paints_opaque_base_background,
            should_clear_document_background,
            viewport,
        } = &**a;
        *device_pixel_ratio == b.device_pixel_ratio
            && *preferred_color_scheme == b.preferred_color_scheme
            && *layout_zoom_factor == b.layout_zoom_factor
            // The base background (ViewPainter's opaque backdrop and clear)
            // is read only by the view, canvas-background and page-container
            // painters, which all carry RESOURCE_VIEWPORT. A document laid out
            // both as itself and as a document image alternates these facts;
            // that must not dirty every other fragment of the document.
            && (usage & RESOURCE_VIEWPORT == 0
                || (*viewport == b.viewport
                    && *base_background_color == b.base_background_color
                    && *paints_opaque_base_background == b.paints_opaque_base_background
                    && *should_clear_document_background
                        == b.should_clear_document_background))
            && (usage & RESOURCE_FONTS == 0
                || (fonts.len() == b.fonts.len()
                    && (fonts.ptr_eq(&b.fonts)
                        || fonts.iter().zip(b.fonts.iter()).all(|(a, b)| {
                            font_engine::SharedFontBytes::ptr_eq(&a.bytes, &b.bytes) && a == b
                        }))))
            && (usage & RESOURCE_IMAGES == 0
                || (images.len() == b.images.len()
                    && images.iter().zip(&b.images).all(|(a, b)| a == b)
                    && canvases.len() == b.canvases.len()
                    && canvases.iter().zip(&b.canvases).all(|(a, b)| a == b)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_owner_tokens_retain_just_created_clients() {
        let mut tracker = PrePaintRevisionTracker::new();
        let mut fragment = FragmentPaintData::default();
        fragment.fragment_instance_id = 1;
        fragment.paint.display_item_client_id = 11;

        tracker.BeginProductionUpdate();
        assert!(tracker.ObserveProductionNode(&mut fragment, false));
        tracker.FinishProductionUpdate();
        let first_revision = fragment.pre_paint_revision;

        tracker.BeginProductionUpdate();
        assert!(!tracker.ObserveProductionNode(&mut fragment, false));
        tracker.FinishProductionUpdate();

        assert!(fragment.pre_paint_owner_retained);
        assert_eq!(fragment.pre_paint_revision, first_revision);
    }
}
