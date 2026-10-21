// Shader owning fields, copies and transfers retain current canonical Arc objects;
// precise native owning operations are marked adapted in coverage, never sk_sp-complete.
use crate::include::core::SkClipOp::SkClipOp;
use crate::src::core::SkPath::SkPath;
use crate::src::core::SkRRect::SkRRect;
use crate::src::core::SkRect::SkRect;
use crate::src::shaders::SkShader::SkShader;
use std::sync::Arc;

// Source: src/core/SkClipStack.h:35-44
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BoundsType {
    kNormal_BoundsType,
    kInsideOut_BoundsType,
}
// Source: src/core/SkClipStack.h:53-67
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DeviceSpaceType {
    kEmpty,
    kRect,
    kRRect,
    kPath,
    kShader,
}
pub const TYPE_COUNT: usize = DeviceSpaceType::kShader as usize + 1;
// Source: src/core/SkClipStack.h:69-72
impl Element {
    pub fn new() -> Self {
        let mut element = Self::storage();
        element.init_common(0, SkClipOp::kIntersect, false);
        element.set_empty();
        element
    }
}
// Source: src/core/SkClipStack.h:76-78
impl Element {
    pub fn from_rect(
        rect: &SkRect,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        op: SkClipOp,
        do_aa: bool,
    ) -> Self {
        let mut element = Self::storage();
        element.init_rect(0, rect, matrix, op, do_aa);
        element
    }
}
// Source: src/core/SkClipStack.h:80-82
impl Element {
    pub fn from_rrect(
        rrect: &SkRRect,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        op: SkClipOp,
        do_aa: bool,
    ) -> Self {
        let mut element = Self::storage();
        element.init_rrect(0, rrect, matrix, op, do_aa);
        element
    }
}
// Source: src/core/SkClipStack.h:84-86
impl Element {
    pub fn from_path(
        path: &SkPath,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        op: SkClipOp,
        do_aa: bool,
    ) -> Self {
        let mut element = Self::storage();
        element.init_path(0, path, matrix, op, do_aa);
        element
    }
}
// Source: src/core/SkClipStack.h:88-88
impl Element {
    pub fn from_shader(shader: Arc<dyn SkShader>) -> Self {
        let mut element = Self::storage();
        element.init_shader(0, shader);
        element
    }
}
// Source: src/core/SkClipStack.h:90-92
impl Element {
    pub fn from_replace_rect(rect: &SkRect, do_aa: bool) -> Self {
        let mut element = Self::storage();
        element.init_replace_rect(0, rect, do_aa);
        element
    }
}
// Source: src/core/SkClipStack.h:100-136
impl Element {
    pub fn get_device_space_type(&self) -> DeviceSpaceType {
        self.device_space_type
    }
    pub fn get_save_count(&self) -> i32 {
        self.save_count
    }
    pub fn get_device_space_path(&self) -> &SkPath {
        crate::SkASSERT!(@source b"src/core/SkClipStack.h\0".as_ptr(), 107; @condition b"DeviceSpaceType::kPath == fDeviceSpaceType\0".as_ptr(); (self.device_space_type) == (DeviceSpaceType::kPath));
        self.device_space_path.as_ref().unwrap()
    }
    pub fn get_device_space_rrect(&self) -> &SkRRect {
        crate::SkASSERT!(@source b"src/core/SkClipStack.h\0".as_ptr(), 113; @condition b"DeviceSpaceType::kRRect == fDeviceSpaceType\0".as_ptr(); (self.device_space_type) == (DeviceSpaceType::kRRect));
        &self.device_space_rrect
    }
    pub fn get_device_space_rect(&self) -> &SkRect {
        crate::SkASSERT!(@source b"src/core/SkClipStack.h\0".as_ptr(), 119; @condition b"DeviceSpaceType::kRect == fDeviceSpaceType && (fDeviceSpaceRRect.isRect() || fDeviceSpaceRRect.isEmpty())\0".as_ptr(); self.device_space_type == DeviceSpaceType::kRect
                && (self.device_space_rrect.isRect() || self.device_space_rrect.isEmpty()));
        self.device_space_rrect.getBounds()
    }
    pub fn ref_shader(&self) -> Option<Arc<dyn SkShader>> {
        self.shader.clone()
    }
    pub fn get_shader(&self) -> Option<&dyn SkShader> {
        self.shader.as_deref()
    }
    pub fn get_op(&self) -> SkClipOp {
        self.op
    }
    pub fn is_replace_op(&self) -> bool {
        self.is_replace
    }
}
// Source: src/core/SkClipStack.h:142-156
impl Element {
    pub fn as_device_space_rrect(&self) -> &SkRRect {
        crate::SkASSERT!(@source b"src/core/SkClipStack.h\0".as_ptr(), 143; @condition b"DeviceSpaceType::kPath != fDeviceSpaceType\0".as_ptr(); (self.device_space_type) != (DeviceSpaceType::kPath));
        &self.device_space_rrect
    }
    pub fn is_aa(&self) -> bool {
        self.do_aa
    }
    pub fn get_gen_id(&self) -> u32 {
        crate::SkASSERT!(@source b"src/core/SkClipStack.h\0".as_ptr(), 156; @condition b"kInvalidGenID != fGenID\0".as_ptr(); (self.gen_id) != ( INVALID_GEN_ID));
        self.gen_id
    }
}
// Source: src/core/SkClipStack.h:174-177
impl Element {
    pub fn is_inverse_filled(&self) -> bool {
        self.device_space_type == DeviceSpaceType::kPath
            && self.device_space_path.as_ref().unwrap().isInverseFillType()
    }
}
// Source: src/core/SkClipStack.h:190-216
pub struct Element {
    device_space_path: Option<SkPath>,
    device_space_rrect: SkRRect,
    shader: Option<Arc<dyn SkShader>>,
    save_count: i32,
    op: SkClipOp,
    device_space_type: DeviceSpaceType,
    do_aa: bool,
    is_replace: bool,
    finite_bound_type: BoundsType,
    finite_bound: SkRect,
    is_intersection_of_rects: bool,
    gen_id: u32,
}
// Source: src/core/SkClipStack.h:217-220
impl Element {
    fn with_save_count(save_count: i32) -> Self {
        let mut element = Self::storage();
        element.init_common(save_count, SkClipOp::kIntersect, false);
        element.set_empty();
        element
    }
}

// Source: src/core/SkClipStack.h:263-268
#[derive(Clone, Copy)]
#[repr(i32)]
enum FillCombo {
    kPrev_Cur_FillCombo,
    kPrev_InvCur_FillCombo,
    kInvPrev_Cur_FillCombo,
    kInvPrev_InvCur_FillCombo,
}
// Source: src/core/SkClipStack.h:285-285
impl SkClipStack<'_> {
    pub fn get_save_count(&self) -> i32 {
        self.save_count
    }
}
// Source: src/core/SkClipStack.h:291-307
#[must_use]
pub struct AutoRestore<'stack, 'storage> {
    stack: &'stack mut SkClipStack<'storage>,
    save_count: i32,
}
impl<'stack, 'storage> AutoRestore<'stack, 'storage> {
    pub fn new(stack: &'stack mut SkClipStack<'storage>, do_save: bool) -> Self {
        let save_count = stack.get_save_count();
        if do_save {
            stack.save();
        }
        Self { stack, save_count }
    }
}
impl Drop for AutoRestore<'_, '_> {
    fn drop(&mut self) {
        crate::SkASSERT!(@source b"src/core/SkClipStack.h\0".as_ptr(), 299; @condition b"fCS->getSaveCount() >= fSaveCount\0".as_ptr(); self.stack.get_save_count() >= self.save_count);
        while self.stack.get_save_count() > self.save_count {
            self.stack.restore();
        }
    }
}
impl<'storage> core::ops::Deref for AutoRestore<'_, 'storage> {
    type Target = SkClipStack<'storage>;
    fn deref(&self) -> &Self::Target {
        self.stack
    }
}
impl core::ops::DerefMut for AutoRestore<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.stack
    }
}
// Source: src/core/SkClipStack.h:331-337
impl SkClipStack<'_> {
    pub fn quick_contains_rect(&self, rect: &SkRect) -> bool {
        self.is_wide_open() || self.internal_quick_contains_rect(rect)
    }
    pub fn quick_contains_rrect(&self, rrect: &SkRRect) -> bool {
        self.is_wide_open() || self.internal_quick_contains_rrect(rrect)
    }
}
// Source: src/core/SkClipStack.h:339-343
impl SkClipStack<'_> {
    pub fn clip_dev_rect(&mut self, rect: &crate::src::core::SkRect::SkIRect, op: SkClipOp) {
        let mut r = SkRect::MakeEmpty();
        r.set(rect);
        self.clip_rect(
            &r,
            &crate::src::core::SkMatrix::SkMatrix::default(),
            op,
            false,
        );
    }
}
// Source: src/core/SkClipStack.h:357-357
impl SkClipStack<'_> {
    pub fn is_wide_open(&self) -> bool {
        self.get_topmost_gen_id() == WIDE_OPEN_GEN_ID
    }
}
// Source: src/core/SkClipStack.h:378-382
pub const INVALID_GEN_ID: u32 = 0;
pub const EMPTY_GEN_ID: u32 = 1;
pub const WIDE_OPEN_GEN_ID: u32 = 2;

// Source: src/core/SkClipStack.h:397-400
#[derive(Clone, Copy)]
#[repr(i32)]
pub enum IterStart {
    kBottom_IterStart = 0,
    kTop_IterStart = 1,
}
// Source: src/core/SkClipStack.h:428-429
pub struct Iter<'stack, 'storage> {
    stack: Option<&'stack SkClipStack<'storage>>,
    iter: crate::src::core::SkDeque::Iter<'stack, 'storage>,
}
// Source: src/core/SkClipStack.h:436-438
pub struct B2TIter<'stack, 'storage> {
    iter: Iter<'stack, 'storage>,
}
impl Default for B2TIter<'_, '_> {
    fn default() -> Self {
        Self {
            iter: Iter::default(),
        }
    }
}
// Source: src/core/SkClipStack.h:444-448
impl<'stack, 'storage> B2TIter<'stack, 'storage> {
    pub fn new(stack: &'stack SkClipStack<'storage>) -> Self {
        Self {
            iter: Iter::new(stack, IterStart::kBottom_IterStart),
        }
    }
    pub fn next(&mut self) -> Option<&'stack Element> {
        self.iter.next()
    }
}
// Source: src/core/SkClipStack.h:454-456
impl<'stack, 'storage> B2TIter<'stack, 'storage> {
    pub fn reset(&mut self, stack: &'stack SkClipStack<'storage>) {
        self.iter.reset(stack, IterStart::kBottom_IterStart);
    }
}
// Source: src/core/SkClipStack.h:486-487
pub struct SkClipStack<'storage> {
    deque: crate::src::core::SkDeque::SkDeque<'storage>,
    save_count: i32,
}
// Source: src/core/SkClipStack.cpp:21-53
impl Clone for Element {
    fn clone(&self) -> Self {
        let mut rrect = SkRRect::default();
        let mut path = None;
        let mut shader = None;
        match self.device_space_type {
            DeviceSpaceType::kEmpty => {
                rrect.setEmpty();
            }
            DeviceSpaceType::kRect | DeviceSpaceType::kRRect => {
                rrect = self.device_space_rrect;
            }
            DeviceSpaceType::kPath => {
                path = Some(self.device_space_path.as_ref().unwrap().clone());
            }
            DeviceSpaceType::kShader => {
                shader = self.shader.clone();
            }
        }
        Self {
            device_space_path: path,
            device_space_rrect: rrect,
            shader,
            save_count: self.save_count,
            op: self.op,
            device_space_type: self.device_space_type,
            do_aa: self.do_aa,
            is_replace: self.is_replace,
            finite_bound_type: self.finite_bound_type,
            finite_bound: self.finite_bound,
            is_intersection_of_rects: self.is_intersection_of_rects,
            gen_id: self.gen_id,
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.assign_members(source);
    }
}
// Source: src/core/SkClipStack.h:190-216
// The implicit native copy assignment copies members, including inactive
// optional state; it is distinct from the explicit discriminated copy ctor.
impl Element {
    fn assign_members(&mut self, source: &Self) {
        self.device_space_path.clone_from(&source.device_space_path);
        self.device_space_rrect = source.device_space_rrect;
        let retained = source.shader.clone();
        let old = core::mem::replace(&mut self.shader, retained);
        drop(old);
        self.save_count = source.save_count;
        self.op = source.op;
        self.device_space_type = source.device_space_type;
        self.do_aa = source.do_aa;
        self.is_replace = source.is_replace;
        self.finite_bound_type = source.finite_bound_type;
        self.finite_bound = source.finite_bound;
        self.is_intersection_of_rects = source.is_intersection_of_rects;
        self.gen_id = source.gen_id;
    }
}
// Source: src/core/SkClipStack.cpp:55-55
// C++ implicit member destruction is Shader, RRect (trivial), Path. Rust field
// declaration order differs, so release retained members explicitly in native order.
// The Shader Arc remains an ownership adaptation, not a native sk_sp claim.
impl Drop for Element {
    fn drop(&mut self) {
        drop(self.shader.take());
        drop(self.device_space_path.take());
    }
}
// Source: src/core/SkClipStack.cpp:57-81
impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        if core::ptr::eq(self, other) {
            return true;
        }
        if self.op != other.op
            || self.device_space_type != other.device_space_type
            || self.do_aa != other.do_aa
            || self.is_replace != other.is_replace
            || self.save_count != other.save_count
        {
            return false;
        }
        match self.device_space_type {
            DeviceSpaceType::kShader => match (&self.shader, &other.shader) {
                (None, None) => true,
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                _ => false,
            },
            DeviceSpaceType::kPath => self.get_device_space_path() == other.get_device_space_path(),
            DeviceSpaceType::kRRect => self.device_space_rrect == other.device_space_rrect,
            DeviceSpaceType::kRect => self.get_device_space_rect() == other.get_device_space_rect(),
            DeviceSpaceType::kEmpty => true,
        }
    }
}
// Source: src/core/SkClipStack.cpp:83-103
impl Element {
    pub fn get_bounds(&self) -> &SkRect {
        static EMPTY: SkRect = SkRect::MakeEmpty();
        static INFINITE: std::sync::LazyLock<SkRect> =
            std::sync::LazyLock::new(crate::src::core::SkRectPriv::SkRectPriv::MakeLargeS32);
        match self.device_space_type {
            DeviceSpaceType::kRect | DeviceSpaceType::kRRect => self.device_space_rrect.getBounds(),
            DeviceSpaceType::kPath => self.device_space_path.as_ref().unwrap().getBounds(),
            DeviceSpaceType::kShader => &INFINITE,
            DeviceSpaceType::kEmpty => &EMPTY,
        }
    }
}
// Source: src/core/SkClipStack.cpp:105-120
impl Element {
    pub fn contains_rect(&self, rect: &SkRect) -> bool {
        match self.device_space_type {
            DeviceSpaceType::kRect => self.get_device_space_rect().contains(rect),
            DeviceSpaceType::kRRect => self.device_space_rrect.contains(rect),
            DeviceSpaceType::kPath => self
                .device_space_path
                .as_ref()
                .unwrap()
                .conservativelyContainsRect(rect),
            DeviceSpaceType::kEmpty | DeviceSpaceType::kShader => false,
        }
    }
}
// Source: src/core/SkClipStack.cpp:122-138
impl Element {
    pub fn contains_rrect(&self, rrect: &SkRRect) -> bool {
        match self.device_space_type {
            DeviceSpaceType::kRect => self.get_device_space_rect().contains(rrect.getBounds()),
            DeviceSpaceType::kRRect => {
                self.device_space_rrect.contains(rrect.getBounds())
                    || *rrect == self.device_space_rrect
            }
            DeviceSpaceType::kPath => self
                .device_space_path
                .as_ref()
                .unwrap()
                .conservativelyContainsRect(rrect.getBounds()),
            DeviceSpaceType::kEmpty | DeviceSpaceType::kShader => false,
        }
    }
}
// Source: src/core/SkClipStack.cpp:140-151
impl Element {
    fn init_common(&mut self, save_count: i32, op: SkClipOp, do_aa: bool) {
        self.save_count = save_count;
        self.op = op;
        self.do_aa = do_aa;
        self.is_replace = false;
        self.finite_bound_type = BoundsType::kInsideOut_BoundsType;
        self.finite_bound.setEmpty();
        self.is_intersection_of_rects = false;
        self.gen_id = INVALID_GEN_ID;
    }
}
// Source: src/core/SkClipStack.cpp:153-164
impl Element {
    fn init_rect(
        &mut self,
        save_count: i32,
        rect: &SkRect,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        op: SkClipOp,
        do_aa: bool,
    ) {
        if matrix.rectStaysRect() {
            let mut device_rect = SkRect::MakeEmpty();
            matrix.mapRect(&mut device_rect, rect);
            self.device_space_rrect.setRect(&device_rect);
            self.device_space_type = DeviceSpaceType::kRect;
            self.init_common(save_count, op, do_aa);
            return;
        }
        self.init_as_path(
            save_count,
            &SkPath::Rect(
                rect,
                crate::include::core::SkPathTypes::SkPathFillType::kWinding,
                crate::include::core::SkPathTypes::SkPathDirection::kCW,
                0,
            ),
            matrix,
            op,
            do_aa,
        );
    }
}
// Source: src/core/SkClipStack.cpp:166-180
impl Element {
    fn init_rrect(
        &mut self,
        save_count: i32,
        rrect: &SkRRect,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        op: SkClipOp,
        do_aa: bool,
    ) {
        if let Some(rr) = rrect.transform(matrix) {
            self.device_space_rrect = rr;
            let typ = self.device_space_rrect.getType();
            self.device_space_type = if typ == crate::src::core::SkRRect::Type::kRect_Type
                || typ == crate::src::core::SkRRect::Type::kEmpty_Type
            {
                DeviceSpaceType::kRect
            } else {
                DeviceSpaceType::kRRect
            };
            self.init_common(save_count, op, do_aa);
            return;
        }
        self.init_as_path(
            save_count,
            &SkPath::RRect(
                rrect,
                crate::include::core::SkPathTypes::SkPathDirection::kCW,
                6,
            ),
            matrix,
            op,
            do_aa,
        );
    }
}
// Source: src/core/SkClipStack.cpp:182-199
impl Element {
    fn init_path(
        &mut self,
        save_count: i32,
        path: &SkPath,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        op: SkClipOp,
        do_aa: bool,
    ) {
        if !path.isInverseFillType() {
            let mut rect = SkRect::MakeEmpty();
            if path.isRect(Some(&mut rect), None, None) {
                self.init_rect(save_count, &rect, matrix, op, do_aa);
                return;
            }
            let mut oval_rect = SkRect::MakeEmpty();
            if path.isOval(Some(&mut oval_rect)) {
                let mut rrect = SkRRect::default();
                rrect.setOval(&oval_rect);
                self.init_rrect(save_count, &rrect, matrix, op, do_aa);
                return;
            }
        }
        self.init_as_path(save_count, path, matrix, op, do_aa);
    }
}
// Source: src/core/SkClipStack.cpp:201-210
impl Element {
    fn init_as_path(
        &mut self,
        save_count: i32,
        path: &SkPath,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        op: SkClipOp,
        do_aa: bool,
    ) {
        let mut builder = crate::src::core::SkPathBuilder::SkPathBuilder::from_path(path);
        builder.transform(matrix);
        builder.setIsVolatile(true);
        self.device_space_path = Some(builder.detach(None));
        self.device_space_type = DeviceSpaceType::kPath;
        self.init_common(save_count, op, do_aa);
    }
}

// Source: src/core/SkClipStack.cpp:212-217
impl Element {
    fn init_shader(&mut self, save_count: i32, shader: Arc<dyn SkShader>) {
        self.device_space_type = DeviceSpaceType::kShader;
        let old_shader = core::mem::replace(&mut self.shader, Some(shader));
        drop(old_shader);
        self.init_common(save_count, SkClipOp::kIntersect, false);
    }
}
// Source: src/core/SkClipStack.cpp:219-224
impl Element {
    fn init_replace_rect(&mut self, save_count: i32, rect: &SkRect, do_aa: bool) {
        self.device_space_rrect.setRect(rect);
        self.device_space_type = DeviceSpaceType::kRect;
        self.init_common(save_count, SkClipOp::kIntersect, do_aa);
        self.is_replace = true;
    }
}
// Source: src/core/SkClipStack.cpp:226-244
impl Element {
    pub fn as_device_space_path(&self) -> SkPath {
        use crate::include::core::SkPathTypes::SkPathDirection;
        let mut builder = crate::src::core::SkPathBuilder::SkPathBuilder::new();
        match self.device_space_type {
            DeviceSpaceType::kEmpty => {}
            DeviceSpaceType::kRect => {
                builder.addRect(self.get_device_space_rect(), SkPathDirection::kCW, 0);
            }
            DeviceSpaceType::kRRect => {
                builder.addRRect(&self.device_space_rrect, SkPathDirection::kCW, 6);
            }
            DeviceSpaceType::kPath => return self.device_space_path.as_ref().unwrap().clone(),
            DeviceSpaceType::kShader => {
                builder.addRect(
                    &crate::src::core::SkRectPriv::SkRectPriv::MakeLargeS32(),
                    SkPathDirection::kCW,
                    0,
                );
            }
        }
        builder.detach(None)
    }
}

// Rust requires initialized fields before the upstream init methods can borrow
// self. These staging values remain private; every constructor below immediately
// invokes the original initializer before returning the element.
impl Element {
    fn storage() -> Self {
        Self {
            device_space_path: None,
            device_space_rrect: SkRRect::default(),
            shader: None,
            save_count: 0,
            op: SkClipOp::kIntersect,
            device_space_type: DeviceSpaceType::kEmpty,
            do_aa: false,
            is_replace: false,
            finite_bound_type: BoundsType::kInsideOut_BoundsType,
            finite_bound: SkRect::MakeEmpty(),
            is_intersection_of_rects: false,
            gen_id: INVALID_GEN_ID,
        }
    }
}
// Source: src/core/SkClipStack.cpp:246-256
impl Element {
    fn set_empty(&mut self) {
        self.device_space_type = DeviceSpaceType::kEmpty;
        self.finite_bound.setEmpty();
        self.finite_bound_type = BoundsType::kNormal_BoundsType;
        self.is_intersection_of_rects = false;
        self.device_space_rrect.setEmpty();
        self.device_space_path = None;
        let old_shader = self.shader.take();
        drop(old_shader);
        self.gen_id = EMPTY_GEN_ID;
        #[cfg(sk_debug)]
        self.check_empty();
    }
}
// Source: src/core/SkClipStack.cpp:258-266
impl Element {
    fn check_empty(&self) {
        crate::SkASSERT!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 259; @condition b"fFiniteBound.isEmpty()\0".as_ptr(); self.finite_bound.isEmpty());
        crate::SkASSERT!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 260; @condition b"kNormal_BoundsType == fFiniteBoundType\0".as_ptr(); (self.finite_bound_type) == (BoundsType::kNormal_BoundsType));
        crate::SkASSERT!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 261; @condition b"!fIsIntersectionOfRects\0".as_ptr(); !self.is_intersection_of_rects);
        crate::SkASSERT!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 262; @condition b"kEmptyGenID == fGenID\0".as_ptr(); (self.gen_id) == ( EMPTY_GEN_ID));
        crate::SkASSERT!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 263; @condition b"fDeviceSpaceRRect.isEmpty()\0".as_ptr(); self.device_space_rrect.isEmpty());
        crate::SkASSERT!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 264; @condition b"!fDeviceSpacePath.has_value()\0".as_ptr(); self.device_space_path.is_none());
        crate::SkASSERT!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 265; @condition b"!fShader\0".as_ptr(); self.shader.is_none());
    }
}
// Source: src/core/SkClipStack.cpp:268-278
impl Element {
    fn can_be_intersected_in_place(&self, save_count: i32, op: SkClipOp) -> bool {
        if self.device_space_type == DeviceSpaceType::kEmpty
            && (op == SkClipOp::kDifference || op == SkClipOp::kIntersect)
        {
            return true;
        }
        self.save_count == save_count
            && op == SkClipOp::kIntersect
            && (self.op == SkClipOp::kIntersect || self.is_replace)
    }
}
// Source: src/core/SkClipStack.cpp:280-305
impl Element {
    fn rect_rect_intersect_allowed(&self, new_rect: &SkRect, new_aa: bool) -> bool {
        crate::SkASSERT!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 281; @condition b"DeviceSpaceType::kRect == fDeviceSpaceType\0".as_ptr(); (self.device_space_type) == (DeviceSpaceType::kRect));
        if self.do_aa == new_aa {
            return true;
        }
        if !SkRect::Intersects(self.device_space_rrect.getBounds(), new_rect) {
            return true;
        }
        if self.device_space_rrect.getBounds().contains(new_rect) {
            return true;
        }
        false
    }
}

// Source: src/core/SkClipStack.cpp:308-349
impl Element {
    #[allow(unreachable_patterns)]
    fn combine_bounds_diff(&mut self, combination: FillCombo, previous: &SkRect) {
        match combination {
            FillCombo::kInvPrev_InvCur_FillCombo => {
                self.finite_bound_type = BoundsType::kNormal_BoundsType;
            }
            FillCombo::kInvPrev_Cur_FillCombo => {
                self.finite_bound.join(previous);
                self.finite_bound_type = BoundsType::kInsideOut_BoundsType;
            }
            FillCombo::kPrev_InvCur_FillCombo => {
                if !self.finite_bound.intersect(previous) {
                    self.finite_bound.setEmpty();
                    self.gen_id = EMPTY_GEN_ID;
                }
                self.finite_bound_type = BoundsType::kNormal_BoundsType;
            }
            FillCombo::kPrev_Cur_FillCombo => {
                self.finite_bound = *previous;
            }
            _ => {
                crate::SkDEBUGFAIL!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 345; b"SkClipStack::Element::combineBoundsDiff Invalid fill combination\0".as_ptr());
            }
        }
    }
}
// Source: src/core/SkClipStack.cpp:351-379
impl Element {
    fn combine_bounds_intersection(&mut self, combination: i32, previous: &SkRect) {
        match combination {
            3 => {
                self.finite_bound.join(previous);
                self.finite_bound_type = BoundsType::kInsideOut_BoundsType;
            }
            2 => {}
            1 => {
                self.finite_bound = *previous;
                self.finite_bound_type = BoundsType::kNormal_BoundsType;
            }
            0 => {
                if !self.finite_bound.intersect(previous) {
                    self.set_empty();
                }
            }
            _ => {
                crate::SkDEBUGFAIL!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 376; b"SkClipStack::Element::combineBoundsIntersection Invalid fill combination\0".as_ptr());
            }
        }
    }
}

// Source: src/core/SkClipStack.cpp:381-469
impl Element {
    fn update_bound_and_gen_id(&mut self, prior: Option<&Self>) {
        self.gen_id = get_next_gen_id();
        self.is_intersection_of_rects = false;
        match self.device_space_type {
            DeviceSpaceType::kRect => {
                self.finite_bound = *self.device_space_rrect.getBounds();
                self.finite_bound_type = BoundsType::kNormal_BoundsType;
                if self.is_replace
                    || (self.op == SkClipOp::kIntersect && prior.is_none())
                    || (self.op == SkClipOp::kIntersect
                        && prior.unwrap().is_intersection_of_rects
                        && prior.unwrap().rect_rect_intersect_allowed(
                            self.device_space_rrect.getBounds(),
                            self.do_aa,
                        ))
                {
                    self.is_intersection_of_rects = true;
                }
            }
            DeviceSpaceType::kRRect => {
                self.finite_bound = *self.device_space_rrect.getBounds();
                self.finite_bound_type = BoundsType::kNormal_BoundsType;
            }
            DeviceSpaceType::kPath => {
                let path = self.device_space_path.as_ref().unwrap();
                self.finite_bound = *path.getBounds();
                self.finite_bound_type = if path.isInverseFillType() {
                    BoundsType::kInsideOut_BoundsType
                } else {
                    BoundsType::kNormal_BoundsType
                };
            }
            DeviceSpaceType::kShader => {
                self.finite_bound = crate::src::core::SkRectPriv::SkRectPriv::MakeLargeS32();
                self.finite_bound_type = BoundsType::kNormal_BoundsType;
            }
            DeviceSpaceType::kEmpty => {
                crate::SkDEBUGFAIL!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 423; b"We shouldn't get here with an empty element.\0".as_ptr());
            }
        }
        let (previous_finite, previous_type) = if let Some(prior) = prior {
            (prior.finite_bound, prior.finite_bound_type)
        } else {
            (SkRect::MakeEmpty(), BoundsType::kInsideOut_BoundsType)
        };
        let mut combination = 0;
        if self.finite_bound_type == BoundsType::kInsideOut_BoundsType {
            combination |= 0x01;
        }
        if previous_type == BoundsType::kInsideOut_BoundsType {
            combination |= 0x02;
        }
        crate::SkASSERT!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 449; @condition b"kInvPrev_InvCur_FillCombo == combination || kInvPrev_Cur_FillCombo == combination || kPrev_InvCur_FillCombo == combination || kPrev_Cur_FillCombo == combination\0".as_ptr(); combination == 3 || combination == 2 || combination == 1 || combination == 0);
        if !self.is_replace {
            match self.op {
                SkClipOp::kDifference => {
                    let combination = match combination {
                        0 => FillCombo::kPrev_Cur_FillCombo,
                        1 => FillCombo::kPrev_InvCur_FillCombo,
                        2 => FillCombo::kInvPrev_Cur_FillCombo,
                        3 => FillCombo::kInvPrev_InvCur_FillCombo,
                        _ => unreachable!(),
                    };
                    self.combine_bounds_diff(combination, &previous_finite);
                }
                SkClipOp::kIntersect => {
                    self.combine_bounds_intersection(combination, &previous_finite);
                }
            }
        }
    }
}

// Source: src/core/SkClipStack.cpp:475-480
impl SkClipStack<'_> {
    pub fn new() -> Self {
        Self {
            deque: crate::src::core::SkDeque::SkDeque::new(core::mem::size_of::<Element>(), 8),
            save_count: 0,
        }
    }
}
// Source: src/core/SkClipStack.cpp:482-485
impl<'storage> SkClipStack<'storage> {
    /// Safety: storage satisfies canonical SkDeque::with_storage's alignment,
    /// lifetime and exclusive-access contract. This stack only pushes/pops back.
    pub unsafe fn with_storage(storage: *mut u8, size: usize) -> Self {
        Self {
            deque: unsafe {
                crate::src::core::SkDeque::SkDeque::with_storage(
                    core::mem::size_of::<Element>(),
                    storage,
                    size,
                    8,
                )
            },
            save_count: 0,
        }
    }
}
// Source: src/core/SkClipStack.cpp:487-490
impl Clone for SkClipStack<'_> {
    fn clone(&self) -> Self {
        let mut stack = Self::new();
        stack.assign(self);
        stack
    }
}
// Source: src/core/SkClipStack.cpp:492-494
impl Drop for SkClipStack<'_> {
    fn drop(&mut self) {
        self.reset();
    }
}
// Source: src/core/SkClipStack.cpp:496-511
impl SkClipStack<'_> {
    pub fn assign(&mut self, other: &SkClipStack<'_>) -> &mut Self {
        if core::ptr::eq(self, other) {
            return self;
        }
        self.reset();
        self.save_count = other.save_count;
        let mut iter = crate::src::core::SkDeque::F2BIter::new(&other.deque);
        loop {
            let element = iter.next().cast::<Element>();
            if element.is_null() {
                break;
            }
            unsafe {
                self.deque
                    .push_back()
                    .cast::<Element>()
                    .write((*element).clone());
            }
        }
        self
    }
}
// Source: src/core/SkClipStack.cpp:513-534
impl PartialEq for SkClipStack<'_> {
    fn eq(&self, other: &Self) -> bool {
        if self.get_topmost_gen_id() == other.get_topmost_gen_id() {
            return true;
        }
        if self.save_count != other.save_count || self.deque.count() != other.deque.count() {
            return false;
        }
        let mut ours = crate::src::core::SkDeque::F2BIter::new(&self.deque);
        let mut theirs = crate::src::core::SkDeque::F2BIter::new(&other.deque);
        let mut a = ours.next().cast::<Element>();
        let mut b = theirs.next().cast::<Element>();
        while !a.is_null() && !b.is_null() {
            if unsafe { &*a != &*b } {
                return false;
            }
            a = ours.next().cast();
            b = theirs.next().cast();
        }
        a.is_null() && b.is_null()
    }
}
// Source: src/core/SkClipStack.cpp:536-546
impl SkClipStack<'_> {
    pub fn reset(&mut self) {
        while !self.deque.empty() {
            unsafe {
                core::ptr::drop_in_place(self.deque.back_mut().cast::<Element>());
                self.deque.pop_back();
            }
        }
        self.save_count = 0;
    }
}
// Source: src/core/SkClipStack.cpp:548-550
impl SkClipStack<'_> {
    pub fn save(&mut self) {
        self.save_count = self.save_count.wrapping_add(1);
    }
}
// Source: src/core/SkClipStack.cpp:552-555
impl SkClipStack<'_> {
    pub fn restore(&mut self) {
        self.save_count = self.save_count.wrapping_sub(1);
        self.restore_to(self.save_count);
    }
}
// Source: src/core/SkClipStack.cpp:557-566
impl SkClipStack<'_> {
    fn restore_to(&mut self, save_count: i32) {
        while !self.deque.empty() {
            unsafe {
                let element = self.deque.back_mut().cast::<Element>();
                if (*element).save_count <= save_count {
                    break;
                }
                core::ptr::drop_in_place(element);
                self.deque.pop_back();
            }
        }
    }
}
// Source: src/core/SkClipStack.cpp:568-577
impl SkClipStack<'_> {
    pub fn bounds(&self, device_bounds: &crate::src::core::SkRect::SkIRect) -> SkRect {
        let mut rect = SkRect::MakeEmpty();
        let mut bound_type = BoundsType::kNormal_BoundsType;
        self.get_bounds(&mut rect, &mut bound_type, None);
        if bound_type == BoundsType::kInsideOut_BoundsType {
            return SkRect::Make(device_bounds);
        }
        if rect.intersect(&SkRect::Make(device_bounds)) {
            rect
        } else {
            SkRect::MakeEmpty()
        }
    }
}
// Source: src/core/SkClipStack.cpp:580-580
impl SkClipStack<'_> {
    pub fn is_empty(&self, rect: &crate::src::core::SkRect::SkIRect) -> bool {
        self.bounds(rect).isEmpty()
    }
}
// Source: src/core/SkClipStack.cpp:582-604
impl SkClipStack<'_> {
    pub fn get_bounds(
        &self,
        bounds: &mut SkRect,
        bound_type: &mut BoundsType,
        is_intersection_of_rects: Option<&mut bool>,
    ) {
        let element = self.deque.back().cast::<Element>();
        if element.is_null() {
            bounds.setEmpty();
            *bound_type = BoundsType::kInsideOut_BoundsType;
            if let Some(result) = is_intersection_of_rects {
                *result = false;
            }
            return;
        }
        unsafe {
            *bounds = (*element).finite_bound;
            *bound_type = (*element).finite_bound_type;
            if let Some(result) = is_intersection_of_rects {
                *result = (*element).is_intersection_of_rects;
            }
        }
    }
}
// Source: src/core/SkClipStack.cpp:606-630
impl SkClipStack<'_> {
    fn internal_quick_contains_rect(&self, rect: &SkRect) -> bool {
        let mut iter = Iter::new(self, IterStart::kTop_IterStart);
        while let Some(element) = iter.prev() {
            if element.get_op() != SkClipOp::kIntersect && !element.is_replace_op() {
                return false;
            }
            if element.is_inverse_filled() {
                if SkRect::Intersects(element.get_bounds(), rect) {
                    return false;
                }
            } else if !element.contains_rect(rect) {
                return false;
            }
            if element.is_replace_op() {
                break;
            }
        }
        true
    }
}
// Source: src/core/SkClipStack.cpp:632-656
impl SkClipStack<'_> {
    fn internal_quick_contains_rrect(&self, rrect: &SkRRect) -> bool {
        let mut iter = Iter::new(self, IterStart::kTop_IterStart);
        while let Some(element) = iter.prev() {
            if element.get_op() != SkClipOp::kIntersect && !element.is_replace_op() {
                return false;
            }
            if element.is_inverse_filled() {
                if SkRect::Intersects(element.get_bounds(), rrect.getBounds()) {
                    return false;
                }
            } else if !element.contains_rrect(rrect) {
                return false;
            }
            if element.is_replace_op() {
                break;
            }
        }
        true
    }
}
// Source: src/core/SkClipStack.cpp:658-712
impl SkClipStack<'_> {
    fn push_element(&mut self, element: &Element) {
        use crate::src::core::SkDeque::{Iter, IterStart};
        let mut iter = Iter::new(&self.deque, IterStart::kBack_IterStart);
        let mut prior = iter.prev().cast::<Element>();
        unsafe {
            if !prior.is_null() {
                if element.is_replace_op() {
                    self.restore_to(self.save_count.wrapping_sub(1));
                    prior = self.deque.back_mut().cast();
                } else if (*prior).can_be_intersected_in_place(self.save_count, element.get_op()) {
                    match (*prior).device_space_type {
                        DeviceSpaceType::kEmpty => {
                            #[cfg(sk_debug)]
                            (*prior).check_empty();
                            return;
                        }
                        DeviceSpaceType::kShader => {
                            if element.device_space_type == DeviceSpaceType::kShader {
                                let shader = crate::src::shaders::SkBlendShader::Blend(
                                    crate::src::core::SkBlendMode::SkBlendMode::kSrcIn,
                                    element.shader.clone(),
                                    (*prior).shader.clone(),
                                );
                                let old = core::mem::replace(&mut (*prior).shader, shader);
                                drop(old);
                                let prior_prior = iter.prev().cast::<Element>();
                                (*prior).update_bound_and_gen_id(prior_prior.as_ref());
                                return;
                            }
                        }
                        DeviceSpaceType::kRect => {
                            if element.device_space_type == DeviceSpaceType::kRect {
                                if (*prior).rect_rect_intersect_allowed(
                                    element.get_device_space_rect(),
                                    element.is_aa(),
                                ) {
                                    let mut intersection = SkRect::MakeEmpty();
                                    if !intersection.intersect_ab(
                                        *(*prior).get_device_space_rect(),
                                        *element.get_device_space_rect(),
                                    ) {
                                        (*prior).set_empty();
                                        return;
                                    }
                                    (*prior).device_space_rrect.setRect(&intersection);
                                    (*prior).do_aa = element.is_aa();
                                    let prior_prior = iter.prev().cast::<Element>();
                                    (*prior).update_bound_and_gen_id(prior_prior.as_ref());
                                    return;
                                }
                            } else if !SkRect::Intersects(
                                (*prior).get_bounds(),
                                element.get_bounds(),
                            ) {
                                (*prior).set_empty();
                                return;
                            }
                        }
                        _ => {
                            if !SkRect::Intersects((*prior).get_bounds(), element.get_bounds()) {
                                (*prior).set_empty();
                                return;
                            }
                        }
                    }
                }
            }
            let new_element = self.deque.push_back().cast::<Element>();
            new_element.write(element.clone());
            (*new_element).update_bound_and_gen_id(prior.as_ref());
        }
    }
}
// Source: src/core/SkClipStack.cpp:714-717
impl SkClipStack<'_> {
    pub fn clip_rrect(
        &mut self,
        rrect: &SkRRect,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        op: SkClipOp,
        aa: bool,
    ) {
        let mut element = Element::storage();
        element.init_rrect(self.save_count, rrect, matrix, op, aa);
        self.push_element(&element);
    }
}
// Source: src/core/SkClipStack.cpp:719-722
impl SkClipStack<'_> {
    pub fn clip_rect(
        &mut self,
        rect: &SkRect,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        op: SkClipOp,
        aa: bool,
    ) {
        let mut element = Element::storage();
        element.init_rect(self.save_count, rect, matrix, op, aa);
        self.push_element(&element);
    }
}
// Source: src/core/SkClipStack.cpp:724-728
impl SkClipStack<'_> {
    pub fn clip_path(
        &mut self,
        path: &SkPath,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        op: SkClipOp,
        aa: bool,
    ) {
        let mut element = Element::storage();
        element.init_path(self.save_count, path, matrix, op, aa);
        self.push_element(&element);
    }
}
// Source: src/core/SkClipStack.cpp:730-733
impl SkClipStack<'_> {
    pub fn clip_shader(&mut self, shader: Arc<dyn SkShader>) {
        let mut element = Element::storage();
        element.init_shader(self.save_count, shader);
        self.push_element(&element);
    }
}
// Source: src/core/SkClipStack.cpp:735-738
impl SkClipStack<'_> {
    pub fn replace_clip(&mut self, rect: &SkRect, aa: bool) {
        let mut element = Element::storage();
        element.init_replace_rect(self.save_count, rect, aa);
        self.push_element(&element);
    }
}
// Source: src/core/SkClipStack.cpp:740-749
impl SkClipStack<'_> {
    pub fn clip_empty(&mut self) {
        unsafe {
            let element = self.deque.back_mut().cast::<Element>();
            if !element.is_null()
                && (*element).can_be_intersected_in_place(self.save_count, SkClipOp::kIntersect)
            {
                (*element).set_empty();
            }
            self.deque
                .push_back()
                .cast::<Element>()
                .write(Element::with_save_count(self.save_count));
            (*self.deque.back_mut().cast::<Element>()).gen_id = EMPTY_GEN_ID;
        }
    }
}

// Source: src/core/SkClipStack.cpp:753-754
impl Default for Iter<'_, '_> {
    fn default() -> Self {
        Self {
            stack: None,
            iter: Default::default(),
        }
    }
}
// Source: src/core/SkClipStack.cpp:756-759
impl<'stack, 'storage> Iter<'stack, 'storage> {
    pub fn new(stack: &'stack SkClipStack<'storage>, start: IterStart) -> Self {
        let mut iter = Self {
            stack: Some(stack),
            iter: Default::default(),
        };
        iter.reset(stack, start);
        iter
    }
}
// Source: src/core/SkClipStack.cpp:761-763
impl<'stack> Iter<'stack, '_> {
    pub fn next(&mut self) -> Option<&'stack Element> {
        unsafe { self.iter.next().cast::<Element>().as_ref() }
    }
}
// Source: src/core/SkClipStack.cpp:765-767
impl<'stack> Iter<'stack, '_> {
    pub fn prev(&mut self) -> Option<&'stack Element> {
        unsafe { self.iter.prev().cast::<Element>().as_ref() }
    }
}
// Source: src/core/SkClipStack.cpp:769-805
impl<'stack> Iter<'stack, '_> {
    pub fn skip_to_topmost(&mut self, op: SkClipOp) -> Option<&'stack Element> {
        use crate::src::core::SkDeque::IterStart as DequeStart;
        let stack = self.stack?;
        self.iter.reset(&stack.deque, DequeStart::kBack_IterStart);
        let mut element;
        loop {
            element = self.iter.prev().cast::<Element>();
            if element.is_null() {
                break;
            }
            if unsafe { (*element).op == op } {
                if self.iter.next().is_null() {
                    self.iter.reset(&stack.deque, DequeStart::kFront_IterStart);
                }
                break;
            }
        }
        if element.is_null() {
            self.iter.reset(&stack.deque, DequeStart::kFront_IterStart);
        }
        self.next()
    }
}
// Source: src/core/SkClipStack.cpp:807-810
impl<'stack, 'storage> Iter<'stack, 'storage> {
    pub fn reset(&mut self, stack: &'stack SkClipStack<'storage>, start: IterStart) {
        self.stack = Some(stack);
        self.iter.reset(
            &stack.deque,
            match start {
                IterStart::kBottom_IterStart => {
                    crate::src::core::SkDeque::IterStart::kFront_IterStart
                }
                IterStart::kTop_IterStart => crate::src::core::SkDeque::IterStart::kBack_IterStart,
            },
        );
    }
}
// Source: src/core/SkClipStack.cpp:813-839
impl SkClipStack<'_> {
    pub fn get_conservative_bounds(
        &self,
        offset_x: i32,
        offset_y: i32,
        max_width: i32,
        max_height: i32,
        device_bounds: &mut SkRect,
        is_intersection_of_rects: Option<&mut bool>,
    ) {
        device_bounds.setLTRB(0., 0., max_width as f32, max_height as f32);
        let mut temporary = SkRect::MakeEmpty();
        let mut bound_type = BoundsType::kNormal_BoundsType;
        self.get_bounds(&mut temporary, &mut bound_type, is_intersection_of_rects);
        if bound_type == BoundsType::kInsideOut_BoundsType {
            return;
        }
        temporary.offset(offset_x as f32, offset_y as f32);
        if !device_bounds.intersect(&temporary) {
            device_bounds.setEmpty();
        }
    }
}
// Source: src/core/SkClipStack.cpp:841-894
impl SkClipStack<'_> {
    pub fn is_rrect(&self, bounds: &SkRect, rrect: &mut SkRRect, aa: &mut bool) -> bool {
        let back = self.deque.back().cast::<Element>();
        let Some(back) = (unsafe { back.as_ref() }) else {
            return false;
        };
        if back.is_intersection_of_rects && back.finite_bound_type == BoundsType::kNormal_BoundsType
        {
            rrect.setRect(&back.finite_bound);
            *aa = back.is_aa();
            return true;
        }
        if back.device_space_type != DeviceSpaceType::kRect
            && back.device_space_type != DeviceSpaceType::kRRect
        {
            return false;
        }
        if back.is_replace_op() {
            *rrect = *back.as_device_space_rrect();
            *aa = back.is_aa();
            return true;
        }
        if back.get_op() == SkClipOp::kIntersect {
            let mut back_bounds = SkRect::MakeEmpty();
            if !back_bounds.intersect_ab(*bounds, *back.as_device_space_rrect().rect()) {
                return false;
            }
            let count = self.deque.count();
            if count > 17 {
                return false;
            }
            if count > 1 {
                let mut iter = crate::src::core::SkDeque::Iter::new(
                    &self.deque,
                    crate::src::core::SkDeque::IterStart::kBack_IterStart,
                );
                let first = iter.prev().cast::<Element>();
                crate::SkAssertResult!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 877; @condition b"static_cast<const Element*>(iter.prev()) == back\0".as_ptr(); core::ptr::eq(first, back));
                loop {
                    let prior = iter.prev().cast::<Element>();
                    let Some(prior) = (unsafe { prior.as_ref() }) else {
                        break;
                    };
                    if (prior.get_op() != SkClipOp::kIntersect && !prior.is_replace_op())
                        || !prior.contains_rect(&back_bounds)
                    {
                        return false;
                    }
                    if prior.is_replace_op() {
                        break;
                    }
                }
            }
            *rrect = *back.as_device_space_rrect();
            *aa = back.is_aa();
            return true;
        }
        false
    }
}

// Source: src/core/SkClipStack.cpp:896-907
fn get_next_gen_id() -> u32 {
    static NEXT_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(3);
    loop {
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if id >= 3 {
            return id;
        }
    }
}
// Source: src/core/SkClipStack.cpp:908-920
impl SkClipStack<'_> {
    pub fn get_topmost_gen_id(&self) -> u32 {
        if self.deque.empty() {
            return WIDE_OPEN_GEN_ID;
        }
        let back = unsafe { &*self.deque.back().cast::<Element>() };
        if back.finite_bound_type == BoundsType::kInsideOut_BoundsType
            && back.finite_bound.isEmpty()
            && back.device_space_type != DeviceSpaceType::kShader
        {
            return WIDE_OPEN_GEN_ID;
        }
        back.get_gen_id()
    }
}

// Rust-only provenance rebinding for a pinned embedding owner's native inline storage.
impl SkClipStack<'_> {
    /// # Safety
    /// Same permanent inline allocation and size originally passed to with_storage;
    /// caller owns the current exclusive full-allocation provenance and must not move it.
    pub(crate) unsafe fn rebind_initial_storage(&mut self, storage: *mut u8, size: usize) {
        unsafe {
            self.deque.rebind_initial_storage(storage, size);
        }
    }
}

// Source: src/core/SkClipStack.cpp:923-961
#[cfg(sk_debug)]
impl Element {
    pub fn dump(&self) {
        const TYPE_STRINGS: [&[u8]; 5] =
            [b"empty\0", b"rect\0", b"rrect\0", b"path\0", b"shader\0"];
        const _: () = assert!(DeviceSpaceType::kEmpty as i32 == 0);
        const _: () = assert!(DeviceSpaceType::kRect as i32 == 1);
        const _: () = assert!(DeviceSpaceType::kRRect as i32 == 2);
        const _: () = assert!(DeviceSpaceType::kPath as i32 == 3);
        const _: () = assert!(DeviceSpaceType::kShader as i32 == 4);
        const _: () = assert!(TYPE_STRINGS.len() == TYPE_COUNT);
        let op_name: &[u8] = if self.is_replace_op() {
            b"replace\0"
        } else if self.op == SkClipOp::kDifference {
            b"difference\0"
        } else {
            b"intersect\0"
        };
        let aa: &[u8] = if self.do_aa { b"yes\0" } else { b"no\0" };
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 940; b"Type: %s, Op: %s, AA: %s, Save Count: %d\n\0".as_ptr(),
            TYPE_STRINGS[self.device_space_type as usize].as_ptr(),op_name.as_ptr(),aa.as_ptr(),self.save_count as core::ffi::c_int);
        }
        match self.device_space_type {
            DeviceSpaceType::kEmpty => unsafe {
                crate::SkDebugf!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 944; b"\n\0".as_ptr());
            },
            DeviceSpaceType::kRect => {
                self.get_device_space_rect().dump();
                unsafe {
                    crate::SkDebugf!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 948; b"\n\0".as_ptr());
                }
            }
            DeviceSpaceType::kRRect => {
                self.get_device_space_rrect().dump();
                unsafe {
                    crate::SkDebugf!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 952; b"\n\0".as_ptr());
                }
            }
            DeviceSpaceType::kPath => self.get_device_space_path().dump(None, false),
            DeviceSpaceType::kShader => {}
        }
    }
}
// Source: src/core/SkClipStack.cpp:963-970
#[cfg(sk_debug)]
impl SkClipStack<'_> {
    pub fn dump(&self) {
        let mut iter = B2TIter::new(self);
        while let Some(element) = iter.next() {
            element.dump();
            unsafe {
                crate::SkDebugf!(@source b"src/core/SkClipStack.cpp\0".as_ptr(), 968; b"\n\0".as_ptr());
            }
        }
    }
}
