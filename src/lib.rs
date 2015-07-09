#![feature(convert)]
//! Bindings for libsystemd's event subsystem

extern crate libc;
extern crate time;
extern crate libsystemd_sys as ffi;
extern crate signalfd;

mod error;
mod clock;
// mod daemon;
mod event;

pub use error::Error;

// pub use daemon::{
//     listen_fds,
//     notify,
//     watchdog_enabled,
// };

pub use clock::{
    Clock,
    ClockTimestamp,
    RealtimeClockTimestamp,
    MonotonicClockTimestamp,
};

pub use event::{
    IoEventMask,
    IoEventMaskBuilder,
    IoEventTriggering,
};

pub use event::{
    Event,
    EventState,
    EventSource,
    EventSourceEnabled,
    IoEventSource,
    TimeEventSource,
    SignalEventSource,
    /*ChildEventSource,*/
    DeferEventSource,
    PostEventSource,
    ExitEventSource,
};
