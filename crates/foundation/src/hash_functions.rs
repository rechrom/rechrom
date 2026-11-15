// C++: src/foundation/blink_base/wtf/hash_functions.h:63-109,145-161
// cpp: foundation/blink_base/wtf/hash_functions.h:63-71

// C++ internal::HashInt(uint32_t), including unsigned wraparound.
pub const fn HashInt(mut key: u32) -> u32 {
    key = key.wrapping_add(!(key << 15));
    key ^= key >> 10;
    key = key.wrapping_add(key << 3);
    key ^= key >> 6;
    key = key.wrapping_add(!(key << 11));
    key ^= key >> 16;
    key
}

// cpp: foundation/blink_base/wtf/hash_functions.h:85-98
pub const fn HashInt64(mut key: u64) -> u32 {
    key = key.wrapping_add(!(key << 32));
    key ^= key >> 22;
    key = key.wrapping_add(!(key << 13));
    key ^= key >> 8;
    key = key.wrapping_add(key << 3);
    key ^= key >> 15;
    key = key.wrapping_add(!(key << 27));
    key ^= key >> 31;
    key as u32
}

// cpp: foundation/blink_base/wtf/hash_functions.h:140-143
pub fn HashPointer<T>(key: *const T) -> u32 {
    if std::mem::size_of::<usize>() == 8 {
        HashInt64(key as usize as u64)
    } else {
        HashInt(key as usize as u32)
    }
}

// GenericHashTraits<P*>::GetHash delegates to HashPointer.
// cpp: foundation/blink_base/wtf/hash_traits.h:277-280
pub fn GetHash<T>(key: *const T) -> u32 {
    HashPointer(key)
}

// cpp: foundation/blink_base/wtf/hash_functions.h:101-109
pub const fn HashInts(key1: u32, key2: u32) -> u32 {
    let short_random1 = 277_951_225u64;
    let short_random2 = 95_187_966u64;
    let long_random = 19_248_658_165_952_623u64;
    let product = long_random
        .wrapping_mul(short_random1)
        .wrapping_mul(key1 as u64)
        .wrapping_add(
            long_random
                .wrapping_mul(short_random2)
                .wrapping_mul(key2 as u64),
        );
    (product >> 32) as u32
}

// cpp: foundation/blink_base/wtf/hash_functions.h:119-125
pub const fn NormalizeSign(value: f32) -> f32 {
    value + 0.0
}
// cpp: foundation/blink_base/wtf/hash_functions.h:127-132
pub const fn HashFloat(value: f32) -> u32 {
    HashInt(NormalizeSign(value).to_bits())
}
// cpp: foundation/blink_base/wtf/hash_functions.h:134-139
pub const fn FloatEqualForHash(a: f32, b: f32) -> bool {
    NormalizeSign(a).to_bits() == NormalizeSign(b).to_bits()
}
// cpp: foundation/blink_base/wtf/hash_functions.h:145-148
pub const fn AddIntToHash(hash: &mut u32, key: u32) {
    *hash = hash.wrapping_mul(33).wrapping_add(key);
}
// cpp: foundation/blink_base/wtf/hash_functions.h:150-155
pub const fn AddFloatToHash(hash: &mut u32, value: f32) {
    AddIntToHash(hash, HashFloat(value));
}
