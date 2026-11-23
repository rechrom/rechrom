//! Unit test of PAC's PaintRecord op walk (layerize
//! runtime/paint/artifact_record.rs, a blanket impl over LayerizeFamily) on
//! the concrete CPU family, which only the compositor owns.
#[cfg(test)]
use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        adapter::{
            display_item_list::DisplayPaintOpBuffer,
            paint_artifact_compositor::{ArtifactPaintRecord, ArtifactRecordOp},
        },
        runtime::{
            paint::{CpuPaintOp, CpuPaintOpBuffer, CpuPaintRecord, SkiaPaintDiagnostics},
            paint_services::{CpuPaintFeatures, CpuMainPaintServices},
            skia::SkRegion,
            types::CpuCompositorTypes,
        },
    };
    use std::{collections::BTreeMap, sync::Arc};

    fn values() -> Arc<dyn CpuPaintValueServices<SkRegion>> {
        CpuMainPaintServices::new_cpu(
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
        .expect("CPU Paint services")
        .PaintValues()
    }

    fn ops(
        record: &CpuPaintRecord<SkRegion>,
    ) -> impl Iterator<Item = ArtifactRecordOp<'_, CpuCompositorTypes<SkRegion>>> {
        <CpuPaintRecord<SkRegion> as ArtifactPaintRecord<CpuCompositorTypes<SkRegion>>>::ops(record)
    }

    #[test]
    fn replaces_custom_data_in_outer_and_nested_records() {
        let values = values();
        let mut replacement = CpuPaintOpBuffer::new(values.clone());
        replacement.push(CpuPaintOp::Noop);
        let replacement = replacement.ReleaseAsRecord();

        let mut nested = CpuPaintOpBuffer::new(values.clone());
        nested.push(CpuPaintOp::CustomData(7));
        let nested = nested.ReleaseAsRecord();

        let mut source = CpuPaintOpBuffer::new(values);
        source.push(CpuPaintOp::CustomData(7));
        source.push(CpuPaintOp::CustomData(8));
        source.push(CpuPaintOp::DrawRecord(nested));
        let source = source.ReleaseAsRecord();

        let replacements = BTreeMap::from([(7, replacement)]);
        let replaced = <CpuPaintRecord<SkRegion> as ArtifactPaintRecord<
            CpuCompositorTypes<SkRegion>,
        >>::ReplaceCustomData(&source, &replacements);

        let mut outer_ops = ops(&replaced);
        assert!(matches!(
            outer_ops.next(),
            Some(ArtifactRecordOp::DrawRecord(_))
        ));
        assert!(matches!(
            outer_ops.next(),
            Some(ArtifactRecordOp::CustomData(8))
        ));
        let nested = match outer_ops.next() {
            Some(ArtifactRecordOp::DrawRecord(record)) => record,
            _ => panic!("recursive DrawRecord replacement"),
        };
        assert!(outer_ops.next().is_none());
        assert!(matches!(
            ops(nested).next(),
            Some(ArtifactRecordOp::DrawRecord(_))
        ));
    }
}
