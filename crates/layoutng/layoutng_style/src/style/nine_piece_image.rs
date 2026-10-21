use std::cell::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};

use foundation::{
    LayoutUnit, Length, LengthBox, MakeGarbageCollected, Member, Persistent, Visitor,
};

use super::border_image_length::BorderImageLength;
use super::border_image_length_box::BorderImageLengthBox;
use super::style_image::StyleImage;

impl foundation::Traceable for NinePieceImageData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        NinePieceImageData::Trace(self, visitor);
    }
}

impl foundation::Traceable for NinePieceImage {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        NinePieceImage::Trace(self, visitor);
    }
}

// cpp: layoutng_style/style/nine_piece_image.h:39-44
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ENinePieceImageRule {
    kStretchImageRule,
    kRoundImageRule,
    kSpaceImageRule,
    kRepeatImageRule,
}

// cpp: layoutng_style/style/nine_piece_image.h:46-65
pub struct NinePieceImageData {
    pub single_owner: AtomicBool,
    pub fill: bool,
    pub horizontal_rule: ENinePieceImageRule,
    pub vertical_rule: ENinePieceImageRule,
    pub image: Member<StyleImage>,
    pub image_slices: LengthBox,
    pub border_slices: BorderImageLengthBox,
    pub outset: BorderImageLengthBox,
}

// cpp: layoutng_style/style/nine_piece_image.h:49-50
// cpp: layoutng_style/style/nine_piece_image.h:56-64
impl Default for NinePieceImageData {
    fn default() -> Self {
        Self {
            single_owner: AtomicBool::new(true),
            fill: false,
            horizontal_rule: ENinePieceImageRule::kStretchImageRule,
            vertical_rule: ENinePieceImageRule::kStretchImageRule,
            image: Member::default(),
            image_slices: LengthBox::new(
                Length::Percent(100.0),
                Length::Percent(100.0),
                Length::Percent(100.0),
                Length::Percent(100.0),
            ),
            border_slices: BorderImageLengthBox::from_number(1.0),
            outset: BorderImageLengthBox::from_number(0.0),
        }
    }
}

// cpp: layoutng_style/style/nine_piece_image.h:50
impl Clone for NinePieceImageData {
    fn clone(&self) -> Self {
        Self {
            single_owner: AtomicBool::new(self.single_owner.load(Ordering::Relaxed)),
            fill: self.fill,
            horizontal_rule: self.horizontal_rule,
            vertical_rule: self.vertical_rule,
            image: self.image.clone(),
            image_slices: self.image_slices.clone(),
            border_slices: self.border_slices.clone(),
            outset: self.outset.clone(),
        }
    }
}

#[allow(non_snake_case)]
impl NinePieceImageData {
    // cpp: layoutng_style/style/nine_piece_image.h:54
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.image);
    }
}

// cpp: layoutng_style/style/nine_piece_image.h:52
// cpp: layoutng_style/style/nine_piece_image.cc:72-78
impl PartialEq for NinePieceImageData {
    fn eq(&self, other: &Self) -> bool {
        foundation::ValuesEquivalent(&self.image, &other.image)
            && self.image_slices == other.image_slices
            && self.fill == other.fill
            && self.border_slices == other.border_slices
            && self.outset == other.outset
            && self.horizontal_rule == other.horizontal_rule
            && self.vertical_rule == other.vertical_rule
    }
}

// cpp: layoutng_style/style/nine_piece_image.cc:33-38
#[allow(non_snake_case)]
fn DefaultData() -> *mut NinePieceImageData {
    // Persistent roots and their allocations belong to the current layout
    // thread, like ComputedStyle::GetInitialStyleSingleton.
    thread_local! {
        static DATA: OnceCell<Persistent<NinePieceImageData>> = OnceCell::new();
    }
    DATA.with(|slot| {
        let data = slot.get_or_init(|| {
            Persistent::from_ptr(MakeGarbageCollected(NinePieceImageData::default()))
        });
        let ptr = data.Get();
        unsafe { (*ptr).single_owner.store(false, Ordering::Relaxed) };
        ptr
    })
}

// cpp: layoutng_style/style/nine_piece_image.h:67-79
pub struct NinePieceImage {
    data_: Member<NinePieceImageData>,
}

impl Default for NinePieceImage {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(non_snake_case)]
impl NinePieceImage {
    // cpp: layoutng_style/style/nine_piece_image.h:71
    // cpp: layoutng_style/style/nine_piece_image.cc:40
    pub fn new() -> Self {
        Self {
            data_: Member::from_ptr(DefaultData()),
        }
    }

    // cpp: layoutng_style/style/nine_piece_image.h:72-78
    // cpp: layoutng_style/style/nine_piece_image.cc:42-57
    pub fn from_parts(
        image: *mut StyleImage,
        image_slices: LengthBox,
        fill: bool,
        border_slices: &BorderImageLengthBox,
        outset: &BorderImageLengthBox,
        horizontal_rule: ENinePieceImageRule,
        vertical_rule: ENinePieceImageRule,
    ) -> Self {
        let data = MakeGarbageCollected(NinePieceImageData::default());
        unsafe {
            (*data).image = Member::from_ptr(image);
            (*data).image_slices = image_slices;
            (*data).border_slices = border_slices.clone();
            (*data).outset = outset.clone();
            (*data).fill = fill;
            (*data).horizontal_rule = horizontal_rule;
            (*data).vertical_rule = vertical_rule;
        }
        Self {
            data_: Member::from_ptr(data),
        }
    }

    // cpp: layoutng_style/style/nine_piece_image.h:92
    // cpp: layoutng_style/style/nine_piece_image.cc:59-70
    pub fn MaskDefaults() -> Self {
        thread_local! {
            static DATA: OnceCell<Persistent<NinePieceImageData>> = OnceCell::new();
        }
        DATA.with(|slot| {
            let data = slot.get_or_init(|| {
                Persistent::from_ptr(MakeGarbageCollected(NinePieceImageData::default()))
            });
            let ptr = data.Get();
            unsafe {
                if (*ptr).single_owner.load(Ordering::Relaxed) {
                    (*ptr).image_slices = LengthBox::from_int(0);
                    (*ptr).fill = true;
                    (*ptr).border_slices =
                        BorderImageLengthBox::from_length(Length::Auto().clone());
                    (*ptr).single_owner.store(false, Ordering::Relaxed);
                }
            }
            Self::from_shared_data(ptr)
        })
    }

    // cpp: layoutng_style/style/nine_piece_image.h:98
    pub fn HasImage(&self) -> bool {
        !self.data().image.Get().is_null()
    }

    // cpp: layoutng_style/style/nine_piece_image.h:99
    pub fn GetImage(&self) -> *mut StyleImage {
        self.data().image.Get()
    }

    // cpp: layoutng_style/style/nine_piece_image.h:100
    pub fn SetImage(&mut self, image: *mut StyleImage) {
        self.Access().image = Member::from_ptr(image);
    }

    // cpp: layoutng_style/style/nine_piece_image.h:102
    pub fn ImageSlices(&self) -> &LengthBox {
        &self.data().image_slices
    }

    // cpp: layoutng_style/style/nine_piece_image.h:103-105
    pub fn SetImageSlices(&mut self, slices: &LengthBox) {
        self.Access().image_slices = slices.clone();
    }

    // cpp: layoutng_style/style/nine_piece_image.h:107
    pub fn Fill(&self) -> bool {
        self.data().fill
    }

    // cpp: layoutng_style/style/nine_piece_image.h:108
    pub fn SetFill(&mut self, fill: bool) {
        self.Access().fill = fill;
    }

    // cpp: layoutng_style/style/nine_piece_image.h:110-112
    pub fn BorderSlices(&self) -> &BorderImageLengthBox {
        &self.data().border_slices
    }

    // cpp: layoutng_style/style/nine_piece_image.h:113-115
    pub fn SetBorderSlices(&mut self, slices: &BorderImageLengthBox) {
        self.Access().border_slices = slices.clone();
    }

    // cpp: layoutng_style/style/nine_piece_image.h:117
    pub fn Outset(&self) -> &BorderImageLengthBox {
        &self.data().outset
    }

    // cpp: layoutng_style/style/nine_piece_image.h:118-120
    pub fn SetOutset(&mut self, outset: &BorderImageLengthBox) {
        self.Access().outset = outset.clone();
    }

    // cpp: layoutng_style/style/nine_piece_image.h:122-124
    pub fn HorizontalRule(&self) -> ENinePieceImageRule {
        self.data().horizontal_rule
    }

    // cpp: layoutng_style/style/nine_piece_image.h:125-127
    pub fn SetHorizontalRule(&mut self, rule: ENinePieceImageRule) {
        self.Access().horizontal_rule = rule;
    }

    // cpp: layoutng_style/style/nine_piece_image.h:129-131
    pub fn VerticalRule(&self) -> ENinePieceImageRule {
        self.data().vertical_rule
    }

    // cpp: layoutng_style/style/nine_piece_image.h:132-134
    pub fn SetVerticalRule(&mut self, rule: ENinePieceImageRule) {
        self.Access().vertical_rule = rule;
    }

    // cpp: layoutng_style/style/nine_piece_image.h:136-139
    pub fn CopyImageSlicesFrom(&mut self, other: &Self) {
        let data = self.Access();
        data.image_slices = other.data().image_slices.clone();
        data.fill = other.data().fill;
    }

    // cpp: layoutng_style/style/nine_piece_image.h:141-143
    pub fn CopyBorderSlicesFrom(&mut self, other: &Self) {
        self.Access().border_slices = other.data().border_slices.clone();
    }

    // cpp: layoutng_style/style/nine_piece_image.h:145-147
    pub fn CopyOutsetFrom(&mut self, other: &Self) {
        self.Access().outset = other.data().outset.clone();
    }

    // cpp: layoutng_style/style/nine_piece_image.h:149-152
    pub fn CopyRepeatFrom(&mut self, other: &Self) {
        let data = self.Access();
        data.horizontal_rule = other.data().horizontal_rule;
        data.vertical_rule = other.data().vertical_rule;
    }

    // cpp: layoutng_style/style/nine_piece_image.h:154-161
    pub fn ComputeOutset(outset_side: &BorderImageLength, border_side: i32) -> LayoutUnit {
        if outset_side.IsNumber() {
            return LayoutUnit::from_f64(outset_side.Number() * f64::from(border_side));
        }
        debug_assert!(outset_side.length().IsFixed());
        LayoutUnit::from_f32(outset_side.length().Pixels())
    }

    // cpp: layoutng_style/style/nine_piece_image.h:163
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.data_);
    }

    // cpp: layoutng_style/style/nine_piece_image.h:166-169
    fn from_shared_data(data: *mut NinePieceImageData) -> Self {
        debug_assert!(!unsafe { (*data).single_owner.load(Ordering::Relaxed) });
        Self {
            data_: Member::from_ptr(data),
        }
    }

    // cpp: layoutng_style/style/nine_piece_image.h:171-177
    fn Access(&mut self) -> &mut NinePieceImageData {
        let data = self.data_.Get();
        unsafe {
            if !(*data).single_owner.load(Ordering::Relaxed) {
                let cloned = MakeGarbageCollected((*data).clone());
                self.data_ = Member::from_ptr(cloned);
                (*cloned).single_owner.store(true, Ordering::Relaxed);
            }
            &mut *self.data_.Get()
        }
    }

    fn data(&self) -> &NinePieceImageData {
        unsafe { &*self.data_.Get() }
    }
}

// cpp: layoutng_style/style/nine_piece_image.h:80-82
impl Clone for NinePieceImage {
    fn clone(&self) -> Self {
        let data = self.data_.Get();
        unsafe { (*data).single_owner.store(false, Ordering::Relaxed) };
        Self {
            data_: Member::from_ptr(data),
        }
    }
}

// cpp: layoutng_style/style/nine_piece_image.h:85-89
impl NinePieceImage {
    pub fn copy_from(&mut self, other: &Self) {
        self.data_ = Member::from_ptr(other.data_.Get());
        unsafe {
            (*self.data_.Get())
                .single_owner
                .store(false, Ordering::Relaxed)
        };
    }
}

// cpp: layoutng_style/style/nine_piece_image.h:94-96
impl PartialEq for NinePieceImage {
    fn eq(&self, other: &Self) -> bool {
        foundation::ValuesEquivalent(&self.data_, &other.data_)
    }
}

// cpp: layoutng_style/style/nine_piece_image.h:90
// Move assignment is Rust's ordinary move assignment.

#[cfg(test)]
mod thread_root_tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    #[test]
    fn default_roots_stay_on_each_worker_heap_and_preserve_copy_on_write() {
        let barrier = Arc::new(Barrier::new(2));
        let workers: Vec<_> = (0..2)
            .map(|_| {
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    let result = std::panic::catch_unwind(|| {
                        let pointers;
                        {
                            let _heap = foundation::LayoutHeapScope::new();
                            Length::Initialize();
                            let border = NinePieceImage::new();
                            let mask = NinePieceImage::MaskDefaults();
                            assert!(!border.HasImage());
                            assert!(!border.Fill());
                            assert!(
                                border.ImageSlices()
                                    == &LengthBox::new(
                                        Length::Percent(100.0),
                                        Length::Percent(100.0),
                                        Length::Percent(100.0),
                                        Length::Percent(100.0),
                                    )
                            );
                            assert!(
                                border.BorderSlices() == &BorderImageLengthBox::from_number(1.0)
                            );
                            assert!(border.Outset() == &BorderImageLengthBox::from_number(0.0));
                            assert!(
                                border.HorizontalRule() == ENinePieceImageRule::kStretchImageRule
                            );
                            assert!(
                                border.VerticalRule() == ENinePieceImageRule::kStretchImageRule
                            );
                            assert!(mask.Fill());
                            assert!(mask.ImageSlices() == &LengthBox::from_int(0));
                            assert!(
                                mask.BorderSlices()
                                    == &BorderImageLengthBox::from_length(Length::Auto().clone())
                            );
                            pointers = (border.data_.Get() as usize, mask.data_.Get() as usize);
                            let mut changed_border = border.clone();
                            changed_border.SetFill(true);
                            changed_border.SetHorizontalRule(ENinePieceImageRule::kRepeatImageRule);
                            assert!(!border.Fill());
                            assert!(changed_border.Fill());
                            assert!(
                                border.HorizontalRule() == ENinePieceImageRule::kStretchImageRule
                            );
                            assert_ne!(changed_border.data_.Get(), border.data_.Get());
                            let mut changed_mask = mask.clone();
                            changed_mask.SetFill(false);
                            assert!(mask.Fill());
                            assert!(!changed_mask.Fill());
                            assert_ne!(changed_mask.data_.Get(), mask.data_.Get());
                        }
                        // The thread-local roots must survive this heap collection.
                        {
                            let _heap = foundation::LayoutHeapScope::new();
                            let border = NinePieceImage::new();
                            let mask = NinePieceImage::MaskDefaults();
                            assert_eq!(border.data_.Get() as usize, pointers.0);
                            assert_eq!(mask.data_.Get() as usize, pointers.1);
                            assert!(!border.Fill());
                            assert!(mask.Fill());
                            assert!(foundation::IsManagedLayoutAddress(border.data_.Get()));
                            assert!(foundation::IsManagedLayoutAddress(mask.data_.Get()));
                        }
                        pointers
                    });
                    // Retain both TLS heaps until each worker finished its
                    // checks. A failed check still reaches the barrier, so a
                    // regression reports a panic instead of hanging its peer.
                    barrier.wait();
                    match result {
                        Ok(pointers) => pointers,
                        Err(error) => std::panic::resume_unwind(error),
                    }
                })
            })
            .collect();
        let mut pointers = workers.into_iter().map(|worker| worker.join().unwrap());
        let first = pointers.next().unwrap();
        let second = pointers.next().unwrap();
        assert_ne!(
            first.0, second.0,
            "border defaults must belong to separate worker heaps"
        );
        assert_ne!(
            first.1, second.1,
            "mask defaults must belong to separate worker heaps"
        );
    }
}
