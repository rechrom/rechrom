use paint_model::PaintRect;

/// Source: `region_capture_data.h:18-31`. The sorted Chromium flat_map is kept
/// as a compact vector because capture ids are already unique at insertion.
pub type PaintRegionCaptureData = Vec<([u64; 2], PaintRect)>;

/// Source: `region_capture_data.cc:12-27`.
pub fn RegionCaptureDataToString(data: &PaintRegionCaptureData) -> String {
    let entries = data
        .iter()
        .map(|(id, rect)| format!("{{{:?}: {:?}}}", id, rect))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{entries}}}")
}
