// we need both Duration and Instant from std::time
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::ops::Sub;


fn main() {
    let now = Instant::now();

    println!("hi");

    // add some delay
    sleep(Duration::from_millis(3));

    println!("This frame took {:#?}", now.elapsed());
    sleep(Duration::from_secs(2).sub( now.elapsed()));
}
