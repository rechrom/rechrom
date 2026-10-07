//! quickjs-libc.c:88..162: address-stable intrusive host state.
//! These are host-only layouts, never members of the QuickJS engine runtime.
use quickjs::{list::list_head, quickjs_header::*};
use std::sync::atomic::AtomicI32;
#[repr(C)]
pub struct JSOSRWHandler {
    pub link: list_head,
    pub fd: i32,
    pub poll_fd_index: i32,
    pub rw_func: [JSValue; 2],
}
#[repr(C)]
pub struct JSOSSignalHandler {
    pub link: list_head,
    pub sig_num: i32,
    pub func: JSValue,
}
#[repr(C)]
pub struct JSOSTimer {
    pub link: list_head,
    pub timer_id: i32,
    pub timeout: i64,
    pub func: JSValue,
}
#[repr(C)]
pub struct JSWorkerMessage {
    pub link: list_head,
    pub data: *mut u8,
    pub data_len: usize,
    pub sab_tab: *mut *mut u8,
    pub sab_tab_len: usize,
}
#[repr(C)]
pub struct JSWaker {
    pub read_fd: i32,
    pub write_fd: i32,
}
#[repr(C)]
pub struct JSWorkerMessagePipe {
    pub ref_count: AtomicI32,
    pub mutex: libc::pthread_mutex_t,
    pub msg_queue: list_head,
    pub waker: JSWaker,
}
#[repr(C)]
pub struct JSWorkerMessageHandler {
    pub link: list_head,
    pub recv_pipe: *mut JSWorkerMessagePipe,
    pub on_message_func: JSValue,
    pub poll_fd_index: i32,
}
#[repr(C)]
pub struct JSRejectedPromiseEntry {
    pub link: list_head,
    pub promise: JSValue,
    pub reason: JSValue,
}
#[repr(C)]
pub struct JSThreadState {
    pub os_rw_handlers: list_head,
    pub os_signal_handlers: list_head,
    pub os_timers: list_head,
    pub port_list: list_head,
    pub rejected_promise_list: list_head,
    pub eval_script_recurse: i32,
    pub next_timer_id: i32,
    pub recv_pipe: *mut JSWorkerMessagePipe,
    pub send_pipe: *mut JSWorkerMessagePipe,
    pub poll_fds: *mut libc::pollfd,
    pub poll_fds_size: i32,
}
