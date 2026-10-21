use foundation::{HashInt, HashInts, Length, LengthType};

// cpp: layoutng_style/style/grid_track_size.h:41-45
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum GridTrackSizeType {
    kLengthTrackSizing,
    kMinMaxTrackSizing,
    kFitContentTrackSizing,
}

// cpp: layoutng_style/style/grid_track_size.h:47-61
#[allow(non_snake_case)]
fn IsTrackSizeIntrinsic(length: &Length, track_size_type: GridTrackSizeType) -> bool {
    match track_size_type {
        GridTrackSizeType::kLengthTrackSizing | GridTrackSizeType::kMinMaxTrackSizing => {
            let type_ = length.GetType();
            type_ == LengthType::kAuto
                || type_ == LengthType::kMinContent
                || type_ == LengthType::kMaxContent
        }
        GridTrackSizeType::kFitContentTrackSizing => true,
    }
}

// cpp: layoutng_style/style/grid_track_size.h:63-271
#[derive(Clone)]
pub struct GridTrackSize {
    min_track_breadth_: Length,
    max_track_breadth_: Length,
    fit_content_track_breadth_: Length,
    type_: GridTrackSizeType,
    flags_: u16,
}

#[allow(non_snake_case)]
impl GridTrackSize {
    const MIN_AUTO: u16 = 1 << 0;
    const MAX_AUTO: u16 = 1 << 1;
    const MIN_FIXED: u16 = 1 << 2;
    const MAX_FIXED: u16 = 1 << 3;
    const MIN_FLEX: u16 = 1 << 4;
    const MAX_FLEX: u16 = 1 << 5;
    const MIN_INTRINSIC: u16 = 1 << 6;
    const MAX_INTRINSIC: u16 = 1 << 7;
    const MIN_MAX_CONTENT: u16 = 1 << 8;
    const MAX_MAX_CONTENT: u16 = 1 << 9;
    const MIN_MIN_CONTENT: u16 = 1 << 10;
    const MAX_MIN_CONTENT: u16 = 1 << 11;
    const DEFINITION_INTRINSIC: u16 = 1 << 12;
    const DELETED: u16 = 1 << 13;
    const EMPTY: u16 = 1 << 14;

    fn flag(&self, mask: u16) -> bool {
        self.flags_ & mask != 0
    }
    fn set_flag(&mut self, mask: u16, value: bool) {
        if value {
            self.flags_ |= mask;
        } else {
            self.flags_ &= !mask;
        }
    }

    // cpp: layoutng_style/style/grid_track_size.h:80-98
    pub fn new(length: &Length, track_size_type: GridTrackSizeType) -> Self {
        let is_fit_content = track_size_type == GridTrackSizeType::kFitContentTrackSizing;
        let min_track_breadth = if is_fit_content {
            Length::Auto().clone()
        } else {
            length.clone()
        };
        let max_track_breadth = if is_fit_content {
            Length::Auto().clone()
        } else {
            length.clone()
        };
        let fit_content_track_breadth = if is_fit_content {
            length.clone()
        } else {
            Length::Fixed(0)
        };
        let definition_intrinsic = IsTrackSizeIntrinsic(length, track_size_type);
        let mut size = Self {
            min_track_breadth_: min_track_breadth,
            max_track_breadth_: max_track_breadth,
            fit_content_track_breadth_: fit_content_track_breadth,
            type_: track_size_type,
            flags_: if definition_intrinsic {
                Self::DEFINITION_INTRINSIC
            } else {
                0
            },
        };
        debug_assert!(
            track_size_type == GridTrackSizeType::kLengthTrackSizing
                || track_size_type == GridTrackSizeType::kFitContentTrackSizing
        );
        debug_assert!(track_size_type == GridTrackSizeType::kLengthTrackSizing || !length.IsFlex());
        size.CacheMinMaxTrackBreadthTypes();
        size
    }

    pub fn from_length(length: &Length) -> Self {
        Self::new(length, GridTrackSizeType::kLengthTrackSizing)
    }

    // cpp: layoutng_style/style/grid_track_size.h:100-110
    pub fn new_minmax(min_track_breadth: &Length, max_track_breadth: &Length) -> Self {
        let min_breadth = min_track_breadth.clone();
        let max_breadth = max_track_breadth.clone();
        let fit_content_breadth = Length::Fixed(0);
        let definition_intrinsic =
            IsTrackSizeIntrinsic(min_track_breadth, GridTrackSizeType::kMinMaxTrackSizing)
                && IsTrackSizeIntrinsic(max_track_breadth, GridTrackSizeType::kMinMaxTrackSizing);
        let mut size = Self {
            min_track_breadth_: min_breadth,
            max_track_breadth_: max_breadth,
            fit_content_track_breadth_: fit_content_breadth,
            type_: GridTrackSizeType::kMinMaxTrackSizing,
            flags_: if definition_intrinsic {
                Self::DEFINITION_INTRINSIC
            } else {
                0
            },
        };
        size.CacheMinMaxTrackBreadthTypes();
        size
    }

    // cpp: layoutng_style/style/grid_track_size.h:112-122
    pub fn DeletedTrackSize() -> Self {
        let mut size = Self::from_length(Length::Auto());
        size.set_flag(Self::DELETED, true);
        size
    }
    pub fn EmptyTrackSize() -> Self {
        let mut size = Self::from_length(Length::Auto());
        size.set_flag(Self::EMPTY, true);
        size
    }

    // cpp: layoutng_style/style/grid_track_size.h:124-125
    pub fn IsDeletedValue(&self) -> bool {
        self.flag(Self::DELETED)
    }
    pub fn IsEmptyValue(&self) -> bool {
        self.flag(Self::EMPTY)
    }

    // cpp: layoutng_style/style/grid_track_size.h:127-130
    pub fn FitContentTrackBreadth(&self) -> &Length {
        debug_assert_eq!(self.type_, GridTrackSizeType::kFitContentTrackSizing);
        &self.fit_content_track_breadth_
    }

    // cpp: layoutng_style/style/grid_track_size.h:132-133
    pub fn MinTrackBreadth(&self) -> &Length {
        &self.min_track_breadth_
    }
    pub fn MaxTrackBreadth(&self) -> &Length {
        &self.max_track_breadth_
    }

    // cpp: layoutng_style/style/grid_track_size.h:135-148
    pub fn MinOrFitContentTrackBreadth(&self) -> &Length {
        if self.IsFitContent() {
            return &self.fit_content_track_breadth_;
        }
        &self.min_track_breadth_
    }
    pub fn MaxOrFitContentTrackBreadth(&self) -> &Length {
        if self.IsFitContent() {
            return &self.fit_content_track_breadth_;
        }
        &self.max_track_breadth_
    }

    // cpp: layoutng_style/style/grid_track_size.h:150-164
    pub fn GetType(&self) -> GridTrackSizeType {
        self.type_
    }
    pub fn IsContentSized(&self) -> bool {
        self.min_track_breadth_.HasAutoOrContentOrIntrinsic()
            || self.max_track_breadth_.HasAutoOrContentOrIntrinsic()
    }
    pub fn IsFitContent(&self) -> bool {
        self.type_ == GridTrackSizeType::kFitContentTrackSizing
    }
    pub fn HasPercentage(&self) -> bool {
        if self.IsFitContent() {
            return self.FitContentTrackBreadth().MayHavePercentDependence();
        }
        self.min_track_breadth_.MayHavePercentDependence()
            || self.max_track_breadth_.MayHavePercentDependence()
    }

    // cpp: layoutng_style/style/grid_track_size.h:174-195
    pub fn CacheMinMaxTrackBreadthTypes(&mut self) {
        self.set_flag(Self::MIN_AUTO, self.min_track_breadth_.IsAuto());
        self.set_flag(
            Self::MIN_FIXED,
            self.min_track_breadth_.HasOnlyFixedAndPercent(),
        );
        self.set_flag(Self::MIN_FLEX, self.min_track_breadth_.IsFlex());
        self.set_flag(
            Self::MIN_MAX_CONTENT,
            self.min_track_breadth_.IsMaxContent(),
        );
        self.set_flag(
            Self::MIN_MIN_CONTENT,
            self.min_track_breadth_.IsMinContent(),
        );

        self.set_flag(Self::MAX_AUTO, self.max_track_breadth_.IsAuto());
        self.set_flag(
            Self::MAX_FIXED,
            self.max_track_breadth_.HasOnlyFixedAndPercent(),
        );
        self.set_flag(Self::MAX_FLEX, self.max_track_breadth_.IsFlex());
        self.set_flag(
            Self::MAX_MAX_CONTENT,
            self.max_track_breadth_.IsMaxContent(),
        );
        self.set_flag(
            Self::MAX_MIN_CONTENT,
            self.max_track_breadth_.IsMinContent(),
        );

        self.set_flag(
            Self::MIN_INTRINSIC,
            self.flag(Self::MIN_MAX_CONTENT)
                || self.flag(Self::MIN_MIN_CONTENT)
                || self.flag(Self::MIN_AUTO)
                || self.IsFitContent(),
        );
        self.set_flag(
            Self::MAX_INTRINSIC,
            self.flag(Self::MAX_MAX_CONTENT)
                || self.flag(Self::MAX_MIN_CONTENT)
                || self.flag(Self::MAX_AUTO)
                || self.IsFitContent(),
        );
    }

    // cpp: layoutng_style/style/grid_track_size.h:197-202
    pub fn HasIntrinsicMinTrackBreadth(&self) -> bool {
        self.flag(Self::MIN_INTRINSIC)
    }
    pub fn HasIntrinsicMaxTrackBreadth(&self) -> bool {
        self.flag(Self::MAX_INTRINSIC)
    }

    // cpp: layoutng_style/style/grid_track_size.h:203-206
    pub fn HasMinOrMaxContentMinTrackBreadth(&self) -> bool {
        self.flag(Self::MIN_MAX_CONTENT) || self.flag(Self::MIN_MIN_CONTENT)
    }

    // cpp: layoutng_style/style/grid_track_size.h:207-214
    pub fn HasAutoMaxTrackBreadth(&self) -> bool {
        self.flag(Self::MAX_AUTO)
    }
    pub fn HasAutoMinTrackBreadth(&self) -> bool {
        self.flag(Self::MIN_AUTO)
    }
    pub fn HasMaxContentMinTrackBreadth(&self) -> bool {
        self.flag(Self::MIN_MAX_CONTENT)
    }
    pub fn HasMinContentMinTrackBreadth(&self) -> bool {
        self.flag(Self::MIN_MIN_CONTENT)
    }

    // cpp: layoutng_style/style/grid_track_size.h:215-227
    pub fn HasMinOrMaxContentMaxTrackBreadth(&self) -> bool {
        self.flag(Self::MAX_MAX_CONTENT) || self.flag(Self::MAX_MIN_CONTENT)
    }
    pub fn HasMaxContentMaxTrackBreadth(&self) -> bool {
        self.flag(Self::MAX_MAX_CONTENT)
    }
    pub fn HasMaxContentOrAutoMaxTrackBreadth(&self) -> bool {
        self.flag(Self::MAX_MAX_CONTENT) || self.flag(Self::MAX_AUTO)
    }
    pub fn HasMinContentMaxTrackBreadth(&self) -> bool {
        self.flag(Self::MAX_MIN_CONTENT)
    }

    // cpp: layoutng_style/style/grid_track_size.h:228-235
    pub fn HasMaxContentMinTrackBreadthAndMaxContentMaxTrackBreadth(&self) -> bool {
        self.flag(Self::MIN_MAX_CONTENT) && self.flag(Self::MAX_MAX_CONTENT)
    }
    pub fn HasAutoOrMinContentMinTrackBreadthAndIntrinsicMaxTrackBreadth(&self) -> bool {
        (self.flag(Self::MIN_MIN_CONTENT) || self.flag(Self::MIN_AUTO))
            && self.flag(Self::MAX_INTRINSIC)
    }

    // cpp: layoutng_style/style/grid_track_size.h:236-239
    pub fn HasFixedMinTrackBreadth(&self) -> bool {
        self.flag(Self::MIN_FIXED)
    }
    pub fn HasFixedMaxTrackBreadth(&self) -> bool {
        self.flag(Self::MAX_FIXED)
    }
    pub fn HasFlexMinTrackBreadth(&self) -> bool {
        self.flag(Self::MIN_FLEX)
    }
    pub fn HasFlexMaxTrackBreadth(&self) -> bool {
        self.flag(Self::MAX_FLEX)
    }

    // cpp: layoutng_style/style/grid_track_size.h:241-244
    pub fn IsDefinite(&self) -> bool {
        self.flag(Self::MIN_FIXED)
            && self.flag(Self::MAX_FIXED)
            && self.min_track_breadth_ == self.max_track_breadth_
    }

    // cpp: layoutng_style/style/grid_track_size.h:246-248
    pub fn IsTrackDefinitionIntrinsic(&self) -> bool {
        self.flag(Self::DEFINITION_INTRINSIC)
    }
}

// cpp: layoutng_style/style/grid_track_size.h:166-172
impl PartialEq for GridTrackSize {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_
            && self.min_track_breadth_ == other.min_track_breadth_
            && self.max_track_breadth_ == other.max_track_breadth_
            && self.fit_content_track_breadth_ == other.fit_content_track_breadth_
            && self.IsDeletedValue() == other.IsDeletedValue()
            && self.IsEmptyValue() == other.IsEmptyValue()
    }
}

impl Eq for GridTrackSize {}

// cpp: layoutng_style/style/grid_track_size.h:273-289
impl std::hash::Hash for GridTrackSize {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(GridTrackSizeHashTraits::GetHash(self));
    }
}

// cpp: layoutng_style/style/grid_track_size.h:273-298
/// Local adapter for the C++ foundation HashTraits specialization.
pub struct GridTrackSizeHashTraits;

// cpp: layoutng_style/style/grid_track_size.h:291-297
impl foundation::hash_containers::HashKeyTraits<GridTrackSize> for GridTrackSizeHashTraits {
    fn is_valid_key(key: &GridTrackSize) -> bool {
        !key.IsEmptyValue() && !key.IsDeletedValue()
    }
}

#[allow(non_snake_case, non_upper_case_globals)]
impl GridTrackSizeHashTraits {
    // cpp: layoutng_style/style/grid_track_size.h:277-289
    pub fn GetHash(key: &GridTrackSize) -> u32 {
        let type_hash = HashInt(key.GetType() as u32);
        let min_breadth_hash = key.MinTrackBreadth().GetHash();
        let max_breadth_hash = key.MaxTrackBreadth().GetHash();
        let fit_content_hash = if key.IsFitContent() {
            key.FitContentTrackBreadth().GetHash()
        } else {
            0
        };
        let intrinsic_hash = HashInt(key.IsTrackDefinitionIntrinsic() as u32);
        HashInts(
            HashInts(type_hash, min_breadth_hash),
            HashInts(HashInts(max_breadth_hash, fit_content_hash), intrinsic_hash),
        )
    }

    // cpp: layoutng_style/style/grid_track_size.h:291
    pub const kEmptyValueIsZero: bool = false;

    // cpp: layoutng_style/style/grid_track_size.h:293-297
    pub fn EmptyValue() -> GridTrackSize {
        GridTrackSize::EmptyTrackSize()
    }
    pub fn DeletedValue() -> GridTrackSize {
        GridTrackSize::DeletedTrackSize()
    }
}
