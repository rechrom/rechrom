//! Typed performance output for one Paint publication.
//!
//! The facade aggregates package-owned counters without reinterpreting them.
//! Trace categories use the same package names, so a slow frame can first be
//! assigned to a stage and then inspected with that stage's counters.

#[derive(Clone, Copy, Debug, Default)]
pub struct PaintPerformanceSnapshot {
    pub prepaint: prepaint::PrePaintUpdateStats,
    pub walk: paint_walk::PaintWalkMetrics,
    pub reuse: paint_reuse::PaintReuseMetrics,
    pub record: paint_record::PaintRecordMetrics,
    pub chunk: paint_chunk::PaintChunkMetrics,
    pub commit: paint_commit::PaintCommitMetrics,
}
