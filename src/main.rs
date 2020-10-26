// we need both Duration and Instant from std::time
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::ops::Sub;

#[derive(PartialEq)]
enum Pixel{
    Empty,
    Vertical
}

fn clear_terminal_and_reset_cursor() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn main() {
    // we can't do [[Pixel::Empty; 70]; 70] because of https://github.com/rust-lang/rust/issues/49147
    // ;(
    // so we gotta make a Vector :)
    let mut display: Vec<Vec<Pixel>> = Vec::new();
    for y in 0..4 {
        display.push(Vec::new());
        for x in 0..10 {
            display[y].push(if y == 0 {Pixel::Vertical} else {Pixel::Empty});
        }
    }

    let FPS = 60;

    let program_start = Instant::now();
    for _ in 0..FPS {
        let now = Instant::now();

        clear_terminal_and_reset_cursor();

        println!("hi {}", display[0][1] == Pixel::Vertical);

        for row in display.iter() {
            for elem in row.iter() {
                println!();
                match elem{
                    Pixel::Empty => {
                        print!(" ");
                    }
                    Pixel::Vertical => {
                        print!("|");
                    }
                }
            }
        }

        // add some delay
        sleep(Duration::from_millis(3));

        println!("This frame took {:#?}", now.elapsed());
        sleep(Duration::from_secs_f32(1f32 / (FPS as f32)).sub(now.elapsed()));
        println!("Total frame time: {:#?}", now.elapsed());
    }

    println!("Extra time: {:#?}ms", (program_start.elapsed().as_secs_f64() - 1f64) * 1000f64);
}
