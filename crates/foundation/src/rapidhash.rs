// C++: foundation/blink_base/wtf/text/rapidhash.h and string_hasher.h.
// Plain byte reader specialization used by 8-bit and genuine UTF-16 strings.

const RAPID_SEED: u64 = 0xbdd89aa982704029;
const RAPID_SECRET: [u64; 3] = [0x2d358dccaa6c78a5, 0x8bb84b93962eacc9, 0x4b33a62ed433d4a3];

// cpp: foundation/blink_base/wtf/text/rapidhash.h:187-230
fn rapid_mul128(a: u64, b: u64) -> (u64, u64) {
    let product = (a as u128) * (b as u128);
    (product as u64, (product >> 64) as u64)
}

// cpp: foundation/blink_base/wtf/text/rapidhash.h:232-246
fn rapid_mix(a: u64, b: u64) -> u64 {
    let (low, high) = rapid_mul128(a, b);
    low ^ high
}

// cpp: foundation/blink_base/wtf/text/rapidhash.h:105-146
fn read32(data: &[u8], offset: usize) -> u64 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as u64
}

fn read64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap())
}

fn read_small(data: &[u8]) -> u64 {
    let len = data.len();
    ((data[0] as u64) << 56) | ((data[len >> 1] as u64) << 32) | data[len - 1] as u64
}

// cpp: foundation/blink_base/wtf/text/rapidhash.h:260-343
pub fn rapidhash(data: &[u8]) -> u64 {
    let len = data.len();
    let mut seed = RAPID_SEED;
    seed ^= rapid_mix(seed ^ RAPID_SECRET[0], RAPID_SECRET[1]) ^ len as u64;
    let (mut a, mut b);
    if len <= 16 {
        if len >= 4 {
            let last = len - 4;
            a = (read32(data, 0) << 32) | read32(data, last);
            let delta = (len & 24) >> (len >> 3);
            b = (read32(data, delta) << 32) | read32(data, last - delta);
        } else if len > 0 {
            a = read_small(data);
            b = 0;
        } else {
            a = 0;
            b = 0;
        }
    } else {
        let mut remaining = len;
        let mut position = 0;
        if remaining > 48 {
            let mut see1 = seed;
            let mut see2 = seed;
            loop {
                seed = rapid_mix(
                    read64(data, position) ^ RAPID_SECRET[0],
                    read64(data, position + 8) ^ seed,
                );
                see1 = rapid_mix(
                    read64(data, position + 16) ^ RAPID_SECRET[1],
                    read64(data, position + 24) ^ see1,
                );
                see2 = rapid_mix(
                    read64(data, position + 32) ^ RAPID_SECRET[2],
                    read64(data, position + 40) ^ see2,
                );
                position += 48;
                remaining -= 48;
                if remaining < 48 {
                    break;
                }
            }
            seed ^= see1 ^ see2;
        }
        if remaining > 16 {
            seed = rapid_mix(
                read64(data, position) ^ RAPID_SECRET[2],
                read64(data, position + 8) ^ seed ^ RAPID_SECRET[1],
            );
            if remaining > 32 {
                seed = rapid_mix(
                    read64(data, position + 16) ^ RAPID_SECRET[2],
                    read64(data, position + 24) ^ seed,
                );
            }
        }
        a = read64(data, position + remaining - 16);
        b = read64(data, position + remaining - 8);
    }
    a ^= RAPID_SECRET[1];
    b ^= seed;
    (a, b) = rapid_mul128(a, b);
    rapid_mix(a ^ RAPID_SECRET[0] ^ len as u64, b ^ RAPID_SECRET[1])
}

// cpp: foundation/blink_base/wtf/text/string_hasher.h:53-58,103-118
pub fn ComputeHashAndMaskTop8Bits(data: &[u8]) -> u32 {
    let masked = (rapidhash(data) as u32) & 0x00ff_ffff;
    if masked == 0 {
        0x0080_0000
    } else {
        masked
    }
}
