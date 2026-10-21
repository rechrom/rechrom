//! CPU retained identity and native factory/flatten protocols.
use crate::include::core::SkRefCnt::SkRefCntObject;
// Source: include/core/SkFlattenable.h:30-39
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Type {
    kSkColorFilter_Type,
    kSkBlender_Type,
    kSkDrawable_Type,
    kSkDrawLooper_Type,
    kSkImageFilter_Type,
    kSkMaskFilter_Type,
    kSkPathEffect_Type,
    kSkShader_Type,
}
// Source: include/core/SkFlattenable.h:41-41
// Adapted: preserve the existing concrete Arc or intrusive owner; no proxy allocation.
pub type Factory = for<'a> fn(&mut super::SkReadBuffer::SkReadBuffer<'a>) -> FactoryResult;
pub type FactoryResult = Result<Option<FlattenableOwner>, crate::RasterError>;
// Source: include/core/SkFlattenable.h:49-69
// Borrowed native virtual protocol, independent of its concrete reference-count storage.
// This readonly dispatch does not add Send/Sync to the actual retained payload;
// the factory result keeps its existing concrete Arc or sole intrusive owner.
pub trait SkFlattenableView {
    fn getFactory(&self) -> Option<Factory>;
    fn getTypeName(&self) -> Option<&str>;
    // Source: include/core/SkFlattenable.h:67. The actual native base body is empty.
    fn flatten(
        &self,
        writer: &mut dyn super::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        flatten_base(writer)
    }
    fn getFlattenableType(&self) -> Type;
}
// Rust ownership projection only; the actual native intrusive base remains sole owner.
pub trait SkFlattenable: SkRefCntObject + SkFlattenableView {
    // Rust metadata projection of the native checked downcast. Non-drawable
    // types have no such base; actual Drawable complete owners override this.
    unsafe fn project_drawable_owner(
        &self,
        _allocation: std::ptr::NonNull<()>,
    ) -> Option<std::ptr::NonNull<dyn super::SkDrawable::SkDrawable>> {
        None
    }
    unsafe fn project_path_effect_owner(
        &self,
        allocation: std::ptr::NonNull<()>,
    ) -> Option<std::ptr::NonNull<dyn super::SkPathEffect::SkPathEffect>>;
}
pub enum FlattenableOwner {
    Intrusive(crate::include::core::SkRefCnt::sk_sp<dyn SkFlattenable>),
    Shader(std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>),
    ColorFilter(std::sync::Arc<dyn super::SkColorFilter::SkColorFilter>),
    ImageFilter(std::sync::Arc<dyn super::SkImageFilter::SkImageFilter>),
    MaskFilter(std::sync::Arc<dyn super::SkMaskFilter::SkMaskFilter>),
    Blender(crate::include::core::SkRefCnt::sk_sp<dyn crate::include::core::SkBlender::SkBlender>),
}
impl FlattenableOwner {
    pub fn as_view(&self) -> &dyn SkFlattenableView {
        match self {
            Self::Intrusive(owner) => owner.get().expect("factory owner must be nonnull"),
            Self::Shader(owner) => &**owner,
            Self::ColorFilter(owner) => &**owner,
            Self::ImageFilter(owner) => &**owner,
            Self::MaskFilter(owner) => &**owner,
            Self::Blender(owner) => owner.get().expect("factory owner must be nonnull"),
        }
    }
}
impl super::SkPtrRecorder::SkNamedFactorySet {
    // Source: src/core/SkPtrRecorder.h:150-150
    // Source: src/core/SkFlattenable.cpp:25-25
    pub fn new() -> Self {
        Self {
            fNextAddedFactory: 0,
            fFactorySet: super::SkPtrRecorder::SkFactorySet::default(),
            fNames: Vec::new(),
        }
    }

    // Source: src/core/SkPtrRecorder.h:157-157
    // Source: src/core/SkFlattenable.cpp:27-38
    pub fn find(&mut self, factory: Option<Factory>) -> u32 {
        let index = self.fFactorySet.find_nullable(factory);
        if index > 0 {
            return index;
        }
        let Some(name) = FactoryToNameNullable(factory) else {
            return 0;
        };
        self.fNames.push(name);
        self.fFactorySet
            .add(factory.expect("registered factories are nonnull"))
    }

    // Source: src/core/SkPtrRecorder.h:164-164
    // Source: src/core/SkFlattenable.cpp:40-45
    pub fn getNextAddedFactoryName(&mut self) -> Option<&'static [u8]> {
        if self.fNextAddedFactory < self.fNames.len() {
            let name = self.fNames[self.fNextAddedFactory];
            self.fNextAddedFactory += 1;
            return Some(name);
        }
        None
    }
}

// Source: src/core/SkFlattenable.cpp:65-69
#[derive(Clone, Copy)]
struct Entry {
    name: &'static [u8],
    factory: Factory,
}
// Source: src/core/SkFlattenable.cpp:71-81
// Registered names and lookup keys are the bytes before the first NUL.
// Byte-slice ordering therefore preserves strcmp's unsigned-byte ordering.
struct EntryComparator;
impl EntryComparator {
    fn entries(a: &Entry, b: &Entry) -> bool {
        a.name < b.name
    }
    fn entry_name(a: &Entry, b: &[u8]) -> bool {
        a.name < b
    }
    fn name_entry(a: &[u8], b: &Entry) -> bool {
        a < b.name
    }
}
// Source: src/core/SkFlattenable.cpp:83-84
// Native fixed-capacity table and count. Only the initialization thread writes;
// RegisterFlattenablesIfNeeded publishes the finalized immutable prefix. There
// is no native per-operation mutex, and no heap Vec or substitute lock here.
struct RegistryStorage {
    count: core::cell::UnsafeCell<i32>,
    entries: core::cell::UnsafeCell<[core::mem::MaybeUninit<Entry>; 128]>,
}
// Safety relies on the native registration phase contract: all mutation is
// exclusive and ends before the global initialization once publishes completion.
unsafe impl Sync for RegistryStorage {}
static REGISTRY: RegistryStorage = RegistryStorage {
    count: core::cell::UnsafeCell::new(0),
    entries: core::cell::UnsafeCell::new([core::mem::MaybeUninit::uninit(); 128]),
};
// Source: src/core/SkFlattenable.cpp:88-90
/// Safety: exclusive initialization phase; no lookup or registration overlaps.
/// Exactly count entries have been fully initialized by Register.
pub unsafe fn Finalize() {
    let count = unsafe { *REGISTRY.count.get() } as usize;
    let entries =
        unsafe { core::slice::from_raw_parts_mut(REGISTRY.entries.get().cast::<Entry>(), count) };
    entries.sort_unstable_by(|a, b| {
        if EntryComparator::entries(a, b) {
            core::cmp::Ordering::Less
        } else if EntryComparator::entries(b, a) {
            core::cmp::Ordering::Greater
        } else {
            core::cmp::Ordering::Equal
        }
    });
}
// Source: src/core/SkFlattenable.cpp:92-100
/// Safety: native exclusive registration phase, before publication and before
/// concurrent readers. The actual fixed table has fewer than 128 used entries;
/// registered name bytes remain live forever. Native unchecked capacity is not
/// converted into a successful overflow or a growable replacement registry.
pub unsafe fn Register(name: &'static [u8], factory: Factory) {
    let count = unsafe { *REGISTRY.count.get() };
    debug_assert!(count < 128);
    let name = name.split(|b| *b == 0).next().unwrap();
    let entry = unsafe { REGISTRY.entries.get().cast::<Entry>().add(count as usize) };
    unsafe {
        core::ptr::addr_of_mut!((*entry).name).write(name);
        core::ptr::addr_of_mut!((*entry).factory).write(factory);
        *REGISTRY.count.get() = count + 1;
    }
}
// Source: src/core/SkFlattenable.cpp:102-111
pub fn NameToFactory(name: &[u8]) -> Option<Factory> {
    let name = name.split(|b| *b == 0).next().unwrap();
    super::SkGlobalInitialization_core::RegisterFlattenablesIfNeeded();
    let entries = unsafe {
        core::slice::from_raw_parts(
            REGISTRY.entries.get().cast::<Entry>(),
            *REGISTRY.count.get() as usize,
        )
    };
    debug_assert!(entries
        .windows(2)
        .all(|p| !EntryComparator::entries(&p[1], &p[0])));
    let first = entries.partition_point(|entry| EntryComparator::entry_name(entry, name));
    let last =
        first + entries[first..].partition_point(|entry| !EntryComparator::name_entry(name, entry));
    if first == last {
        None
    } else {
        Some(entries[first].factory)
    }
}
// Source: src/core/SkFlattenable.cpp:113-123
pub fn FactoryToName(factory: Factory) -> Option<&'static [u8]> {
    FactoryToNameNullable(Some(factory))
}
// Source: src/core/SkFlattenable.cpp:113-123
// Null factory still performs actual global initialization before lookup.
pub fn FactoryToNameNullable(factory: Option<Factory>) -> Option<&'static [u8]> {
    super::SkGlobalInitialization_core::RegisterFlattenablesIfNeeded();
    let entries = unsafe {
        core::slice::from_raw_parts(
            REGISTRY.entries.get().cast::<Entry>(),
            *REGISTRY.count.get() as usize,
        )
    };
    entries
        .iter()
        .rev()
        .find(|entry| factory.is_some_and(|factory| core::ptr::fn_addr_eq(entry.factory, factory)))
        .map(|entry| entry.name)
}

// Source: include/core/SkFlattenable.h:67-67
// Actual native base implementation; subclasses with data supply their real override.
pub fn flatten_base(
    _writer: &mut dyn super::SkWriteBuffer::SkWriteBuffer,
) -> Result<(), crate::RasterError> {
    Ok(())
}

// Source: src/core/SkFlattenable.cpp:127-139
impl dyn SkFlattenableView + '_ {
    pub fn serialize(
        &self,
        procs: Option<&crate::include::core::SkSerialProcs::SkSerialProcs>,
    ) -> Result<std::sync::Arc<super::SkData::SkData>, crate::RasterError> {
        use super::SkWriteBuffer::SkWriteBuffer;
        let p = procs.copied().unwrap_or_default();
        let mut writer = super::SkWriteBuffer::SkBinaryWriteBuffer::new(p);
        writer.writeFlattenable(Some(self))?;
        let size = writer.bytesWritten();
        // The actual writer fills every byte before this newly allocated owner is published.
        let data = unsafe { super::SkData::SkData::MakeUninitialized(size) };
        unsafe {
            writer.writeToMemory(data.writable_data());
        }
        Ok(data)
    }
    // Source: src/core/SkFlattenable.cpp:141-150
    /// Safety: memory is exclusive writable storage for memory_size bytes until serialization returns.
    pub unsafe fn serialize_into(
        &self,
        memory: *mut u8,
        memory_size: usize,
        procs: Option<&crate::include::core::SkSerialProcs::SkSerialProcs>,
    ) -> Result<usize, crate::RasterError> {
        use super::SkWriteBuffer::SkWriteBuffer;
        let p = procs.copied().unwrap_or_default();
        let mut writer = unsafe {
            super::SkWriteBuffer::SkBinaryWriteBuffer::with_storage(memory, memory_size, p)
        };
        writer.writeFlattenable(Some(self))?;
        Ok(if writer.usingInitialStorage() {
            writer.bytesWritten()
        } else {
            0
        })
    }
}
// Source: src/core/SkFlattenable.cpp:152-159
pub fn Deserialize(
    expected: Type,
    data: &[u8],
    procs: Option<&crate::include::core::SkSerialProcs::SkDeserialProcs>,
) -> FactoryResult {
    let mut buffer = super::SkReadBuffer::SkReadBuffer::new(data);
    if let Some(procs) = procs {
        buffer.setDeserialProcs(*procs);
    }
    buffer.readFlattenable(expected)
}

// Rust-only forwarding keeps existing intrusive call sites on their canonical owner.
impl dyn SkFlattenable + '_ {
    pub fn serialize(
        &self,
        procs: Option<&crate::include::core::SkSerialProcs::SkSerialProcs>,
    ) -> Result<std::sync::Arc<super::SkData::SkData>, crate::RasterError> {
        (self as &dyn SkFlattenableView).serialize(procs)
    }
    pub unsafe fn serialize_into(
        &self,
        memory: *mut u8,
        size: usize,
        procs: Option<&crate::include::core::SkSerialProcs::SkSerialProcs>,
    ) -> Result<usize, crate::RasterError> {
        unsafe { (self as &dyn SkFlattenableView).serialize_into(memory, size, procs) }
    }
}

// Source: include/core/SkFlattenable.h:81-84
pub(crate) struct PrivateInitializer;
impl PrivateInitializer {
    pub(crate) unsafe fn InitEffects() {
        unsafe {
            crate::src::ports::SkGlobalInitialization_default::InitEffects();
        }
    }
    pub(crate) unsafe fn InitImageFilters() {
        unsafe {
            crate::src::ports::SkGlobalInitialization_default::InitImageFilters();
        }
    }
}
// Source: include/core/SkFlattenable.h:88-88
pub(crate) fn RegisterFlattenablesIfNeeded() {
    super::SkGlobalInitialization_core::RegisterFlattenablesIfNeeded();
}
// Source: include/core/SkFlattenable.h:74-78
impl dyn SkFlattenableView + '_ {
    pub fn serialize_default(
        &self,
    ) -> Result<std::sync::Arc<super::SkData::SkData>, crate::RasterError> {
        self.serialize(None)
    }
    /// Safety: same exclusive output storage contract as serialize_into.
    pub unsafe fn serialize_into_default(
        &self,
        memory: *mut u8,
        size: usize,
    ) -> Result<usize, crate::RasterError> {
        unsafe { self.serialize_into(memory, size, None) }
    }
}
pub fn Deserialize_default(expected: Type, data: &[u8]) -> FactoryResult {
    Deserialize(expected, data, None)
}

// Source: include/core/SkFlattenable.h:98-112
// CreateProc remains an actual associated function with its native body. These
// metadata hooks neither implement a missing constructor nor manufacture one.
#[cfg(sk_disable_effect_deserialization)]
#[macro_export]
macro_rules! SK_REGISTER_FLATTENABLE {
    ($type:ident) => {{}};
}
#[cfg(not(sk_disable_effect_deserialization))]
#[macro_export]
macro_rules! SK_REGISTER_FLATTENABLE {
    ($type:ident) => {
        $crate::src::core::SkFlattenable::Register(stringify!($type).as_bytes(), $type::CreateProc)
    };
}
#[cfg(sk_disable_effect_deserialization)]
#[macro_export]
macro_rules! SK_FLATTENABLE_HOOKS {
    ($type:ident) => {
        fn getFactory(&self) -> Option<$crate::src::core::SkFlattenable::Factory> {
            None
        }
        fn getTypeName(&self) -> Option<&str> {
            Some(stringify!($type))
        }
    };
}
#[cfg(not(sk_disable_effect_deserialization))]
#[macro_export]
macro_rules! SK_FLATTENABLE_HOOKS {
    ($type:ident) => {
        fn getFactory(&self) -> Option<$crate::src::core::SkFlattenable::Factory> {
            Some($type::CreateProc)
        }
        fn getTypeName(&self) -> Option<&str> {
            Some(stringify!($type))
        }
    };
}
