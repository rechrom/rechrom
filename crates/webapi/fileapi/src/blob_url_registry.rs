//! Chromium storage/browser/blob/blob_url_registry.{h:100-166,cc:97-243};
//! blob_url_store_impl.cc:40-83,125-166,204-248. Immutable Blob backing is
//! retained by token leases even after revoke, exactly like the source remote.
use foundation::security::{OriginKey, StorageKey};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
#[derive(Debug)]
pub struct BlobData {
    pub bytes: Arc<[u8]>,
    pub content_type: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MappingStatus {
    IsMapped,
    CrossPartitionSameOriginFirstParty,
    CrossPartitionSameOriginThirdParty,
    Other,
}
struct BlobURLData {
    blob: Arc<BlobData>,
    key: StorageKey,
    origin: OriginKey,
}
#[derive(Default)]
struct RegistryState {
    urls: HashMap<String, BlobURLData>,
    tokens: HashMap<[u64; 2], (String, Arc<BlobData>)>,
}
#[derive(Default)]
pub struct BlobURLRegistry {
    state: Mutex<RegistryState>,
}
#[derive(Debug)]
pub enum BlobTokenResolution {
    Resolved,
    Disconnected,
}
/// Opaque browser navigation capability. The shared lease owns its existing
/// immutable Blob resource; cloning never reparses URL/origin or copies bytes.
pub struct BlobURLToken {
    id: Option<[u64; 2]>,
    registry: Arc<BlobURLRegistry>,
}
impl std::fmt::Debug for BlobURLToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlobURLToken")
            .field("id", &self.id)
            .finish()
    }
}
impl BlobURLToken {
    pub fn Id(&self) -> Option<[u64; 2]> {
        self.id
    }
    pub fn Resolve(&self) -> Option<(String, Arc<BlobData>)> {
        let registry = &self.registry;
        let state = registry.state.lock().expect("Blob URL registry");
        state.tokens.get(&self.id?).cloned()
    }
}
impl Drop for BlobURLToken {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.registry
                .state
                .lock()
                .expect("Blob URL registry")
                .tokens
                .remove(&id);
        }
    }
}
fn without_fragment(url: &str) -> &str {
    url.split_once('#').map_or(url, |(url, _)| url)
}
impl BlobURLRegistry {
    pub fn AddUrlMapping(
        &self,
        url: String,
        blob: Arc<BlobData>,
        key: StorageKey,
        origin: OriginKey,
    ) -> bool {
        assert!(!url.contains('#'));
        let mut state = self.state.lock().expect("Blob URL registry");
        if state.urls.get(&url).is_some_and(|data| data.key == key) {
            return false;
        }
        state.urls.insert(url, BlobURLData { blob, key, origin });
        true
    }
    pub fn RemoveUrlMapping(&self, url: &str, key: &StorageKey) -> bool {
        let mut state = self.state.lock().expect("Blob URL registry");
        if !state.urls.get(url).is_some_and(|data| &data.key == key) {
            return false;
        }
        state.urls.remove(url);
        true
    }
    pub fn OriginForURL(&self, url: &str) -> Option<OriginKey> {
        self.state
            .lock()
            .expect("Blob URL registry")
            .urls
            .get(without_fragment(url))
            .map(|data| data.origin.clone())
    }
    pub fn IsUrlMapped(&self, url: &str, key: &StorageKey) -> MappingStatus {
        let state = self.state.lock().expect("Blob URL registry");
        let Some(data) = state.urls.get(without_fragment(url)) else {
            return MappingStatus::Other;
        };
        if &data.key == key {
            MappingStatus::IsMapped
        } else if data.key.Origin() == key.Origin() {
            if data.key.IsFirstPartyContext() {
                MappingStatus::CrossPartitionSameOriginFirstParty
            } else {
                MappingStatus::CrossPartitionSameOriginThirdParty
            }
        } else {
            MappingStatus::Other
        }
    }
    pub fn GetBlobFromURL(&self, url: &str) -> Option<Arc<BlobData>> {
        self.state
            .lock()
            .expect("Blob URL registry")
            .urls
            .get(without_fragment(url))
            .map(|data| data.blob.clone())
    }
    pub fn ResolveAsBlobURLToken(
        self: &Arc<Self>,
        url: &str,
        key: &StorageKey,
        top_level: bool,
        partitioning_enabled: bool,
        disabled_by_policy: bool,
        storage_access: bool,
        token_id: [u64; 2],
    ) -> (Arc<BlobURLToken>, Option<bool>) {
        let mapping = self.IsUrlMapped(url, key);
        let mut partition_notice = None;
        if !top_level
            && matches!(
                mapping,
                MappingStatus::CrossPartitionSameOriginFirstParty
                    | MappingStatus::CrossPartitionSameOriginThirdParty
            )
        {
            let blocked = partitioning_enabled
                && !disabled_by_policy
                && !(storage_access
                    && mapping == MappingStatus::CrossPartitionSameOriginFirstParty);
            partition_notice = Some(blocked);
            if blocked {
                return (
                    Arc::new(BlobURLToken {
                        id: None,
                        registry: self.clone(),
                    }),
                    partition_notice,
                );
            }
        }
        let mut state = self.state.lock().expect("Blob URL registry");
        let blob = state
            .urls
            .get(without_fragment(url))
            .map(|data| data.blob.clone());
        let id = blob.map(|blob| {
            assert_ne!(token_id, [0, 0]);
            assert!(!state.tokens.contains_key(&token_id));
            state.tokens.insert(token_id, (url.into(), blob));
            token_id
        });
        (
            Arc::new(BlobURLToken {
                id,
                registry: self.clone(),
            }),
            partition_notice,
        )
    }
}
