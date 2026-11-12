//! Browser-side resource services scoped to one StoragePartition.
//! Source storage_partition_impl.{h:278,825;cc:1604-1607,1949-1951} owns BlobUrlRegistry;
//! renderer PublicURLManagers borrow it and manage only their own registrations.
use std::sync::Arc;
#[derive(Clone)]
pub struct BrowserStoragePartitionServices {
    pub(crate) network_permissions: Arc<BrowserNetworkPermissionSettings>,
    pub(crate) blob_urls: Arc<fileapi::blob_url_registry::BlobURLRegistry>,
}
impl BrowserStoragePartitionServices {
    /// The BrowserHost constructs one value and shares it explicitly with its
    /// independent Pages. This contains no Page, tree or document-side state.
    pub fn new() -> Self {
        Self {
            network_permissions: Arc::new(BrowserNetworkPermissionSettings::default()),
            blob_urls: Arc::new(fileapi::blob_url_registry::BlobURLRegistry::default()),
        }
    }
    /// Legacy/headless admissions intentionally own isolated storage domains.
    pub fn Isolated() -> Self {
        Self::new()
    }
}
impl Default for BrowserStoragePartitionServices {
    fn default() -> Self {
        Self::Isolated()
    }
}

/// PermissionContextBase.cc:599-606 and content_settings_registry.cc:805-825
/// read the BrowserContext settings map for
/// the requesting/embedding origin pair. This product admits exact-origin
/// permission records; absent user decisions retain the source ASK default.
#[derive(Default)]
pub(crate) struct BrowserNetworkPermissionSettings {
    entries: std::sync::RwLock<
        std::collections::HashMap<
            (
                foundation::security::OriginKey,
                foundation::security::OriginKey,
                u8,
            ),
            document_loader::RawLocalNetworkPermissionStatus,
        >,
    >,
}
impl BrowserNetworkPermissionSettings {
    pub(crate) fn Get(
        &self,
        requesting: &foundation::security::OriginKey,
        embedding: &foundation::security::OriginKey,
        permission: document_loader::RawLocalNetworkPermission,
    ) -> document_loader::RawLocalNetworkPermissionStatus {
        self.entries
            .read()
            .expect("Browser permission settings")
            .get(&(requesting.clone(), embedding.clone(), permission as u8))
            .copied()
            .unwrap_or(document_loader::RawLocalNetworkPermissionStatus::Ask)
    }
    pub(crate) fn Set(
        &self,
        requesting: foundation::security::OriginKey,
        embedding: foundation::security::OriginKey,
        permission: document_loader::RawLocalNetworkPermission,
        status: document_loader::RawLocalNetworkPermissionStatus,
    ) {
        self.entries
            .write()
            .expect("Browser permission settings")
            .insert((requesting, embedding, permission as u8), status);
    }
}
