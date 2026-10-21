#![allow(non_snake_case)]

// cpp: foundation/blink_base/wtf/text/bidi_class_data.h:13-14,16-736
// Exact Unicode 17.0 Bidi_Class ranges from the source header.
#[derive(Clone, Copy)]
struct BidiClassRange {
    first: u32,
    last: u32,
    value: u8,
}

const BIDI_CLASS_RANGES: [BidiClassRange; 721] = [
    BidiClassRange {
        first: 0x0,
        last: 0x8,
        value: 18,
    },
    BidiClassRange {
        first: 0x9,
        last: 0x9,
        value: 8,
    },
    BidiClassRange {
        first: 0xA,
        last: 0xA,
        value: 7,
    },
    BidiClassRange {
        first: 0xB,
        last: 0xB,
        value: 8,
    },
    BidiClassRange {
        first: 0xC,
        last: 0xC,
        value: 9,
    },
    BidiClassRange {
        first: 0xD,
        last: 0xD,
        value: 7,
    },
    BidiClassRange {
        first: 0xE,
        last: 0x1B,
        value: 18,
    },
    BidiClassRange {
        first: 0x1C,
        last: 0x1E,
        value: 7,
    },
    BidiClassRange {
        first: 0x1F,
        last: 0x1F,
        value: 8,
    },
    BidiClassRange {
        first: 0x20,
        last: 0x20,
        value: 9,
    },
    BidiClassRange {
        first: 0x21,
        last: 0x22,
        value: 10,
    },
    BidiClassRange {
        first: 0x23,
        last: 0x25,
        value: 4,
    },
    BidiClassRange {
        first: 0x26,
        last: 0x2A,
        value: 10,
    },
    BidiClassRange {
        first: 0x2B,
        last: 0x2B,
        value: 3,
    },
    BidiClassRange {
        first: 0x2C,
        last: 0x2C,
        value: 6,
    },
    BidiClassRange {
        first: 0x2D,
        last: 0x2D,
        value: 3,
    },
    BidiClassRange {
        first: 0x2E,
        last: 0x2F,
        value: 6,
    },
    BidiClassRange {
        first: 0x30,
        last: 0x39,
        value: 2,
    },
    BidiClassRange {
        first: 0x3A,
        last: 0x3A,
        value: 6,
    },
    BidiClassRange {
        first: 0x3B,
        last: 0x40,
        value: 10,
    },
    BidiClassRange {
        first: 0x5B,
        last: 0x60,
        value: 10,
    },
    BidiClassRange {
        first: 0x7B,
        last: 0x7E,
        value: 10,
    },
    BidiClassRange {
        first: 0x7F,
        last: 0x84,
        value: 18,
    },
    BidiClassRange {
        first: 0x85,
        last: 0x85,
        value: 7,
    },
    BidiClassRange {
        first: 0x86,
        last: 0x9F,
        value: 18,
    },
    BidiClassRange {
        first: 0xA0,
        last: 0xA0,
        value: 6,
    },
    BidiClassRange {
        first: 0xA1,
        last: 0xA1,
        value: 10,
    },
    BidiClassRange {
        first: 0xA2,
        last: 0xA5,
        value: 4,
    },
    BidiClassRange {
        first: 0xA6,
        last: 0xA9,
        value: 10,
    },
    BidiClassRange {
        first: 0xAB,
        last: 0xAC,
        value: 10,
    },
    BidiClassRange {
        first: 0xAD,
        last: 0xAD,
        value: 18,
    },
    BidiClassRange {
        first: 0xAE,
        last: 0xAF,
        value: 10,
    },
    BidiClassRange {
        first: 0xB0,
        last: 0xB1,
        value: 4,
    },
    BidiClassRange {
        first: 0xB2,
        last: 0xB3,
        value: 2,
    },
    BidiClassRange {
        first: 0xB4,
        last: 0xB4,
        value: 10,
    },
    BidiClassRange {
        first: 0xB6,
        last: 0xB8,
        value: 10,
    },
    BidiClassRange {
        first: 0xB9,
        last: 0xB9,
        value: 2,
    },
    BidiClassRange {
        first: 0xBB,
        last: 0xBF,
        value: 10,
    },
    BidiClassRange {
        first: 0xD7,
        last: 0xD7,
        value: 10,
    },
    BidiClassRange {
        first: 0xF7,
        last: 0xF7,
        value: 10,
    },
    BidiClassRange {
        first: 0x2B9,
        last: 0x2BA,
        value: 10,
    },
    BidiClassRange {
        first: 0x2C2,
        last: 0x2CF,
        value: 10,
    },
    BidiClassRange {
        first: 0x2D2,
        last: 0x2DF,
        value: 10,
    },
    BidiClassRange {
        first: 0x2E5,
        last: 0x2ED,
        value: 10,
    },
    BidiClassRange {
        first: 0x2EF,
        last: 0x2FF,
        value: 10,
    },
    BidiClassRange {
        first: 0x300,
        last: 0x36F,
        value: 17,
    },
    BidiClassRange {
        first: 0x374,
        last: 0x375,
        value: 10,
    },
    BidiClassRange {
        first: 0x37E,
        last: 0x37E,
        value: 10,
    },
    BidiClassRange {
        first: 0x384,
        last: 0x385,
        value: 10,
    },
    BidiClassRange {
        first: 0x387,
        last: 0x387,
        value: 10,
    },
    BidiClassRange {
        first: 0x3F6,
        last: 0x3F6,
        value: 10,
    },
    BidiClassRange {
        first: 0x483,
        last: 0x489,
        value: 17,
    },
    BidiClassRange {
        first: 0x58A,
        last: 0x58A,
        value: 10,
    },
    BidiClassRange {
        first: 0x58D,
        last: 0x58E,
        value: 10,
    },
    BidiClassRange {
        first: 0x58F,
        last: 0x58F,
        value: 4,
    },
    BidiClassRange {
        first: 0x590,
        last: 0x590,
        value: 1,
    },
    BidiClassRange {
        first: 0x591,
        last: 0x5BD,
        value: 17,
    },
    BidiClassRange {
        first: 0x5BE,
        last: 0x5BE,
        value: 1,
    },
    BidiClassRange {
        first: 0x5BF,
        last: 0x5BF,
        value: 17,
    },
    BidiClassRange {
        first: 0x5C0,
        last: 0x5C0,
        value: 1,
    },
    BidiClassRange {
        first: 0x5C1,
        last: 0x5C2,
        value: 17,
    },
    BidiClassRange {
        first: 0x5C3,
        last: 0x5C3,
        value: 1,
    },
    BidiClassRange {
        first: 0x5C4,
        last: 0x5C5,
        value: 17,
    },
    BidiClassRange {
        first: 0x5C6,
        last: 0x5C6,
        value: 1,
    },
    BidiClassRange {
        first: 0x5C7,
        last: 0x5C7,
        value: 17,
    },
    BidiClassRange {
        first: 0x5C8,
        last: 0x5FF,
        value: 1,
    },
    BidiClassRange {
        first: 0x600,
        last: 0x605,
        value: 5,
    },
    BidiClassRange {
        first: 0x606,
        last: 0x607,
        value: 10,
    },
    BidiClassRange {
        first: 0x608,
        last: 0x608,
        value: 13,
    },
    BidiClassRange {
        first: 0x609,
        last: 0x60A,
        value: 4,
    },
    BidiClassRange {
        first: 0x60B,
        last: 0x60B,
        value: 13,
    },
    BidiClassRange {
        first: 0x60C,
        last: 0x60C,
        value: 6,
    },
    BidiClassRange {
        first: 0x60D,
        last: 0x60D,
        value: 13,
    },
    BidiClassRange {
        first: 0x60E,
        last: 0x60F,
        value: 10,
    },
    BidiClassRange {
        first: 0x610,
        last: 0x61A,
        value: 17,
    },
    BidiClassRange {
        first: 0x61B,
        last: 0x64A,
        value: 13,
    },
    BidiClassRange {
        first: 0x64B,
        last: 0x65F,
        value: 17,
    },
    BidiClassRange {
        first: 0x660,
        last: 0x669,
        value: 5,
    },
    BidiClassRange {
        first: 0x66A,
        last: 0x66A,
        value: 4,
    },
    BidiClassRange {
        first: 0x66B,
        last: 0x66C,
        value: 5,
    },
    BidiClassRange {
        first: 0x66D,
        last: 0x66F,
        value: 13,
    },
    BidiClassRange {
        first: 0x670,
        last: 0x670,
        value: 17,
    },
    BidiClassRange {
        first: 0x671,
        last: 0x6D5,
        value: 13,
    },
    BidiClassRange {
        first: 0x6D6,
        last: 0x6DC,
        value: 17,
    },
    BidiClassRange {
        first: 0x6DD,
        last: 0x6DD,
        value: 5,
    },
    BidiClassRange {
        first: 0x6DE,
        last: 0x6DE,
        value: 10,
    },
    BidiClassRange {
        first: 0x6DF,
        last: 0x6E4,
        value: 17,
    },
    BidiClassRange {
        first: 0x6E5,
        last: 0x6E6,
        value: 13,
    },
    BidiClassRange {
        first: 0x6E7,
        last: 0x6E8,
        value: 17,
    },
    BidiClassRange {
        first: 0x6E9,
        last: 0x6E9,
        value: 10,
    },
    BidiClassRange {
        first: 0x6EA,
        last: 0x6ED,
        value: 17,
    },
    BidiClassRange {
        first: 0x6EE,
        last: 0x6EF,
        value: 13,
    },
    BidiClassRange {
        first: 0x6F0,
        last: 0x6F9,
        value: 2,
    },
    BidiClassRange {
        first: 0x6FA,
        last: 0x710,
        value: 13,
    },
    BidiClassRange {
        first: 0x711,
        last: 0x711,
        value: 17,
    },
    BidiClassRange {
        first: 0x712,
        last: 0x72F,
        value: 13,
    },
    BidiClassRange {
        first: 0x730,
        last: 0x74A,
        value: 17,
    },
    BidiClassRange {
        first: 0x74B,
        last: 0x7A5,
        value: 13,
    },
    BidiClassRange {
        first: 0x7A6,
        last: 0x7B0,
        value: 17,
    },
    BidiClassRange {
        first: 0x7B1,
        last: 0x7BF,
        value: 13,
    },
    BidiClassRange {
        first: 0x7C0,
        last: 0x7EA,
        value: 1,
    },
    BidiClassRange {
        first: 0x7EB,
        last: 0x7F3,
        value: 17,
    },
    BidiClassRange {
        first: 0x7F4,
        last: 0x7F5,
        value: 1,
    },
    BidiClassRange {
        first: 0x7F6,
        last: 0x7F9,
        value: 10,
    },
    BidiClassRange {
        first: 0x7FA,
        last: 0x7FC,
        value: 1,
    },
    BidiClassRange {
        first: 0x7FD,
        last: 0x7FD,
        value: 17,
    },
    BidiClassRange {
        first: 0x7FE,
        last: 0x815,
        value: 1,
    },
    BidiClassRange {
        first: 0x816,
        last: 0x819,
        value: 17,
    },
    BidiClassRange {
        first: 0x81A,
        last: 0x81A,
        value: 1,
    },
    BidiClassRange {
        first: 0x81B,
        last: 0x823,
        value: 17,
    },
    BidiClassRange {
        first: 0x824,
        last: 0x824,
        value: 1,
    },
    BidiClassRange {
        first: 0x825,
        last: 0x827,
        value: 17,
    },
    BidiClassRange {
        first: 0x828,
        last: 0x828,
        value: 1,
    },
    BidiClassRange {
        first: 0x829,
        last: 0x82D,
        value: 17,
    },
    BidiClassRange {
        first: 0x82E,
        last: 0x858,
        value: 1,
    },
    BidiClassRange {
        first: 0x859,
        last: 0x85B,
        value: 17,
    },
    BidiClassRange {
        first: 0x85C,
        last: 0x85F,
        value: 1,
    },
    BidiClassRange {
        first: 0x860,
        last: 0x88F,
        value: 13,
    },
    BidiClassRange {
        first: 0x890,
        last: 0x891,
        value: 5,
    },
    BidiClassRange {
        first: 0x892,
        last: 0x896,
        value: 13,
    },
    BidiClassRange {
        first: 0x897,
        last: 0x89F,
        value: 17,
    },
    BidiClassRange {
        first: 0x8A0,
        last: 0x8C9,
        value: 13,
    },
    BidiClassRange {
        first: 0x8CA,
        last: 0x8E1,
        value: 17,
    },
    BidiClassRange {
        first: 0x8E2,
        last: 0x8E2,
        value: 5,
    },
    BidiClassRange {
        first: 0x8E3,
        last: 0x902,
        value: 17,
    },
    BidiClassRange {
        first: 0x93A,
        last: 0x93A,
        value: 17,
    },
    BidiClassRange {
        first: 0x93C,
        last: 0x93C,
        value: 17,
    },
    BidiClassRange {
        first: 0x941,
        last: 0x948,
        value: 17,
    },
    BidiClassRange {
        first: 0x94D,
        last: 0x94D,
        value: 17,
    },
    BidiClassRange {
        first: 0x951,
        last: 0x957,
        value: 17,
    },
    BidiClassRange {
        first: 0x962,
        last: 0x963,
        value: 17,
    },
    BidiClassRange {
        first: 0x981,
        last: 0x981,
        value: 17,
    },
    BidiClassRange {
        first: 0x9BC,
        last: 0x9BC,
        value: 17,
    },
    BidiClassRange {
        first: 0x9C1,
        last: 0x9C4,
        value: 17,
    },
    BidiClassRange {
        first: 0x9CD,
        last: 0x9CD,
        value: 17,
    },
    BidiClassRange {
        first: 0x9E2,
        last: 0x9E3,
        value: 17,
    },
    BidiClassRange {
        first: 0x9F2,
        last: 0x9F3,
        value: 4,
    },
    BidiClassRange {
        first: 0x9FB,
        last: 0x9FB,
        value: 4,
    },
    BidiClassRange {
        first: 0x9FE,
        last: 0x9FE,
        value: 17,
    },
    BidiClassRange {
        first: 0xA01,
        last: 0xA02,
        value: 17,
    },
    BidiClassRange {
        first: 0xA3C,
        last: 0xA3C,
        value: 17,
    },
    BidiClassRange {
        first: 0xA41,
        last: 0xA42,
        value: 17,
    },
    BidiClassRange {
        first: 0xA47,
        last: 0xA48,
        value: 17,
    },
    BidiClassRange {
        first: 0xA4B,
        last: 0xA4D,
        value: 17,
    },
    BidiClassRange {
        first: 0xA51,
        last: 0xA51,
        value: 17,
    },
    BidiClassRange {
        first: 0xA70,
        last: 0xA71,
        value: 17,
    },
    BidiClassRange {
        first: 0xA75,
        last: 0xA75,
        value: 17,
    },
    BidiClassRange {
        first: 0xA81,
        last: 0xA82,
        value: 17,
    },
    BidiClassRange {
        first: 0xABC,
        last: 0xABC,
        value: 17,
    },
    BidiClassRange {
        first: 0xAC1,
        last: 0xAC5,
        value: 17,
    },
    BidiClassRange {
        first: 0xAC7,
        last: 0xAC8,
        value: 17,
    },
    BidiClassRange {
        first: 0xACD,
        last: 0xACD,
        value: 17,
    },
    BidiClassRange {
        first: 0xAE2,
        last: 0xAE3,
        value: 17,
    },
    BidiClassRange {
        first: 0xAF1,
        last: 0xAF1,
        value: 4,
    },
    BidiClassRange {
        first: 0xAFA,
        last: 0xAFF,
        value: 17,
    },
    BidiClassRange {
        first: 0xB01,
        last: 0xB01,
        value: 17,
    },
    BidiClassRange {
        first: 0xB3C,
        last: 0xB3C,
        value: 17,
    },
    BidiClassRange {
        first: 0xB3F,
        last: 0xB3F,
        value: 17,
    },
    BidiClassRange {
        first: 0xB41,
        last: 0xB44,
        value: 17,
    },
    BidiClassRange {
        first: 0xB4D,
        last: 0xB4D,
        value: 17,
    },
    BidiClassRange {
        first: 0xB55,
        last: 0xB56,
        value: 17,
    },
    BidiClassRange {
        first: 0xB62,
        last: 0xB63,
        value: 17,
    },
    BidiClassRange {
        first: 0xB82,
        last: 0xB82,
        value: 17,
    },
    BidiClassRange {
        first: 0xBC0,
        last: 0xBC0,
        value: 17,
    },
    BidiClassRange {
        first: 0xBCD,
        last: 0xBCD,
        value: 17,
    },
    BidiClassRange {
        first: 0xBF3,
        last: 0xBF8,
        value: 10,
    },
    BidiClassRange {
        first: 0xBF9,
        last: 0xBF9,
        value: 4,
    },
    BidiClassRange {
        first: 0xBFA,
        last: 0xBFA,
        value: 10,
    },
    BidiClassRange {
        first: 0xC00,
        last: 0xC00,
        value: 17,
    },
    BidiClassRange {
        first: 0xC04,
        last: 0xC04,
        value: 17,
    },
    BidiClassRange {
        first: 0xC3C,
        last: 0xC3C,
        value: 17,
    },
    BidiClassRange {
        first: 0xC3E,
        last: 0xC40,
        value: 17,
    },
    BidiClassRange {
        first: 0xC46,
        last: 0xC48,
        value: 17,
    },
    BidiClassRange {
        first: 0xC4A,
        last: 0xC4D,
        value: 17,
    },
    BidiClassRange {
        first: 0xC55,
        last: 0xC56,
        value: 17,
    },
    BidiClassRange {
        first: 0xC62,
        last: 0xC63,
        value: 17,
    },
    BidiClassRange {
        first: 0xC78,
        last: 0xC7E,
        value: 10,
    },
    BidiClassRange {
        first: 0xC81,
        last: 0xC81,
        value: 17,
    },
    BidiClassRange {
        first: 0xCBC,
        last: 0xCBC,
        value: 17,
    },
    BidiClassRange {
        first: 0xCCC,
        last: 0xCCD,
        value: 17,
    },
    BidiClassRange {
        first: 0xCE2,
        last: 0xCE3,
        value: 17,
    },
    BidiClassRange {
        first: 0xD00,
        last: 0xD01,
        value: 17,
    },
    BidiClassRange {
        first: 0xD3B,
        last: 0xD3C,
        value: 17,
    },
    BidiClassRange {
        first: 0xD41,
        last: 0xD44,
        value: 17,
    },
    BidiClassRange {
        first: 0xD4D,
        last: 0xD4D,
        value: 17,
    },
    BidiClassRange {
        first: 0xD62,
        last: 0xD63,
        value: 17,
    },
    BidiClassRange {
        first: 0xD81,
        last: 0xD81,
        value: 17,
    },
    BidiClassRange {
        first: 0xDCA,
        last: 0xDCA,
        value: 17,
    },
    BidiClassRange {
        first: 0xDD2,
        last: 0xDD4,
        value: 17,
    },
    BidiClassRange {
        first: 0xDD6,
        last: 0xDD6,
        value: 17,
    },
    BidiClassRange {
        first: 0xE31,
        last: 0xE31,
        value: 17,
    },
    BidiClassRange {
        first: 0xE34,
        last: 0xE3A,
        value: 17,
    },
    BidiClassRange {
        first: 0xE3F,
        last: 0xE3F,
        value: 4,
    },
    BidiClassRange {
        first: 0xE47,
        last: 0xE4E,
        value: 17,
    },
    BidiClassRange {
        first: 0xEB1,
        last: 0xEB1,
        value: 17,
    },
    BidiClassRange {
        first: 0xEB4,
        last: 0xEBC,
        value: 17,
    },
    BidiClassRange {
        first: 0xEC8,
        last: 0xECE,
        value: 17,
    },
    BidiClassRange {
        first: 0xF18,
        last: 0xF19,
        value: 17,
    },
    BidiClassRange {
        first: 0xF35,
        last: 0xF35,
        value: 17,
    },
    BidiClassRange {
        first: 0xF37,
        last: 0xF37,
        value: 17,
    },
    BidiClassRange {
        first: 0xF39,
        last: 0xF39,
        value: 17,
    },
    BidiClassRange {
        first: 0xF3A,
        last: 0xF3D,
        value: 10,
    },
    BidiClassRange {
        first: 0xF71,
        last: 0xF7E,
        value: 17,
    },
    BidiClassRange {
        first: 0xF80,
        last: 0xF84,
        value: 17,
    },
    BidiClassRange {
        first: 0xF86,
        last: 0xF87,
        value: 17,
    },
    BidiClassRange {
        first: 0xF8D,
        last: 0xF97,
        value: 17,
    },
    BidiClassRange {
        first: 0xF99,
        last: 0xFBC,
        value: 17,
    },
    BidiClassRange {
        first: 0xFC6,
        last: 0xFC6,
        value: 17,
    },
    BidiClassRange {
        first: 0x102D,
        last: 0x1030,
        value: 17,
    },
    BidiClassRange {
        first: 0x1032,
        last: 0x1037,
        value: 17,
    },
    BidiClassRange {
        first: 0x1039,
        last: 0x103A,
        value: 17,
    },
    BidiClassRange {
        first: 0x103D,
        last: 0x103E,
        value: 17,
    },
    BidiClassRange {
        first: 0x1058,
        last: 0x1059,
        value: 17,
    },
    BidiClassRange {
        first: 0x105E,
        last: 0x1060,
        value: 17,
    },
    BidiClassRange {
        first: 0x1071,
        last: 0x1074,
        value: 17,
    },
    BidiClassRange {
        first: 0x1082,
        last: 0x1082,
        value: 17,
    },
    BidiClassRange {
        first: 0x1085,
        last: 0x1086,
        value: 17,
    },
    BidiClassRange {
        first: 0x108D,
        last: 0x108D,
        value: 17,
    },
    BidiClassRange {
        first: 0x109D,
        last: 0x109D,
        value: 17,
    },
    BidiClassRange {
        first: 0x135D,
        last: 0x135F,
        value: 17,
    },
    BidiClassRange {
        first: 0x1390,
        last: 0x1399,
        value: 10,
    },
    BidiClassRange {
        first: 0x1400,
        last: 0x1400,
        value: 10,
    },
    BidiClassRange {
        first: 0x1680,
        last: 0x1680,
        value: 9,
    },
    BidiClassRange {
        first: 0x169B,
        last: 0x169C,
        value: 10,
    },
    BidiClassRange {
        first: 0x1712,
        last: 0x1714,
        value: 17,
    },
    BidiClassRange {
        first: 0x1732,
        last: 0x1733,
        value: 17,
    },
    BidiClassRange {
        first: 0x1752,
        last: 0x1753,
        value: 17,
    },
    BidiClassRange {
        first: 0x1772,
        last: 0x1773,
        value: 17,
    },
    BidiClassRange {
        first: 0x17B4,
        last: 0x17B5,
        value: 17,
    },
    BidiClassRange {
        first: 0x17B7,
        last: 0x17BD,
        value: 17,
    },
    BidiClassRange {
        first: 0x17C6,
        last: 0x17C6,
        value: 17,
    },
    BidiClassRange {
        first: 0x17C9,
        last: 0x17D3,
        value: 17,
    },
    BidiClassRange {
        first: 0x17DB,
        last: 0x17DB,
        value: 4,
    },
    BidiClassRange {
        first: 0x17DD,
        last: 0x17DD,
        value: 17,
    },
    BidiClassRange {
        first: 0x17F0,
        last: 0x17F9,
        value: 10,
    },
    BidiClassRange {
        first: 0x1800,
        last: 0x180A,
        value: 10,
    },
    BidiClassRange {
        first: 0x180B,
        last: 0x180D,
        value: 17,
    },
    BidiClassRange {
        first: 0x180E,
        last: 0x180E,
        value: 18,
    },
    BidiClassRange {
        first: 0x180F,
        last: 0x180F,
        value: 17,
    },
    BidiClassRange {
        first: 0x1885,
        last: 0x1886,
        value: 17,
    },
    BidiClassRange {
        first: 0x18A9,
        last: 0x18A9,
        value: 17,
    },
    BidiClassRange {
        first: 0x1920,
        last: 0x1922,
        value: 17,
    },
    BidiClassRange {
        first: 0x1927,
        last: 0x1928,
        value: 17,
    },
    BidiClassRange {
        first: 0x1932,
        last: 0x1932,
        value: 17,
    },
    BidiClassRange {
        first: 0x1939,
        last: 0x193B,
        value: 17,
    },
    BidiClassRange {
        first: 0x1940,
        last: 0x1940,
        value: 10,
    },
    BidiClassRange {
        first: 0x1944,
        last: 0x1945,
        value: 10,
    },
    BidiClassRange {
        first: 0x19DE,
        last: 0x19FF,
        value: 10,
    },
    BidiClassRange {
        first: 0x1A17,
        last: 0x1A18,
        value: 17,
    },
    BidiClassRange {
        first: 0x1A1B,
        last: 0x1A1B,
        value: 17,
    },
    BidiClassRange {
        first: 0x1A56,
        last: 0x1A56,
        value: 17,
    },
    BidiClassRange {
        first: 0x1A58,
        last: 0x1A5E,
        value: 17,
    },
    BidiClassRange {
        first: 0x1A60,
        last: 0x1A60,
        value: 17,
    },
    BidiClassRange {
        first: 0x1A62,
        last: 0x1A62,
        value: 17,
    },
    BidiClassRange {
        first: 0x1A65,
        last: 0x1A6C,
        value: 17,
    },
    BidiClassRange {
        first: 0x1A73,
        last: 0x1A7C,
        value: 17,
    },
    BidiClassRange {
        first: 0x1A7F,
        last: 0x1A7F,
        value: 17,
    },
    BidiClassRange {
        first: 0x1AB0,
        last: 0x1B03,
        value: 17,
    },
    BidiClassRange {
        first: 0x1B34,
        last: 0x1B34,
        value: 17,
    },
    BidiClassRange {
        first: 0x1B36,
        last: 0x1B3A,
        value: 17,
    },
    BidiClassRange {
        first: 0x1B3C,
        last: 0x1B3C,
        value: 17,
    },
    BidiClassRange {
        first: 0x1B42,
        last: 0x1B42,
        value: 17,
    },
    BidiClassRange {
        first: 0x1B6B,
        last: 0x1B73,
        value: 17,
    },
    BidiClassRange {
        first: 0x1B80,
        last: 0x1B81,
        value: 17,
    },
    BidiClassRange {
        first: 0x1BA2,
        last: 0x1BA5,
        value: 17,
    },
    BidiClassRange {
        first: 0x1BA8,
        last: 0x1BA9,
        value: 17,
    },
    BidiClassRange {
        first: 0x1BAB,
        last: 0x1BAD,
        value: 17,
    },
    BidiClassRange {
        first: 0x1BE6,
        last: 0x1BE6,
        value: 17,
    },
    BidiClassRange {
        first: 0x1BE8,
        last: 0x1BE9,
        value: 17,
    },
    BidiClassRange {
        first: 0x1BED,
        last: 0x1BED,
        value: 17,
    },
    BidiClassRange {
        first: 0x1BEF,
        last: 0x1BF1,
        value: 17,
    },
    BidiClassRange {
        first: 0x1C2C,
        last: 0x1C33,
        value: 17,
    },
    BidiClassRange {
        first: 0x1C36,
        last: 0x1C37,
        value: 17,
    },
    BidiClassRange {
        first: 0x1CD0,
        last: 0x1CD2,
        value: 17,
    },
    BidiClassRange {
        first: 0x1CD4,
        last: 0x1CE0,
        value: 17,
    },
    BidiClassRange {
        first: 0x1CE2,
        last: 0x1CE8,
        value: 17,
    },
    BidiClassRange {
        first: 0x1CED,
        last: 0x1CED,
        value: 17,
    },
    BidiClassRange {
        first: 0x1CF4,
        last: 0x1CF4,
        value: 17,
    },
    BidiClassRange {
        first: 0x1CF8,
        last: 0x1CF9,
        value: 17,
    },
    BidiClassRange {
        first: 0x1CFB,
        last: 0x1CFF,
        value: 17,
    },
    BidiClassRange {
        first: 0x1DC0,
        last: 0x1DFF,
        value: 17,
    },
    BidiClassRange {
        first: 0x1FBD,
        last: 0x1FBD,
        value: 10,
    },
    BidiClassRange {
        first: 0x1FBF,
        last: 0x1FC1,
        value: 10,
    },
    BidiClassRange {
        first: 0x1FCD,
        last: 0x1FCF,
        value: 10,
    },
    BidiClassRange {
        first: 0x1FDD,
        last: 0x1FDF,
        value: 10,
    },
    BidiClassRange {
        first: 0x1FED,
        last: 0x1FEF,
        value: 10,
    },
    BidiClassRange {
        first: 0x1FFD,
        last: 0x1FFE,
        value: 10,
    },
    BidiClassRange {
        first: 0x2000,
        last: 0x200A,
        value: 9,
    },
    BidiClassRange {
        first: 0x200B,
        last: 0x200D,
        value: 18,
    },
    BidiClassRange {
        first: 0x200F,
        last: 0x200F,
        value: 1,
    },
    BidiClassRange {
        first: 0x2010,
        last: 0x2027,
        value: 10,
    },
    BidiClassRange {
        first: 0x2028,
        last: 0x2028,
        value: 9,
    },
    BidiClassRange {
        first: 0x2029,
        last: 0x2029,
        value: 7,
    },
    BidiClassRange {
        first: 0x202A,
        last: 0x202A,
        value: 11,
    },
    BidiClassRange {
        first: 0x202B,
        last: 0x202B,
        value: 14,
    },
    BidiClassRange {
        first: 0x202C,
        last: 0x202C,
        value: 16,
    },
    BidiClassRange {
        first: 0x202D,
        last: 0x202D,
        value: 12,
    },
    BidiClassRange {
        first: 0x202E,
        last: 0x202E,
        value: 15,
    },
    BidiClassRange {
        first: 0x202F,
        last: 0x202F,
        value: 6,
    },
    BidiClassRange {
        first: 0x2030,
        last: 0x2034,
        value: 4,
    },
    BidiClassRange {
        first: 0x2035,
        last: 0x2043,
        value: 10,
    },
    BidiClassRange {
        first: 0x2044,
        last: 0x2044,
        value: 6,
    },
    BidiClassRange {
        first: 0x2045,
        last: 0x205E,
        value: 10,
    },
    BidiClassRange {
        first: 0x205F,
        last: 0x205F,
        value: 9,
    },
    BidiClassRange {
        first: 0x2060,
        last: 0x2065,
        value: 18,
    },
    BidiClassRange {
        first: 0x2066,
        last: 0x2066,
        value: 20,
    },
    BidiClassRange {
        first: 0x2067,
        last: 0x2067,
        value: 21,
    },
    BidiClassRange {
        first: 0x2068,
        last: 0x2068,
        value: 19,
    },
    BidiClassRange {
        first: 0x2069,
        last: 0x2069,
        value: 22,
    },
    BidiClassRange {
        first: 0x206A,
        last: 0x206F,
        value: 18,
    },
    BidiClassRange {
        first: 0x2070,
        last: 0x2070,
        value: 2,
    },
    BidiClassRange {
        first: 0x2072,
        last: 0x2079,
        value: 2,
    },
    BidiClassRange {
        first: 0x207A,
        last: 0x207B,
        value: 3,
    },
    BidiClassRange {
        first: 0x207C,
        last: 0x207E,
        value: 10,
    },
    BidiClassRange {
        first: 0x2080,
        last: 0x2089,
        value: 2,
    },
    BidiClassRange {
        first: 0x208A,
        last: 0x208B,
        value: 3,
    },
    BidiClassRange {
        first: 0x208C,
        last: 0x208E,
        value: 10,
    },
    BidiClassRange {
        first: 0x208F,
        last: 0x208F,
        value: 2,
    },
    BidiClassRange {
        first: 0x209D,
        last: 0x209F,
        value: 2,
    },
    BidiClassRange {
        first: 0x20A0,
        last: 0x20CF,
        value: 4,
    },
    BidiClassRange {
        first: 0x20D0,
        last: 0x20FF,
        value: 17,
    },
    BidiClassRange {
        first: 0x2100,
        last: 0x2101,
        value: 10,
    },
    BidiClassRange {
        first: 0x2103,
        last: 0x2106,
        value: 10,
    },
    BidiClassRange {
        first: 0x2108,
        last: 0x2109,
        value: 10,
    },
    BidiClassRange {
        first: 0x2114,
        last: 0x2114,
        value: 10,
    },
    BidiClassRange {
        first: 0x2116,
        last: 0x2118,
        value: 10,
    },
    BidiClassRange {
        first: 0x211E,
        last: 0x2123,
        value: 10,
    },
    BidiClassRange {
        first: 0x2125,
        last: 0x2125,
        value: 10,
    },
    BidiClassRange {
        first: 0x2127,
        last: 0x2127,
        value: 10,
    },
    BidiClassRange {
        first: 0x2129,
        last: 0x2129,
        value: 10,
    },
    BidiClassRange {
        first: 0x212E,
        last: 0x212E,
        value: 4,
    },
    BidiClassRange {
        first: 0x213A,
        last: 0x213B,
        value: 10,
    },
    BidiClassRange {
        first: 0x2140,
        last: 0x2144,
        value: 10,
    },
    BidiClassRange {
        first: 0x214A,
        last: 0x214D,
        value: 10,
    },
    BidiClassRange {
        first: 0x2150,
        last: 0x215F,
        value: 10,
    },
    BidiClassRange {
        first: 0x2189,
        last: 0x218B,
        value: 10,
    },
    BidiClassRange {
        first: 0x2190,
        last: 0x2211,
        value: 10,
    },
    BidiClassRange {
        first: 0x2212,
        last: 0x2212,
        value: 3,
    },
    BidiClassRange {
        first: 0x2213,
        last: 0x2213,
        value: 4,
    },
    BidiClassRange {
        first: 0x2214,
        last: 0x2335,
        value: 10,
    },
    BidiClassRange {
        first: 0x237B,
        last: 0x2394,
        value: 10,
    },
    BidiClassRange {
        first: 0x2396,
        last: 0x2487,
        value: 10,
    },
    BidiClassRange {
        first: 0x2488,
        last: 0x249B,
        value: 2,
    },
    BidiClassRange {
        first: 0x24EA,
        last: 0x26AB,
        value: 10,
    },
    BidiClassRange {
        first: 0x26AD,
        last: 0x27FF,
        value: 10,
    },
    BidiClassRange {
        first: 0x2900,
        last: 0x2BFF,
        value: 10,
    },
    BidiClassRange {
        first: 0x2CE5,
        last: 0x2CEA,
        value: 10,
    },
    BidiClassRange {
        first: 0x2CEF,
        last: 0x2CF1,
        value: 17,
    },
    BidiClassRange {
        first: 0x2CF9,
        last: 0x2CFF,
        value: 10,
    },
    BidiClassRange {
        first: 0x2D7F,
        last: 0x2D7F,
        value: 17,
    },
    BidiClassRange {
        first: 0x2DE0,
        last: 0x2DFF,
        value: 17,
    },
    BidiClassRange {
        first: 0x2E00,
        last: 0x2FDF,
        value: 10,
    },
    BidiClassRange {
        first: 0x2FF0,
        last: 0x2FFF,
        value: 10,
    },
    BidiClassRange {
        first: 0x3000,
        last: 0x3000,
        value: 9,
    },
    BidiClassRange {
        first: 0x3001,
        last: 0x3004,
        value: 10,
    },
    BidiClassRange {
        first: 0x3008,
        last: 0x3020,
        value: 10,
    },
    BidiClassRange {
        first: 0x302A,
        last: 0x302D,
        value: 17,
    },
    BidiClassRange {
        first: 0x3030,
        last: 0x3030,
        value: 10,
    },
    BidiClassRange {
        first: 0x3036,
        last: 0x3037,
        value: 10,
    },
    BidiClassRange {
        first: 0x303D,
        last: 0x303F,
        value: 10,
    },
    BidiClassRange {
        first: 0x3099,
        last: 0x309A,
        value: 17,
    },
    BidiClassRange {
        first: 0x309B,
        last: 0x309C,
        value: 10,
    },
    BidiClassRange {
        first: 0x30A0,
        last: 0x30A0,
        value: 10,
    },
    BidiClassRange {
        first: 0x30FB,
        last: 0x30FB,
        value: 10,
    },
    BidiClassRange {
        first: 0x31C0,
        last: 0x31EF,
        value: 10,
    },
    BidiClassRange {
        first: 0x321D,
        last: 0x321E,
        value: 10,
    },
    BidiClassRange {
        first: 0x3250,
        last: 0x325F,
        value: 10,
    },
    BidiClassRange {
        first: 0x327C,
        last: 0x327E,
        value: 10,
    },
    BidiClassRange {
        first: 0x32B1,
        last: 0x32BF,
        value: 10,
    },
    BidiClassRange {
        first: 0x32CC,
        last: 0x32CF,
        value: 10,
    },
    BidiClassRange {
        first: 0x3377,
        last: 0x337A,
        value: 10,
    },
    BidiClassRange {
        first: 0x33DE,
        last: 0x33DF,
        value: 10,
    },
    BidiClassRange {
        first: 0x33FF,
        last: 0x33FF,
        value: 10,
    },
    BidiClassRange {
        first: 0x4DC0,
        last: 0x4DFF,
        value: 10,
    },
    BidiClassRange {
        first: 0xA490,
        last: 0xA4CF,
        value: 10,
    },
    BidiClassRange {
        first: 0xA60D,
        last: 0xA60F,
        value: 10,
    },
    BidiClassRange {
        first: 0xA66F,
        last: 0xA672,
        value: 17,
    },
    BidiClassRange {
        first: 0xA673,
        last: 0xA673,
        value: 10,
    },
    BidiClassRange {
        first: 0xA674,
        last: 0xA67D,
        value: 17,
    },
    BidiClassRange {
        first: 0xA67E,
        last: 0xA67F,
        value: 10,
    },
    BidiClassRange {
        first: 0xA69E,
        last: 0xA69F,
        value: 17,
    },
    BidiClassRange {
        first: 0xA6F0,
        last: 0xA6F1,
        value: 17,
    },
    BidiClassRange {
        first: 0xA700,
        last: 0xA721,
        value: 10,
    },
    BidiClassRange {
        first: 0xA788,
        last: 0xA788,
        value: 10,
    },
    BidiClassRange {
        first: 0xA802,
        last: 0xA802,
        value: 17,
    },
    BidiClassRange {
        first: 0xA806,
        last: 0xA806,
        value: 17,
    },
    BidiClassRange {
        first: 0xA80B,
        last: 0xA80B,
        value: 17,
    },
    BidiClassRange {
        first: 0xA825,
        last: 0xA826,
        value: 17,
    },
    BidiClassRange {
        first: 0xA828,
        last: 0xA82B,
        value: 10,
    },
    BidiClassRange {
        first: 0xA82C,
        last: 0xA82C,
        value: 17,
    },
    BidiClassRange {
        first: 0xA838,
        last: 0xA839,
        value: 4,
    },
    BidiClassRange {
        first: 0xA874,
        last: 0xA877,
        value: 10,
    },
    BidiClassRange {
        first: 0xA8C4,
        last: 0xA8C5,
        value: 17,
    },
    BidiClassRange {
        first: 0xA8E0,
        last: 0xA8F1,
        value: 17,
    },
    BidiClassRange {
        first: 0xA8FF,
        last: 0xA8FF,
        value: 17,
    },
    BidiClassRange {
        first: 0xA926,
        last: 0xA92D,
        value: 17,
    },
    BidiClassRange {
        first: 0xA947,
        last: 0xA951,
        value: 17,
    },
    BidiClassRange {
        first: 0xA980,
        last: 0xA982,
        value: 17,
    },
    BidiClassRange {
        first: 0xA9B3,
        last: 0xA9B3,
        value: 17,
    },
    BidiClassRange {
        first: 0xA9B6,
        last: 0xA9B9,
        value: 17,
    },
    BidiClassRange {
        first: 0xA9BC,
        last: 0xA9BD,
        value: 17,
    },
    BidiClassRange {
        first: 0xA9E5,
        last: 0xA9E5,
        value: 17,
    },
    BidiClassRange {
        first: 0xAA29,
        last: 0xAA2E,
        value: 17,
    },
    BidiClassRange {
        first: 0xAA31,
        last: 0xAA32,
        value: 17,
    },
    BidiClassRange {
        first: 0xAA35,
        last: 0xAA36,
        value: 17,
    },
    BidiClassRange {
        first: 0xAA43,
        last: 0xAA43,
        value: 17,
    },
    BidiClassRange {
        first: 0xAA4C,
        last: 0xAA4C,
        value: 17,
    },
    BidiClassRange {
        first: 0xAA7C,
        last: 0xAA7C,
        value: 17,
    },
    BidiClassRange {
        first: 0xAAB0,
        last: 0xAAB0,
        value: 17,
    },
    BidiClassRange {
        first: 0xAAB2,
        last: 0xAAB4,
        value: 17,
    },
    BidiClassRange {
        first: 0xAAB7,
        last: 0xAAB8,
        value: 17,
    },
    BidiClassRange {
        first: 0xAABE,
        last: 0xAABF,
        value: 17,
    },
    BidiClassRange {
        first: 0xAAC1,
        last: 0xAAC1,
        value: 17,
    },
    BidiClassRange {
        first: 0xAAEC,
        last: 0xAAED,
        value: 17,
    },
    BidiClassRange {
        first: 0xAAF6,
        last: 0xAAF6,
        value: 17,
    },
    BidiClassRange {
        first: 0xAB6A,
        last: 0xAB6B,
        value: 10,
    },
    BidiClassRange {
        first: 0xABE5,
        last: 0xABE5,
        value: 17,
    },
    BidiClassRange {
        first: 0xABE8,
        last: 0xABE8,
        value: 17,
    },
    BidiClassRange {
        first: 0xABED,
        last: 0xABED,
        value: 17,
    },
    BidiClassRange {
        first: 0xFB07,
        last: 0xFB12,
        value: 1,
    },
    BidiClassRange {
        first: 0xFB18,
        last: 0xFB1D,
        value: 1,
    },
    BidiClassRange {
        first: 0xFB1E,
        last: 0xFB1E,
        value: 17,
    },
    BidiClassRange {
        first: 0xFB1F,
        last: 0xFB28,
        value: 1,
    },
    BidiClassRange {
        first: 0xFB29,
        last: 0xFB29,
        value: 3,
    },
    BidiClassRange {
        first: 0xFB2A,
        last: 0xFB4F,
        value: 1,
    },
    BidiClassRange {
        first: 0xFB50,
        last: 0xFBC2,
        value: 13,
    },
    BidiClassRange {
        first: 0xFBC3,
        last: 0xFBD2,
        value: 10,
    },
    BidiClassRange {
        first: 0xFBD3,
        last: 0xFD3D,
        value: 13,
    },
    BidiClassRange {
        first: 0xFD3E,
        last: 0xFD4F,
        value: 10,
    },
    BidiClassRange {
        first: 0xFD50,
        last: 0xFD8F,
        value: 13,
    },
    BidiClassRange {
        first: 0xFD90,
        last: 0xFD91,
        value: 10,
    },
    BidiClassRange {
        first: 0xFD92,
        last: 0xFDC7,
        value: 13,
    },
    BidiClassRange {
        first: 0xFDC8,
        last: 0xFDCF,
        value: 10,
    },
    BidiClassRange {
        first: 0xFDD0,
        last: 0xFDEF,
        value: 18,
    },
    BidiClassRange {
        first: 0xFDF0,
        last: 0xFDFC,
        value: 13,
    },
    BidiClassRange {
        first: 0xFDFD,
        last: 0xFDFF,
        value: 10,
    },
    BidiClassRange {
        first: 0xFE00,
        last: 0xFE0F,
        value: 17,
    },
    BidiClassRange {
        first: 0xFE10,
        last: 0xFE1F,
        value: 10,
    },
    BidiClassRange {
        first: 0xFE20,
        last: 0xFE2F,
        value: 17,
    },
    BidiClassRange {
        first: 0xFE30,
        last: 0xFE4F,
        value: 10,
    },
    BidiClassRange {
        first: 0xFE50,
        last: 0xFE50,
        value: 6,
    },
    BidiClassRange {
        first: 0xFE51,
        last: 0xFE51,
        value: 10,
    },
    BidiClassRange {
        first: 0xFE52,
        last: 0xFE52,
        value: 6,
    },
    BidiClassRange {
        first: 0xFE53,
        last: 0xFE54,
        value: 10,
    },
    BidiClassRange {
        first: 0xFE55,
        last: 0xFE55,
        value: 6,
    },
    BidiClassRange {
        first: 0xFE56,
        last: 0xFE5E,
        value: 10,
    },
    BidiClassRange {
        first: 0xFE5F,
        last: 0xFE5F,
        value: 4,
    },
    BidiClassRange {
        first: 0xFE60,
        last: 0xFE61,
        value: 10,
    },
    BidiClassRange {
        first: 0xFE62,
        last: 0xFE63,
        value: 3,
    },
    BidiClassRange {
        first: 0xFE64,
        last: 0xFE68,
        value: 10,
    },
    BidiClassRange {
        first: 0xFE69,
        last: 0xFE6A,
        value: 4,
    },
    BidiClassRange {
        first: 0xFE6B,
        last: 0xFE6F,
        value: 10,
    },
    BidiClassRange {
        first: 0xFE70,
        last: 0xFEFE,
        value: 13,
    },
    BidiClassRange {
        first: 0xFEFF,
        last: 0xFEFF,
        value: 18,
    },
    BidiClassRange {
        first: 0xFF01,
        last: 0xFF02,
        value: 10,
    },
    BidiClassRange {
        first: 0xFF03,
        last: 0xFF05,
        value: 4,
    },
    BidiClassRange {
        first: 0xFF06,
        last: 0xFF0A,
        value: 10,
    },
    BidiClassRange {
        first: 0xFF0B,
        last: 0xFF0B,
        value: 3,
    },
    BidiClassRange {
        first: 0xFF0C,
        last: 0xFF0C,
        value: 6,
    },
    BidiClassRange {
        first: 0xFF0D,
        last: 0xFF0D,
        value: 3,
    },
    BidiClassRange {
        first: 0xFF0E,
        last: 0xFF0F,
        value: 6,
    },
    BidiClassRange {
        first: 0xFF10,
        last: 0xFF19,
        value: 2,
    },
    BidiClassRange {
        first: 0xFF1A,
        last: 0xFF1A,
        value: 6,
    },
    BidiClassRange {
        first: 0xFF1B,
        last: 0xFF20,
        value: 10,
    },
    BidiClassRange {
        first: 0xFF3B,
        last: 0xFF40,
        value: 10,
    },
    BidiClassRange {
        first: 0xFF5B,
        last: 0xFF65,
        value: 10,
    },
    BidiClassRange {
        first: 0xFFE0,
        last: 0xFFE1,
        value: 4,
    },
    BidiClassRange {
        first: 0xFFE2,
        last: 0xFFE4,
        value: 10,
    },
    BidiClassRange {
        first: 0xFFE5,
        last: 0xFFE6,
        value: 4,
    },
    BidiClassRange {
        first: 0xFFE8,
        last: 0xFFEE,
        value: 10,
    },
    BidiClassRange {
        first: 0xFFF0,
        last: 0xFFF8,
        value: 18,
    },
    BidiClassRange {
        first: 0xFFF9,
        last: 0xFFFD,
        value: 10,
    },
    BidiClassRange {
        first: 0xFFFE,
        last: 0xFFFF,
        value: 18,
    },
    BidiClassRange {
        first: 0x10101,
        last: 0x10101,
        value: 10,
    },
    BidiClassRange {
        first: 0x10140,
        last: 0x1018C,
        value: 10,
    },
    BidiClassRange {
        first: 0x1018F,
        last: 0x101CF,
        value: 10,
    },
    BidiClassRange {
        first: 0x101FD,
        last: 0x101FD,
        value: 17,
    },
    BidiClassRange {
        first: 0x102E0,
        last: 0x102E0,
        value: 17,
    },
    BidiClassRange {
        first: 0x102E1,
        last: 0x102FF,
        value: 2,
    },
    BidiClassRange {
        first: 0x10376,
        last: 0x1037A,
        value: 17,
    },
    BidiClassRange {
        first: 0x10800,
        last: 0x1091E,
        value: 1,
    },
    BidiClassRange {
        first: 0x1091F,
        last: 0x1091F,
        value: 10,
    },
    BidiClassRange {
        first: 0x10920,
        last: 0x10A00,
        value: 1,
    },
    BidiClassRange {
        first: 0x10A01,
        last: 0x10A03,
        value: 17,
    },
    BidiClassRange {
        first: 0x10A04,
        last: 0x10A04,
        value: 1,
    },
    BidiClassRange {
        first: 0x10A05,
        last: 0x10A06,
        value: 17,
    },
    BidiClassRange {
        first: 0x10A07,
        last: 0x10A0B,
        value: 1,
    },
    BidiClassRange {
        first: 0x10A0C,
        last: 0x10A0F,
        value: 17,
    },
    BidiClassRange {
        first: 0x10A10,
        last: 0x10A37,
        value: 1,
    },
    BidiClassRange {
        first: 0x10A38,
        last: 0x10A3A,
        value: 17,
    },
    BidiClassRange {
        first: 0x10A3B,
        last: 0x10A3E,
        value: 1,
    },
    BidiClassRange {
        first: 0x10A3F,
        last: 0x10A3F,
        value: 17,
    },
    BidiClassRange {
        first: 0x10A40,
        last: 0x10AE4,
        value: 1,
    },
    BidiClassRange {
        first: 0x10AE5,
        last: 0x10AE6,
        value: 17,
    },
    BidiClassRange {
        first: 0x10AE7,
        last: 0x10B38,
        value: 1,
    },
    BidiClassRange {
        first: 0x10B39,
        last: 0x10B3F,
        value: 10,
    },
    BidiClassRange {
        first: 0x10B40,
        last: 0x10CFF,
        value: 1,
    },
    BidiClassRange {
        first: 0x10D00,
        last: 0x10D23,
        value: 13,
    },
    BidiClassRange {
        first: 0x10D24,
        last: 0x10D27,
        value: 17,
    },
    BidiClassRange {
        first: 0x10D28,
        last: 0x10D2F,
        value: 13,
    },
    BidiClassRange {
        first: 0x10D30,
        last: 0x10D39,
        value: 5,
    },
    BidiClassRange {
        first: 0x10D3A,
        last: 0x10D3F,
        value: 13,
    },
    BidiClassRange {
        first: 0x10D40,
        last: 0x10D49,
        value: 5,
    },
    BidiClassRange {
        first: 0x10D4A,
        last: 0x10D68,
        value: 1,
    },
    BidiClassRange {
        first: 0x10D69,
        last: 0x10D6D,
        value: 17,
    },
    BidiClassRange {
        first: 0x10D6E,
        last: 0x10D6E,
        value: 10,
    },
    BidiClassRange {
        first: 0x10D6F,
        last: 0x10E5F,
        value: 1,
    },
    BidiClassRange {
        first: 0x10E60,
        last: 0x10E7E,
        value: 5,
    },
    BidiClassRange {
        first: 0x10E7F,
        last: 0x10EAA,
        value: 1,
    },
    BidiClassRange {
        first: 0x10EAB,
        last: 0x10EAC,
        value: 17,
    },
    BidiClassRange {
        first: 0x10EAD,
        last: 0x10EBF,
        value: 1,
    },
    BidiClassRange {
        first: 0x10EC0,
        last: 0x10ECF,
        value: 13,
    },
    BidiClassRange {
        first: 0x10ED0,
        last: 0x10ED8,
        value: 10,
    },
    BidiClassRange {
        first: 0x10ED9,
        last: 0x10EF9,
        value: 13,
    },
    BidiClassRange {
        first: 0x10EFA,
        last: 0x10EFF,
        value: 17,
    },
    BidiClassRange {
        first: 0x10F00,
        last: 0x10F2F,
        value: 1,
    },
    BidiClassRange {
        first: 0x10F30,
        last: 0x10F45,
        value: 13,
    },
    BidiClassRange {
        first: 0x10F46,
        last: 0x10F50,
        value: 17,
    },
    BidiClassRange {
        first: 0x10F51,
        last: 0x10F6F,
        value: 13,
    },
    BidiClassRange {
        first: 0x10F70,
        last: 0x10F81,
        value: 1,
    },
    BidiClassRange {
        first: 0x10F82,
        last: 0x10F85,
        value: 17,
    },
    BidiClassRange {
        first: 0x10F86,
        last: 0x10FFF,
        value: 1,
    },
    BidiClassRange {
        first: 0x11001,
        last: 0x11001,
        value: 17,
    },
    BidiClassRange {
        first: 0x11038,
        last: 0x11046,
        value: 17,
    },
    BidiClassRange {
        first: 0x11052,
        last: 0x11065,
        value: 10,
    },
    BidiClassRange {
        first: 0x11070,
        last: 0x11070,
        value: 17,
    },
    BidiClassRange {
        first: 0x11073,
        last: 0x11074,
        value: 17,
    },
    BidiClassRange {
        first: 0x1107F,
        last: 0x11081,
        value: 17,
    },
    BidiClassRange {
        first: 0x110B3,
        last: 0x110B6,
        value: 17,
    },
    BidiClassRange {
        first: 0x110B9,
        last: 0x110BA,
        value: 17,
    },
    BidiClassRange {
        first: 0x110C2,
        last: 0x110C2,
        value: 17,
    },
    BidiClassRange {
        first: 0x11100,
        last: 0x11102,
        value: 17,
    },
    BidiClassRange {
        first: 0x11127,
        last: 0x1112B,
        value: 17,
    },
    BidiClassRange {
        first: 0x1112D,
        last: 0x11134,
        value: 17,
    },
    BidiClassRange {
        first: 0x11173,
        last: 0x11173,
        value: 17,
    },
    BidiClassRange {
        first: 0x11180,
        last: 0x11181,
        value: 17,
    },
    BidiClassRange {
        first: 0x111B6,
        last: 0x111BE,
        value: 17,
    },
    BidiClassRange {
        first: 0x111C9,
        last: 0x111CC,
        value: 17,
    },
    BidiClassRange {
        first: 0x111CF,
        last: 0x111CF,
        value: 17,
    },
    BidiClassRange {
        first: 0x1122F,
        last: 0x11231,
        value: 17,
    },
    BidiClassRange {
        first: 0x11234,
        last: 0x11234,
        value: 17,
    },
    BidiClassRange {
        first: 0x11236,
        last: 0x11237,
        value: 17,
    },
    BidiClassRange {
        first: 0x1123E,
        last: 0x1123E,
        value: 17,
    },
    BidiClassRange {
        first: 0x11241,
        last: 0x11241,
        value: 17,
    },
    BidiClassRange {
        first: 0x112DF,
        last: 0x112DF,
        value: 17,
    },
    BidiClassRange {
        first: 0x112E3,
        last: 0x112EA,
        value: 17,
    },
    BidiClassRange {
        first: 0x11300,
        last: 0x11301,
        value: 17,
    },
    BidiClassRange {
        first: 0x1133B,
        last: 0x1133C,
        value: 17,
    },
    BidiClassRange {
        first: 0x11340,
        last: 0x11340,
        value: 17,
    },
    BidiClassRange {
        first: 0x11366,
        last: 0x1136C,
        value: 17,
    },
    BidiClassRange {
        first: 0x11370,
        last: 0x11374,
        value: 17,
    },
    BidiClassRange {
        first: 0x113BB,
        last: 0x113C0,
        value: 17,
    },
    BidiClassRange {
        first: 0x113CE,
        last: 0x113CE,
        value: 17,
    },
    BidiClassRange {
        first: 0x113D0,
        last: 0x113D0,
        value: 17,
    },
    BidiClassRange {
        first: 0x113D2,
        last: 0x113D2,
        value: 17,
    },
    BidiClassRange {
        first: 0x113E1,
        last: 0x113E2,
        value: 17,
    },
    BidiClassRange {
        first: 0x11438,
        last: 0x1143F,
        value: 17,
    },
    BidiClassRange {
        first: 0x11442,
        last: 0x11444,
        value: 17,
    },
    BidiClassRange {
        first: 0x11446,
        last: 0x11446,
        value: 17,
    },
    BidiClassRange {
        first: 0x1145E,
        last: 0x1145E,
        value: 17,
    },
    BidiClassRange {
        first: 0x114B3,
        last: 0x114B8,
        value: 17,
    },
    BidiClassRange {
        first: 0x114BA,
        last: 0x114BA,
        value: 17,
    },
    BidiClassRange {
        first: 0x114BF,
        last: 0x114C0,
        value: 17,
    },
    BidiClassRange {
        first: 0x114C2,
        last: 0x114C3,
        value: 17,
    },
    BidiClassRange {
        first: 0x115B2,
        last: 0x115B5,
        value: 17,
    },
    BidiClassRange {
        first: 0x115BC,
        last: 0x115BD,
        value: 17,
    },
    BidiClassRange {
        first: 0x115BF,
        last: 0x115C0,
        value: 17,
    },
    BidiClassRange {
        first: 0x115DC,
        last: 0x115DD,
        value: 17,
    },
    BidiClassRange {
        first: 0x11633,
        last: 0x1163A,
        value: 17,
    },
    BidiClassRange {
        first: 0x1163D,
        last: 0x1163D,
        value: 17,
    },
    BidiClassRange {
        first: 0x1163F,
        last: 0x11640,
        value: 17,
    },
    BidiClassRange {
        first: 0x11660,
        last: 0x1167F,
        value: 10,
    },
    BidiClassRange {
        first: 0x116AB,
        last: 0x116AB,
        value: 17,
    },
    BidiClassRange {
        first: 0x116AD,
        last: 0x116AD,
        value: 17,
    },
    BidiClassRange {
        first: 0x116B0,
        last: 0x116B5,
        value: 17,
    },
    BidiClassRange {
        first: 0x116B7,
        last: 0x116B7,
        value: 17,
    },
    BidiClassRange {
        first: 0x1171D,
        last: 0x1171D,
        value: 17,
    },
    BidiClassRange {
        first: 0x1171F,
        last: 0x1171F,
        value: 17,
    },
    BidiClassRange {
        first: 0x11722,
        last: 0x11725,
        value: 17,
    },
    BidiClassRange {
        first: 0x11727,
        last: 0x1172B,
        value: 17,
    },
    BidiClassRange {
        first: 0x1182F,
        last: 0x11837,
        value: 17,
    },
    BidiClassRange {
        first: 0x11839,
        last: 0x1183A,
        value: 17,
    },
    BidiClassRange {
        first: 0x1193B,
        last: 0x1193C,
        value: 17,
    },
    BidiClassRange {
        first: 0x1193E,
        last: 0x1193E,
        value: 17,
    },
    BidiClassRange {
        first: 0x11943,
        last: 0x11943,
        value: 17,
    },
    BidiClassRange {
        first: 0x119D4,
        last: 0x119D7,
        value: 17,
    },
    BidiClassRange {
        first: 0x119DA,
        last: 0x119DB,
        value: 17,
    },
    BidiClassRange {
        first: 0x119E0,
        last: 0x119E0,
        value: 17,
    },
    BidiClassRange {
        first: 0x11A01,
        last: 0x11A06,
        value: 17,
    },
    BidiClassRange {
        first: 0x11A09,
        last: 0x11A0A,
        value: 17,
    },
    BidiClassRange {
        first: 0x11A33,
        last: 0x11A38,
        value: 17,
    },
    BidiClassRange {
        first: 0x11A3B,
        last: 0x11A3E,
        value: 17,
    },
    BidiClassRange {
        first: 0x11A47,
        last: 0x11A47,
        value: 17,
    },
    BidiClassRange {
        first: 0x11A51,
        last: 0x11A56,
        value: 17,
    },
    BidiClassRange {
        first: 0x11A59,
        last: 0x11A5B,
        value: 17,
    },
    BidiClassRange {
        first: 0x11A8A,
        last: 0x11A96,
        value: 17,
    },
    BidiClassRange {
        first: 0x11A98,
        last: 0x11A99,
        value: 17,
    },
    BidiClassRange {
        first: 0x11B60,
        last: 0x11B60,
        value: 17,
    },
    BidiClassRange {
        first: 0x11B62,
        last: 0x11B64,
        value: 17,
    },
    BidiClassRange {
        first: 0x11B66,
        last: 0x11B66,
        value: 17,
    },
    BidiClassRange {
        first: 0x11B68,
        last: 0x11B7F,
        value: 17,
    },
    BidiClassRange {
        first: 0x11C30,
        last: 0x11C36,
        value: 17,
    },
    BidiClassRange {
        first: 0x11C38,
        last: 0x11C3D,
        value: 17,
    },
    BidiClassRange {
        first: 0x11C90,
        last: 0x11CA8,
        value: 17,
    },
    BidiClassRange {
        first: 0x11CAA,
        last: 0x11CB0,
        value: 17,
    },
    BidiClassRange {
        first: 0x11CB2,
        last: 0x11CB3,
        value: 17,
    },
    BidiClassRange {
        first: 0x11CB5,
        last: 0x11CBF,
        value: 17,
    },
    BidiClassRange {
        first: 0x11D31,
        last: 0x11D36,
        value: 17,
    },
    BidiClassRange {
        first: 0x11D3A,
        last: 0x11D3A,
        value: 17,
    },
    BidiClassRange {
        first: 0x11D3C,
        last: 0x11D3D,
        value: 17,
    },
    BidiClassRange {
        first: 0x11D3F,
        last: 0x11D45,
        value: 17,
    },
    BidiClassRange {
        first: 0x11D47,
        last: 0x11D47,
        value: 17,
    },
    BidiClassRange {
        first: 0x11D90,
        last: 0x11D91,
        value: 17,
    },
    BidiClassRange {
        first: 0x11D95,
        last: 0x11D95,
        value: 17,
    },
    BidiClassRange {
        first: 0x11D97,
        last: 0x11D97,
        value: 17,
    },
    BidiClassRange {
        first: 0x11EF3,
        last: 0x11EF4,
        value: 17,
    },
    BidiClassRange {
        first: 0x11F00,
        last: 0x11F01,
        value: 17,
    },
    BidiClassRange {
        first: 0x11F36,
        last: 0x11F3A,
        value: 17,
    },
    BidiClassRange {
        first: 0x11F40,
        last: 0x11F40,
        value: 17,
    },
    BidiClassRange {
        first: 0x11F42,
        last: 0x11F42,
        value: 17,
    },
    BidiClassRange {
        first: 0x11F5A,
        last: 0x11F5A,
        value: 17,
    },
    BidiClassRange {
        first: 0x11FD5,
        last: 0x11FDC,
        value: 10,
    },
    BidiClassRange {
        first: 0x11FDD,
        last: 0x11FE0,
        value: 4,
    },
    BidiClassRange {
        first: 0x11FE1,
        last: 0x11FFE,
        value: 10,
    },
    BidiClassRange {
        first: 0x13440,
        last: 0x13440,
        value: 17,
    },
    BidiClassRange {
        first: 0x13447,
        last: 0x13455,
        value: 17,
    },
    BidiClassRange {
        first: 0x1611E,
        last: 0x16129,
        value: 17,
    },
    BidiClassRange {
        first: 0x1612D,
        last: 0x1612F,
        value: 17,
    },
    BidiClassRange {
        first: 0x16AF0,
        last: 0x16AF4,
        value: 17,
    },
    BidiClassRange {
        first: 0x16B30,
        last: 0x16B36,
        value: 17,
    },
    BidiClassRange {
        first: 0x16F4F,
        last: 0x16F4F,
        value: 17,
    },
    BidiClassRange {
        first: 0x16F8F,
        last: 0x16F92,
        value: 17,
    },
    BidiClassRange {
        first: 0x16FE2,
        last: 0x16FE2,
        value: 10,
    },
    BidiClassRange {
        first: 0x16FE4,
        last: 0x16FE4,
        value: 17,
    },
    BidiClassRange {
        first: 0x1BC9D,
        last: 0x1BC9E,
        value: 17,
    },
    BidiClassRange {
        first: 0x1BCA0,
        last: 0x1BCAF,
        value: 18,
    },
    BidiClassRange {
        first: 0x1CC00,
        last: 0x1CCD5,
        value: 10,
    },
    BidiClassRange {
        first: 0x1CCF0,
        last: 0x1CCF9,
        value: 2,
    },
    BidiClassRange {
        first: 0x1CCFA,
        last: 0x1CEFF,
        value: 10,
    },
    BidiClassRange {
        first: 0x1CF00,
        last: 0x1CF2D,
        value: 17,
    },
    BidiClassRange {
        first: 0x1CF30,
        last: 0x1CF46,
        value: 17,
    },
    BidiClassRange {
        first: 0x1D167,
        last: 0x1D169,
        value: 17,
    },
    BidiClassRange {
        first: 0x1D173,
        last: 0x1D17A,
        value: 18,
    },
    BidiClassRange {
        first: 0x1D17B,
        last: 0x1D182,
        value: 17,
    },
    BidiClassRange {
        first: 0x1D185,
        last: 0x1D18B,
        value: 17,
    },
    BidiClassRange {
        first: 0x1D1AA,
        last: 0x1D1AD,
        value: 17,
    },
    BidiClassRange {
        first: 0x1D1E9,
        last: 0x1D1EA,
        value: 10,
    },
    BidiClassRange {
        first: 0x1D200,
        last: 0x1D241,
        value: 10,
    },
    BidiClassRange {
        first: 0x1D242,
        last: 0x1D244,
        value: 17,
    },
    BidiClassRange {
        first: 0x1D245,
        last: 0x1D24F,
        value: 10,
    },
    BidiClassRange {
        first: 0x1D300,
        last: 0x1D35F,
        value: 10,
    },
    BidiClassRange {
        first: 0x1D6C1,
        last: 0x1D6C1,
        value: 10,
    },
    BidiClassRange {
        first: 0x1D6DB,
        last: 0x1D6DB,
        value: 10,
    },
    BidiClassRange {
        first: 0x1D6FB,
        last: 0x1D6FB,
        value: 10,
    },
    BidiClassRange {
        first: 0x1D715,
        last: 0x1D715,
        value: 10,
    },
    BidiClassRange {
        first: 0x1D735,
        last: 0x1D735,
        value: 10,
    },
    BidiClassRange {
        first: 0x1D74F,
        last: 0x1D74F,
        value: 10,
    },
    BidiClassRange {
        first: 0x1D76F,
        last: 0x1D76F,
        value: 10,
    },
    BidiClassRange {
        first: 0x1D789,
        last: 0x1D789,
        value: 10,
    },
    BidiClassRange {
        first: 0x1D7A9,
        last: 0x1D7A9,
        value: 10,
    },
    BidiClassRange {
        first: 0x1D7C3,
        last: 0x1D7C3,
        value: 10,
    },
    BidiClassRange {
        first: 0x1D7CE,
        last: 0x1D7FF,
        value: 2,
    },
    BidiClassRange {
        first: 0x1DA00,
        last: 0x1DA36,
        value: 17,
    },
    BidiClassRange {
        first: 0x1DA3B,
        last: 0x1DA6C,
        value: 17,
    },
    BidiClassRange {
        first: 0x1DA75,
        last: 0x1DA75,
        value: 17,
    },
    BidiClassRange {
        first: 0x1DA84,
        last: 0x1DA84,
        value: 17,
    },
    BidiClassRange {
        first: 0x1DA9B,
        last: 0x1DA9F,
        value: 17,
    },
    BidiClassRange {
        first: 0x1DAA1,
        last: 0x1DAAF,
        value: 17,
    },
    BidiClassRange {
        first: 0x1E000,
        last: 0x1E02F,
        value: 17,
    },
    BidiClassRange {
        first: 0x1E08F,
        last: 0x1E08F,
        value: 17,
    },
    BidiClassRange {
        first: 0x1E130,
        last: 0x1E136,
        value: 17,
    },
    BidiClassRange {
        first: 0x1E2AE,
        last: 0x1E2AE,
        value: 17,
    },
    BidiClassRange {
        first: 0x1E2EC,
        last: 0x1E2EF,
        value: 17,
    },
    BidiClassRange {
        first: 0x1E2FF,
        last: 0x1E2FF,
        value: 4,
    },
    BidiClassRange {
        first: 0x1E4EC,
        last: 0x1E4EF,
        value: 17,
    },
    BidiClassRange {
        first: 0x1E5EE,
        last: 0x1E5EF,
        value: 17,
    },
    BidiClassRange {
        first: 0x1E6E3,
        last: 0x1E6E3,
        value: 17,
    },
    BidiClassRange {
        first: 0x1E6E6,
        last: 0x1E6E6,
        value: 17,
    },
    BidiClassRange {
        first: 0x1E6EE,
        last: 0x1E6EF,
        value: 17,
    },
    BidiClassRange {
        first: 0x1E6F5,
        last: 0x1E6F5,
        value: 17,
    },
    BidiClassRange {
        first: 0x1E800,
        last: 0x1E8CF,
        value: 1,
    },
    BidiClassRange {
        first: 0x1E8D0,
        last: 0x1E8D6,
        value: 17,
    },
    BidiClassRange {
        first: 0x1E8D7,
        last: 0x1E943,
        value: 1,
    },
    BidiClassRange {
        first: 0x1E944,
        last: 0x1E94A,
        value: 17,
    },
    BidiClassRange {
        first: 0x1E94B,
        last: 0x1EC6F,
        value: 1,
    },
    BidiClassRange {
        first: 0x1EC70,
        last: 0x1ECBF,
        value: 13,
    },
    BidiClassRange {
        first: 0x1ECC0,
        last: 0x1ECFF,
        value: 1,
    },
    BidiClassRange {
        first: 0x1ED00,
        last: 0x1ED4F,
        value: 13,
    },
    BidiClassRange {
        first: 0x1ED50,
        last: 0x1EDFF,
        value: 1,
    },
    BidiClassRange {
        first: 0x1EE00,
        last: 0x1EEEF,
        value: 13,
    },
    BidiClassRange {
        first: 0x1EEF0,
        last: 0x1EEF1,
        value: 10,
    },
    BidiClassRange {
        first: 0x1EEF2,
        last: 0x1EEFF,
        value: 13,
    },
    BidiClassRange {
        first: 0x1EF00,
        last: 0x1EFFF,
        value: 1,
    },
    BidiClassRange {
        first: 0x1F000,
        last: 0x1F0FF,
        value: 10,
    },
    BidiClassRange {
        first: 0x1F100,
        last: 0x1F10A,
        value: 2,
    },
    BidiClassRange {
        first: 0x1F10B,
        last: 0x1F10F,
        value: 10,
    },
    BidiClassRange {
        first: 0x1F12F,
        last: 0x1F12F,
        value: 10,
    },
    BidiClassRange {
        first: 0x1F16A,
        last: 0x1F16F,
        value: 10,
    },
    BidiClassRange {
        first: 0x1F1AD,
        last: 0x1F1AD,
        value: 10,
    },
    BidiClassRange {
        first: 0x1F260,
        last: 0x1F265,
        value: 10,
    },
    BidiClassRange {
        first: 0x1F300,
        last: 0x1FBEF,
        value: 10,
    },
    BidiClassRange {
        first: 0x1FBF0,
        last: 0x1FBF9,
        value: 2,
    },
    BidiClassRange {
        first: 0x1FBFA,
        last: 0x1FBFF,
        value: 10,
    },
    BidiClassRange {
        first: 0x1FFFE,
        last: 0x1FFFF,
        value: 18,
    },
    BidiClassRange {
        first: 0x2FFFE,
        last: 0x2FFFF,
        value: 18,
    },
    BidiClassRange {
        first: 0x3FFFE,
        last: 0x3FFFF,
        value: 18,
    },
    BidiClassRange {
        first: 0x4FFFE,
        last: 0x4FFFF,
        value: 18,
    },
    BidiClassRange {
        first: 0x5FFFE,
        last: 0x5FFFF,
        value: 18,
    },
    BidiClassRange {
        first: 0x6FFFE,
        last: 0x6FFFF,
        value: 18,
    },
    BidiClassRange {
        first: 0x7FFFE,
        last: 0x7FFFF,
        value: 18,
    },
    BidiClassRange {
        first: 0x8FFFE,
        last: 0x8FFFF,
        value: 18,
    },
    BidiClassRange {
        first: 0x9FFFE,
        last: 0x9FFFF,
        value: 18,
    },
    BidiClassRange {
        first: 0xAFFFE,
        last: 0xAFFFF,
        value: 18,
    },
    BidiClassRange {
        first: 0xBFFFE,
        last: 0xBFFFF,
        value: 18,
    },
    BidiClassRange {
        first: 0xCFFFE,
        last: 0xCFFFF,
        value: 18,
    },
    BidiClassRange {
        first: 0xDFFFE,
        last: 0xE00FF,
        value: 18,
    },
    BidiClassRange {
        first: 0xE0100,
        last: 0xE01EF,
        value: 17,
    },
    BidiClassRange {
        first: 0xE01F0,
        last: 0xE0FFF,
        value: 18,
    },
    BidiClassRange {
        first: 0xEFFFE,
        last: 0xEFFFF,
        value: 18,
    },
    BidiClassRange {
        first: 0xFFFFE,
        last: 0xFFFFF,
        value: 18,
    },
    BidiClassRange {
        first: 0x10FFFE,
        last: 0x10FFFF,
        value: 18,
    },
];

// cpp: foundation/blink_base/wtf/text/bidi_class_data.h:738-748
pub fn BidiClass(code_point: u32) -> u8 {
    let index = BIDI_CLASS_RANGES.partition_point(|range| range.first <= code_point);
    if index == 0 {
        return 0;
    }
    let range = BIDI_CLASS_RANGES[index - 1];
    if code_point <= range.last {
        range.value
    } else {
        0
    }
}
