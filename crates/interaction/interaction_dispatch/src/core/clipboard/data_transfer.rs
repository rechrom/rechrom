/*
 * Copyright (C) 2006, 2007, 2008 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE COMPUTER, INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE COMPUTER, INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
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
// Source: core/clipboard/data_transfer.cc:172-267,273-369,399-445,547-704.
// Scope: native drag data, item/FileList observers, operation/access/image owners.
// File item admission and image rasterization belong to the supplied resource
// and Paint owners; their retained values never become DOM or Layout copies.
use crate::core::dom::events::event::EventTargetRef;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataTransferAccessPolicy {
    Numb,
    TypesReadable,
    Readable,
    Writable,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum DragOperation {
    #[default]
    None = 0,
    Copy = 1,
    Link = 2,
    Move = 16,
}
pub const DRAG_OPERATION_EVERY: u32 = u32::MAX;

fn EffectMask(effect: &str) -> Option<u32> {
    Some(match effect {
        "uninitialized" | "all" => DRAG_OPERATION_EVERY,
        "none" => 0,
        "copy" => 1,
        "link" => 2,
        "move" => 16,
        "copyLink" => 3,
        "copyMove" => 17,
        "linkMove" => 18,
        _ => return None,
    })
}
fn NormalizeType(kind: &str) -> (String, bool) {
    let kind = kind.trim().to_ascii_lowercase();
    if kind == "text" || kind.starts_with("text/plain;") {
        ("text/plain".into(), false)
    } else if kind == "url" {
        ("text/uri-list".into(), true)
    } else {
        (kind, false)
    }
}
// The public name remains at the existing native assembly boundary.
pub use super::data_object::DragDataObject;
use super::{
    data_object::DataObjectObserver, data_object_item::ItemKind,
    data_transfer_item_list::DataTransferItemList,
};
use crate::core::fileapi::{file::File, file_list::FileList};
// Preserve the existing public packet path for native assembly.
pub use super::data_object_item::DragDataFile;
/// Source DataObject::ToWebDragData serialization. The platform receives an
/// immutable value packet and shared byte resources; the live DataObject owner
/// remains on the document sequence with DragEvent/DataTransfer.
#[derive(Clone)]
pub struct WebDragFileSystemFile {
    /// The same canonical File/Blob backing supplied by the source or native
    /// target host. A URL and metadata alone cannot grant file contents.
    pub file: Arc<File>,
    pub url: String,
    pub size: u64,
    pub file_system_id: String,
}
#[derive(Clone)]
pub struct WebDragData {
    pub strings: Vec<(String, String)>,
    pub utf16_strings: Vec<(String, Arc<[u16]>)>,
    pub source_effect_allowed: Option<String>,
    pub html_base_url: Option<String>,
    pub url_title: Option<String>,
    pub files: Vec<Arc<DragDataFile>>,
    /// Source FilenameItem only; Blob/FileSystem branches are admitted below.
    pub file_items: Vec<Arc<File>>,
    pub file_system_files: Vec<WebDragFileSystemFile>,
}
/// Immutable image resource identity; a resource implementation retains its
/// original pixels. Raster or host services must provide the image payload.
pub struct DragImageBitmap {
    pub width: u32,
    pub height: u32,
    pub row_bytes: usize,
    pub rgba8: Arc<Vec<u8>>,
}
pub trait DragImageResource {
    fn Bitmap(&self) -> Option<Arc<DragImageBitmap>>;
    /// Source Create + Scale happens before the native bitmap packet is
    /// admitted, preserving its original premultiplied resource pixels.
    fn BitmapScaled(&self, device_scale: f32) -> Option<Arc<DragImageBitmap>>;
}
#[derive(Clone)]
pub enum DragImageSource {
    Node(EventTargetRef),
    Resource(Rc<dyn DragImageResource>),
}
#[derive(Clone)]
pub struct DragImage {
    pub source: DragImageSource,
    pub location: (i32, i32),
}

pub struct DataTransfer {
    drag_and_drop: bool,
    policy: Cell<DataTransferAccessPolicy>,
    data_object: Rc<DragDataObject>,
    effect_allowed: RefCell<String>,
    drop_effect: Cell<Option<DragOperation>>,
    drag_image: RefCell<Option<DragImage>>,
    files_: Rc<FileList>,
    data_store_item_list_changed_: Cell<bool>,
}
/// LocalFrame/VisualViewport coordinate leaves of DataTransfer.cc442-449.
/// The original active viewport supplies its VisibleRect, never a LayoutView
/// or document-root scroll approximation.
pub trait DragVisualViewportClip {
    fn VisibleRect(&self) -> foundation::gfx::RectF;
    fn ConvertRectFromRootFrame(&self, rect: foundation::gfx::Rect) -> foundation::gfx::Rect;
}
impl DataTransfer {
    pub fn ClipByVisualViewport(
        absolute_rect: foundation::gfx::RectF,
        frame: &impl DragVisualViewportClip,
    ) -> foundation::gfx::RectF {
        let viewport = foundation::gfx::ToEnclosingRect(frame.VisibleRect());
        let mut viewport = foundation::gfx::RectF::from(frame.ConvertRectFromRootFrame(viewport));
        viewport.Intersect(absolute_rect);
        viewport
    }
    /// DragController creates a fresh access-policy wrapper around the same
    /// imported platform DataObject for each target event transaction.
    pub fn CreateDragTarget(
        data: Rc<DragDataObject>,
        policy: DataTransferAccessPolicy,
        source_operations: u32,
    ) -> Rc<Self> {
        let effect = data
            .source_effect_allowed
            .borrow()
            .clone()
            .filter(|effect| EffectMask(effect).is_some())
            .unwrap_or_else(|| {
                match source_operations {
                    0 => "none",
                    1 => "copy",
                    2 => "link",
                    3 => "copyLink",
                    16 => "move",
                    17 => "copyMove",
                    18 => "linkMove",
                    _ => "all",
                }
                .into()
            });
        let transfer = Rc::new(Self {
            drag_and_drop: true,
            policy: Cell::new(policy),
            data_object: data,
            effect_allowed: RefCell::new(effect),
            drop_effect: Cell::new(None),
            drag_image: RefCell::new(None),
            files_: Rc::default(),
            data_store_item_list_changed_: Cell::new(true),
        });
        transfer.ObserveDataObject();
        transfer
    }
    pub fn CreateWritableDragAndDrop() -> Rc<Self> {
        let transfer = Rc::new(Self {
            drag_and_drop: true,
            policy: Cell::new(DataTransferAccessPolicy::Writable),
            data_object: Rc::default(),
            effect_allowed: RefCell::new("uninitialized".into()),
            drop_effect: Cell::new(None),
            drag_image: RefCell::new(None),
            files_: Rc::default(),
            data_store_item_list_changed_: Cell::new(true),
        });
        transfer.ObserveDataObject();
        transfer
    }
    pub fn Create() -> Rc<Self> {
        let transfer = Rc::new(Self {
            drag_and_drop: false,
            policy: Cell::new(DataTransferAccessPolicy::Writable),
            data_object: Rc::default(),
            effect_allowed: RefCell::new("none".into()),
            drop_effect: Cell::new(Some(DragOperation::None)),
            drag_image: RefCell::new(None),
            files_: Rc::default(),
            data_store_item_list_changed_: Cell::new(true),
        });
        transfer.ObserveDataObject();
        transfer
    }
    pub fn IsForDragAndDrop(&self) -> bool {
        self.drag_and_drop
    }
    pub fn GetDataObject(&self) -> Rc<DragDataObject> {
        self.data_object.clone()
    }
    pub fn CanReadTypes(&self) -> bool {
        self.policy.get() != DataTransferAccessPolicy::Numb
    }
    pub fn CanReadData(&self) -> bool {
        matches!(
            self.policy.get(),
            DataTransferAccessPolicy::Readable | DataTransferAccessPolicy::Writable
        )
    }
    pub fn CanWriteData(&self) -> bool {
        self.policy.get() == DataTransferAccessPolicy::Writable
    }
    pub fn SetAccessPolicy(&self, policy: DataTransferAccessPolicy) {
        assert!(
            self.policy.get() != DataTransferAccessPolicy::Numb
                || policy == DataTransferAccessPolicy::Numb,
            "a numb DataTransfer cannot regain access"
        );
        self.policy.set(policy);
    }
    pub fn SetAccessPolicyNumb(&self) {
        self.SetAccessPolicy(DataTransferAccessPolicy::Numb);
    }
    pub fn SetAccessPolicyTypesReadable(&self) {
        self.SetAccessPolicy(DataTransferAccessPolicy::TypesReadable);
    }
    pub fn types(&self) -> Vec<String> {
        if self.CanReadTypes() {
            self.data_store_item_list_changed_.set(false);
        }
        if !self.CanReadTypes() {
            return Vec::new();
        }
        self.data_object.Types()
    }
    // Source: data_transfer.cc:370-372. Cached IDL types are invalidated by
    // native item-list notifications, and never reused under the numb policy.
    pub fn hasDataStoreItemListChanged(&self) -> bool {
        self.data_store_item_list_changed_.get() || !self.CanReadTypes()
    }
    pub fn getData(&self, kind: &str) -> foundation::String {
        if !self.CanReadData() {
            return foundation::String::default();
        }
        let (kind, url) = NormalizeType(kind);
        let value = self.data_object.GetData(&kind);
        if !url {
            return value;
        }
        super::clipboard_utilities::ConvertURIListToURL(&value)
    }
    pub fn setDataUTF16(&self, kind: &str, value: &[u16]) {
        if self.CanWriteData() {
            let (kind, _) = NormalizeType(kind);
            self.data_object.SetDataUTF16(&kind, value);
        }
    }
    pub fn setData(&self, kind: &str, value: &str) {
        if self.CanWriteData() {
            self.data_object.SetData(&NormalizeType(kind).0, value);
        }
    }
    pub fn clearData(&self, kind: Option<&str>) {
        if !self.CanWriteData() {
            return;
        }
        match kind {
            None => self.data_object.ClearStringItems(),
            Some(kind) => self.data_object.ClearData(&NormalizeType(kind).0),
        }
    }
    // Source: data_transfer.cc:400-405,660-665. The pinned revision creates
    // each ItemList wrapper over the same store; FileList has retained identity.
    pub fn items(self: &Rc<Self>) -> Rc<DataTransferItemList> {
        DataTransferItemList::new(self.clone(), self.data_object.clone())
    }
    pub fn files(&self) -> Rc<FileList> {
        if !self.CanReadData() {
            self.files_.clear();
        }
        self.files_.clone()
    }
    // Source: data_transfer.cc:698-703. The tracing path reads the retained
    // member without invoking the script getter's access-policy clearing.
    pub fn FileListForTracing(&self) -> Rc<FileList> {
        self.files_.clone()
    }
    // Source: data_transfer.cc:672-684; data_object.cc:357-364. Weak observers
    // break the Rust ownership cycle while the transfer retains the store.
    fn ObserveDataObject(self: &Rc<Self>) {
        let observer: Rc<dyn DataObjectObserver> = self.clone();
        self.data_object.AddObserver(Rc::downgrade(&observer));
        self.OnItemListChanged();
    }

    pub fn effectAllowed(&self) -> String {
        self.effect_allowed.borrow().clone()
    }
    pub fn setEffectAllowed(&self, effect: &str) {
        if self.drag_and_drop && self.CanWriteData() && EffectMask(effect).is_some() {
            *self.effect_allowed.borrow_mut() = effect.into();
            *self.data_object.source_effect_allowed.borrow_mut() = Some(effect.into());
        }
    }
    pub fn SourceOperation(&self) -> u32 {
        EffectMask(&self.effect_allowed.borrow()).expect("validated effectAllowed")
    }
    pub fn DropEffectIsInitialized(&self) -> bool {
        self.drop_effect.get().is_some()
    }
    pub fn DestinationOperation(&self) -> DragOperation {
        self.drop_effect
            .get()
            .expect("dropEffect initialized before reading operation")
    }
    pub fn dropEffect(&self) -> &'static str {
        match self.drop_effect.get().unwrap_or_default() {
            DragOperation::None => "none",
            DragOperation::Copy => "copy",
            DragOperation::Link => "link",
            DragOperation::Move => "move",
        }
    }
    pub fn setDropEffect(&self, effect: &str) {
        if !self.drag_and_drop {
            return;
        }
        if let Some(effect) = match effect {
            "none" => Some(DragOperation::None),
            "copy" => Some(DragOperation::Copy),
            "link" => Some(DragOperation::Link),
            "move" => Some(DragOperation::Move),
            _ => None,
        } {
            self.drop_effect.set(Some(effect));
        }
    }
    pub fn SetDestinationOperation(&self, operation: DragOperation) {
        self.drop_effect.set(Some(operation));
    }
    pub fn SetDestinationOperationNone(&self) {
        self.SetDestinationOperation(DragOperation::None);
    }
    pub fn SetDestinationOperationFromEffectAllowed(&self) {
        let mask = self.SourceOperation();
        self.SetDestinationOperation(if mask == DRAG_OPERATION_EVERY {
            DragOperation::Copy
        } else if mask & 16 != 0 {
            DragOperation::Move
        } else if mask & 1 != 0 {
            DragOperation::Copy
        } else if mask & 2 != 0 {
            DragOperation::Link
        } else {
            DragOperation::None
        });
    }
    pub fn ResetDropEffect(&self) {
        self.drop_effect.set(None);
    }
    pub fn SetDragImageElement(&self, node: EventTargetRef, location: (i32, i32)) {
        if self.CanWriteData() {
            *self.drag_image.borrow_mut() = Some(DragImage {
                source: DragImageSource::Node(node),
                location,
            });
        }
    }
    pub fn SetDragImageResource(&self, resource: Rc<dyn DragImageResource>, location: (i32, i32)) {
        if self.CanWriteData() {
            *self.drag_image.borrow_mut() = Some(DragImage {
                source: DragImageSource::Resource(resource),
                location,
            });
        }
    }
    pub fn ClearDragImage(&self) {
        if self.CanWriteData() {
            *self.drag_image.borrow_mut() = None;
        }
    }
    pub fn DragImage(&self) -> Option<DragImage> {
        self.drag_image.borrow().clone()
    }
    // Source: data_transfer.cc:566-578. Markup is serialized by the original
    // DOM serialization owner with ResolveUrls::kAll before this boundary.
    pub fn WriteURL(&self, url: &str, title: &str, resolved_markup: &str) {
        assert!(!url.is_empty());
        self.data_object.SetURLAndTitle(url, title);
        self.data_object.SetData("text/plain", url);
        self.data_object.SetHTMLAndBaseURL(resolved_markup, url);
    }
    pub fn WriteURLUTF16(&self, url: &str, title: &str, markup: &[u16]) {
        assert!(!url.is_empty());
        self.data_object.SetURLAndTitle(url, title);
        self.data_object.SetData("text/plain", url);
        self.data_object.SetHTMLAndBaseURLUTF16(markup, url);
    }
    pub fn DeclareAndWriteDragImageUTF16(
        &self,
        link: Option<&str>,
        image: &str,
        title: &str,
        markup: &[u16],
        file: Option<Arc<DragDataFile>>,
    ) {
        self.data_object
            .SetURLAndTitle(link.unwrap_or(image), title);
        if let Some(file) = file.filter(|f| !f.bytes.is_empty()) {
            self.data_object.AddFileSharedBuffer(file);
        }
        self.data_object.SetDataUTF16("text/html", markup);
    }
    pub fn WriteSelectionUTF16(&self, html: Option<(&[u16], &str)>, text: &[u16]) {
        if let Some((html, base)) = html {
            self.data_object.SetHTMLAndBaseURLUTF16(html, base);
        }
        let mut value = Vec::with_capacity(text.len());
        for (i, &unit) in text.iter().enumerate() {
            let unit = if unit == 0xa0 { 0x20 } else { unit };
            #[cfg(target_os = "windows")]
            {
                if unit == 13 {
                    value.extend_from_slice(&[13, 10]);
                    continue;
                }
                if unit == 10 {
                    if i == 0 || text[i - 1] != 13 {
                        value.extend_from_slice(&[13, 10]);
                    }
                    continue;
                }
            }
            value.push(unit);
        }
        self.data_object.SetDataUTF16("text/plain", &value);
    }
    // Source: data_transfer.cc:547-564. Encoded resource bytes are optional if
    // the image has not finished loading; URI/markup are still written.
    pub fn DeclareAndWriteDragImage(
        &self,
        link_url: Option<&str>,
        image_url: &str,
        title: &str,
        resolved_markup: &str,
        file: Option<Arc<DragDataFile>>,
    ) {
        self.data_object
            .SetURLAndTitle(link_url.unwrap_or(image_url), title);
        if let Some(file) = file.filter(|file| !file.bytes.is_empty()) {
            self.data_object.AddFileSharedBuffer(file);
        }
        self.data_object.SetData("text/html", resolved_markup);
    }
    // Source: data_transfer.cc:580-597. HTML is omitted for text controls.
    pub fn WriteSelection(&self, html: Option<(&str, &str)>, text: &str) {
        if let Some((html, base)) = html {
            self.data_object.SetHTMLAndBaseURL(html, base);
        }
        let text = text.replace('\u{a0}', " ");
        #[cfg(target_os = "windows")]
        let text = text
            .replace("\r\n", "\n")
            .replace('\r', "\n")
            .replace('\n', "\r\n");
        self.data_object.SetData("text/plain", &text);
    }
}

impl std::fmt::Debug for DataTransfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataTransfer")
            .field("policy", &self.policy.get())
            .field("effect_allowed", &self.effect_allowed.borrow())
            .finish_non_exhaustive()
    }
}

// Source: data_transfer.cc:374-390. FileList retains only the source File
// references; the DataObject's one item list owns the canonical mixed data.
impl DataObjectObserver for DataTransfer {
    fn OnItemListChanged(&self) {
        self.data_store_item_list_changed_.set(true);
        self.files_.clear();
        if !self.CanReadData() {
            return;
        }
        for i in 0..self.data_object.length() {
            let item = self.data_object.Item(i).expect("resident item index");
            if item.Kind() == ItemKind::kFileKind {
                if let Some(file) = item.GetAsFile() {
                    self.files_.Append(file);
                }
            }
        }
    }
}
