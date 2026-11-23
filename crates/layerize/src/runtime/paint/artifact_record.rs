//! PAC's PaintRecord op walk (paint_artifact_compositor.cc custom-data
//! replacement) for the concrete family. `F: LayerizeFamily` proves
//! `PaintArtifactCompositorTypes` through the blanket impl, so
//! `F::PaintRecord` normalizes to `CpuPaintRecord<F::Skia>` and the trait's
//! opaque `ops` iterator needs no boxing. The custom-data replacement is
//! family-independent (`ReplaceCustomDataInRecord`); its unit test is
//! below.
use super::*;
use crate::{
    adapter::paint_artifact_compositor::{ArtifactPaintRecord, ArtifactRecordOp},
    commit::{layerize_family::LayerizeFamily, skia_value_backend::PaintSkiaBackend},
};
impl<F: LayerizeFamily> ArtifactPaintRecord<F> for CpuPaintRecord<F::Skia> {
    fn ops(&self) -> impl Iterator<Item = ArtifactRecordOp<'_, F>> {
        use crate::adapter::display_item_list::DisplayPaintOpBuffer;
        self.buffer()
            .into_iter()
            .flat_map(|b| b.iter())
            .map(|op| match op {
                CpuPaintOpRef::PaintRecord { op, .. }
                    if op.r#type == ::paint::paint_engine::DisplayItemType::kCustomData =>
                {
                    ArtifactRecordOp::CustomData(op.custom_data_id)
                }
                CpuPaintOpRef::Converted(CpuPaintOp::CustomData(id)) => {
                    ArtifactRecordOp::CustomData(*id)
                }
                CpuPaintOpRef::Converted(CpuPaintOp::DrawRecord(r)) => {
                    ArtifactRecordOp::DrawRecord(r)
                }
                _ => ArtifactRecordOp::Other,
            })
    }
    fn ReplaceCustomData(&self, replacements: &std::collections::BTreeMap<u32, Self>) -> Self {
        ReplaceCustomDataInRecord(self, replacements)
    }
}

/// Source: paint_artifact_compositor.cc custom-data replacement over one
/// record and, recursively, its nested DrawRecord ops. It depends only on
/// the Skia backend, not on the family.
fn ReplaceCustomDataInRecord<R: PaintSkiaBackend>(
    record: &CpuPaintRecord<R>,
    replacements: &std::collections::BTreeMap<u32, CpuPaintRecord<R>>,
) -> CpuPaintRecord<R> {
    use crate::adapter::display_item_list::DisplayPaintOpBuffer;
    if replacements.is_empty() {
        return record.clone();
    }
    let Some(source) = record.buffer() else {
        return record.clone();
    };
    let mut output = CpuPaintOpBuffer::new(source.values().clone());
    for (index, op) in source.iter().enumerate() {
        let custom_data_id = match op {
            CpuPaintOpRef::PaintRecord { op, .. }
                if op.r#type == ::paint::paint_engine::DisplayItemType::kCustomData =>
            {
                Some(op.custom_data_id)
            }
            CpuPaintOpRef::Converted(CpuPaintOp::CustomData(id)) => Some(*id),
            _ => None,
        };
        if let Some(id) = custom_data_id {
            if let Some(replacement) = replacements.get(&id) {
                output.push(CpuPaintOp::DrawRecord(replacement.clone()));
            } else {
                output.push(CpuPaintOp::CustomData(id));
            }
            continue;
        }
        match op {
            CpuPaintOpRef::Converted(CpuPaintOp::DrawRecord(nested)) => {
                output.push(CpuPaintOp::DrawRecord(ReplaceCustomDataInRecord(
                    nested,
                    replacements,
                )));
            }
            _ => {
                output.push(source.RetainOpAt(index));
            }
        }
    }
    output.ReleaseAsRecord()
}

// Moved from compositor/src/runtime/paint/artifact_record.rs. The family
// impl delegates to ReplaceCustomDataInRecord, so the SkRegion backend
// exercises the replacement without the compositor's concrete family.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        adapter::display_item_list::DisplayPaintOpBuffer,
        commit::{
            main_paint_config::{CpuPaintFeatures, SkiaPaintDiagnostics},
            skia_region::SkRegion,
        },
    };
    use std::{collections::BTreeMap, sync::Arc};

    fn values() -> Arc<dyn CpuPaintValueServices<SkRegion>> {
        super::super::NewCpuMainPaintServices(
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

    /// The converted ops of a record: Some(Ok(nested)) for DrawRecord,
    /// Some(Err(id)) for CustomData, None for any other op.
    fn ops(
        record: &CpuPaintRecord<SkRegion>,
    ) -> Vec<Option<Result<&CpuPaintRecord<SkRegion>, u32>>> {
        record
            .buffer()
            .into_iter()
            .flat_map(|b| b.iter())
            .map(|op| match op {
                CpuPaintOpRef::Converted(CpuPaintOp::DrawRecord(r)) => Some(Ok(r)),
                CpuPaintOpRef::Converted(CpuPaintOp::CustomData(id)) => Some(Err(*id)),
                _ => None,
            })
            .collect()
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
        let replaced = ReplaceCustomDataInRecord(&source, &replacements);

        let outer = ops(&replaced);
        assert_eq!(outer.len(), 3);
        assert!(matches!(outer[0], Some(Ok(_))));
        assert!(matches!(outer[1], Some(Err(8))));
        let Some(Ok(nested)) = outer[2] else {
            panic!("recursive DrawRecord replacement");
        };
        assert!(matches!(ops(nested).first(), Some(Some(Ok(_)))));
    }
}
