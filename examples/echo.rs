extern crate libsystemd;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use libsystemd::*;


fn main() {
    let mut e = Event::new().unwrap();

    let io1fd = unsafe { <std::fs::File as std::os::unix::io::FromRawFd>::from_raw_fd(0) };
    let io1events = IoEventMask::builder().set_epollin(true).build();
    let io1: Rc<RefCell<Option<IoEventSource<std::fs::File>>>> = Rc::new(RefCell::new(None));
    let io1_for_io1 = io1.clone();
    let mut e_for_exiting = e.clone();
    *(io1.borrow_mut()) = e.add_io(io1fd, IoEventTriggering::LevelTriggered(io1events), move |revents| {
        let ref mut io1_for_io1 = *io1_for_io1.borrow_mut();
        let io1 = match io1_for_io1 {
            &mut Some(ref mut io1) => io1,
            &mut None => return 0,
        };

        if revents.epollin() {
            use std::io::Read;

            let mut buf = [0u8; 32];
            match io1.fd_mut().read(&mut buf) {
                Err(e) => {
                    e_for_exiting.exit(1);
                }
                Ok(len) if len == 0 => {
                    e_for_exiting.exit(0);
                }
                Ok(len) => {
                    let s = <std::ffi::OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(&buf[..len]);
                    match s.to_str() {
                        Some(s) => println!("{}", s),
                        None => println!("{:?}", s),
                    }
                }
            }
        }
        0
    }).ok();

    let _ = e.run_loop();
}
