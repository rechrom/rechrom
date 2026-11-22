use super::filter_operation::{
    BlurFilterOperation, BoxReflectFilterOperation, DropShadowFilterOperation, OperationType,
};
use super::forward::FilterOperation;
use foundation::{gfx, FloatValueForLength, HeapVector, Member, RectF, Visitor, WtfSizeT};

impl foundation::Traceable for FilterOperations {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.operations_);
    }
}

impl foundation::Traceable for FilterOperationsWrapper {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        FilterOperationsWrapper::Trace(self, visitor);
    }
}

// cpp: layoutng_style/style/filter_operations.h:59
pub type FilterOperationVector = HeapVector<Member<FilterOperation>>;

// cpp: layoutng_style/style/filter_operations.h:44-53
// cpp: layoutng_style/style/filter_operations.h:87-92
#[derive(Clone)]
pub struct FilterOperations {
    pub(super) operations_: FilterOperationVector,
}

#[allow(non_snake_case)]
impl FilterOperations {
    // cpp: layoutng_style/style/filter_operations.h:57
    pub fn clear(&mut self) {
        self.operations_.clear();
    }

    // cpp: layoutng_style/style/filter_operations.h:61
    pub fn OperationsMut(&mut self) -> &mut FilterOperationVector {
        &mut self.operations_
    }

    // cpp: layoutng_style/style/filter_operations.h:62
    pub fn Operations(&self) -> &FilterOperationVector {
        &self.operations_
    }

    // cpp: layoutng_style/style/filter_operations.h:64
    pub fn IsEmpty(&self) -> bool {
        self.operations_.size() == 0
    }

    // cpp: layoutng_style/style/filter_operations.h:65
    pub fn size(&self) -> WtfSizeT {
        self.operations_.size()
    }

    // cpp: layoutng_style/style/filter_operations.h:66-68
    pub fn at(&self, index: WtfSizeT) -> *const FilterOperation {
        if index < self.operations_.size() {
            self.operations_.at(index).Get()
        } else {
            std::ptr::null()
        }
    }

    // cpp: layoutng_style/style/filter_operations.h:70
    // cpp: core/style/filter_operations.cc:63-112
    pub fn CanInterpolateWith(&self, other: &Self) -> bool {
        // cpp: core/style/filter_operations.cc:63-82
        self.Operations()
            .iter()
            .chain(other.Operations().iter())
            .all(|operation| {
                FilterOperation::CanInterpolate(unsafe { &*operation.Get() }.GetType())
            })
            && self
                .Operations()
                .iter()
                .zip(other.Operations().iter())
                .all(|(a, b)| unsafe { &*a.Get() }.IsSameType(unsafe { &*b.Get() }))
    }

    // cpp: layoutng_style/style/filter_operations.h:72-75
    // cpp: core/style/filter_operations.cc:90-96
    pub fn MapRect(&self, rect: &RectF) -> RectF {
        self.Operations().iter().fold(*rect, |mapped, operation| {
            unsafe { &*operation.Get() }.MapRect(&mapped)
        })
    }

    // cpp: layoutng_style/style/filter_operations.h:77
    // cpp: core/style/filter_operations.cc:63-112
    pub fn HasFilterThatAffectsOpacity(&self) -> bool {
        self.Operations()
            .iter()
            .any(|operation| unsafe { &*operation.Get() }.AffectsOpacity())
    }

    // cpp: layoutng_style/style/filter_operations.h:78
    // cpp: core/style/filter_operations.cc:63-112
    pub fn HasFilterThatMovesPixels(&self) -> bool {
        self.Operations()
            .iter()
            .any(|operation| unsafe { &*operation.Get() }.MovesPixels())
    }

    // cpp: layoutng_style/style/filter_operations.h:79
    // cpp: core/style/filter_operations.cc:63-112
    pub fn HasReferenceFilter(&self) -> bool {
        self.Operations()
            .iter()
            .any(|operation| unsafe { &*operation.Get() }.GetType() == OperationType::kReference)
    }

    // cpp: layoutng_style/style/filter_operations.h:80
    // cpp: core/style/filter_operations.cc:63-112
    pub fn UsesCurrentColor(&self) -> bool {
        self.Operations()
            .iter()
            .any(|operation| unsafe { &*operation.Get() }.UsesCurrentColor())
    }
}

// cpp: layoutng_style/style/filter_operations.h:55
// cpp: core/style/filter_operations.cc:46-61
impl PartialEq for FilterOperations {
    fn eq(&self, other: &Self) -> bool {
        // cpp: core/style/filter_operations.cc:46-61
        self.Operations().len() == other.Operations().len()
            && self
                .Operations()
                .iter()
                .zip(other.Operations().iter())
                .all(|(a, b)| (unsafe { &*a.Get() }) == (unsafe { &*b.Get() }))
    }
}

// cpp: layoutng_style/style/filter_operations.h:97-111
#[derive(Clone)]
pub struct FilterOperationsWrapper {
    operations_: FilterOperations,
}

#[allow(non_snake_case)]
impl FilterOperationsWrapper {
    // cpp: layoutng_style/style/filter_operations.h:102-103
    pub fn new(operations: &FilterOperations) -> Self {
        Self {
            operations_: operations.clone(),
        }
    }

    // cpp: layoutng_style/style/filter_operations.h:105
    pub fn Operations(&self) -> &FilterOperations {
        &self.operations_
    }

    // cpp: layoutng_style/style/filter_operations.h:107
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.operations_);
    }
}

// cpp: layoutng_style/style/filter_operations.h:101
impl Default for FilterOperationsWrapper {
    fn default() -> Self {
        Self {
            operations_: FilterOperations::new(),
        }
    }
}

fn GaussianBlurMapEffect(std_deviation: gfx::SizeF, rect: &RectF) -> RectF {
    // Source: platform/graphics/filters/fe_gaussian_blur.cc:39-70.
    fn kernel_size(deviation: f32) -> i32 {
        if deviation == 0.0 {
            return 0;
        }
        let approximate_box_width =
            (deviation * (0.75 * (2.0 * std::f32::consts::PI).sqrt()) + 0.5).floor() as i32;
        2.max(approximate_box_width)
    }

    let horizontal = 1.5 * kernel_size(std_deviation.width()) as f32;
    let vertical = 1.5 * kernel_size(std_deviation.height()) as f32;
    let mut result = *rect;
    result.OutsetEdges(gfx::OutsetsF::new(
        vertical, horizontal, vertical, horizontal,
    ));
    result
}

impl FilterOperation {
    // Source: core/style/filter_operation.h:128-129 and
    // core/style/filter_operation.cc:45-50,78-90. ReferenceFilterOperation's
    // source SVG Filter edge is intentionally absent from the immutable style
    // value until resource resolution installs it, so its source fallback is
    // the identity mapping.
    pub fn MapRect(&self, rect: &RectF) -> RectF {
        unsafe {
            match self.GetType() {
                OperationType::kBlur => {
                    let blur = &*(self as *const _ as *const BlurFilterOperation);
                    let deviation = gfx::SizeF::new(
                        FloatValueForLength(blur.StdDeviationXY().X(), 0.0),
                        FloatValueForLength(blur.StdDeviationXY().Y(), 0.0),
                    );
                    GaussianBlurMapEffect(deviation, rect)
                }
                OperationType::kDropShadow => {
                    let shadow =
                        (&*(self as *const _ as *const DropShadowFilterOperation)).Shadow();
                    let mut offset_rect = *rect;
                    offset_rect.Offset(shadow.X(), shadow.Y());
                    let deviation = gfx::SizeF::new(shadow.BlurValue(), shadow.BlurValue());
                    let mut result = GaussianBlurMapEffect(deviation, &offset_rect);
                    result.Union(*rect);
                    result
                }
                OperationType::kBoxReflect => (&*(self as *const _
                    as *const BoxReflectFilterOperation))
                    .Reflection()
                    .MapRect(rect),
                _ => *rect,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::shadow_data::{ShadowData, ShadowStyle};
    use super::*;
    use crate::css::style_color::StyleColor;
    use foundation::Length;

    #[test]
    fn moving_css_filters_map_rect_with_source_kernel_geometry() {
        let rect = RectF::new(gfx::PointF::new(10.0, 20.0), gfx::SizeF::new(30.0, 40.0));
        let blur = BlurFilterOperation::new(&Length::Fixed(2.0));
        assert_eq!(
            blur.MapRect(&rect),
            RectF::new(gfx::PointF::new(4.0, 14.0), gfx::SizeF::new(42.0, 52.0))
        );

        let shadow = DropShadowFilterOperation::new(ShadowData::new(
            gfx::Vector2dF::new(4.0, -2.0),
            2.0,
            0.0,
            ShadowStyle::kNormal,
            StyleColor::default(),
            1.0,
        ));
        assert_eq!(
            shadow.MapRect(&rect),
            RectF::new(gfx::PointF::new(8.0, 12.0), gfx::SizeF::new(42.0, 52.0))
        );
    }
}
