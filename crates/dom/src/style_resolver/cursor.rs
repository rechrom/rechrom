use layoutng_assembly::internal::paint_input::ECursor;

#[allow(non_snake_case)]
pub(crate) fn ParseCursor(value: &str) -> Option<ECursor> {
    // Custom cursor images are not loaded yet. Honor their required final
    // keyword fallback, without interpreting commas inside url()/image-set().
    let value = value.rsplit(',').next()?.trim().to_ascii_lowercase();
    Some(match value.as_str() {
        "auto" => ECursor::kAuto,
        "crosshair" => ECursor::kCrosshair,
        "default" => ECursor::kDefault,
        "pointer" => ECursor::kPointer,
        "move" => ECursor::kMove,
        "vertical-text" => ECursor::kVerticalText,
        "cell" => ECursor::kCell,
        "context-menu" => ECursor::kContextMenu,
        "alias" => ECursor::kAlias,
        "progress" => ECursor::kProgress,
        "no-drop" => ECursor::kNoDrop,
        "not-allowed" => ECursor::kNotAllowed,
        "zoom-in" => ECursor::kZoomIn,
        "zoom-out" => ECursor::kZoomOut,
        "e-resize" => ECursor::kEResize,
        "ne-resize" => ECursor::kNeResize,
        "nw-resize" => ECursor::kNwResize,
        "n-resize" => ECursor::kNResize,
        "se-resize" => ECursor::kSeResize,
        "sw-resize" => ECursor::kSwResize,
        "s-resize" => ECursor::kSResize,
        "w-resize" => ECursor::kWResize,
        "ew-resize" => ECursor::kEwResize,
        "ns-resize" => ECursor::kNsResize,
        "nesw-resize" => ECursor::kNeswResize,
        "nwse-resize" => ECursor::kNwseResize,
        "col-resize" => ECursor::kColResize,
        "row-resize" => ECursor::kRowResize,
        "text" => ECursor::kText,
        "wait" => ECursor::kWait,
        "help" => ECursor::kHelp,
        "all-scroll" => ECursor::kAllScroll,
        "grab" => ECursor::kGrab,
        "grabbing" => ECursor::kGrabbing,
        "copy" => ECursor::kCopy,
        "none" => ECursor::kNone,
        _ => return None,
    })
}
