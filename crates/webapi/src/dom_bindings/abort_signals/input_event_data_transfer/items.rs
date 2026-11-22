//! Chromium data_transfer_item_list.cc:36-99, data_transfer_item.cc:47-110,
//! file_list.cc:33-58. Bindings retain the original collection/item/File owners.
use super::*;
use interaction::dispatch::core::{
    clipboard::{
        data_transfer_item::DataTransferItem,
        data_transfer_item_list::{DataTransferItemList, DataTransferItemListError},
    },
    fileapi::{file::File, file_list::FileList, form_data::EntryValue},
};
use std::sync::Arc;

struct NativeInputDataTransferList(Rc<DataTransferItemList>);
struct NativeInputDataTransferItem(Rc<DataTransferItem>);
struct NativeInputFileList(Rc<FileList>);
macro_rules! native_input_wrapper {
    ($wrapper:ident, $name:literal) => {
        impl ScriptNativeObject for $wrapper {
            fn AsAny(&self) -> &dyn Any {
                self
            }
            fn ScriptIdentity(&self) -> usize {
                Rc::as_ptr(&self.0) as usize
            }
            fn ScriptInterfaceName(&self) -> &'static str {
                $name
            }
        }
    };
}
native_input_wrapper!(NativeInputDataTransferList, "DataTransferItemList");
native_input_wrapper!(NativeInputDataTransferItem, "DataTransferItem");
native_input_wrapper!(NativeInputFileList, "FileList");

impl AbortBindings {
    pub(in crate::dom_bindings::abort_signals) fn InputDataTransferNativeObjects(
        &self,
        object: &Rc<dyn ScriptNativeObject>,
        natives: &mut Vec<Rc<dyn ScriptNativeObject>>,
    ) {
        if let Some(transfer) = object.AsAny().downcast_ref::<NativeInputDataTransfer>() {
            natives.push(Rc::new(NativeInputFileList(
                transfer.transfer.FileListForTracing(),
            )));
        } else if let Some(list) = object.AsAny().downcast_ref::<NativeInputDataTransferList>() {
            natives.push(self.InputTransferNative(list.0.GetDataTransfer()));
        } else if let Some(item) = object.AsAny().downcast_ref::<NativeInputDataTransferItem>() {
            natives.push(self.InputTransferNative(item.0.GetDataTransfer()));
            if let Some(file) = item.0.GetDataObjectItem().GetAsFile() {
                natives.push(self.FormFile(file));
            }
        } else if let Some(list) = object.AsAny().downcast_ref::<NativeInputFileList>() {
            for i in 0..list.0.length() {
                if let Some(file) = list.0.item(i) {
                    natives.push(self.FormFile(file));
                }
            }
        }
    }

    pub(in crate::dom_bindings) fn IsInputIndexedObject(&self, receiver: u64) -> bool {
        self.objects.borrow().get(&receiver).is_some_and(|native| {
            native.AsAny().is::<NativeInputFileList>()
                || native.AsAny().is::<NativeInputDataTransferList>()
        })
    }

    pub(in crate::dom_bindings) fn PrepareInputIndexedGet(
        self: &Rc<Self>,
        receiver: u64,
        member: &str,
    ) -> Option<HostContinuation> {
        if !self.IsInputIndexedObject(receiver) {
            return None;
        }
        let index = member
            .parse::<u32>()
            .ok()
            .filter(|index| *index != u32::MAX && index.to_string() == member)?;
        let owner = self.clone();
        Some(Box::new(move |runtime| {
            owner.InvokeInputDataTransfer(
                receiver,
                "__inputDataTransferIndexedItem",
                &[HostValue::Number(index as f64)],
                runtime,
            )
        }))
    }

    pub(super) fn InputTransferItemsValue(&self, list: Rc<DataTransferItemList>) -> HostValue {
        self.InputNativeValue(Rc::new(NativeInputDataTransferList(list)))
    }
    pub(super) fn InputTransferFilesValue(&self, list: Rc<FileList>) -> HostValue {
        self.InputNativeValue(Rc::new(NativeInputFileList(list)))
    }
    fn InputItemValue(&self, item: Option<Rc<DataTransferItem>>) -> HostValue {
        item.map_or(HostValue::Null(JavaScriptNull), |item| {
            self.InputNativeValue(Rc::new(NativeInputDataTransferItem(item)))
        })
    }
    fn InputFileValue(
        &self,
        file: Option<Arc<File>>,
        runtime: &mut dyn JavaScriptHostRuntime,
    ) -> HostResult {
        let Some(file) = file else {
            return HostResult {
                value: HostValue::Null(JavaScriptNull),
                ..Default::default()
            };
        };
        let value = self.FormValue(EntryValue::File(file));
        let Some(project) = self.input_file_projection.borrow().clone() else {
            return HostResult {
                value,
                ..Default::default()
            };
        };
        runtime.CallValue(
            &HostValue::JavaScriptFunction(project),
            &HostValue::Null(JavaScriptNull),
            &[value],
        )
    }

    pub(super) fn InvokeInputTransferItems(
        &self,
        receiver: u64,
        member: &str,
        args: &[HostValue],
        runtime: &mut dyn JavaScriptHostRuntime,
    ) -> Option<HostResult> {
        let object = HostValue::Object(HostObjectRef { id: receiver });
        let result = |value| HostResult {
            value,
            ..Default::default()
        };
        let index = || match args.first() {
            Some(HostValue::Number(n)) => Some(*n as u32),
            _ => None,
        };
        if let Some(list) = self.NativeFromValue::<NativeInputFileList>(&object) {
            return Some(match member {
                "__inputDataTransferListLength" => {
                    result(HostValue::Number(list.0.length() as f64))
                }
                "__inputDataTransferIndexedItem" | "__inputDataTransferFileListItem" => {
                    let Some(index) = index() else {
                        return Some(Self::InvalidInputDataTransfer());
                    };
                    let file = list.0.item(index);
                    if file.is_none() && member == "__inputDataTransferIndexedItem" {
                        return Some(HostResult::default());
                    }
                    self.InputFileValue(file, runtime)
                }
                _ => Self::InvalidInputDataTransfer(),
            });
        }
        if let Some(list) = self.NativeFromValue::<NativeInputDataTransferList>(&object) {
            let changed = match member {
                "__inputDataTransferListBrand" => return Some(result(HostValue::Boolean(true))),
                "__inputDataTransferListLength" => {
                    return Some(result(HostValue::Number(list.0.length() as f64)))
                }
                "__inputDataTransferIndexedItem" | "__inputDataTransferListItem" => {
                    let Some(index) = index() else {
                        return Some(Self::InvalidInputDataTransfer());
                    };
                    let item = list.0.item(index);
                    if item.is_none() && member == "__inputDataTransferIndexedItem" {
                        return Some(HostResult::default());
                    }
                    return Some(result(self.InputItemValue(item)));
                }
                "__inputDataTransferListAdd" => {
                    let Some(HostValue::String(kind)) = args.get(1) else {
                        return Some(Self::InvalidInputDataTransfer());
                    };
                    let data = match args.first() {
                        Some(HostValue::String(value)) => value.encode_utf16().collect::<Vec<_>>(),
                        Some(HostValue::String16(value)) => value.to_vec(),
                        _ => return Some(Self::InvalidInputDataTransfer()),
                    };
                    list.0.addUTF16(&data, kind)
                }
                "__inputDataTransferListAddFile" => {
                    let Some(file) = args.first().and_then(|v| self.InputFileFromValue(v)) else {
                        return Some(Self::InvalidInputDataTransfer());
                    };
                    Ok(list.0.addFile(file))
                }
                "__inputDataTransferListRemove" => {
                    let Some(index) = index() else {
                        return Some(Self::InvalidInputDataTransfer());
                    };
                    list.0.deleteItem(index).map(|_| None)
                }
                "__inputDataTransferListClear" => {
                    list.0.clear();
                    Ok(None)
                }
                _ => return Some(Self::InvalidInputDataTransfer()),
            };
            self.Trace(list);
            return Some(match changed {
                Ok(item)
                    if matches!(
                        member,
                        "__inputDataTransferListAdd" | "__inputDataTransferListAddFile"
                    ) =>
                {
                    result(self.InputItemValue(item))
                }
                Ok(_) => HostResult::default(),
                Err(DataTransferItemListError::InvalidState) => HostResult {
                    thrown: Some((self.exception)(
                        "InvalidStateError",
                        "The list is not writable.",
                    )),
                    ..Default::default()
                },
                Err(DataTransferItemListError::NotSupported(kind)) => HostResult {
                    thrown: Some((self.exception)(
                        "NotSupportedError",
                        &format!("An item already exists for type '{kind}'."),
                    )),
                    ..Default::default()
                },
            });
        }
        let item = self.NativeFromValue::<NativeInputDataTransferItem>(&object)?;
        Some(match member {
            "__inputDataTransferItemKind" => result(HostValue::String(item.0.kind().into())),
            "__inputDataTransferItemType" => result(HostValue::String(item.0.type_().into())),
            "__inputDataTransferItemFile" => self.InputFileValue(item.0.getAsFile(), runtime),
            "__inputDataTransferItemString" => {
                if matches!(
                    args.first(),
                    None | Some(HostValue::Null(_)) | Some(HostValue::Undefined(_))
                ) {
                    return Some(HostResult::default());
                }
                let Some(HostValue::JavaScriptFunction(callback)) = args.first() else {
                    return Some(Self::InvalidInputDataTransfer());
                };
                let Some(text) = item.0.GetAsStringForTask() else {
                    return Some(HostResult::default());
                };
                let Some(schedule) = self.scheduler.get() else {
                    return Some(HostResult::Failure(
                        JavaScriptExceptionKind::kRuntimeError,
                        "DataTransferItem callback task runner is unavailable",
                    ));
                };
                let text = HostValue::String16(
                    (0..text.length())
                        .map(|i| text.CodeUnitAt(i))
                        .collect::<Vec<_>>()
                        .into(),
                );
                let callback = callback.clone();
                let owner = self.self_.clone();
                schedule(
                    0,
                    false,
                    Box::new(move |runtime| {
                        if owner.upgrade().is_none_or(|owner| owner.destroyed.get()) {
                            return;
                        }
                        let called = runtime.CallValue(
                            &HostValue::JavaScriptFunction(callback),
                            &HostValue::Null(JavaScriptNull),
                            &[text],
                        );
                        if let Some(error) = called.thrown {
                            runtime.ReportException(&error);
                        }
                    }),
                );
                HostResult::default()
            }
            _ => Self::InvalidInputDataTransfer(),
        })
    }
}
