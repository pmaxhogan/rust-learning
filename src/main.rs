const WIDTH:usize = 10;
const HEIGHT:usize = 5;

// we need both Duration and Instant from std::time
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::ops::Sub;

// we need PartialEq to compare values
// Copy and Clone to be able to use the stack for storage (faster)
#[derive(PartialEq,Copy,Clone)]
enum Pixel{
    Empty,
    Vertical,
    Full
}

fn clear_terminal_and_reset_cursor() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn physics(display: &mut [[Pixel; HEIGHT]; WIDTH]) {
    display[3][1] = Pixel::Vertical;
}

fn draw(display: &[[Pixel; HEIGHT]; WIDTH]) {
    // we use y for the outer to allow us to do display[x][y] instead of display[y][x]
    for y in 0..display[0].len() {
        for x in 0..display.len() {
            let elem = display[x][y];
            match elem{
                Pixel::Empty => {
                    print!(" ");
                }
                Pixel::Vertical => {
                    print!("|");
                }
                Pixel::Full => {
                    print!("#");
                }
            }
        }
        println!();
    }
}

fn main() {
    let mut display = [[Pixel::Empty; HEIGHT]; WIDTH];

    display[0][1] = Pixel::Vertical;

    for x in 0..display.len() {
        display[x][display[0].len() - 1] = Pixel::Full;
    }


    let FPS = 60;

    let program_start = Instant::now();
    for _ in 0..FPS {
        let now = Instant::now();

        physics(&mut display);

        clear_terminal_and_reset_cursor();

        draw(&display);

        println!("This frame took {:#?}", now.elapsed());
        sleep(Duration::from_secs_f32(1f32 / (FPS as f32)).sub(now.elapsed()));
        println!("Total frame time: {:#?}", now.elapsed());
    }

    println!("Extra time: {:#?}ms", (program_start.elapsed().as_secs_f64() - 1f64) * 1000f64);
}
