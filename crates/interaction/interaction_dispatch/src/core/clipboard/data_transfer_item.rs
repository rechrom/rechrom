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
// Source: data_transfer_item.h:52-78; data_transfer_item.cc:47-110.
use super::{
    data_object_item::{DataObjectItem, ItemKind},
    data_transfer::DataTransfer,
};
use crate::core::fileapi::file::File;
use std::{rc::Rc, sync::Arc};
pub struct DataTransferItem {
    data_transfer_: Rc<DataTransfer>,
    item_: Rc<DataObjectItem>,
}
impl DataTransferItem {
    pub fn new(data_transfer: Rc<DataTransfer>, item: Rc<DataObjectItem>) -> Rc<Self> {
        Rc::new(Self {
            data_transfer_: data_transfer,
            item_: item,
        })
    }
    pub fn kind(&self) -> &'static str {
        if !self.data_transfer_.CanReadTypes() {
            return "";
        }
        match self.item_.Kind() {
            ItemKind::kStringKind => "string",
            ItemKind::kFileKind => "file",
        }
    }
    pub fn type_(&self) -> &str {
        if !self.data_transfer_.CanReadTypes() {
            return "";
        }
        self.item_.GetType()
    }
    // Source: cc:67-82. Permission and kind are checked synchronously; the
    // binding posts this string value to its real document task runner. The
    // source itself captures GetAsString before posting (not at callback time).
    pub fn GetAsStringForTask(&self) -> Option<foundation::String> {
        if !self.data_transfer_.CanReadData() || self.item_.Kind() != ItemKind::kStringKind {
            return None;
        }
        Some(self.item_.GetAsString())
    }
    pub fn getAsFile(&self) -> Option<Arc<File>> {
        if !self.data_transfer_.CanReadData() {
            return None;
        }
        self.item_.GetAsFile()
    }
    pub fn GetDataTransfer(&self) -> Rc<DataTransfer> {
        self.data_transfer_.clone()
    }
    pub fn GetDataObjectItem(&self) -> Rc<DataObjectItem> {
        self.item_.clone()
    }
}
