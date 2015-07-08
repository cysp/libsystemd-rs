extern crate libsystemd;
extern crate time;
extern crate signal;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use time::Duration;

use libsystemd::*;
use signal::*;


fn ordinal_suffix(n: u32) -> &'static str
{
    if n >= 10 && n < 20 {
        Some("th")
    } else {
        None
    }.or_else(|| {
        match n % 10 {
            1 => Some("st"),
            2 => Some("nd"),
            3 => Some("rd"),
            _ => None,
        }
    }).unwrap_or("th")
}


fn main() {
    let mut e = Event::new().unwrap();

    let sigset = SigSet::builder()
        .add(1)
        .add(2)
        .build();
    // let mut sigset = SigSet::new();
    // sigset.add(1);
    // sigset.add(2);
    pthread_sigmask(SigMaskHow::Block(sigset), None).unwrap();

    let sigint_counter = Cell::new(0u32);
    let mut e_for_exiting = e.clone();
    let s1 = e.add_signal(2, move |si| {
        let sigint_count = sigint_counter.get() + 1;
        sigint_counter.set(sigint_count);
        println!("sigint ({}{}): {:?}", sigint_count, ordinal_suffix(sigint_count), si);
        if sigint_count >= 4 {
            let _ = e_for_exiting.exit(24);
        }
        0
    }).unwrap();

    let s2 = e.add_signal(1, move |si| {
        println!("sighup: {:?}", si);
        0
    }).unwrap();

    // let _ = e.run(time::Duration::milliseconds(1));
    let mut e_for_exiting = e.clone();

    let now = MonotonicClockTimestamp::now();

    let t1: Rc<RefCell<Option<TimeEventSource<_>>>> = Rc::new(RefCell::new(None));
    let (t1_for_t1, counter) = (t1.clone(), Cell::new(0u32));
    *(t1.borrow_mut()) = e.add_time(now + Duration::seconds(1), Duration::milliseconds(500), move |d| {
        let count = counter.get() + 1;
        counter.set(count);
        println!("t1 called for the {}{} time ({:?})", count, ordinal_suffix(count), d);
        if count >= 24 {
            let _ = e_for_exiting.exit(count as i32);
        } else {
            if let Some(ref t1) = *t1_for_t1.borrow_mut() {
                let _ = t1.set_time(d + Duration::seconds(1));
                let _ = t1.set_enabled(EventSourceEnabled::OneShot);
            }
        }
        0
    }).ok();

    let io1events = IoEventMask::builder().set_epollin(true).build();
    let io1 = e.add_io(0, IoEventTriggering::LevelTriggered(io1events), move |fd, revents| {
        println!("io1 called for fd: {}, revents: {:#?}", fd, revents);
        0
    }).unwrap();

    let _ = e.run_loop();

    // let _ = (s1, t1);

    let ec = e.exit_code();
    println!("exit_code: {:?}", ec);
}
