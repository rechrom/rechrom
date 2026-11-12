//! Immutable structured-serialization storage. Chromium fixed revision
//! bindings/core/v8/serialization/serialized_script_value.{h,cc};
//! v8/src/objects/value-serializer.cc. This is a versioned engine-neutral
//! storage representation, not QuickJS bytecode or a V8 wire-format claim.
use std::sync::Arc;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SerializedScriptValue {
    bytes: Arc<[u8]>,
}
impl SerializedScriptValue {
    pub fn Bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Retain immutable storage when session history crosses document threads.
    pub fn StorageBytes(&self) -> Arc<[u8]> {
        self.bytes.clone()
    }
    /// Restore immutable session-storage bytes. Deserialization validates the
    /// version and full graph; no realm object is attached to this owner.
    pub fn FromStorageBytes(bytes: Arc<[u8]>) -> Self {
        Self { bytes }
    }
    pub(crate) fn FromBytes(bytes: Vec<u8>) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }
}

/// An original author exception remains an owned engine value in the current
/// invocation. Callers rethrow it in that realm; storage never retains it.
pub enum JavaScriptSerializationError {
    DataCloneError(String),
    Exception {
        value: crate::javascript_runtime::HostValue,
        exception: crate::javascript_runtime::JavaScriptException,
    },
}

impl std::fmt::Debug for JavaScriptSerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DataCloneError(message) => {
                f.debug_tuple("DataCloneError").field(message).finish()
            }
            Self::Exception { exception, .. } => f
                .debug_struct("Exception")
                .field("metadata", exception)
                .finish_non_exhaustive(),
        }
    }
}
