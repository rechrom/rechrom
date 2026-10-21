// Copyright 2011 Google Inc. BSD-style license.
use super::{SkReadBuffer::SkReadBuffer, SkRect::SkRect};
use std::ops::Deref;
// Source: src/core/SkPictureData.h:41-60
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SkPictInfo {
    pub fMagic: [u8; 8],
    fVersion: u32,
    pub fCullRect: SkRect,
}
impl SkPictInfo {
    // Source: src/core/SkPictureData.h:42-42
    // C++ leaves magic/cull uninitialized. Zero bytes only make the Rust value
    // valid; the native invalid version sentinel remains, and parsing must fill
    // all header fields before accepting it as a real picture header.
    pub fn new() -> Self {
        Self {
            fMagic: [0; 8],
            fVersion: u32::MAX,
            fCullRect: SkRect::MakeEmpty(),
        }
    }
    // Source: src/core/SkPictureData.h:44-47
    pub fn getVersion(&self) -> u32 {
        crate::SkASSERT!(@source b"src/core/SkPictureData.h\0".as_ptr(),45; @condition b"fVersion != ~0U\0".as_ptr(); self.fVersion!=u32::MAX);
        self.fVersion
    }
    // Source: src/core/SkPictureData.h:49-52
    pub fn setVersion(&mut self, version: u32) {
        crate::SkASSERT!(@source b"src/core/SkPictureData.h\0".as_ptr(),50; @condition b"version != ~0U\0".as_ptr(); version!=u32::MAX);
        self.fVersion = version;
    }
}
// Source: src/core/SkPictureData.h:62-66
pub const SK_PICT_READER_TAG: u32 = u32::from_be_bytes(*b"read");
pub const SK_PICT_FACTORY_TAG: u32 = u32::from_be_bytes(*b"fact");
pub const SK_PICT_TYPEFACE_TAG: u32 = u32::from_be_bytes(*b"tpfc");
pub const SK_PICT_PICTURE_TAG: u32 = u32::from_be_bytes(*b"pctr");
pub const SK_PICT_DRAWABLE_TAG: u32 = u32::from_be_bytes(*b"draw");
// Source: src/core/SkPictureData.h:69-76
pub const SK_PICT_BUFFER_SIZE_TAG: u32 = u32::from_be_bytes(*b"aray");
pub const SK_PICT_PAINT_BUFFER_TAG: u32 = u32::from_be_bytes(*b"pnt ");
pub const SK_PICT_PATH_BUFFER_TAG: u32 = u32::from_be_bytes(*b"pth ");
pub const SK_PICT_TEXTBLOB_BUFFER_TAG: u32 = u32::from_be_bytes(*b"blob");
pub const SK_PICT_SLUG_BUFFER_TAG: u32 = u32::from_be_bytes(*b"slug");
pub const SK_PICT_VERTICES_BUFFER_TAG: u32 = u32::from_be_bytes(*b"vert");
pub const SK_PICT_IMAGE_BUFFER_TAG: u32 = u32::from_be_bytes(*b"imag");
// Source: src/core/SkPictureData.h:79-79
pub const SK_PICT_EOF_TAG: u32 = u32::from_be_bytes(*b"eof ");
// Source: src/core/SkPictureData.h:81-86
// Deref is Rust's container-only substitution for both actual retained owner
// forms already used in this crate (intrusive sk_sp and Arc); no object is copied
// or rebound and the result borrows precisely the original array allocation.
pub fn read_index_base_1_or_null<'a, R: Deref>(
    reader: &mut SkReadBuffer<'_>,
    array: &'a [R],
) -> Option<&'a R::Target> {
    let index = reader.readInt();
    if reader.validate(index > 0 && (index as usize) <= array.len()) {
        Some(&*array[index as usize - 1])
    } else {
        None
    }
}
use super::{
    SkBitmap::SkBitmap,
    SkData::SkData,
    SkDrawable::SkDrawable,
    SkPaint::SkPaint,
    SkPath::SkPath,
    SkPicture::SkPicture,
    SkPictureFlat::{SkFactoryPlayback, SkTypefacePlayback},
    SkTextBlob::SkTextBlob,
    SkVertices::SkVertices,
};
use crate::include::core::SkRefCnt::sk_sp;
use crate::include::private::chromium::Slug::Slug;
use crate::src::image::SkImage::SkImage;
use core::{mem::ManuallyDrop, pin::Pin};
use std::sync::Arc;
// Source: src/core/SkPictureData.h:163-181
// Native member order is retained. Explicit reverse teardown preserves each
// retained object's original final-unref position before typeface/factory and
// source data disappear. Canonical Slug and playback types are dependencies,
// never locally fabricated substitute owners.
pub struct SkPictureData {
    fPaints: ManuallyDrop<Vec<SkPaint>>,
    fPaths: ManuallyDrop<Vec<SkPath>>,
    fOpData: ManuallyDrop<Option<Arc<SkData>>>,
    fEmptyPath: ManuallyDrop<SkPath>,
    fEmptyBitmap: ManuallyDrop<SkBitmap>,
    fPictures: ManuallyDrop<Vec<sk_sp<dyn SkPicture>>>,
    fDrawables: ManuallyDrop<Vec<sk_sp<dyn SkDrawable>>>,
    fTextBlobs: ManuallyDrop<Vec<sk_sp<SkTextBlob>>>,
    fVertices: ManuallyDrop<Vec<sk_sp<SkVertices>>>,
    fImages: ManuallyDrop<Vec<Arc<dyn SkImage>>>,
    fSlugs: ManuallyDrop<Vec<sk_sp<dyn Slug>>>,
    fTFPlayback: ManuallyDrop<SkTypefacePlayback>,
    fFactoryPlayback: ManuallyDrop<Option<Box<SkFactoryPlayback>>>,
    fInfo: SkPictInfo,
}
impl Drop for SkPictureData {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fFactoryPlayback);
            ManuallyDrop::drop(&mut self.fTFPlayback);
            ManuallyDrop::drop(&mut self.fSlugs);
            ManuallyDrop::drop(&mut self.fImages);
            ManuallyDrop::drop(&mut self.fVertices);
            ManuallyDrop::drop(&mut self.fTextBlobs);
            ManuallyDrop::drop(&mut self.fDrawables);
            ManuallyDrop::drop(&mut self.fPictures);
            ManuallyDrop::drop(&mut self.fEmptyBitmap);
            ManuallyDrop::drop(&mut self.fEmptyPath);
            ManuallyDrop::drop(&mut self.fOpData);
            ManuallyDrop::drop(&mut self.fPaths);
            ManuallyDrop::drop(&mut self.fPaints);
        }
    }
}
impl SkPictureData {
    // Source: src/core/SkPictureData.h:102-104
    pub fn info(&self) -> &SkPictInfo {
        &self.fInfo
    }
    pub fn opData(&self) -> Option<&Arc<SkData>> {
        self.fOpData.as_ref()
    }
    // Source: src/core/SkPictureData.h:115-119
    pub fn getImage(&self, reader: &mut SkReadBuffer<'_>) -> Option<&dyn SkImage> {
        self.getImageOwner(reader).map(|owner| &**owner)
    }
    // Borrow the original retained image token; no new image owner is fabricated.
    pub(crate) fn getImageOwner(&self, reader: &mut SkReadBuffer<'_>) -> Option<&Arc<dyn SkImage>> {
        let index = reader.readInt();
        if reader.validateIndex(index, self.fImages.len() as i32) {
            Some(&self.fImages[index as usize])
        } else {
            None
        }
    }
    // Source: src/core/SkPictureData.h:121-125
    pub fn getPath(&self, reader: &mut SkReadBuffer<'_>) -> &SkPath {
        let index = reader.readInt();
        if reader.validate(index > 0 && (index as usize) <= self.fPaths.len()) {
            &self.fPaths[index as usize - 1]
        } else {
            &self.fEmptyPath
        }
    }
    // Source: src/core/SkPictureData.h:127-129
    pub fn getPicture(&self, reader: &mut SkReadBuffer<'_>) -> Option<&dyn SkPicture> {
        read_index_base_1_or_null(reader, &self.fPictures)
    }
    // Source: src/core/SkPictureData.h:131-133
    pub fn getDrawable(&self, reader: &mut SkReadBuffer<'_>) -> Option<&dyn SkDrawable> {
        read_index_base_1_or_null(reader, &self.fDrawables)
    }
    // Source: src/core/SkPictureData.h:142-144
    pub fn getTextBlob(&self, reader: &mut SkReadBuffer<'_>) -> Option<&SkTextBlob> {
        read_index_base_1_or_null(reader, &self.fTextBlobs)
    }
    // Source: src/core/SkPictureData.h:146-148
    pub fn getSlug(&self, reader: &mut SkReadBuffer<'_>) -> Option<&dyn Slug> {
        read_index_base_1_or_null(reader, &self.fSlugs)
    }
    // Source: src/core/SkPictureData.h:150-152
    pub fn getVertices(&self, reader: &mut SkReadBuffer<'_>) -> Option<&SkVertices> {
        read_index_base_1_or_null(reader, &self.fVertices)
    }
}
// Source: src/core/SkPictureData.cpp:37-39
fn SafeCount<T>(obj: Option<&[T]>) -> i32 {
    obj.map_or(0, |items| items.len() as i32)
}
// Source: src/core/SkPictureData.cpp:41-42
impl SkPictureData {
    pub fn new(info: &SkPictInfo) -> Self {
        Self {
            fPaints: ManuallyDrop::new(Vec::new()),
            fPaths: ManuallyDrop::new(Vec::new()),
            fOpData: ManuallyDrop::new(None),
            fEmptyPath: ManuallyDrop::new(SkPath::default()),
            fEmptyBitmap: ManuallyDrop::new(SkBitmap::default()),
            fPictures: ManuallyDrop::new(Vec::new()),
            fDrawables: ManuallyDrop::new(Vec::new()),
            fTextBlobs: ManuallyDrop::new(Vec::new()),
            fVertices: ManuallyDrop::new(Vec::new()),
            fImages: ManuallyDrop::new(Vec::new()),
            fSlugs: ManuallyDrop::new(Vec::new()),
            fTFPlayback: ManuallyDrop::new(SkTypefacePlayback::new()),
            fFactoryPlayback: ManuallyDrop::new(None),
            fInfo: *info,
        }
    }
}
// Source: src/core/SkPictureData.cpp:44-64
impl SkPictureData {
    pub fn from_record(
        record: &super::SkPictureRecord::SkPictureRecord,
        info: &SkPictInfo,
    ) -> Self {
        let mut data = Self::new(info);
        *data.fPictures = record.getPictures().to_vec();
        *data.fDrawables = record.getDrawables().to_vec();
        *data.fTextBlobs = record.getTextBlobs().to_vec();
        *data.fVertices = record.getVertices().to_vec();
        *data.fImages = record.getImages().to_vec();
        *data.fSlugs = record.getSlugs().to_vec();
        *data.fOpData = Some(record.opData());
        *data.fPaints = (*record.storage().fPaints).clone();
        data.fPaths
            .resize_with(record.storage().fPaths.count() as usize, SkPath::default);
        record.storage().fPaths.foreach(|path, n| {
            data.fPaths[*n as usize - 1] = path.clone();
        });
        data
    }
}
// Source: src/core/SkPictureData.cpp:69-84
fn compute_chunk_size(array: &[Option<super::SkFlattenable::Factory>]) -> usize {
    let mut size = 4;
    for factory in array {
        let name = super::SkFlattenable::FactoryToNameNullable(*factory);
        if let Some(name) = name.filter(|name| !name.is_empty()) {
            size += super::SkStream::SizeOfPackedUInt(name.len()) as usize + name.len();
        } else {
            size += super::SkStream::SizeOfPackedUInt(0) as usize;
        }
    }
    size
}
// Source: src/core/SkPictureData.cpp:86-89
fn write_tag_size_buffer(
    buffer: &mut dyn super::SkWriteBuffer::SkWriteBuffer,
    tag: u32,
    size: usize,
) {
    buffer.writeUInt(tag);
    buffer.writeUInt(size as u32);
}
// Source: src/core/SkPictureData.cpp:91-94
fn write_tag_size_stream(
    mut stream: Pin<&mut dyn super::SkStream::SkWStream>,
    tag: u32,
    size: usize,
) {
    stream.as_mut().write32(tag);
    stream.as_mut().write32(size as u32);
}
// Source: src/core/SkPictureData.cpp:96-122
impl SkPictureData {
    pub fn WriteFactories(
        mut stream: Pin<&mut dyn super::SkStream::SkWStream>,
        rec: &super::SkPtrRecorder::SkFactorySet,
    ) {
        let count = rec.count();
        let mut array = vec![None; count as usize];
        rec.copyToArray(&mut array);
        let size = compute_chunk_size(&array);
        write_tag_size_stream(stream.as_mut(), SK_PICT_FACTORY_TAG, size);
        #[cfg(sk_debug)]
        let start = stream.bytesWritten();
        stream.as_mut().write32(count as u32);
        for factory in array {
            let name = super::SkFlattenable::FactoryToNameNullable(factory);
            if let Some(name) = name.filter(|name| !name.is_empty()) {
                stream.as_mut().writePackedUInt(name.len());
                unsafe {
                    stream.as_mut().write(name.as_ptr(), name.len());
                }
            } else {
                stream.as_mut().writePackedUInt(0);
            }
        }
        crate::SkASSERT!(@source b"src/core/SkPictureData.cpp\0".as_ptr(),121; @condition b"size == (stream->bytesWritten() - start)\0".as_ptr(); size==stream.bytesWritten()-start);
    }
}
// Source: src/core/SkPictureData.cpp:124-149
impl SkPictureData {
    pub unsafe fn WriteTypefaces(
        mut stream: Pin<&mut dyn super::SkStream::SkWStream>,
        rec: &super::SkPtrRecorder::SkRefCntSet,
        procs: &crate::include::core::SkSerialProcs::SkSerialProcs,
    ) {
        let count = rec.count();
        write_tag_size_stream(stream.as_mut(), SK_PICT_TYPEFACE_TAG, count as usize);
        let mut array = vec![core::ptr::null(); count as usize];
        rec.copyToArray(&mut array);
        for ptr in array {
            let tf = unsafe { &*ptr };
            if let Some(proc) = procs.fTypefaceProc {
                if let Some(data) = unsafe { proc(tf, procs.fTypefaceCtx) } {
                    unsafe {
                        stream.as_mut().write(data.data(), data.size());
                    }
                    continue;
                }
            }
            unsafe {
                tf.serialize(
                    stream.as_mut(),
                    super::SkTypeface::SerializeBehavior::kDoIncludeData,
                );
            }
        }
    }
}
// Source: src/core/SkPictureData.cpp:151-200
impl SkPictureData {
    fn flattenToBuffer(
        &self,
        buffer: &mut dyn super::SkWriteBuffer::SkWriteBuffer,
        textBlobsOnly: bool,
    ) -> Result<(), crate::RasterError> {
        if !textBlobsOnly {
            let numPaints = self.fPaints.len();
            if numPaints > 0 {
                write_tag_size_buffer(buffer, SK_PICT_PAINT_BUFFER_TAG, numPaints);
                for paint in &*self.fPaints {
                    buffer.writePaint(paint)?;
                }
            }
            let numPaths = self.fPaths.len();
            if numPaths > 0 {
                write_tag_size_buffer(buffer, SK_PICT_PATH_BUFFER_TAG, numPaths);
                buffer.writeInt(numPaths as i32);
                for path in &*self.fPaths {
                    buffer.writePath(path);
                }
            }
        }
        if !self.fTextBlobs.is_empty() {
            write_tag_size_buffer(buffer, SK_PICT_TEXTBLOB_BUFFER_TAG, self.fTextBlobs.len());
            for blob in &*self.fTextBlobs {
                unsafe {
                    super::SkTextBlobPriv::SkTextBlobPriv::Flatten(&**blob, buffer);
                }
            }
        }
        if !textBlobsOnly {
            write_tag_size_buffer(buffer, SK_PICT_SLUG_BUFFER_TAG, self.fSlugs.len());
            for slug in &*self.fSlugs {
                slug.doFlatten(buffer);
            }
        }
        if !textBlobsOnly {
            if !self.fVertices.is_empty() {
                write_tag_size_buffer(buffer, SK_PICT_VERTICES_BUFFER_TAG, self.fVertices.len());
                for vertices in &*self.fVertices {
                    super::SkVerticesPriv::SkVerticesPriv {
                        fVertices: &**vertices,
                    }
                    .encode(buffer);
                }
            }
            if !self.fImages.is_empty() {
                write_tag_size_buffer(buffer, SK_PICT_IMAGE_BUFFER_TAG, self.fImages.len());
                for image in &*self.fImages {
                    unsafe {
                        buffer.writeImage(&**image);
                    }
                }
            }
        }
        Ok(())
    }
}
// Source: src/core/SkPictureData.cpp:207-212
fn skip_typeface_proc(
    procs: &crate::include::core::SkSerialProcs::SkSerialProcs,
) -> crate::include::core::SkSerialProcs::SkSerialProcs {
    let mut newProcs = *procs;
    newProcs.fTypefaceProc = None;
    newProcs.fTypefaceCtx = core::ptr::null_mut();
    newProcs
}
// Source: src/core/SkPictureData.cpp:219-271
impl SkPictureData {
    /// Safety: all actual recorder payload accesses in this serialization tree
    /// are externally synchronized and nonreentrant. Streams/typeface/factory
    /// callbacks cannot concurrently mutate these sets or retain payload borrows.
    pub unsafe fn serialize(
        &self,
        mut stream: Pin<&mut dyn super::SkStream::SkWStream>,
        procs: &crate::include::core::SkSerialProcs::SkSerialProcs,
        topLevelTypeFaceSet: Option<&super::SkPtrRecorder::SkRefCntSetOwner>,
        textBlobsOnly: bool,
    ) -> Result<(), crate::RasterError> {
        let data = self.fOpData.as_ref().expect("native picture op data");
        write_tag_size_stream(stream.as_mut(), SK_PICT_READER_TAG, data.size());
        unsafe {
            stream.as_mut().write(data.bytes(), data.size());
        }
        // Source: src/core/SkPictureData.cpp:226-227
        // Native recursive parameter borrows the existing complete set owner.
        // Only the buffer takes an additional retained reference below.
        let localTypefaceSet = super::SkPtrRecorder::SkRefCntSet::default().into_owner();
        let typefaceSet = topLevelTypeFaceSet.unwrap_or(&localTypefaceSet);
        let factSet = super::SkPtrRecorder::SkFactorySet::default().into_owner();
        let mut buffer = super::SkWriteBuffer::SkBinaryWriteBuffer::new(skip_typeface_proc(procs));
        // Source: src/core/SkPictureData.cpp:231-234
        // Retain the exact local/external owners once for the buffer. Locals
        // precede buffer creation, so buffer release precedes their destruction.
        unsafe {
            buffer.setFactoryRecorder(Some(
                crate::include::core::SkRefCnt::SkRefCntHandle::ref_handle(&factSet),
            ));
            buffer.setTypefaceRecorder(Some(
                crate::include::core::SkRefCnt::SkRefCntHandle::ref_handle(typefaceSet),
            ));
        }
        self.flattenToBuffer(&mut buffer, textBlobsOnly)?;
        struct DevNull {
            fBytesWritten: usize,
        }
        unsafe impl super::SkStream::SkWStream for DevNull {
            unsafe fn write(self: Pin<&mut Self>, _: *const u8, size: usize) -> bool {
                self.get_mut().fBytesWritten += size;
                true
            }
            fn bytesWritten(&self) -> usize {
                self.fBytesWritten
            }
        }
        let mut devnull = core::pin::pin!(DevNull { fBytesWritten: 0 });
        for pic in &*self.fPictures {
            unsafe {
                super::SkPicture::serialize(&**pic, devnull.as_mut(), None, Some(typefaceSet), true)
            }?;
        }
        if textBlobsOnly {
            return Ok(());
        }
        // Native caller holds the external nonreentrant operation lease; these
        // shared payload borrows end before subsequent recursive serialization.
        unsafe {
            Self::WriteFactories(stream.as_mut(), &*factSet.get());
            Self::WriteTypefaces(stream.as_mut(), &*typefaceSet.get(), procs);
        }
        write_tag_size_stream(
            stream.as_mut(),
            SK_PICT_BUFFER_SIZE_TAG,
            buffer.bytesWritten(),
        );
        unsafe {
            buffer.writeToStream(stream.as_mut());
        }
        if !self.fPictures.is_empty() {
            write_tag_size_stream(stream.as_mut(), SK_PICT_PICTURE_TAG, self.fPictures.len());
            for pic in &*self.fPictures {
                unsafe {
                    super::SkPicture::serialize(
                        &**pic,
                        stream.as_mut(),
                        Some(procs),
                        Some(typefaceSet),
                        false,
                    )
                }?;
            }
        }
        stream.as_mut().write32(SK_PICT_EOF_TAG);
        Ok(())
    }
}
// Source: src/core/SkPictureData.cpp:273-294
impl SkPictureData {
    pub fn flatten(
        &self,
        buffer: &mut dyn super::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        let data = self.fOpData.as_ref().expect("native picture op data");
        write_tag_size_buffer(buffer, SK_PICT_READER_TAG, data.size());
        unsafe {
            buffer.writeByteArray(data.bytes(), data.size());
        }
        if !self.fPictures.is_empty() {
            write_tag_size_buffer(buffer, SK_PICT_PICTURE_TAG, self.fPictures.len());
            for pic in &*self.fPictures {
                super::SkPicture::PicturePriv_Flatten(pic.clone(), buffer)?;
            }
        }
        if !self.fDrawables.is_empty() {
            write_tag_size_buffer(buffer, SK_PICT_DRAWABLE_TAG, self.fDrawables.len());
            for draw in &*self.fDrawables {
                buffer.writeFlattenable(Some(&**draw))?;
            }
        }
        self.flattenToBuffer(buffer, false)?;
        buffer.writeUInt(SK_PICT_EOF_TAG);
        Ok(())
    }
}
// Source: src/core/SkPictureData.cpp:298-417
impl SkPictureData {
    unsafe fn parseStreamTag(
        &mut self,
        mut stream: Pin<&mut dyn super::SkStream::SkStream>,
        mut tag: u32,
        mut size: u32,
        procs: &crate::include::core::SkSerialProcs::SkDeserialProcs,
        topLevelTFPlayback: *const SkTypefacePlayback,
        recursionLimit: i32,
    ) -> Result<bool, crate::RasterError> {
        use super::SkStreamPriv::RemainingLengthIsBelow;
        if let Some(proc) = procs.fAllowTagsProc {
            if !unsafe { proc(tag, procs.fAllowTagsCtx) } {
                return Ok(size == 0);
            }
        }
        match tag {
            SK_PICT_READER_TAG => {
                crate::SkASSERT!(@source b"src/core/SkPictureData.cpp\0".as_ptr(),311; @condition b"nullptr == fOpData\0".as_ptr(); self.fOpData.is_none());
                *self.fOpData = unsafe { SkData::MakeFromStream(stream.as_mut(), size as usize) };
                if self.fOpData.is_none() {
                    return Ok(false);
                }
            }
            SK_PICT_FACTORY_TAG => {
                if !stream.as_mut().readU32(&mut size) {
                    return Ok(false);
                }
                if RemainingLengthIsBelow(&*stream, size as usize) {
                    return Ok(false);
                }
                *self.fFactoryPlayback = Some(Box::new(SkFactoryPlayback::new(size as i32)));
                for i in 0..size as usize {
                    let mut len = 0;
                    if !stream.as_mut().readPackedUInt(&mut len) {
                        return Ok(false);
                    }
                    if RemainingLengthIsBelow(&*stream, len) {
                        return Ok(false);
                    }
                    let mut str = vec![0; len];
                    if unsafe { stream.as_mut().read(str.as_mut_ptr(), len) } != len {
                        return Ok(false);
                    }
                    self.fFactoryPlayback.as_mut().unwrap().base()[i] =
                        super::SkFlattenable::NameToFactory(&str);
                }
            }
            SK_PICT_TYPEFACE_TAG => {
                if RemainingLengthIsBelow(&*stream, size as usize) {
                    return Ok(false);
                }
                self.fTFPlayback.setCount(size as usize);
                for i in 0..size as usize {
                    if stream.isAtEnd() {
                        return Ok(false);
                    }
                    let tf = if let Some(proc) = procs.fTypefaceStreamProc {
                        unsafe { proc(stream.as_mut(), procs.fTypefaceCtx) }
                    } else {
                        super::SkTypeface::SkTypeface::MakeDeserialize(
                            stream.as_mut(),
                            Default::default(),
                            None,
                        )
                    };
                    *self.fTFPlayback.get_mut(i) =
                        Some(tf.unwrap_or_else(super::SkTypeface::SkTypeface::MakeEmpty));
                }
            }
            SK_PICT_PICTURE_TAG => {
                crate::SkASSERT!(@source b"src/core/SkPictureData.cpp\0".as_ptr(),361; @condition b"fPictures.empty()\0".as_ptr(); self.fPictures.is_empty());
                if RemainingLengthIsBelow(&*stream, size as usize) {
                    return Ok(false);
                }
                self.fPictures.reserve_exact(size as usize);
                for _ in 0..size {
                    let pic = unsafe {
                        super::SkPicture::MakeFromStreamPriv(
                            stream.as_mut(),
                            Some(procs),
                            topLevelTFPlayback,
                            recursionLimit - 1,
                        )
                    }?;
                    let Some(pic) = pic else {
                        return Ok(false);
                    };
                    self.fPictures.push(pic);
                }
            }
            SK_PICT_BUFFER_SIZE_TAG => {
                if RemainingLengthIsBelow(&*stream, size as usize) {
                    return Ok(false);
                }
                let mut storage = vec![0; size as usize];
                if unsafe { stream.as_mut().read(storage.as_mut_ptr(), size as usize) }
                    != size as usize
                {
                    return Ok(false);
                }
                // Dictionaries remain borrowed from their original owner while
                // only disjoint resource arrays are mutated through raw field
                // projections. No whole-data mutable reference spans those leases.
                let data = self as *mut Self;
                let mut buffer = SkReadBuffer::new(&storage);
                buffer.setVersion(self.fInfo.getVersion() as i32);
                buffer.setRecursionLimit(recursionLimit - 1);
                let Some(factory) = self.fFactoryPlayback.as_ref() else {
                    return Ok(false);
                };
                factory.setupBuffer(&mut buffer);
                buffer.setDeserialProcs(*procs);
                if self.fTFPlayback.count() > 0 {
                    self.fTFPlayback.setupBuffer(&mut buffer);
                } else {
                    unsafe {
                        (&*topLevelTFPlayback).setupBuffer(&mut buffer);
                    }
                }
                while !buffer.eof() && buffer.isValid() {
                    tag = buffer.readUInt();
                    size = buffer.readUInt();
                    unsafe { Self::parseBufferTag_raw(data, &mut buffer, tag, size) }?;
                }
                if !buffer.isValid() {
                    return Ok(false);
                }
            }
            _ => {}
        }
        Ok(true)
    }
}
// Source: src/core/SkPictureData.cpp:419-421
fn create_image_from_buffer(
    buffer: &mut SkReadBuffer<'_>,
) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
    buffer.readImage()
}
// Source: src/core/SkPictureData.cpp:423-425
fn create_drawable_from_buffer(
    buffer: &mut SkReadBuffer<'_>,
) -> Result<Option<sk_sp<dyn SkDrawable>>, crate::RasterError> {
    use super::SkFlattenable::{FlattenableOwner, Type};
    let Some(obj) = buffer.readFlattenable(Type::kSkDrawable_Type)? else {
        return Ok(None);
    };
    let FlattenableOwner::Intrusive(mut obj) = obj else {
        panic!("checked native Drawable requires its intrusive complete owner");
    };
    let owner = obj.as_ptr().expect("factory returned live owner");
    let projected = unsafe { owner.as_ref().project_drawable_owner(owner.cast()) }
        .expect("checked Drawable type requires complete owner metadata");
    obj.release();
    Ok(Some(unsafe { sk_sp::from_raw(projected.as_ptr()) }))
}
// Source: src/core/SkPictureData.cpp:428-453
fn new_array_from_buffer<T, F>(
    buffer: &mut SkReadBuffer<'_>,
    inCount: u32,
    array: &mut Vec<T>,
    mut factory: F,
) -> Result<bool, crate::RasterError>
where
    F: FnMut(&mut SkReadBuffer<'_>) -> Result<Option<T>, crate::RasterError>,
{
    if !buffer.validate(array.is_empty() && inCount <= i32::MAX as u32) {
        return Ok(false);
    }
    if inCount == 0 {
        return Ok(true);
    }
    for _ in 0..inCount {
        let obj = factory(buffer)?;
        if !buffer.validate(obj.is_some()) {
            array.clear();
            return Ok(false);
        }
        array.push(obj.unwrap());
    }
    Ok(true)
}
impl SkPictureData {
    // Structural borrowed/raw projection; dictionary borrows are disjoint.
    pub fn parseBufferTag(
        &mut self,
        buffer: &mut SkReadBuffer<'_>,
        tag: u32,
        size: u32,
    ) -> Result<(), crate::RasterError> {
        unsafe { Self::parseBufferTag_raw(self, buffer, tag, size) }
    }
}
// Source: src/core/SkPictureData.cpp:455-535
impl SkPictureData {
    unsafe fn parseBufferTag_raw(
        data: *mut Self,
        buffer: &mut SkReadBuffer<'_>,
        tag: u32,
        size: u32,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            if !buffer.allowTags(tag) {
                buffer.validate(size == 0);
                return Ok(());
            }
            match tag {
                SK_PICT_PAINT_BUFFER_TAG => {
                    if !buffer.validate(size <= i32::MAX as u32) {
                        return Ok(());
                    }
                    for _ in 0..size {
                        (*data).fPaints.push(buffer.readPaint()?);
                        if !buffer.isValid() {
                            return Ok(());
                        }
                    }
                }
                SK_PICT_PATH_BUFFER_TAG => {
                    if size > 0 {
                        let count = buffer.readInt();
                        if !buffer.validate(count >= 0) {
                            return Ok(());
                        }
                        for _ in 0..count {
                            if let Some(path) = buffer.readPath() {
                                (*data).fPaths.push(path);
                            } else {
                                crate::SkASSERT!(@source b"src/core/SkPictureData.cpp\0".as_ptr(),490; @condition b"!buffer.isValid()\0".as_ptr(); !buffer.isValid());
                                return Ok(());
                            }
                        }
                    }
                }
                SK_PICT_TEXTBLOB_BUFFER_TAG => {
                    new_array_from_buffer(buffer, size, &mut (*data).fTextBlobs, |buffer| {
                        let blob = super::SkTextBlobPriv::SkTextBlobPriv::MakeFromBuffer(buffer);
                        Ok(if blob.is_some() { Some(blob) } else { None })
                    })?;
                }
                SK_PICT_SLUG_BUFFER_TAG => {
                    new_array_from_buffer(
                        buffer,
                        size,
                        &mut (*data).fSlugs,
                        crate::include::private::chromium::Slug::MakeFromBuffer,
                    )?;
                }
                SK_PICT_VERTICES_BUFFER_TAG => {
                    new_array_from_buffer(buffer, size, &mut (*data).fVertices, |buffer| {
                        let vertices = super::SkVerticesPriv::SkVerticesPriv::Decode(buffer);
                        Ok(if vertices.is_some() {
                            Some(vertices)
                        } else {
                            None
                        })
                    })?;
                }
                SK_PICT_IMAGE_BUFFER_TAG => {
                    new_array_from_buffer(
                        buffer,
                        size,
                        &mut (*data).fImages,
                        create_image_from_buffer,
                    )?;
                }
                SK_PICT_READER_TAG => {
                    if !buffer.validateCanReadN::<u8>(size as usize) {
                        return Ok(());
                    }
                    let op_data = unsafe { SkData::MakeUninitialized(size as usize) };
                    if !unsafe { buffer.readByteArray_raw(op_data.writable_data(), size as usize) }
                        || !buffer.validate((*data).fOpData.is_none())
                    {
                        return Ok(());
                    }
                    crate::SkASSERT!(@source b"src/core/SkPictureData.cpp\0".as_ptr(),518; @condition b"nullptr == fOpData\0".as_ptr(); (*data).fOpData.is_none());
                    *(*data).fOpData = Some(op_data);
                }
                SK_PICT_PICTURE_TAG => {
                    buffer.downLevel();
                    let result = new_array_from_buffer(
                        buffer,
                        size,
                        &mut (*data).fPictures,
                        super::SkPicture::PicturePriv_MakeFromBuffer,
                    );
                    buffer.upLevel();
                    result?;
                }
                SK_PICT_DRAWABLE_TAG => {
                    buffer.downLevel();
                    let result = new_array_from_buffer(
                        buffer,
                        size,
                        &mut (*data).fDrawables,
                        create_drawable_from_buffer,
                    );
                    buffer.upLevel();
                    result?;
                }
                _ => {
                    buffer.validate(false);
                }
            }
            Ok(())
        }
    }
}
// Source: src/core/SkPictureData.cpp:537-554
impl SkPictureData {
    pub unsafe fn CreateFromStream(
        mut stream: Pin<&mut dyn super::SkStream::SkStream>,
        info: &SkPictInfo,
        procs: &crate::include::core::SkSerialProcs::SkDeserialProcs,
        mut topLevelTFPlayback: *const SkTypefacePlayback,
        recursionLimit: i32,
    ) -> Result<Option<Box<Self>>, crate::RasterError> {
        if recursionLimit <= 0 {
            return Ok(None);
        }
        let mut data = Box::new(Self::new(info));
        if topLevelTFPlayback.is_null() {
            topLevelTFPlayback = &*data.fTFPlayback;
        }
        if !unsafe { data.parseStream(stream.as_mut(), procs, topLevelTFPlayback, recursionLimit) }?
        {
            return Ok(None);
        }
        Ok(Some(data))
    }
}
// Source: src/core/SkPictureData.cpp:556-568
impl SkPictureData {
    pub fn CreateFromBuffer(
        buffer: &mut SkReadBuffer<'_>,
        info: &SkPictInfo,
    ) -> Result<Option<Box<Self>>, crate::RasterError> {
        if !buffer.isValid() {
            return Ok(None);
        }
        let mut data = Box::new(Self::new(info));
        buffer.setVersion(info.getVersion() as i32);
        if !data.parseBuffer(buffer)? {
            return Ok(None);
        }
        Ok(Some(data))
    }
}
// Source: src/core/SkPictureData.cpp:570-588
impl SkPictureData {
    unsafe fn parseStream(
        &mut self,
        mut stream: Pin<&mut dyn super::SkStream::SkStream>,
        procs: &crate::include::core::SkSerialProcs::SkDeserialProcs,
        topLevelTFPlayback: *const SkTypefacePlayback,
        recursionLimit: i32,
    ) -> Result<bool, crate::RasterError> {
        loop {
            let mut tag = 0;
            if !stream.as_mut().readU32(&mut tag) {
                return Ok(false);
            }
            if tag == SK_PICT_EOF_TAG {
                break;
            }
            let mut size = 0;
            if !stream.as_mut().readU32(&mut size) {
                return Ok(false);
            }
            if !unsafe {
                self.parseStreamTag(
                    stream.as_mut(),
                    tag,
                    size,
                    procs,
                    topLevelTFPlayback,
                    recursionLimit,
                )
            }? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
// Source: src/core/SkPictureData.cpp:590-606
impl SkPictureData {
    fn parseBuffer(&mut self, buffer: &mut SkReadBuffer<'_>) -> Result<bool, crate::RasterError> {
        while buffer.isValid() {
            let tag = buffer.readUInt();
            if tag == SK_PICT_EOF_TAG {
                break;
            }
            let size = buffer.readUInt();
            self.parseBufferTag(buffer, tag, size)?;
        }
        if !buffer.validate(self.opData().is_some()) {
            return Ok(false);
        }
        Ok(true)
    }
}
// Source: src/core/SkPictureData.cpp:608-615
impl SkPictureData {
    pub fn optionalPaint(&self, reader: &mut SkReadBuffer<'_>) -> Option<&SkPaint> {
        let index = reader.readInt();
        if index == 0 {
            return None;
        }
        if reader.validate(index > 0 && (index as usize) <= self.fPaints.len()) {
            Some(&self.fPaints[index as usize - 1])
        } else {
            None
        }
    }
}
// Source: src/core/SkPictureData.cpp:617-624
impl SkPictureData {
    pub fn requiredPaint(&self, reader: &mut SkReadBuffer<'_>) -> &SkPaint {
        let paint = self.optionalPaint(reader);
        if reader.validate(paint.is_some()) {
            return paint.unwrap();
        }
        // The native error path intentionally retains this process-wide paint.
        // It is never used as a successful decoded paint or missing producer.
        struct DefaultPaint(SkPaint);
        // This instance is permanently the child-free default paint. It is
        // initialized once and never mutated or given dynamic effect owners.
        unsafe impl Send for DefaultPaint {}
        unsafe impl Sync for DefaultPaint {}
        static STUB: std::sync::OnceLock<DefaultPaint> = std::sync::OnceLock::new();
        &STUB.get_or_init(|| DefaultPaint(SkPaint::default())).0
    }
}
