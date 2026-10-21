// Fixed upstream cacf77bdba7ba7df8ea7236d7e14b08c658ff368; unverified.
use super::SkOTTable_name::{Iterator, IteratorRecord, Predefined};
use crate::src::core::SkTypeface::{LocalizedString, LocalizedStrings, SkTypeface};
// Source: src/sfnt/SkOTUtils.cpp:22-30
/// data must address ceil(length/4) readable u32 words, as in the native API.
pub unsafe fn CalcTableChecksum(mut data: *const u32, length: usize) -> u32 {
    let mut sum = 0u32;
    let dataEnd = unsafe { data.add(((length + 3) & !3) / core::mem::size_of::<u32>()) };
    while data < dataEnd {
        sum = sum.wrapping_add(u32::from_be(unsafe { data.read() }));
        data = unsafe { data.add(1) };
    }
    sum
}
// Source: src/sfnt/SkOTUtils.cpp:31-168
/// Safety: stream contains a native valid SFNT allocation, and fontName addresses
/// fontNameLen readable native signed chars. The original algorithm assumes valid
/// table bounds, directory counts, and a nonnegative name length throughout.
pub unsafe fn RenameFont(
    mut fontData: core::pin::Pin<&mut dyn crate::src::core::SkStream::SkStreamAsset>,
    fontName: *const i8,
    fontNameLen: i32,
) -> Option<std::sync::Arc<crate::src::core::SkData::SkData>> {
    use super::{
        SkOTTable_head::SkOTTableHead,
        SkOTTable_name::{PlatformID, Record, SkOTTableName, WindowsEncoding, WindowsLanguage},
        SkSFNTHeader::{SkSFNTHeader, TableDirectoryEntry},
    };
    use crate::src::core::{
        SkData::SkData,
        SkStream::{SkStream, SkStreamAsset, SkStreamRewindable},
    };
    use core::{
        mem::{size_of, MaybeUninit},
        ptr,
    };
    let mut sfntHeader = MaybeUninit::<SkSFNTHeader>::uninit();
    if fontData
        .as_mut()
        .read(sfntHeader.as_mut_ptr().cast(), size_of::<SkSFNTHeader>())
        < size_of::<SkSFNTHeader>()
    {
        return None;
    }
    let sfntHeader = sfntHeader.assume_init();
    let mut tableIndex = 0usize;
    let mut tableEntry = MaybeUninit::<TableDirectoryEntry>::uninit();
    let numTables = u16::from_be(sfntHeader.numTables) as usize;
    while tableIndex < numTables {
        if fontData.as_mut().read(
            tableEntry.as_mut_ptr().cast(),
            size_of::<TableDirectoryEntry>(),
        ) < size_of::<TableDirectoryEntry>()
        {
            return None;
        }
        if SkOTTableName::TAG == ptr::addr_of!((*tableEntry.as_ptr()).tag).read_unaligned() {
            break;
        }
        tableIndex += 1;
    }
    if tableIndex == numTables {
        return None;
    }
    let tableEntry = tableEntry.assume_init();
    if !SkStreamRewindable::rewind(fontData.as_mut()) {
        return None;
    }
    const NAMES: [u16; 5] = [
        Predefined::FontFamilyName,
        Predefined::FontSubfamilyName,
        Predefined::UniqueFontIdentifier,
        Predefined::FullFontName,
        Predefined::PostscriptName,
    ];
    const ENCODINGS: [u16; 2] = [WindowsEncoding::Symbol, WindowsEncoding::UnicodeBMPUCS2];
    let nameTableLogicalSize = size_of::<SkOTTableName>()
        .wrapping_add(ENCODINGS.len() * NAMES.len() * size_of::<Record>())
        .wrapping_add((fontNameLen as usize).wrapping_mul(size_of::<u16>()));
    let nameTablePhysicalSize = nameTableLogicalSize.wrapping_add(3) & !3;
    let oldNameTablePhysicalSize =
        (u32::from_be(tableEntry.logicalLength).wrapping_add(3) & !3) as usize;
    let oldNameTableOffset = u32::from_be(tableEntry.offset) as usize;
    let originalDataSize =
        SkStreamAsset::getLength(&*fontData).wrapping_sub(oldNameTablePhysicalSize);
    let newDataSize = originalDataSize.wrapping_add(nameTablePhysicalSize);
    let rewrittenFontData = SkData::MakeUninitialized(newDataSize);
    let data = rewrittenFontData.writable_data();
    if fontData.as_mut().read(data, oldNameTableOffset) < oldNameTableOffset {
        return None;
    }
    if fontData.as_mut().skip(oldNameTablePhysicalSize) < oldNameTablePhysicalSize {
        return None;
    }
    let remaining = originalDataSize.wrapping_sub(oldNameTableOffset);
    if fontData
        .as_mut()
        .read(data.add(oldNameTableOffset), remaining)
        < remaining
    {
        return None;
    }
    let mut currentEntry = data
        .add(size_of::<SkSFNTHeader>())
        .cast::<TableDirectoryEntry>();
    let endEntry = currentEntry.add(numTables);
    let mut headTableEntry = ptr::null_mut::<TableDirectoryEntry>();
    while currentEntry < endEntry {
        let oldOffset =
            u32::from_be(ptr::addr_of!((*currentEntry).offset).read_unaligned()) as usize;
        if oldOffset > oldNameTableOffset {
            ptr::addr_of_mut!((*currentEntry).offset)
                .write_unaligned((oldOffset.wrapping_sub(oldNameTablePhysicalSize) as u32).to_be());
        }
        if SkOTTableHead::TAG == ptr::addr_of!((*currentEntry).tag).read_unaligned() {
            headTableEntry = currentEntry;
        }
        currentEntry = currentEntry.add(1);
    }
    let nameTableEntry = data
        .add(size_of::<SkSFNTHeader>())
        .cast::<TableDirectoryEntry>()
        .add(tableIndex);
    ptr::addr_of_mut!((*nameTableEntry).logicalLength)
        .write_unaligned((nameTableLogicalSize as u32).to_be());
    ptr::addr_of_mut!((*nameTableEntry).offset).write_unaligned((originalDataSize as u32).to_be());
    let nameTable = data.add(originalDataSize).cast::<SkOTTableName>();
    let stringOffset =
        (size_of::<SkOTTableName>() + ENCODINGS.len() * NAMES.len() * size_of::<Record>()) as u16;
    ptr::addr_of_mut!((*nameTable).format).write_unaligned(SkOTTableName::format_0);
    ptr::addr_of_mut!((*nameTable).count)
        .write_unaligned(((ENCODINGS.len() * NAMES.len()) as u16).to_be());
    ptr::addr_of_mut!((*nameTable).stringOffset).write_unaligned(stringOffset.to_be());
    let mut nameRecord = data
        .add(originalDataSize + size_of::<SkOTTableName>())
        .cast::<Record>();
    for encoding in ENCODINGS {
        for name in NAMES {
            ptr::addr_of_mut!((*nameRecord).platformID).write_unaligned(PlatformID::Windows);
            ptr::addr_of_mut!((*nameRecord).encodingID).write_unaligned(encoding);
            ptr::addr_of_mut!((*nameRecord).languageID)
                .write_unaligned(WindowsLanguage::English_UnitedStates);
            ptr::addr_of_mut!((*nameRecord).nameID).write_unaligned(name);
            ptr::addr_of_mut!((*nameRecord).offset).write_unaligned(0u16.to_be());
            ptr::addr_of_mut!((*nameRecord).length)
                .write_unaligned(((fontNameLen as usize * size_of::<u16>()) as u16).to_be());
            nameRecord = nameRecord.add(1);
        }
    }
    let nameString = data
        .add(originalDataSize + stringOffset as usize)
        .cast::<u16>();
    for i in 0..fontNameLen {
        nameString
            .add(i as usize)
            .write_unaligned((*fontName.add(i as usize) as u16).to_be());
    }
    let mut logical = data.add(originalDataSize + nameTableLogicalSize);
    let physical = data.add(originalDataSize + nameTablePhysicalSize);
    while logical < physical {
        logical.write(0);
        logical = logical.add(1);
    }
    ptr::addr_of_mut!((*nameTableEntry).checksum)
        .write_unaligned(CalcTableChecksum(nameTable.cast(), nameTableLogicalSize).to_be());
    if !headTableEntry.is_null() {
        let headTableOffset =
            u32::from_be(ptr::addr_of!((*headTableEntry).offset).read_unaligned()) as usize;
        if headTableOffset.wrapping_add(size_of::<SkOTTableHead>()) < originalDataSize {
            let headTable = data.add(headTableOffset).cast::<SkOTTableHead>();
            ptr::addr_of_mut!((*headTable).checksumAdjustment).write_unaligned(0u32.to_be());
            let unadjustedFontChecksum =
                CalcTableChecksum(data.cast(), originalDataSize + nameTablePhysicalSize);
            ptr::addr_of_mut!((*headTable).checksumAdjustment).write_unaligned(
                SkOTTableHead::fontChecksum
                    .wrapping_sub(unadjustedFontChecksum)
                    .to_be(),
            );
        }
    }
    Some(rewrittenFontData)
}

// Source: src/sfnt/SkOTUtils.h:43-77
// Box retains the stable table allocation; the native borrowed type list is an
// explicit borrowed pointer. Unsafe constructors require the native caller
// to preserve its type-list allocation and values across every reset/next.
pub struct LocalizedStrings_NameTable {
    fTypes: core::ptr::NonNull<u16>,
    fTypesCount: usize,
    fTypesIndex: usize,
    fNameTableData: Box<[u8]>,
    fFamilyNameIter: Iterator,
}
impl LocalizedStrings_NameTable {
    // Source: src/sfnt/SkOTUtils.h:46-53
    pub fn new(nameTableData: Box<[u8]>, types: &'static [u16]) -> Self {
        unsafe { Self::new_borrowed(nameTableData, types.as_ptr(), types.len()) }
    }
    /// Safety: types addresses at least typesCount initialized native-endian u16s;
    /// typesCount is positive and the list remains alive and unchanged until this
    /// iterator is destroyed. This is the native non-owning SK_OT_USHORT* lease.
    pub unsafe fn new_borrowed(
        nameTableData: Box<[u8]>,
        types: *const u16,
        typesCount: usize,
    ) -> Self {
        let iter = Iterator::with_type(nameTableData.as_ptr(), nameTableData.len(), *types);
        Self {
            fTypes: core::ptr::NonNull::new_unchecked(types.cast_mut()),
            fTypesCount: typesCount,
            fTypesIndex: 0,
            fNameTableData: nameTableData,
            fFamilyNameIter: iter,
        }
    }
    // Source: src/sfnt/SkOTUtils.cpp:170-189
    pub fn Make(typeface: &SkTypeface, types: &'static [u16]) -> Option<Box<Self>> {
        unsafe { Self::Make_borrowed(typeface, types.as_ptr(), types.len()) }
    }
    /// Safety: same non-owning native type-list lease as new_borrowed.
    pub unsafe fn Make_borrowed(
        typeface: &SkTypeface,
        types: *const u16,
        typesCount: usize,
    ) -> Option<Box<Self>> {
        let nameTag = u32::from_be_bytes(*b"name");
        let nameTableSize = typeface.getTableSize(nameTag);
        if nameTableSize == 0 {
            return None;
        }
        // The safe table-data API takes an initialized byte slice. Initialization is
        // a Rust storage adaptation only: the exact native copied-size check remains,
        // so no short table read is ever published to the name iterator.
        let mut nameTableData = vec![0u8; nameTableSize].into_boxed_slice();
        let copied = typeface.getTableData(nameTag, 0, nameTableSize, Some(&mut nameTableData));
        if copied != nameTableSize {
            return None;
        }
        Some(Box::new(Self::new_borrowed(
            nameTableData,
            types,
            typesCount,
        )))
    }
    // Source: src/sfnt/SkOTUtils.cpp:191-196
    pub fn MakeForFamilyNames(typeface: &SkTypeface) -> Option<Box<Self>> {
        Self::Make(typeface, &familyNameTypes)
    }
}
impl LocalizedStrings for LocalizedStrings_NameTable {
    // Source: src/sfnt/SkOTUtils.cpp:198-212
    fn next(&mut self, localizedString: &mut LocalizedString) -> bool {
        loop {
            let mut record = IteratorRecord::default();
            if self.fFamilyNameIter.next(&mut record) {
                localizedString.fString = record.name;
                localizedString.fLanguage = record.language;
                return true;
            }
            if self.fTypesCount == self.fTypesIndex + 1 {
                return false;
            }
            self.fTypesIndex += 1;
            self.fFamilyNameIter
                .reset(unsafe { *self.fTypes.as_ptr().add(self.fTypesIndex) });
        }
    }
}
// Source: src/sfnt/SkOTUtils.cpp:214-218
static familyNameTypes: [u16; 3] = [
    Predefined::FontFamilyName,
    Predefined::PreferredFamily,
    Predefined::WWSFamilyName,
];
// Source: src/sfnt/SkOTUtils.h:79-98
pub struct LocalizedStrings_SingleName {
    fName: String,
    fLanguage: String,
    fHasNext: bool,
}
impl LocalizedStrings_SingleName {
    pub fn new(name: String, language: String) -> Self {
        Self {
            fName: name,
            fLanguage: language,
            fHasNext: true,
        }
    }
}
impl LocalizedStrings for LocalizedStrings_SingleName {
    fn next(&mut self, localizedString: &mut LocalizedString) -> bool {
        localizedString.fString = self.fName.clone();
        localizedString.fLanguage = self.fLanguage.clone();
        let hadNext = self.fHasNext;
        self.fHasNext = false;
        hadNext
    }
}
// Source: src/sfnt/SkOTUtils.h:100-101
// Source: src/sfnt/SkOTUtils.cpp:219-231
pub fn SetAdvancedTypefaceFlags(
    fsType: super::SkOTTable_OS_2_V4::SkOTTableOS2_V4_Type,
    info: &mut crate::src::core::SkAdvancedTypefaceMetrics::SkAdvancedTypefaceMetrics,
) {
    use crate::src::core::SkAdvancedTypefaceMetrics::FontFlags;
    // info is nonnull through the native exclusive reference. The two real union
    // projections inspect the same complete two-byte V4 fsType storage.
    if unsafe { fsType.raw.value } != 0 {
        let field = unsafe { fsType.field };
        if field.Restricted() || field.Bitmap() {
            info.fFlags |= FontFlags::kNotEmbeddable_FontFlag;
        }
        if field.NoSubsetting() {
            info.fFlags |= FontFlags::kNotSubsettable_FontFlag;
        }
    }
}
