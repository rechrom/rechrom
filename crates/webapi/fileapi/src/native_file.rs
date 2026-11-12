//! Lazy native file backing. Interaction retains identity/metadata; only the
//! embedding reader performs I/O and completes immutable bytes outside compute.
use std::sync::{Arc, Mutex, OnceLock};
type BytesCompletion = Box<dyn FnOnce(Result<Arc<[u8]>, NativeFileError>) + Send>;
#[derive(Default)]
struct ReadState {
    in_flight: bool,
    waiters: Vec<BytesCompletion>,
}
#[derive(Clone, Debug)]
pub struct NativeFileMetadata {
    pub length: usize,
    pub last_modified: i64,
    /// Original snapshot precision; the WebIDL projection above is milliseconds.
    pub modification_time_microseconds: Option<i64>,
    pub content_type: String,
}
#[derive(Clone, Debug)]
pub enum NativeFileError {
    Unavailable,
    MetadataPending,
    Changed,
    Io(String),
}
impl std::fmt::Display for NativeFileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable => formatter.write_str("native file is unavailable"),
            Self::MetadataPending => formatter.write_str("native file metadata is pending"),
            Self::Changed => formatter.write_str("native file changed since selection"),
            Self::Io(message) => formatter.write_str(message),
        }
    }
}
impl std::error::Error for NativeFileError {}

pub struct NativeFileReadResult {
    pub metadata: NativeFileMetadata,
    pub bytes: Arc<[u8]>,
}
pub type NativeFileReadCompletion =
    Box<dyn FnOnce(Result<NativeFileReadResult, NativeFileError>) + Send>;
/// Host enumeration returns canonical lazy File owners. No filesystem path is
/// interpreted by Interaction or the renderer.
pub type NativeDirectoryCompletion =
    Box<dyn FnOnce(Result<Vec<Arc<crate::File>>, NativeFileError>) + Send>;
pub trait NativeFileReader: Send + Sync {
    /// File::CaptureSnapshotIfNeeded supplies a synchronous metadata query.
    /// A provider without that capability fails explicitly, never with empty
    /// data. The admitted reader remains the only filesystem authority.
    fn SnapshotMetadataSync(&self, _path: &str) -> Result<NativeFileMetadata, NativeFileError> {
        Err(NativeFileError::Unavailable)
    }
    fn ReadSync(
        &self,
        _path: &str,
        _expected: Option<NativeFileMetadata>,
    ) -> Result<NativeFileReadResult, NativeFileError> {
        Err(NativeFileError::Unavailable)
    }
    fn EnumerateDirectory(&self, _root: Arc<crate::File>, completion: NativeDirectoryCompletion) {
        completion(Err(NativeFileError::Unavailable));
    }

    fn ContentType(&self, path: &str) -> Result<String, NativeFileError>;
    fn Read(
        &self,
        path: String,
        expected: Option<NativeFileMetadata>,
        completion: NativeFileReadCompletion,
    );
}
pub struct NativePathBacking {
    pub path: String,
    pub metadata: OnceLock<NativeFileMetadata>,
    pub reader: Arc<dyn NativeFileReader>,
    bytes: OnceLock<Result<Arc<[u8]>, NativeFileError>>,
    read_state: Mutex<ReadState>,
}
impl NativePathBacking {
    pub fn new(
        path: String,
        metadata: Option<NativeFileMetadata>,
        reader: Arc<dyn NativeFileReader>,
    ) -> Arc<Self> {
        let cell = OnceLock::new();
        if let Some(metadata) = metadata {
            let _ = cell.set(metadata);
        }
        Arc::new(Self {
            path,
            metadata: cell,
            reader,
            bytes: OnceLock::new(),
            read_state: Mutex::new(ReadState::default()),
        })
    }
    pub fn CachedBytes(&self) -> Result<Arc<[u8]>, NativeFileError> {
        self.bytes
            .get()
            .cloned()
            .unwrap_or(Err(NativeFileError::MetadataPending))
    }
    pub fn SnapshotMetadataSync(&self) -> Result<NativeFileMetadata, NativeFileError> {
        let _guard = self.read_state.lock().expect("native File read state");
        if let Some(metadata) = self.metadata.get() {
            return Ok(metadata.clone());
        }
        let metadata = self.reader.SnapshotMetadataSync(&self.path)?;
        let _ = self.metadata.set(metadata);
        Ok(self
            .metadata
            .get()
            .expect("native File snapshot committed")
            .clone())
    }
    fn CommitReadResult(&self, value: NativeFileReadResult) -> Result<Arc<[u8]>, NativeFileError> {
        if self.metadata.get().is_some_and(|expected| {
            expected.length != value.metadata.length
                || expected.modification_time_microseconds.map_or(
                    expected.last_modified != value.metadata.last_modified,
                    |time| Some(time) != value.metadata.modification_time_microseconds,
                )
        }) {
            return Err(NativeFileError::Changed);
        }
        let _ = self.metadata.set(value.metadata);
        Ok(value.bytes)
    }
    pub fn ReadSync(&self) -> Result<Arc<[u8]>, NativeFileError> {
        let mut state = self.read_state.lock().expect("native File read state");
        if let Some(result) = self.bytes.get() {
            return result.clone();
        }
        if state.in_flight {
            return Err(NativeFileError::MetadataPending);
        }
        state.in_flight = true;
        let result = self
            .reader
            .ReadSync(&self.path, self.metadata.get().cloned())
            .and_then(|value| self.CommitReadResult(value));
        state.in_flight = false;
        // An unavailable sync capability must not poison a later async read.
        if !matches!(
            result,
            Err(NativeFileError::Unavailable | NativeFileError::MetadataPending)
        ) {
            let _ = self.bytes.set(result.clone());
        }
        result
    }
    pub fn Read(
        self: &Arc<Self>,
        complete: Box<dyn FnOnce(Result<Arc<[u8]>, NativeFileError>) + Send>,
    ) {
        if let Some(result) = self.bytes.get() {
            complete(result.clone());
            return;
        }
        {
            let mut state = self.read_state.lock().expect("native File read state");
            if let Some(result) = self.bytes.get() {
                let result = result.clone();
                drop(state);
                complete(result);
                return;
            }
            state.waiters.push(complete);
            if state.in_flight {
                return;
            }
            state.in_flight = true;
        }
        let owner = self.clone();
        self.reader.Read(
            self.path.clone(),
            self.metadata.get().cloned(),
            Box::new(move |result| {
                let waiters = {
                    let mut state = owner.read_state.lock().expect("native File read state");
                    let result = result.and_then(|value| owner.CommitReadResult(value));
                    let _ = owner.bytes.set(result);
                    state.in_flight = false;
                    std::mem::take(&mut state.waiters)
                };
                let result = owner
                    .bytes
                    .get()
                    .expect("native read commits result")
                    .clone();
                for complete in waiters {
                    complete(result.clone());
                }
            }),
        );
    }
}
#[derive(Clone)]
pub enum FileBacking {
    Memory(Arc<[u8]>),
    NativePath(Arc<NativePathBacking>),
}
impl FileBacking {
    pub fn ReadSync(&self) -> Result<Arc<[u8]>, NativeFileError> {
        match self {
            Self::Memory(bytes) => Ok(bytes.clone()),
            Self::NativePath(owner) => owner.ReadSync(),
        }
    }
    pub fn CachedBytes(&self) -> Result<Arc<[u8]>, NativeFileError> {
        match self {
            Self::Memory(bytes) => Ok(bytes.clone()),
            Self::NativePath(owner) => owner.CachedBytes(),
        }
    }
    pub fn Read(&self, complete: Box<dyn FnOnce(Result<Arc<[u8]>, NativeFileError>) + Send>) {
        match self {
            Self::Memory(bytes) => complete(Ok(bytes.clone())),
            Self::NativePath(owner) => owner.Read(complete),
        }
    }
}
