use std::cell::{Cell, OnceCell};

use font_engine::FontOrientation;
use foundation::{
    ETextOrientation, EUserSelect, IsHorizontalTypographicMode, MakeGarbageCollected, Member,
    Persistent,
};

use super::computed_grid_track_list::ComputedGridTrackList;
use super::computed_style::{ComputedStyle, ComputedStyleBuilder};
use super::computed_style_base::{ComputedStyleBase, ComputedStyleBuilderBase, IsAtShadowBoundary};
use super::style_cached_data::StyleCachedData;

// cpp: layoutng_style/style/computed_style_data.cc:26
impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            base_: ComputedStyleBase::default(),
            cached_data_: Cell::new(Member::<StyleCachedData>::default()),
        }
    }
}

#[allow(non_snake_case)]
impl ComputedStyle {
    // cpp: layoutng_style/style/computed_style.h:335
    // cpp: layoutng_style/style/computed_style_data.cc:28-29
    pub(crate) fn from_initial_style(initial_style: &Self) -> Self {
        Self {
            base_: initial_style.base_.clone(),
            cached_data_: Cell::new(Member::default()),
        }
    }

    // cpp: layoutng_style/style/computed_style.h:336
    // cpp: layoutng_style/style/computed_style_data.cc:31-32
    pub(crate) fn from_builder(builder: &ComputedStyleBuilder) -> Self {
        Self {
            base_: ComputedStyleBase::from_builder(&builder.base_),
            cached_data_: Cell::new(Member::default()),
        }
    }

    // cpp: layoutng_style/style/computed_style.h:334
    // cpp: layoutng_style/style/computed_style.h:344-345
    // cpp: layoutng_style/style/computed_style_data.cc:34-42
    pub(crate) fn new_with_pass_key(_key: ComputedStylePassKey) -> Self {
        Self::default()
    }
    // cpp: layoutng_style/style/computed_style.h:342-343
    pub(crate) fn new_with_builder_pass_key_from_initial(
        _key: ComputedStyleBuilderPassKey,
        initial_style: &Self,
    ) -> Self {
        Self::from_initial_style(initial_style)
    }
    pub(crate) fn new_with_builder_pass_key(
        _key: ComputedStyleBuilderPassKey,
        builder: &ComputedStyleBuilder,
    ) -> Self {
        Self::from_builder(builder)
    }

    // cpp: layoutng_style/style/computed_style.h:355
    // cpp: layoutng_style/style/computed_style_data.cc:44-52
    pub fn GetInitialStyleSingleton() -> *const Self {
        thread_local! {
            static INITIAL_STYLE: OnceCell<Persistent<ComputedStyle>> = OnceCell::new();
        }
        INITIAL_STYLE.with(|persistent| {
            persistent
                .get_or_init(|| {
                    Persistent::from_ptr(MakeGarbageCollected(Self::new_with_pass_key(
                        ComputedStylePassKey,
                    )))
                })
                .Get()
        })
    }

    // cpp: layoutng_style/style/computed_style.h:2732-2733
    // cpp: layoutng_style/style/computed_style_data.cc:95-105
    pub fn ComputedGridTemplate(
        track_list: &Member<ComputedGridTrackList>,
    ) -> &ComputedGridTrackList {
        let ptr = track_list.Get();
        if !ptr.is_null() {
            return unsafe { &*ptr };
        }
        thread_local! {
            static DEFAULT_TRACK_LIST: Persistent<ComputedGridTrackList> =
                Persistent::from_ptr(MakeGarbageCollected(ComputedGridTrackList::default()));
        }
        DEFAULT_TRACK_LIST.with(|persistent| unsafe { &*persistent.Get() })
    }
}

// C++ PassKey restricts construction; the actual call sites stay in this module.
pub(crate) struct ComputedStylePassKey;
pub(crate) struct ComputedStyleBuilderPassKey;

#[allow(non_snake_case)]
impl ComputedStyleBuilder {
    // cpp: layoutng_style/style/computed_style.h:2928
    // cpp: layoutng_style/style/computed_style_data.cc:53-54
    pub fn from_style(style: &ComputedStyle) -> Self {
        Self {
            base_: ComputedStyleBuilderBase::from_style(&style.base_),
            has_own_animations_: Cell::new(false),
            has_own_transitions_: Cell::new(false),
        }
    }

    // cpp: layoutng_style/style/computed_style.h:2931-2935
    // cpp: layoutng_style/style/computed_style_data.cc:56-78
    pub fn from_initial_and_parent(
        initial_style: &ComputedStyle,
        parent_style: &ComputedStyle,
        is_at_shadow_boundary: IsAtShadowBoundary,
    ) -> Self {
        let mut result = Self {
            base_: ComputedStyleBuilderBase::from_parent_and_noninherited(
                &initial_style.base_,
                &parent_style.base_,
            ),
            has_own_animations_: Cell::new(false),
            has_own_transitions_: Cell::new(false),
        };
        if is_at_shadow_boundary == IsAtShadowBoundary::kAtShadowBoundary {
            result.SetUserModify(initial_style.UserModify());
        }
        if parent_style.UserSelect() == EUserSelect::kContain {
            result.SetUserSelect(EUserSelect::kAuto);
        }
        result
    }

    // cpp: layoutng_style/style/computed_style.h:2942
    // cpp: layoutng_style/style/computed_style_data.cc:80-83
    pub fn TakeStyle(&mut self) -> *const ComputedStyle {
        MakeGarbageCollected(ComputedStyle::new_with_builder_pass_key(
            ComputedStyleBuilderPassKey,
            self,
        ))
    }

    // cpp: layoutng_style/style/computed_style.h:2945
    // cpp: layoutng_style/style/computed_style_data.cc:85-91
    pub fn CloneStyle(&self) -> *const ComputedStyle {
        self.ResetAccess();
        self.has_own_animations_.set(false);
        self.has_own_transitions_.set(false);
        MakeGarbageCollected(ComputedStyle::new_with_builder_pass_key(
            ComputedStyleBuilderPassKey,
            self,
        ))
    }

    // cpp: layoutng_style/style/computed_style.h:3251
    // cpp: layoutng_style/style/computed_style_data.cc:107-123
    pub fn ComputeFontOrientation(&self) -> FontOrientation {
        if IsHorizontalTypographicMode(self.GetWritingMode()) {
            return FontOrientation::kHorizontal;
        }
        match self.GetTextOrientation() {
            ETextOrientation::kMixed => FontOrientation::kVerticalMixed,
            ETextOrientation::kUpright => FontOrientation::kVerticalUpright,
            ETextOrientation::kSideways => FontOrientation::kVerticalRotated,
            _ => unreachable!("unexpected text orientation"),
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3506
    // cpp: layoutng_style/style/computed_style_data.cc:125-134
    pub fn SetEffectiveZoom(&mut self, f: f32) -> bool {
        let clamped_effective_zoom = f.clamp(1e-6, 1e6);
        if self.EffectiveZoom() == clamped_effective_zoom {
            return false;
        }
        self.SetEffectiveZoomInternal(clamped_effective_zoom);
        true
    }
}
