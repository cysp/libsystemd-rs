use std;
use time;

use ffi;


#[derive(Copy,Clone,Debug)]
pub enum Clock {
    Realtime,
    Monotonic,
    BoottimeAlarm,
    Unknown(i32),
}

impl Clock {
    pub fn from_raw(r: ffi::clockid_t) -> Clock {
        match r {
            ffi::CLOCK_REALTIME => Clock::Realtime,
            ffi::CLOCK_MONOTONIC => Clock::Monotonic,
            ffi::CLOCK_BOOTTIME_ALARM => Clock::BoottimeAlarm,
            r => Clock::Unknown(r),
        }
    }

    pub fn into_raw(self) -> ffi::clockid_t {
        match self {
            Clock::Realtime => ffi::CLOCK_REALTIME,
            Clock::Monotonic => ffi::CLOCK_MONOTONIC,
            Clock::BoottimeAlarm => ffi::CLOCK_BOOTTIME_ALARM,
            Clock::Unknown(r) => r as ffi::clockid_t,
        }
    }
}


pub trait ClockTimestamp {
    fn now() -> Self where Self: Sized {
        unsafe {
            let mut tp: ffi::timespec = std::mem::uninitialized();
            let clock = Self::clock();
            let rv = ffi::clock_gettime(clock.into_raw(), &mut tp);
            if rv < 0 {
                panic!("clock_gettime returned {}", rv);
            }
            let usec = tp.tv_sec * 1000000 + tp.tv_nsec / 1000;
            Self::from_usec(usec as u64)
        }
    }
    fn clock() -> Clock;
    fn from_usec(usec: u64) -> Self;
    fn as_usec(&self) -> u64;
}

#[derive(Copy,Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub struct RealtimeClockTimestamp(u64);
impl ClockTimestamp for RealtimeClockTimestamp {
    fn clock() -> Clock { Clock::Realtime }

    fn from_usec(usec: u64) -> RealtimeClockTimestamp {
        RealtimeClockTimestamp(usec)
    }

    fn as_usec(&self) -> u64 { self.0 }
}

impl std::ops::Add<time::Duration> for RealtimeClockTimestamp {
    type Output = RealtimeClockTimestamp;

    fn add(self, rhs: time::Duration) -> RealtimeClockTimestamp {
        let rhs_usec = match rhs.num_microseconds() {
            None => panic!(),
            Some(usec) => usec,
        };
        RealtimeClockTimestamp(((self.0 as i64) + rhs_usec) as u64)
    }
}

#[derive(Copy,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub struct MonotonicClockTimestamp(u64);
impl ClockTimestamp for MonotonicClockTimestamp {
    fn clock() -> Clock { Clock::Monotonic }

    fn from_usec(usec: u64) -> MonotonicClockTimestamp {
        MonotonicClockTimestamp(usec)
    }

    fn as_usec(&self) -> u64 { self.0 }
}

impl std::ops::Add<time::Duration> for MonotonicClockTimestamp {
    type Output = MonotonicClockTimestamp;

    fn add(self, rhs: time::Duration) -> MonotonicClockTimestamp {
        let rhs_usec = match rhs.num_microseconds() {
            None => panic!(),
            Some(usec) => usec,
        };
        MonotonicClockTimestamp(((self.0 as i64) + rhs_usec) as u64)
    }
}

impl std::fmt::Debug for MonotonicClockTimestamp {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let usec = self.as_usec();
        let (sec, remainder) = (usec / 1_000_000, usec % 1_000_000);
        write!(fmt, "MonotonicClockTimestamp({},{})", sec, remainder)
    }
}
