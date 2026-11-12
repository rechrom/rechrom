/*
 * Copyright (C) 2011 Google Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 *     * Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above
 * copyright notice, this list of conditions and the following disclaimer
 * in the documentation and/or other materials provided with the
 * distribution.
 *     * Neither the name of Google Inc. nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: data_object_item.h:49-56,64-72,83-94,113-124;
// data_object_item.cc:47-61,77-112,127-132,146-169,199-203,253-255.
// Internal and clipboard source branches retain their original data owners;
// filesystem admission remains with that owner.
use crate::core::fileapi::file::File;
use std::{rc::Rc, sync::Arc};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    kStringKind,
    kFileKind,
}
/// The original SystemClipboard data owner. These are native data reads, not
/// command/event/DOM callbacks. Item reads validate its actual sequence token.
pub trait ClipboardDataSource {
    fn SequenceNumber(&self) -> u64;
    fn ReadPlainText(&self) -> foundation::String;
    fn ReadRTF(&self) -> foundation::String;
    fn ReadHTML(&self) -> foundation::String;
    fn ReadCustomData(&self, kind: &str) -> foundation::String;
    fn ReadPNG(&self) -> Arc<[u8]>;
}
enum ItemData {
    String(foundation::String),
    Clipboard {
        source: Rc<dyn ClipboardDataSource>,
        sequence: u64,
        kind: ItemKind,
    },
    File(Arc<File>),
    SharedBuffer(Arc<DragDataFile>),
}
pub struct DataObjectItem {
    type_: String,
    data_: ItemData,
    title_: Option<String>,
    base_url_: Option<String>,
    file_system_id_: String,
}
impl DataObjectItem {
    // data_object_item.cc:114-144. The item retains SystemClipboard itself;
    // listener reentry must not freeze a copied string/file at construction.
    pub fn CreateFromClipboard(
        source: Rc<dyn ClipboardDataSource>,
        content_type: &str,
        sequence: u64,
    ) -> Rc<Self> {
        Rc::new(Self {
            type_: content_type.into(),
            data_: ItemData::Clipboard {
                source,
                sequence,
                kind: if content_type == "image/png" {
                    ItemKind::kFileKind
                } else {
                    ItemKind::kStringKind
                },
            },
            title_: None,
            base_url_: None,
            file_system_id_: String::new(),
        })
    }
    pub fn CreateFromString(content_type: &str, data: &str) -> Rc<Self> {
        Rc::new(Self {
            type_: content_type.into(),
            data_: ItemData::String(data.into()),
            title_: None,
            base_url_: None,
            file_system_id_: String::new(),
        })
    }
    pub fn CreateFromStringUTF16(content_type: &str, data: &[u16]) -> Rc<Self> {
        Rc::new(Self {
            type_: content_type.into(),
            data_: ItemData::String(foundation::String::from_utf16(data)),
            title_: None,
            base_url_: None,
            file_system_id_: String::new(),
        })
    }
    pub fn CreateFromFile(file: Arc<File>) -> Rc<Self> {
        Rc::new(Self {
            type_: file.type_().into(),
            data_: ItemData::File(file),
            title_: None,
            base_url_: None,
            file_system_id_: String::new(),
        })
    }
    // Source data_object_item.cc:64-75. Identity belongs to this same item.
    pub fn CreateFromFileWithFileSystemId(file: Arc<File>, file_system_id: String) -> Rc<Self> {
        Rc::new(Self {
            type_: file.type_().into(),
            data_: ItemData::File(file),
            title_: None,
            base_url_: None,
            file_system_id_: file_system_id,
        })
    }
    pub fn IsFilename(&self) -> bool {
        matches!(self.data_, ItemData::File(_))
    }
    pub fn HasFileSystemId(&self) -> bool {
        self.Kind() == ItemKind::kFileKind && !self.file_system_id_.is_empty()
    }
    pub fn FileSystemId(&self) -> &str {
        &self.file_system_id_
    }
    pub fn CreateFromURL(url: &str, title: &str) -> Rc<Self> {
        Rc::new(Self {
            type_: "text/uri-list".into(),
            data_: ItemData::String(url.into()),
            title_: Some(title.into()),
            base_url_: None,
            file_system_id_: String::new(),
        })
    }
    pub fn CreateFromHTML(html: &str, base_url: &str) -> Rc<Self> {
        Rc::new(Self {
            type_: "text/html".into(),
            data_: ItemData::String(html.into()),
            title_: None,
            base_url_: Some(base_url.into()),
            file_system_id_: String::new(),
        })
    }
    pub fn CreateFromHTMLUTF16(html: &[u16], base_url: &str) -> Rc<Self> {
        Rc::new(Self {
            type_: "text/html".into(),
            data_: ItemData::String(foundation::String::from_utf16(html)),
            title_: None,
            base_url_: Some(base_url.into()),
            file_system_id_: String::new(),
        })
    }
    pub fn CreateFromFileSharedBuffer(file: Arc<DragDataFile>) -> Rc<Self> {
        Rc::new(Self {
            type_: super::mime_type_registry::GetWellKnownMIMETypeForExtension(
                &file.filename_extension,
            )
            .into(),
            data_: ItemData::SharedBuffer(file),
            title_: None,
            base_url_: None,
            file_system_id_: String::new(),
        })
    }
    pub fn Kind(&self) -> ItemKind {
        match &self.data_ {
            ItemData::String(_) => ItemKind::kStringKind,
            ItemData::Clipboard { kind, .. } => *kind,
            _ => ItemKind::kFileKind,
        }
    }
    pub fn GetType(&self) -> &str {
        &self.type_
    }
    pub fn GetAsString(&self) -> foundation::String {
        match &self.data_ {
            ItemData::String(data) => data.clone(),
            ItemData::Clipboard {
                source,
                sequence,
                kind,
            } => {
                assert!(*kind == ItemKind::kStringKind);
                let data = match self.type_.as_str() {
                    "text/plain" => source.ReadPlainText(),
                    "text/rtf" => source.ReadRTF(),
                    "text/html" => source.ReadHTML(),
                    kind => source.ReadCustomData(kind),
                };
                if source.SequenceNumber() == *sequence {
                    data
                } else {
                    foundation::String::default()
                }
            }
            _ => unreachable!("GetAsString requires a string item"),
        }
    }
    pub fn GetAsFile(&self) -> Option<Arc<File>> {
        match &self.data_ {
            ItemData::String(_) => None,
            ItemData::Clipboard {
                source,
                sequence,
                kind,
            } => {
                if *kind != ItemKind::kFileKind || source.SequenceNumber() != *sequence {
                    return None;
                }
                if self.type_ == "image/png" {
                    Some(File::Create(
                        "image.png".into(),
                        "image/png".into(),
                        File::NowMilliseconds(),
                        source.ReadPNG(),
                    ))
                } else {
                    None
                }
            }
            ItemData::File(file) => Some(file.clone()),
            ItemData::SharedBuffer(file) => {
                if !file.is_image_accessible {
                    return None;
                }
                Some(File::Create(
                    file.filename.clone(),
                    self.type_.clone(),
                    File::NowMilliseconds(),
                    file.bytes.clone(),
                ))
            }
        }
    }
    pub fn Title(&self) -> Option<&str> {
        match &self.data_ {
            ItemData::SharedBuffer(file) => Some(&file.content_disposition),
            _ => self.title_.as_deref(),
        }
    }
    pub fn BaseURL(&self) -> Option<&str> {
        match &self.data_ {
            ItemData::SharedBuffer(file) => Some(&file.source_url),
            _ => self.base_url_.as_deref(),
        }
    }
    // Source: data_object_item.cc:253-255, Trace(file_). Do not call
    // GetAsFile for tracing: shared-buffer items create a new File on each call.
    pub fn FileReference(&self) -> Option<Arc<File>> {
        match &self.data_ {
            ItemData::File(file) => Some(file.clone()),
            _ => None,
        }
    }
    pub fn GetSharedBuffer(&self) -> Option<Arc<DragDataFile>> {
        match &self.data_ {
            ItemData::SharedBuffer(file) => Some(file.clone()),
            _ => None,
        }
    }
}

// Source: data_object_item.cc:96-112,146-169. These shared-buffer fields
// and decoded filename feed the canonical item owner, never DataTransfer state.
/// Native source image bytes and response metadata supplied by the resource
/// owner. A platform file serialization never refetches the source URL.
pub struct DragDataFile {
    pub bytes: Arc<[u8]>,
    pub is_image_accessible: bool,
    pub source_url: String,
    // KURL::LastPathComponent + DecodeUrlEscapeSequences(kUtf8OrIsomorphic),
    // supplied by the URL/resource owner before encoded bytes cross this edge.
    pub filename: String,
    pub filename_extension: String,
    pub content_disposition: String,
}
impl DragDataFile {
    /// DataObjectItem::CreateFromFileSharedBuffer uses KURL's decoded final
    /// path component; filesystem names and native promises stay host-owned.
    pub fn FromImageResource(
        bytes: Arc<[u8]>,
        is_image_accessible: bool,
        source_url: String,
        filename_extension: String,
        content_disposition: String,
    ) -> Arc<Self> {
        let filename = url::Url::parse(&source_url)
            .ok()
            .and_then(|url| {
                url.path_segments()
                    .and_then(|mut segments| segments.next_back())
                    .map(str::to_owned)
            })
            .map(|component| {
                foundation::url_escape::DecodeUrlEscapeSequencesUtf8OrIsomorphic(&component)
            })
            .unwrap_or_default();
        Arc::new(Self {
            bytes,
            is_image_accessible,
            source_url,
            filename,
            filename_extension,
            content_disposition,
        })
    }
}
