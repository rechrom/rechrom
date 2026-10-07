use foundation::{
    g_null_atom, AtomicString, AtomicStringHashTraits, GCedHeapHashMap, HashInts, Member, Vector,
    Visitor,
};

use super::applied_text_decoration::AppliedTextDecorationVector;
use super::computed_style::ComputedStyle;
use super::computed_style_constants::PseudoId;

impl foundation::Traceable for StyleCachedData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        StyleCachedData::Trace(self, visitor);
    }
}

// cpp: layoutng_style/style/style_cached_data.h:23-31
pub struct PseudoElementStyleCacheKey {
    pub pseudo_type: PseudoId,
    pub pseudo_argument: AtomicString,
}

// cpp: layoutng_style/style/style_cached_data.h:27-30
impl PartialEq for PseudoElementStyleCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.pseudo_type == other.pseudo_type && self.pseudo_argument == other.pseudo_argument
    }
}

// cpp: layoutng_style/style/style_cached_data.h:33-59
pub struct PseudoElementStyleCacheKeyHashTraits;

#[allow(non_snake_case, non_upper_case_globals)]
impl PseudoElementStyleCacheKeyHashTraits {
    // cpp: layoutng_style/style/style_cached_data.h:37-41
    pub fn GetHash(key: &PseudoElementStyleCacheKey) -> u32 {
        HashInts(
            u32::from(key.pseudo_type.value()),
            if key.pseudo_argument.IsNull() {
                0
            } else {
                key.pseudo_argument.Hash()
            },
        )
    }

    // cpp: layoutng_style/style/style_cached_data.h:43
    pub const kEmptyValueIsZero: bool = false;

    // cpp: layoutng_style/style/style_cached_data.h:44-48
    pub fn ConstructDeletedValue(slot: &mut PseudoElementStyleCacheKey) {
        *slot = PseudoElementStyleCacheKey {
            pseudo_type: PseudoId::kPseudoIdNone,
            pseudo_argument: g_null_atom.clone(),
        };
        AtomicStringHashTraits::ConstructDeletedValue(&mut slot.pseudo_argument);
    }

    // cpp: layoutng_style/style/style_cached_data.h:49-51
    pub fn IsDeletedValue(value: &PseudoElementStyleCacheKey) -> bool {
        AtomicStringHashTraits::IsDeletedValue(&value.pseudo_argument)
    }

    // cpp: layoutng_style/style/style_cached_data.h:52-55
    pub fn IsEmptyValue(value: &PseudoElementStyleCacheKey) -> bool {
        value.pseudo_type == PseudoId::kPseudoIdNone && value.pseudo_argument.IsNull()
    }

    // cpp: layoutng_style/style/style_cached_data.h:56-58
    pub fn EmptyValue() -> PseudoElementStyleCacheKey {
        PseudoElementStyleCacheKey {
            pseudo_type: PseudoId::kPseudoIdNone,
            pseudo_argument: g_null_atom.clone(),
        }
    }
}

// cpp: layoutng_style/style/style_cached_data.h:61-62
pub type PseudoElementStyleCache =
    GCedHeapHashMap<PseudoElementStyleCacheKey, Member<ComputedStyle>>;

// cpp: layoutng_style/style/style_cached_data.h:64-104
#[derive(Default)]
pub struct StyleCachedData {
    pub(crate) pseudo_element_styles_: Member<PseudoElementStyleCache>,
    pub(crate) variable_names_: Option<Box<Vector<AtomicString>>>,
    pub(crate) applied_text_decorations_: Member<AppliedTextDecorationVector>,
}

#[allow(non_snake_case)]
impl StyleCachedData {
    // cpp: layoutng_style/style/style_cached_data.h:68-71
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.pseudo_element_styles_);
        visitor.Trace(&self.applied_text_decorations_);
    }
}
