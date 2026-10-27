//! Native CPU import/export bookkeeping for the external Viz resource provider.
//! The allocation is shared only between its provider wrapper and ResourcePool
//! platform capability. Callback targets are borrowed per call, never retained.
use super::{
    host_owners::{NativeClientResourceProvider, NativePoolReleaseClient},
    resources::{RasterReturnedResource, RasterSyncToken, RasterTransferableResource},
};
use crate::{
    adapter::{
        layer_tree_host_impl::{
            HostSharedImageInterfaceId, HostUIResourceProvider, HostUIResourceReleaseClient,
            HostUIResourceReleaseTask,
        },
        layer_tree_impl::ResourceId,
        resource_pool::{
            PoolReleaseClient, ResourcePoolId, ResourcePoolReleaseCallback, ResourceSyncToken,
        },
    },
    cc::resources::resource_pool::ResourcePool,
    NativeCompositorTypes,
};
use compositor_resource::client_resource_lifecycle::{
    ApplyReturnedClientResources, ClientResourceImport, DrainClientResourcesForShutdown,
    ExportableClientResource, PrepareClientResourcesForExport, ReleaseAllExportedClientResources,
    ReturnedClientResource,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

// Source dependency: components/viz/client/client_resource_provider.cc:127-184.
// TransferableResource is a source value copy retaining the same native image.
enum Release {
    Ui(HostUIResourceReleaseTask),
    Pool(ResourcePoolReleaseCallback),
}
struct Imported {
    resource: RasterTransferableResource,
    release: Release,
    exported_count: i32,
    marked_for_deletion: bool,
    returned_sync_token: RasterSyncToken,
    returned_lost: bool,
}
impl ClientResourceImport<RasterSyncToken> for Imported {
    fn ExportedCount(&self) -> i32 {
        self.exported_count
    }
    fn SetExportedCount(&mut self, count: i32) {
        self.exported_count = count;
    }
    fn ReturnedLost(&self) -> bool {
        self.returned_lost
    }
    fn SetReturnedLost(&mut self, lost: bool) {
        self.returned_lost = lost;
    }
    fn MarkedForDeletion(&self) -> bool {
        self.marked_for_deletion
    }
    fn SetReturnedSyncToken(&mut self, token: RasterSyncToken) {
        self.returned_sync_token = token;
    }
}
impl ExportableClientResource for Imported {
    type Transferable = RasterTransferableResource;

    fn Transferable(&self) -> &Self::Transferable {
        &self.resource
    }
    fn ExportTokenHasData(&self) -> bool {
        self.resource.sync_token.HasData()
    }
    fn ExportTokenIsVerified(&self) -> bool {
        self.resource.sync_token.verified_flush
    }
    fn IncrementExportedCount(&mut self) {
        self.exported_count += 1;
    }
}
impl ReturnedClientResource<RasterSyncToken> for RasterReturnedResource {
    fn Id(&self) -> ResourceId {
        self.id
    }
    fn Count(&self) -> i32 {
        self.count
    }
    fn Lost(&self) -> bool {
        self.lost
    }
    fn SyncToken(&self) -> &RasterSyncToken {
        &self.sync_token
    }
}
struct ProviderState {
    release_capabilities: Option<ReleaseCapabilities>,
    imports: HashMap<ResourceId, Imported>,
    next_id: u32,
    visible: bool,
    evicted: bool,
    batch_release_callbacks: bool,
}
// Source dependency: components/viz/client/client_resource_provider.cc:205-214.
// These are the original provider's runner/callback leases, shared by its
// clones alongside the one resident import table, never another provider.
struct ReleaseCapabilities {
    flush: crate::adapter::task_runner::RepeatingClosure,
    main: Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>,
    impl_runner: Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>,
    threaded_release_callbacks_supported: bool,
}
#[derive(Clone)]
pub(crate) struct RasterClientResources(Arc<Mutex<ProviderState>>);
impl RasterClientResources {
    pub(crate) fn new() -> Self {
        // Source dependency: client_resource_provider.h:171-182, resource_id.h:32-46.
        Self(Arc::new(Mutex::new(ProviderState {
            release_capabilities: None,
            imports: HashMap::new(),
            next_id: 1,
            visible: false,
            evicted: false,
            batch_release_callbacks: false,
        })))
    }
    pub(crate) fn ConfigureReleaseCapabilities(
        &self,
        main_id: crate::adapter::layer_tree_host_impl::HostImplTaskRunnerId,
        impl_id: crate::adapter::layer_tree_host_impl::HostImplTaskRunnerId,
        main: Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>,
        impl_runner: Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>,
        flush: crate::adapter::task_runner::RepeatingClosure,
    ) {
        let mut state = self.0.lock().expect("native client resource provider");
        assert!(
            state.release_capabilities.is_none(),
            "configure actual provider once"
        );
        state.release_capabilities = Some(ReleaseCapabilities {
            flush,
            main,
            impl_runner,
            threaded_release_callbacks_supported: main_id != impl_id,
        });
    }
    // Source dependency: components/viz/client/client_resource_provider.cc:436-440.
    // This checks the same resident import table used for export/return; callers
    // never receive a copied resource set or a validation snapshot.
    pub(crate) fn ValidateResource(&self, id: ResourceId) {
        #[cfg(debug_assertions)]
        {
            debug_assert_ne!(id, ResourceId(0));
            debug_assert!(
                self.0
                    .lock()
                    .expect("native client resource provider")
                    .imports
                    .contains_key(&id),
                "resource must belong to the resident client provider"
            );
        }
        #[cfg(not(debug_assertions))]
        let _ = id;
    }
    // Source dependency: client_resource_provider.cc:343-359.
    fn Import(&self, mut resource: RasterTransferableResource, release: Release) -> ResourceId {
        let mut state = self.0.lock().expect("native client resource provider");
        let id = ResourceId(state.next_id);
        state.next_id = state
            .next_id
            .checked_add(1)
            .expect("native resource ID exhausted");
        resource.id = id;
        let returned_sync_token = resource.sync_token;
        assert!(state
            .imports
            .insert(
                id,
                Imported {
                    resource,
                    release,
                    exported_count: 0,
                    marked_for_deletion: false,
                    returned_sync_token,
                    returned_lost: false
                }
            )
            .is_none());
        id
    }
    pub(crate) fn ImportPoolResource(
        &self,
        resource: RasterTransferableResource,
        callback: ResourcePoolReleaseCallback,
    ) -> ResourceId {
        self.Import(resource, Release::Pool(callback))
    }
    // Source dependency: client_resource_provider.cc:361-374.
    fn Remove(&self, id: ResourceId) -> Option<Imported> {
        let mut state = self.0.lock().expect("native client resource provider");
        let imported = state
            .imports
            .get_mut(&id)
            .expect("source imported resource");
        imported.marked_for_deletion = true;
        if imported.exported_count == 0 {
            state.imports.remove(&id)
        } else {
            None
        }
    }
    pub(crate) fn RemovePoolResource(
        &self,
        owner: ResourcePoolId,
        id: ResourceId,
        client: &mut dyn PoolReleaseClient<NativeCompositorTypes>,
    ) {
        let Some(imported) = self.Remove(id) else {
            return;
        };
        let Release::Pool(callback) = imported.release else {
            panic!("UI resource removed through pool capability")
        };
        if !callback.owner.IsValid() {
            return;
        }
        assert_eq!(
            callback.owner.id, owner,
            "resource released through another pool"
        );
        client.OnResourceReleased(
            callback.unique_id,
            &imported.returned_sync_token,
            imported.returned_lost,
        );
    }
    fn RunReleases(
        releases: Vec<Imported>,
        mut pool: Option<&mut dyn NativePoolReleaseClient>,
        ui: &mut dyn HostUIResourceReleaseClient<NativeCompositorTypes>,
    ) {
        // Source dependency: client_resource_provider.cc:330-341. The source
        // erases all returned entries before invoking impl release callbacks.
        for imported in releases {
            match imported.release {
                Release::Ui(callback) => callback.Run::<NativeCompositorTypes>(
                    &imported.returned_sync_token,
                    imported.returned_lost,
                    ui,
                ),
                Release::Pool(callback) => {
                    if !callback.owner.IsValid() {
                        continue;
                    }
                    pool.as_deref_mut()
                        .expect("live pool callback requires the resident pool")
                        .OnPoolResourceReleased(
                            callback,
                            &imported.returned_sync_token,
                            imported.returned_lost,
                        );
                }
            }
        }
    }
}
impl HostUIResourceProvider<NativeCompositorTypes> for RasterClientResources {
    fn ImportResource(
        &mut self,
        resource: RasterTransferableResource,
        release: HostUIResourceReleaseTask,
    ) -> ResourceId {
        self.Import(resource, Release::Ui(release))
    }
    fn RemoveImportedResource(
        &mut self,
        id: ResourceId,
        client: &mut dyn HostUIResourceReleaseClient<NativeCompositorTypes>,
    ) {
        if let Some(imported) = self.Remove(id) {
            assert!(
                matches!(imported.release, Release::Ui(_)),
                "pool resource removed through UI capability"
            );
            Self::RunReleases(vec![imported], None, client);
        }
    }
}
impl NativeClientResourceProvider<NativeCompositorTypes> for RasterClientResources {
    fn CreateScopedBatchResourcesRelease(
        &mut self,
        enabled: bool,
    ) -> Box<dyn crate::adapter::client_layer_tree_host_impl::ClientBatchResourceRelease> {
        self.0
            .lock()
            .expect("native client resources")
            .batch_release_callbacks = enabled;
        Box::new(NativeBatchResourceRelease(self.clone()))
    }

    fn SetVisible(&mut self, visible: bool) {
        self.0
            .lock()
            .expect("native client resource provider")
            .visible = visible;
        // Source cc:479-515: this API registers no eviction callback; therefore
        // HandleEviction has no callbacks to invoke. Import state remains real.
    }
    fn SetEvicted(&mut self, evicted: bool) {
        self.0
            .lock()
            .expect("native client resource provider")
            .evicted = evicted;
    }
    fn ShutdownAndReleaseAllResources(
        &mut self,
        client: &mut dyn HostUIResourceReleaseClient<NativeCompositorTypes>,
    ) {
        // Source dependency: client_resource_provider.cc:409-435.
        let releases = {
            let mut state = self.0.lock().expect("native client resource provider");
            DrainClientResourcesForShutdown(&mut state.imports)
        };
        // Host destruction has already destroyed ResourcePool. Its weak-bound
        // release tasks expire; still-live UI callbacks use this call's client.
        Self::RunReleases(releases, None, client);
    }
    fn ReceiveReturnsFromParent(
        &mut self,
        resources: Vec<RasterReturnedResource>,
        pool: Option<&mut dyn NativePoolReleaseClient>,
        client: &mut dyn HostUIResourceReleaseClient<NativeCompositorTypes>,
    ) {
        // Source dependency: client_resource_provider.cc:275-341.
        let releases = {
            let mut state = self.0.lock().expect("native client resource provider");
            ApplyReturnedClientResources(&mut state.imports, resources).0
        };
        Self::RunReleases(releases, pool, client);
    }
    fn ReleaseAllExportedResources(
        &mut self,
        pool: Option<&mut dyn NativePoolReleaseClient>,
        lose: bool,
        client: &mut dyn HostUIResourceReleaseClient<NativeCompositorTypes>,
    ) {
        // Source dependency: client_resource_provider.cc:376-407. Mutating
        // retain visits the resident entries once, without ID relookup/snapshot.
        let releases = {
            let mut state = self.0.lock().expect("native client resource provider");
            ReleaseAllExportedClientResources(&mut state.imports, lose).0
        };
        Self::RunReleases(releases, pool, client);
    }
    fn PrepareSendToParent(
        &mut self,
        resources: &[ResourceId],
        output: &mut Vec<RasterTransferableResource>,
        interface: HostSharedImageInterfaceId,
    ) {
        // Source dependency: client_resource_provider.cc:230-273. CPU tokens
        // come from a real in-process image completion capability, not a GPU
        // stream; verification checks the existing verified token value.
        let mut state = self.0.lock().expect("native client resource provider");
        PrepareClientResourcesForExport(&mut state.imports, resources, output, interface.0 != 0);
    }
}

pub(crate) struct ResidentPoolRelease<'a>(pub(crate) &'a mut ResourcePool<NativeCompositorTypes>);
impl NativePoolReleaseClient for ResidentPoolRelease<'_> {
    fn OnPoolResourceReleased(
        &mut self,
        callback: ResourcePoolReleaseCallback,
        token: &RasterSyncToken,
        lost: bool,
    ) {
        self.0.RunReleaseCallback(callback, token, lost);
    }
}

// Source: components/viz/client/client_resource_provider.cc:466-477,560-566.
// Imports in this CPU provider register only impl release callbacks, matching
// its ImportResource call sites. There are no main-thread release callbacks to
// flush; the same provider's batch flag still resets at source scope exit.
struct NativeBatchResourceRelease(RasterClientResources);
impl crate::adapter::client_layer_tree_host_impl::ClientBatchResourceRelease
    for NativeBatchResourceRelease
{
}
impl Drop for NativeBatchResourceRelease {
    fn drop(&mut self) {
        self.0
             .0
            .lock()
            .expect("native client resources")
            .batch_release_callbacks = false;
    }
}
