extern crate libc;
extern crate time;
extern crate libsystemd_sys as ffi;

mod error;
// mod daemon;
mod event;

pub use error::Error;

// pub use daemon::{
//     listen_fds,
//     notify,
//     watchdog_enabled,
// };

pub use event::{
    Event,
    EventState,
    EventClock,
    EventSource,
    EventSourceEnabled,
    /*IoEventSource,*/
    TimeEventSource,
    /*SignalEventSource,*/
    /*ChildEventSource,*/
    DeferEventSource,
    PostEventSource,
    ExitEventSource,
};
