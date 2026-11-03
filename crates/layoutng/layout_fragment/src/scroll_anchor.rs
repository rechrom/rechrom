#![allow(non_snake_case)]

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use foundation::{gfx, DynamicTo, EOverflowAnchor, MinimumValueForLength};

use super::super::layout_box::LayoutBox;
use super::super::layout_box_model_object::LayoutBoxModelObject;
use super::super::layout_object::{AncestorSkipInfo, LayoutObject};
use super::super::layout_scrollable_area::PaintLayerScrollableArea;
use super::super::layout_text::LayoutText;
use super::super::map_coordinates_flags::MapCoordinatesFlags;
use super::super::scroll_types::ScrollOffset;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WalkStatus {
    Skip,
    Constrain,
    Continue,
    Return,
}

impl WalkStatus {
    fn viable(self) -> bool {
        matches!(self, Self::Constrain | Self::Return)
    }
}

// Chromium queues ScrollAnchor::Adjust on LocalFrameView. LayoutNG is the
// frame lifecycle owner in this package, so retain the same enqueue/adjust
// boundary in the active layout pass instead of walking every scroll owner.
thread_local! {
    static ADJUSTMENT_QUEUE: RefCell<Option<Vec<*mut PaintLayerScrollableArea>>> = const { RefCell::new(None) };
}

pub struct ScrollAnchoringAdjustmentScope {
    previous_: Option<Vec<*mut PaintLayerScrollableArea>>,
    _thread_bound: PhantomData<Rc<()>>,
}

impl ScrollAnchoringAdjustmentScope {
    pub fn new() -> Self {
        let previous = ADJUSTMENT_QUEUE.with(|queue| queue.borrow_mut().replace(Vec::new()));
        Self {
            previous_: previous,
            _thread_bound: PhantomData,
        }
    }

    pub fn Perform(&mut self) {
        let pending = ADJUSTMENT_QUEUE.with(|queue| {
            queue
                .borrow_mut()
                .as_mut()
                .map(std::mem::take)
                .unwrap_or_default()
        });
        for area in pending {
            if !area.is_null() {
                unsafe { &mut *area }.AdjustScrollAnchor();
            }
        }
    }

    fn Enqueue(area: *mut PaintLayerScrollableArea) {
        ADJUSTMENT_QUEUE.with(|queue| {
            let mut queue = queue.borrow_mut();
            let Some(queue) = queue.as_mut() else {
                return;
            };
            if !queue.contains(&area) {
                queue.push(area);
            }
        });
    }
}

impl Drop for ScrollAnchoringAdjustmentScope {
    fn drop(&mut self) {
        ADJUSTMENT_QUEUE.with(|queue| {
            *queue.borrow_mut() = self.previous_.take();
        });
    }
}

#[derive(Default)]
pub struct ScrollAnchor {
    anchor_object_: *mut LayoutObject,
    saved_block_offset_: i32,
    queued_: bool,
    disabling_style_changed_: bool,
}

impl ScrollAnchor {
    fn ClearSelf(&mut self) {
        let object = std::mem::replace(&mut self.anchor_object_, std::ptr::null_mut());
        if !object.is_null() {
            unsafe { &mut *object }.MaybeClearIsScrollAnchorObject();
        }
    }

    pub fn RefersTo(&self, object: *const LayoutObject) -> bool {
        self.anchor_object_ == object.cast_mut()
    }

    pub fn NotifyRemoved(&mut self, object: *mut LayoutObject) {
        if self.anchor_object_ == object {
            self.ClearSelf();
        }
    }

    pub fn Clear(&mut self) {
        self.ClearSelf();
    }

    pub fn Dispose(&mut self) {
        self.queued_ = false;
        self.Clear();
    }

    fn RelativeBounds(
        area: &PaintLayerScrollableArea,
        object: &LayoutObject,
    ) -> Option<gfx::RectF> {
        let box_ = DynamicTo::<LayoutBox>(object as *const LayoutObject as *mut LayoutObject);
        let local = if !box_.is_null() {
            let box_ = unsafe { &*box_ };
            let mut local = box_.PhysicalBorderBoxRect();
            // Source: core/layout/scroll_anchor.cc:132-140. An unclipped box's
            // descendants and floats are part of the candidate bounds.
            if !object.ShouldClipOverflowAlongEitherAxis() {
                local.ShiftBottomEdgeTo(std::cmp::max(
                    local.Bottom(),
                    box_.ScrollableOverflowRect().Bottom(),
                ));
            }
            local
        } else {
            let text = DynamicTo::<LayoutText>(object as *const LayoutObject as *mut LayoutObject);
            if text.is_null() {
                return None;
            }
            // Source: core/layout/scroll_anchor.cc:141-149. Text is a real
            // priority in Chromium's recursive anchor search; skipping it can
            // retain a much larger ancestor whose edge moves differently.
            unsafe { &*text }.PhysicalLinesBoundingBox()
        };
        // PaintLayerScrollableArea::LocalToVisibleContentQuad delegates to
        // LocalToAncestorQuad with this scroller box. Keep the floating quad
        // through BoundingBox, as Chromium does; LocalToAncestorRect encloses
        // into integer LayoutUnits too early and can change the chosen edge.
        Some(
            object
                .LocalToAncestorQuad(
                    &gfx::QuadF::from(gfx::RectF::from(local)),
                    area.GetLayoutBox().cast::<LayoutBoxModelObject>(),
                    MapCoordinatesFlags::default(),
                )
                .BoundingBox(),
        )
    }

    fn VisibleRect(area: &PaintLayerScrollableArea) -> gfx::RectF {
        // Source: core/layout/scroll_anchor.cc:340-350. Scroll padding narrows
        // the visible candidate area; using the raw clip can select an object
        // Chromium deliberately considers outside the anchor viewport.
        let box_ = unsafe { &*area.GetLayoutBox() };
        let mut visible = box_.OverflowClipRect();
        let style = box_.StyleRef();
        visible.ContractEdges(
            MinimumValueForLength(style.ScrollPaddingTop(), visible.Height()),
            MinimumValueForLength(style.ScrollPaddingRight(), visible.Width()),
            MinimumValueForLength(style.ScrollPaddingBottom(), visible.Height()),
            MinimumValueForLength(style.ScrollPaddingLeft(), visible.Width()),
        );
        gfx::RectF::from(visible)
    }

    fn Contains(outer: &gfx::RectF, inner: &gfx::RectF) -> bool {
        inner.x() >= outer.x()
            && inner.y() >= outer.y()
            && inner.right() <= outer.right()
            && inner.bottom() <= outer.bottom()
    }

    fn Intersects(a: &gfx::RectF, b: &gfx::RectF) -> bool {
        a.x() < b.right() && b.x() < a.right() && a.y() < b.bottom() && b.y() < a.bottom()
    }

    fn Examine(area: &PaintLayerScrollableArea, candidate: &LayoutObject) -> WalkStatus {
        if candidate as *const LayoutObject as *mut LayoutObject
            == area.GetLayoutBox().cast::<LayoutObject>()
        {
            return WalkStatus::Continue;
        }
        if candidate.StyleRef().OverflowAnchor() == EOverflowAnchor::kNone {
            return WalkStatus::Skip;
        }
        if candidate.IsLayoutInline() || candidate.IsAnonymous() {
            return WalkStatus::Continue;
        }
        if (!candidate.IsText() && !candidate.IsBox())
            || !Self::CandidateMayMoveWithScroller(area, candidate)
        {
            return WalkStatus::Skip;
        }
        let Some(candidate_rect) = Self::RelativeBounds(area, candidate) else {
            return WalkStatus::Skip;
        };
        let visible = Self::VisibleRect(area);
        if candidate_rect.width() <= 0.0
            || candidate_rect.height() <= 0.0
            || !Self::Intersects(&visible, &candidate_rect)
        {
            return WalkStatus::Skip;
        }
        if Self::Contains(&visible, &candidate_rect) {
            WalkStatus::Return
        } else {
            WalkStatus::Constrain
        }
    }

    // Source: core/layout/scroll_anchor.cc:98-108. An out-of-flow candidate
    // whose containing-block chain skips this scroller cannot preserve this
    // scroller's visible content.
    fn CandidateMayMoveWithScroller(
        area: &PaintLayerScrollableArea,
        candidate: &LayoutObject,
    ) -> bool {
        if candidate.IsFixedPositioned() || candidate.StyleRef().HasStickyConstrainedPosition() {
            return false;
        }
        let scroller = area.GetLayoutBox().cast::<LayoutObject>();
        let mut skip_info = AncestorSkipInfo::new(scroller);
        candidate.ContainerWithSkipInfo(&mut skip_info);
        !skip_info.AncestorSkipped()
    }

    fn FindAnchorRecursive(
        &mut self,
        area: &PaintLayerScrollableArea,
        candidate: *mut LayoutObject,
    ) -> WalkStatus {
        if candidate.is_null() || !unsafe { &*candidate }.EverHadLayout() {
            return WalkStatus::Skip;
        }
        let status = Self::Examine(area, unsafe { &*candidate });
        if status.viable() {
            self.anchor_object_ = candidate;
        }
        if matches!(status, WalkStatus::Return | WalkStatus::Skip) {
            return status;
        }
        let mut child = unsafe { &*candidate }.SlowFirstChild();
        while !child.is_null() {
            let child_status = self.FindAnchorRecursive(area, child);
            if child_status.viable() {
                return child_status;
            }
            child = unsafe { &*child }.NextSibling();
        }
        status
    }

    fn BlockOffset(area: &PaintLayerScrollableArea, object: &LayoutObject) -> Option<i32> {
        let rect = Self::RelativeBounds(area, object)?;
        Some(
            if unsafe { &*area.GetLayoutBox() }.IsHorizontalWritingMode() {
                rect.y().round() as i32
            } else {
                rect.x().round() as i32
            },
        )
    }

    fn ComputeDisablingStyleChanged(
        area: &PaintLayerScrollableArea,
        mut object: *mut LayoutObject,
    ) -> bool {
        let scroller = area.GetLayoutBox().cast::<LayoutObject>();
        while !object.is_null() {
            let current = unsafe { &*object };
            if current.ScrollAnchorDisablingStyleChanged() {
                return true;
            }
            if object == scroller {
                return false;
            }
            object = current.Parent();
        }
        false
    }

    pub fn NotifyBeforeLayout(&mut self, area: &PaintLayerScrollableArea) {
        if self.queued_ {
            self.disabling_style_changed_ |=
                Self::ComputeDisablingStyleChanged(area, self.anchor_object_);
            return;
        }
        let scroll = area.GetScrollOffset();
        let block_offset = if unsafe { &*area.GetLayoutBox() }.IsHorizontalWritingMode() {
            scroll.y()
        } else {
            scroll.x()
        };
        if block_offset == 0.0 {
            self.ClearSelf();
            return;
        }
        if self.anchor_object_.is_null() {
            self.FindAnchorRecursive(area, area.GetLayoutBox().cast::<LayoutObject>());
            if self.anchor_object_.is_null() {
                return;
            }
            let Some(saved) = Self::BlockOffset(area, unsafe { &*self.anchor_object_ }) else {
                self.anchor_object_ = std::ptr::null_mut();
                return;
            };
            self.saved_block_offset_ = saved;
            unsafe { &mut *self.anchor_object_ }.SetIsScrollAnchorObject();
            let node = unsafe { &*self.anchor_object_ }.GetNode();
            browser_tracing::instant(
                "layout",
                "ScrollAnchor.Select",
                &[
                    (
                        "anchor_id",
                        if node.is_null() {
                            0.0
                        } else {
                            unsafe { &*node }.InputId() as f64
                        },
                    ),
                    ("saved_block_offset", saved as f64),
                ],
            );
        }
        self.disabling_style_changed_ =
            Self::ComputeDisablingStyleChanged(area, self.anchor_object_);
        self.queued_ = true;
        ScrollAnchoringAdjustmentScope::Enqueue(
            area as *const PaintLayerScrollableArea as *mut PaintLayerScrollableArea,
        );
    }

    pub fn Adjust(&mut self, area: &mut PaintLayerScrollableArea) {
        if !self.queued_ {
            return;
        }
        self.queued_ = false;
        let object = self.anchor_object_;
        if object.is_null() {
            return;
        }
        let Some(current) = Self::BlockOffset(area, unsafe { &*object }) else {
            return;
        };
        let delta = current - self.saved_block_offset_;
        if delta == 0 {
            return;
        }
        if self.disabling_style_changed_ {
            let node = unsafe { &*object }.GetNode();
            browser_tracing::instant(
                "layout",
                "ScrollAnchor.Suppress",
                &[
                    ("delta", delta as f64),
                    (
                        "anchor_id",
                        if node.is_null() {
                            0.0
                        } else {
                            unsafe { &*node }.InputId() as f64
                        },
                    ),
                ],
            );
            self.ClearSelf();
            return;
        }
        let old = area.GetScrollOffset();
        let requested = if unsafe { &*area.GetLayoutBox() }.IsHorizontalWritingMode() {
            ScrollOffset::new(old.x(), old.y() + delta as f32)
        } else {
            ScrollOffset::new(old.x() + delta as f32, old.y())
        };
        let adjusted = area.ClampScrollOffset(&requested);
        area.SetScrollOffsetFromAnchoring(adjusted);
        let node = unsafe { &*object }.GetNode();
        browser_tracing::instant(
            "layout",
            "ScrollAnchor.Adjust",
            &[
                ("delta", delta as f64),
                (
                    "anchor_id",
                    if node.is_null() {
                        0.0
                    } else {
                        unsafe { &*node }.InputId() as f64
                    },
                ),
            ],
        );
    }
}
