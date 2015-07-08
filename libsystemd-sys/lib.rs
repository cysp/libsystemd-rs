#![allow(non_camel_case_types)]

//! Low-level bindings for libsystemd

extern crate libc;
extern crate signalfd;


extern {
    pub static LIBSYSTEMD_SYS__SD_EVENT_OFF: libc::c_int;
    pub static LIBSYSTEMD_SYS__SD_EVENT_ON: libc::c_int;
    pub static LIBSYSTEMD_SYS__SD_EVENT_ONESHOT: libc::c_int;

    pub static LIBSYSTEMD_SYS__SD_EVENT_INITIAL: libc::c_int;
    pub static LIBSYSTEMD_SYS__SD_EVENT_ARMED: libc::c_int;
    pub static LIBSYSTEMD_SYS__SD_EVENT_PENDING: libc::c_int;
    pub static LIBSYSTEMD_SYS__SD_EVENT_RUNNING: libc::c_int;
    pub static LIBSYSTEMD_SYS__SD_EVENT_EXITING: libc::c_int;
    pub static LIBSYSTEMD_SYS__SD_EVENT_FINISHED: libc::c_int;

    // pub static LIBSYSTEMD_SYS__SD_EVENT_PRIORITY_IMPORTANT: libc::c_int;
    // pub static LIBSYSTEMD_SYS__SD_EVENT_PRIORITY_NORMAL: libc::c_int;
    // pub static LIBSYSTEMD_SYS__SD_EVENT_PRIORITY_IDLE: libc::c_int;
}


// daemon
extern {
    pub fn sd_listen_fds(unset_environment: libc::c_int) -> libc::c_int;
    pub fn sd_notify(unset_environment: libc::c_int, state: *const libc::c_char) -> libc::c_int;
    pub fn sd_watchdog_enabled(unset_environment: libc::c_int, usec: *mut libc::uint64_t) -> libc::c_int;
}


// event
pub type sd_event = *const libc::c_void;
pub type sd_event_source = *const libc::c_void;

pub type sd_event_handler_t = extern fn(/* s */ sd_event_source, /* userdata */ *const libc::c_void) -> libc::c_int;
pub type sd_event_io_handler_t = extern fn(/* s */ sd_event_source, /* fd */ libc::c_int, /* revents */ libc::uint32_t, /* userdata */ *const libc::c_void) -> libc::c_int;
pub type sd_event_time_handler_t = extern fn(/* s */ sd_event_source, /* usec */ libc::uint64_t, /* userdata */ *const libc::c_void) -> libc::c_int;
pub type sd_event_signal_handler_t = extern fn(/* s */ sd_event_source, /* si */ *const signalfd::sys::signalfd_siginfo, /* userdata */ *const libc::c_void) -> libc::c_int;
// pub type sd_event_child_handler_t = extern fn(/* s */ sd_event_source, /* si */ *const siginfo_t, /* userdata */ *const libc::c_void) -> libc::c_int;


pub const EPOLLIN: libc::uint32_t = 0x001;
pub const EPOLLPRI: libc::uint32_t = 0x002;
pub const EPOLLOUT: libc::uint32_t = 0x004;
pub const EPOLLRDNORM: libc::uint32_t = 0x040;
pub const EPOLLRDBAND: libc::uint32_t = 0x080;
pub const EPOLLWRNORM: libc::uint32_t = 0x100;
pub const EPOLLWRBAND: libc::uint32_t = 0x200;
pub const EPOLLMSG: libc::uint32_t = 0x400;
pub const EPOLLERR: libc::uint32_t = 0x008;
pub const EPOLLHUP: libc::uint32_t = 0x010;
pub const EPOLLRDHUP: libc::uint32_t = 0x2000;
pub const EPOLLWAKEUP: libc::uint32_t = 1 << 29;
pub const EPOLLONESHOT: libc::uint32_t = 1 << 30;
pub const EPOLLET: libc::uint32_t = 1 << 31;

pub type clockid_t = libc::c_int;
pub const CLOCK_REALTIME: clockid_t = 0;
pub const CLOCK_MONOTONIC: clockid_t = 1;
// CLOCK_PROCESS_CPUTIME_ID = 2,
// CLOCK_THREAD_CPUTIME_ID = 3,
// CLOCK_MONOTONIC_RAW = 4,
// CLOCK_REALTIME_COARSE = 5,
// CLOCK_MONOTONIC_COARSE = 6,
// CLOCK_BOOTTIME = 7,
// CLOCK_REALTIME_ALARM = 8,
pub const CLOCK_BOOTTIME_ALARM: clockid_t = 9;
// CLOCK_SGI_CYCLE = 10,
// CLOCK_TAI = 11,

// #[repr(C)]
// #[derive(Copy,Clone,Debug)]
// pub struct timespec {
//     pub tv_sec: std::os::unix::raw::time_t,
//     pub tv_nsec: libc::c_long,
// }

pub use libc::timespec;
extern {
    pub fn clock_gettime(clk_id: clockid_t, tp: *mut libc::timespec) -> libc::c_int;
}


#[link(name="systemd")]
extern {
    pub fn sd_event_new(e: *mut sd_event) -> libc::c_int;
    pub fn sd_event_default(e: *mut sd_event) -> libc::c_int;

    pub fn sd_event_ref(e: sd_event) -> sd_event;
    pub fn sd_event_unref(e: sd_event) -> sd_event;

    pub fn sd_event_add_io(e: sd_event, s: *mut sd_event_source, fd: libc::c_int, events: libc::uint32_t, callback: sd_event_io_handler_t, userdata: *const libc::c_void) -> libc::c_int;
    pub fn sd_event_add_time(e: sd_event, s: *mut sd_event_source, clock: clockid_t, usec: libc::uint64_t, accuracy: libc::uint64_t, callback: sd_event_time_handler_t, userdata: *const libc::c_void) -> libc::c_int;
    pub fn sd_event_add_signal(e: sd_event, s: *mut sd_event_source, sig: libc::c_int, callback: sd_event_signal_handler_t, userdata: *const libc::c_void) -> libc::c_int;
    // int sd_event_add_child(sd_event *e, sd_event_source **s, pid_t pid, int options, sd_event_child_handler_t callback, void *userdata);
    pub fn sd_event_add_defer(e: sd_event, s: *mut sd_event_source, callback: sd_event_handler_t, userdata: *const libc::c_void) -> libc::c_int;
    pub fn sd_event_add_post(e: sd_event, s: *mut sd_event_source, callback: sd_event_handler_t, userdata: *const libc::c_void) -> libc::c_int;
    pub fn sd_event_add_exit(e: sd_event, s: *mut sd_event_source, callback: sd_event_handler_t, userdata: *const libc::c_void) -> libc::c_int;

    pub fn sd_event_prepare(e: sd_event) -> libc::c_int;
    pub fn sd_event_wait(e: sd_event, timeout: libc::uint64_t) -> libc::c_int;
    pub fn sd_event_dispatch(e: sd_event) -> libc::c_int;
    pub fn sd_event_run(e: sd_event, timeout: libc::uint64_t) -> libc::c_int;
    pub fn sd_event_loop(e: sd_event) -> libc::c_int;
    pub fn sd_event_exit(e: sd_event, code: libc::c_int) -> libc::c_int;

    pub fn sd_event_now(e: sd_event, clock: clockid_t, usec: *mut libc::uint64_t) -> libc::c_int;

    // int sd_event_get_fd(sd_event *e);
    pub fn sd_event_get_state(e: sd_event) -> libc::c_int;
    // int sd_event_get_tid(sd_event *e, pid_t *tid);
    pub fn sd_event_get_exit_code(e: sd_event, code: *mut libc::c_int) -> libc::c_int;
    // int sd_event_set_watchdog(sd_event *e, int b);
    // int sd_event_get_watchdog(sd_event *e);

    pub fn sd_event_source_ref(e: sd_event_source) -> sd_event_source;
    pub fn sd_event_source_unref(e: sd_event_source) -> sd_event_source;

    // sd_event *sd_event_source_get_event(sd_event_source *s);
    // void* sd_event_source_get_userdata(sd_event_source *s);
    // void* sd_event_source_set_userdata(sd_event_source *s, void *userdata);

    // int sd_event_source_set_description(sd_event_source *s, const char *description);
    // int sd_event_source_get_description(sd_event_source *s, const char **description);
    // int sd_event_source_set_prepare(sd_event_source *s, sd_event_handler_t callback);
    // int sd_event_source_get_pending(sd_event_source *s);
    // int sd_event_source_get_priority(sd_event_source *s, int64_t *priority);
    // int sd_event_source_set_priority(sd_event_source *s, int64_t priority);
    pub fn sd_event_source_get_enabled(s: sd_event_source, enabled: *mut libc::c_int) -> libc::c_int;
    pub fn sd_event_source_set_enabled(s: sd_event_source, enabled: libc::c_int) -> libc::c_int;

    // int sd_event_source_get_io_fd(sd_event_source *s);
    // int sd_event_source_set_io_fd(sd_event_source *s, int fd);
    // int sd_event_source_get_io_events(sd_event_source *s, uint32_t* events);
    // int sd_event_source_set_io_events(sd_event_source *s, uint32_t events);
    // int sd_event_source_get_io_revents(sd_event_source *s, uint32_t* revents);

    pub fn sd_event_source_get_time(s: sd_event_source, usec: *mut libc::uint64_t) -> libc::c_int;
    pub fn sd_event_source_set_time(s: sd_event_source, usec: libc::uint64_t) -> libc::c_int;
    pub fn sd_event_source_get_time_accuracy(s: sd_event_source, usec: *mut libc::uint64_t) -> libc::c_int;
    pub fn sd_event_source_set_time_accuracy(s: sd_event_source, usec: libc::uint64_t) -> libc::c_int;
    pub fn sd_event_source_get_time_clock(s: sd_event_source, clock: *mut clockid_t) -> libc::c_int;

    pub fn sd_event_source_get_signal(s: sd_event_source) -> libc::c_int;

    // int sd_event_source_get_child_pid(sd_event_source *s, pid_t *pid);
}
