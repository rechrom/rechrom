//! PAC's PaintRecord op walk (paint_artifact_compositor.cc custom-data
//! replacement) for the concrete family. `F: LayerizeFamily` proves
//! `PaintArtifactCompositorTypes` through the blanket impl, so
//! `F::PaintRecord` normalizes to `CpuPaintRecord<F::Skia>` and the trait's
//! opaque `ops` iterator needs no boxing. The unit test stays in the
//! compositor (compositor/src/runtime/paint/artifact_record.rs), which owns
//! the concrete family.
use super::*;
use crate::{
    adapter::paint_artifact_compositor::{ArtifactPaintRecord, ArtifactRecordOp},
    commit::layerize_family::LayerizeFamily,
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
        use crate::adapter::display_item_list::DisplayPaintOpBuffer;
        if replacements.is_empty() {
            return self.clone();
        }
        let Some(source) = self.buffer() else {
            return self.clone();
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
                CpuPaintOpRef::Converted(CpuPaintOp::DrawRecord(record)) => {
                    output.push(CpuPaintOp::DrawRecord(
                        <Self as ArtifactPaintRecord<F>>::ReplaceCustomData(record, replacements),
                    ));
                }
                _ => {
                    output.push(source.RetainOpAt(index));
                }
            }
        }
        output.ReleaseAsRecord()
    }
}

