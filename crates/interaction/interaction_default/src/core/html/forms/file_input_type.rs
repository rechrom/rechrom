//! FileInputType::ReceiveDroppedFiles/SetFilesAndDispatchEvents,
//! file_input_type.cc:436-479,527-568. The native File owner is passed through;
//! no path, bytes or FileList is reconstructed from script-visible strings.
use super::input_type::{FormDataAppendAdmission, InputFormDataOperations};
use interaction_dispatch::core::fileapi::form_data::FormData;
use interaction_dispatch::core::fileapi::{file::File, file_list::FileList};
use std::{rc::Rc, sync::Arc};
pub struct FileInputType;
impl FileInputType {
    /// file_input_type.cc:97-100.
    pub fn ValueMissingWithState(required: bool, files_empty: bool) -> bool {
        required && files_empty
    }

    /// FileInputType::AppendToFormData, cc:147-158. Preserve canonical File
    /// backing identity; an empty chooser contributes the source empty File.
    pub fn AppendWithOperations(
        element: &dyn InputFormDataOperations,
        data: &mut FormData,
    ) -> Result<(), FormDataAppendAdmission> {
        let files = element
            .FilesForFormData()
            .ok_or(FormDataAppendAdmission::FileList)?;
        let name = element.NameForFormData();
        if files.length() == 0 {
            data.AppendFileFromElement(
                &name,
                File::Create(
                    String::new(),
                    "application/octet-stream".into(),
                    File::NowMilliseconds(),
                    Arc::from([]),
                ),
            );
        } else {
            for index in 0..files.length() {
                data.AppendFileFromElement(
                    &name,
                    files.item(index).expect("canonical FileList entry"),
                );
            }
        }
        Ok(())
    }
}
pub trait FileInputDropOperations {
    fn Multiple(&self) -> bool;
    fn Directory(&self) -> bool;
    fn CurrentFiles(&self) -> Rc<FileList>;
    fn SetFiles(&self, files: Rc<FileList>);
    fn DispatchInput(&self);
    fn DispatchChange(&self);
    fn DispatchCancel(&self);
    fn EnumerateDroppedDirectory(&self, directory: Arc<File>) -> Result<(), &'static str>;
}
pub fn ReceiveDroppedFiles(
    operations: &dyn FileInputDropOperations,
    files: &[Arc<File>],
) -> Result<bool, &'static str> {
    let mut paths = files.iter().filter(|file| !file.GetPath().is_empty());
    let Some(first) = paths.next() else {
        return Ok(false);
    };
    if operations.Directory() {
        operations.EnumerateDroppedDirectory(first.clone())?;
        return Ok(true);
    }
    let selected = Rc::new(FileList::default());
    selected.Append(first.clone());
    if operations.Multiple() {
        for file in paths {
            selected.Append(file.clone());
        }
    }
    SetFilesAndDispatchEvents(operations, selected);
    Ok(true)
}
/// Shared FileInputType::SetFilesAndDispatchEvents commit point for drops and
/// asynchronous native directory chooser completion.
pub fn SetFilesAndDispatchEvents(operations: &dyn FileInputDropOperations, selected: Rc<FileList>) {
    let old = operations.CurrentFiles();
    let changed = old.length() != selected.length()
        || (0..old.length()).any(|index| {
            !old.item(index)
                .zip(selected.item(index))
                .is_some_and(|(a, b)| a.HasSameSource(&b))
        });
    operations.SetFiles(selected);
    if changed {
        operations.DispatchInput();
        operations.DispatchChange();
    } else {
        operations.DispatchCancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use interaction_dispatch::core::fileapi::{file_list::FileList, form_data::EntryValue};
    use std::rc::Rc;
    struct Input(Rc<FileList>);
    impl InputFormDataOperations for Input {
        fn NameForFormData(&self) -> Rc<[u16]> {
            Rc::from([117, 112, 108, 111, 97, 100])
        }
        fn ValueForFormData(&self) -> Rc<[u16]> {
            panic!("file entry never reads the input string value")
        }
        fn FilesForFormData(&self) -> Option<Rc<FileList>> {
            Some(self.0.clone())
        }
    }
    #[test]
    fn file_entries_preserve_backing_and_empty_selection_is_an_empty_file() {
        let input = Input(Rc::new(FileList::default()));
        let mut empty = FormData::default();
        FileInputType::AppendWithOperations(&input, &mut empty).unwrap();
        let EntryValue::File(file) = empty.Get("upload").unwrap() else {
            panic!("expected File");
        };
        assert_eq!(file.name(), "");
        assert_eq!(file.type_(), "application/octet-stream");
        assert_eq!(file.TrySize().unwrap(), 0);
        let selected = File::Create(
            "report.bin".into(),
            "application/octet-stream".into(),
            1,
            Arc::from([1, 2, 3]),
        );
        input.0.Append(selected.clone());
        let mut data = FormData::default();
        FileInputType::AppendWithOperations(&input, &mut data).unwrap();
        let EntryValue::File(file) = data.Get("upload").unwrap() else {
            panic!("expected File");
        };
        assert!(Arc::ptr_eq(&selected, &file));
    }
}

/// FileInputType.cc:122-145. Only File's canonical native source triples are
/// persisted. Blob-created files are skipped by File::AppendToControlState.
pub fn SaveFormControlState(files: &FileList, drop_referenced_paths: bool) -> Vec<Rc<[u16]>> {
    if drop_referenced_paths {
        return Vec::new();
    }
    let mut state = Vec::new();
    for index in 0..files.length() {
        files
            .item(index)
            .expect("live file list")
            .AppendToControlState(&mut state);
    }
    state
        .into_iter()
        .map(|value| value.encode_utf16().collect::<Vec<_>>().into())
        .collect()
}
/// Restored native files keep lazy host-backed content. No bytes or metadata
/// are manufactured from a serialized path.
pub fn RestoreFormControlState(
    state: &[Rc<[u16]>],
    reader: Arc<dyn interaction_dispatch::core::fileapi::native_file::NativeFileReader>,
) -> Result<Option<Rc<FileList>>, navigation_history::FormStateRestoreError> {
    if state.len() % 3 != 0 {
        return Ok(None);
    }
    let files = Rc::new(FileList::default());
    let mut index = 0;
    while index < state.len() {
        if let Some(file) = File::CreateFromControlState(state, &mut index, reader.clone())
            .map_err(|error| {
                navigation_history::FormStateRestoreError::NativeFileUnavailable(format!(
                    "{error:?}"
                ))
            })?
        {
            files.Append(file);
        }
    }
    Ok(Some(files))
}

/// FileInputType::FilesFromFormControlState, cc:116-120.
pub fn FilesFromFormControlState(state: &[Rc<[u16]>]) -> Vec<Rc<[u16]>> {
    let mut paths = Vec::new();
    let mut index = 0;
    while index < state.len() {
        paths.push(File::PathFromControlStateUTF16(state, &mut index));
    }
    paths
}
