use libc;
use libc::consts::os::posix88::EINVAL;
use std;
// use std::ffi::{CString};
// use std::ops::Range;
// use std::os::unix::io::RawFd;
use time;
use signalfd;

use {ffi, Error};

// mod event_timestamp;
use clock::*;


#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum EventState {
    Initial,
    Armed,
    Pending,
    Running,
    Exiting,
    Finished,
    Unknown(i32),
}
impl EventState {
    fn from_raw(r: libc::c_int) -> EventState {
        if r == ffi::LIBSYSTEMD_SYS__SD_EVENT_INITIAL {
            EventState::Initial
        } else if r == ffi::LIBSYSTEMD_SYS__SD_EVENT_ARMED {
            EventState::Armed
        } else if r == ffi::LIBSYSTEMD_SYS__SD_EVENT_PENDING {
            EventState::Pending
        } else if r == ffi::LIBSYSTEMD_SYS__SD_EVENT_RUNNING {
            EventState::Running
        } else if r == ffi::LIBSYSTEMD_SYS__SD_EVENT_EXITING {
            EventState::Exiting
        } else if r == ffi::LIBSYSTEMD_SYS__SD_EVENT_FINISHED {
            EventState::Finished
        } else {
            EventState::Unknown(r)
        }
    }
    #[allow(dead_code)]
    fn into_raw(self) -> libc::c_int {
        match self {
            EventState::Initial => ffi::LIBSYSTEMD_SYS__SD_EVENT_INITIAL,
            EventState::Armed => ffi::LIBSYSTEMD_SYS__SD_EVENT_ARMED,
            EventState::Pending => ffi::LIBSYSTEMD_SYS__SD_EVENT_PENDING,
            EventState::Running => ffi::LIBSYSTEMD_SYS__SD_EVENT_RUNNING,
            EventState::Exiting => ffi::LIBSYSTEMD_SYS__SD_EVENT_EXITING,
            EventState::Finished => ffi::LIBSYSTEMD_SYS__SD_EVENT_FINISHED,
            EventState::Unknown(r) => r as libc::c_int,
        }
    }
}


#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum EventSourceEnabled {
    Off,
    On,
    OneShot,
    Unknown(i32),
}
impl EventSourceEnabled {
    fn from_raw(r: libc::c_int) -> EventSourceEnabled {
        if r == ffi::LIBSYSTEMD_SYS__SD_EVENT_OFF {
            EventSourceEnabled::Off
        } else if r == ffi::LIBSYSTEMD_SYS__SD_EVENT_ON {
            EventSourceEnabled::On
        } else if r == ffi::LIBSYSTEMD_SYS__SD_EVENT_ONESHOT {
            EventSourceEnabled::OneShot
        } else {
            EventSourceEnabled::Unknown(r)
        }
    }
    fn into_raw(self) -> libc::c_int {
        match self {
            EventSourceEnabled::Off => ffi::LIBSYSTEMD_SYS__SD_EVENT_OFF,
            EventSourceEnabled::On => ffi::LIBSYSTEMD_SYS__SD_EVENT_ON,
            EventSourceEnabled::OneShot => ffi::LIBSYSTEMD_SYS__SD_EVENT_ONESHOT,
            EventSourceEnabled::Unknown(r) => r,
        }
    }
}

#[cfg(test)]
mod event_source_enabled_tests {
    use super::*;

    #[test]
    fn smoke() {
        assert_eq!(EventSourceEnabled::from_raw(0), EventSourceEnabled::Off);
        assert_eq!(EventSourceEnabled::from_raw(1), EventSourceEnabled::On);
        assert_eq!(EventSourceEnabled::from_raw(-1), EventSourceEnabled::OneShot);
        assert_eq!(EventSourceEnabled::from_raw(2), EventSourceEnabled::Unknown(2));

        assert_eq!(EventSourceEnabled::Off.into_raw(), 0);
        assert_eq!(EventSourceEnabled::On.into_raw(), 1);
        assert_eq!(EventSourceEnabled::OneShot.into_raw(), -1);
        assert_eq!(EventSourceEnabled::Unknown(2).into_raw(), 2);
    }
}


#[derive(Debug)]
pub struct Event {
    e: ffi::sd_event,
}

impl Clone for Event {
    fn clone(&self) -> Event {
        Event {
            e: unsafe { ffi::sd_event_ref(self.e) },
        }
    }
}

impl std::ops::Drop for Event {
    fn drop(&mut self) {
        unsafe {
            ffi::sd_event_unref(self.e);
        }
    }
}


impl<'e> Event {
    pub fn new() -> Result<Event, Error> {
        let mut e: ffi::sd_event = 0 as ffi::sd_event;
        let rv = unsafe { ffi::sd_event_new(&mut e) };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(Event {
            e: e,
        })
    }

    pub fn default() -> Result<Event, Error> {
        let mut e: ffi::sd_event = 0 as ffi::sd_event;
        let rv = unsafe { ffi::sd_event_default(&mut e) };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(Event {
            e: e,
        })
    }

    pub fn as_ptr(&self) -> *const ffi::sd_event {
        &self.e
    }

    pub fn state(&'e self) -> EventState {
        let state = unsafe { ffi::sd_event_get_state(self.e) };
        EventState::from_raw(state)
    }

    pub fn exit(&'e mut self, code: i32) -> Result<(), Error> {
        let rv = unsafe { ffi::sd_event_exit(self.e, code as libc::c_int) };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(())
    }

    pub fn exit_code(&'e self) -> Result<i32, Error> {
        let mut exit_code: i32 = 0;
        let rv = unsafe { ffi::sd_event_get_exit_code(self.e, &mut exit_code) };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(exit_code)
    }

    // pub fn sd_event_add_io(e: sd_event, s: *mut sd_event_source, fd: libc::c_int, events: libc::uint32_t, callback: sd_event_io_handler_t, userdata: *const libc::c_void) -> libc::c_int;
    // pub type sd_event_io_handler_t = extern fn(/* s */ sd_event_source, /* fd */ libc::c_int, /* revents */ libc::uint32_t, /* userdata */ *const libc::c_void) -> libc::c_int;

    pub fn add_io<'z, 's, FD, F>(&'z mut self, fd: FD, events: IoEventTriggering, callback: F) -> Result<IoEventSource<'s, FD>, Error>
        where 'e: 'z, 'e: 's,
            FD: std::os::unix::io::AsRawFd,
            F: 's + FnMut(IoEventMask) -> i32
    {
        let callback = Box::new(callback);
        let userdata = &*callback as *const _ as *const libc::c_void;

        extern fn event_source_io_tramp<F>(_: ffi::sd_event_source, _fd: libc::c_int, revents: libc::uint32_t, userdata: *const libc::c_void) -> libc::c_int
            where F: FnMut(IoEventMask) -> i32
        {
            let cb_ptr = userdata as *mut F;
            let cb: &mut F = unsafe { &mut *cb_ptr };
            (*cb)(revents.into()) as libc::c_int
        }

        let mut s: ffi::sd_event_source = 0 as ffi::sd_event_source;
        let rv = unsafe { ffi::sd_event_add_io(self.e, &mut s, fd.as_raw_fd(), events.into(), event_source_io_tramp::<F>, userdata) };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(IoEventSource {
            _e: std::marker::PhantomData,
            s: s,
            fd: fd,
            cb: callback,
        })
    }

    pub fn add_time<'z, 's, EC, F>(&'z mut self, usec: EC, accuracy: time::Duration, callback: F) -> Result<TimeEventSource<'s, EC>, Error>
        where 'e: 'z, 'e: 's,
            EC: ClockTimestamp,
            F: 's + FnMut(EC) -> i32
    {
        let (clock, usec) = (EC::clock(), usec.as_usec());
        let accuracy = match accuracy.num_microseconds() {
            None => return Err(Error::from_negative_errno(-EINVAL)),
            Some(accuracy) if accuracy < 0 => return Err(Error::from_negative_errno(-EINVAL)),
            Some(accuracy) => accuracy,
        };

        let callback = Box::new(callback);
        let userdata = &*callback as *const _ as *const libc::c_void;

        extern fn event_source_time_tramp<EC, F>(_: ffi::sd_event_source, usec: libc::uint64_t, userdata: *const libc::c_void) -> libc::c_int
            where EC: ClockTimestamp, F: FnMut(EC) -> i32
        {
            let cb_ptr = userdata as *mut F;
            let cb: &mut F = unsafe { &mut *cb_ptr };
            let usec = EC::from_usec(usec as u64);
            (*cb)(usec) as libc::c_int
        }

        let mut s: ffi::sd_event_source = 0 as ffi::sd_event_source;
        let rv = unsafe { ffi::sd_event_add_time(self.e, &mut s, clock.into_raw(), usec as libc::uint64_t, accuracy as libc::uint64_t, event_source_time_tramp::<EC, F>, userdata) };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(TimeEventSource {
            _e: std::marker::PhantomData,
            _ec: std::marker::PhantomData,
            s: s,
            cb: callback,
        })
    }

    pub fn add_signal<'z, 's, F>(&'z mut self, signal: i32, callback: F) -> Result<SignalEventSource<'s>, Error>
        where 'e: 'z, 'e: 's, F: 's + FnMut(signalfd::SignalfdSigInfo) -> i32
    {
        let callback = Box::new(callback);
        let userdata = &*callback as *const _ as *const libc::c_void;

        extern fn event_source_signal_tramp<F>(_: ffi::sd_event_source, si: *const signalfd::sys::signalfd_siginfo, userdata: *const libc::c_void) -> libc::c_int
            where F: FnMut(signalfd::SignalfdSigInfo) -> i32
        {
            let cb_ptr = userdata as *mut F;
            let cb: &mut F = unsafe { &mut *cb_ptr };
            let si: signalfd::sys::signalfd_siginfo = unsafe { *si };
            (*cb)(si.into()) as libc::c_int
        }

        let mut s: ffi::sd_event_source = 0 as ffi::sd_event_source;
        let rv = unsafe { ffi::sd_event_add_signal(self.e, &mut s, signal, event_source_signal_tramp::<F>, userdata) };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(SignalEventSource {
            _e: std::marker::PhantomData,
            s: s,
            cb: callback,
        })
    }

    pub fn add_defer<'z, 's, F>(&'z mut self, callback: F) -> Result<DeferEventSource<'s>, Error>
        where 'e: 'z, 'e: 's, F: 's + FnMut() -> i32
    {
        let callback = Box::new(callback);
        let userdata = &*callback as *const _ as *const libc::c_void;

        extern fn event_source_defer_tramp<F>(_: ffi::sd_event_source, userdata: *const libc::c_void) -> libc::c_int
            where F: FnMut() -> i32
        {
            let cb_ptr = userdata as *mut F;
            let cb: &mut F = unsafe { &mut *cb_ptr };
            (*cb)() as libc::c_int
        }

        let mut s: ffi::sd_event_source = 0 as ffi::sd_event_source;
        let rv = unsafe { ffi::sd_event_add_defer(self.e, &mut s, event_source_defer_tramp::<F>, userdata) };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(DeferEventSource {
            _e: std::marker::PhantomData,
            s: s,
            cb: callback,
        })
    }

    pub fn add_post<'z, 's, F>(&'z mut self, callback: F) -> Result<PostEventSource<'s>, Error>
        where 'e: 'z, 'e: 's, F: 's + FnMut() -> i32
    {
        let callback = Box::new(callback);
        let userdata = &*callback as *const _ as *const libc::c_void;

        extern fn event_source_post_tramp<F>(_: ffi::sd_event_source, userdata: *const libc::c_void) -> libc::c_int
            where F: FnMut() -> i32
        {
            let cb_ptr = userdata as *mut F;
            let cb: &mut F = unsafe { &mut *cb_ptr };
            (*cb)() as libc::c_int
        }

        let mut s: ffi::sd_event_source = 0 as ffi::sd_event_source;
        let rv = unsafe { ffi::sd_event_add_post(self.e, &mut s, event_source_post_tramp::<F>, userdata) };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(PostEventSource {
            _e: std::marker::PhantomData,
            s: s,
            cb: callback,
        })
    }

    pub fn add_exit<'z, 's, F>(&'z mut self, callback: F) -> Result<ExitEventSource<'s>, Error>
        where 'e: 'z, 'e: 's, F: 's + FnMut() -> i32
    {
        let callback = Box::new(callback);
        let userdata = &*callback as *const _ as *const libc::c_void;

        extern fn event_source_exit_tramp<F>(_: ffi::sd_event_source, userdata: *const libc::c_void) -> libc::c_int
            where F: FnMut() -> i32
        {
            let cb_ptr = userdata as *mut F;
            let cb: &mut F = unsafe { &mut *cb_ptr };
            (*cb)() as libc::c_int
        }

        let mut s: ffi::sd_event_source = 0 as ffi::sd_event_source;
        let rv = unsafe { ffi::sd_event_add_exit(self.e, &mut s, event_source_exit_tramp::<F>, userdata) };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(ExitEventSource {
            _e: std::marker::PhantomData,
            s: s,
            cb: callback,
        })
    }
}

impl<'e> Event {
    pub fn run(&'e self, timeout: time::Duration) -> Result<bool, Error> {
        let timeout = match timeout.num_microseconds() {
            None => return Err(Error::from_negative_errno(-EINVAL)),
            Some(timeout) if timeout < 0 => return Err(Error::from_negative_errno(-EINVAL)),
            Some(timeout) => timeout,
        };

        let rv = unsafe { ffi::sd_event_run(self.e, timeout as libc::uint64_t) };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        if rv > 0 {
            return Ok(true)
        }
        Ok(false)
    }
    pub fn run_loop(&'e self) -> Result<(), Error> {
        let rv = unsafe { ffi::sd_event_loop(self.e) };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(())
    }

    pub fn now<EC: ClockTimestamp = MonotonicClockTimestamp>(&'e self) -> Result<EC, Error> {
        let clock = EC::clock();
        let mut usec: libc::uint64_t = 0;
        let rv = unsafe {
            ffi::sd_event_now(self.e, clock.into_raw(), &mut usec)
        };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(EC::from_usec(usec as u64))
    }
}

pub trait EventSource {
    fn as_raw(&self) -> ffi::sd_event_source;

    fn enabled(&self) -> Result<EventSourceEnabled, Error> {
        let mut enabled: libc::c_int = 0;
        let rv = unsafe {
            ffi::sd_event_source_get_enabled(self.as_raw(), &mut enabled)
        };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(EventSourceEnabled::from_raw(enabled))
    }

    fn set_enabled(&self, enabled: EventSourceEnabled) -> Result<(), Error> {
        let rv = unsafe {
            ffi::sd_event_source_set_enabled(self.as_raw(), enabled.into_raw())
        };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(())
    }
}

#[derive(Copy,Clone)]
pub struct IoEventMask(u32);
impl IoEventMask {
    pub fn new() -> IoEventMask {
        IoEventMask(ffi::EPOLLERR | ffi::EPOLLHUP)
    }
    pub fn builder() -> IoEventMaskBuilder {
        IoEventMaskBuilder(ffi::EPOLLERR | ffi::EPOLLHUP)
    }
    pub fn epollin(&self) -> bool {
        (self.0 & ffi::EPOLLIN) != 0
    }
    pub fn set_epollin(&mut self, v: bool) {
        if v {
            self.0 = self.0 | ffi::EPOLLIN
        } else {
            self.0 = self.0 & (0u32 ^ ffi::EPOLLIN)
        };
    }
    pub fn epollout(&self) -> bool {
        (self.0 & ffi::EPOLLOUT) != 0
    }
    pub fn set_epollout(&mut self, v: bool) {
        if v {
            self.0 = self.0 | ffi::EPOLLOUT
        } else {
            self.0 = self.0 & (0u32 ^ ffi::EPOLLOUT)
        };
    }
    pub fn epollrdhup(&self) -> bool {
        (self.0 & ffi::EPOLLRDHUP) != 0
    }
    pub fn set_epollrdhup(&mut self, v: bool) {
        if v {
            self.0 = self.0 | ffi::EPOLLRDHUP
        } else {
            self.0 = self.0 & (0u32 ^ ffi::EPOLLRDHUP)
        };
    }
    pub fn epollpri(&self) -> bool {
        (self.0 & ffi::EPOLLPRI) != 0
    }
    pub fn set_epollpri(&mut self, v: bool) {
        if v {
            self.0 = self.0 | ffi::EPOLLPRI
        } else {
            self.0 = self.0 & (0u32 ^ ffi::EPOLLPRI)
        };
    }
    pub fn epollerr(&self) -> bool {
        (self.0 & ffi::EPOLLERR) != 0
    }
    pub fn epollhup(&self) -> bool {
        (self.0 & ffi::EPOLLHUP) != 0
    }
}
impl From<libc::uint32_t> for IoEventMask {
    fn from(e: libc::uint32_t) -> IoEventMask {
        IoEventMask(e)
    }
}
impl From<IoEventMask> for libc::uint32_t {
    fn from(e: IoEventMask) -> libc::uint32_t {
        e.0
    }
}
impl std::fmt::Debug for IoEventMask {
    //EPOLLIN|EPOLLOUT|EPOLLRDHUP|EPOLLPRI //|EPOLLERR|EPOLLHUP|EPOLLET
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let mut v: Vec<&'static str> = Vec::new();
        if self.0 & ffi::EPOLLIN != 0 {
            v.push("EPOLLIN");
        }
        if self.0 & ffi::EPOLLOUT != 0 {
            v.push("EPOLLOUT");
        }
        if self.0 & ffi::EPOLLRDHUP != 0 {
            v.push("EPOLLRDHUP");
        }
        if self.0 & ffi::EPOLLPRI != 0 {
            v.push("EPOLLPRI");
        }
        if self.0 & ffi::EPOLLERR != 0 {
            v.push("EPOLLERR");
        }
        if self.0 & ffi::EPOLLHUP != 0 {
            v.push("EPOLLHUP");
        }
        write!(fmt, "IoEventMask({})", v.as_slice().connect("+"))
    }
}

#[derive(Copy,Clone,Debug)]
pub struct IoEventMaskBuilder(u32);
impl IoEventMaskBuilder {
    pub fn set_epollin(mut self, v: bool) -> IoEventMaskBuilder {
        if v {
            self.0 = self.0 | ffi::EPOLLIN
        } else {
            self.0 = self.0 & (0u32 ^ ffi::EPOLLIN)
        }
        self
    }
    pub fn set_epollout(mut self, v: bool) -> IoEventMaskBuilder {
        if v {
            self.0 = self.0 | ffi::EPOLLOUT
        } else {
            self.0 = self.0 & (0u32 ^ ffi::EPOLLOUT)
        }
        self
    }
    pub fn set_epollrdhup(mut self, v: bool) -> IoEventMaskBuilder {
        if v {
            self.0 = self.0 | ffi::EPOLLRDHUP
        } else {
            self.0 = self.0 & (0u32 ^ ffi::EPOLLRDHUP)
        }
        self
    }
    pub fn set_epollpri(mut self, v: bool) -> IoEventMaskBuilder {
        if v {
            self.0 = self.0 | ffi::EPOLLPRI
        } else {
            self.0 = self.0 & (0u32 ^ ffi::EPOLLPRI)
        };
        self
    }
    pub fn build(self) -> IoEventMask {
        IoEventMask(self.0)
    }
}

impl From<IoEventMaskBuilder> for IoEventMask {
    fn from(builder: IoEventMaskBuilder) -> IoEventMask {
        builder.build()
    }
}

pub enum IoEventTriggering {
    LevelTriggered(IoEventMask),
    EdgeTriggered(IoEventMask),
}

impl From<IoEventTriggering> for libc::uint32_t {
    fn from(e: IoEventTriggering) -> libc::uint32_t {
        let mut mask: libc::uint32_t;
        match e {
            IoEventTriggering::LevelTriggered(e) => mask = e.into(),
            IoEventTriggering::EdgeTriggered(e) => {
                mask = e.into();
                mask = mask | ffi::EPOLLET;
            }
        };
        mask
    }
}

pub struct IoEventSource<'e, FD> where FD: std::os::unix::io::AsRawFd {
    _e: std::marker::PhantomData<&'e Event>,
    s: ffi::sd_event_source,
    fd: FD,
    #[allow(dead_code)] cb: Box<FnMut(IoEventMask) -> i32 + 'e>,
}

impl<'e, FD> std::fmt::Debug for IoEventSource<'e, FD> where FD: std::os::unix::io::AsRawFd {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "IoEventSource{{ s: {:?} }}", self.s)
    }
}

impl<'e, FD> std::ops::Drop for IoEventSource<'e, FD> where FD: std::os::unix::io::AsRawFd {
    fn drop(&mut self) {
        unsafe {
            ffi::sd_event_source_unref(self.s);
        }
    }
}

impl<'e, FD> EventSource for IoEventSource<'e, FD> where FD: std::os::unix::io::AsRawFd {
    fn as_raw(&self) -> ffi::sd_event_source {
        self.s
    }
}

impl<'e, FD> IoEventSource<'e, FD> where FD: std::os::unix::io::AsRawFd {
    pub fn fd(&self) -> &FD {
        &self.fd
    }
    pub fn fd_mut(&mut self) -> &mut FD {
        &mut self.fd
    }
    pub fn signal(&self) -> Result<i32, Error> {
        let rv = unsafe {
            ffi::sd_event_source_get_signal(self.s)
        };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(rv)
    }
}


pub struct TimeEventSource<'e, EC> {
    _e: std::marker::PhantomData<&'e Event>,
    _ec: std::marker::PhantomData<EC>,
    s: ffi::sd_event_source,
    #[allow(dead_code)] cb: Box<FnMut(EC) -> i32 + 'e>,
}

impl<'e, EC> std::fmt::Debug for TimeEventSource<'e, EC> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str("timeeventsource")
    }
}

impl<'e, EC> std::ops::Drop for TimeEventSource<'e, EC> {
    fn drop(&mut self) {
        unsafe {
            ffi::sd_event_source_unref(self.s);
        }
    }
}

impl<'e, EC> EventSource for TimeEventSource<'e, EC> {
    fn as_raw(&self) -> ffi::sd_event_source {
        self.s
    }
}

impl<'e, EC: ClockTimestamp> TimeEventSource<'e, EC> {
    pub fn time(&self) -> Result<EC, Error> {
        let mut usec: libc::uint64_t = 0;
        let rv = unsafe {
            ffi::sd_event_source_get_time(self.s, &mut usec)
        };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(EC::from_usec(usec as u64))
    }
    pub fn set_time(&self, usec: EC) -> Result<(), Error> {
        let usec = usec.as_usec();
        let rv = unsafe {
            ffi::sd_event_source_set_time(self.s, usec as libc::uint64_t)
        };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(())
    }

    pub fn accuracy(&self) -> Result<time::Duration, Error> {
        let mut usec: libc::uint64_t = 0;
        let rv = unsafe {
            ffi::sd_event_source_get_time_accuracy(self.s, &mut usec)
        };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(time::Duration::microseconds(usec as i64))
    }
    pub fn set_accuracy(&self, usec: time::Duration) -> Result<(), Error> {
        let usec = match usec.num_microseconds() {
            None => return Err(Error::from_negative_errno(-EINVAL)),
            Some(usec) if usec < 0 => return Err(Error::from_negative_errno(-EINVAL)),
            Some(usec) => usec,
        };

        let rv = unsafe {
            ffi::sd_event_source_set_time_accuracy(self.s, usec as libc::uint64_t)
        };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(())
    }

    pub fn clock(&self) -> Result<Clock, Error> {
        let mut clock: ffi::clockid_t = 0;
        let rv = unsafe {
            ffi::sd_event_source_get_time_clock(self.s, &mut clock)
        };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(Clock::from_raw(clock))
    }
}


pub struct SignalEventSource<'e> {
    _e: std::marker::PhantomData<&'e Event>,
    s: ffi::sd_event_source,
    #[allow(dead_code)] cb: Box<FnMut(signalfd::SignalfdSigInfo) -> i32 + 'e>,
}

impl<'e> std::fmt::Debug for SignalEventSource<'e> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "SignalEventSource{{ s: {:?} }}", self.s)
    }
}

impl<'e> std::ops::Drop for SignalEventSource<'e> {
    fn drop(&mut self) {
        unsafe {
            ffi::sd_event_source_unref(self.s);
        }
    }
}

impl<'e> EventSource for SignalEventSource<'e> {
    fn as_raw(&self) -> ffi::sd_event_source {
        self.s
    }
}

impl<'e> SignalEventSource<'e> {
    pub fn signal(&self) -> Result<i32, Error> {
        let rv = unsafe {
            ffi::sd_event_source_get_signal(self.s)
        };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(rv)
    }
}


pub struct DeferEventSource<'e> {
    _e: std::marker::PhantomData<&'e Event>,
    s: ffi::sd_event_source,
    #[allow(dead_code)] cb: Box<FnMut() -> i32 + 'e>,
}

impl<'e> std::fmt::Debug for DeferEventSource<'e> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "DeferEventSource{{ s: {:?} }}", self.s)
    }
}

impl<'e> std::ops::Drop for DeferEventSource<'e> {
    fn drop(&mut self) {
        unsafe {
            ffi::sd_event_source_unref(self.s);
        }
    }
}

impl<'e> EventSource for DeferEventSource<'e> {
    fn as_raw(&self) -> ffi::sd_event_source {
        self.s
    }
}


pub struct PostEventSource<'e> {
    _e: std::marker::PhantomData<&'e Event>,
    s: ffi::sd_event_source,
    #[allow(dead_code)] cb: Box<FnMut() -> i32 + 'e>,
}

impl<'e> std::fmt::Debug for PostEventSource<'e> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "DeferEventSource{{ s: {:?} }}", self.s)
    }
}

impl<'e> std::ops::Drop for PostEventSource<'e> {
    fn drop(&mut self) {
        unsafe {
            ffi::sd_event_source_unref(self.s);
        }
    }
}

impl<'e> EventSource for PostEventSource<'e> {
    fn as_raw(&self) -> ffi::sd_event_source {
        self.s
    }
}


pub struct ExitEventSource<'e> {
    _e: std::marker::PhantomData<&'e Event>,
    s: ffi::sd_event_source,
    #[allow(dead_code)] cb: Box<FnMut() -> i32 + 'e>,
}

impl<'e> std::fmt::Debug for ExitEventSource<'e> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "ExitEventSource{{ s: {:?} }}", self.s)
    }
}

impl<'e> std::ops::Drop for ExitEventSource<'e> {
    fn drop(&mut self) {
        unsafe {
            ffi::sd_event_source_unref(self.s);
        }
    }
}

impl<'e> EventSource for ExitEventSource<'e> {
    fn as_raw(&self) -> ffi::sd_event_source {
        self.s
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use time;

    #[test]
    pub fn test_smoke() {
        let e: Event = Event::new().unwrap();
        println!("{:?}", e);
    }

    #[test]
    pub fn test_event_add_defer() {
        let mut e: Event = Event::new().unwrap();
        let s: DeferEventSource = e.add_defer(move || 0 ).unwrap();
        let cancontinue = e.run(time::Duration::milliseconds(5)).unwrap();
        assert_eq!(cancontinue, true);
        let cancontinue = e.run(time::Duration::milliseconds(5)).unwrap();
        assert_eq!(cancontinue, false);
        let cancontinue = e.run(time::Duration::milliseconds(5)).unwrap();
        assert_eq!(cancontinue, false);
        let _ = s;
    }
}
