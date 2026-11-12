//! Native File reads run off the Page/interaction sequence. Every File keeps
//! its original path/reader and receives a checked immutable snapshot.
use std::sync::Arc;
mod directory;
pub(crate) struct NativeReader {
    executor: Arc<dyn executor::Executor>,
}
impl fileapi::NativeFileReader for NativeReader {
    fn SnapshotMetadataSync(
        &self,
        path: &str,
    ) -> Result<fileapi::NativeFileMetadata, fileapi::NativeFileError> {
        let metadata = std::fs::metadata(path)
            .map_err(|error| fileapi::NativeFileError::Io(error.to_string()))?;
        metadata_for_file(
            &metadata,
            crate::native_mime::from_file(path)?.unwrap_or_default(),
        )
    }
    fn ReadSync(
        &self,
        path: &str,
        expected: Option<fileapi::NativeFileMetadata>,
    ) -> Result<fileapi::native_file::NativeFileReadResult, fileapi::NativeFileError> {
        read_path_snapshot(path, expected)
    }
    fn ContentType(&self, path: &str) -> Result<String, fileapi::NativeFileError> {
        Ok(crate::native_mime::from_file(path)?.unwrap_or_default())
    }
    fn Read(
        &self,
        path: String,
        expected: Option<fileapi::NativeFileMetadata>,
        completion: fileapi::native_file::NativeFileReadCompletion,
    ) {
        let completion = Arc::new(std::sync::Mutex::new(Some(completion)));
        let reply = completion.clone();
        let accepted = self.executor.execute(Box::new(move || {
            let result = read_path_snapshot(&path, expected);
            if let Some(completion) = reply.lock().expect("native File completion").take() {
                completion(result);
            }
        }));
        if let Err(error) = accepted {
            if let Some(completion) = completion.lock().expect("native File completion").take() {
                completion(Err(fileapi::NativeFileError::Io(error.to_string())));
            }
        }
    }
}
pub(crate) fn create() -> std::io::Result<Arc<dyn fileapi::NativeFileReader>> {
    Ok(Arc::new(NativeReader {
        executor: Arc::new(executor::PoolExecutor::new(
            "native-file-read",
            2 * 1024 * 1024,
            2,
        )?),
    }))
}

/// Native destination offers retain the granted handle. The path is only the
/// File's public source identity; reads never reopen that path after admission.
pub(crate) struct OfferedReader {
    file: Arc<std::fs::File>,
    access: Option<Arc<dyn winit::event::NativeDragFileAccess>>,
    content_type: String,
    executor: Arc<dyn executor::Executor>,
}
impl fileapi::NativeFileReader for OfferedReader {
    fn SnapshotMetadataSync(
        &self,
        _path: &str,
    ) -> Result<fileapi::NativeFileMetadata, fileapi::NativeFileError> {
        let metadata = self
            .file
            .metadata()
            .map_err(|error| fileapi::NativeFileError::Io(error.to_string()))?;
        metadata_for_file(&metadata, self.content_type.clone())
    }
    fn ReadSync(
        &self,
        _path: &str,
        expected: Option<fileapi::NativeFileMetadata>,
    ) -> Result<fileapi::native_file::NativeFileReadResult, fileapi::NativeFileError> {
        read_file_snapshot(&self.file, expected, self.content_type.clone())
    }
    fn EnumerateDirectory(
        &self,
        root: Arc<fileapi::File>,
        completion: fileapi::native_file::NativeDirectoryCompletion,
    ) {
        let directory = self.file.clone();
        let executor = self.executor.clone();
        let access = self.access.clone();
        let complete = Arc::new(std::sync::Mutex::new(Some(completion)));
        let reply = complete.clone();
        let accepted = self.executor.execute(Box::new(move || {
            let result = directory::enumerate(directory, &root, executor, access);
            if let Some(complete) = reply.lock().expect("directory chooser completion").take() {
                complete(result);
            }
        }));
        if let Err(error) = accepted {
            if let Some(complete) = complete
                .lock()
                .expect("directory chooser completion")
                .take()
            {
                complete(Err(fileapi::NativeFileError::Io(error.to_string())));
            }
        }
    }

    fn ContentType(&self, _path: &str) -> Result<String, fileapi::NativeFileError> {
        Ok(self.content_type.clone())
    }
    fn Read(
        &self,
        _path: String,
        expected: Option<fileapi::NativeFileMetadata>,
        completion: fileapi::native_file::NativeFileReadCompletion,
    ) {
        let file = self.file.clone();
        let access = self.access.clone();
        let complete = Arc::new(std::sync::Mutex::new(Some(completion)));
        let reply = complete.clone();
        let content_type = self.content_type.clone();
        let accepted = self.executor.execute(Box::new(move || {
            let _access = access;
            let result = read_file_snapshot(&file, expected, content_type);
            if let Some(completion) = reply.lock().expect("offered File completion").take() {
                completion(result);
            }
        }));
        if let Err(error) = accepted {
            if let Some(completion) = complete.lock().expect("offered File completion").take() {
                completion(Err(fileapi::NativeFileError::Io(error.to_string())));
            }
        }
    }
}
fn read_path_snapshot(
    path: &str,
    expected: Option<fileapi::NativeFileMetadata>,
) -> Result<fileapi::native_file::NativeFileReadResult, fileapi::NativeFileError> {
    let file = std::fs::File::open(path)
        .map_err(|error| fileapi::NativeFileError::Io(error.to_string()))?;
    let content_type = expected
        .as_ref()
        .map(|metadata| metadata.content_type.clone())
        .map_or_else(
            || crate::native_mime::from_file(path).map(|mime| mime.unwrap_or_default()),
            Ok,
        )?;
    read_file_snapshot(&file, expected, content_type)
}
fn read_file_snapshot(
    file: &std::fs::File,
    expected: Option<fileapi::NativeFileMetadata>,
    content_type: String,
) -> Result<fileapi::native_file::NativeFileReadResult, fileapi::NativeFileError> {
    let before = file
        .metadata()
        .map_err(|error| fileapi::NativeFileError::Io(error.to_string()))?;
    let metadata = metadata_for_file(&before, content_type)?;
    if expected.is_some_and(|expected| {
        expected.length != metadata.length
            || expected
                .modification_time_microseconds
                .map_or(expected.last_modified != metadata.last_modified, |time| {
                    Some(time) != metadata.modification_time_microseconds
                })
    }) {
        return Err(fileapi::NativeFileError::Changed);
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(metadata.length)
        .map_err(|error| fileapi::NativeFileError::Io(error.to_string()))?;
    bytes.resize(metadata.length, 0);
    let mut offset = 0;
    while offset < bytes.len() {
        #[cfg(unix)]
        let count = {
            use std::os::unix::fs::FileExt;
            file.read_at(&mut bytes[offset..], offset as u64)
        };
        #[cfg(windows)]
        let count = {
            use std::os::windows::fs::FileExt;
            file.seek_read(&mut bytes[offset..], offset as u64)
        };
        #[cfg(not(any(unix, windows)))]
        let count: std::io::Result<usize> = Err(std::io::Error::other(
            "native offered-file reads unsupported",
        ));
        let count = count.map_err(|error| fileapi::NativeFileError::Io(error.to_string()))?;
        if count == 0 {
            return Err(fileapi::NativeFileError::Changed);
        }
        offset += count;
    }
    let after = file
        .metadata()
        .map_err(|error| fileapi::NativeFileError::Io(error.to_string()))?;
    let after = metadata_for_file(&after, String::new())?;
    if after.length != metadata.length
        || after.modification_time_microseconds != metadata.modification_time_microseconds
    {
        return Err(fileapi::NativeFileError::Changed);
    }
    Ok(fileapi::native_file::NativeFileReadResult {
        metadata,
        bytes: bytes.into(),
    })
}
fn metadata_for_file(
    metadata: &std::fs::Metadata,
    content_type: String,
) -> Result<fileapi::NativeFileMetadata, fileapi::NativeFileError> {
    let modified = metadata
        .modified()
        .map_err(|error| fileapi::NativeFileError::Io(error.to_string()))?;
    let micros = match modified.duration_since(std::time::UNIX_EPOCH) {
        Ok(time) => i128::try_from(time.as_micros()).ok(),
        Err(error) => i128::try_from(error.duration().as_micros())
            .ok()
            .map(|time| -time),
    }
    .and_then(|time| i64::try_from(time).ok())
    .ok_or_else(|| {
        fileapi::NativeFileError::Io("native offered File timestamp outside Time range".into())
    })?;
    Ok(fileapi::NativeFileMetadata {
        length: usize::try_from(metadata.len()).map_err(|_| {
            fileapi::NativeFileError::Io("native offered File size overflow".into())
        })?,
        last_modified: micros / 1000,
        modification_time_microseconds: Some(micros),
        content_type,
    })
}
pub(crate) fn from_offer(
    offer: &winit::event::NativeDragFileOffer,
) -> std::io::Result<Arc<fileapi::File>> {
    let path = offer
        .path
        .to_str()
        .ok_or_else(|| {
            std::io::Error::other(
                "native drag path cannot be represented by the current File path owner",
            )
        })?
        .to_owned();
    let name = offer
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            std::io::Error::other(
                "native drag filename cannot be represented by the current File name owner",
            )
        })?
        .to_owned();
    let content_type = match if offer.metadata.is_dir() {
        Ok(None)
    } else {
        crate::native_mime::from_file(&path)
    } {
        Ok(mime) => mime.unwrap_or_default(),
        Err(fileapi::NativeFileError::Unavailable) => String::new(),
        Err(error) => return Err(std::io::Error::other(format!("{error:?}"))),
    };
    let metadata = metadata_for_file(&offer.metadata, content_type.clone())
        .map_err(|error| std::io::Error::other(format!("{error:?}")))?;
    let executor = offered_executor()?;
    let reader = Arc::new(OfferedReader {
        file: offer.file.clone(),
        access: offer.access.clone(),
        content_type: content_type.clone(),
        executor,
    });
    Ok(fileapi::File::CreateForNativePath(
        path,
        name,
        String::new(),
        content_type,
        Some(metadata),
        reader,
    ))
}

/// Pending native promises and retained COM streams expose a Blob source
/// identity. Their display name never becomes a filesystem authority.
struct VirtualReader {
    owner: Arc<dyn winit::event::NativeDragDataReader>,
    content_type: String,
    fallback_modified: std::time::SystemTime,
    executor: Arc<dyn executor::Executor>,
}
impl fileapi::NativeFileReader for VirtualReader {
    fn SnapshotMetadataSync(
        &self,
        _path: &str,
    ) -> Result<fileapi::NativeFileMetadata, fileapi::NativeFileError> {
        let metadata = self
            .owner
            .metadata()
            .map_err(|error| fileapi::NativeFileError::Io(error.to_string()))?;
        virtual_metadata(&metadata, self.content_type.clone(), self.fallback_modified)
    }
    fn ReadSync(
        &self,
        _path: &str,
        expected: Option<fileapi::NativeFileMetadata>,
    ) -> Result<fileapi::native_file::NativeFileReadResult, fileapi::NativeFileError> {
        read_virtual_snapshot(
            self.owner.as_ref(),
            expected,
            self.content_type.clone(),
            self.fallback_modified,
        )
    }
    fn ContentType(&self, _path: &str) -> Result<String, fileapi::NativeFileError> {
        Ok(self.content_type.clone())
    }
    fn Read(
        &self,
        _path: String,
        expected: Option<fileapi::NativeFileMetadata>,
        completion: fileapi::native_file::NativeFileReadCompletion,
    ) {
        let owner = self.owner.clone();
        let content_type = self.content_type.clone();
        let fallback = self.fallback_modified;
        let complete = Arc::new(std::sync::Mutex::new(Some(completion)));
        let reply = complete.clone();
        let accepted = self.executor.execute(Box::new(move || {
            let result = read_virtual_snapshot(owner.as_ref(), expected, content_type, fallback);
            if let Some(complete) = reply.lock().expect("virtual File completion").take() {
                complete(result);
            }
        }));
        if let Err(error) = accepted {
            if let Some(complete) = complete.lock().expect("virtual File completion").take() {
                complete(Err(fileapi::NativeFileError::Io(error.to_string())));
            }
        }
    }
}
fn read_virtual_snapshot(
    owner: &dyn winit::event::NativeDragDataReader,
    expected: Option<fileapi::NativeFileMetadata>,
    content_type: String,
    fallback: std::time::SystemTime,
) -> Result<fileapi::native_file::NativeFileReadResult, fileapi::NativeFileError> {
    let before = owner
        .metadata()
        .map_err(|error| fileapi::NativeFileError::Io(error.to_string()))?;
    let metadata = virtual_metadata(&before, content_type, fallback)?;
    if expected.as_ref().is_some_and(|expected| {
        expected.length != metadata.length
            || (before.modified.is_some()
                && expected.modification_time_microseconds.is_some()
                && expected.modification_time_microseconds
                    != metadata.modification_time_microseconds)
    }) {
        return Err(fileapi::NativeFileError::Changed);
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(metadata.length)
        .map_err(|error| fileapi::NativeFileError::Io(error.to_string()))?;
    while bytes.len() < metadata.length {
        let count = (metadata.length - bytes.len()).min(1024 * 1024);
        let part = owner
            .read_range(bytes.len() as u64, count)
            .map_err(|error| fileapi::NativeFileError::Io(error.to_string()))?;
        if part.is_empty() || part.len() > count {
            return Err(fileapi::NativeFileError::Changed);
        }
        bytes.extend_from_slice(&part);
    }
    let after = owner
        .metadata()
        .map_err(|error| fileapi::NativeFileError::Io(error.to_string()))?;
    if before.length != after.length || before.modified != after.modified {
        return Err(fileapi::NativeFileError::Changed);
    }
    Ok(fileapi::native_file::NativeFileReadResult {
        metadata,
        bytes: bytes.into(),
    })
}
fn virtual_metadata(
    metadata: &winit::event::NativeDragFileMetadata,
    content_type: String,
    fallback: std::time::SystemTime,
) -> Result<fileapi::NativeFileMetadata, fileapi::NativeFileError> {
    let modified = metadata.modified.unwrap_or(fallback);
    let micros = match modified.duration_since(std::time::UNIX_EPOCH) {
        Ok(time) => i128::try_from(time.as_micros()).ok(),
        Err(error) => i128::try_from(error.duration().as_micros())
            .ok()
            .map(|time| -time),
    }
    .and_then(|time| i64::try_from(time).ok())
    .ok_or_else(|| {
        fileapi::NativeFileError::Io("virtual File timestamp outside Time range".into())
    })?;
    Ok(fileapi::NativeFileMetadata {
        length: usize::try_from(metadata.length)
            .map_err(|_| fileapi::NativeFileError::Io("virtual File size overflow".into()))?,
        last_modified: micros / 1000,
        modification_time_microseconds: metadata.modified.map(|_| micros),
        content_type,
    })
}
pub(crate) fn from_virtual_offer(
    offer: &winit::event::NativeDragVirtualFileOffer,
) -> std::io::Result<Arc<fileapi::File>> {
    let name = offer
        .name
        .to_str()
        .ok_or_else(|| {
            std::io::Error::other("virtual filename cannot be represented without loss")
        })?
        .to_owned();
    if name.is_empty()
        || std::path::Path::new(&name).file_name() != Some(offer.name.as_os_str())
        || name == "."
        || name == ".."
    {
        return Err(std::io::Error::other(
            "native virtual file requires a display filename",
        ));
    }
    let content_type = if let Some(content_type) = &offer.content_type {
        content_type.clone()
    } else {
        match crate::native_mime::from_file(&name) {
            Ok(value) => value.unwrap_or_default(),
            Err(fileapi::NativeFileError::Unavailable) => String::new(),
            Err(error) => return Err(std::io::Error::other(error.to_string())),
        }
    };
    let fallback = std::time::SystemTime::now();
    // Never call provider metadata on the UI: an AppKit promise may only
    // materialize after Drop, and a COM stream belongs to its worker apartment.
    let metadata = offer
        .metadata
        .as_ref()
        .map(|metadata| virtual_metadata(metadata, content_type.clone(), fallback))
        .transpose()
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    let reader = Arc::new(VirtualReader {
        owner: offer.reader.clone(),
        content_type: content_type.clone(),
        fallback_modified: fallback,
        executor: offered_executor()?,
    });
    Ok(fileapi::File::CreateForNativeData(
        name,
        content_type,
        metadata,
        reader,
    ))
}

/// Finite host I/O pool shared by offered-file reads and directory enumeration.
pub(crate) fn offered_executor() -> std::io::Result<Arc<dyn executor::Executor>> {
    static POOL: std::sync::OnceLock<Result<Arc<dyn executor::Executor>, String>> =
        std::sync::OnceLock::new();
    POOL.get_or_init(|| {
        executor::PoolExecutor::new("native-drop-file-read", 2 * 1024 * 1024, 2)
            .map(|pool| Arc::new(pool) as Arc<dyn executor::Executor>)
            .map_err(|error| error.to_string())
    })
    .clone()
    .map_err(std::io::Error::other)
}
