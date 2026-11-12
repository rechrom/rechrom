//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: core/html/forms/form_data.cc:250-362;
//! platform/network/form_data_encoder.cc:45-91,128-238.
//! UTF-8 form entries retain immutable File owners; encoded output copies only
//! final network bytes. The embedding supplies the random boundary bytes.
use crate::File;
use std::sync::Arc;
#[derive(Debug)]
pub enum FormDataStateRestoreError {
    NativeFileReaderUnavailable,
    NativeFile(crate::native_file::NativeFileError),
}
#[derive(Clone)]
pub enum EntryValue {
    String(String),
    File(Arc<File>),
}
#[derive(Clone)]
pub struct Entry {
    pub name: String,
    pub value: EntryValue,
    filename: Option<String>,
}
#[derive(Clone)]
pub struct FormData {
    entries: Vec<Entry>,
    encoding: &'static encoding_rs::Encoding,
    contains_password_data: bool,
}
impl Default for FormData {
    fn default() -> Self {
        Self::WithEncoding("UTF-8")
    }
}
impl FormData {
    /// Blink FormData::AppendToControlState, form_data.cc:399-411.
    pub fn AppendToControlState(&self, state: &mut Vec<std::rc::Rc<[u16]>>) {
        fn append(state: &mut Vec<std::rc::Rc<[u16]>>, text: &str) {
            state.push(text.encode_utf16().collect::<Vec<_>>().into());
        }
        append(state, &self.entries.len().to_string());
        for entry in &self.entries {
            append(state, &entry.name);
            match &entry.value {
                EntryValue::String(value) => {
                    append(state, "USVString");
                    append(state, value);
                }
                EntryValue::File(file) => {
                    append(state, "File");
                    let mut file_state = Vec::new();
                    file.AppendToControlState(&mut file_state);
                    for value in file_state {
                        append(state, &value);
                    }
                }
            }
        }
    }
    /// Blink FormData::CreateFromControlState, form_data.cc:413-444. File
    /// admission uses the same injected native reader as ordinary file inputs.
    pub fn CreateFromControlState(
        state: &[std::rc::Rc<[u16]>],
        index: &mut usize,
        reader: Option<Arc<dyn crate::native_file::NativeFileReader>>,
    ) -> Result<Option<Self>, FormDataStateRestoreError> {
        let Some(length) = state
            .get(*index)
            .and_then(|value| String::from_utf16(value).ok())
            .filter(|value| !value.is_empty() && value.bytes().all(|c| c.is_ascii_digit()))
            .and_then(|value| value.parse::<u64>().ok())
        else {
            return Ok(None);
        };
        *index += 1;
        let mut data = Self::default();
        for _ in 0..length {
            if index.saturating_add(2) >= state.len() {
                return Ok(None);
            }
            let name = String::from_utf16_lossy(&state[*index]);
            let kind = &state[*index + 1];
            *index += 2;
            if kind.as_ref() == [70, 105, 108, 101] {
                let reader = reader
                    .clone()
                    .ok_or(FormDataStateRestoreError::NativeFileReaderUnavailable)?;
                let Some(file) = File::CreateFromControlState(state, index, reader)
                    .map_err(FormDataStateRestoreError::NativeFile)?
                else {
                    return Ok(None);
                };
                data.AppendFile(name, file);
            } else if kind.as_ref() == [85, 83, 86, 83, 116, 114, 105, 110, 103] {
                data.AppendString(name, String::from_utf16_lossy(&state[*index]));
                *index += 1;
            } else {
                return Ok(None);
            }
        }
        Ok(Some(data))
    }
    pub fn WithEncoding(label: &str) -> Self {
        let encoding =
            encoding_rs::Encoding::for_label(label.as_bytes()).expect("valid source form encoding");
        let encoding = if matches!(encoding.name(), "UTF-16LE" | "UTF-16BE") {
            encoding_rs::UTF_8
        } else {
            encoding
        };
        Self {
            entries: Vec::new(),
            encoding,
            contains_password_data: false,
        }
    }
    /// FormDataEncoder.cc:102-119, TextEncoding.cc:93-96. The fallback is the
    /// original Document decoder's actual encoding, not an HTTP label guess.
    pub fn EncodingFromAcceptCharset(accept: &str, fallback: &str) -> &'static str {
        accept
            .replace(',', " ")
            .split(' ')
            .filter(|part| !part.is_empty())
            .find_map(|part| encoding_rs::Encoding::for_label(part.as_bytes()))
            .unwrap_or_else(|| {
                encoding_rs::Encoding::for_label(fallback.as_bytes())
                    .expect("actual document encoding")
            })
            .name()
    }
    pub fn GenerateIdentifier() -> i64 {
        use std::sync::{
            atomic::{AtomicI64, Ordering},
            OnceLock,
        };
        static NEXT: OnceLock<AtomicI64> = OnceLock::new();
        NEXT.get_or_init(|| {
            AtomicI64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("form timestamp epoch")
                    .as_micros()
                    .try_into()
                    .expect("form identifier fits i64"),
            )
        })
        .fetch_add(1, Ordering::Relaxed)
            + 1
    }
    pub fn EncodingName(&self) -> String {
        self.encoding.name().into()
    }
    pub fn SetContainsPasswordData(&mut self, value: bool) {
        self.contains_password_data = value;
    }
    pub fn ContainsPasswordData(&self) -> bool {
        self.contains_password_data
    }
    // Source USVString conversion occurs exactly at AppendFromElement.
    pub fn AppendFromElement(&mut self, name: &[u16], value: &[u16]) {
        self.AppendString(
            String::from_utf16_lossy(name),
            String::from_utf16_lossy(value),
        );
    }
    pub fn AppendFileFromElement(&mut self, name: &[u16], file: Arc<File>) {
        self.AppendFile(String::from_utf16_lossy(name), file);
    }
    pub fn Delete(&mut self, name: &str) {
        self.entries.retain(|entry| entry.name != name);
    }
    pub fn Get(&self, name: &str) -> Option<EntryValue> {
        self.entries
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.ValueForBindings())
    }
    pub fn GetAll(&self, name: &str) -> Vec<EntryValue> {
        self.entries
            .iter()
            .filter(|entry| entry.name == name)
            .map(|entry| entry.ValueForBindings())
            .collect()
    }
    pub fn Has(&self, name: &str) -> bool {
        self.entries.iter().any(|entry| entry.name == name)
    }
    pub fn Set(&mut self, name: String, value: EntryValue) {
        if let Some(index) = self.entries.iter().position(|entry| entry.name == name) {
            self.entries[index].value = value;
            self.entries[index].filename = None;
            let mut position = 0;
            self.entries.retain(|entry| {
                let keep = position <= index || entry.name != name;
                position += 1;
                keep
            });
        } else {
            self.entries.push(Entry {
                name,
                value,
                filename: None,
            });
        }
    }
    fn Encode(&self, value: &str) -> Vec<u8> {
        self.encoding.encode(value).0.into_owned()
    }
    pub fn EncodeFormData(&self, plain_text: bool) -> Vec<u8> {
        let mut output = Vec::new();
        for entry in &self.entries {
            let key = self.Encode(&entry.name);
            let value = self.Encode(entry.value.StringForFormData());
            if plain_text {
                output.extend(NormalizeBytes(&key));
                output.push(b'=');
                output.extend(NormalizeBytes(&value));
                output.extend(b"\r\n");
            } else {
                if !output.is_empty() {
                    output.push(b'&');
                }
                EncodeStringAsFormData(&mut output, &key, true);
                output.push(b'=');
                EncodeStringAsFormData(&mut output, &value, true);
            }
        }
        output
    }

    pub fn AppendString(&mut self, name: String, value: String) {
        self.entries.push(Entry {
            name,
            value: EntryValue::String(value),
            filename: None,
        });
    }
    pub fn AppendFile(&mut self, name: String, file: Arc<File>) {
        self.entries.push(Entry {
            name,
            value: EntryValue::File(file),
            filename: None,
        });
    }
    pub fn AppendFileWithFilename(
        &mut self,
        name: String,
        file: Arc<File>,
        filename: Option<String>,
    ) {
        self.entries.push(Entry {
            name,
            value: EntryValue::File(
                filename.map_or_else(|| file.clone(), |name| file.CloneWithName(name)),
            ),
            filename: None,
        });
    }
    pub fn SetFileWithFilename(&mut self, name: String, file: Arc<File>, filename: Option<String>) {
        let file = filename.map_or_else(|| file.clone(), |name| file.CloneWithName(name));
        self.Set(name, EntryValue::File(file));
    }
    pub fn EntryAt(&self, index: usize) -> Option<(String, EntryValue)> {
        self.entries
            .get(index)
            .map(|entry| (entry.name.clone(), entry.ValueForBindings()))
    }
    pub fn Entries(&self) -> &[Entry] {
        &self.entries
    }
    pub fn EncodeMultiPartFormData(
        &self,
        boundary: &str,
    ) -> Arc<crate::encoded_form_data::EncodedFormData> {
        let mut output = Vec::new();
        let mut elements = Vec::new();
        for entry in &self.entries {
            output.extend_from_slice(
                format!("--{boundary}\r\nContent-Disposition: form-data; name=\"").as_bytes(),
            );
            AppendQuotedBytes(&mut output, &self.Encode(&entry.name), true);
            output.push(b'"');
            if let EntryValue::File(file) = &entry.value {
                output.extend_from_slice(b"; filename=\"");
                let name = if file.webkitRelativePath().is_empty() {
                    file.name()
                } else {
                    file.webkitRelativePath()
                };
                AppendQuotedBytes(
                    &mut output,
                    &self.Encode(entry.filename.as_deref().unwrap_or(name)),
                    false,
                );
                output.extend_from_slice(b"\"\r\nContent-Type: ");
                output.extend_from_slice(if file.type_().is_empty() {
                    b"application/octet-stream"
                } else {
                    file.type_().as_bytes()
                });
            }
            output.extend_from_slice(b"\r\n\r\n");
            match &entry.value {
                EntryValue::String(value) => {
                    output.extend_from_slice(&self.Encode(&NormalizeLineEndingsToCRLF(value)))
                }
                EntryValue::File(file) => {
                    if !file.HasBackingFile() || !file.GetPath().is_empty() {
                        if !output.is_empty() {
                            elements.push(crate::encoded_form_data::EncodedFormDataElement::Data(
                                Arc::from(std::mem::take(&mut output)),
                            ));
                        }
                        elements.push(crate::encoded_form_data::EncodedFormDataElement::File(
                            file.clone(),
                        ));
                    }
                }
            }
            output.extend_from_slice(b"\r\n");
        }
        output.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        if !output.is_empty() {
            elements.push(crate::encoded_form_data::EncodedFormDataElement::Data(
                Arc::from(output),
            ));
        }
        crate::encoded_form_data::EncodedFormData::FromElements(elements)
    }
}
impl EntryValue {
    pub fn StringForFormData(&self) -> &str {
        match self {
            Self::String(value) => value,
            Self::File(file) => file.name(),
        }
    }
}
pub fn GenerateUniqueBoundaryString(random_bytes: [u8; 16]) -> String {
    const MAP: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789AB";
    let mut boundary = String::from("----WebKitFormBoundary");
    for byte in random_bytes {
        boundary.push(MAP[(byte & 0x3f) as usize] as char);
    }
    boundary
}
pub fn NormalizeLineEndingsToCRLF(value: &str) -> String {
    value
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n")
}
fn AppendQuotedBytes(output: &mut Vec<u8>, value: &[u8], normalize: bool) {
    let mut bytes = value.iter().copied().peekable();
    while let Some(byte) = bytes.next() {
        match byte {
            b'\n' => output.extend_from_slice(if normalize { b"%0D%0A" } else { b"%0A" }),
            b'\r' => {
                output.extend_from_slice(if normalize { b"%0D%0A" } else { b"%0D" });
                if normalize && bytes.peek() == Some(&b'\n') {
                    bytes.next();
                }
            }
            b'"' => output.extend_from_slice(b"%22"),
            byte => output.push(byte),
        }
    }
}

fn NormalizeBytes(value: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut bytes = value.iter().copied().peekable();
    while let Some(byte) = bytes.next() {
        match byte {
            b'\r' => {
                output.extend(b"\r\n");
                if bytes.peek() == Some(&b'\n') {
                    bytes.next();
                }
            }
            b'\n' => output.extend(b"\r\n"),
            byte => output.push(byte),
        }
    }
    output
}
/// FormDataEncoder.cc:269-310. This byte-level stage follows the form's
/// selected text codec and therefore cannot use a UTF-8 URL serializer.
pub fn EncodeStringAsFormData(output: &mut Vec<u8>, bytes: &[u8], normalize: bool) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let normalized;
    if normalize {
        normalized = NormalizeBytes(bytes);
    } else {
        normalized = bytes.to_vec();
    }
    for byte in normalized {
        match byte {
            b' ' => output.push(b'+'),
            b'*' | b'-' | b'.' | b'_' | b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' => {
                output.push(byte)
            }
            byte => output.extend([b'%', HEX[(byte >> 4) as usize], HEX[(byte & 15) as usize]]),
        }
    }
}

impl Entry {
    pub fn ValueForBindings(&self) -> EntryValue {
        match (&self.value, &self.filename) {
            (EntryValue::File(file), Some(name)) => {
                EntryValue::File(file.CloneWithName(name.clone()))
            }
            _ => self.value.clone(),
        }
    }
}
