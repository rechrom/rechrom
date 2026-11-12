/*
 * Copyright (c) 2008, 2009, 2012 Google Inc. All rights reserved.
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
// Source: data_object.h:59-71,84-98; data_object.cc:125-179,192-278,318-365.
use super::{
    data_object_item::{DataObjectItem, ItemKind},
    data_transfer::{DragDataFile, WebDragData, WebDragFileSystemFile},
};
use crate::core::fileapi::file::File;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    sync::Arc,
};
/// ExecutionContext's source feature and real ImageDecoder factory. File and
/// Blob reads retain their canonical owning File API values below.
pub trait WebDragDataContext {
    fn DragAndDropJSFileObjectsEnabled(&self) -> bool;
    fn IsImageDataValid(&self, data: Arc<[u8]>) -> bool;
}
pub trait DataObjectObserver {
    fn OnItemListChanged(&self);
}
#[derive(Default)]
pub struct DragDataObject {
    item_list_: RefCell<Vec<Rc<DataObjectItem>>>,
    observers_: RefCell<Vec<Weak<dyn DataObjectObserver>>>,
    pub source_effect_allowed: RefCell<Option<String>>,
}
impl DragDataObject {
    /// DataObject::Create(WebDragData), data_object.cc:373-447. Values are
    /// imported once; all target DataTransfer wrappers borrow this live owner.
    pub fn FromWebDragData(data: &WebDragData) -> Result<Rc<Self>, &'static str> {
        let result = Rc::new(Self::default());
        *result.source_effect_allowed.borrow_mut() = data.source_effect_allowed.clone();
        for (kind, value) in &data.strings {
            if kind == "text/html" {
                result.SetHTMLAndBaseURL(value, data.html_base_url.as_deref().unwrap_or(""));
            } else if kind == "text/uri-list" {
                result.SetURLAndTitle(value, data.url_title.as_deref().unwrap_or(""));
            } else {
                result.SetData(kind, value);
            }
        }
        for (kind, value) in &data.utf16_strings {
            if kind == "text/html" {
                result.SetHTMLAndBaseURLUTF16(value, data.html_base_url.as_deref().unwrap_or(""));
            } else {
                result.SetDataUTF16(kind, value);
            }
        }
        for file in &data.files {
            result.AddFileSharedBuffer(file.clone());
        }
        for file in &data.file_items {
            result.AddFile(file.clone());
        }
        for item in &data.file_system_files {
            if item.file.FileSystemURL() != item.url {
                return Err("filesystem drag URL differs from its retained File owner");
            }
            result.AddFileWithFileSystemId(item.file.clone(), item.file_system_id.clone());
        }
        Ok(result)
    }

    pub fn length(&self) -> u32 {
        self.item_list_.borrow().len() as u32
    }
    pub fn Item(&self, index: u32) -> Option<Rc<DataObjectItem>> {
        self.item_list_.borrow().get(index as usize).cloned()
    }
    pub fn DeleteItem(&self, index: u32) {
        let mut items = self.item_list_.borrow_mut();
        if index as usize >= items.len() {
            return;
        }
        items.remove(index as usize);
        drop(items);
        self.NotifyItemListChanged();
    }
    pub fn ClearStringItems(&self) {
        let mut items = self.item_list_.borrow_mut();
        let before = items.len();
        items.retain(|item| item.Kind() != ItemKind::kStringKind);
        let changed = before != items.len();
        drop(items);
        if changed {
            self.NotifyItemListChanged();
        }
    }
    pub fn ClearAll(&self) {
        let mut items = self.item_list_.borrow_mut();
        if items.is_empty() {
            return;
        }
        items.clear();
        drop(items);
        self.NotifyItemListChanged();
    }
    pub fn Add(&self, data: &str, content_type: &str) -> Option<Rc<DataObjectItem>> {
        self.AddUTF16(&data.encode_utf16().collect::<Vec<_>>(), content_type)
    }
    pub fn AddUTF16(&self, data: &[u16], content_type: &str) -> Option<Rc<DataObjectItem>> {
        let item = DataObjectItem::CreateFromStringUTF16(content_type, data);
        if !self.InternalAddStringItem(item.clone()) {
            return None;
        }
        Some(item)
    }
    pub fn SetDataUTF16(&self, content_type: &str, data: &[u16]) {
        self.ClearData(content_type);
        assert!(
            self.InternalAddStringItem(DataObjectItem::CreateFromStringUTF16(content_type, data))
        );
    }
    pub fn SetHTMLAndBaseURLUTF16(&self, html: &[u16], base_url: &str) {
        self.ClearData("text/html");
        assert!(self.InternalAddStringItem(DataObjectItem::CreateFromHTMLUTF16(html, base_url)));
    }
    pub fn AddFile(&self, file: Arc<File>) -> Rc<DataObjectItem> {
        let item = DataObjectItem::CreateFromFile(file);
        self.InternalAddFileItem(item.clone());
        item
    }
    pub fn AddFileWithFileSystemId(&self, file: Arc<File>, id: String) -> Rc<DataObjectItem> {
        let item = DataObjectItem::CreateFromFileWithFileSystemId(file, id);
        self.InternalAddFileItem(item.clone());
        item
    }
    pub fn ClearData(&self, content_type: &str) {
        let mut items = self.item_list_.borrow_mut();
        let Some(index) = items.iter().position(|item| {
            item.Kind() == ItemKind::kStringKind && item.GetType() == content_type
        }) else {
            return;
        };
        items.remove(index);
        drop(items);
        self.NotifyItemListChanged();
    }
    pub fn Types(&self) -> Vec<String> {
        let mut results = Vec::new();
        let mut contains_files = false;
        for item in self.item_list_.borrow().iter() {
            match item.Kind() {
                ItemKind::kStringKind => results.push(item.GetType().into()),
                ItemKind::kFileKind => contains_files = true,
            }
        }
        if contains_files {
            results.push("Files".into());
        }
        results
    }
    pub fn GetData(&self, content_type: &str) -> foundation::String {
        self.FindStringItem(content_type)
            .map_or_else(foundation::String::default, |item| item.GetAsString())
    }
    pub fn SetData(&self, content_type: &str, data: &str) {
        self.ClearData(content_type);
        assert!(self.Add(data, content_type).is_some());
    }
    pub fn SetURLAndTitle(&self, url: &str, title: &str) {
        self.ClearData("text/uri-list");
        assert!(self.InternalAddStringItem(DataObjectItem::CreateFromURL(url, title)));
    }
    pub fn SetHTMLAndBaseURL(&self, html: &str, base_url: &str) {
        self.ClearData("text/html");
        assert!(self.InternalAddStringItem(DataObjectItem::CreateFromHTML(html, base_url)));
    }
    // data_object.cc:62-103. MIME discovery belongs to SystemClipboard; each
    // retained item performs the source sequence check when its data is read.
    pub fn AddFromClipboard(
        &self,
        source: Rc<dyn super::data_object_item::ClipboardDataSource>,
        kind: &str,
        sequence: u64,
    ) {
        let item = DataObjectItem::CreateFromClipboard(source, kind, sequence);
        if item.Kind() == ItemKind::kFileKind {
            self.InternalAddFileItem(item);
        } else {
            assert!(self.InternalAddStringItem(item));
        }
    }
    pub fn AddFileSharedBuffer(&self, file: Arc<DragDataFile>) {
        self.InternalAddFileItem(DataObjectItem::CreateFromFileSharedBuffer(file));
    }
    fn FindStringItem(&self, content_type: &str) -> Option<Rc<DataObjectItem>> {
        self.item_list_
            .borrow()
            .iter()
            .find(|item| item.Kind() == ItemKind::kStringKind && item.GetType() == content_type)
            .cloned()
    }
    fn InternalAddStringItem(&self, new_item: Rc<DataObjectItem>) -> bool {
        debug_assert!(new_item.Kind() == ItemKind::kStringKind);
        if self.FindStringItem(new_item.GetType()).is_some() {
            return false;
        }
        self.item_list_.borrow_mut().push(new_item);
        self.NotifyItemListChanged();
        true
    }
    fn InternalAddFileItem(&self, new_item: Rc<DataObjectItem>) {
        debug_assert!(new_item.Kind() == ItemKind::kFileKind);
        self.item_list_.borrow_mut().push(new_item);
        self.NotifyItemListChanged();
    }
    pub fn AddObserver(&self, observer: Weak<dyn DataObjectObserver>) {
        self.observers_.borrow_mut().push(observer);
    }
    fn NotifyItemListChanged(&self) {
        // Upgrade observer identities, never copy the item tree. End the
        // registry borrow before calling the original DataTransfer observer.
        let observers: Vec<_> = self
            .observers_
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .collect();
        self.observers_
            .borrow_mut()
            .retain(|observer| observer.strong_count() != 0);
        for observer in observers {
            observer.OnItemListChanged();
        }
    }
    // Source data_object.cc:500-601. The output is the native immutable packet.
    pub fn ToWebDragData(&self, context: Option<&dyn WebDragDataContext>) -> WebDragData {
        let mut result = WebDragData {
            strings: Vec::new(),
            utf16_strings: Vec::new(),
            source_effect_allowed: self.source_effect_allowed.borrow().clone(),
            html_base_url: None,
            url_title: None,
            files: Vec::new(),
            file_items: Vec::new(),
            file_system_files: Vec::new(),
        };
        for item in self.item_list_.borrow().iter() {
            match item.Kind() {
                ItemKind::kStringKind => {
                    let value = item.GetAsString();
                    let units = value.Span16().unwrap_or_default();
                    match String::from_utf16(units) {
                        Ok(value) => result.strings.push((item.GetType().into(), value)),
                        Err(_) => result
                            .utf16_strings
                            .push((item.GetType().into(), Arc::from(units))),
                    }
                    if item.GetType() == "text/html" {
                        result.html_base_url = item.BaseURL().map(str::to_owned);
                    }
                    if item.GetType() == "text/uri-list" {
                        result.url_title = item.Title().map(str::to_owned);
                    }
                }
                ItemKind::kFileKind => {
                    if let Some(buffer) = item.GetSharedBuffer() {
                        result.files.push(buffer);
                    } else if item.IsFilename() {
                        let file = item
                            .GetAsFile()
                            .expect("source filename item owns its File");
                        if file.HasBackingFile() {
                            result.file_items.push(file);
                        } else if !file.FileSystemURL().is_empty() {
                            result.file_system_files.push(WebDragFileSystemFile {
                                file: file.clone(),
                                url: file.FileSystemURL().to_owned(),
                                size: file.size() as u64,
                                file_system_id: item.FileSystemId().to_owned(),
                            });
                        } else {
                            let buffer = context
                                .filter(|context| context.DragAndDropJSFileObjectsEnabled())
                                .and_then(|_| {
                                    fileapi::synced_blob_reader::SyncReadBlobDataHandle(
                                        file.GetBlobDataHandle(),
                                    )
                                });
                            if let Some(buffer) = buffer.filter(|bytes| {
                                !bytes.is_empty()
                                    && context.is_some_and(|context| {
                                        context.IsImageDataValid(bytes.clone())
                                    })
                            }) {
                                let (source_url, content_disposition, filename) =
                                    if file.name().is_empty() {
                                        (String::new(), String::new(), String::new())
                                    } else {
                                        let escaped =
                                            foundation::url_escape::EncodeWithUrlEscapeSequences(
                                                file.name(),
                                            );
                                        let url =
                                            url::Url::parse(&format!("https://local/{escaped}"))
                                                .expect("source generated local URL");
                                        let last_component = url
                                            .path_segments()
                                            .and_then(|mut segments| segments.next_back())
                                            .unwrap_or("");
                                        let filename = foundation::url_escape::
                                        DecodeUrlEscapeSequencesUtf8OrIsomorphic(last_component);
                                        let quoted =
                                            file.name().replace('\\', "\\\\").replace('"', "\\\"");
                                        (
                                            url.to_string(),
                                            format!("attachment; filename=\"{quoted}\""),
                                            filename,
                                        )
                                    };
                                let extension = file
                                    .name()
                                    .rfind('.')
                                    .filter(|&dot| dot + 1 < file.name().len())
                                    .map_or("", |dot| &file.name()[dot + 1..]);
                                result.files.push(Arc::new(DragDataFile {
                                    bytes: buffer,
                                    is_image_accessible: true,
                                    source_url,
                                    filename,
                                    filename_extension: extension.into(),
                                    content_disposition,
                                }));
                            } else {
                                result
                                    .strings
                                    .push(("text/plain".into(), file.name().to_owned()));
                            }
                        }
                    } else {
                        unreachable!("source file item is SharedBuffer or Filename");
                    }
                }
            }
        }
        result
    }
}
