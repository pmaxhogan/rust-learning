const WIDTH:usize = 10;
const HEIGHT:usize = 5;
const FPS:usize = 60;

// we need both Duration and Instant from std::time
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::ops::Sub;
use std::thread;
use std::sync::mpsc;

struct Obstacle{
    x: usize,
    y: usize,
    height: usize,
}

struct State {
    player: Player,
    obstacles: Vec<Obstacle>,
}

struct Player {
    y_pos: usize,
}

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

fn initialize_display(display: &mut [[Pixel; HEIGHT]; WIDTH]) {
    for y in 0..display[0].len() {
        for x in 0..display.len() {
            display[x][y] = if y == HEIGHT {Pixel::Full} else {Pixel::Empty}
        }
    }
}

fn physics(display: &mut [[Pixel; HEIGHT]; WIDTH], state: &mut State, tick: i32) {
    initialize_display(display);
    display[3][state.player.y_pos] = Pixel::Vertical;
    for obstacle in &state.obstacles{
        for y in obstacle.y..(obstacle.y + obstacle.height) {
            if obstacle.x < WIDTH && obstacle.y < HEIGHT {
                display[obstacle.x][y] = Pixel::Full;
            }
        }
    }

    if tick % 10 == 0{
        if state.player.y_pos + 1 < HEIGHT {
            state.player.y_pos += 1;
        }else{
            panic!("Died!")
        }

        for idx in 0..state.obstacles.len() {
            let obstacle = &mut state.obstacles[idx];
            if obstacle.x < 1 {
                state.obstacles.remove(idx);
            } else {
                obstacle.x -= 1;
            }
        }
    }
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
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for i in 1..30 {
            println!("hi number {} from the spawned thread!", i);
            thread::sleep(Duration::from_millis(1));
            tx.send(String::from("hi")).unwrap();
        }
    });


    let mut display = [[Pixel::Empty; HEIGHT]; WIDTH];
    let mut state = State{
        player: Player {
            y_pos: 0
        },
        obstacles: Vec::new()
    };

    for x in 0..display.len() {
        display[x][display[0].len() - 1] = Pixel::Full;
    }

    state.obstacles.push(Obstacle {
        x: WIDTH,
        y: 0,
        height: 2
    });

    let program_start = Instant::now();

    let mut tick = 0;
    for _ in 0..FPS {
        let now = Instant::now();

        physics(&mut display, &mut state, tick);

        clear_terminal_and_reset_cursor();

        draw(&display);

        if let Ok(res) = rx.try_recv() {
            println!("{}", res);
        };

        println!("This frame took {:#?}", now.elapsed());
        sleep(Duration::from_secs_f32(1f32 / (FPS as f32)).sub(now.elapsed()));
        println!("Total frame time: {:#?}", now.elapsed());

        tick += 1;
    }

    println!("Extra time: {:#?}ms", (program_start.elapsed().as_secs_f64() - 1f64) * 1000f64);
}
