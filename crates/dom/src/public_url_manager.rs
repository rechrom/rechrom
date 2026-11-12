//! Source core/fileapi/public_url_manager.{h:57-115,cc:156-252};
//! platform/blob/blob_url.cc:43-65, blob_url_null_origin_map.cc:42-66.
use crate::persistent_document::SecurityOrigin;
use fileapi::blob_url_registry::{BlobData, BlobURLRegistry, BlobURLToken};
use foundation::security::{OriginKey, StorageKey};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    rc::Rc,
    sync::Arc,
};
pub struct PublicURLManager {
    origin: Rc<SecurityOrigin>,
    key: RefCell<StorageKey>,
    registry: Arc<BlobURLRegistry>,
    urls: RefCell<HashSet<String>>,
    stopped: Cell<bool>,
    storage_access: Cell<bool>,
    partitioning_disabled: Cell<bool>,
}
impl PublicURLManager {
    pub(crate) fn new(
        origin: Rc<SecurityOrigin>,
        key: StorageKey,
        registry: Arc<BlobURLRegistry>,
    ) -> Rc<Self> {
        Rc::new(Self {
            origin,
            key: RefCell::new(key),
            registry,
            urls: RefCell::new(HashSet::new()),
            stopped: Cell::new(false),
            storage_access: Cell::new(false),
            partitioning_disabled: Cell::new(false),
        })
    }
    pub(crate) fn ReadyForStoragePartitionCommit(&self) -> bool {
        self.urls.borrow().is_empty()
    }
    pub(crate) fn Registry(&self) -> Arc<BlobURLRegistry> {
        self.registry.clone()
    }
    pub fn SetPartitioningDisabled(&self, value: bool) {
        self.partitioning_disabled.set(value);
    }
    pub fn PartitioningDisabled(&self) -> bool {
        self.partitioning_disabled.get()
    }
    pub fn HasStorageAccess(&self) -> bool {
        self.storage_access.get()
    }
    pub fn SetStorageAccess(&self, value: bool) {
        self.storage_access.set(value);
    }
    pub fn StorageKey(&self) -> std::cell::Ref<'_, StorageKey> {
        self.key.borrow()
    }
    /// The loader's admitted StorageKey, not a key inferred from the current URL.
    pub fn SetStorageKey(&self, key: StorageKey) {
        assert_eq!(key.Origin(), &self.origin.Key());
        assert!(
            self.urls.borrow().is_empty(),
            "StorageKey commit precedes PublicURL registration"
        );
        *self.key.borrow_mut() = key;
    }
    pub fn RegisterURL(&self, uuid: &str, blob: Arc<BlobData>) -> String {
        if self.stopped.get() {
            return String::new();
        }
        // UUID bytes are supplied by the original cryptographic UUID producer.
        assert!(
            uuid.len() == 36
                && uuid
                    .bytes()
                    .enumerate()
                    .all(|(i, c)| if matches!(i, 8 | 13 | 18 | 23) {
                        c == b'-'
                    } else {
                        c.is_ascii_hexdigit()
                    })
        );
        let url = format!("blob:{}/{}", self.origin.ToString(), uuid);
        self.registry.AddUrlMapping(
            url.clone(),
            blob,
            self.key.borrow().clone(),
            self.origin.Key(),
        );
        self.urls.borrow_mut().insert(url.clone());
        url
    }
    pub fn Revoke(&self, url: &str) {
        if self.stopped.get() || !url.starts_with("blob:") || url.contains('#') {
            return;
        }
        // A registered null-origin URL retains the original opaque identity.
        // Normal blob URL origins use SecurityOrigin::Create(url), per source.
        let target = self
            .registry
            .OriginForURL(url)
            .or_else(|| self.origin.URLKey(url));
        if target.as_ref() != Some(&self.origin.Key()) {
            return;
        }
        self.registry.RemoveUrlMapping(url, &self.key.borrow());
        self.urls.borrow_mut().remove(url);
    }
    pub fn OriginForURL(&self, url: &str) -> Option<OriginKey> {
        self.registry.OriginForURL(url)
    }
    pub fn Read(&self, url: &str) -> Option<Arc<BlobData>> {
        if self.stopped.get() {
            return None;
        }
        let mapping = self.registry.IsUrlMapped(url, &self.key.borrow());
        use fileapi::blob_url_registry::MappingStatus;
        if matches!(
            mapping,
            MappingStatus::CrossPartitionSameOriginFirstParty
                | MappingStatus::CrossPartitionSameOriginThirdParty
        ) && !self.partitioning_disabled.get()
            && !(self.storage_access.get()
                && mapping == MappingStatus::CrossPartitionSameOriginFirstParty)
        {
            return None;
        }
        self.registry.GetBlobFromURL(url)
    }
    pub fn ResolveAsBlobURLToken(
        &self,
        url: &str,
        top_level: bool,
        partitioning: bool,
        disabled: bool,
        storage_access: bool,
        token_id: [u64; 2],
    ) -> (Arc<BlobURLToken>, Option<bool>) {
        self.registry.ResolveAsBlobURLToken(
            url,
            &self.key.borrow(),
            top_level,
            partitioning,
            disabled,
            storage_access,
            token_id,
        )
    }
    pub fn ContextDestroyed(&self) {
        if self.stopped.replace(true) {
            return;
        }
        for url in self.urls.borrow_mut().drain() {
            self.registry.RemoveUrlMapping(&url, &self.key.borrow());
        }
    }
}
impl Drop for PublicURLManager {
    fn drop(&mut self) {
        self.ContextDestroyed();
    }
}

impl PublicURLManager {
    /// A new Window receives the cloned StorageKey/partition and access facts,
    /// never the old Window's registered-URL set or stopped lifetime.
    pub(crate) fn CopyForNavigation(&self, origin: Rc<SecurityOrigin>) -> Rc<Self> {
        let result = Self::new(origin, self.key.borrow().clone(), self.registry.clone());
        result.storage_access.set(self.storage_access.get());
        result
            .partitioning_disabled
            .set(self.partitioning_disabled.get());
        result
    }
}
