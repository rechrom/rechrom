use foundation::{
    gfx, Length, LengthPoint, LengthSize, Path, Vector, VectorExt, Visitor, WindRule, RULE_NONZERO,
};

// cpp: layoutng_style/style/basic_shapes.h:57-65
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ShapeType {
    kBasicShapeEllipseType,
    kBasicShapePolygonType,
    kBasicShapeCircleType,
    kBasicShapeInsetType,
    kStyleRayType,
    kStylePathType,
    kStyleShapeType,
}

// cpp: layoutng_style/style/basic_shapes.h:52-84
#[allow(non_snake_case)]
pub trait BasicShape {
    // cpp: layoutng_style/style/basic_shapes.h:55
    fn Trace(&self, _visitor: &mut Visitor) {}

    // cpp: layoutng_style/style/basic_shapes.h:67-69
    fn IsSameType(&self, other: &dyn BasicShape) -> bool {
        self.GetType() == other.GetType()
    }

    // cpp: layoutng_style/style/basic_shapes.h:71-73
    fn GetPath(&self, rect: &gfx::RectF, zoom: f32, path_scale: f32) -> Path;

    // cpp: layoutng_style/style/basic_shapes.h:78
    fn GetType(&self) -> ShapeType;

    // cpp: layoutng_style/style/basic_shapes.h:83
    fn IsEqualAssumingSameType(&self, other: &dyn BasicShape) -> bool;
}

// cpp: layoutng_style/style/basic_shapes.h:74-76
impl PartialEq for dyn BasicShape + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.IsSameType(other) && self.IsEqualAssumingSameType(other)
    }
}

// cpp: layoutng_style/style/basic_shapes.h:90-96
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadiusType {
    kValue,
    kClosestSide,
    kFarthestSide,
    kClosestCorner,
    kFarthestCorner,
}

// cpp: layoutng_style/style/basic_shapes.h:86-112
#[derive(Clone)]
pub struct BasicShapeRadius {
    value_: Length,
    type_: RadiusType,
}

// cpp: layoutng_style/style/basic_shapes.h:97
impl Default for BasicShapeRadius {
    fn default() -> Self {
        Self {
            value_: Length::default(),
            type_: RadiusType::kClosestSide,
        }
    }
}

#[allow(non_snake_case)]
impl BasicShapeRadius {
    // cpp: layoutng_style/style/basic_shapes.h:98-99
    pub fn from_value(value: &Length) -> Self {
        Self {
            value_: value.clone(),
            type_: RadiusType::kValue,
        }
    }
    pub fn from_type(radius_type: RadiusType) -> Self {
        Self {
            value_: Length::default(),
            type_: radius_type,
        }
    }

    // cpp: layoutng_style/style/basic_shapes.h:106-107
    pub fn Value(&self) -> &Length {
        &self.value_
    }
    pub fn GetType(&self) -> RadiusType {
        self.type_
    }
}

// cpp: layoutng_style/style/basic_shapes.h:102-104
impl PartialEq for BasicShapeRadius {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_ && self.value_ == other.value_
    }
}

// cpp: layoutng_style/style/basic_shapes.h:114-133
#[derive(Clone)]
pub struct CenterAndRadiiFields {
    center_: LengthPoint,
    is_center_explicitly_set_: bool,
}

impl Default for CenterAndRadiiFields {
    fn default() -> Self {
        Self {
            center_: LengthPoint::default(),
            is_center_explicitly_set_: true,
        }
    }
}

#[allow(non_snake_case)]
pub trait BasicShapeWithCenterAndRadii: BasicShape {
    fn center_fields(&self) -> &CenterAndRadiiFields;
    fn center_fields_mut(&mut self) -> &mut CenterAndRadiiFields;

    // cpp: layoutng_style/style/basic_shapes.h:116-122
    fn SetHasExplicitCenter(&mut self, value: bool) {
        self.center_fields_mut().is_center_explicitly_set_ = value;
    }
    fn HasExplicitCenter(&self) -> bool {
        self.center_fields().is_center_explicitly_set_
    }
    fn SetCenter(&mut self, center: LengthPoint) {
        self.center_fields_mut().center_ = center;
    }
    fn Center(&self) -> &LengthPoint {
        &self.center_fields().center_
    }

    // cpp: layoutng_style/style/basic_shapes.h:124-126
    fn GetPathFromCenter(&self, center: &gfx::PointF, rect: &gfx::RectF, path_scale: f32) -> Path;
}

// cpp: layoutng_style/style/basic_shapes.h:135-142
#[allow(non_snake_case)]
pub fn AllowBasicShapeWithCenterAndRadii(value: &dyn BasicShape) -> bool {
    let type_ = value.GetType();
    type_ == ShapeType::kBasicShapeCircleType || type_ == ShapeType::kBasicShapeEllipseType
}

// cpp: layoutng_style/style/basic_shapes.h:144-166
#[derive(Default)]
pub struct BasicShapeCircle {
    center_data_: CenterAndRadiiFields,
    radius_: BasicShapeRadius,
}

#[allow(non_snake_case)]
impl BasicShapeCircle {
    // cpp: layoutng_style/style/basic_shapes.h:148
    pub fn Radius(&self) -> &BasicShapeRadius {
        &self.radius_
    }
    // cpp: layoutng_style/style/basic_shapes.h:150-152
    // The radius calculation has no supplied definition.
    pub fn FloatValueForRadiusInBox(&self, center: &gfx::PointF, box_size: &gfx::SizeF) -> f32 {
        unsafe { BasicShapeCircleFloatValueForRadiusInBox(self, center, box_size) }
    }
    pub fn SetRadius(&mut self, radius: BasicShapeRadius) {
        self.radius_ = radius;
    }

    // cpp: layoutng_style/style/basic_shapes.h:168-173
    pub fn AllowFrom(value: &dyn BasicShape) -> bool {
        value.GetType() == ShapeType::kBasicShapeCircleType
    }
}

#[allow(non_snake_case)]
impl BasicShape for BasicShapeCircle {
    // cpp: layoutng_style/style/basic_shapes.h:154
    fn GetPath(&self, rect: &gfx::RectF, zoom: f32, scale: f32) -> Path {
        unsafe { BasicShapeCircleGetPath(self, rect, zoom, scale) }
    }
    // cpp: layoutng_style/style/basic_shapes.h:159
    fn GetType(&self) -> ShapeType {
        ShapeType::kBasicShapeCircleType
    }
    // cpp: layoutng_style/style/basic_shapes.h:162
    fn IsEqualAssumingSameType(&self, other: &dyn BasicShape) -> bool {
        unsafe { BasicShapeCircleIsEqualAssumingSameType(self, other) }
    }
}

#[allow(non_snake_case)]
impl BasicShapeWithCenterAndRadii for BasicShapeCircle {
    fn center_fields(&self) -> &CenterAndRadiiFields {
        &self.center_data_
    }
    fn center_fields_mut(&mut self) -> &mut CenterAndRadiiFields {
        &mut self.center_data_
    }
    // cpp: layoutng_style/style/basic_shapes.h:155-157
    fn GetPathFromCenter(&self, center: &gfx::PointF, rect: &gfx::RectF, scale: f32) -> Path {
        unsafe { BasicShapeCircleGetPathFromCenter(self, center, rect, scale) }
    }
}

// cpp: layoutng_style/style/basic_shapes.h:175-204
#[derive(Default)]
pub struct BasicShapeEllipse {
    center_data_: CenterAndRadiiFields,
    radius_x_: BasicShapeRadius,
    radius_y_: BasicShapeRadius,
}

#[allow(non_snake_case)]
impl BasicShapeEllipse {
    // cpp: layoutng_style/style/basic_shapes.h:179-180
    pub fn RadiusX(&self) -> &BasicShapeRadius {
        &self.radius_x_
    }
    pub fn RadiusY(&self) -> &BasicShapeRadius {
        &self.radius_y_
    }

    // cpp: layoutng_style/style/basic_shapes.h:181-182
    // No definition is supplied in this package.
    pub fn ResolveRadii(&self, center: &gfx::PointF, box_size: &gfx::SizeF) -> gfx::SizeF {
        unsafe { BasicShapeEllipseResolveRadii(self, center, box_size) }
    }

    // cpp: layoutng_style/style/basic_shapes.h:184-185
    pub fn SetRadiusX(&mut self, radius_x: BasicShapeRadius) {
        self.radius_x_ = radius_x;
    }
    pub fn SetRadiusY(&mut self, radius_y: BasicShapeRadius) {
        self.radius_y_ = radius_y;
    }

    // cpp: layoutng_style/style/basic_shapes.h:198-200
    // No definition is supplied in this package.
    fn FloatValueForRadiusInBox(
        &self,
        radius: &BasicShapeRadius,
        center: f32,
        box_width_or_height: f32,
    ) -> f32 {
        unsafe {
            BasicShapeEllipseFloatValueForRadiusInBox(self, radius, center, box_width_or_height)
        }
    }

    // cpp: layoutng_style/style/basic_shapes.h:206-211
    pub fn AllowFrom(value: &dyn BasicShape) -> bool {
        value.GetType() == ShapeType::kBasicShapeEllipseType
    }
}

#[allow(non_snake_case)]
impl BasicShape for BasicShapeEllipse {
    // cpp: layoutng_style/style/basic_shapes.h:187
    fn GetPath(&self, rect: &gfx::RectF, zoom: f32, scale: f32) -> Path {
        unsafe { BasicShapeEllipseGetPath(self, rect, zoom, scale) }
    }
    // cpp: layoutng_style/style/basic_shapes.h:192
    fn GetType(&self) -> ShapeType {
        ShapeType::kBasicShapeEllipseType
    }
    // cpp: layoutng_style/style/basic_shapes.h:195
    fn IsEqualAssumingSameType(&self, other: &dyn BasicShape) -> bool {
        unsafe { BasicShapeEllipseIsEqualAssumingSameType(self, other) }
    }
}

#[allow(non_snake_case)]
impl BasicShapeWithCenterAndRadii for BasicShapeEllipse {
    fn center_fields(&self) -> &CenterAndRadiiFields {
        &self.center_data_
    }
    fn center_fields_mut(&mut self) -> &mut CenterAndRadiiFields {
        &mut self.center_data_
    }
    // cpp: layoutng_style/style/basic_shapes.h:188-190
    fn GetPathFromCenter(&self, center: &gfx::PointF, rect: &gfx::RectF, scale: f32) -> Path {
        unsafe { BasicShapeEllipseGetPathFromCenter(self, center, rect, scale) }
    }
}

// cpp: layoutng_style/style/basic_shapes.h:213-244
pub struct BasicShapePolygon {
    wind_rule_: WindRule,
    rounding_radius_: Length,
    values_: Vector<Length>,
}

// cpp: layoutng_style/style/basic_shapes.h:215-216
impl Default for BasicShapePolygon {
    fn default() -> Self {
        Self {
            wind_rule_: RULE_NONZERO,
            rounding_radius_: Length::Fixed(0),
            values_: Vector::default(),
        }
    }
}

#[allow(non_snake_case)]
impl BasicShapePolygon {
    // cpp: layoutng_style/style/basic_shapes.h:218-220
    pub fn Values(&self) -> &Vector<Length> {
        &self.values_
    }
    pub fn HasRoundingRadius(&self) -> bool {
        !self.rounding_radius_.IsZero()
    }
    pub fn RoundingRadius(&self) -> &Length {
        &self.rounding_radius_
    }

    // cpp: layoutng_style/style/basic_shapes.h:222-229
    pub fn SetWindRule(&mut self, rule: WindRule) {
        self.wind_rule_ = rule;
    }
    pub fn SetRoundingRadius(&mut self, radius: &Length) {
        self.rounding_radius_ = radius.clone();
    }
    pub fn AppendPoint(&mut self, x: &Length, y: &Length) {
        self.values_.push_back(x.clone());
        self.values_.push_back(y.clone());
    }

    // cpp: layoutng_style/style/basic_shapes.h:233
    pub fn GetWindRule(&self) -> WindRule {
        self.wind_rule_
    }

    // cpp: layoutng_style/style/basic_shapes.h:246-251
    pub fn AllowFrom(value: &dyn BasicShape) -> bool {
        value.GetType() == ShapeType::kBasicShapePolygonType
    }
}

#[allow(non_snake_case)]
impl BasicShape for BasicShapePolygon {
    // cpp: layoutng_style/style/basic_shapes.h:231
    fn GetPath(&self, rect: &gfx::RectF, zoom: f32, scale: f32) -> Path {
        let _ = (rect, zoom, scale);
        unimplemented!("BasicShapePolygon::GetPath requires the owning vector Path package")
    }
    // cpp: layoutng_style/style/basic_shapes.h:235
    fn GetType(&self) -> ShapeType {
        ShapeType::kBasicShapePolygonType
    }
    // cpp: layoutng_style/style/basic_shapes.h:238
    fn IsEqualAssumingSameType(&self, other: &dyn BasicShape) -> bool {
        // core/style/basic_shapes.cc:322-325.
        debug_assert!(self.IsSameType(other));
        let other = unsafe { &*(other as *const dyn BasicShape as *const Self) };
        self.wind_rule_ == other.wind_rule_
            && self.rounding_radius_ == other.rounding_radius_
            && self.values_ == other.values_
    }
}

impl foundation::Traceable for BasicShapePolygon {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}

// cpp: layoutng_style/style/basic_shapes.h:253-299
#[derive(Default)]
pub struct BasicShapeInset {
    right_: Length,
    top_: Length,
    bottom_: Length,
    left_: Length,
    top_left_radius_: LengthSize,
    top_right_radius_: LengthSize,
    bottom_right_radius_: LengthSize,
    bottom_left_radius_: LengthSize,
}

#[allow(non_snake_case)]
impl BasicShapeInset {
    // cpp: layoutng_style/style/basic_shapes.h:260-268
    pub fn Top(&self) -> &Length {
        &self.top_
    }
    pub fn Right(&self) -> &Length {
        &self.right_
    }
    pub fn Bottom(&self) -> &Length {
        &self.bottom_
    }
    pub fn Left(&self) -> &Length {
        &self.left_
    }
    pub fn TopLeftRadius(&self) -> &LengthSize {
        &self.top_left_radius_
    }
    pub fn TopRightRadius(&self) -> &LengthSize {
        &self.top_right_radius_
    }
    pub fn BottomRightRadius(&self) -> &LengthSize {
        &self.bottom_right_radius_
    }
    pub fn BottomLeftRadius(&self) -> &LengthSize {
        &self.bottom_left_radius_
    }

    // cpp: layoutng_style/style/basic_shapes.h:270-284
    pub fn SetTop(&mut self, top: &Length) {
        self.top_ = top.clone();
    }
    pub fn SetRight(&mut self, right: &Length) {
        self.right_ = right.clone();
    }
    pub fn SetBottom(&mut self, bottom: &Length) {
        self.bottom_ = bottom.clone();
    }
    pub fn SetLeft(&mut self, left: &Length) {
        self.left_ = left.clone();
    }
    pub fn SetTopLeftRadius(&mut self, radius: &LengthSize) {
        self.top_left_radius_ = radius.clone();
    }
    pub fn SetTopRightRadius(&mut self, radius: &LengthSize) {
        self.top_right_radius_ = radius.clone();
    }
    pub fn SetBottomRightRadius(&mut self, radius: &LengthSize) {
        self.bottom_right_radius_ = radius.clone();
    }
    pub fn SetBottomLeftRadius(&mut self, radius: &LengthSize) {
        self.bottom_left_radius_ = radius.clone();
    }

    // cpp: layoutng_style/style/basic_shapes.h:301-306
    pub fn AllowFrom(value: &dyn BasicShape) -> bool {
        value.GetType() == ShapeType::kBasicShapeInsetType
    }
}

#[allow(non_snake_case)]
impl BasicShape for BasicShapeInset {
    // cpp: layoutng_style/style/basic_shapes.h:257
    fn GetType(&self) -> ShapeType {
        ShapeType::kBasicShapeInsetType
    }
    // cpp: layoutng_style/style/basic_shapes.h:258
    fn GetPath(&self, rect: &gfx::RectF, zoom: f32, scale: f32) -> Path {
        unsafe { BasicShapeInsetGetPath(self, rect, zoom, scale) }
    }
    // cpp: layoutng_style/style/basic_shapes.h:288
    fn IsEqualAssumingSameType(&self, other: &dyn BasicShape) -> bool {
        unsafe { BasicShapeInsetIsEqualAssumingSameType(self, other) }
    }
}

// The source declares these operations but supplies no definitions in the package.
unsafe extern "Rust" {
    fn BasicShapeCircleFloatValueForRadiusInBox(
        value: &BasicShapeCircle,
        center: &gfx::PointF,
        size: &gfx::SizeF,
    ) -> f32;
    fn BasicShapeCircleGetPath(
        value: &BasicShapeCircle,
        rect: &gfx::RectF,
        zoom: f32,
        scale: f32,
    ) -> Path;
    fn BasicShapeCircleGetPathFromCenter(
        value: &BasicShapeCircle,
        center: &gfx::PointF,
        rect: &gfx::RectF,
        scale: f32,
    ) -> Path;
    fn BasicShapeCircleIsEqualAssumingSameType(
        value: &BasicShapeCircle,
        other: &dyn BasicShape,
    ) -> bool;
    fn BasicShapeEllipseResolveRadii(
        value: &BasicShapeEllipse,
        center: &gfx::PointF,
        size: &gfx::SizeF,
    ) -> gfx::SizeF;
    fn BasicShapeEllipseFloatValueForRadiusInBox(
        value: &BasicShapeEllipse,
        radius: &BasicShapeRadius,
        center: f32,
        width_or_height: f32,
    ) -> f32;
    fn BasicShapeEllipseGetPath(
        value: &BasicShapeEllipse,
        rect: &gfx::RectF,
        zoom: f32,
        scale: f32,
    ) -> Path;
    fn BasicShapeEllipseGetPathFromCenter(
        value: &BasicShapeEllipse,
        center: &gfx::PointF,
        rect: &gfx::RectF,
        scale: f32,
    ) -> Path;
    fn BasicShapeEllipseIsEqualAssumingSameType(
        value: &BasicShapeEllipse,
        other: &dyn BasicShape,
    ) -> bool;
    fn BasicShapePolygonGetPath(
        value: &BasicShapePolygon,
        rect: &gfx::RectF,
        zoom: f32,
        scale: f32,
    ) -> Path;
    fn BasicShapePolygonIsEqualAssumingSameType(
        value: &BasicShapePolygon,
        other: &dyn BasicShape,
    ) -> bool;
    fn BasicShapeInsetGetPath(
        value: &BasicShapeInset,
        rect: &gfx::RectF,
        zoom: f32,
        scale: f32,
    ) -> Path;
    fn BasicShapeInsetIsEqualAssumingSameType(
        value: &BasicShapeInset,
        other: &dyn BasicShape,
    ) -> bool;
}
