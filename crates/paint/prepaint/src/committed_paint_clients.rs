//! Validation of the live DisplayItemClients a committed paint used.
//!
//! Source: platform/graphics/paint/paint_controller.cc:127-143. When the
//! PaintController cycle ends, Chromium validates the clients collected in
//! `clients_to_validate_` (MarkClientForValidation, paint_controller.cc:
//! 528-533; paint_chunker.cc:22-28) through their own pointers; it never
//! walks the LayoutObject tree, and clients the new paint did not use keep
//! their invalidation state. Rechrom's display items name clients by a
//! process-unique id; PrePaint resolves each used id on the walked fragment
//! occurrences whose payloads carry it.

#![allow(non_snake_case)]

use std::collections::HashSet;

use layout::fragment::ResidentBoxGeometry as _;

use crate::pre_paint_tree_walk::PrePaintTreeWalk;

impl PrePaintTreeWalk {
    /// Validate the live clients of `clients` that the committed walk's
    /// occurrences carry, and return the ids none of them carries (clients
    /// outside the fragment tree, such as the caret clients, or a client of
    /// an occurrence no walk reached). The Layout commit validates the
    /// returned ids against its source owners.
    pub fn ValidateLiveClientsForCommittedPaint(&self, clients: &HashSet<u64>) -> HashSet<u64> {
        let mut trace = browser_tracing::span("prepaint", "PrePaint.ValidateCommittedClients");
        trace.set("used_clients", clients.len() as f64);
        let Some(root) = self.RetainedRoot().filter(|_| !clients.is_empty()) else {
            trace.set("unresolved", clients.len() as f64);
            return clients.clone();
        };
        let mut unresolved = clients.clone();
        self.VisitWalkedOccurrences(root, &mut |cursor, _| {
            if unresolved.is_empty() {
                return;
            }
            let fragment = cursor.PaintData();
            let mut ids = vec![
                fragment.paint.DisplayItemClientId(),
                fragment.paint.SelectionClientId(),
                fragment.paint.PaintLayerClientId(),
            ];
            if let Some((id, _)) = fragment.ViewScrollingBackgroundClient() {
                ids.push(id);
            }
            if let Some(bars) = fragment.ViewScrollbarClients() {
                ids.extend(
                    [bars.horizontal, bars.vertical, bars.corner]
                        .into_iter()
                        .flatten()
                        .map(|(id, _)| id),
                );
            }
            for id in ids {
                if id != 0
                    && unresolved.contains(&id)
                    && fragment.paint.ValidateLiveClientForCommittedPaint(id)
                {
                    unresolved.remove(&id);
                }
            }
        });
        trace.set("unresolved", unresolved.len() as f64);
        unresolved
    }
}
