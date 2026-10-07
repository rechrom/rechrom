// C++: font_engine/minimal_font.h
use std::collections::BTreeMap;

pub type Bytes = Vec<u8>;

// cpp: font_engine/minimal_font.h:12-16
pub const fn Tag(s: &[u8; 4]) -> u32 {
    (s[0] as u32) << 24 | (s[1] as u32) << 16 | (s[2] as u32) << 8 | s[3] as u32
}

// cpp: font_engine/minimal_font.h:17-20
pub fn Put16(bytes: &mut Bytes, offset: usize, value: i32) {
    let value = value as u16;
    bytes[offset] = (value >> 8) as u8;
    bytes[offset + 1] = (value & 255) as u8;
}

// cpp: font_engine/minimal_font.h:21-24
pub fn Put32(bytes: &mut Bytes, offset: usize, value: u32) {
    Put16(bytes, offset, (value >> 16) as i32);
    Put16(bytes, offset + 2, (value & 65535) as i32);
}

// cpp: font_engine/minimal_font.h:25-31
pub fn Checksum(bytes: &Bytes) -> u32 {
    let mut sum = 0u32;
    for (i, byte) in bytes.iter().enumerate() {
        sum = sum.wrapping_add((*byte as u32) << (24 - 8 * (i % 4)));
    }
    sum
}

// cpp: font_engine/minimal_font.h:35-40
pub fn Build(
    mut with_names: bool,
    name_language: u16,
    rectangle_character: u32,
    with_math: bool,
    web_font: bool,
) -> Bytes {
    let mut tables: BTreeMap<u32, Bytes> = BTreeMap::new();
    // cpp: font_engine/minimal_font.h:41-52
    if web_font {
        with_names = true;
        let os2 = tables.entry(Tag(b"OS/2")).or_default();
        os2.resize(78, 0);
        Put16(os2, 2, 500);
        Put16(os2, 4, 400);
        Put16(os2, 6, 5);
        Put16(os2, 62, 0x40);
        Put16(os2, 64, 0x20);
        Put16(os2, 66, 0x5d0);
        Put16(os2, 68, 800);
        Put16(os2, 70, -200);
        Put16(os2, 72, 100);
        Put16(os2, 74, 800);
        Put16(os2, 76, 200);
        let post = tables.entry(Tag(b"post")).or_default();
        post.resize(32, 0);
        Put32(post, 0, 0x30000);
    }

    // cpp: font_engine/minimal_font.h:53-76
    let head = tables.entry(Tag(b"head")).or_default();
    head.resize(54, 0);
    Put32(head, 0, 0x10000);
    Put32(head, 12, 0x5f0f3cf5);
    Put16(head, 18, 1000);
    Put16(head, 38, -100);
    Put16(head, 40, 650);
    Put16(head, 42, 800);
    let maxp = tables.entry(Tag(b"maxp")).or_default();
    maxp.resize(32, 0);
    Put32(maxp, 0, 0x10000);
    Put16(maxp, 4, 4);
    Put16(maxp, 6, 4);
    Put16(maxp, 8, 1);
    Put16(maxp, 14, 1);
    let hhea = tables.entry(Tag(b"hhea")).or_default();
    hhea.resize(36, 0);
    Put32(hhea, 0, 0x10000);
    Put16(hhea, 4, 800);
    Put16(hhea, 6, -200);
    Put16(hhea, 8, 100);
    Put16(hhea, 10, 700);
    Put16(hhea, 18, 1);
    Put16(hhea, 34, 4);
    let vhea = tables.entry(Tag(b"vhea")).or_default();
    vhea.resize(36, 0);
    Put32(vhea, 0, 0x10000);
    Put16(vhea, 4, 500);
    Put16(vhea, 6, -500);
    Put16(vhea, 10, 1200);
    Put16(vhea, 20, 1);
    Put16(vhea, 34, 4);
    let hmtx = tables.entry(Tag(b"hmtx")).or_default();
    hmtx.resize(16, 0);
    let vmtx = tables.entry(Tag(b"vmtx")).or_default();
    vmtx.resize(16, 0);
    let h_adv = [500, 250, 600, 700];
    let v_adv = [1000, 1000, 1100, 1200];
    for i in 0..4 {
        Put16(tables.get_mut(&Tag(b"hmtx")).unwrap(), i * 4, h_adv[i]);
        Put16(tables.get_mut(&Tag(b"vmtx")).unwrap(), i * 4, v_adv[i]);
    }
    Put16(tables.get_mut(&Tag(b"hmtx")).unwrap(), 14, 50);

    // cpp: font_engine/minimal_font.h:77-93
    let mut chars = vec![0x20, 0x30, rectangle_character, 0x5d0];
    let mut glyphs = vec![1, 2, 3, 3];
    if with_math {
        chars.extend_from_slice(&[0x221a, 0x222b]);
        glyphs.extend_from_slice(&[3, 3]);
    }
    let cmap = tables.entry(Tag(b"cmap")).or_default();
    cmap.resize(28 + chars.len() * 12, 0);
    Put16(cmap, 2, 1);
    Put16(cmap, 4, 3);
    Put16(cmap, 6, 10);
    Put32(cmap, 8, 12);
    Put16(cmap, 12, 12);
    Put32(cmap, 16, (cmap.len() - 12) as u32);
    Put32(cmap, 24, chars.len() as u32);
    for i in 0..chars.len() {
        Put32(cmap, 28 + i * 12, chars[i]);
        Put32(cmap, 32 + i * 12, chars[i]);
        Put32(cmap, 36 + i * 12, glyphs[i]);
    }

    // cpp: font_engine/minimal_font.h:94-108
    let glyf = tables.entry(Tag(b"glyf")).or_default();
    glyf.resize(68, 0);
    for i in 0..2 {
        let start = i * 34;
        let (x, y, width, height) = if i != 0 {
            (50, -100, 600, 900)
        } else {
            (0, 0, 500, 700)
        };
        Put16(glyf, start, 1);
        Put16(glyf, start + 2, x);
        Put16(glyf, start + 4, y);
        Put16(glyf, start + 6, x + width);
        Put16(glyf, start + 8, y + height);
        Put16(glyf, start + 10, 3);
        for j in 0..4 {
            glyf[start + 14 + j] = 1;
        }
        Put16(glyf, start + 18, x);
        Put16(glyf, start + 20, width);
        Put16(glyf, start + 24, -width);
        Put16(glyf, start + 26, y);
        Put16(glyf, start + 30, height);
    }
    let loca = tables.entry(Tag(b"loca")).or_default();
    loca.resize(10, 0);
    Put16(loca, 6, 17);
    Put16(loca, 8, 34);

    // cpp: font_engine/minimal_font.h:110-157
    if with_math {
        let math = tables.entry(Tag(b"MATH")).or_default();
        const K_CONSTANTS_OFFSET: usize = 10;
        const K_GLYPH_INFO_OFFSET: usize = 224;
        const K_VARIANTS_OFFSET: usize = 232;
        math.resize(258, 0);
        Put16(math, 0, 1);
        Put16(math, 2, 0);
        Put16(math, 4, K_CONSTANTS_OFFSET as i32);
        Put16(math, 6, K_GLYPH_INFO_OFFSET as i32);
        Put16(math, 8, K_VARIANTS_OFFSET as i32);
        let mut set_constant = |index: usize, value: i32| {
            let offset = if index < 4 {
                K_CONSTANTS_OFFSET + index * 2
            } else if index < 55 {
                K_CONSTANTS_OFFSET + 8 + (index - 4) * 4
            } else {
                K_CONSTANTS_OFFSET + 212
            };
            Put16(math, offset, value);
        };
        for (index, value) in [
            (0, 80),
            (1, 60),
            (3, 1000),
            (5, 250),
            (38, 40),
            (49, 80),
            (50, 100),
            (51, 40),
            (52, 40),
            (53, 50),
            (54, -100),
            (55, 60),
        ] {
            set_constant(index, value);
        }
        Put16(math, K_VARIANTS_OFFSET, 20);
        Put16(math, K_VARIANTS_OFFSET + 2, 12);
        Put16(math, K_VARIANTS_OFFSET + 6, 1);
        Put16(math, K_VARIANTS_OFFSET + 10, 18);
        Put16(math, K_VARIANTS_OFFSET + 12, 1);
        Put16(math, K_VARIANTS_OFFSET + 14, 1);
        Put16(math, K_VARIANTS_OFFSET + 16, 3);
        Put16(math, K_VARIANTS_OFFSET + 20, 1);
        Put16(math, K_VARIANTS_OFFSET + 22, 3);
        Put16(math, K_VARIANTS_OFFSET + 24, 900);
    }

    // cpp: font_engine/minimal_font.h:159-176
    if with_names {
        let name = tables.entry(Tag(b"name")).or_default();
        name.resize(42, 0);
        Put16(name, 2, 3);
        Put16(name, 4, 42);
        let ids = [1, 6, 16];
        let names = ["LayoutNG Legacy", "LayoutNG-Test", "LayoutNG 布局"];
        for (i, text) in names.iter().enumerate() {
            let record = 6 + i * 12;
            Put16(name, record, 3);
            Put16(name, record + 2, 1);
            Put16(name, record + 4, name_language as i32);
            Put16(name, record + 6, ids[i]);
            Put16(name, record + 8, (text.encode_utf16().count() * 2) as i32);
            Put16(name, record + 10, (name.len() - 42) as i32);
            for unit in text.encode_utf16() {
                let offset = name.len();
                name.resize(offset + 2, 0);
                Put16(name, offset, unit as i32);
            }
        }
    }

    // cpp: font_engine/minimal_font.h:178-196
    let mut sfnt = vec![0; 12 + tables.len() * 16];
    Put32(&mut sfnt, 0, 0x10000);
    Put16(&mut sfnt, 4, tables.len() as i32);
    let table_power = 1usize << (usize::BITS - 1 - tables.len().leading_zeros());
    Put16(&mut sfnt, 6, (table_power * 16) as i32);
    Put16(
        &mut sfnt,
        8,
        (usize::BITS - 1 - table_power.leading_zeros()) as i32,
    );
    Put16(&mut sfnt, 10, ((tables.len() - table_power) * 16) as i32);
    let (mut directory, mut head_offset) = (12, 0);
    for (tag, bytes) in tables {
        Put32(&mut sfnt, directory, tag);
        Put32(&mut sfnt, directory + 4, Checksum(&bytes));
        let table_offset = sfnt.len() as u32;
        Put32(&mut sfnt, directory + 8, table_offset);
        Put32(&mut sfnt, directory + 12, bytes.len() as u32);
        if tag == Tag(b"head") {
            head_offset = sfnt.len();
        }
        sfnt.extend_from_slice(&bytes);
        sfnt.resize((sfnt.len() + 3) & !3usize, 0);
        directory += 16;
    }
    let checksum_adjustment = 0xb1b0afbau32.wrapping_sub(Checksum(&sfnt));
    Put32(&mut sfnt, head_offset + 8, checksum_adjustment);
    assert_eq!(Checksum(&sfnt), 0xb1b0afba);
    sfnt
}
