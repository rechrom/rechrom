// Packed big-endian directory storage. Callers access these fields through
// addr_of!/read_unaligned when reading bytes, never borrowed aligned C objects.
// Source: src/sfnt/SkSFNTHeader.h:15-16
pub type SK_SFNT_USHORT = u16;
pub type SK_SFNT_ULONG = u32;
// Source: src/sfnt/SkSFNTHeader.h:20-62
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkSFNTHeader {
    pub fontType: SK_SFNT_ULONG,
    pub numTables: SK_SFNT_USHORT,
    pub searchRange: SK_SFNT_USHORT,
    pub entrySelector: SK_SFNT_USHORT,
    pub rangeShift: SK_SFNT_USHORT,
}
// Source: src/sfnt/SkSFNTHeader.h:22-28
pub struct fontType_WindowsTrueType;
impl fontType_WindowsTrueType {
    pub const TAG0: i8 = 0;
    pub const TAG1: i8 = 1;
    pub const TAG2: i8 = 0;
    pub const TAG3: i8 = 0;
    pub const TAG: u32 = u32::from_be_bytes([0, 1, 0, 0]).to_be();
}
// Source: src/sfnt/SkSFNTHeader.h:29-35
pub struct fontType_MacTrueType;
impl fontType_MacTrueType {
    pub const TAG0: i8 = b't' as i8;
    pub const TAG1: i8 = b'r' as i8;
    pub const TAG2: i8 = b'u' as i8;
    pub const TAG3: i8 = b'e' as i8;
    pub const TAG: u32 = u32::from_be_bytes(*b"true").to_be();
}
// Source: src/sfnt/SkSFNTHeader.h:36-42
pub struct fontType_PostScript;
impl fontType_PostScript {
    pub const TAG0: i8 = b't' as i8;
    pub const TAG1: i8 = b'y' as i8;
    pub const TAG2: i8 = b'p' as i8;
    pub const TAG3: i8 = b'1' as i8;
    pub const TAG: u32 = u32::from_be_bytes(*b"typ1").to_be();
}
// Source: src/sfnt/SkSFNTHeader.h:43-49
pub struct fontType_OpenTypeCFF;
impl fontType_OpenTypeCFF {
    pub const TAG0: i8 = b'O' as i8;
    pub const TAG1: i8 = b'T' as i8;
    pub const TAG2: i8 = b'T' as i8;
    pub const TAG3: i8 = b'O' as i8;
    pub const TAG: u32 = u32::from_be_bytes(*b"OTTO").to_be();
}
// Source: src/sfnt/SkSFNTHeader.h:56-61
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct TableDirectoryEntry {
    pub tag: u32,
    pub checksum: u32,
    pub offset: u32,
    pub logicalLength: u32,
}
// Source: src/sfnt/SkSFNTHeader.h:67-67
const _: () = assert!(core::mem::size_of::<SkSFNTHeader>() == 12);
// Source: src/sfnt/SkSFNTHeader.h:68-68
const _: () = assert!(core::mem::size_of::<TableDirectoryEntry>() == 16);

// Native template static-member projection, with the same raw big-endian tag.
macro_rules! sfnt_tag_bytes {
    ($ty:ty) => {
        impl crate::src::sfnt::SkOTTableTypes::SkOTTagBytes for $ty {
            const TAG0: i8 = Self::TAG0;
            const TAG1: i8 = Self::TAG1;
            const TAG2: i8 = Self::TAG2;
            const TAG3: i8 = Self::TAG3;
        }
    };
}
sfnt_tag_bytes!(fontType_WindowsTrueType);
sfnt_tag_bytes!(fontType_MacTrueType);
sfnt_tag_bytes!(fontType_PostScript);
sfnt_tag_bytes!(fontType_OpenTypeCFF);
