//! Validation of the live DisplayItemClients a committed paint used.
//!
//! Source: platform/graphics/paint/paint_controller.cc:127-143. When the
//! PaintController cycle ends, Chromium validates the clients collected in
//! `clients_to_validate_` (MarkClientForValidation, paint_controller.cc:
//! 528-533; paint_chunker.cc:22-28) through their own pointers; it never
//! walks the LayoutObject tree, and clients the new paint did not use keep
//! their invalidation state. Rechrom's display items name clients by a
//! process-unique id, so PrePaint keeps a directory from each id to the
//! fragment occurrence that last carried it. The walk updates the directory
//! only for the occurrences it visits, and commit resolves each used id
//! through it: O(used clients), no tree walk.
//!
//! Lifetime: an entry names the occurrence PrePaint last visited for the id.
//! A new fragment is always visited (its walk record names another walked
//! owner, and LayoutNG replaces every ancestor fragment of a relaid-out
//! box), so an entry that was not refreshed names a fragment the current
//! result still reaches through unchanged ancestors, or an occurrence that
//! no longer paints (whose id the paint does not use). The entry's token
//! dies with the occurrence's FragmentData, so the directory never names a
//! destroyed occurrence and drops it without dereferencing it. A different
//! Paint publication state (fresh FragmentData) clears the directory.

#![allow(non_snake_case)]

use std::collections::HashSet;
use std::sync::{Arc, Weak};

use foundation::TrustedIdentityHashMap as HashMap;
use layout::fragment::ResidentBoxGeometry as _;
use layout::paint::{FragmentPaintData, PaintFragmentCursor};

use crate::fragment_data::PrePaintFragmentData;
use crate::pre_paint_tree_walk::PrePaintTreeWalk;

/// The most DisplayItemClients one occurrence carries: its LayoutObject, its
/// LayoutText selection client, its PaintLayer, its scrolling background and
/// its horizontal scrollbar, vertical scrollbar and scroll corner.
const MAX_CARRIED_CLIENTS: usize = 7;

/// The client ids one occurrence carried at its last visit and the liveness
/// token its directory entries hold weakly. Lives on the occurrence's
/// PrePaint FragmentData, so the token is dropped with that FragmentData.
#[derive(Default)]
pub struct CarriedClients {
    token: Option<Arc<()>>,
    ids: [u64; MAX_CARRIED_CLIENTS],
    len: u8,
}

impl CarriedClients {
    fn Token(&mut self) -> Weak<()> {
        Arc::downgrade(self.token.get_or_insert_with(|| Arc::new(())))
    }

    fn Ids(&self) -> &[u64] {
        &self.ids[..self.len as usize]
    }

    fn SetIds(&mut self, ids: &[u64]) {
        self.ids[..ids.len()].copy_from_slice(ids);
        self.len = ids.len() as u8;
    }
}

struct CommittedClientOccurrence {
    /// The walk's cursor of the occurrence (its display-item fragment scope
    /// included); resident while the lease of the walk that visited it, or
    /// of a later walk over a result that reuses its fragment, is held.
    cursor: PaintFragmentCursor<'static>,
    alive: Weak<()>,
}

/// DisplayItemClient id -> the occurrence the PrePaint walk last visited
/// that carries it.
#[derive(Default)]
pub(crate) struct CommittedClientDirectory {
    occurrences: HashMap<u64, CommittedClientOccurrence>,
    /// Dead entries are swept once the directory has doubled since the last
    /// sweep (amortized O(1) per registration).
    prune_at: usize,
}

impl CommittedClientDirectory {
    /// The resident occurrence that carries `id`, if its FragmentData lives.
    fn Live(&self, id: u64) -> Option<PaintFragmentCursor<'static>> {
        self.occurrences
            .get(&id)
            .filter(|entry| entry.alive.strong_count() > 0)
            .map(|entry| entry.cursor)
    }

    /// A different Paint publication state: no FragmentData of the previous
    /// one is reachable.
    pub(crate) fn Clear(&mut self) {
        self.occurrences.clear();
        self.prune_at = 0;
    }

    /// Drop the entries whose occurrence's FragmentData is gone.
    pub(crate) fn Prune(&mut self) {
        if self.occurrences.len() < self.prune_at {
            return;
        }
        self.occurrences
            .retain(|_, entry| entry.alive.strong_count() > 0);
        self.prune_at = (self.occurrences.len() * 2).max(1024);
    }
}

fn PushClientId(ids: &mut [u64; MAX_CARRIED_CLIENTS], len: &mut usize, id: u64) {
    if id != 0 && *len < MAX_CARRIED_CLIENTS && !ids[..*len].contains(&id) {
        ids[*len] = id;
        *len += 1;
    }
}

/// Every DisplayItemClient id `fragment`'s occurrence carries (the clients
/// PaintProperties::ValidateLiveClientForCommittedPaint and the detached
/// client commit reach on its payload).
fn CarriedClientIds(fragment: &FragmentPaintData) -> ([u64; MAX_CARRIED_CLIENTS], usize) {
    let mut ids = [0u64; MAX_CARRIED_CLIENTS];
    let mut len = 0;
    PushClientId(&mut ids, &mut len, fragment.paint.DisplayItemClientId());
    PushClientId(&mut ids, &mut len, fragment.paint.SelectionClientId());
    PushClientId(&mut ids, &mut len, fragment.paint.PaintLayerClientId());
    if let Some((id, _)) = fragment.ViewScrollingBackgroundClient() {
        PushClientId(&mut ids, &mut len, id);
    }
    if let Some(bars) = fragment.ViewScrollbarClients() {
        for (id, _) in [bars.horizontal, bars.vertical, bars.corner]
            .into_iter()
            .flatten()
        {
            PushClientId(&mut ids, &mut len, id);
        }
    }
    (ids, len)
}

impl PrePaintTreeWalk {
    /// Record the visited occurrence `cursor` as the one carrying each of its
    /// client ids, and drop the entries it held for ids it no longer carries.
    pub(crate) fn RegisterCommittedClients(
        &mut self,
        cursor: PaintFragmentCursor<'_>,
        fragment: &FragmentPaintData,
        data: &mut PrePaintFragmentData,
    ) {
        let (ids, len) = CarriedClientIds(fragment);
        let ids = &ids[..len];
        let carried = data.CarriedClientsMut();
        // The occurrence's last registration still stands when it carries
        // the same ids and every entry still names this cursor with its
        // token (no other occurrence took an id, no Clear).
        if carried.Ids() == ids {
            if let Some(token) = carried.token.as_ref() {
                let token = Arc::as_ptr(token);
                let directory = &self.committed_clients;
                if ids.iter().all(|id| {
                    directory.occurrences.get(id).is_some_and(|entry| {
                        std::ptr::eq(entry.alive.as_ptr(), token) && entry.cursor.IsSameCursor(cursor)
                    })
                }) {
                    return;
                }
            }
        }
        let token = carried.Token();
        // SAFETY: the walk's lease keeps the fragment resident; the entry is
        // replaced by the visit of any new fragment of this occurrence and
        // dropped with its FragmentData (see the module comment).
        let cursor = unsafe {
            std::mem::transmute::<PaintFragmentCursor<'_>, PaintFragmentCursor<'static>>(cursor)
        };
        let directory = &mut self.committed_clients;
        for old in carried.Ids() {
            if !ids.contains(old)
                && directory
                    .occurrences
                    .get(old)
                    .is_some_and(|entry| Weak::ptr_eq(&entry.alive, &token))
            {
                directory.occurrences.remove(old);
            }
        }
        for &id in ids {
            directory.occurrences.insert(
                id,
                CommittedClientOccurrence {
                    cursor,
                    alive: token.clone(),
                },
            );
        }
        carried.SetIds(ids);
    }

    /// Validate the live clients of `clients` that the directory resolves,
    /// and return the ids it cannot resolve (clients outside the fragment
    /// tree, such as the caret clients, an occurrence no walk reached, or a
    /// payload without a live client). The Layout commit validates the
    /// returned ids against its source owners.
    pub fn ValidateLiveClientsForCommittedPaint(&self, clients: &HashSet<u64>) -> HashSet<u64> {
        let mut trace = browser_tracing::span("prepaint", "PrePaint.ValidateCommittedClients");
        trace.set("used_clients", clients.len() as f64);
        let mut unresolved = HashSet::new();
        for &id in clients {
            let validated = self.committed_clients.Live(id).is_some_and(|cursor| {
                cursor
                    .PaintDataOption()
                    .is_some_and(|fragment| fragment.paint.ValidateLiveClientForCommittedPaint(id))
            });
            if !validated {
                unresolved.insert(id);
            }
        }
        trace.set("unresolved", unresolved.len() as f64);
        unresolved
    }

    /// The occurrence the directory resolves for a committed client id.
    pub(crate) fn CommittedClientOccurrence(&self, id: u64) -> Option<PaintFragmentCursor<'static>> {
        self.committed_clients.Live(id)
    }
}
