use fileapi::{
    form_data::{EntryValue, FormData, FormDataStateRestoreError},
    native_file::NativeFileReadCompletion,
    File, NativeFileError, NativeFileMetadata, NativeFileReader,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Default)]
struct Reader {
    metadata: AtomicUsize,
    reads: AtomicUsize,
}
fn metadata() -> NativeFileMetadata {
    NativeFileMetadata {
        length: 7,
        last_modified: 42,
        modification_time_microseconds: Some(42_000),
        content_type: "text/plain".into(),
    }
}
impl NativeFileReader for Reader {
    fn ContentType(&self, _: &str) -> Result<String, NativeFileError> {
        Ok("text/plain".into())
    }
    fn SnapshotMetadataSync(&self, path: &str) -> Result<NativeFileMetadata, NativeFileError> {
        assert_eq!(path, "/admitted/report.txt");
        self.metadata.fetch_add(1, Ordering::SeqCst);
        Ok(metadata())
    }
    fn ReadSync(
        &self,
        path: &str,
        _: Option<NativeFileMetadata>,
    ) -> Result<fileapi::native_file::NativeFileReadResult, NativeFileError> {
        assert_eq!(path, "/admitted/report.txt");
        self.reads.fetch_add(1, Ordering::SeqCst);
        Ok(fileapi::native_file::NativeFileReadResult {
            metadata: metadata(),
            bytes: Arc::from(b"secret!".as_slice()),
        })
    }
    fn Read(
        &self,
        path: String,
        expected: Option<NativeFileMetadata>,
        complete: NativeFileReadCompletion,
    ) {
        complete(self.ReadSync(&path, expected));
    }
}

#[test]
fn formdata_history_state_retains_native_authority_without_reading_contents() {
    let reader = Arc::new(Reader::default());
    let file = File::CreateForNativePath(
        "/admitted/report.txt".into(),
        "report.txt".into(),
        "folder/report.txt".into(),
        "text/plain".into(),
        None,
        reader.clone(),
    );
    let mut original = FormData::default();
    original.AppendString("same".into(), "first".into());
    original.AppendString("same".into(), "second".into());
    original.AppendFile("attachment".into(), file);
    let mut state = Vec::new();
    original.AppendToControlState(&mut state);
    assert!(!state.iter().any(
        |value| value.as_ref() == b"secret!".iter().map(|&c| u16::from(c)).collect::<Vec<_>>()
    ));
    assert!(matches!(
        FormData::CreateFromControlState(&state, &mut 0, None),
        Err(FormDataStateRestoreError::NativeFileReaderUnavailable)
    ));
    let mut index = 0;
    let restored = FormData::CreateFromControlState(&state, &mut index, Some(reader.clone()))
        .unwrap()
        .unwrap();
    assert_eq!(index, state.len());
    assert_eq!(
        restored
            .GetAll("same")
            .iter()
            .map(|value| value.StringForFormData())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    let EntryValue::File(file) = restored.Get("attachment").unwrap() else {
        panic!("restored File owner");
    };
    assert_eq!(file.GetPath(), "/admitted/report.txt");
    assert_eq!(reader.metadata.load(Ordering::SeqCst), 0);
    assert_eq!(reader.reads.load(Ordering::SeqCst), 0);
    assert_eq!(file.SnapshotMetadataSync().unwrap().length, 7);
    assert_eq!(file.SnapshotMetadataSync().unwrap().last_modified, 42);
    assert_eq!(reader.metadata.load(Ordering::SeqCst), 1);
    assert_eq!(reader.reads.load(Ordering::SeqCst), 0);
    let bytes = file.ReadBytesSync().unwrap();
    assert_eq!(bytes.as_ref(), b"secret!");
    file.ReadBytes(Box::new(move |value| {
        assert!(Arc::ptr_eq(&bytes, &value.unwrap()))
    }));
    assert_eq!(reader.reads.load(Ordering::SeqCst), 1);
}

#[test]
fn unavailable_sync_capability_does_not_poison_the_canonical_async_owner() {
    struct AsyncReader;
    impl NativeFileReader for AsyncReader {
        fn ContentType(&self, _: &str) -> Result<String, NativeFileError> {
            Ok("text/plain".into())
        }
        fn Read(
            &self,
            _: String,
            _: Option<NativeFileMetadata>,
            complete: NativeFileReadCompletion,
        ) {
            complete(Ok(fileapi::native_file::NativeFileReadResult {
                metadata: metadata(),
                bytes: Arc::from(b"secret!".as_slice()),
            }));
        }
    }
    let file = File::CreateForNativePath(
        "/admitted/report.txt".into(),
        "report.txt".into(),
        String::new(),
        "text/plain".into(),
        None,
        Arc::new(AsyncReader),
    );
    assert!(matches!(
        file.ReadBytesSync(),
        Err(NativeFileError::Unavailable)
    ));
    assert!(matches!(
        file.SnapshotMetadataSync(),
        Err(NativeFileError::Unavailable)
    ));
    file.ReadBytes(Box::new(|value| {
        assert_eq!(value.unwrap().as_ref(), b"secret!")
    }));
    assert_eq!(file.ReadBytesSync().unwrap().as_ref(), b"secret!");
}
