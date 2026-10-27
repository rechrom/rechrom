//! Imported client-resource return and reclamation policy.
//!
//! The embedding owns concrete transferable resources and callbacks. This
//! module owns the export counts, returned-token merge, delayed deletion and
//! shutdown reclamation decisions that determine resource lifetime.

use crate::{resource_pool_boundary::ResourceSyncToken, ResourceId};
use std::collections::HashMap;

pub trait ClientResourceImport<T: ResourceSyncToken> {
    fn ExportedCount(&self) -> i32;
    fn SetExportedCount(&mut self, count: i32);
    fn ReturnedLost(&self) -> bool;
    fn SetReturnedLost(&mut self, lost: bool);
    fn MarkedForDeletion(&self) -> bool;
    fn SetReturnedSyncToken(&mut self, token: T);
}

pub trait ReturnedClientResource<T: ResourceSyncToken> {
    fn Id(&self) -> ResourceId;
    fn Count(&self) -> i32;
    fn Lost(&self) -> bool;
    fn SyncToken(&self) -> &T;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClientResourceReturnStats {
    pub returned: usize,
    pub unknown: usize,
    pub still_exported: usize,
    pub sync_tokens_updated: usize,
    pub released: usize,
    pub lost: usize,
}

pub fn ApplyReturnedClientResources<I, T, R>(
    imports: &mut HashMap<ResourceId, I>,
    resources: impl IntoIterator<Item = R>,
) -> (Vec<I>, ClientResourceReturnStats)
where
    I: ClientResourceImport<T>,
    T: ResourceSyncToken,
    R: ReturnedClientResource<T>,
{
    let mut trace =
        browser_tracing::span("compositor.resource", "Compositor.Resource.ReceiveReturns");
    let mut releases = Vec::new();
    let mut stats = ClientResourceReturnStats::default();

    for returned in resources {
        stats.returned += 1;
        let id = returned.Id();
        let release = {
            let Some(imported) = imports.get_mut(&id) else {
                stats.unknown += 1;
                continue;
            };
            debug_assert!(imported.ExportedCount() >= returned.Count());
            assert!(returned.Count() > 0, "returned count must be positive");
            imported.SetExportedCount(imported.ExportedCount() - returned.Count());
            imported.SetReturnedLost(imported.ReturnedLost() || returned.Lost());
            if returned.Lost() {
                stats.lost += 1;
            }
            if imported.ExportedCount() != 0 {
                stats.still_exported += 1;
                false
            } else {
                if returned.SyncToken().HasData() {
                    imported.SetReturnedSyncToken(returned.SyncToken().clone());
                    stats.sync_tokens_updated += 1;
                }
                imported.MarkedForDeletion()
            }
        };
        if release {
            releases.push(imports.remove(&id).expect("returned import still resident"));
            stats.released += 1;
        }
    }

    trace.set("returned", stats.returned as f64);
    trace.set("unknown", stats.unknown as f64);
    trace.set("still_exported", stats.still_exported as f64);
    trace.set("sync_tokens_updated", stats.sync_tokens_updated as f64);
    trace.set("released", stats.released as f64);
    trace.set("lost", stats.lost as f64);
    (releases, stats)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClientResourceReleaseAllStats {
    pub resident: usize,
    pub exported: usize,
    pub released: usize,
    pub marked_lost: usize,
}

pub fn ReleaseAllExportedClientResources<I, T>(
    imports: &mut HashMap<ResourceId, I>,
    lose: bool,
) -> (Vec<I>, ClientResourceReleaseAllStats)
where
    I: ClientResourceImport<T>,
    T: ResourceSyncToken,
{
    let mut trace = browser_tracing::span(
        "compositor.resource",
        "Compositor.Resource.ReleaseAllExported",
    );
    let mut stats = ClientResourceReleaseAllStats {
        resident: imports.len(),
        ..Default::default()
    };
    let releases = imports
        .extract_if(|_, imported| {
            if imported.ExportedCount() == 0 {
                return false;
            }
            stats.exported += 1;
            imported.SetExportedCount(0);
            imported.SetReturnedLost(imported.ReturnedLost() || lose);
            if lose {
                stats.marked_lost += 1;
            }
            let release = imported.MarkedForDeletion();
            if release {
                stats.released += 1;
            }
            release
        })
        .map(|(_, imported)| imported)
        .collect();

    trace.set("resident", stats.resident as f64);
    trace.set("exported", stats.exported as f64);
    trace.set("released", stats.released as f64);
    trace.set("marked_lost", stats.marked_lost as f64);
    (releases, stats)
}

pub fn DrainClientResourcesForShutdown<I, T>(imports: &mut HashMap<ResourceId, I>) -> Vec<I>
where
    I: ClientResourceImport<T>,
    T: ResourceSyncToken,
{
    let mut trace = browser_tracing::span(
        "compositor.resource",
        "Compositor.Resource.ShutdownReclamation",
    );
    let releases: Vec<_> = imports
        .drain()
        .map(|(_, mut imported)| {
            debug_assert!(imported.MarkedForDeletion());
            debug_assert!(imported.ExportedCount() != 0);
            imported.SetReturnedLost(true);
            imported
        })
        .collect();
    trace.set("released", releases.len() as f64);
    releases
}

pub trait ExportableClientResource {
    type Transferable: Clone;

    fn Transferable(&self) -> &Self::Transferable;
    fn ExportTokenHasData(&self) -> bool;
    fn ExportTokenIsVerified(&self) -> bool;
    fn IncrementExportedCount(&mut self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClientResourceExportStats {
    pub requested: usize,
    pub synchronized: usize,
    pub exported: usize,
}

pub fn PrepareClientResourcesForExport<I: ExportableClientResource>(
    imports: &mut HashMap<ResourceId, I>,
    resources: &[ResourceId],
    output: &mut Vec<I::Transferable>,
    has_shared_image_interface: bool,
) -> ClientResourceExportStats {
    let mut trace =
        browser_tracing::span("compositor.resource", "Compositor.Resource.PrepareExports");
    let mut stats = ClientResourceExportStats {
        requested: resources.len(),
        ..Default::default()
    };
    output.reserve(resources.len());
    for id in resources {
        let imported = imports
            .get_mut(id)
            .expect("source imported export resource");
        if imported.ExportTokenHasData() {
            stats.synchronized += 1;
        }
        if has_shared_image_interface {
            assert!(
                !imported.ExportTokenHasData() || imported.ExportTokenIsVerified(),
                "image completion token must be verified by its native producer"
            );
        } else {
            debug_assert!(!imported.ExportTokenIsVerified());
        }
        output.push(imported.Transferable().clone());
        imported.IncrementExportedCount();
        stats.exported += 1;
    }
    trace.set("requested", stats.requested as f64);
    trace.set("synchronized", stats.synchronized as f64);
    trace.set("exported", stats.exported as f64);
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    struct Token(u64);
    impl ResourceSyncToken for Token {
        fn HasData(&self) -> bool {
            self.0 != 0
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Import {
        exports: i32,
        lost: bool,
        delete: bool,
        token: Token,
    }
    impl ClientResourceImport<Token> for Import {
        fn ExportedCount(&self) -> i32 {
            self.exports
        }
        fn SetExportedCount(&mut self, count: i32) {
            self.exports = count;
        }
        fn ReturnedLost(&self) -> bool {
            self.lost
        }
        fn SetReturnedLost(&mut self, lost: bool) {
            self.lost = lost;
        }
        fn MarkedForDeletion(&self) -> bool {
            self.delete
        }
        fn SetReturnedSyncToken(&mut self, token: Token) {
            self.token = token;
        }
    }

    struct Returned {
        id: u32,
        count: i32,
        lost: bool,
        token: Token,
    }
    impl ReturnedClientResource<Token> for Returned {
        fn Id(&self) -> ResourceId {
            ResourceId(self.id)
        }
        fn Count(&self) -> i32 {
            self.count
        }
        fn Lost(&self) -> bool {
            self.lost
        }
        fn SyncToken(&self) -> &Token {
            &self.token
        }
    }

    fn imported(exports: i32, delete: bool) -> Import {
        Import {
            exports,
            lost: false,
            delete,
            token: Token(1),
        }
    }

    #[test]
    fn returns_release_only_deleted_imports_after_last_export() {
        let mut imports = HashMap::from([
            (ResourceId(1), imported(2, true)),
            (ResourceId(2), imported(1, false)),
        ]);
        let (released, stats) = ApplyReturnedClientResources(
            &mut imports,
            [
                Returned {
                    id: 1,
                    count: 1,
                    lost: false,
                    token: Token(2),
                },
                Returned {
                    id: 1,
                    count: 1,
                    lost: true,
                    token: Token(3),
                },
                Returned {
                    id: 2,
                    count: 1,
                    lost: false,
                    token: Token(4),
                },
                Returned {
                    id: 99,
                    count: 1,
                    lost: false,
                    token: Token(5),
                },
            ],
        );

        assert_eq!(released.len(), 1);
        assert_eq!(released[0].token, Token(3));
        assert!(released[0].lost);
        assert_eq!(imports[&ResourceId(2)].exports, 0);
        assert_eq!(imports[&ResourceId(2)].token, Token(4));
        assert_eq!(stats.returned, 4);
        assert_eq!(stats.unknown, 1);
        assert_eq!(stats.still_exported, 1);
        assert_eq!(stats.released, 1);
    }

    #[test]
    fn release_all_keeps_live_imports_and_reclaims_deleted_ones() {
        let mut imports = HashMap::from([
            (ResourceId(1), imported(3, true)),
            (ResourceId(2), imported(2, false)),
            (ResourceId(3), imported(0, true)),
        ]);
        let (released, stats) = ReleaseAllExportedClientResources::<_, Token>(&mut imports, true);

        assert_eq!(released.len(), 1);
        assert!(released[0].lost);
        assert!(!imports.contains_key(&ResourceId(1)));
        assert_eq!(imports[&ResourceId(2)].exports, 0);
        assert!(imports[&ResourceId(2)].lost);
        assert_eq!(imports[&ResourceId(3)].exports, 0);
        assert_eq!(stats.resident, 3);
        assert_eq!(stats.exported, 2);
        assert_eq!(stats.released, 1);
    }

    #[test]
    fn shutdown_drains_and_marks_all_remaining_resources_lost() {
        let mut imports = HashMap::from([
            (ResourceId(1), imported(1, true)),
            (ResourceId(2), imported(2, true)),
        ]);
        let released = DrainClientResourcesForShutdown::<_, Token>(&mut imports);
        assert!(imports.is_empty());
        assert_eq!(released.len(), 2);
        assert!(released.iter().all(|entry| entry.lost));
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Transferable(u32);
    struct Exportable {
        value: Transferable,
        has_token: bool,
        verified: bool,
        exports: usize,
    }
    impl ExportableClientResource for Exportable {
        type Transferable = Transferable;
        fn Transferable(&self) -> &Transferable {
            &self.value
        }
        fn ExportTokenHasData(&self) -> bool {
            self.has_token
        }
        fn ExportTokenIsVerified(&self) -> bool {
            self.verified
        }
        fn IncrementExportedCount(&mut self) {
            self.exports += 1;
        }
    }

    #[test]
    fn export_preserves_requested_order_duplicates_and_counts() {
        let mut imports = HashMap::from([
            (
                ResourceId(1),
                Exportable {
                    value: Transferable(10),
                    has_token: true,
                    verified: true,
                    exports: 0,
                },
            ),
            (
                ResourceId(2),
                Exportable {
                    value: Transferable(20),
                    has_token: false,
                    verified: false,
                    exports: 0,
                },
            ),
        ]);
        let mut output = vec![Transferable(5)];
        let stats = PrepareClientResourcesForExport(
            &mut imports,
            &[ResourceId(2), ResourceId(1), ResourceId(1)],
            &mut output,
            true,
        );
        assert_eq!(
            output,
            [
                Transferable(5),
                Transferable(20),
                Transferable(10),
                Transferable(10)
            ]
        );
        assert_eq!(imports[&ResourceId(1)].exports, 2);
        assert_eq!(imports[&ResourceId(2)].exports, 1);
        assert_eq!(stats.requested, 3);
        assert_eq!(stats.synchronized, 2);
        assert_eq!(stats.exported, 3);
    }
}
