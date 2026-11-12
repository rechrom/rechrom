/*
 * Copyright (C) 2008 Apple Inc. All Rights Reserved.
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
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: file.h:65-69,161-163,208-229; file.cc:118-148,246-257,307-321,336-344.
// The canonical Blob/native/filesystem File value. Immutable bytes keep one
// backing owner; script ArrayBuffer reads materialize only boundary copies.
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
// Source: file.cc:151-179,226-244,285-305,358-385.
// Source identity is retained independently of metadata and byte storage.
#[derive(Clone, PartialEq, Eq)]
enum FileSource {
    Blob(Arc<()>),
    Native(String),
    FileSystem(String),
}
pub struct File {
    name_: String,
    type_: String,
    last_modified_: i64,
    backing_: crate::native_file::FileBacking,
    source_: FileSource,
    user_visible_: bool,
    relative_path_: String,
}
impl File {
    pub fn Create(
        name: String,
        content_type: String,
        last_modified: i64,
        bytes: Arc<[u8]>,
    ) -> Arc<Self> {
        Arc::new(Self {
            name_: name,
            type_: content_type,
            last_modified_: last_modified,
            backing_: crate::native_file::FileBacking::Memory(bytes),
            source_: FileSource::Blob(Arc::new(())),
            user_visible_: false,
            relative_path_: String::new(),
        })
    }
    /// The host admits immutable data through the same File owner. Host file
    /// I/O completes outside interaction compute; source identity stays intact.
    pub fn CreateForUserProvidedFile(
        path: String,
        display_name: String,
        content_type: String,
        last_modified: i64,
        bytes: Arc<[u8]>,
    ) -> Arc<Self> {
        let name = if display_name.is_empty() {
            foundation::file_path::BaseName(&path)
        } else {
            display_name
        };
        Arc::new(Self {
            name_: name,
            type_: content_type,
            last_modified_: last_modified,
            backing_: crate::native_file::FileBacking::Memory(bytes),
            source_: FileSource::Native(path),
            user_visible_: true,
            relative_path_: String::new(),
        })
    }
    pub fn CreateWithRelativePath(
        path: String,
        name: String,
        relative_path: String,
        content_type: String,
        last_modified: i64,
        bytes: Arc<[u8]>,
    ) -> Arc<Self> {
        Arc::new(Self {
            name_: name,
            type_: content_type,
            last_modified_: last_modified,
            backing_: crate::native_file::FileBacking::Memory(bytes),
            source_: FileSource::Native(path),
            user_visible_: true,
            relative_path_: relative_path,
        })
    }
    pub fn CreateForFileSystemFile(
        url: String,
        name: String,
        content_type: String,
        last_modified: i64,
        bytes: Arc<[u8]>,
        user_visible: bool,
    ) -> Arc<Self> {
        Arc::new(Self {
            name_: name,
            type_: content_type,
            last_modified_: last_modified,
            backing_: crate::native_file::FileBacking::Memory(bytes),
            source_: FileSource::FileSystem(url),
            user_visible_: user_visible,
            relative_path_: String::new(),
        })
    }
    pub fn CreateForNativePath(
        path: String,
        name: String,
        relative_path: String,
        content_type: String,
        metadata: Option<crate::native_file::NativeFileMetadata>,
        reader: Arc<dyn crate::native_file::NativeFileReader>,
    ) -> Arc<Self> {
        let last_modified = metadata
            .as_ref()
            .map_or_else(Self::NowMilliseconds, |m| m.last_modified);
        let backing = crate::native_file::NativePathBacking::new(path.clone(), metadata, reader);
        Arc::new(Self {
            name_: name,
            type_: content_type,
            last_modified_: last_modified,
            backing_: crate::native_file::FileBacking::NativePath(backing),
            source_: FileSource::Native(path),
            user_visible_: true,
            relative_path_: relative_path,
        })
    }
    /// A native provider owns promised or virtual content without granting a
    /// filesystem path. Keep Blob identity while reusing lazy read completion.
    pub fn CreateForNativeData(
        name: String,
        content_type: String,
        metadata: Option<crate::native_file::NativeFileMetadata>,
        reader: Arc<dyn crate::native_file::NativeFileReader>,
    ) -> Arc<Self> {
        let last_modified = metadata
            .as_ref()
            .map_or_else(Self::NowMilliseconds, |metadata| metadata.last_modified);
        let backing = crate::native_file::NativePathBacking::new(String::new(), metadata, reader);
        Arc::new(Self {
            name_: name,
            type_: content_type,
            last_modified_: last_modified,
            backing_: crate::native_file::FileBacking::NativePath(backing),
            source_: FileSource::Blob(Arc::new(())),
            user_visible_: true,
            relative_path_: String::new(),
        })
    }
    /// Enumeration is performed only by the admitted native backing owner;
    /// neither the display path nor script-created File values grant access.
    pub fn EnumerateDirectory(
        self: &Arc<Self>,
        completion: crate::native_file::NativeDirectoryCompletion,
    ) {
        match &self.backing_ {
            crate::native_file::FileBacking::NativePath(backing) => {
                backing.reader.EnumerateDirectory(self.clone(), completion);
            }
            _ => completion(Err(crate::native_file::NativeFileError::Unavailable)),
        }
    }
    pub fn Backing(&self) -> &crate::native_file::FileBacking {
        &self.backing_
    }
    pub fn CachedBytes(&self) -> Result<Arc<[u8]>, crate::native_file::NativeFileError> {
        self.backing_.CachedBytes()
    }
    pub fn ReadBytesSync(&self) -> Result<Arc<[u8]>, crate::NativeFileError> {
        self.backing_.ReadSync()
    }
    pub fn SnapshotMetadataSync(
        &self,
    ) -> Result<crate::NativeFileMetadata, crate::NativeFileError> {
        match &self.backing_ {
            crate::FileBacking::Memory(bytes) => Ok(crate::NativeFileMetadata {
                length: bytes.len(),
                last_modified: self.last_modified_,
                modification_time_microseconds: None,
                content_type: self.type_.clone(),
            }),
            crate::FileBacking::NativePath(owner) => owner.SnapshotMetadataSync(),
        }
    }
    pub fn TrySize(&self) -> Result<usize, crate::native_file::NativeFileError> {
        match &self.backing_ {
            crate::native_file::FileBacking::Memory(bytes) => Ok(bytes.len()),
            crate::native_file::FileBacking::NativePath(owner) => owner
                .metadata
                .get()
                .map(|m| m.length)
                .ok_or(crate::native_file::NativeFileError::MetadataPending),
        }
    }
    pub fn ReadBytes(
        &self,
        complete: Box<dyn FnOnce(Result<Arc<[u8]>, crate::native_file::NativeFileError>) + Send>,
    ) {
        self.backing_.Read(complete)
    }
    pub fn HasBackingFile(&self) -> bool {
        matches!(self.source_, FileSource::Native(_))
    }
    pub fn GetPath(&self) -> &str {
        match &self.source_ {
            FileSource::Native(path) => path,
            _ => "",
        }
    }
    // Source File::FileSystemURL, file.h:159-160.
    pub fn FileSystemURL(&self) -> &str {
        match &self.source_ {
            FileSource::FileSystem(url) => url,
            _ => "",
        }
    }
    pub fn IsUserVisible(&self) -> bool {
        self.user_visible_
    }
    pub fn webkitRelativePath(&self) -> &str {
        &self.relative_path_
    }
    pub fn HasSameSource(&self, other: &Self) -> bool {
        match (&self.source_, &other.source_) {
            (FileSource::Blob(a), FileSource::Blob(b)) => Arc::ptr_eq(a, b),
            (FileSource::Native(a), FileSource::Native(b))
            | (FileSource::FileSystem(a), FileSource::FileSystem(b)) => a == b,
            _ => false,
        }
    }
    pub fn Clone(&self) -> Arc<Self> {
        Arc::new(Self {
            name_: self.name_.clone(),
            type_: self.type_.clone(),
            last_modified_: self.last_modified_,
            backing_: self.backing_.clone(),
            source_: self.source_.clone(),
            user_visible_: self.user_visible_,
            relative_path_: self.relative_path_.clone(),
        })
    }
    /// File.cc:300-304. Name overrides preserve the original immutable backing,
    /// source identity, snapshot metadata and relative path.
    pub fn CloneWithName(&self, name: String) -> Arc<Self> {
        Arc::new(Self {
            name_: name,
            type_: self.type_.clone(),
            last_modified_: self.last_modified_,
            backing_: self.backing_.clone(),
            source_: self.source_.clone(),
            user_visible_: self.user_visible_,
            relative_path_: self.relative_path_.clone(),
        })
    }
    pub fn AppendToControlState(&self, state: &mut Vec<String>) -> bool {
        if !self.HasBackingFile() {
            return false;
        }
        state.extend([
            self.GetPath().to_owned(),
            self.name_.clone(),
            self.relative_path_.clone(),
        ]);
        true
    }
    /// File::CreateFromControlState, cc:151-164. Admission preserves the
    /// native lazy backing and delegates all I/O to the embedding reader.
    pub fn CreateFromControlState(
        state: &[std::rc::Rc<[u16]>],
        index: &mut usize,
        reader: Arc<dyn crate::native_file::NativeFileReader>,
    ) -> Result<Option<Arc<Self>>, crate::native_file::NativeFileError> {
        if index.saturating_add(2) >= state.len() {
            *index = state.len();
            return Ok(None);
        }
        let decode = |value: &[u16]| {
            String::from_utf16(value)
                .map_err(|error| crate::native_file::NativeFileError::Io(error.to_string()))
        };
        let path = decode(&state[*index])?;
        let name = decode(&state[*index + 1])?;
        let relative = decode(&state[*index + 2])?;
        *index += 3;
        let name = if relative.is_empty() && name.is_empty() {
            foundation::file_path::BaseName(&path)
        } else {
            name
        };
        let content_type = reader.ContentType(if name.is_empty() { &path } else { &name })?;
        Ok(Some(Self::CreateForNativePath(
            path,
            name,
            relative,
            content_type,
            None,
            reader,
        )))
    }
    pub fn PathFromControlStateUTF16(
        state: &[std::rc::Rc<[u16]>],
        index: &mut usize,
    ) -> std::rc::Rc<[u16]> {
        if index.saturating_add(2) >= state.len() {
            *index = state.len();
            return std::rc::Rc::from([]);
        }
        let path = state[*index].clone();
        *index += 3;
        path
    }
    pub fn PathFromControlState(state: &[String], index: &mut usize) -> String {
        if index.saturating_add(2) >= state.len() {
            *index = state.len();
            return String::new();
        }
        let path = state[*index].clone();
        *index += 3;
        path
    }
    pub fn name(&self) -> &str {
        &self.name_
    }
    pub fn type_(&self) -> &str {
        &self.type_
    }
    pub fn TryLastModified(&self) -> Result<i64, crate::native_file::NativeFileError> {
        match &self.backing_ {
            crate::native_file::FileBacking::Memory(_) => Ok(self.last_modified_),
            crate::native_file::FileBacking::NativePath(owner) => owner
                .metadata
                .get()
                .map(|m| m.last_modified)
                .ok_or(crate::native_file::NativeFileError::MetadataPending),
        }
    }
    pub fn lastModified(&self) -> i64 {
        self.TryLastModified()
            .expect("native File snapshot must precede synchronous metadata projection")
    }
    pub fn size(&self) -> usize {
        self.TrySize()
            .expect("native file metadata must be read before synchronous size projection")
    }
    pub fn GetBlobDataHandle(&self) -> Arc<[u8]> {
        self.CachedBytes()
            .expect("native file bytes require the asynchronous original reader")
    }
    pub fn NowMilliseconds() -> i64 {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(time) => time.as_millis().min(i64::MAX as u128) as i64,
            Err(time) => -(time.duration().as_millis().min(i64::MAX as u128) as i64),
        }
    }
}
