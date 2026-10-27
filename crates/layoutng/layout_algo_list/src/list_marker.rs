#![allow(non_snake_case)]

use foundation::{keywords, AtomicString, LayoutUnit};
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::layout_inside_list_marker::LayoutInsideListMarker;
use crate::layout_list_item::LayoutListItem;
use crate::layout_outside_list_marker::LayoutOutsideListMarker;

// cpp: layoutng_list/list_marker.h:70-72
pub enum ListStyleCategory {
    kNone,
    kSymbol,
    kLanguage,
    kStaticString,
}

// cpp: layoutng_list/list_marker.h:75-89
#[derive(Clone, Copy, PartialEq, Eq)]
enum MarkerTextType {
    kNotText,
    kUnresolved,
    kOrdinalValue,
    kStatic,
    kSymbolValue,
}

// cpp: layoutng_list/list_marker.h:18-25,104-104
pub struct ListMarker {
    marker_text_type_: MarkerTextType,
}

impl Default for ListMarker {
    // cpp: layoutng_list/list_marker_core.cc:17-17
    fn default() -> Self {
        Self {
            marker_text_type_: MarkerTextType::kNotText,
        }
    }
}

impl ListMarker {
    // cpp: layoutng_list/list_marker.h:96-96
    // The declaration has no definition in the supplied source tree.
    pub fn OrdinalValueChanged(&mut self, marker: &mut LayoutObject) {
        unsafe { ListMarkerOrdinalValueChanged(self, marker) }
    }

    // cpp: layoutng_list/list_marker.h:45-45,95-95
    pub fn UpdateMarkerContentIfNeeded(&mut self, marker: &mut LayoutObject) {
        unsafe { ListMarkerUpdateMarkerContentIfNeeded(self, marker) }
    }
    pub fn ListStyleTypeChanged(&mut self, marker: &mut LayoutObject) {
        unsafe { ListMarkerListStyleTypeChanged(self, marker) }
    }
    // cpp: layoutng_list/list_marker.h:27-28
    // cpp: layoutng_list/list_marker_core.cc:19-29
    pub fn Get(marker: *const LayoutObject) -> *const Self {
        if marker.is_null() {
            return std::ptr::null();
        }
        let object = unsafe { &*marker };
        if object.IsLayoutOutsideListMarker() {
            return unsafe { &*marker.cast::<LayoutOutsideListMarker>() }.Marker();
        }
        if object.IsLayoutInsideListMarker() {
            return unsafe { &*marker.cast::<LayoutInsideListMarker>() }.Marker();
        }
        std::ptr::null()
    }

    pub fn GetMut(marker: *mut LayoutObject) -> *mut Self {
        Self::Get(marker).cast_mut()
    }

    // cpp: layoutng_list/list_marker.h:30-30
    // cpp: layoutng_list/list_marker_core.cc:31-35
    pub fn MarkerFromListItem(list_item: *const LayoutObject) -> *mut LayoutObject {
        if list_item.is_null() || !unsafe { &*list_item }.IsLayoutListItem() {
            return std::ptr::null_mut();
        }
        unsafe { &*list_item.cast::<LayoutListItem>() }.Marker()
    }

    // cpp: layoutng_list/list_marker.h:32-32
    // cpp: layoutng_list/list_marker_core.cc:37-44
    pub fn ListItem(&self, marker: &LayoutObject) -> *mut LayoutObject {
        debug_assert_eq!(Self::Get(marker), self as *const Self);
        let node = marker.GetNode();
        let parent = if node.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*node }.parentNode()
        };
        let list_item = if parent.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*parent }.GetLayoutObject()
        };
        debug_assert!(!list_item.is_null());
        debug_assert!(unsafe { &*list_item }.IsListItem());
        list_item
    }

    // cpp: layoutng_list/list_marker.h:101-102
    // cpp: layoutng_list/list_marker_core.cc:46-49
    fn GetContentChild(&self, marker: &LayoutObject) -> *mut LayoutObject {
        let child = marker.SlowFirstChild();
        if !child.is_null() && unsafe { &*child }.IsLayoutTextCombine() {
            unsafe { &*child }.SlowFirstChild()
        } else {
            child
        }
    }

    // cpp: layoutng_list/list_marker.h:47-47
    // cpp: layoutng_list/list_marker_core.cc:51-57
    pub fn SymbolMarkerLayoutText(&self, marker: &LayoutObject) -> *mut LayoutObject {
        debug_assert_eq!(Self::Get(marker), self as *const Self);
        if self.marker_text_type_ != MarkerTextType::kSymbolValue {
            return std::ptr::null_mut();
        }
        self.GetContentChild(marker)
    }

    // cpp: layoutng_list/list_marker.h:93-93
    // cpp: layoutng_list/list_marker_core.cc:59-63
    fn UpdateMarkerText(&mut self, _marker: &mut LayoutObject) -> ! {
        panic!("C++ NOTREACHED: standalone generated marker text is already resolved")
    }

    // cpp: layoutng_list/list_marker.h:40-44
    pub fn UpdateMarkerTextIfNeeded(&mut self, marker: &mut LayoutObject) {
        debug_assert_eq!(Self::Get(marker), self as *const Self);
        if self.marker_text_type_ == MarkerTextType::kUnresolved {
            self.UpdateMarkerText(marker);
        }
    }

    // cpp: layoutng_list/list_marker.h:49-56
    pub fn SetStandaloneSymbolMarker(&mut self, symbol: bool) {
        self.marker_text_type_ = if symbol {
            MarkerTextType::kSymbolValue
        } else {
            MarkerTextType::kStatic
        };
    }

    // cpp: layoutng_list/list_marker.h:67-68
    // cpp: layoutng_list/list_marker_core.cc:65-76
    pub fn WidthOfSymbol(style: &ComputedStyle, list_style: &AtomicString) -> LayoutUnit {
        let font = style.GetFont();
        let font_data = if font.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*font }.PrimaryFont()
        };
        if font_data.is_null() || style.SpecifiedFontSize() == 0.0 {
            return LayoutUnit::default();
        }
        if *list_style == *keywords::kDisclosureOpen || *list_style == *keywords::kDisclosureClosed
        {
            return LayoutUnit::from_f32(style.SpecifiedFontSize() * style.EffectiveZoom() * 0.66);
        }
        LayoutUnit::from_signed(
            (unsafe { &*font_data }.GetFontMetrics().Ascent() * 2 / 3 + 1) / 2 + 2,
        )
    }

    // cpp: layoutng_list/list_marker.h:58-62
    // cpp: layoutng_list/list_marker_core.cc:78-113
    pub fn InlineMarginsForOutside(
        marker_style: &ComputedStyle,
        list_item_style: &ComputedStyle,
        marker_inline_size: LayoutUnit,
    ) -> (LayoutUnit, LayoutUnit) {
        const K_MARKER_PADDING: i32 = 7;
        let mut margin_start = LayoutUnit::default();
        let mut margin_end = LayoutUnit::default();
        if !marker_style.ContentBehavesAsNormal() {
            margin_start = -marker_inline_size;
        } else if list_item_style.GeneratesMarkerImage() {
            margin_start = -marker_inline_size - K_MARKER_PADDING;
            margin_end = LayoutUnit::from_signed(K_MARKER_PADDING);
        } else {
            let type_ptr = list_item_style.ListStyleType().Get();
            if !type_ptr.is_null() {
                let list_type = unsafe { &*type_ptr };
                let name = list_type.GetCounterStyleName();
                if list_type.IsCounterStyle()
                    && (*name == *keywords::kDisc
                        || *name == *keywords::kCircle
                        || *name == *keywords::kSquare
                        || *name == *keywords::kDisclosureOpen
                        || *name == *keywords::kDisclosureClosed)
                {
                    let font = marker_style.GetFont();
                    assert!(!font.is_null());
                    let font_data = unsafe { &*font }.PrimaryFont();
                    assert!(!font_data.is_null());
                    let offset = if *name == *keywords::kDisclosureOpen
                        || *name == *keywords::kDisclosureClosed
                    {
                        LayoutUnit::from_f32(
                            marker_style.SpecifiedFontSize() * marker_style.EffectiveZoom() * 0.66,
                        )
                    } else {
                        LayoutUnit::from_signed(
                            unsafe { &*font_data }.GetFontMetrics().Ascent() * 2 / 3,
                        )
                    };
                    margin_start = -offset - K_MARKER_PADDING - 1;
                    margin_end = offset + K_MARKER_PADDING + 1 - marker_inline_size;
                } else {
                    margin_start = -marker_inline_size;
                }
            }
        }
        debug_assert_eq!(-margin_start - margin_end, marker_inline_size);
        (margin_start, margin_end)
    }
}

unsafe extern "Rust" {
    fn ListMarkerOrdinalValueChanged(marker_data: &mut ListMarker, marker: &mut LayoutObject);
    fn ListMarkerUpdateMarkerContentIfNeeded(
        marker_data: &mut ListMarker,
        marker: &mut LayoutObject,
    );
    fn ListMarkerListStyleTypeChanged(marker_data: &mut ListMarker, marker: &mut LayoutObject);
}

// Other list_marker.h declarations have no definitions in this source tree.
// They remain blocked in the package translation status.
pub fn WidthOfSymbol(style: &ComputedStyle, list_style: &AtomicString) -> LayoutUnit {
    ListMarker::WidthOfSymbol(style, list_style)
}
