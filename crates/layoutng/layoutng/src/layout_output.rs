//! Borrowed access to the engine's resident layout result. No exported tree,
//! DOM ownership or native LayoutObject handle crosses this interface.
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

#[path = "../../layout_fragment/src/native_client_rects.rs"]
mod layout_fragment_native_client_rects;
#[path = "../../layout_fragment/src/native_fragment_cursor_children.rs"]
mod layout_fragment_native_fragment_cursor_children;
#[path = "../../layout_fragment/src/native_layout_metrics.rs"]
mod layout_fragment_native_layout_metrics;
#[path = "../../layout_fragment/src/native_text_fragment_queries.rs"]
mod layout_fragment_native_text_fragment_queries;

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
pub enum NativeFragmentCursor<'a> {
    Physical {
        fragment: &'a PhysicalFragment,
        offset: PhysicalOffset,
    },
    Inline {
        item: &'a layout_fragment::fragment_item::FragmentItem,
        items: &'a layout_fragment::fragment_items::FragmentItems,
        index: usize,
        offset: PhysicalOffset,
    },
}

/// Paint cursor over the same native links. Dereferencing exposes only the
/// fragment-local data attached to the native owner; `Children` always follows
/// PhysicalFragment/FragmentItem pointers and never a copied child vector.
#[derive(Clone, Copy)]
pub struct NativePaintFragmentCursor<'a>(NativeFragmentCursor<'a>);

pub(crate) fn NativeRootForHandle(handle: &LayoutResultHandle) -> &PhysicalFragment {
    let result = unsafe { handle.NativeResult::<LayoutResult>() };
    assert!(
        !result.is_null(),
        "empty fragment handle has no native result"
    );
    unsafe { &*result }.GetPhysicalFragment()
}

impl<'a> NativePaintFragmentCursor<'a> {
    /// Enter the native fragment graph through the opaque owner supplied by
    /// the standalone `layout_fragment` package. Concrete type recovery stays
    /// inside the LayoutNG assembly adapter.
    pub fn FromHandle(handle: &'a LayoutResultHandle) -> Self {
        Self::FromRoot(NativeRootForHandle(handle))
    }

    pub fn FromRoot(fragment: &'a PhysicalFragment) -> Self {
        NativeFragmentCursor::Physical {
            fragment,
            offset: PhysicalOffset::default(),
        }
        .ForPaint()
        .expect("layout result root has no committed paint input")
    }

    pub fn Native(self) -> NativeFragmentCursor<'a> {
        self.0
    }

    pub fn Children(self) -> Vec<Self> {
        layout_fragment_native_fragment_cursor_children::native_paint_fragment_children(self)
    }

    pub fn PaintData(self) -> &'a FragmentPaintData {
        self.0
            .PaintInput()
            .expect("native fragment has no committed paint input")
    }

    pub unsafe fn PaintDataMut(self) -> &'a mut FragmentPaintData {
        unsafe { self.0.PaintInputMut() }.expect("native fragment has no committed paint input")
    }

    /// Build the final immutable glyph payload from the canonical
    /// FragmentItem/ShapeResult only when Paint records this text item. A
    /// cached drawing never calls this method, matching Blink's
    /// TextFragmentPainter ownership and avoiding a document-wide glyph
    /// projection during Layout publication.
    pub fn PaintGlyphRuns(
        self,
    ) -> Option<std::sync::Arc<Vec<layout_fragment::fragment_tree::PaintGlyphRun>>> {
        layout_fragment_native_text_fragment_queries::paint_glyph_runs(self)
    }

    /// Primary-font text origin used by caret geometry. This reads the native
    /// font metrics directly and does not allocate or enumerate glyphs.
    pub fn TextPaintBaseline(self) -> Option<f64> {
        layout_fragment_native_text_fragment_queries::text_paint_baseline(self)
    }

    /// Query the canonical FragmentItem/ShapeResult for an insertion
    /// position. Chromium keeps this data beside FragmentItem and performs
    /// the query only for editing or hit testing; it is not projected into
    /// every paint fragment during layout publication.
    pub fn TextInlinePosition(self, offset: u32) -> Option<f64> {
        layout_fragment_native_text_fragment_queries::text_inline_position(self, offset)
    }

    /// Return the physical local rectangle of a logical text range directly
    /// from FragmentItem. The result is valid only while this cursor borrows
    /// its LayoutResult owner.
    pub fn TextRangeRect(self, start: u32, end: u32) -> Option<LayoutRect> {
        layout_fragment_native_text_fragment_queries::text_range_rect(self, start, end)
    }

    /// Resolve a local physical point to a UTF-16 text offset through the
    /// native FragmentItem/ShapeResult hit-test path.
    pub fn TextOffsetForPoint(self, point: crate::internal::layout_input::Offset) -> Option<u32> {
        layout_fragment_native_text_fragment_queries::text_offset_for_point(self, point)
    }
}

impl<'a> std::ops::Deref for NativePaintFragmentCursor<'a> {
    type Target = FragmentPaintData;

    fn deref(&self) -> &Self::Target {
        self.0
            .PaintInput()
            .expect("native fragment has no committed paint input")
    }
}

impl<'a> NativeFragmentCursor<'a> {
    pub fn PaintInput(self) -> Option<&'a FragmentPaintData> {
        match self {
            Self::Physical { fragment, .. } => {
                let data = fragment.GetFragmentData();
                unsafe { data.as_ref() }.and_then(|data| data.PaintInput())
            }
            Self::Inline { items, index, .. } => items.PaintInput(index),
        }
    }

    pub unsafe fn PaintInputMut(self) -> Option<&'a mut FragmentPaintData> {
        match self {
            Self::Physical { fragment, .. } => {
                let data = fragment.GetFragmentData();
                unsafe { data.as_ref() }.and_then(|data| unsafe { data.PaintInputMut() })
            }
            Self::Inline { items, index, .. } => unsafe { items.PaintInputMut(index) },
        }
    }

    pub fn ForPaint(self) -> Option<NativePaintFragmentCursor<'a>> {
        self.PaintInput().map(|_| NativePaintFragmentCursor(self))
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

    /// Direct child links in LayoutNG paint order. The returned vector owns
    /// cursors only; every cursor still borrows the native fragment/item.
    /// This is an interim ergonomic API and is intentionally not cached as a
    /// second tree.
    pub fn Children(self) -> Vec<Self> {
        layout_fragment_native_fragment_cursor_children::native_fragment_children(self)
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
    pub fn NativeRoot(self) -> Option<NativeFragmentCursor<'a>> {
        let root = DynamicTo::<LayoutBox>(self.engine.tree_.as_ref()?.Root() as *const _);
        let root = unsafe { root.as_ref() }?;
        let result = root.GetLayoutResult(0);
        let result = unsafe { result.as_ref() }?;
        Some(NativeFragmentCursor::Physical {
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
        layout_fragment_native_client_rects::client_rects(self)
    }
    pub fn ScrollRange(&self) -> LayoutScrollRange {
        layout_fragment_native_layout_metrics::scroll_range(self)
    }
    pub fn Metric(&self, metric: LayoutMetric) -> Option<f64> {
        layout_fragment_native_layout_metrics::metric(self, metric)
    }
}
