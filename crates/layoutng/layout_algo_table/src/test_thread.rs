//! The source source heap and DEFINE_STATIC_LOCAL caches belong to one table
//! thread. Keep all source table regression work on that thread, including
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
            .name("source-table-source-tests".into())
            .spawn(move || {
                for (body, reply) in receiver {
                    let result = std::panic::catch_unwind(body);
                    let _ = reply.send(result);
                }
            })
            .expect("source table test thread");
        sender
    });
    let (reply, result) = mpsc::channel();
    sender
        .send((body, reply))
        .expect("source table test dispatch");
    if let Err(error) = result.recv().expect("source table test result") {
        std::panic::resume_unwind(error);
    }
}
