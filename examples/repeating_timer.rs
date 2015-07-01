extern crate libsystemd;
extern crate time;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use time::Duration;

use libsystemd::*;


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
    let mut e = Event::default();

    let _ = e.run(time::Duration::milliseconds(1));
    let mut e_for_exiting = e.clone();

    let now = e.now::<MonotonicEventClockTimestamp>().unwrap();

    let t1: Rc<RefCell<Option<TimeEventSource<_>>>> = Rc::new(RefCell::new(None));
    let (t1_for_t1, counter) = (t1.clone(), Cell::new(0u32));
    *(t1.borrow_mut()) = e.add_time(now + Duration::seconds(1), Duration::milliseconds(500), move |d| {
        let count = counter.get() + 1;
        counter.set(count);
        println!("t1 called for the {}{} time", count, ordinal_suffix(count));
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

    let _ = e.run_loop();

    let ec = e.exit_code();
    println!("exit_code: {:?}", ec);
}
