// Raw OpenType scalars retain their native big-endian storage representation.
// Source: src/sfnt/SkOTTableTypes.h:16-34
pub type SK_OT_BYTE = u8;
pub type SK_OT_CHAR = i8;
pub type SK_OT_SHORT = u16;
pub type SK_OT_USHORT = u16;
pub type SK_OT_ULONG = u32;
pub type SK_OT_LONG = u32;
pub type SK_OT_Fixed = i32;
pub type SK_OT_F2DOT14 = u16;
pub type SK_OT_FWORD = u16;
pub type SK_OT_UFWORD = u16;
pub type SK_OT_LONGDATETIME = u64;
// Source: src/sfnt/SkOTTableTypes.h:38-47
pub trait SkOTTagBytes {
    const TAG0: i8;
    const TAG1: i8;
    const TAG2: i8;
    const TAG3: i8;
}
pub struct SkOTTableTAG<T>(core::marker::PhantomData<T>);
impl<T: SkOTTagBytes> SkOTTableTAG<T> {
    pub const value: u32 =
        u32::from_be_bytes([T::TAG0 as u8, T::TAG1 as u8, T::TAG2 as u8, T::TAG3 as u8]).to_be();
}
// Source: src/sfnt/SkOTTableTypes.h:50-54
pub struct SkOTSetUSHORTBit<const N: u32>;
impl<const N: u32> SkOTSetUSHORTBit<N> {
    pub const bit: u16 = {
        assert!(N < 16);
        1u16 << N
    };
    pub const value: u16 = Self::bit.to_be();
}
// Source: src/sfnt/SkOTTableTypes.h:57-61
pub struct SkOTSetULONGBit<const N: u32>;
impl<const N: u32> SkOTSetULONGBit<N> {
    pub const bit: u32 = {
        assert!(N < 32);
        1u32 << N
    };
    pub const value: u32 = Self::bit.to_be();
}

// Source: src/sfnt/SkOTTableTypes.h:36-36
// Native byte bitfields share their containing byte. A field-fragment template
// therefore names its actual shift/width and operates on that same byte; it
// adds no per-field storage and truncates assignments to the declared width.
pub struct SK_OT_BYTE_BITFIELD<const SHIFT: u32, const WIDTH: u32>;
impl<const SHIFT: u32, const WIDTH: u32> SK_OT_BYTE_BITFIELD<SHIFT, WIDTH> {
    const MASK: u16 = {
        assert!(WIDTH <= 8 && SHIFT <= 8 && SHIFT + WIDTH <= 8);
        (1u16 << WIDTH) - 1
    };
    pub const fn get(storage: u8) -> u8 {
        ((storage as u16 >> SHIFT) & Self::MASK) as u8
    }
    pub fn set(storage: &mut u8, value: u32) {
        let mask = Self::MASK << SHIFT;
        let old = *storage as u16;
        *storage = ((old & !mask) | (((value as u16) & Self::MASK) << SHIFT)) as u8;
    }
}
