//! Chromium FormData's binding projection. The canonical entry-list and File
//! owners are shared by form construction, formdata dispatch and navigation.
use super::*;
use interaction::dispatch::core::{
    dom::events::event::EventInit,
    events::form_data_event::FormDataEvent,
    fileapi::{
        file::File,
        form_data::{EntryValue, FormData},
        form_data_object::{FormDataConstructionError, FormDataObject},
    },
};
use std::sync::Arc;

pub(super) struct NativeFormFile {
    file: Arc<File>,
    original: RefCell<Option<Value>>,
}
impl ScriptNativeObject for NativeFormFile {
    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "FormDataFile"
    }
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        if let Some(original) = self.original.borrow().clone() {
            visit(original);
        }
    }
}
impl AbortBindings {
    pub(in crate::dom_bindings) fn ElementInternalsStateValue(
        &self,
        value: dom::element_internals::ElementInternalsValue,
    ) -> HostValue {
        use dom::element_internals::ElementInternalsValue as V;
        match value {
            V::String(value) => HostValue::String(value),
            V::File(file) => self.FormValue(EntryValue::File(file)),
            V::FormData(data) => {
                let data = FormDataObject::new(Rc::unwrap_or_clone(data));
                let id = self.EnsureObject(data.clone());
                self.Trace(data);
                self.NativeValue(id)
            }
        }
    }
    pub(in crate::dom_bindings) fn ElementInternalsValue(
        &self,
        value: &HostValue,
    ) -> Option<Option<dom::element_internals::ElementInternalsValue>> {
        use dom::element_internals::ElementInternalsValue as V;
        match value {
            HostValue::Null(_) => Some(None),
            HostValue::String(text) => Some(Some(V::String(text.clone()))),
            HostValue::String16(text) => Some(Some(V::String(String::from_utf16_lossy(text)))),
            value => self
                .InputFileFromValue(value)
                .map(|file| Some(V::File(file)))
                .or_else(|| {
                    self.NativeFromValue::<FormDataObject>(value)
                        .map(|data| Some(V::FormData(Rc::new(data.Read(Clone::clone)))))
                }),
        }
    }
    pub(in crate::dom_bindings) fn BindFormDataConstructor(
        &self,
        constructor: Option<super::super::services::FormDataConstructor>,
    ) {
        *self.form_data_constructor.borrow_mut() = constructor;
    }
    pub(super) fn FormFile(&self, file: Arc<File>) -> Rc<NativeFormFile> {
        let key = Arc::as_ptr(&file) as usize;
        if let Some(existing) = self.form_files.borrow().get(&key).and_then(Weak::upgrade) {
            return existing;
        }
        let native = Rc::new(NativeFormFile {
            file,
            original: Default::default(),
        });
        self.form_files
            .borrow_mut()
            .insert(key, Rc::downgrade(&native));
        native
    }
    pub(super) fn FormDataNativeObjects(
        &self,
        object: &Rc<dyn ScriptNativeObject>,
        natives: &mut Vec<Rc<dyn ScriptNativeObject>>,
    ) {
        if let Some(data) = object.AsAny().downcast_ref::<FormDataObject>() {
            data.Read(|data| {
                for entry in data.Entries() {
                    if let EntryValue::File(file) = &entry.value {
                        natives.push(self.FormFile(file.clone()));
                    }
                }
            });
        }
    }
    pub(super) fn FormValue(&self, entry: EntryValue) -> HostValue {
        match entry {
            EntryValue::String(text) => HostValue::String(text),
            EntryValue::File(file) => {
                let file = self.FormFile(file);
                let id = self.EnsureObject(file.clone());
                self.Trace(file);
                self.NativeValue(id)
            }
        }
    }
    pub(super) fn InputFileFromValue(&self, value: &HostValue) -> Option<Arc<File>> {
        self.NativeFromValue::<NativeFormFile>(value)
            .map(|file| file.file.clone())
    }
    pub(super) fn InvokeFormData(
        &self,
        receiver: u64,
        member: &str,
        args: &[HostValue],
        runtime: &mut dyn JavaScriptHostRuntime,
    ) -> HostResult {
        let invalid = || {
            HostResult::Failure(
                JavaScriptExceptionKind::kTypeError,
                "Illegal FormData invocation",
            )
        };
        let result = |value| HostResult {
            value,
            ..Default::default()
        };
        let text = |n: usize| match args.get(n) {
            Some(HostValue::String(text)) => text.clone(),
            Some(HostValue::String16(text)) => String::from_utf16_lossy(text),
            _ => String::new(),
        };
        if member == "__formDataCreate" && receiver == 0 {
            let data = if args.is_empty() {
                Ok(FormDataObject::new(FormData::default()))
            } else if let Some(HostValue::Object(form)) = args.first() {
                let constructor = self.form_data_constructor.borrow().clone();
                match constructor {
                    Some(constructor) => constructor(
                        form.id,
                        args.get(1).and_then(|v| match v {
                            HostValue::Object(v) => Some(v.id),
                            _ => None,
                        }),
                        runtime,
                    ),
                    None => Err(FormDataConstructionError::InvalidState(
                        "FormData construction service is unavailable.",
                    )),
                }
            } else {
                Err(FormDataConstructionError::TypeError(
                    "The provided value is not an HTMLFormElement.",
                ))
            };
            return match data {
                Ok(data) => {
                    let id = self.EnsureObject(data.clone());
                    self.Trace(data);
                    result(self.NativeValue(id))
                }
                Err(FormDataConstructionError::TypeError(message)) => {
                    HostResult::Failure(JavaScriptExceptionKind::kTypeError, message)
                }
                Err(FormDataConstructionError::NotFound(message)) => HostResult {
                    thrown: Some((self.exception)("NotFoundError", message)),
                    ..Default::default()
                },
                Err(FormDataConstructionError::InvalidState(message)) => HostResult {
                    thrown: Some((self.exception)("InvalidStateError", message)),
                    ..Default::default()
                },
            };
        }
        if member == "__formDataFileCreate" && receiver == 0 {
            let Some(HostValue::Bytes(bytes)) = args.first() else {
                return invalid();
            };
            let modified = match args.get(3) {
                Some(HostValue::Number(n)) => *n as i64,
                _ => 0,
            };
            let native = self.FormFile(File::Create(text(1), text(2), modified, bytes.clone()));
            let id = self.EnsureObject(native.clone());
            self.Trace(native);
            return result(self.NativeValue(id));
        }
        if member == "__formDataEventCreate" && receiver == 0 {
            let Some(data) = args
                .get(1)
                .and_then(|v| self.NativeFromValue::<FormDataObject>(v))
            else {
                return invalid();
            };
            let Some(clock) = self.clock.as_ref() else {
                return invalid();
            };
            let b = |n| matches!(args.get(n), Some(HostValue::Boolean(true)));
            let event = EventRef::new(FormDataEvent::WithType(
                &text(0),
                EventInit {
                    bubbles: b(2),
                    cancelable: b(3),
                    composed: b(4),
                },
                data.clone(),
                clock.as_ref(),
            ));
            let id = self
                .events
                .borrow_mut()
                .BindConstructedNative(&event, || self.Allocate());
            let data_id = self.EnsureObject(data.clone());
            self.Trace(data);
            self.Publish([C::Reference {
                owner: id,
                slot: 0,
                target: Some(data_id),
            }]);
            return result(self.NativeValue(id));
        }
        if member == "__formDataEventData" {
            let event = self.events.borrow().NativeEvent(receiver);
            let data = event.and_then(|event| {
                event
                    .borrow()
                    .AsFormDataEvent()
                    .map(|event| event.formData())
            });
            let Some(data) = data else {
                return invalid();
            };
            let id = self.EnsureObject(data.clone());
            self.Trace(data);
            self.Publish([C::Reference {
                owner: receiver,
                slot: 0,
                target: Some(id),
            }]);
            return result(self.NativeValue(id));
        }
        let object = self.objects.borrow().get(&receiver).cloned();
        let Some(object) = object else {
            return invalid();
        };
        if let Some(file) = object.AsAny().downcast_ref::<NativeFormFile>() {
            return match member {
                "__formDataFileRemember" => {
                    *file.original.borrow_mut() = args.first().cloned().map(value);
                    self.Trace(object.clone());
                    result(HostValue::default())
                }
                "__formDataFileOriginal" => result(
                    file.original
                        .borrow()
                        .as_ref()
                        .map(host)
                        .unwrap_or_default(),
                ),
                "__formDataFileIdentity" => result(HostValue::Record(vec![
                    ("name".into(), HostValue::String(file.file.name().into())),
                    ("type".into(), HostValue::String(file.file.type_().into())),
                ])),
                "__formDataFileMetadata" => match file.file.SnapshotMetadataSync() {
                    Ok(metadata) => result(HostValue::Record(vec![
                        ("size".into(), HostValue::Number(metadata.length as f64)),
                        (
                            "lastModified".into(),
                            HostValue::Number(metadata.last_modified as f64),
                        ),
                    ])),
                    Err(_) => HostResult {
                        thrown: Some((self.exception)(
                            "NotReadableError",
                            "The native File reader cannot synchronously capture this snapshot.",
                        )),
                        ..Default::default()
                    },
                },
                "__formDataFileRead" => match file.file.ReadBytesSync() {
                    Ok(bytes) => result(HostValue::Bytes(bytes)),
                    Err(error) => HostResult {
                        thrown: Some((self.exception)(
                            "NotReadableError",
                            &format!("Native File snapshot read failed: {error}"),
                        )),
                        ..Default::default()
                    },
                },
                _ => invalid(),
            };
        }
        let Some(data) = object.AsAny().downcast_ref::<FormDataObject>() else {
            return invalid();
        };
        let name = text(0);
        let output = match member {
            "__formDataBrand" => HostValue::default(),
            "__formDataGet" => data
                .Read(|d| d.Get(&name))
                .map(|v| self.FormValue(v))
                .unwrap_or(HostValue::Null(JavaScriptNull)),
            "__formDataGetAll" => HostValue::Array(
                data.Read(|d| d.GetAll(&name))
                    .into_iter()
                    .map(|v| self.FormValue(v))
                    .collect(),
            ),
            "__formDataHas" => HostValue::Boolean(data.Read(|d| d.Has(&name))),
            "__formDataDelete" => {
                data.Mutate(|d| d.Delete(&name));
                HostValue::default()
            }
            "__formDataAppend" => {
                data.Mutate(|d| d.AppendString(name, text(1)));
                HostValue::default()
            }
            "__formDataSet" => {
                data.Mutate(|d| d.Set(name, EntryValue::String(text(1))));
                HostValue::default()
            }
            "__formDataAppendFile" | "__formDataSetFile" => {
                let Some(file) = args
                    .get(1)
                    .and_then(|v| self.NativeFromValue::<NativeFormFile>(v))
                else {
                    return invalid();
                };
                let filename =
                    matches!(args.get(2), Some(HostValue::Boolean(true))).then(|| text(3));
                data.Mutate(|d| {
                    if member == "__formDataSetFile" {
                        d.SetFileWithFilename(name, file.file.clone(), filename)
                    } else {
                        d.AppendFileWithFilename(name, file.file.clone(), filename)
                    }
                });
                HostValue::default()
            }
            "__formDataEntryAt" => {
                let index = match args.first() {
                    Some(HostValue::Number(n)) => *n as usize,
                    _ => return invalid(),
                };
                data.Read(|d| d.EntryAt(index))
                    .map(|(name, v)| {
                        HostValue::Array(vec![HostValue::String(name), self.FormValue(v)])
                    })
                    .unwrap_or(HostValue::Null(JavaScriptNull))
            }
            _ => return invalid(),
        };
        self.Trace(object);
        result(output)
    }
}
