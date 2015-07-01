use libc;
use std::ffi::{CString};
use std::ops::Range;
use std::os::unix::io::RawFd;

use {ffi, Error};


const SD_LISTEN_FDS_START: libc::c_int = 3;


pub fn listen_fds(unset_environment: bool) -> Result<Option<Range<RawFd>>, Error> {
    let rv = unsafe {
        ffi::sd_listen_fds(unset_environment as libc::c_int)
    };
    if rv < 0 {
        Err(Error::from_negative_errno(rv))
    } else if rv > 0 {
        Ok(Some(Range {
            start: SD_LISTEN_FDS_START,
            end: SD_LISTEN_FDS_START + rv,
        }))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod test_listen_fds {
    use std;
    use libc;

    #[test]
    pub fn test() {
        let pid = unsafe { libc::funcs::posix88::unistd::getpid() };
        let pid = format!("{}", pid);

        std::env::set_var("LISTEN_PID", &pid);
        std::env::set_var("LISTEN_FDS", "0");

        let listen_fds = super::listen_fds(true);
        assert_eq!(listen_fds.unwrap(), None);

        // let listen_fds = super::listen_fds(true);
        // assert_eq!(listen_fds.unwrap(), None);

        // std::env::set_var("LISTEN_PID", &pid);
        // std::env::set_var("LISTEN_FDS", "1");

        // let listen_fds = super::listen_fds(true);
        // assert_eq!(listen_fds.unwrap().unwrap().collect::<Vec<i32>>() , vec![ 3i32 ]);
    }

}


pub fn notify(unset_environment: bool, state: &str) -> Result<(), Error> {
    let state = try!(CString::new(state));
    let rv = unsafe {
        ffi::sd_notify(unset_environment as libc::c_int, state.as_bytes().as_ptr() as *const libc::c_char)
    };
    if rv < 0 {
        Err(Error::from_negative_errno(rv))
    } else {
        Ok(())
    }
}


pub fn watchdog_enabled(unset_environment: bool) -> Result<Option<u64>, Error> {
    let mut usec: libc::uint64_t = 0;
    let rv = unsafe {
        ffi::sd_watchdog_enabled(unset_environment as libc::c_int, &mut usec)
    };
    if rv < 0 {
        Err(Error::from_negative_errno(rv))
    } else if rv > 0 {
        Ok(Some(usec))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod test_watchdog_enabled {
    use std;

    #[test]
    pub fn test() {
        std::env::set_var("WATCHDOG_USEC", "500");

        let watchdog_enabled = super::watchdog_enabled(false);
        assert_eq!(watchdog_enabled.unwrap(), Some(500));

        let watchdog_enabled = super::watchdog_enabled(true);
        assert_eq!(watchdog_enabled.unwrap(), Some(500));

        let watchdog_enabled = super::watchdog_enabled(false);
        assert_eq!(watchdog_enabled.unwrap(), None);
    }

}
