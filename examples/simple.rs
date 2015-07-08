extern crate libsystemd;
extern crate time;

use time::Duration;

use libsystemd::*;


fn main() {
    let mut e = Event::new().unwrap();

    let _ = e.run(time::Duration::milliseconds(1));
    let now = e.now::<MonotonicClockTimestamp>().unwrap();
    println!("now: ({:?}", now);

    let t1 = e.add_time(now + Duration::seconds(1), Duration::milliseconds(500), move |d| {
        println!("t1 called! ({:?}", d);
        0
    }).unwrap();

    let t2 = e.add_time(now + Duration::seconds(3), Duration::milliseconds(500), move |d| {
        println!("t2 called! ({:?}", d);
        0
    }).unwrap();

    let mut e_for_exiting = e.clone();
    let t3 = e.add_time(now + Duration::seconds(5), Duration::milliseconds(500), move |d| {
        println!("t3 called! ({:?}", d);
        let _ = e_for_exiting.exit(42);
        0
    }).unwrap();

    let _ = e.run_loop();

    let _ = (t1, t2, t3);

    let ec = e.exit_code();
    println!("exit_code: {:?}", ec);
}
