/*
 * Copyright (C) 2006, 2007 Apple Inc.  All rights reserved.
 * Copyright (C) 2008, 2009 Google Inc.
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

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: data_transfer_item_list.h:49-68; data_transfer_item_list.cc:36-99.
use super::{
    data_object::DragDataObject, data_transfer::DataTransfer, data_transfer_item::DataTransferItem,
};
use crate::core::fileapi::file::File;
use std::{rc::Rc, sync::Arc};
pub enum DataTransferItemListError {
    InvalidState,
    NotSupported(String),
}
pub struct DataTransferItemList {
    data_transfer_: Rc<DataTransfer>,
    data_object_: Rc<DragDataObject>,
}
impl DataTransferItemList {
    pub fn new(data_transfer: Rc<DataTransfer>, data_object: Rc<DragDataObject>) -> Rc<Self> {
        Rc::new(Self {
            data_transfer_: data_transfer,
            data_object_: data_object,
        })
    }
    pub fn length(&self) -> u32 {
        if !self.data_transfer_.CanReadTypes() {
            return 0;
        }
        self.data_object_.length()
    }
    pub fn item(&self, index: u32) -> Option<Rc<DataTransferItem>> {
        if !self.data_transfer_.CanReadTypes() {
            return None;
        }
        self.data_object_
            .Item(index)
            .map(|item| DataTransferItem::new(self.data_transfer_.clone(), item))
    }
    pub fn deleteItem(&self, index: u32) -> Result<(), DataTransferItemListError> {
        if !self.data_transfer_.CanWriteData() {
            return Err(DataTransferItemListError::InvalidState);
        }
        self.data_object_.DeleteItem(index);
        Ok(())
    }
    pub fn clear(&self) {
        if self.data_transfer_.CanWriteData() {
            self.data_object_.ClearAll();
        }
    }
    pub fn add(
        &self,
        data: &str,
        content_type: &str,
    ) -> Result<Option<Rc<DataTransferItem>>, DataTransferItemListError> {
        self.addUTF16(&data.encode_utf16().collect::<Vec<_>>(), content_type)
    }
    pub fn addUTF16(
        &self,
        data: &[u16],
        content_type: &str,
    ) -> Result<Option<Rc<DataTransferItem>>, DataTransferItemListError> {
        if !self.data_transfer_.CanWriteData() {
            return Ok(None);
        }
        let item = self
            .data_object_
            .AddUTF16(data, content_type)
            .ok_or_else(|| DataTransferItemListError::NotSupported(content_type.into()))?;
        Ok(Some(DataTransferItem::new(
            self.data_transfer_.clone(),
            item,
        )))
    }
    pub fn addFile(&self, file: Arc<File>) -> Option<Rc<DataTransferItem>> {
        if !self.data_transfer_.CanWriteData() {
            return None;
        }
        Some(DataTransferItem::new(
            self.data_transfer_.clone(),
            self.data_object_.AddFile(file),
        ))
    }
    pub fn GetDataTransfer(&self) -> Rc<DataTransfer> {
        self.data_transfer_.clone()
    }
}
