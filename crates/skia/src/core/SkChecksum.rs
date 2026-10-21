// Source: src/core/SkChecksum.h:30-37
pub fn Mix(mut hash: u32) -> u32 {
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x85ebca6b);
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(0xc2b2ae35);
    hash ^= hash >> 16;
    hash
}
// Source: src/core/SkChecksum.h:45-50
pub fn CheapMix(mut hash: u32) -> u32 {
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x85ebca6b);
    hash ^= hash >> 16;
    hash
}
// Rust substitutes this explicit unsafe representation bound for the C++ unique-object-representation trait.
// Safety: every byte must be initialized, with no padding and one representation per equal value.
pub unsafe trait UniqueObjectRepresentation: Sized {}
macro_rules! unique_integer { ($($t:ty),*) => { $(unsafe impl UniqueObjectRepresentation for $t {})* }; }
unique_integer!(bool, u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
unsafe impl<T: UniqueObjectRepresentation, const N: usize> UniqueObjectRepresentation for [T; N] {}
// Source: src/core/SkChecksum.h:74-79
// Source: src/core/SkChecksum.h:81-85
pub struct SkGoodHash;
// Stateless C++ HashK() callable construction, Rust protocol glue only; no source credit.
impl Default for SkGoodHash {
    fn default() -> Self {
        Self
    }
}
impl SkGoodHash {
    pub fn hash<K: UniqueObjectRepresentation>(&self, key: &K) -> u32 {
        let bytes = unsafe {
            std::slice::from_raw_parts((key as *const K).cast::<u8>(), std::mem::size_of::<K>())
        };
        if bytes.len() == 4 {
            Mix(u32::from_ne_bytes(bytes.try_into().unwrap()))
        } else {
            Hash32(bytes, 0)
        }
    }
    /// Adapted, uncredited thin-object-pointer representation bridge for the selected ARM64 ABI.
    /// Hashes only the explicit numeric address, never pointee bytes or provenance-bearing
    /// pointer storage. No dereference, fat-pointer, or cross-target ABI claim is made.
    pub fn hash_thin_pointer<T: Sized>(&self, pointer: *const T) -> u32 {
        self.hash(&pointer.addr())
    }
    // Source: src/core/SkChecksum.h:87-89
    pub fn hash_sk_string_bytes(&self, key: &[u8]) -> u32 {
        Hash32(key, 0)
    }
    // Source: src/core/SkChecksum.h:91-93
    pub fn hash_std_string_bytes(&self, key: &[u8]) -> u32 {
        Hash32(key, 0)
    }
    // Source: src/core/SkChecksum.h:95-98
    pub fn hash_string_view(&self, key: &[u8]) -> u32 {
        Hash32(key, 0)
    }
}
// Source: src/core/SkChecksum.h:111-116
pub struct SkForceDirectHash<K>(std::marker::PhantomData<K>);
impl<K> Default for SkForceDirectHash<K> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}
impl<K> SkForceDirectHash<K> {
    // Safety: all object bytes, including padding, must be initialized. Key representations must
    // satisfy the original caller's direct-hash equality contract.
    pub unsafe fn hash(&self, key: &K) -> u32 {
        let bytes = unsafe {
            std::slice::from_raw_parts((key as *const K).cast::<u8>(), std::mem::size_of::<K>())
        };
        Hash32(bytes, 0)
    }
}
// Source: src/core/SkChecksum.cpp:16-16
// Source: src/core/SkChecksum.cpp:18-21
#[cfg(not(all(
    target_pointer_width = "32",
    any(target_arch = "x86", target_arch = "arm")
)))]
fn wymum(a: &mut u64, b: &mut u64) {
    let mut r = *a as u128;
    r *= *b as u128;
    *a = r as u64;
    *b = (r >> 64) as u64;
}
// Source: src/core/SkChecksum.cpp:25-32
// Selected 32-bit ARM/x86 C targets lack __SIZEOF_INT128__; retain their
// four-half multiplication and explicit carry algorithm, with unsigned wrap.
#[cfg(all(
    target_pointer_width = "32",
    any(target_arch = "x86", target_arch = "arm")
))]
fn wymum(a: &mut u64, b: &mut u64) {
    let ha = *a >> 32;
    let hb = *b >> 32;
    let la = *a as u32 as u64;
    let lb = *b as u32 as u64;
    let rh = ha.wrapping_mul(hb);
    let rm0 = ha.wrapping_mul(lb);
    let rm1 = hb.wrapping_mul(la);
    let rl = la.wrapping_mul(lb);
    let t = rl.wrapping_add(rm0 << 32);
    let mut c = (t < rl) as u64;
    let lo = t.wrapping_add(rm1 << 32);
    c += (lo < t) as u64;
    let hi = rh
        .wrapping_add(rm0 >> 32)
        .wrapping_add(rm1 >> 32)
        .wrapping_add(c);
    *a = lo;
    *b = hi;
}
// Source: src/core/SkChecksum.cpp:37-40
fn wymix(mut a: u64, mut b: u64) -> u64 {
    wymum(&mut a, &mut b);
    a ^ b
}
// Source: src/core/SkChecksum.cpp:43-47
fn wyr8(p: &[u8], offset: usize) -> u64 {
    u64::from_ne_bytes(p[offset..offset + 8].try_into().unwrap())
}
// Source: src/core/SkChecksum.cpp:49-53
fn wyr4(p: &[u8], offset: usize) -> u64 {
    u32::from_ne_bytes(p[offset..offset + 4].try_into().unwrap()) as u64
}
// Source: src/core/SkChecksum.cpp:55-57
fn wyr3(p: &[u8], k: usize) -> u64 {
    ((p[0] as u64) << 16) | ((p[k >> 1] as u64) << 8) | p[k - 1] as u64
}
// Source: src/core/SkChecksum.cpp:60-98
fn wyhash(key: &[u8], mut seed: u64, secret: &[u64; 4]) -> u64 {
    let len = key.len();
    let mut p = 0;
    seed ^= wymix(seed ^ secret[0], secret[1]);
    let (mut a, mut b);
    if len <= 16 {
        if len >= 4 {
            a = (wyr4(key, p) << 32) | wyr4(key, p + ((len >> 3) << 2));
            b = (wyr4(key, p + len - 4) << 32) | wyr4(key, p + len - 4 - ((len >> 3) << 2));
        } else if len > 0 {
            a = wyr3(key, len);
            b = 0;
        } else {
            a = 0;
            b = 0;
        }
    } else {
        let mut i = len;
        if i > 48 {
            let (mut see1, mut see2) = (seed, seed);
            loop {
                seed = wymix(wyr8(key, p) ^ secret[1], wyr8(key, p + 8) ^ seed);
                see1 = wymix(wyr8(key, p + 16) ^ secret[2], wyr8(key, p + 24) ^ see1);
                see2 = wymix(wyr8(key, p + 32) ^ secret[3], wyr8(key, p + 40) ^ see2);
                p += 48;
                i -= 48;
                if i <= 48 {
                    break;
                }
            }
            seed ^= see1 ^ see2;
        }
        while i > 16 {
            seed = wymix(wyr8(key, p) ^ secret[1], wyr8(key, p + 8) ^ seed);
            i -= 16;
            p += 16;
        }
        a = wyr8(key, p + i - 16);
        b = wyr8(key, p + i - 8);
    }
    a ^= secret[1];
    b ^= seed;
    wymum(&mut a, &mut b);
    wymix(a ^ secret[0] ^ len as u64, b ^ secret[1])
}
// Source: src/core/SkChecksum.cpp:101-102
const WYP: [u64; 4] = [
    0xa0761d6478bd642f,
    0xe7037ed1a0b428db,
    0x8ebc6af09c88c6e3,
    0x589965cc75374cc3,
];
// Source: src/core/SkChecksum.cpp:106-108
pub fn Hash32(data: &[u8], seed: u32) -> u32 {
    wyhash(data, seed as u64, &WYP) as u32
}
// Source: src/core/SkChecksum.cpp:110-112
pub fn Hash64(data: &[u8], seed: u64) -> u64 {
    wyhash(data, seed, &WYP)
}

// Source: src/core/SkChecksum.h:59-59
pub fn Hash32_default(data: &[u8]) -> u32 {
    Hash32(data, 0)
}
// Source: src/core/SkChecksum.h:68-68
pub fn Hash64_default(data: &[u8]) -> u64 {
    Hash64(data, 0)
}
