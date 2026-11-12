//! Chromium platform/network/encoded_form_data.{h,cc}: data/blob/file elements.
//! Native file identity and expected snapshot stay on the original File owner;
//! encoding never synchronously reads a native path on the document sequence.
use crate::{native_file::NativeFileError, File};
use std::sync::Arc;
#[derive(Clone)]
pub enum EncodedFormDataElement {
    Data(Arc<[u8]>),
    File(Arc<File>),
}
impl std::fmt::Debug for EncodedFormDataElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Data(data) => f.debug_tuple("Data").field(&data.len()).finish(),
            Self::File(file) => f.debug_tuple("File").field(&file.name()).finish(),
        }
    }
}
#[derive(Debug)]
pub struct EncodedFormData {
    elements: Vec<EncodedFormDataElement>,
    identifier: i64,
    contains_password_data: bool,
}
impl EncodedFormData {
    pub fn FromBytes(bytes: impl Into<Arc<[u8]>>) -> Arc<Self> {
        Arc::new(Self {
            elements: vec![EncodedFormDataElement::Data(bytes.into())],
            identifier: 0,
            contains_password_data: false,
        })
    }
    pub fn FromElements(elements: Vec<EncodedFormDataElement>) -> Arc<Self> {
        Arc::new(Self {
            elements,
            identifier: 0,
            contains_password_data: false,
        })
    }
    pub fn WithSubmissionMetadata(
        mut self: Arc<Self>,
        identifier: i64,
        contains_password_data: bool,
    ) -> Arc<Self> {
        let body = Arc::get_mut(&mut self)
            .expect("set source submission metadata before publishing EncodedFormData");
        body.identifier = identifier;
        body.contains_password_data = contains_password_data;
        self
    }
    pub fn Elements(&self) -> &[EncodedFormDataElement] {
        &self.elements
    }
    pub fn Identifier(&self) -> i64 {
        self.identifier
    }
    pub fn ContainsPasswordData(&self) -> bool {
        self.contains_password_data
    }
    pub fn FlattenDataOnly(&self) -> Option<Arc<[u8]>> {
        if self.elements.len() == 1 {
            if let EncodedFormDataElement::Data(bytes) = &self.elements[0] {
                return Some(bytes.clone());
            }
        }
        let mut bytes = Vec::new();
        for element in &self.elements {
            match element {
                EncodedFormDataElement::Data(data) => bytes.extend_from_slice(data),
                EncodedFormDataElement::File(_) => return None,
            }
        }
        Some(Arc::from(bytes))
    }
    /// Resource/network admission runs this outside document compute. File's
    /// injected reader performs real I/O and validates its original snapshot;
    /// no DOM or script callback crosses into that completion thread.
    pub fn ReadAll(
        self: &Arc<Self>,
        complete: Box<dyn FnOnce(Result<Arc<[u8]>, NativeFileError>) + Send>,
    ) {
        ReadPart(self.clone(), 0, Vec::new(), complete);
    }
}
fn ReadPart(
    owner: Arc<EncodedFormData>,
    mut index: usize,
    mut bytes: Vec<u8>,
    complete: Box<dyn FnOnce(Result<Arc<[u8]>, NativeFileError>) + Send>,
) {
    while let Some(part) = owner.elements.get(index) {
        match part {
            EncodedFormDataElement::Data(data) => {
                bytes.extend_from_slice(data);
                index += 1;
            }
            EncodedFormDataElement::File(file) => {
                let file = file.clone();
                file.ReadBytes(Box::new(move |result| match result {
                    Ok(data) => {
                        bytes.extend_from_slice(&data);
                        ReadPart(owner, index + 1, bytes, complete);
                    }
                    Err(error) => complete(Err(error)),
                }));
                return;
            }
        }
    }
    complete(Ok(Arc::from(bytes)));
}
