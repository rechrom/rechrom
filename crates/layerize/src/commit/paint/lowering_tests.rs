//! Lowering checks that need the main paint services (moved with cc/paint
//! out of this crate; the lowering itself is cc_paint's).
use super::*;
use crate::commit::main_paint_config::{CpuPaintFeatures, SkiaPaintDiagnostics};
use paint_model::paint_op::PaintOpType as N;
use paint_model::PaintRect;
use std::sync::Arc;

#[test]
fn lowering_retains_prepaint_canonical_filter_owner() {
    let services = crate::runtime::paint::NewCpuMainPaintServices(
        CpuPaintFeatures {
            cc_slimming: false,
            singular_transform_fix: false,
            merge_fixed_layers: false,
            merge_sticky_layers: false,
            raster_inducing_scroll: false,
            scrolling_contents_cull_rect: false,
            remove_scroll_node_workaround: false,
            selection_edge_visibility_uses_full_edge: false,
            canvas_draw_element: false,
            expensive_dchecks: false,
            paint_under_invalidation_checking: false,
            layer_merge_distance_limit: 0,
        },
        SkiaPaintDiagnostics {
            trace: false,
            verbose_level: 0,
            histogram: false,
        },
    )
    .unwrap();
    let canonical_filters =
        paint_model::property_filter_operations::PropertyFilterOperations::Ready {
            operations: paint_model::cc::paint::filter_operations::FilterOperations::new(vec![
                paint_model::cc::paint::filter_operation::FilterOperation::CreateOpacityFilter(0.5),
            ]),
            origin_tainted: false,
        };
    let mut artifact = paint::paint_engine::PaintArtifact::default();
    artifact.items = Arc::new(vec![PaintOp {
        r#type: N::kSaveLayerFilter,
        canonical_filters,
        ..PaintOp::default()
    }]);

    let lowered = LowerPaintRecord(
        paint::paint_engine::PaintRecordForArtifact(&Arc::new(artifact)),
        services.PaintValues(),
    );
    let [CpuPaintOp::Atom(source)] = lowered.as_slice() else {
        panic!("filter item must remain a native source lease");
    };
    assert_eq!(source.op().canonical_filters.len(), 1);
}

#[test]
fn equal_wavy_geometry_and_color_reuse_the_cached_tile_record() {
    let services = crate::runtime::paint::NewCpuMainPaintServices(
        CpuPaintFeatures {
            cc_slimming: false,
            singular_transform_fix: false,
            merge_fixed_layers: false,
            merge_sticky_layers: false,
            raster_inducing_scroll: false,
            scrolling_contents_cull_rect: false,
            remove_scroll_node_workaround: false,
            selection_edge_visibility_uses_full_edge: false,
            canvas_draw_element: false,
            expensive_dchecks: false,
            paint_under_invalidation_checking: false,
            layer_merge_distance_limit: 0,
        },
        SkiaPaintDiagnostics {
            trace: false,
            verbose_level: 0,
            histogram: false,
        },
    )
    .unwrap();
    let op = PaintOp {
        r#type: N::kStrokeWavyLine,
        rect: PaintRect {
            width: 40.0,
            ..PaintRect::default()
        },
        decoration_thickness: 2.0,
        ..PaintOp::default()
    };
    let mut artifact = paint::paint_engine::PaintArtifact::default();
    artifact.items = Arc::new(vec![op.clone(), op]);
    let lowered = LowerPaintRecord(
        paint::paint_engine::PaintRecordForArtifact(&Arc::new(artifact)),
        services.PaintValues(),
    );
    let records: Vec<_> = lowered
        .iter()
        .filter_map(|op| {
            let CpuPaintOp::DrawRect(_, flags) = op else {
                return None;
            };
            let CpuPaintShader::PaintRecord {
                record: CpuPaintRecord::Buffer(buffer),
                ..
            } = flags.shader.as_deref()?
            else {
                return None;
            };
            Some(buffer)
        })
        .collect();
    assert_eq!(records.len(), 2);
    assert!(Arc::ptr_eq(records[0], records[1]));
}
