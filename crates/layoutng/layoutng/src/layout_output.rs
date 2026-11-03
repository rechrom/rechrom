//! Borrowed access to the engine's resident layout result. No exported tree,
//! DOM ownership or source LayoutObject handle crosses this interface.
use super::*;
use crate::internal::{
    layout_box_model_object::LayoutBoxModelObject, layout_inline::LayoutInline,
    layout_node_metadata::Element,
};
use foundation::{gfx, PhysicalOffset, PhysicalRect, PhysicalSize};
use layout_fragment::fragment_tree::{FragmentPaintData, LayoutResultHandle};
use layout_fragment::layout_result::LayoutResult;
use layout_fragment::physical_box_fragment::PhysicalBoxFragment;
use layout_fragment::physical_fragment::PhysicalFragment;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_style::style::style_transform::{
    ApplyIndependentTransformProperties, ApplyMotionPath, ApplyTransformOperations,
    ApplyTransformOrigin,
};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::sync::Arc;

/// Opaque lifetime lease for the canonical per-result Paint sidecar. It
/// carries no Layout API and exposes no storage to Paint.
#[derive(Clone)]
pub struct PaintFragmentLease(Arc<dyn std::any::Any>);

impl PaintFragmentLease {
    pub fn PtrEq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[path = "../../layout_fragment/src/fragment_client_rect_queries.rs"]
mod layout_fragment_fragment_client_rect_queries;
#[path = "../../layout_fragment/src/fragment_cursor_children.rs"]
mod layout_fragment_fragment_cursor_children;
#[path = "../../layout_fragment/src/layout_metrics.rs"]
mod layout_fragment_layout_metrics;
#[path = "../../layout_fragment/src/text_fragment_queries.rs"]
mod layout_fragment_text_fragment_queries;

#[derive(Clone, Copy)]
pub struct LayoutOutput<'a> {
    pub(super) engine: &'a LayoutEngine,
}
pub struct LayoutNode<'a> {
    output: LayoutOutput<'a>,
    object: &'a LayoutObject,
}

/// Pointer-preserving cursor over LayoutNG's canonical fragment graph. It
/// carries no projected geometry or owned descendants. Inline cursors point
/// directly into FragmentItems; physical cursors point directly at the
/// PhysicalFragment retained by LayoutResult.
#[derive(Clone, Copy)]
pub enum FragmentCursor<'a> {
    Physical {
        fragment: &'a PhysicalFragment,
        offset: PhysicalOffset,
    },
    Inline {
        item: &'a layout_fragment::fragment_item::FragmentItem,
        items: &'a layout_fragment::fragment_items::FragmentItems,
        container: &'a PhysicalBoxFragment,
        index: usize,
        offset: PhysicalOffset,
    },
}

/// Paint cursor over the same source links. Dereferencing resolves result-local
/// Paint data by source owner identity; `Children` always follows
/// PhysicalFragment/FragmentItem pointers and never a copied child vector.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum PaintFragmentOwner {
    Physical(*const PhysicalFragment),
    Inline(*const layout_fragment::fragment_items::FragmentItems, usize),
}

/// Mutable Paint state belongs to a published LayoutResult, not to a possibly
/// shared PhysicalFragment. Geometry and child topology remain canonical and
/// are always read through `FragmentCursor`.
pub(crate) struct LayoutPaintState {
    inputs: UnsafeCell<HashMap<PaintFragmentOwner, Box<FragmentPaintData>>>,
}

impl LayoutPaintState {
    pub(crate) fn New() -> Arc<Self> {
        Arc::new(Self {
            inputs: UnsafeCell::new(HashMap::new()),
        })
    }

    pub(crate) fn SetPhysical(&self, fragment: &PhysicalFragment, mut input: FragmentPaintData) {
        input.inline_geometry.clear();
        let inputs = unsafe { &mut *self.inputs.get() };
        let owner = PaintFragmentOwner::Physical(fragment as *const PhysicalFragment);
        // Keep a stable sidecar allocation for targeted updates to this one
        // published handle. A complete publication owns a new LayoutPaintState.
        if let Some(current) = inputs.get_mut(&owner) {
            **current = input;
        } else {
            inputs.insert(owner, Box::new(input));
        }
    }

    pub(crate) fn SetInline(
        &self,
        items: &layout_fragment::fragment_items::FragmentItems,
        index: usize,
        mut input: FragmentPaintData,
    ) {
        input.inline_geometry.clear();
        let inputs = unsafe { &mut *self.inputs.get() };
        let owner = PaintFragmentOwner::Inline(items as *const _, index);
        // FragmentItems are immutable for the lifetime of this published
        // handle; preserve the sidecar allocation across targeted updates.
        if let Some(current) = inputs.get_mut(&owner) {
            **current = input;
        } else {
            inputs.insert(owner, Box::new(input));
        }
    }

    fn Get(&self, owner: PaintFragmentOwner) -> Option<&FragmentPaintData> {
        unsafe { &*self.inputs.get() }.get(&owner).map(Box::as_ref)
    }

    unsafe fn GetMut(&self, owner: PaintFragmentOwner) -> Option<&mut FragmentPaintData> {
        unsafe { &mut *self.inputs.get() }
            .get_mut(&owner)
            .map(Box::as_mut)
    }
}

#[derive(Clone, Copy)]
pub struct PaintFragmentCursor<'a> {
    source: FragmentCursor<'a>,
    state: Option<&'a LayoutPaintState>,
}

pub(crate) fn RootForHandle(handle: &LayoutResultHandle) -> &PhysicalFragment {
    let result = unsafe { handle.LayoutResult::<LayoutResult>() };
    assert!(
        !result.is_null(),
        "empty fragment handle has no source result"
    );
    unsafe { &*result }.GetPhysicalFragment()
}

impl<'a> PaintFragmentCursor<'a> {
    /// Enter the source fragment graph through the opaque owner supplied by
    /// the standalone `layout_fragment` package. Concrete type recovery stays
    /// inside the LayoutNG assembly adapter.
    pub fn FromHandle(handle: &'a LayoutResultHandle) -> Self {
        let source = FragmentCursor::Physical {
            fragment: RootForHandle(handle),
            offset: PhysicalOffset::default(),
        };
        let state = Some(
            handle
                .PaintState::<LayoutPaintState>()
                .expect("layout result handle has no committed paint state"),
        );
        let cursor = Self { source, state };
        assert!(
            cursor.PaintDataOption().is_some(),
            "layout result root has no committed paint input"
        );
        cursor
    }

    pub fn Cursor(self) -> FragmentCursor<'a> {
        self.source
    }

    pub fn Children(self) -> impl Iterator<Item = Self> + 'a {
        layout_fragment_fragment_cursor_children::paint_fragment_children(self)
    }

    pub fn PaintData(self) -> &'a FragmentPaintData {
        self.PaintDataOption()
            .expect("source fragment has no committed paint input")
    }

    pub unsafe fn PaintDataMut(self) -> &'a mut FragmentPaintData {
        unsafe {
            self.state
                .expect("paint cursor has no result state")
                .GetMut(self.source.PaintOwner())
        }
        .expect("source fragment has no committed paint input")
    }

    fn PaintDataOption(self) -> Option<&'a FragmentPaintData> {
        self.state?.Get(self.source.PaintOwner())
    }

    /// Build the final immutable glyph payload from the canonical
    /// FragmentItem/ShapeResult only when Paint records this text item. A
    /// cached drawing never calls this method, matching Blink's
    /// TextFragmentPainter ownership and avoiding a document-wide glyph
    /// projection during Layout publication.
    pub fn PaintGlyphRuns(
        self,
    ) -> Option<std::sync::Arc<Vec<layout_fragment::fragment_tree::PaintGlyphRun>>> {
        layout_fragment_text_fragment_queries::paint_glyph_runs(self)
    }

    /// Primary-font text origin used by caret geometry. This reads the source
    /// font metrics directly and does not allocate or enumerate glyphs.
    pub fn TextPaintBaseline(self) -> Option<f64> {
        layout_fragment_text_fragment_queries::text_paint_baseline(self)
    }

    /// Query the canonical FragmentItem/ShapeResult for an insertion
    /// position. Chromium keeps this data beside FragmentItem and performs
    /// the query only for editing or hit testing; it is not projected into
    /// every paint fragment during layout publication.
    pub fn TextInlinePosition(self, offset: u32) -> Option<f64> {
        layout_fragment_text_fragment_queries::text_inline_position(self, offset)
    }

    /// Measure the next grapheme with the source effective style. This is the
    /// width source used by Chromium's block and underscore caret geometry.
    pub fn TextNextCharacterWidth(self, offset: u32) -> Option<f64> {
        layout_fragment_text_fragment_queries::text_next_character_width(self, offset)
    }

    /// Return the physical local rectangle of a logical text range directly
    /// from FragmentItem. The result is valid only while this cursor borrows
    /// its LayoutResult owner.
    pub fn TextRangeRect(self, start: u32, end: u32) -> Option<LayoutRect> {
        layout_fragment_text_fragment_queries::text_range_rect(self, start, end)
    }

    /// Return Chromium's complete local selection rect, including containing
    /// line block expansion and an explicitly selected soft line break.
    pub fn TextSelectionRangeRect(
        self,
        start: u32,
        end: u32,
        select_soft_line_break: bool,
    ) -> Option<LayoutRect> {
        layout_fragment_text_fragment_queries::text_selection_range_rect(
            self,
            start,
            end,
            select_soft_line_break,
        )
    }

    /// Return HighlightPainter's line-relative local range rectangle. This is
    /// deliberately distinct from the physical selection-background rect:
    /// Paint uses it for selected glyph/decorations clipping after applying
    /// the writing-mode rotation.
    pub fn TextHighlightRangeRect(self, start: u32, end: u32) -> Option<LayoutRect> {
        layout_fragment_text_fragment_queries::text_highlight_range_rect(self, start, end)
    }

    /// Resolve a local physical point to a UTF-16 text offset through the
    /// source FragmentItem/ShapeResult hit-test path.
    pub fn TextOffsetForPoint(self, point: crate::internal::layout_input::Offset) -> Option<u32> {
        layout_fragment_text_fragment_queries::text_offset_for_point(self, point)
    }
}

pub(crate) fn PaintLeaseForHandle(handle: &LayoutResultHandle) -> PaintFragmentLease {
    PaintFragmentLease(
        handle
            .PaintStateLease()
            .expect("layout result handle has no committed paint state"),
    )
}

impl<'a> std::ops::Deref for PaintFragmentCursor<'a> {
    type Target = FragmentPaintData;

    fn deref(&self) -> &Self::Target {
        self.PaintDataOption()
            .expect("source fragment has no committed paint input")
    }
}

impl<'a> FragmentCursor<'a> {
    fn PaintOwner(self) -> PaintFragmentOwner {
        match self {
            Self::Physical { fragment, .. } => {
                PaintFragmentOwner::Physical(fragment as *const PhysicalFragment)
            }
            Self::Inline { items, index, .. } => {
                PaintFragmentOwner::Inline(items as *const _, index)
            }
        }
    }

    pub fn PhysicalFragment(self) -> Option<&'a PhysicalFragment> {
        match self {
            Self::Physical { fragment, .. } => Some(fragment),
            Self::Inline { item, .. } => unsafe { item.BoxFragment().as_ref() }
                .map(|fragment| fragment as &PhysicalBoxFragment as &PhysicalFragment),
        }
    }

    pub fn InlineItem(self) -> Option<&'a layout_fragment::fragment_item::FragmentItem> {
        match self {
            Self::Physical { .. } => None,
            Self::Inline { item, .. } => Some(item),
        }
    }

    pub fn Offset(self) -> PhysicalOffset {
        match self {
            Self::Physical { offset, .. } | Self::Inline { offset, .. } => offset,
        }
    }

    pub fn Size(self) -> PhysicalSize {
        match self {
            Self::Physical { fragment, .. } => fragment.Size(),
            Self::Inline { item, .. } => item.Size(),
        }
    }

    pub fn GetLayoutObject(self) -> *const LayoutObject {
        match self {
            Self::Physical { fragment, .. } => fragment.GetLayoutObject(),
            Self::Inline { item, .. } => item.GetLayoutObject(),
        }
    }

    /// Direct child links in LayoutNG paint order. Iteration follows the
    /// source PhysicalFragment/FragmentItem storage without materializing a
    /// parallel child vector.
    pub fn Children(self) -> impl Iterator<Item = Self> + 'a {
        layout_fragment_fragment_cursor_children::fragment_children(self)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutScrollRange {
    pub minimum: crate::internal::layout_input::Offset,
    pub maximum: crate::internal::layout_input::Offset,
}
#[derive(Clone, Copy, Debug)]
pub enum LayoutMetric {
    OffsetWidth,
    OffsetHeight,
    OffsetLeft,
    OffsetTop,
    ClientWidth,
    ClientHeight,
    ClientLeft,
    ClientTop,
    ScrollWidth,
    ScrollHeight,
    ContentWidth,
    ContentHeight,
    BorderBoxWidth,
    BorderBoxHeight,
    MinimumScrollLeft,
    MaximumScrollLeft,
    MinimumScrollTop,
    MaximumScrollTop,
}
impl TryFrom<&str> for LayoutMetric {
    type Error = ();
    fn try_from(name: &str) -> Result<Self, ()> {
        use LayoutMetric::*;
        Ok(match name {
            "offsetWidth" => OffsetWidth,
            "offsetHeight" => OffsetHeight,
            "offsetLeft" => OffsetLeft,
            "offsetTop" => OffsetTop,
            "clientWidth" => ClientWidth,
            "clientHeight" => ClientHeight,
            "clientLeft" => ClientLeft,
            "clientTop" => ClientTop,
            "scrollWidth" => ScrollWidth,
            "scrollHeight" => ScrollHeight,
            "contentWidth" => ContentWidth,
            "contentHeight" => ContentHeight,
            "borderBoxWidth" => BorderBoxWidth,
            "borderBoxHeight" => BorderBoxHeight,
            "minimumScrollLeft" => MinimumScrollLeft,
            "maximumScrollLeft" => MaximumScrollLeft,
            "minimumScrollTop" => MinimumScrollTop,
            "maximumScrollTop" => MaximumScrollTop,
            _ => return Err(()),
        })
    }
}
impl<'a> LayoutOutput<'a> {
    pub fn FragmentRoot(self) -> Option<FragmentCursor<'a>> {
        let root = DynamicTo::<LayoutBox>(self.engine.tree_.as_ref()?.Root() as *const _);
        let root = unsafe { root.as_ref() }?;
        let result = root.GetLayoutResult(0);
        let result = unsafe { result.as_ref() }?;
        Some(FragmentCursor::Physical {
            fragment: result.GetPhysicalFragment(),
            offset: PhysicalOffset::default(),
        })
    }

    pub fn Node(self, id: u64) -> Option<LayoutNode<'a>> {
        Some(LayoutNode {
            object: self.engine.tree_.as_ref()?.OutputNode(id)?,
            output: self,
        })
    }
    pub fn Root(self) -> LayoutNode<'a> {
        LayoutNode {
            object: self.engine.tree_.as_ref().unwrap().Root(),
            output: self,
        }
    }
    pub fn ScrollOffsets(self) -> Vec<(u64, crate::internal::layout_input::Offset)> {
        self.engine.tree_.as_ref().unwrap().OutputScrollOffsets()
    }
}
impl LayoutNode<'_> {
    pub fn PaintClientId(&self) -> u64 {
        // LayoutObject's base-first representation retains DisplayItemClient.
        unsafe { &*(self.object as *const _ as *const foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient) }.Id() as u64
    }
    pub fn ClientRects(&self) -> Vec<LayoutRect> {
        layout_fragment_fragment_client_rect_queries::client_rects(self)
    }
    pub fn ScrollRange(&self) -> LayoutScrollRange {
        layout_fragment_layout_metrics::scroll_range(self)
    }
    pub fn Metric(&self, metric: LayoutMetric) -> Option<f64> {
        layout_fragment_layout_metrics::metric(self, metric)
    }
}
