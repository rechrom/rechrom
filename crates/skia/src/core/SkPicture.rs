// Copyright 2007 The Android Open Source Project. BSD-style license.
use super::{SkCanvas::SkCanvas, SkRect::SkRect};
use crate::include::core::SkRefCnt::{SkRefCnt, SkRefCntBase, SkRefCntObject};
use core::{
    pin::Pin,
    ptr::NonNull,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
};
// Source: include/core/SkPicture.h:97-101
// Source: include/core/SkPicture.h:117-123
// An abstract callback has no fabricated implementation, Clone or Copy. Rust's
// Box<dyn AbortCallback> supplies dynamic destruction for actual callback owners.
pub trait AbortCallback {
    fn abort(&mut self) -> bool;
}
// Source: include/core/SkPicture.h:44-46
// This is the actual native abstract virtual contract, not a playback provider
// for a different retained recording. Every concrete owner must embed its one
// genuine base state/counter and implement the original required operations.
pub trait SkPicture: SkRefCntObject + Send + Sync + 'static {
    fn picture_state(&self) -> &SkPictureState;
    // Source: include/core/SkPicture.h:136-136
    /// Safety: Canvas is the original live pinned complete owner, with its native
    /// exclusive drawing lease; no whole-Canvas reference spans playback callbacks.
    unsafe fn playback<'device>(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        callback: Option<&mut dyn AbortCallback>,
    ) -> Result<(), crate::RasterError>;
    // Source: include/core/SkPicture.h:149-149
    fn cullRect(&self) -> SkRect;
    // Source: include/core/SkPicture.h:155-155
    fn uniqueID(&self) -> u32 {
        self.picture_state().fUniqueID
    }
    // Source: include/core/SkPicture.h:219-219
    fn approximateOpCount(&self, nested: bool) -> i32;
    // Source: include/core/SkPicture.h:228-228
    fn approximateBytesUsed(&self) -> usize;
    // Source: include/core/SkPicture.h:277-277
    fn asSkBigPicture(&self) -> Option<&super::SkBigPicture::SkBigPicture> {
        None
    }
}
// Source: include/core/SkPicture.h:243-244
pub use crate::src::shaders::SkPictureShader::PictureMakeShader as makeShader;
// Source: include/core/SkPicture.h:246-248
pub fn makeShader_default(
    picture: &crate::include::core::SkRefCnt::sk_sp<dyn SkPicture>,
    tmx: crate::include::core::SkTileMode::SkTileMode,
    tmy: crate::include::core::SkTileMode::SkTileMode,
    mode: crate::include::core::SkSamplingOptions::SkFilterMode,
) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
    makeShader(picture, tmx, tmy, mode, None, None)
}
// Source: include/core/SkPicture.h:287-288
// Rust separates the abstract class storage from its vtable. The intrusive
// counter belongs to this same embedded Base, never to an Arc/proxy allocation.
pub struct SkPictureState {
    // Source: include/core/SkPicture.h:288-288
    // Rust member destruction is declaration-order. Store native members in
    // reverse native declaration order, followed by the actual native parent. The
    // constructor's named initializers below still initialize parent first.
    pub(crate) fAddedToCache: AtomicBool,
    // Source: include/core/SkPicture.h:287-287
    fUniqueID: u32,
    // Source: include/core/SkPicture.h:44-46
    // This is the sole actual SkRefCnt parent; it drops after both own members
    // and after the original SkPicture destructor body has posted its purge.
    pub(crate) ref_count: SkRefCnt,
}
// Source: src/core/SkPicture.cpp:37-41
const kFailure_TrailingStreamByteAfterPictInfo: u8 = 0;
const kPictureData_TrailingStreamByteAfterPictInfo: u8 = 1;
const kCustom_TrailingStreamByteAfterPictInfo: u8 = 2;
// Source: src/core/SkPicture.cpp:45-50
impl SkPictureState {
    // Source: include/core/SkPicture.h:252-252
    pub(crate) fn new() -> Self {
        static NEXT_ID: AtomicU32 = AtomicU32::new(1);
        let unique_id = loop {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            if id != 0 {
                break id;
            }
        };
        Self {
            ref_count: SkRefCnt::new(),
            fUniqueID: unique_id,
            fAddedToCache: AtomicBool::new(false),
        }
    }
}
// Source: src/core/SkPicture.cpp:52-56
impl Drop for SkPictureState {
    fn drop(&mut self) {
        if self.fAddedToCache.load(Ordering::SeqCst) {
            super::SkResourceCache::PostPurgeSharedID(super::SkPicturePriv::MakeSharedID(
                self.fUniqueID,
            ));
        }
    }
}
// Source: src/core/SkPicture.cpp:58-58
const kMagic: [u8; 8] = *b"skiapict";
// Source: src/core/SkPicture.cpp:60-71
// Source: include/core/SkPicture.h:284-284
pub fn createHeader(picture: &dyn SkPicture) -> super::SkPictureData::SkPictInfo {
    let mut info = super::SkPictureData::SkPictInfo::new();
    const _: () = assert!(std::mem::size_of_val(&kMagic) == 8);
    info.fMagic = kMagic;
    info.setVersion(super::SkPicturePriv::kCurrent_Version as u32);
    info.fCullRect = picture.cullRect();
    info
}
// Source: src/core/SkPicture.cpp:73-82
// Source: include/core/SkPicture.h:279-279
pub fn IsValidPictInfo(info: &super::SkPictureData::SkPictInfo) -> bool {
    if info.fMagic != kMagic {
        return false;
    }
    if info.getVersion() < (super::SkPicturePriv::kMin_Version as u32)
        || info.getVersion() > (super::SkPicturePriv::kCurrent_Version as u32)
    {
        return false;
    }
    true
}
// Source: src/core/SkPicture.cpp:84-107
// Source: include/core/SkPicture.h:272-272
pub fn StreamIsSKP(
    stream: Option<Pin<&mut dyn super::SkStream::SkStream>>,
    pInfo: Option<&mut super::SkPictureData::SkPictInfo>,
) -> bool {
    let Some(mut stream) = stream else {
        return false;
    };
    let mut info = super::SkPictureData::SkPictInfo::new();
    crate::SkASSERT!(@source b"src/core/SkPicture.cpp\0".as_ptr(),90; @condition b"sizeof(kMagic) == sizeof(info.fMagic)\0".as_ptr(); kMagic.len()==info.fMagic.len());
    if unsafe { stream.as_mut().read(info.fMagic.as_mut_ptr(), kMagic.len()) } != kMagic.len() {
        return false;
    }
    let mut version = 0;
    if !stream.as_mut().readU32(&mut version) {
        return false;
    }
    info.setVersion(version);
    if !stream.as_mut().readScalar(&mut info.fCullRect.fLeft) {
        return false;
    }
    if !stream.as_mut().readScalar(&mut info.fCullRect.fTop) {
        return false;
    }
    if !stream.as_mut().readScalar(&mut info.fCullRect.fRight) {
        return false;
    }
    if !stream.as_mut().readScalar(&mut info.fCullRect.fBottom) {
        return false;
    }
    if let Some(output) = pInfo {
        *output = info;
    }
    IsValidPictInfo(&info)
}
// Source: src/core/SkPicture.cpp:109-111
pub fn SkPicture_StreamIsSKP(
    stream: Option<Pin<&mut dyn super::SkStream::SkStream>>,
    pInfo: Option<&mut super::SkPictureData::SkPictInfo>,
) -> bool {
    StreamIsSKP(stream, pInfo)
}
// Source: src/core/SkPicture.cpp:113-128
// Source: include/core/SkPicture.h:273-273
pub fn BufferIsSKP(
    buffer: &mut super::SkReadBuffer::SkReadBuffer<'_>,
    pInfo: Option<&mut super::SkPictureData::SkPictInfo>,
) -> bool {
    let mut info = super::SkPictureData::SkPictInfo::new();
    crate::SkASSERT!(@source b"src/core/SkPicture.cpp\0".as_ptr(),115; @condition b"sizeof(kMagic) == sizeof(info.fMagic)\0".as_ptr(); kMagic.len()==info.fMagic.len());
    if !buffer.readByteArray(&mut info.fMagic) {
        return false;
    }
    info.setVersion(buffer.readUInt());
    buffer.readRectInto(&mut info.fCullRect);
    if IsValidPictInfo(&info) {
        if let Some(output) = pInfo {
            *output = info;
        }
        return true;
    }
    false
}
use super::{
    SkPictureData::{SkPictInfo, SkPictureData},
    SkPictureFlat::SkTypefacePlayback,
    SkReadBuffer::SkReadBuffer,
    SkStream::{SkStream, SkWStream},
};
use crate::include::core::SkRefCnt::sk_sp;
use crate::include::core::SkSerialProcs::{SkDeserialProcs, SkSerialProcs};
// Source: src/core/SkPicture.cpp:130-143
// Source: include/core/SkPicture.h:280-282
pub fn Forwardport(
    info: &SkPictInfo,
    data: Option<&SkPictureData>,
    buffer: Option<&mut SkReadBuffer<'_>>,
) -> Result<Option<sk_sp<dyn SkPicture>>, crate::RasterError> {
    let Some(data) = data else {
        return Ok(None);
    };
    if data.opData().is_none() {
        return Ok(None);
    }
    let playback = super::SkPicturePlayback::SkPicturePlayback::new(data);
    let mut r = super::SkPictureRecorder::SkPictureRecorder::new();
    let canvas = unsafe { r.beginRecording(&info.fCullRect, sk_sp::default()) }?;
    unsafe { playback.draw(canvas, None, buffer) }?;
    Ok(Some(unsafe { r.finishRecordingAsPicture() }?))
}
// Source: src/core/SkPicture.cpp:145-147
// Source: include/core/SkPicture.h:61-62
pub unsafe fn MakeFromStream(
    stream: Option<Pin<&mut dyn SkStream>>,
    procs: Option<&SkDeserialProcs>,
) -> Result<Option<sk_sp<dyn SkPicture>>, crate::RasterError> {
    let Some(stream) = stream else {
        return Ok(None);
    };
    unsafe {
        MakeFromStreamPriv(
            stream,
            procs,
            core::ptr::null(),
            super::SkPicturePriv::kDefaultRecursionLimit,
        )
    }
}
// Source: src/core/SkPicture.cpp:149-156
// Source: include/core/SkPicture.h:87-88
pub unsafe fn MakeFromData(
    data: *const u8,
    size: usize,
    procs: Option<&SkDeserialProcs>,
) -> Result<Option<sk_sp<dyn SkPicture>>, crate::RasterError> {
    if data.is_null() {
        return Ok(None);
    }
    let mut stream =
        core::pin::pin!(unsafe { super::SkStream::SkMemoryStream::from_memory(data, size, false) });
    unsafe {
        MakeFromStreamPriv(
            stream.as_mut(),
            procs,
            core::ptr::null(),
            super::SkPicturePriv::kDefaultRecursionLimit,
        )
    }
}
// Source: src/core/SkPicture.cpp:158-164
// Source: include/core/SkPicture.h:77-78
pub fn MakeFromDataObject(
    data: Option<&super::SkData::SkData>,
    procs: Option<&SkDeserialProcs>,
) -> Result<Option<sk_sp<dyn SkPicture>>, crate::RasterError> {
    let Some(data) = data else {
        return Ok(None);
    };
    let mut stream = core::pin::pin!(unsafe {
        super::SkStream::SkMemoryStream::from_memory(data.data(), data.size(), false)
    });
    unsafe {
        MakeFromStreamPriv(
            stream.as_mut(),
            procs,
            core::ptr::null(),
            super::SkPicturePriv::kDefaultRecursionLimit,
        )
    }
}
// Source: src/core/SkPicture.cpp:166-209
// Source: include/core/SkPicture.h:259-261
pub unsafe fn MakeFromStreamPriv(
    mut stream: Pin<&mut dyn SkStream>,
    procsPtr: Option<&SkDeserialProcs>,
    typefaces: *const SkTypefacePlayback,
    recursionLimit: i32,
) -> Result<Option<sk_sp<dyn SkPicture>>, crate::RasterError> {
    if recursionLimit <= 0 {
        return Ok(None);
    }
    let mut info = SkPictInfo::new();
    if !StreamIsSKP(Some(stream.as_mut()), Some(&mut info)) {
        return Ok(None);
    }
    let procs = procsPtr.copied().unwrap_or_default();
    let mut trailing = 0;
    if !stream.as_mut().readU8(&mut trailing) {
        return Ok(None);
    }
    match trailing {
        kPictureData_TrailingStreamByteAfterPictInfo => {
            let data = unsafe {
                SkPictureData::CreateFromStream(
                    stream.as_mut(),
                    &info,
                    &procs,
                    typefaces,
                    recursionLimit,
                )
            }?;
            Forwardport(&info, data.as_deref(), None)
        }
        kCustom_TrailingStreamByteAfterPictInfo => {
            let mut ssize = 0;
            if !stream.as_mut().readS32(&mut ssize) || ssize >= 0 || procs.fPictureProc.is_none() {
                return Ok(None);
            }
            let size = (-(ssize as i64)) as usize;
            if super::SkStreamPriv::RemainingLengthIsBelow(&*stream, size) {
                return Ok(None);
            }
            let data = unsafe { super::SkData::SkData::MakeUninitialized(size) };
            if unsafe { stream.as_mut().read(data.writable_data(), size) } != size {
                return Ok(None);
            }
            Ok(unsafe { procs.fPictureProc.unwrap()(data.data(), size, procs.fPictureCtx) })
        }
        _ => Ok(None),
    }
}
// Source: src/core/SkPicture.cpp:211-235
// Definition stays with its native .cpp, re-exported by SkPicturePriv namespace.
pub fn PicturePriv_MakeFromBuffer(
    buffer: &mut SkReadBuffer<'_>,
) -> Result<Option<sk_sp<dyn SkPicture>>, crate::RasterError> {
    if !buffer.isValid() {
        return Ok(None);
    }
    let mut info = SkPictInfo::new();
    if !BufferIsSKP(buffer, Some(&mut info)) {
        return Ok(None);
    }
    let ssize = buffer.read32();
    if ssize < 0 {
        let procs = *buffer.getDeserialProcs();
        let Some(proc) = procs.fPictureProc else {
            return Ok(None);
        };
        let size = (-(ssize as i64)) as usize;
        let bytes = buffer.skip(size);
        return Ok(unsafe {
            proc(
                bytes.map_or(core::ptr::null(), |b| b.as_ptr()),
                size,
                procs.fPictureCtx,
            )
        });
    }
    if ssize != 1 {
        return Ok(None);
    }
    let data = SkPictureData::CreateFromBuffer(buffer, &info)?;
    Forwardport(&info, data.as_deref(), Some(buffer))
}
// Source: src/core/SkPicture.cpp:237-244
// Source: include/core/SkPicture.h:285-285
pub fn backport(picture: &dyn SkPicture) -> Result<Option<Box<SkPictureData>>, crate::RasterError> {
    let info = createHeader(picture);
    let mut rec = super::SkPictureRecord::SkPictureRecord::new(&info.fCullRect.roundOut(), 0);
    unsafe { rec.as_mut().beginRecording() }?;
    let canvas = unsafe { NonNull::from(rec.as_mut().canvas().get_unchecked_mut()) };
    unsafe { picture.playback(canvas, None) }?;
    unsafe { rec.as_mut().endRecording() }?;
    Ok(Some(Box::new(SkPictureData::from_record(&rec, &info))))
}
// Source: src/core/SkPicture.cpp:246-248
// Source: include/core/SkPicture.h:192-192
pub fn serialize_stream(
    picture: &dyn SkPicture,
    mut stream: Pin<&mut dyn SkWStream>,
    procs: Option<&SkSerialProcs>,
) -> Result<(), crate::RasterError> {
    unsafe { serialize(picture, stream, procs, None, false) }
}
// Source: src/core/SkPicture.cpp:250-254
// Source: include/core/SkPicture.h:174-174
pub fn serialize_data(
    picture: &dyn SkPicture,
    procs: Option<&SkSerialProcs>,
) -> Result<std::sync::Arc<super::SkData::SkData>, crate::RasterError> {
    let mut stream = core::pin::pin!(super::SkStream::SkDynamicMemoryWStream::new());
    unsafe { serialize(picture, stream.as_mut(), procs, None, false) }?;
    Ok(stream.as_mut().detachAsData())
}
// Source: src/core/SkPicture.cpp:256-268
fn custom_serialize(
    picture: &dyn SkPicture,
    procs: &SkSerialProcs,
) -> Option<std::sync::Arc<super::SkData::SkData>> {
    if let Some(proc) = procs.fPictureProc {
        if let Some(data) = unsafe { proc(picture, procs.fPictureCtx) } {
            let size = data.size();
            if size > i32::MAX as usize || size <= 1 {
                return Some(super::SkData::SkData::MakeEmpty());
            }
            return Some(data);
        }
    }
    None
}
// Source: src/core/SkPicture.cpp:270-279
unsafe fn write_pad32(mut stream: Pin<&mut dyn SkWStream>, data: *const u8, size: usize) -> bool {
    if !unsafe { stream.as_mut().write(data, size) } {
        return false;
    }
    if size & 3 != 0 {
        let zero = 0u32;
        return unsafe {
            stream
                .as_mut()
                .write((&zero as *const u32).cast(), 4 - (size & 3))
        };
    }
    true
}
// Source: src/core/SkPicture.cpp:284-313
// Source: include/core/SkPicture.h:257-258
/// Safety: the borrowed recorder owner retains the actual payload; caller
/// externally synchronizes all its accesses and excludes set-operation reentry
/// through the complete serialization tree and its callbacks.
pub unsafe fn serialize(
    picture: &dyn SkPicture,
    mut stream: Pin<&mut dyn SkWStream>,
    procsPtr: Option<&SkSerialProcs>,
    typefaceSet: Option<&super::SkPtrRecorder::SkRefCntSetOwner>,
    textBlobsOnly: bool,
) -> Result<(), crate::RasterError> {
    let procs = procsPtr.copied().unwrap_or_default();
    let info = createHeader(picture);
    unsafe {
        stream.as_mut().write(
            (&info as *const SkPictInfo).cast(),
            core::mem::size_of::<SkPictInfo>(),
        );
    }
    if let Some(custom) = custom_serialize(picture, &procs) {
        let size = custom.size() as i32;
        if size == 0 {
            stream
                .as_mut()
                .write8(kFailure_TrailingStreamByteAfterPictInfo as u32);
            return Ok(());
        }
        stream
            .as_mut()
            .write8(kCustom_TrailingStreamByteAfterPictInfo as u32);
        stream.as_mut().write32((-size) as u32);
        unsafe {
            write_pad32(stream.as_mut(), custom.data(), size as usize);
        }
        return Ok(());
    }
    let data = backport(picture)?;
    if let Some(data) = data {
        stream
            .as_mut()
            .write8(kPictureData_TrailingStreamByteAfterPictInfo as u32);
        unsafe { data.serialize(stream.as_mut(), &procs, typefaceSet, textBlobsOnly) }?;
    } else {
        stream
            .as_mut()
            .write8(kFailure_TrailingStreamByteAfterPictInfo as u32);
    }
    Ok(())
}
// Source: src/core/SkPicture.cpp:315-336
pub fn PicturePriv_Flatten(
    picture: sk_sp<dyn SkPicture>,
    buffer: &mut dyn super::SkWriteBuffer::SkWriteBuffer,
) -> Result<(), crate::RasterError> {
    let info = createHeader(&*picture);
    let data = backport(&*picture)?;
    unsafe {
        buffer.writeByteArray(info.fMagic.as_ptr(), info.fMagic.len());
    }
    buffer.writeUInt(info.getVersion());
    buffer.writeRect(&info.fCullRect);
    if let Some(custom) = custom_serialize(&*picture, buffer.serialProcs()) {
        let size = custom.size() as i32;
        buffer.write32(-size);
        unsafe {
            buffer.writePad32(custom.data(), size as usize);
        }
        return Ok(());
    }
    if let Some(data) = data {
        buffer.write32(1);
        data.flatten(buffer)?;
    } else {
        buffer.write32(0);
    }
    Ok(())
}

// Source: src/core/SkPicture.cpp:338-355
// Source: include/core/SkPicture.h:206-206
pub fn MakePlaceholder(cull: SkRect) -> sk_sp<dyn SkPicture> {
    // This is the original native empty Picture subclass, with its own actual
    // intrusive Picture base and cull storage, not a recording/playback provider.
    struct Placeholder {
        fCull: SkRect,
        base: SkPictureState,
    }
    unsafe impl SkRefCntObject for Placeholder {
        fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
            crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
        }
        fn ref_cnt_base(&self) -> &SkRefCntBase {
            &self.base.ref_count
        }
        fn ref_(&self) {
            self.base.ref_count.ref_();
        }
    }
    impl SkPicture for Placeholder {
        fn picture_state(&self) -> &SkPictureState {
            &self.base
        }
        unsafe fn playback<'device>(
            &self,
            _: NonNull<SkCanvas<'device>>,
            _: Option<&mut dyn AbortCallback>,
        ) -> Result<(), crate::RasterError> {
            Ok(())
        }
        fn approximateOpCount(&self, _: bool) -> i32 {
            super::SkCanvasPriv::kMaxPictureOpsToUnrollInsteadOfRef + 1
        }
        fn approximateBytesUsed(&self) -> usize {
            core::mem::size_of::<Self>()
        }
        fn cullRect(&self) -> SkRect {
            self.fCull
        }
    }
    let owner = Box::new(Placeholder {
        fCull: cull,
        base: SkPictureState::new(),
    });
    let dynamic: *mut dyn SkPicture = Box::into_raw(owner);
    unsafe { sk_sp::from_raw(dynamic) }
}
