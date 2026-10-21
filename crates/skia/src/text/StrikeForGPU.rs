use crate::src::core::SkDescriptor::SkDescriptor;
use crate::src::core::SkDrawable::SkDrawable;
use crate::src::core::SkGlyph::{
    ActionType, SkGlyph, SkGlyphDigest, SkGlyphPositionRoundingSpec, SkPackedGlyphID,
};
use crate::src::core::SkPath::SkPath;
use crate::src::core::SkStrike::SkStrike;
use crate::src::core::SkStrikeSpec::SkStrikeSpec;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::ptr::NonNull;
use std::sync::Arc;

// Source: src/text/StrikeForGPU.h:39-43,66
// Adapted: variant storage uses the existing canonical Arc<SkStrike> owner, without
// another reference counter. Option preserves a null retained strike. No ABI claim.
// Native deleted default/copy construction is preserved by omitting Default/Clone.
// Ordinary methods below translate the admitted common .cpp definitions.
pub struct SkStrikePromise {
    fStrikeOrSpec: StrikeOrSpec,
}
enum StrikeOrSpec {
    Strike(Option<Arc<SkStrike>>),
    Spec(Option<Box<SkStrikeSpec>>),
}

// Source: src/text/StrikeForGPU.h:70-94
// Adapted: pure virtual protocol only. Shared owners use Arc at the existing
// SkStrike owner boundary; this protocol embeds no SkRefCnt or auxiliary counter.
// Raw glyph pointers preserve the native operation lease. Fallibility carries
// canonical CPU allocation/prepare errors rather than manufacturing a success.
pub trait StrikeForGPU {
    fn lock(&self);
    /// Safety: current thread owns the strike lock; all state references ended.
    /// Its original non-owning cache is live through post-unlock accounting.
    unsafe fn unlock(&self);
    unsafe fn digestFor(
        &self,
        action: ActionType,
        id: SkPackedGlyphID,
    ) -> Result<SkGlyphDigest, crate::RasterError>;
    unsafe fn prepareForImage(&self, glyph: *mut SkGlyph) -> Result<bool, crate::RasterError>;
    unsafe fn prepareForPath(&self, glyph: *mut SkGlyph) -> Result<bool, crate::RasterError>;
    unsafe fn prepareForDrawable(&self, glyph: *mut SkGlyph) -> Result<bool, crate::RasterError>;
    fn getDescriptor(&self) -> &SkDescriptor;
    fn roundingSpec(&self) -> &SkGlyphPositionRoundingSpec;
    fn strikePromise(&self) -> SkStrikePromise;
}

// Source: src/text/StrikeForGPU.h:97-105
// Adapted: real overlapping union storage, with explicit active-member duties.
// ManuallyDrop prevents automatic destruction of fPath, just as the native empty
// union destructor does. repr(C) describes Rust member layout, not C++ SkPath ABI.
#[repr(C)]
pub union IDOrPath {
    pub fGlyphID: u16,
    pub fPath: ManuallyDrop<SkPath>,
}
impl IDOrPath {
    // Source: src/text/StrikeForGPU.h:98
    // The empty native constructor activates no member. MaybeUninit preserves
    // that state without producing an invalid initialized Rust union value.
    pub const fn uninit() -> MaybeUninit<Self> {
        MaybeUninit::uninit()
    }
    // Source: src/text/StrikeForGPU.h:99
    pub const fn from_glyph_id(glyph_id: u16) -> Self {
        Self { fGlyphID: glyph_id }
    }
}
// Source: src/text/StrikeForGPU.h:102
impl Drop for IDOrPath {
    fn drop(&mut self) {}
}

// Source: src/text/StrikeForGPU.h:109-112
// Adapted: a nullable, non-owning Rust trait-object pointer replaces the native
// SkDrawable*. It retains no owner and has Rust fat-pointer layout, not C++ ABI.
#[repr(C)]
#[derive(Clone, Copy)]
pub union IDOrDrawable {
    pub fGlyphID: u16,
    pub fDrawable: Option<NonNull<dyn SkDrawable>>,
}

// Source: src/text/StrikeForGPU.h:115-122
// Adapted: non-owning pointer with the native monitor lifetime; methods below
// execute the admitted common constructor/destructor lock/unlock bodies.
pub struct StrikeMutationMonitor<'a> {
    fStrike: Option<NonNull<dyn StrikeForGPU + 'a>>,
}

// Source: src/text/StrikeForGPU.h:125-128
// Adapted: native virtual default destructor maps to concrete owner destruction
// through a Rust trait object; nullable shared ownership uses canonical Arc.
pub trait StrikeForGPUCacheInterface {
    // Safety: this actual cache stays alive and stationary for subsequent
    // cache-using strike operations (notably accounting after unlock). Resource
    // graphs meet the CPU transfer contract. Plain native Strike destruction
    // does not read its cache pointer. Returned owners do not retain this cache;
    // both global and explicitly leased native cache objects preserve identity.
    unsafe fn findOrCreateScopedStrike(
        &self,
        strike_spec: &SkStrikeSpec,
    ) -> Result<Option<Arc<dyn StrikeForGPU>>, crate::RasterError>;
}

impl SkStrikePromise {
    // Source: src/text/StrikeForGPU.cpp:20-20
    // Defaulted variant move preserves the active alternative in the source,
    // leaving its owner null. Rust ordinary moves also transfer without retain.
    pub fn move_from(source: &mut Self) -> Self {
        let fStrikeOrSpec = match &mut source.fStrikeOrSpec {
            StrikeOrSpec::Strike(strike) => StrikeOrSpec::Strike(strike.take()),
            StrikeOrSpec::Spec(spec) => StrikeOrSpec::Spec(spec.take()),
        };
        Self { fStrikeOrSpec }
    }
    // Source: src/text/StrikeForGPU.cpp:21-21
    pub fn move_assign(&mut self, source: &mut Self) {
        *self = Self::move_from(source);
    }
    // Source: src/text/StrikeForGPU.h:46
    // Source: src/text/StrikeForGPU.cpp:22-23
    // The native rvalue-reference constructor can leave its source handle live.
    pub fn from_strike_source(strike: &mut Option<Arc<SkStrike>>) -> Self {
        Self {
            fStrikeOrSpec: StrikeOrSpec::Strike(strike.take()),
        }
    }
    // Source: src/text/StrikeForGPU.cpp:22-23
    pub fn from_strike(strike: Option<Arc<SkStrike>>) -> Self {
        Self {
            fStrikeOrSpec: StrikeOrSpec::Strike(strike),
        }
    }
    // Source: src/text/StrikeForGPU.h:47
    // Source: src/text/StrikeForGPU.cpp:24-25
    pub fn from_spec(spec: &SkStrikeSpec) -> Self {
        Self {
            fStrikeOrSpec: StrikeOrSpec::Spec(Some(Box::new(spec.clone()))),
        }
    }
    // Source: src/text/StrikeForGPU.cpp:27-35
    /// Safety: retained scaler/typeface/effect graphs meet GlobalStrikeCache's
    /// transferable resource contract for their entire cached lifetime.
    pub unsafe fn strike(&mut self) -> Option<&SkStrike> {
        if let StrikeOrSpec::Spec(slot) = &mut self.fStrikeOrSpec {
            // Native exchange happens before cache lookup; the spec alternative
            // remains active with a null owner until lookup finishes.
            let spec = slot.take().expect("native nonnull strike spec");
            let strike = unsafe {
                crate::src::core::SkStrikeCache::SkStrikeCache::GlobalStrikeCache()
                    .findOrCreateStrike(&spec)
            };
            self.fStrikeOrSpec = StrikeOrSpec::Strike(Some(strike));
        }
        match &self.fStrikeOrSpec {
            StrikeOrSpec::Strike(strike) => strike.as_deref(),
            StrikeOrSpec::Spec(_) => unreachable!(),
        }
    }
    // Source: src/text/StrikeForGPU.cpp:37-39
    pub fn resetStrike(&mut self) {
        self.fStrikeOrSpec = StrikeOrSpec::Strike(None);
    }
    // Source: src/text/StrikeForGPU.cpp:41-47
    pub fn descriptor(&self) -> &SkDescriptor {
        match &self.fStrikeOrSpec {
            StrikeOrSpec::Spec(spec) => spec
                .as_deref()
                .expect("native nonnull strike spec")
                .descriptor(),
            StrikeOrSpec::Strike(strike) => strike
                .as_deref()
                .expect("native nonnull strike")
                .getDescriptor(),
        }
    }
    // Source: src/text/StrikeForGPU.cpp:49-51
    // Actual Descriptor::flatten -> WriteBuffer::writePad32 is infallible unit.
    // Keep the established caller Result boundary without inventing writer
    // errors. The unit pattern rejects a future ignored fallible descriptor call.
    pub fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        let () = self.descriptor().flatten(buffer);
        Ok(())
    }
}
impl<'a> StrikeMutationMonitor<'a> {
    // Source: src/text/StrikeForGPU.cpp:54-57
    /// Safety: strike is valid and remains live through monitor destruction;
    /// its native lock/unlock protocol permits this synchronous operation lease.
    /// For CPU strikes, the originating non-owning cache stays stationary and
    /// live through the monitor Drop and its post-unlock memory accounting.
    pub unsafe fn new(strike: NonNull<dyn StrikeForGPU + 'a>) -> Self {
        let result = Self {
            fStrike: Some(strike),
        };
        unsafe { strike.as_ref() }.lock();
        result
    }
}
// Source: src/text/StrikeForGPU.cpp:59-61
impl Drop for StrikeMutationMonitor<'_> {
    fn drop(&mut self) {
        unsafe {
            self.fStrike
                .expect("native nonnull monitored strike")
                .as_ref()
                .unlock()
        };
    }
}

// Source: src/text/StrikeForGPU.h:51-53
// Ordinary static MakeFromBuffer declaration only. Its definition is not in the
// admitted common StrikeForGPU.cpp; no provider/trait or successful decoder is
// manufactured. Its native client and cache are nullable nonowning pointers.
// Pending native body: SkStrikePromise::MakeFromBuffer(SkReadBuffer&,
//     const SkStrikeClient*,SkStrikeCache*) -> optional<SkStrikePromise>.
// Source: src/text/StrikeForGPU.h:97-105
// IDOrPath's user-provided empty destructor does not destroy its path member.
// Its nontrivial path member deletes the implicit C++ union copy operation;
// this Rust union deliberately has neither Copy nor Clone. The existing
// MaybeUninit constructor leaves no active member; placement/drop remain with
// the actual consumer. No C++ SkPath bit-layout/ABI identity is asserted.
// Source: src/text/StrikeForGPU.h:109-112
// IDOrDrawable is a trivially copyable nonowning union. Copy/Clone above copy
// only its active payload bits and never retain/release a drawable owner.
impl<'a> StrikeMutationMonitor<'a> {
    // Source: src/text/StrikeForGPU.h:115-122
    /// Safety: this is the native implicit memberwise copy, not a new lock.
    /// Every copied object's subsequent Drop must independently satisfy the
    /// native unlock precondition. Ordinary use must keep exactly one guard.
    pub unsafe fn copy_from(source: &Self) -> Self {
        Self {
            fStrike: source.fStrike,
        }
    }
}

impl SkStrikePromise {
    // Source: src/text/StrikeForGPU.cpp:21
    /// Safety: live exclusive complete objects. Equal pointers express native
    /// variant self-move; same-alternative intrusive/unique pointers retain
    /// their original pointer under release/reset, without duplicate borrows.
    pub unsafe fn move_assign_raw(dst: NonNull<Self>, source: NonNull<Self>) {
        if dst != source {
            unsafe {
                (&mut *dst.as_ptr()).move_assign(&mut *source.as_ptr());
            }
        }
    }
}
