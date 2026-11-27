//! Validation of the live DisplayItemClients a committed paint used.
//!
//! Source: platform/graphics/paint/paint_controller.cc:127-143. When the
//! PaintController cycle ends, Chromium validates the clients collected in
//! `clients_to_validate_` (MarkClientForValidation, paint_controller.cc:
//! 528-533; paint_chunker.cc:22-28) through their own pointers; it never
//! walks the LayoutObject tree, and clients the new paint did not use keep
//! their invalidation state. Rechrom's display items name clients by a
//! process-unique id, so the resident PrePaint client directory (the same
//! directory `CommitClients` and the direct transform updates use) resolves
//! each used id to the payloads whose live clients carry it.

#![allow(non_snake_case)]

use std::collections::HashSet;

use crate::pre_paint_tree_walk::PrePaintTreeWalk;

impl PrePaintTreeWalk {
    /// Validate the live clients of `clients` that the resident client
    /// directory resolves, and return the ids it could not resolve (clients
    /// the directory does not index, such as the caret clients, or a client
    /// created after the directory was last rebuilt). The Layout commit
    /// validates the returned ids against its source owners.
    pub fn ValidateLiveClientsForCommittedPaint(&self, clients: &HashSet<u64>) -> HashSet<u64> {
        let mut trace = browser_tracing::span("prepaint", "PrePaint.ValidateCommittedClients");
        trace.set("used_clients", clients.len() as f64);
        if self.retained_tree.is_none() {
            trace.set("unresolved", clients.len() as f64);
            return clients.clone();
        }
        let mut unresolved = HashSet::new();
        for &id in clients {
            let mut validated = false;
            if let Some(nodes) = self.client_nodes.get(&id) {
                for &pointer in nodes {
                    // The directory is owned by the retained tree; its boxes
                    // did not move since RetainTree rebuilt it.
                    let node = unsafe { &*pointer };
                    if let Some(fragment) = node.fragment.as_deref() {
                        validated |= fragment.paint.ValidateLiveClientForCommittedPaint(id);
                    }
                }
            }
            if !validated {
                unresolved.insert(id);
            }
        }
        trace.set("unresolved", unresolved.len() as f64);
        unresolved
    }
}
