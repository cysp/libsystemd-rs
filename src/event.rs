use libc;
use libc::consts::os::posix88::EINVAL;
use std;
// use std::ffi::{CString};
// use std::ops::Range;
// use std::os::unix::io::RawFd;
use time;

use {ffi, Error};

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

#[derive(Copy,Clone,Debug)]
pub enum EventClock {
    Realtime,
    Monotonic,
    BoottimeAlarm,
    Unknown(i32),
}
impl EventClock {
    fn from_raw(r: ffi::clockid_t) -> EventClock {
        match r {
            ffi::CLOCK_REALTIME => EventClock::Realtime,
            ffi::CLOCK_MONOTONIC => EventClock::Monotonic,
            ffi::CLOCK_BOOTTIME_ALARM => EventClock::BoottimeAlarm,
            r => EventClock::Unknown(r),
        }
    }
    fn into_raw(self) -> ffi::clockid_t {
        match self {
            EventClock::Realtime => ffi::CLOCK_REALTIME,
            EventClock::Monotonic => ffi::CLOCK_MONOTONIC,
            EventClock::BoottimeAlarm => ffi::CLOCK_BOOTTIME_ALARM,
            EventClock::Unknown(r) => r as ffi::clockid_t,
        }
    }
}

pub trait EventClockTimestamp {
    fn clock() -> EventClock;
    fn from_usec(usec: u64) -> Self;
    fn usec(&self) -> u64;
}

#[derive(Copy,Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub struct RealtimeEventClockTimestamp(u64);
impl EventClockTimestamp for RealtimeEventClockTimestamp {
    fn clock() -> EventClock { EventClock::Realtime }
    fn from_usec(usec: u64) -> RealtimeEventClockTimestamp {
        RealtimeEventClockTimestamp(usec)
    }
    fn usec(&self) -> u64 { self.0 }
}

impl std::ops::Add<time::Duration> for RealtimeEventClockTimestamp {
    type Output = RealtimeEventClockTimestamp;
    fn add(self, rhs: time::Duration) -> RealtimeEventClockTimestamp {
        let rhs_usec = match rhs.num_microseconds() {
            None => panic!(),
            Some(usec) => usec,
        };
        RealtimeEventClockTimestamp(((self.0 as i64) + rhs_usec) as u64)
    }
}

#[derive(Copy,Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub struct MonotonicEventClockTimestamp(u64);
impl EventClockTimestamp for MonotonicEventClockTimestamp {
    fn clock() -> EventClock { EventClock::Monotonic }
    fn from_usec(usec: u64) -> MonotonicEventClockTimestamp {
        MonotonicEventClockTimestamp(usec)
    }
    fn usec(&self) -> u64 { self.0 }
}

impl std::ops::Add<time::Duration> for MonotonicEventClockTimestamp {
    type Output = MonotonicEventClockTimestamp;
    fn add(self, rhs: time::Duration) -> MonotonicEventClockTimestamp {
        let rhs_usec = match rhs.num_microseconds() {
            None => panic!(),
            Some(usec) => usec,
        };
        MonotonicEventClockTimestamp(((self.0 as i64) + rhs_usec) as u64)
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

impl Default for Event {
    fn default() -> Event {
        let mut e: ffi::sd_event = 0 as ffi::sd_event;
        let rv = unsafe { ffi::sd_event_default(&mut e) };
        if rv < 0 {
            panic!("sd_event_default() failed");
        }
        Event {
            e: e,
        }
    }
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

    pub fn add_time<'z, 's, EC, F>(&'z mut self, usec: EC, accuracy: time::Duration, callback: F) -> Result<TimeEventSource<'s, EC>, Error>
        where 'e: 'z, 'e: 's,
            EC: EventClockTimestamp,
            F: 's + FnMut(EC) -> i32
    {
        let (clock, usec) = (EC::clock(), usec.usec());
        let accuracy = match accuracy.num_microseconds() {
            None => return Err(Error::from_negative_errno(-EINVAL)),
            Some(accuracy) if accuracy < 0 => return Err(Error::from_negative_errno(-EINVAL)),
            Some(accuracy) => accuracy,
        };

        let callback = Box::new(callback);
        let userdata = &*callback as *const _ as *const libc::c_void;

        extern fn event_source_time_tramp<EC, F>(_: ffi::sd_event_source, usec: libc::uint64_t, userdata: *const libc::c_void) -> libc::c_int
            where EC: EventClockTimestamp, F: FnMut(EC) -> i32
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

    pub fn now<EC: EventClockTimestamp = MonotonicEventClockTimestamp>(&'e self) -> Result<EC, Error> {
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

impl<'e, EC: EventClockTimestamp> TimeEventSource<'e, EC> {
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
        let usec = usec.usec();
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

    pub fn clock(&self) -> Result<EventClock, Error> {
        let mut clock: ffi::clockid_t = 0;
        let rv = unsafe {
            ffi::sd_event_source_get_time_clock(self.s, &mut clock)
        };
        if rv < 0 {
            return Err(Error::from_negative_errno(rv))
        }
        Ok(EventClock::from_raw(clock))
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
        let e: Event = Default::default();
        println!("{:?}", e);
    }

    #[test]
    pub fn test_event_add_defer() {
        let mut e: Event = Default::default();
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
