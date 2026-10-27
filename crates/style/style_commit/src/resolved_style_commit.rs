// Copyright 2026 The Chromium Authors. BSD-style license; see Chromium LICENSE.

//! Publication boundary for a completed batch of resolved styles.
//!
//! Resolution owns construction of each style and the document adapter owns
//! storage. This operation transfers each completed style into that adapter in
//! traversal order, without cloning or projecting the native style payload.

#![allow(non_snake_case)]

pub trait ResolvedStyleCommitBackend {
    type NodeId;
    type ResolvedStyle;

    fn CommitResolvedStyle(&mut self, node: Self::NodeId, style: Self::ResolvedStyle);
}

pub fn CommitResolvedStyles<B, I>(backend: &mut B, styles: I)
where
    B: ResolvedStyleCommitBackend,
    I: IntoIterator<Item = (B::NodeId, B::ResolvedStyle)>,
{
    let styles = styles.into_iter();
    let mut trace = browser_tracing::span("style_commit", "ResolvedStyle.CommitBatch");
    let (lower_bound, upper_bound) = styles.size_hint();
    trace.set("styles_lower_bound", lower_bound as f64);
    if let Some(upper_bound) = upper_bound {
        trace.set("styles_upper_bound", upper_bound as f64);
    }
    for (node, style) in styles {
        backend.CommitResolvedStyle(node, style);
    }
}
