//! Hash containers for trusted process-local identities.
//!
//! Blink's WTF hash tables hash pointer and integer identity keys with the
//! compact functions in `wtf/hash_functions.h`. Rust's default `HashMap`
//! intentionally uses a keyed SipHash-family hasher, which is unnecessary for
//! resident pointers and opaque IDs produced inside the rendering lifecycle.
//! Keep this adapter restricted to such trusted keys; ordinary content-derived
//! keys continue to use the standard randomized containers.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

/// Incremental adapter for tuples/enums composed of trusted integer or pointer
/// identities. Equality remains the final authority in `HashMap`/`HashSet`.
#[derive(Default)]
pub struct TrustedIdentityHasher(u64);

impl Hasher for TrustedIdentityHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = (self.0.rotate_left(5) ^ u64::from(byte)).wrapping_mul(0x9e37_79b1_85eb_ca87);
        }
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = (self.0.rotate_left(17) ^ value).wrapping_mul(0x9e37_79b1_85eb_ca87);
    }

    fn write_usize(&mut self, value: usize) {
        self.write_u64(value as u64);
    }
}

pub type TrustedIdentityHashMap<K, V> = HashMap<K, V, BuildHasherDefault<TrustedIdentityHasher>>;
pub type TrustedIdentityHashSet<K> = HashSet<K, BuildHasherDefault<TrustedIdentityHasher>>;
