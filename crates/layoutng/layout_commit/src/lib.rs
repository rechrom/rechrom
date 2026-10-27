#![allow(non_snake_case)]

//! Typed boundary between resident LayoutNG state and downstream publication.

use std::collections::HashSet;

pub mod trace;
pub use trace::{span as trace_span, TracePhase};

/// Layout updates retain native fragments for borrowed queries. Paint also
/// materializes the current paint boundary output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutUpdateKind {
    Layout,
    Paint,
}

/// Opaque client set produced by Paint after its artifact is complete.
/// Layout consumes it to advance native DisplayItemClient state.
pub struct PaintClientCommit {
    used_clients: HashSet<u64>,
}

impl PaintClientCommit {
    pub fn FromClientSet(used_clients: HashSet<u64>) -> Self {
        Self { used_clients }
    }

    pub fn UsedClients(&self) -> &HashSet<u64> {
        &self.used_clients
    }
}

/// Interface implemented by a Paint engine output. It lets Layout consume the
/// opaque commit receipt without depending on the Paint crate or its artifact.
pub trait PaintCommitOutput {
    type Artifact;

    fn IntoLayoutCommit(self) -> (Self::Artifact, PaintClientCommit);
}

/// Adapter implemented by the resident layout owner. `layout_commit` owns the
/// order while the adapter retains access to native clients and output types.
pub trait PaintCommitBackend {
    type Layout;
    type Published;

    fn ValidateAuxiliaryClients(&mut self, commit: &PaintClientCommit);
    fn CommitNativeClients(&mut self, commit: &PaintClientCommit, layout: &mut Self::Layout);
    fn CommitPrePaintInputs(&mut self, commit: &PaintClientCommit);
    fn PublishLayout(&mut self, layout: Self::Layout) -> Self::Published;
}

#[inline(never)]
pub fn CommitPaintResult<B: PaintCommitBackend, P: PaintCommitOutput>(
    backend: &mut B,
    mut layout: B::Layout,
    paint: P,
) -> (B::Published, P::Artifact) {
    let mut trace = trace_span(TracePhase::CommitPaint);
    let (artifact, commit) = paint.IntoLayoutCommit();
    trace.set("used_clients", commit.UsedClients().len() as f64);
    {
        let _phase = trace_span(TracePhase::ValidateAuxiliaryClients);
        backend.ValidateAuxiliaryClients(&commit);
    }
    {
        let _phase = trace_span(TracePhase::CommitNativeClients);
        backend.CommitNativeClients(&commit, &mut layout);
    }
    {
        let _phase = trace_span(TracePhase::CommitPrePaintInputs);
        backend.CommitPrePaintInputs(&commit);
    }
    let published = {
        let _phase = trace_span(TracePhase::PublishPaintResult);
        backend.PublishLayout(layout)
    };
    (published, artifact)
}

/// Native adapters supply ownership-specific operations while this package
/// owns the observable publication order. This keeps LayoutObject and fragment
/// implementation types out of the commit boundary.
pub trait LayoutPublicationBackend {
    type Environment;
    type Output;

    fn CachedObjects(&self) -> usize;
    fn CachedStyles(&self) -> usize;
    fn CreateEnvironment(&mut self) -> Self::Environment;
    fn ExportFragments(&mut self) -> Self::Output;
    fn CommitEnvironment(&mut self, environment: Self::Environment);
}

/// Environment token for one layout publication. Creating the token before
/// layout lets native preparation reuse its resources; consuming it through
/// `Publish` or `Commit` makes the publication order explicit and shared by
/// the ordinary layout path and retained-result snapshots.
#[must_use = "a layout commit transaction must publish or commit its environment"]
pub struct LayoutCommitTransaction<E> {
    environment: E,
}

impl<E> LayoutCommitTransaction<E> {
    #[inline(never)]
    pub fn Begin(create_environment: impl FnOnce() -> E) -> Self {
        let _phase = trace_span(TracePhase::CreateEnvironment);
        Self {
            environment: create_environment(),
        }
    }

    pub fn Environment(&self) -> &E {
        &self.environment
    }

    /// Publish a fragment snapshot, run any native finalization that must
    /// observe it, and only then commit the environment.
    #[inline(never)]
    pub fn Publish<C, O>(
        self,
        context: &mut C,
        export_fragments: impl FnOnce(&mut C) -> O,
        finalize: impl FnOnce(&mut C),
        commit_environment: impl FnOnce(E, &mut C),
    ) -> O {
        let output = {
            let _phase = layout_fragment::trace::export_tree();
            export_fragments(context)
        };
        finalize(context);
        {
            let _phase = trace_span(TracePhase::CommitEnvironment);
            commit_environment(self.environment, context);
        }
        output
    }

    /// Finish a transaction which intentionally produces no exported result.
    #[inline(never)]
    pub fn Commit<C>(self, context: &mut C, commit_environment: impl FnOnce(E, &mut C)) {
        let _phase = trace_span(TracePhase::CommitEnvironment);
        commit_environment(self.environment, context);
    }
}

#[inline(never)]
pub fn PublishLayoutResult<B: LayoutPublicationBackend>(backend: &mut B) -> B::Output {
    let mut trace = trace_span(TracePhase::PublishLayoutResult);
    trace.set("cached_objects", backend.CachedObjects() as f64);
    trace.set("cached_styles", backend.CachedStyles() as f64);

    let transaction = LayoutCommitTransaction::Begin(|| backend.CreateEnvironment());
    transaction.Publish(
        backend,
        |backend| backend.ExportFragments(),
        |_| {},
        |environment, backend| backend.CommitEnvironment(environment),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct RecordingBackend {
        steps: Vec<&'static str>,
    }

    impl LayoutPublicationBackend for RecordingBackend {
        type Environment = u8;
        type Output = u32;

        fn CachedObjects(&self) -> usize {
            2
        }

        fn CachedStyles(&self) -> usize {
            3
        }

        fn CreateEnvironment(&mut self) -> Self::Environment {
            self.steps.push("create");
            7
        }

        fn ExportFragments(&mut self) -> Self::Output {
            self.steps.push("export");
            42
        }

        fn CommitEnvironment(&mut self, environment: Self::Environment) {
            assert_eq!(environment, 7);
            self.steps.push("commit");
        }
    }

    #[test]
    fn publication_order_is_owned_by_the_commit_package() {
        let mut backend = RecordingBackend::default();
        assert_eq!(PublishLayoutResult(&mut backend), 42);
        assert_eq!(backend.steps, ["create", "export", "commit"]);
    }

    #[test]
    fn commit_only_transaction_does_not_export() {
        let mut steps = Vec::new();
        let transaction = LayoutCommitTransaction::Begin(|| {
            steps.push("create");
            7
        });
        assert_eq!(*transaction.Environment(), 7);
        transaction.Commit(&mut steps, |environment, steps| {
            assert_eq!(environment, 7);
            steps.push("commit");
        });
        assert_eq!(steps, ["create", "commit"]);
    }

    struct RecordingPaint {
        clients: HashSet<u64>,
    }

    impl PaintCommitOutput for RecordingPaint {
        type Artifact = u8;

        fn IntoLayoutCommit(self) -> (Self::Artifact, PaintClientCommit) {
            (9, PaintClientCommit::FromClientSet(self.clients))
        }
    }

    #[derive(Default)]
    struct RecordingPaintBackend {
        steps: Vec<&'static str>,
    }

    impl PaintCommitBackend for RecordingPaintBackend {
        type Layout = u32;
        type Published = u64;

        fn ValidateAuxiliaryClients(&mut self, commit: &PaintClientCommit) {
            assert!(commit.UsedClients().contains(&17));
            self.steps.push("auxiliary");
        }

        fn CommitNativeClients(&mut self, commit: &PaintClientCommit, layout: &mut u32) {
            assert!(commit.UsedClients().contains(&17));
            *layout += 1;
            self.steps.push("native");
        }

        fn CommitPrePaintInputs(&mut self, commit: &PaintClientCommit) {
            assert!(commit.UsedClients().contains(&17));
            self.steps.push("prepaint");
        }

        fn PublishLayout(&mut self, layout: u32) -> u64 {
            self.steps.push("publish");
            layout as u64
        }
    }

    #[test]
    fn paint_commit_order_is_owned_by_the_commit_package() {
        let mut backend = RecordingPaintBackend::default();
        let (published, artifact) = CommitPaintResult(
            &mut backend,
            41,
            RecordingPaint {
                clients: HashSet::from([17]),
            },
        );
        assert_eq!((published, artifact), (42, 9));
        assert_eq!(
            backend.steps,
            ["auxiliary", "native", "prepaint", "publish"]
        );
    }
}
