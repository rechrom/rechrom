use foundation::{gfx, LayoutRatioFromSizeF, LayoutUnit, PhysicalSize};

// cpp: layoutng_style/style/natural_sizing_info.h:15-42
#[derive(Clone)]
pub struct NaturalSizingInfo {
    pub size: gfx::SizeF,
    pub aspect_ratio: gfx::SizeF,
    pub has_width: bool,
    pub has_height: bool,
}

#[allow(non_snake_case)]
impl NaturalSizingInfo {
    // cpp: layoutng_style/style/natural_sizing_info.h:18-20
    pub fn None() -> Self {
        Self {
            size: gfx::SizeF::default(),
            aspect_ratio: gfx::SizeF::default(),
            has_width: false,
            has_height: false,
        }
    }

    // cpp: layoutng_style/style/natural_sizing_info.h:21-24
    pub fn MakeFixed(natural_size: &gfx::SizeF) -> Self {
        Self {
            size: natural_size.clone(),
            aspect_ratio: natural_size.clone(),
            has_width: true,
            has_height: true,
        }
    }

    // cpp: layoutng_style/style/natural_sizing_info.h:25-28
    pub fn MakeSize(natural_size: &gfx::SizeF) -> Self {
        Self {
            size: natural_size.clone(),
            aspect_ratio: gfx::SizeF::default(),
            has_width: true,
            has_height: true,
        }
    }

    // cpp: layoutng_style/style/natural_sizing_info.h:29-32
    pub fn MakeHeight(height: f32) -> Self {
        Self {
            size: gfx::SizeF::new(0.0, height),
            aspect_ratio: gfx::SizeF::default(),
            has_width: false,
            has_height: true,
        }
    }

    // cpp: layoutng_style/style/natural_sizing_info.h:34-36
    pub fn IsNone(&self) -> bool {
        !self.has_width && !self.has_height && self.aspect_ratio.IsEmpty()
    }
}

// cpp: layoutng_style/style/natural_sizing_info.h:40-41
impl Default for NaturalSizingInfo {
    fn default() -> Self {
        Self {
            size: gfx::SizeF::default(),
            aspect_ratio: gfx::SizeF::default(),
            has_width: true,
            has_height: true,
        }
    }
}

// cpp: layoutng_style/style/natural_sizing_info.h:44-59
#[derive(Clone, PartialEq)]
pub struct PhysicalNaturalSizingInfo {
    pub size: PhysicalSize,
    pub aspect_ratio: PhysicalSize,
    pub has_width: bool,
    pub has_height: bool,
}

#[allow(non_snake_case)]
impl PhysicalNaturalSizingInfo {
    // cpp: layoutng_style/style/natural_sizing_info.h:47
    pub fn None() -> Self {
        Self {
            size: PhysicalSize::default(),
            aspect_ratio: PhysicalSize::default(),
            has_width: false,
            has_height: false,
        }
    }

    // cpp: layoutng_style/style/natural_sizing_info.h:48-50
    pub fn MakeFixed(natural_size: &PhysicalSize) -> Self {
        Self {
            size: natural_size.clone(),
            aspect_ratio: natural_size.clone(),
            has_width: true,
            has_height: true,
        }
    }

    // cpp: layoutng_style/style/natural_sizing_info.cc:103-109
    pub fn FromSizingInfo(sizing_info: &NaturalSizingInfo) -> Self {
        Self {
            size: PhysicalSize::FromSizeFRound(&sizing_info.size),
            aspect_ratio: LayoutRatioFromSizeF(sizing_info.aspect_ratio.clone()),
            has_width: sizing_info.has_width,
            has_height: sizing_info.has_height,
        }
    }
}

// cpp: layoutng_style/style/natural_sizing_info.h:55-58
impl Default for PhysicalNaturalSizingInfo {
    fn default() -> Self {
        Self {
            size: PhysicalSize::default(),
            aspect_ratio: PhysicalSize::default(),
            has_width: true,
            has_height: true,
        }
    }
}

// cpp: layoutng_style/style/natural_sizing_info.h:61-64
#[allow(non_snake_case)]
pub fn ResolveWidthForRatioF(height: f32, natural_ratio: &gfx::SizeF) -> f32 {
    height * natural_ratio.width() / natural_ratio.height()
}

// cpp: layoutng_style/style/natural_sizing_info.h:65-68
#[allow(non_snake_case)]
pub fn ResolveWidthForRatioLayout(height: LayoutUnit, natural_ratio: &PhysicalSize) -> LayoutUnit {
    height.MulDiv(natural_ratio.width, natural_ratio.height)
}

// cpp: layoutng_style/style/natural_sizing_info.h:70-73
#[allow(non_snake_case)]
pub fn ResolveHeightForRatioF(width: f32, natural_ratio: &gfx::SizeF) -> f32 {
    width * natural_ratio.height() / natural_ratio.width()
}

// cpp: layoutng_style/style/natural_sizing_info.h:74-77
#[allow(non_snake_case)]
pub fn ResolveHeightForRatioLayout(width: LayoutUnit, natural_ratio: &PhysicalSize) -> LayoutUnit {
    width.MulDiv(natural_ratio.height, natural_ratio.width)
}

// cpp: layoutng_style/style/natural_sizing_info.cc:34-51
trait SizingTraits {
    type Size: Clone;
    type Dimension: Copy + PartialOrd;

    fn size(&self) -> &Self::Size;
    fn aspect_ratio(&self) -> &Self::Size;
    fn has_width(&self) -> bool;
    fn has_height(&self) -> bool;
    fn get_width(size: &Self::Size) -> Self::Dimension;
    fn get_height(size: &Self::Size) -> Self::Dimension;
    fn is_empty(size: &Self::Size) -> bool;
    fn make_size(width: Self::Dimension, height: Self::Dimension) -> Self::Size;
    fn resolve_width(height: Self::Dimension, ratio: &Self::Size) -> Self::Dimension;
    fn resolve_height(width: Self::Dimension, ratio: &Self::Size) -> Self::Dimension;
}

// cpp: layoutng_style/style/natural_sizing_info.cc:37-43
impl SizingTraits for NaturalSizingInfo {
    type Size = gfx::SizeF;
    type Dimension = f32;

    fn size(&self) -> &Self::Size {
        &self.size
    }
    fn aspect_ratio(&self) -> &Self::Size {
        &self.aspect_ratio
    }
    fn has_width(&self) -> bool {
        self.has_width
    }
    fn has_height(&self) -> bool {
        self.has_height
    }
    fn get_width(size: &Self::Size) -> f32 {
        size.width()
    }
    fn get_height(size: &Self::Size) -> f32 {
        size.height()
    }
    fn is_empty(size: &Self::Size) -> bool {
        size.IsEmpty()
    }
    fn make_size(width: f32, height: f32) -> Self::Size {
        gfx::SizeF::new(width, height)
    }
    fn resolve_width(height: f32, ratio: &Self::Size) -> f32 {
        ResolveWidthForRatioF(height, ratio)
    }
    fn resolve_height(width: f32, ratio: &Self::Size) -> f32 {
        ResolveHeightForRatioF(width, ratio)
    }
}

// cpp: layoutng_style/style/natural_sizing_info.cc:45-51
impl SizingTraits for PhysicalNaturalSizingInfo {
    type Size = PhysicalSize;
    type Dimension = LayoutUnit;

    fn size(&self) -> &Self::Size {
        &self.size
    }
    fn aspect_ratio(&self) -> &Self::Size {
        &self.aspect_ratio
    }
    fn has_width(&self) -> bool {
        self.has_width
    }
    fn has_height(&self) -> bool {
        self.has_height
    }
    fn get_width(size: &Self::Size) -> LayoutUnit {
        size.width
    }
    fn get_height(size: &Self::Size) -> LayoutUnit {
        size.height
    }
    fn is_empty(size: &Self::Size) -> bool {
        size.IsEmpty()
    }
    fn make_size(width: LayoutUnit, height: LayoutUnit) -> Self::Size {
        PhysicalSize::new(width, height)
    }
    fn resolve_width(height: LayoutUnit, ratio: &Self::Size) -> LayoutUnit {
        ResolveWidthForRatioLayout(height, ratio)
    }
    fn resolve_height(width: LayoutUnit, ratio: &Self::Size) -> LayoutUnit {
        ResolveHeightForRatioLayout(width, ratio)
    }
}

// cpp: layoutng_style/style/natural_sizing_info.cc:53-99
#[allow(non_snake_case)]
fn ConcreteObjectSizeImpl<T: SizingTraits>(
    sizing_info: &T,
    default_object_size: &T::Size,
) -> T::Size {
    if sizing_info.has_width() && sizing_info.has_height() {
        return sizing_info.size().clone();
    }

    if sizing_info.has_width() {
        let width = T::get_width(sizing_info.size());
        if T::is_empty(sizing_info.aspect_ratio()) {
            return T::make_size(width, T::get_height(default_object_size));
        }
        return T::make_size(width, T::resolve_height(width, sizing_info.aspect_ratio()));
    }

    if sizing_info.has_height() {
        let height = T::get_height(sizing_info.size());
        if T::is_empty(sizing_info.aspect_ratio()) {
            return T::make_size(T::get_width(default_object_size), height);
        }
        return T::make_size(T::resolve_width(height, sizing_info.aspect_ratio()), height);
    }

    if !T::is_empty(sizing_info.aspect_ratio()) {
        let default_width = T::get_width(default_object_size);
        let default_height = T::get_height(default_object_size);
        let solution_width = T::resolve_width(default_height, sizing_info.aspect_ratio());
        if solution_width <= default_width {
            return T::make_size(solution_width, default_height);
        }
        let solution_height = T::resolve_height(default_width, sizing_info.aspect_ratio());
        return T::make_size(default_width, solution_height);
    }
    default_object_size.clone()
}

// cpp: layoutng_style/style/natural_sizing_info.cc:111-114
#[allow(non_snake_case)]
pub fn ConcreteObjectSizeF(
    sizing_info: &NaturalSizingInfo,
    default_object_size: &gfx::SizeF,
) -> gfx::SizeF {
    ConcreteObjectSizeImpl(sizing_info, default_object_size)
}

// cpp: layoutng_style/style/natural_sizing_info.cc:116-119
#[allow(non_snake_case)]
pub fn ConcreteObjectSizePhysical(
    sizing_info: &PhysicalNaturalSizingInfo,
    default_object_size: &PhysicalSize,
) -> PhysicalSize {
    ConcreteObjectSizeImpl(sizing_info, default_object_size)
}
