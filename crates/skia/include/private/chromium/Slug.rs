use crate::include::core::SkRefCnt::{SkRefCnt, SkRefCntObject};
use crate::src::core::{SkRect::SkRect, SkWriteBuffer::SkWriteBuffer};
use core::sync::atomic::{AtomicU32, Ordering};
// Source: include/private/chromium/Slug.h:33-33
// Actual inherited counter and native unique-ID member. Initialization remains
// with the native NextUniqueID definition; this header does not invent a factory.
pub struct SlugState {
    pub(crate) fRefCnt: SkRefCnt,
    pub(crate) fUniqueID: u32,
}
// Source: include/private/chromium/Slug.h:57-60
pub trait Slug: SkRefCntObject + 'static {
    fn slug_state(&self) -> &SlugState;
    // Source: include/private/chromium/Slug.h:57
    fn sourceBounds(&self) -> SkRect;
    // Source: include/private/chromium/Slug.h:58
    fn sourceBoundsWithOrigin(&self) -> SkRect;
    // Source: include/private/chromium/Slug.h:60
    fn doFlatten(&self, buffer: &mut dyn SkWriteBuffer);
    // Source: include/private/chromium/Slug.h:62-62
    fn uniqueID(&self) -> u32 {
        self.slug_state().fUniqueID
    }
}
// Source: src/text/SlugFromBuffer.cpp:24-31
pub fn MakeFromBuffer(
    buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
) -> Result<Option<crate::include::core::SkRefCnt::sk_sp<dyn Slug>>, crate::RasterError> {
    let procs = *buffer.getDeserialProcs();
    Ok(procs
        .fSlugProc
        .and_then(|proc| unsafe { proc(buffer, procs.fSlugCtx) }))
}
// Source: src/text/SlugFromBuffer.cpp:33-36
pub fn NextUniqueID() -> u32 {
    static NEXT: AtomicU32 = AtomicU32::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}
// Source: include/private/chromium/Slug.h:65-66
// NextUniqueID and all non-inline factories/serialization/draw definitions are
// pending their actual implementation source. No concrete GPU Slug is fabricated.

// Source: include/private/chromium/Slug.h:37-38
// Pending ordinary static definition: ConvertBlob(SkCanvas*,const SkTextBlob&,
// SkPoint,const SkPaint&) -> sk_sp<Slug>. This is not a virtual producer slot.
// Source: include/private/chromium/Slug.h:41-42
// Pending ordinary overload definitions: serialize() -> sk_sp<SkData> and
// serialize(void*,size_t) -> size_t. No writer/null-success fallback is supplied.
// Source: include/private/chromium/Slug.h:46-49
// Pending ordinary static definitions: Deserialize(const void*,size_t,
// const SkStrikeClient* = nullptr), MakeFromBuffer(SkReadBuffer&) -> sk_sp<Slug>.
// Source: include/private/chromium/Slug.h:52
// Pending AddDeserialProcs(SkDeserialProcs*,const SkStrikeClient* = nullptr).
// Both default client arguments are actual nullable borrows, never a new owner.
// Source: include/private/chromium/Slug.h:55
// Pending nonvirtual draw(SkCanvas*,const SkPaint&) const; not an extra trait slot.
// Source: include/private/chromium/Slug.h:65-66
// Default object construction must invoke the real NextUniqueID before storing
// fUniqueID. Without that definition SlugState has no fake zero/default ctor.
// The inherited selected SkRefCnt is one counter; actual concrete Slug owners
// must project it and its raw disposer, and must override retain through parent.
