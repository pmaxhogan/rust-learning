// we need both Duration and Instant from std::time
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::ops::Sub;


fn main() {
    let FPS = 60;

    let program_start = Instant::now();
    for _ in 0..FPS {
        let now = Instant::now();

        println!("hi");

        // add some delay
        sleep(Duration::from_millis(3));

        println!("This frame took {:#?}", now.elapsed());
        sleep(Duration::from_secs_f32(1f32 / (FPS as f32)).sub(now.elapsed()));
        println!("Total frame time: {:#?}", now.elapsed());
    }

    println!("Extra time: {:#?}ms", (program_start.elapsed().as_secs_f64() - 1f64) * 1000f64);
}
