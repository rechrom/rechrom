#![allow(non_snake_case)]

//! Frame-aligned CSS animation sample routing.
//!
//! The engine owns animation-frame ordering and paint-invalidation
//! classification. It does not read a clock, mutate a document, run style or
//! paint, or know which thread hosts those engines.

use cssom::CSSDeclaration;
use std::collections::BTreeSet;
use std::time::Instant;

#[derive(Clone, Default)]
pub struct AnimationStyleSample {
    pub node_id: u64,
    pub effect_id: u64,
    pub declarations: Vec<CSSDeclaration>,
}

#[derive(Clone, Default)]
pub struct AnimationTick {
    pub monotonic_time: f64,
    pub begin_frame_source_id: u64,
    pub begin_frame_sequence: u64,
    pub samples: Vec<AnimationStyleSample>,
    pub has_active_animations: bool,
}

pub enum AnimationMutation {
    BeginFrame {
        source_id: u64,
        begin_frame_sequence: u64,
        frame_time: Instant,
    },
    Tick(AnimationTick),
    InvalidatePaintBatch,
    CommitPaint,
}

#[derive(Clone)]
pub enum DocumentMutation {
    ApplyStyleBatch(Vec<AnimationStyleSample>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PaintMutation {
    SetTargetedProperties { node_ids: Vec<u64> },
    Commit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationEffect {
    RequestBeginFrame,
}

#[derive(Default)]
pub struct AnimationOutput {
    pub document: Vec<DocumentMutation>,
    pub paint: Vec<PaintMutation>,
    pub effects: Vec<AnimationEffect>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationError {
    InvalidTime,
    OutOfOrderFrame,
}

pub struct AnimationEngine {
    last_time: f64,
    last_begin_frame: Option<(u64, u64)>,
    last_frame_time: Option<Instant>,
    targeted_nodes: BTreeSet<u64>,
    targeted_batch_compatible: bool,
}

impl Default for AnimationEngine {
    fn default() -> Self {
        Self {
            last_time: 0.0,
            last_begin_frame: None,
            last_frame_time: None,
            targeted_nodes: BTreeSet::new(),
            targeted_batch_compatible: true,
        }
    }
}

impl AnimationEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn LastTime(&self) -> f64 {
        self.last_time
    }

    pub fn ApplyMutation(
        &mut self,
        mutation: AnimationMutation,
    ) -> Result<AnimationOutput, AnimationError> {
        match mutation {
            AnimationMutation::BeginFrame {
                source_id,
                begin_frame_sequence,
                frame_time,
            } => {
                if self
                    .last_begin_frame
                    .is_some_and(|(last_source, last_sequence)| {
                        last_source == source_id && begin_frame_sequence < last_sequence
                    })
                    || self
                        .last_frame_time
                        .is_some_and(|last_time| frame_time < last_time)
                {
                    return Err(AnimationError::OutOfOrderFrame);
                }
                self.last_begin_frame = Some((source_id, begin_frame_sequence));
                self.last_frame_time = Some(frame_time);
                Ok(AnimationOutput::default())
            }
            AnimationMutation::Tick(tick) => self.ApplyTick(tick),
            AnimationMutation::InvalidatePaintBatch => {
                self.targeted_nodes.clear();
                self.targeted_batch_compatible = false;
                Ok(AnimationOutput {
                    paint: vec![PaintMutation::SetTargetedProperties {
                        node_ids: Vec::new(),
                    }],
                    ..Default::default()
                })
            }
            AnimationMutation::CommitPaint => {
                self.targeted_nodes.clear();
                self.targeted_batch_compatible = true;
                Ok(AnimationOutput {
                    paint: vec![PaintMutation::Commit],
                    ..Default::default()
                })
            }
        }
    }

    fn ApplyTick(&mut self, tick: AnimationTick) -> Result<AnimationOutput, AnimationError> {
        if !tick.monotonic_time.is_finite() || tick.monotonic_time < self.last_time {
            return Err(AnimationError::InvalidTime);
        }
        if tick.begin_frame_source_id != 0 && tick.begin_frame_sequence != 0 {
            if self
                .last_begin_frame
                .is_some_and(|(last_source, last_sequence)| {
                    last_source == tick.begin_frame_source_id
                        && tick.begin_frame_sequence < last_sequence
                })
            {
                return Err(AnimationError::OutOfOrderFrame);
            }
            self.last_begin_frame = Some((tick.begin_frame_source_id, tick.begin_frame_sequence));
        }
        self.last_time = tick.monotonic_time;

        let targeted = tick.samples.iter().all(|sample| {
            sample.declarations.iter().all(|declaration| {
                matches!(
                    declaration.property.as_str(),
                    "opacity" | "transform" | "transform-origin"
                )
            })
        }) && self.targeted_batch_compatible;

        if targeted {
            self.targeted_nodes
                .extend(tick.samples.iter().map(|sample| sample.node_id));
        } else {
            self.targeted_nodes.clear();
            self.targeted_batch_compatible = false;
        }

        let mut output = AnimationOutput::default();
        if !tick.samples.is_empty() {
            output
                .document
                .push(DocumentMutation::ApplyStyleBatch(tick.samples));
            output.paint.push(PaintMutation::SetTargetedProperties {
                node_ids: self.targeted_nodes.iter().copied().collect(),
            });
        }
        if tick.has_active_animations {
            output.effects.push(AnimationEffect::RequestBeginFrame);
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulates_targeted_properties_until_commit() {
        let mut engine = AnimationEngine::new();
        for (node_id, property) in [(7, "opacity"), (9, "transform")] {
            let output = engine
                .ApplyMutation(AnimationMutation::Tick(AnimationTick {
                    monotonic_time: 1.0,
                    samples: vec![AnimationStyleSample {
                        node_id,
                        effect_id: node_id,
                        declarations: vec![CSSDeclaration {
                            property: property.into(),
                            value: "1".into(),
                            important: false,
                        }],
                    }],
                    ..Default::default()
                }))
                .unwrap();
            let PaintMutation::SetTargetedProperties { node_ids } = &output.paint[0] else {
                panic!("expected targeted paint mutation")
            };
            assert!(node_ids.contains(&node_id));
        }
        engine
            .ApplyMutation(AnimationMutation::CommitPaint)
            .unwrap();
        let output = engine
            .ApplyMutation(AnimationMutation::Tick(AnimationTick {
                monotonic_time: 2.0,
                samples: vec![AnimationStyleSample {
                    node_id: 11,
                    effect_id: 11,
                    declarations: vec![CSSDeclaration {
                        property: "width".into(),
                        value: "1px".into(),
                        important: false,
                    }],
                }],
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(
            output.paint,
            vec![PaintMutation::SetTargetedProperties {
                node_ids: Vec::new()
            }]
        );
    }

    #[test]
    fn begin_frame_order_is_scoped_to_the_source() {
        let mut engine = AnimationEngine::new();
        let start = Instant::now();
        engine
            .ApplyMutation(AnimationMutation::BeginFrame {
                source_id: 3,
                begin_frame_sequence: 90,
                frame_time: start,
            })
            .unwrap();
        engine
            .ApplyMutation(AnimationMutation::BeginFrame {
                source_id: 4,
                begin_frame_sequence: 1,
                frame_time: start,
            })
            .unwrap();
        assert!(matches!(
            engine.ApplyMutation(AnimationMutation::BeginFrame {
                source_id: 4,
                begin_frame_sequence: 0,
                frame_time: start,
            }),
            Err(AnimationError::OutOfOrderFrame)
        ));
    }
}
