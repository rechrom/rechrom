//! The source native heap and DEFINE_STATIC_LOCAL caches belong to one browser
//! thread. Keep all native browser regression work on that thread, including
//! when the Rust test harness runs the surrounding tests concurrently.
use std::{
    any::Any,
    sync::{mpsc, OnceLock},
};
type Reply = Result<(), Box<dyn Any + Send>>;
type Request = (fn(), mpsc::Sender<Reply>);
pub(crate) fn run(body: fn()) {
    static THREAD: OnceLock<mpsc::Sender<Request>> = OnceLock::new();
    let sender = THREAD.get_or_init(|| {
        let (sender, receiver) = mpsc::channel::<Request>();
        std::thread::Builder::new()
            .name("source-browser-native-tests".into())
            // Native layout and reentrant JavaScript callbacks share this
            // thread. Its fixture runtime has a 4 MiB checked JS stack budget.
            .stack_size(32 * 1024 * 1024)
            .spawn(move || {
                for (body, reply) in receiver {
                    let result = std::panic::catch_unwind(body);
                    let _ = reply.send(result);
                }
            })
            .expect("native browser test thread");
        sender
    });
    let (reply, result) = mpsc::channel();
    sender
        .send((body, reply))
        .expect("native browser test dispatch");
    if let Err(error) = result.recv().expect("native browser test result") {
        std::panic::resume_unwind(error);
    }
}

/// Fixture wiring lives in the consuming host, never in production DOM.
pub(crate) fn BuildDOMProjection(
    owner: &mut dom::DOM,
    interaction: &dom::UserInteractionState,
    engine: &mut layoutng_assembly::layout_engine::LayoutEngine,
) -> *mut layoutng_assembly::internal::layout_object::LayoutObject {
    owner.EmitLayoutMutations(interaction, |mutation| {
        engine.ApplyMutation(mutation);
    });
    engine.GetLayoutTree().expect("projected tree").Root() as *const _ as *mut _
}
