// Source: src/core/SkDescriptor.h:25-30
// Rust unique owner: the native three-word header begins at the storage address.
// Layout/capacity live outside those bytes; this wrapper is not a C++ ABI object.
pub struct SkDescriptor {
    words: DescriptorWords,
}
impl SkDescriptor {
    pub fn compute_overhead(entry_count: i32) -> usize {
        crate::SkASSERT!(@source b"src/core/SkDescriptor.h\0".as_ptr(),28; @condition b"entryCount >= 0\0".as_ptr(); entry_count >= 0);
        12 + entry_count as usize * std::mem::size_of::<Entry>()
    }
    // Source: src/core/SkDescriptor.h:42-42
    pub fn length(&self) -> u32 {
        self.words[1]
    }
    // Source: src/core/SkDescriptor.h:50-52
    #[cfg(sk_debug)]
    pub fn assert_checksum(&self) {
        crate::SkASSERT!(@source b"src/core/SkDescriptor.h\0".as_ptr(),51; @condition b"SkDescriptor::ComputeChecksum(this) == fChecksum\0".as_ptr(); self.calculate_checksum() == self.checksum());
    }
    // Source: src/core/SkDescriptor.h:64-64
    pub fn checksum(&self) -> u32 {
        self.words[0]
    }
    // Source: src/core/SkDescriptor.h:71-71
    pub fn count(&self) -> u32 {
        self.words[2]
    }
}

impl SkDescriptor {
    // Source: src/core/SkDescriptor.h:37-38
    // Source: src/core/SkDescriptor.cpp:27-29
    // Ordinary new is forbidden; preserve the actual native abort macro.
    pub unsafe fn operator_new(_size: usize) -> *mut core::ffi::c_void {
        crate::SK_ABORT!(@source b"src/core/SkDescriptor.cpp\0".as_ptr(),28; "Descriptors are created with placement new.");
    }
    // Source: src/core/SkDescriptor.h:38-38
    /// Safety: p is the original suitably aligned exclusive placement storage.
    /// This language allocation operation preserves that address and ownership.
    pub unsafe fn operator_new_placement(
        _size: usize,
        p: *mut core::ffi::c_void,
    ) -> *mut core::ffi::c_void {
        p
    }
    // Source: src/core/SkDescriptor.h:76-76,82-84
    // Only the native three header fields are initialized here. This is not a
    // replacement global allocator, a new Rust Descriptor owner, or tail fill.
    unsafe fn placement_default_header(p: *mut u32) {
        unsafe {
            p.add(0).write(0);
            p.add(1).write(12);
            p.add(2).write(0);
        }
    }
}

// Source: src/core/SkDescriptor.h:66-69
// Native serialized entry header: exactly the two existing aligned u32 words.
#[repr(C)]
pub struct Entry {
    pub fTag: u32,
    pub fLen: u32,
}

// Source: src/core/SkDescriptor.h:87-96
#[must_use]
pub struct SkAutoDescriptor {
    descriptor: Option<SkDescriptor>,
}
// Source: src/core/SkDescriptor.h:103-103
impl SkAutoDescriptor {
    pub fn get_desc(&self) -> &SkDescriptor {
        self.descriptor
            .as_ref()
            .expect("descriptor must be initialized")
    }
    pub fn get_desc_mut(&mut self) -> &mut SkDescriptor {
        self.descriptor
            .as_mut()
            .expect("descriptor must be initialized")
    }
}
// Source: src/core/SkDescriptor.h:107-111
// Source: src/core/SkDescriptor.h:113-115
// The enum owns either aligned inline words or the original heap allocation. Serialized header
// size stays 12; this Rust ownership wrapper does not claim C++ ABI identity.
const AUTO_STORAGE_SIZE: usize = 12
    + 8
    + std::mem::size_of::<crate::src::core::SkScalerContext::SkScalerContextRec>()
    + 8
    + std::mem::size_of::<*const u8>()
    + 32;
// Source: src/core/SkDescriptor.cpp:20-26 (allocation language adaptation)
// Native unsized delete is paired by the unique owner with its original Layout.
// Rust allocation failure follows handle_alloc_error; this does not emulate
// a throwing C++ operator new on targets where allocation throws an exception.
struct DescriptorAllocation {
    pointer: core::ptr::NonNull<u32>,
    layout: std::alloc::Layout,
}
impl Drop for DescriptorAllocation {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.pointer.as_ptr().cast(), self.layout);
        }
    }
}
enum DescriptorWords {
    Inline([core::mem::MaybeUninit<u32>; AUTO_STORAGE_SIZE / 4]),
    Heap(DescriptorAllocation),
}
// Unique allocation/inline bytes contain no borrowed resources. Mutation needs
// exclusive access; shared projections only cover initialized descriptor bytes.
unsafe impl Send for DescriptorWords {}
unsafe impl Sync for DescriptorWords {}
impl DescriptorWords {
    fn as_ptr(&self) -> *const u32 {
        match self {
            Self::Inline(words) => words.as_ptr().cast(),
            Self::Heap(owner) => owner.pointer.as_ptr(),
        }
    }
    fn as_mut_ptr(&mut self) -> *mut u32 {
        match self {
            Self::Inline(words) => words.as_mut_ptr().cast(),
            Self::Heap(owner) => owner.pointer.as_ptr(),
        }
    }
    fn capacity(&self) -> usize {
        match self {
            Self::Inline(_) => AUTO_STORAGE_SIZE,
            Self::Heap(owner) => owner.layout.size(),
        }
    }
}
impl std::ops::Deref for DescriptorWords {
    type Target = [u32];
    fn deref(&self) -> &[u32] {
        // The header is initialized at placement construction. Only the used
        // prefix is exposed; uninitialized allocation tail is never a Rust value.
        unsafe {
            core::slice::from_raw_parts(self.as_ptr(), self.as_ptr().add(1).read() as usize / 4)
        }
    }
}
impl std::ops::DerefMut for DescriptorWords {
    fn deref_mut(&mut self) -> &mut [u32] {
        unsafe {
            let p = self.as_mut_ptr();
            core::slice::from_raw_parts_mut(p, p.add(1).read() as usize / 4)
        }
    }
}

impl SkDescriptor {
    // Source: src/core/SkDescriptor.cpp:20-24
    // Source: src/core/SkDescriptor.h:32-36
    pub fn alloc(length: usize) -> Self {
        crate::SkASSERT!(@source b"src/core/SkDescriptor.cpp\0".as_ptr(),21; @condition b"length >= sizeof(SkDescriptor) && SkIsAlign4(length)\0".as_ptr(); length>=12 && length&3==0);
        // Rust-only precondition for the safe owner API, also in release builds.
        assert!(length >= 12 && length & 3 == 0);
        // Preserve alignment for the descriptor words and its actual core payloads
        // (scaler record and typeface pointer); no foreign allocation ABI claim.
        let alignment = core::mem::align_of::<u32>()
            .max(core::mem::align_of::<usize>())
            .max(core::mem::align_of::<
                crate::src::core::SkScalerContext::SkScalerContextRec,
            >());
        let layout = std::alloc::Layout::from_size_align(length, alignment)
            .expect("descriptor allocation Layout");
        let raw = unsafe { std::alloc::alloc(layout) };
        let pointer = core::ptr::NonNull::new(raw.cast::<u32>())
            .unwrap_or_else(|| std::alloc::handle_alloc_error(layout));
        unsafe {
            let placement = Self::operator_new_placement(12, pointer.as_ptr().cast());
            Self::placement_default_header(placement.cast());
        }
        Self {
            words: DescriptorWords::Heap(DescriptorAllocation { pointer, layout }),
        }
    }
    // Source: src/core/SkDescriptor.h:40-40
    // Source: src/core/SkDescriptor.cpp:31-33
    pub fn flatten(&self, buffer: &mut dyn super::SkWriteBuffer::SkWriteBuffer) {
        // The actual WriteBuffer virtual returns unit, matching native void.
        // A unit pattern makes any future fallible signature change a type
        // mismatch instead of silently discarding a newly introduced Result.
        let () = unsafe { buffer.writePad32(self.words.as_ptr().cast(), self.length() as usize) };
    }
    // Source: src/core/SkDescriptor.cpp:35-38
    // Source: src/core/SkDescriptor.cpp:40-45
    // Source: src/core/SkDescriptor.cpp:47-50
    // Source: src/core/SkDescriptor.h:43-43
    pub fn add_entry(&mut self, tag: u32, length: usize, data: Option<&[u8]>) -> &mut [u8] {
        crate::SkASSERT!(@source b"src/core/SkDescriptor.cpp\0".as_ptr(),36; @condition b"tag\0".as_ptr(); tag != 0);
        crate::SkASSERT!(@source b"src/core/SkDescriptor.cpp\0".as_ptr(),37; @condition b"SkIsAlign4(length)\0".as_ptr(); length & 3 == 0);
        crate::SkASSERT!(@source b"src/core/SkDescriptor.cpp\0".as_ptr(),38; @condition b"this->findEntry(tag, nullptr) == nullptr\0".as_ptr(); self.find_entry(tag).is_none());
        let offset = self.length() as usize;
        let end = offset
            .checked_add(8)
            .and_then(|n| n.checked_add(length))
            .expect("descriptor entry extent");
        assert!(end <= self.words.capacity());
        let stored_len = u32::try_from(length).unwrap();
        let stored_end = u32::try_from(end).unwrap();
        unsafe {
            let entry = self
                .words
                .as_mut_ptr()
                .cast::<u8>()
                .add(offset)
                .cast::<Entry>();
            entry.write(Entry {
                fTag: tag,
                fLen: stored_len,
            });
            let payload = entry.add(1).cast::<u8>();
            if let Some(data) = data {
                assert!(data.len() >= length);
                core::ptr::copy_nonoverlapping(data.as_ptr(), payload, length);
            } else {
                // Language adaptation: native returns writable unspecified bytes.
                // Initialize only this returned payload so the safe Rust byte
                // slice has valid values; unused allocation tail stays uninitialized.
                core::ptr::write_bytes(payload, 0, length);
            }
            let header = self.words.as_mut_ptr();
            header.add(2).write(header.add(2).read() + 1);
            header.add(1).write(stored_end);
            core::slice::from_raw_parts_mut(payload, length)
        }
    }
    // Source: src/core/SkDescriptor.cpp:52-54
    // Source: src/core/SkDescriptor.h:44-44
    pub fn compute_checksum(&mut self) {
        self.words[0] = self.calculate_checksum();
    }
    // Source: src/core/SkDescriptor.cpp:56-58
    // Source: src/core/SkDescriptor.cpp:60-70
    // Source: src/core/SkDescriptor.h:55-55
    pub fn find_entry(&self, tag: u32) -> Option<&[u8]> {
        let mut offset = 12;
        let mut count = self.count() as i32;
        loop {
            count -= 1;
            if count < 0 {
                break;
            }
            let entry = self.entry(offset);
            let length = entry.fLen as usize;
            if entry.fTag == tag {
                return Some(&self.all_bytes()[offset + 8..offset + 8 + length]);
            }
            offset += 8 + length;
        }
        None
    }
    // Source: src/core/SkDescriptor.cpp:72-76
    // Source: src/core/SkDescriptor.h:57-57
    pub fn copy(&self) -> Self {
        let mut desc = Self::alloc(self.length() as usize);
        let length = self.length() as usize;
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.words.as_ptr().cast::<u8>(),
                desc.words.as_mut_ptr().cast::<u8>(),
                length,
            );
        }
        desc
    }
    // Rust-only bounded projections of the native Entry* within the existing
    // inline/heap allocation. No second header, owner, or key is allocated.
    // Slicing retains capacity checks before casting; repr(C) and two u32 fields
    // give exactly the alignment, size and valid bit patterns of these words.
    fn entry(&self, offset: usize) -> &Entry {
        assert_eq!(offset & 3, 0);
        let index = offset / 4;
        let words = &self.words[index..index + 2];
        unsafe { &*words.as_ptr().cast::<Entry>() }
    }
    fn entry_mut(&mut self, offset: usize) -> &mut Entry {
        assert_eq!(offset & 3, 0);
        let index = offset / 4;
        let words = &mut self.words[index..index + 2];
        unsafe { &mut *words.as_mut_ptr().cast::<Entry>() }
    }
    // Initialized used-prefix byte views of the same native allocation.
    fn all_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.words.as_ptr().cast::<u8>(), self.words.len() * 4)
        }
    }
    fn all_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.words.as_mut_ptr().cast::<u8>(),
                self.words.len() * 4,
            )
        }
    }
}
// Source: src/core/SkDescriptor.cpp:78-78
// Source: src/core/SkDescriptor.cpp:83-92
impl PartialEq for SkDescriptor {
    // Source: src/core/SkDescriptor.h:61-61
    fn eq(&self, other: &Self) -> bool {
        crate::SkASSERT!(@source b"src/core/SkDescriptor.cpp\0".as_ptr(),85; @condition b"SkIsAlign4(fLength)\0".as_ptr(); self.length() & 3 == 0);
        let stop = self.length() as usize / 4;
        let mut index = 0;
        loop {
            if self.words[index] != other.words[index] {
                return false;
            }
            index += 1;
            if index >= stop {
                break;
            }
        }
        true
    }
    // Source: src/core/SkDescriptor.h:62-62
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

// Source: src/core/SkDescriptor.h:73-73
// Source: src/core/SkDescriptor.cpp:94-104
impl SkDescriptor {
    pub fn dumpRec(&self) -> crate::src::core::SkString::SkString {
        use crate::src::core::SkScalerContext::{SkScalerContextRec, K_REC_SK_DESCRIPTOR_TAG};
        let rec = self
            .find_entry(K_REC_SK_DESCRIPTOR_TAG)
            .map(|bytes| bytes.as_ptr().cast::<SkScalerContextRec>());
        let mut result = crate::src::core::SkString::SkString::new();
        unsafe {
            result.appendf(b"    Checksum: %x\n\0".as_ptr(), self.checksum());
        }
        if let Some(rec) = rec {
            result.append_string(&unsafe { &*rec }.dump());
        }
        result
    }
}

impl SkDescriptor {
    // Source: src/core/SkDescriptor.cpp:106-110
    // Source: src/core/SkDescriptor.h:80-80
    pub fn calculate_checksum(&self) -> u32 {
        let length = self.length() as usize;
        crate::src::core::SkChecksum::Hash32(&self.all_bytes()[4..length], 0)
    }
    // Source: src/core/SkDescriptor.cpp:112-119
    // Source: src/core/SkDescriptor.cpp:121-125
    // Source: src/core/SkDescriptor.cpp:127-127
    // Source: src/core/SkDescriptor.cpp:129-132
    // Source: src/core/SkDescriptor.cpp:135-137
    // Source: src/core/SkDescriptor.cpp:139-143
    // Source: src/core/SkDescriptor.h:47-47
    pub fn is_valid(&self) -> bool {
        use crate::src::core::SkScalerContext::{SkScalerContextRec, K_REC_SK_DESCRIPTOR_TAG};
        let mut count = self.count();
        let mut length_remaining = self.length() as usize;
        if length_remaining < 12 || length_remaining & 3 != 0 {
            return false;
        }
        length_remaining -= 12;
        let mut offset = 12;
        while length_remaining > 0 && count > 0 {
            if length_remaining < 8 {
                return false;
            }
            length_remaining -= 8;
            let entry = self.entry(offset);
            let entry_tag = entry.fTag;
            let entry_len = entry.fLen as usize;
            if length_remaining < entry_len || entry_len & 3 != 0 {
                return false;
            }
            length_remaining -= entry_len;
            if entry_tag == K_REC_SK_DESCRIPTOR_TAG
                && entry_len != std::mem::size_of::<SkScalerContextRec>()
            {
                return false;
            }
            offset += 8 + entry_len;
            count -= 1;
        }
        length_remaining == 0 && count == 0
    }
}

// Source: src/core/SkDescriptor.cpp:145-145
impl Default for SkAutoDescriptor {
    fn default() -> Self {
        Self { descriptor: None }
    }
}
impl SkAutoDescriptor {
    // Source: src/core/SkDescriptor.cpp:146-147
    pub fn with_size(size: usize) -> Self {
        let mut result = Self::default();
        result.reset(size);
        result
    }
    pub fn from_descriptor(desc: &SkDescriptor) -> Self {
        let mut result = Self::default();
        result.reset_descriptor(desc);
        result
    }
}
// Source: src/core/SkDescriptor.cpp:148-154
impl Clone for SkAutoDescriptor {
    fn clone(&self) -> Self {
        Self::from_descriptor(self.get_desc())
    }
    fn clone_from(&mut self, other: &Self) {
        self.reset_descriptor(other.get_desc());
    }
}
impl SkAutoDescriptor {
    // Source: src/core/SkDescriptor.cpp:155-162
    pub fn take_from(other: &mut Self) -> Self {
        if other
            .descriptor
            .as_ref()
            .is_some_and(|d| matches!(d.words, DescriptorWords::Inline(_)))
        {
            Self::from_descriptor(other.get_desc())
        } else {
            Self {
                descriptor: other.descriptor.take(),
            }
        }
    }
    // Source: src/core/SkDescriptor.cpp:163-172
    pub fn move_from(&mut self, other: &mut Self) {
        if other
            .descriptor
            .as_ref()
            .is_some_and(|d| matches!(d.words, DescriptorWords::Inline(_)))
        {
            self.reset_descriptor(other.get_desc());
        } else {
            self.free();
            self.descriptor = other.descriptor.take();
        }
    }
}
// Source: src/core/SkDescriptor.cpp:174-174
impl Drop for SkAutoDescriptor {
    fn drop(&mut self) {
        self.free();
    }
}
impl SkAutoDescriptor {
    // Source: src/core/SkDescriptor.h:99-99
    // Source: src/core/SkDescriptor.cpp:176-210
    pub fn MakeFromBuffer(buffer: &mut super::SkReadBuffer::SkReadBuffer<'_>) -> Option<Self> {
        // Native descriptor headers are three native-endian u32 words. Keep
        // these serialized bytes independent of Rust's owning enum layout.
        let mut header = [0u32, 12, 0];
        let header_bytes =
            unsafe { std::slice::from_raw_parts_mut(header.as_mut_ptr().cast::<u8>(), 12) };
        if !buffer.readPad32(header_bytes) {
            return None;
        }
        let length = header[1] as usize;
        if length < 12 || length & 3 != 0 {
            return None;
        }
        let body_length = length - 12;
        if body_length > buffer.available() {
            return None;
        }
        let mut ad = Self::with_size(length);
        // readPad32 takes initialized Rust bytes. Initialize its output range
        // before borrowing it, then overwrite with the actual serialized body.
        unsafe {
            core::ptr::write_bytes(
                ad.get_desc_mut().words.as_mut_ptr().cast::<u8>().add(12),
                0,
                body_length,
            );
        }
        ad.get_desc_mut().words[..3].copy_from_slice(&header);
        if !buffer.readPad32(&mut ad.get_desc_mut().all_bytes_mut()[12..length]) {
            return None;
        }
        #[cfg(SK_BUILD_FOR_FUZZER)]
        {
            let _ = ad.get_desc().calculate_checksum();
        }
        #[cfg(not(SK_BUILD_FOR_FUZZER))]
        if ad.get_desc().calculate_checksum() != ad.get_desc().checksum() {
            return None;
        }
        if !ad.get_desc().is_valid() {
            return None;
        }
        Some(ad)
    }
    // Source: src/core/SkDescriptor.cpp:212-219
    // Source: src/core/SkDescriptor.h:101-101
    pub fn reset(&mut self, size: usize) {
        self.free();
        self.descriptor = Some(if size <= AUTO_STORAGE_SIZE {
            let mut words =
                DescriptorWords::Inline([core::mem::MaybeUninit::uninit(); AUTO_STORAGE_SIZE / 4]);
            unsafe {
                let placement = SkDescriptor::operator_new_placement(12, words.as_mut_ptr().cast());
                SkDescriptor::placement_default_header(placement.cast());
            }
            SkDescriptor { words }
        } else {
            SkDescriptor::alloc(size)
        });
    }
    // Source: src/core/SkDescriptor.cpp:221-225
    // Source: src/core/SkDescriptor.h:102-102
    pub fn reset_descriptor(&mut self, desc: &SkDescriptor) {
        let size = desc.length() as usize;
        self.reset(size);
        unsafe {
            core::ptr::copy_nonoverlapping(
                desc.words.as_ptr().cast::<u8>(),
                self.get_desc_mut().words.as_mut_ptr().cast::<u8>(),
                size,
            );
        }
    }
    // Source: src/core/SkDescriptor.cpp:227-233
    // Source: src/core/SkDescriptor.h:106-106
    fn free(&mut self) {
        // Inline storage has trivial native destruction; heap owner performs exactly one paired deallocation.
        drop(self.descriptor.take());
    }
}
