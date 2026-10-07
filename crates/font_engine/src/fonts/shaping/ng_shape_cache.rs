// C++: font_engine/fonts/shaping/ng_shape_cache.h.
// The key owns text and locale; result values retain source GC strength.
use super::font_features::FontFeatureRange;
use super::shape_result::ShapeResult;
use crate::fonts::simple_font_data::SimpleFontData;
#[cfg(feature = "expensive_dchecks")]
use crate::text::native::character::Character;
use foundation::heap_hash_containers::StrongHeapMapKey;
use foundation::{
    AddIntToHash, AtomicString, HeapHashMap, Member, RuntimeEnabledFeatures, String, TextDirection,
    Traceable, Visitor, WeakMember,
};
use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};

// cpp: font_engine/fonts/shaping/ng_shape_cache.h:56-61
pub struct ShaperResult {
    pub shape_result: *const ShapeResult,
    pub can_cache: bool,
}

// cpp: font_engine/fonts/shaping/ng_shape_cache.h:67-119
#[derive(Clone, Debug)]
pub struct ShapeCacheKey {
    text_: String,
    start_offset_: u32,
    end_offset_: u32,
    locale_: AtomicString,
    font_features_: Vec<FontFeatureRange>,
    direction_: TextDirection,
}

impl Default for ShapeCacheKey {
    fn default() -> Self {
        Self {
            text_: String::default(),
            start_offset_: 0,
            end_offset_: 0,
            locale_: AtomicString::default(),
            font_features_: Vec::new(),
            direction_: TextDirection::kLtr,
        }
    }
}

#[allow(non_snake_case)]
impl ShapeCacheKey {
    // cpp: font_engine/fonts/shaping/ng_shape_cache.h:72-85
    pub fn new(
        text: &String,
        start_offset: u32,
        end_offset: u32,
        locale: &AtomicString,
        font_features: &[FontFeatureRange],
        direction: TextDirection,
    ) -> Self {
        assert!(
            !text.empty(),
            "the empty string is a deleted hash-table key"
        );
        Self {
            text_: text.clone(),
            start_offset_: start_offset,
            end_offset_: end_offset,
            locale_: locale.clone(),
            font_features_: font_features.to_vec(),
            direction_: direction,
        }
    }

    // cpp: font_engine/fonts/shaping/ng_shape_cache.h:87-89
    pub fn deleted() -> Self {
        Self {
            text_: String::from(""),
            ..Self::default()
        }
    }
    pub fn IsHashTableDeletedValue(&self) -> bool {
        self.text_ == String::from("")
    }

    // BlinkString currently keeps UTF-16 units but not StringImpl's original
    // 8-bit/16-bit flag. Latin-1-compatible text takes the usual 8-bit path;
    // exact hash parity for an originally 16-bit Latin-1 String is pending on
    // foundation's String representation. Key equality remains exact.
    fn text_hash(&self) -> u32 {
        let units = self.text_.Span16().unwrap_or_default();
        let bytes = if units.iter().all(|unit| *unit <= u8::MAX as u16) {
            units.iter().map(|unit| *unit as u8).collect::<Vec<_>>()
        } else {
            units
                .iter()
                .flat_map(|unit| unit.to_ne_bytes())
                .collect::<Vec<_>>()
        };
        foundation::rapidhash::ComputeHashAndMaskTop8Bits(&bytes)
    }

    // cpp: font_engine/fonts/shaping/ng_shape_cache.h:91-100
    pub fn GetHash(&self) -> u32 {
        let mut hash = self.text_hash();
        AddIntToHash(&mut hash, self.start_offset_);
        AddIntToHash(&mut hash, self.end_offset_);
        AddIntToHash(
            &mut hash,
            if self.locale_.IsNull() {
                0
            } else {
                self.locale_.Hash()
            },
        );
        // FontFeatureRange is repr(C) with four initialized u32 fields.
        // The slice represents the source as_byte_span + HashMemory32 call.
        let bytes = unsafe {
            std::slice::from_raw_parts(
                self.font_features_.as_ptr().cast::<u8>(),
                self.font_features_.len() * std::mem::size_of::<FontFeatureRange>(),
            )
        };
        AddIntToHash(&mut hash, foundation::rapidhash::rapidhash(bytes) as u32);
        AddIntToHash(&mut hash, self.direction_ as u32);
        hash
    }

    // cpp: font_engine/fonts/shaping/ng_shape_cache.h:110
    pub fn GetText(&self) -> &String {
        &self.text_
    }
}

// cpp: font_engine/fonts/shaping/ng_shape_cache.h:102-107
impl PartialEq for ShapeCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.text_ == other.text_
            && self.start_offset_ == other.start_offset_
            && self.end_offset_ == other.end_offset_
            && self.locale_ == other.locale_
            && self.font_features_ == other.font_features_
            && self.direction_ == other.direction_
    }
}
impl Eq for ShapeCacheKey {}
impl Hash for ShapeCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.GetHash());
    }
}
// The key's strings own non-layout-heap storage; only result values have GC edges.
impl Traceable for ShapeCacheKey {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}
impl StrongHeapMapKey for ShapeCacheKey {}

type WeakMap = HeapHashMap<ShapeCacheKey, WeakMember<ShapeResult>>;
type StrongMap = HeapHashMap<ShapeCacheKey, Member<ShapeResult>>;

trait ShapeCacheValue: Default + Traceable {
    fn get(&self) -> *mut ShapeResult;
    fn from_result(result: *const ShapeResult) -> Self;
}
impl ShapeCacheValue for WeakMember<ShapeResult> {
    fn get(&self) -> *mut ShapeResult {
        self.Get()
    }
    fn from_result(result: *const ShapeResult) -> Self {
        Self::from_ptr(result.cast_mut())
    }
}
impl ShapeCacheValue for Member<ShapeResult> {
    fn get(&self) -> *mut ShapeResult {
        self.Get()
    }
    fn from_result(result: *const ShapeResult) -> Self {
        Self::from_ptr(result.cast_mut())
    }
}

// cpp: font_engine/fonts/shaping/ng_shape_cache.h:123-129,166-167,212-214
pub struct NGShapeCache {
    weak_map_: WeakMap,
    strong_map_: StrongMap,
    primary_font_: Member<SimpleFontData>,
}

#[allow(non_snake_case)]
impl NGShapeCache {
    pub const kMaxTextLengthOfEntries: u32 = 30;
    pub const kMaxSize: u32 = 2048;

    // cpp: font_engine/fonts/shaping/ng_shape_cache.h:129-134
    pub fn new(primary_font: *const SimpleFontData) -> Self {
        assert!(!primary_font.is_null());
        Self {
            weak_map_: WeakMap::default(),
            strong_map_: StrongMap::default(),
            primary_font_: Member::from_ptr(primary_font.cast_mut()),
        }
    }

    // cpp: font_engine/fonts/shaping/ng_shape_cache.h:143-146
    pub fn OnReleaseMemory(&mut self) {
        self.weak_map_.clear();
        self.strong_map_.clear();
    }

    // cpp: font_engine/fonts/shaping/ng_shape_cache.h:150-163
    pub fn GetOrCreate<F>(
        &mut self,
        key: &ShapeCacheKey,
        shape_result_func: &F,
    ) -> *const ShapeResult
    where
        F: Fn() -> ShaperResult,
    {
        assert!(!key.GetText().empty());
        if key.GetText().length() > Self::kMaxTextLengthOfEntries {
            return shape_result_func().shape_result;
        }
        let primary_font = self.primary_font_.Get();
        if RuntimeEnabledFeatures::MemoryConsumerForNGShapeCacheEnabled() {
            Self::GetOrCreateImpl(&mut self.strong_map_, key, shape_result_func, primary_font)
        } else {
            Self::GetOrCreateImpl(&mut self.weak_map_, key, shape_result_func, primary_font)
        }
    }

    // cpp: font_engine/fonts/shaping/ng_shape_cache.h:170-209
    fn GetOrCreateImpl<V, F>(
        map: &mut HeapHashMap<ShapeCacheKey, V>,
        key: &ShapeCacheKey,
        shape_result_func: &F,
        primary_font: *mut SimpleFontData,
    ) -> *const ShapeResult
    where
        V: ShapeCacheValue,
        F: Fn() -> ShaperResult,
    {
        if map.size() >= Self::kMaxSize {
            if let Some(cached_result) = map.get(key).map(ShapeCacheValue::get) {
                if !cached_result.is_null() {
                    return cached_result;
                }
            }
            return shape_result_func().shape_result;
        }

        let slot = match map.entry(key.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(V::default()),
        };
        let cached_result = slot.get();
        if !cached_result.is_null() {
            // cpp: font_engine/fonts/shaping/ng_shape_cache.h:187-203
            #[cfg(feature = "expensive_dchecks")]
            {
                let other = shape_result_func();
                let has_private_or_non_characters = key
                    .GetText()
                    .Span16()
                    .unwrap_or_default()
                    .iter()
                    .any(|unit| {
                        Character::IsPrivateUse(*unit as i32)
                            || Character::IsNonCharacter(*unit as i32)
                    });
                if other.can_cache && !has_private_or_non_characters {
                    assert!(unsafe { &*cached_result } == unsafe { &*other.shape_result });
                }
            }
            return cached_result;
        }

        let ShaperResult {
            shape_result,
            can_cache,
        } = shape_result_func();
        // cpp: font_engine/fonts/shaping/ng_shape_cache.h:207-209
        if can_cache && !unsafe { &*shape_result }.HasFallbackFonts(primary_font) {
            *slot = V::from_result(shape_result);
        }
        shape_result
    }

    // Existing SimpleFontData integration reads this owning-font edge.
    pub fn PrimaryFont(&self) -> *const SimpleFontData {
        self.primary_font_.Get()
    }
}

// cpp: font_engine/fonts/shaping/ng_shape_cache.h:138-142
impl Traceable for NGShapeCache {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.weak_map_);
        visitor.Trace(&self.strong_map_);
        visitor.Trace(&self.primary_font_);
    }
}
