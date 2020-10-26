const WIDTH:usize = 100;
const HEIGHT:usize = 50;
const FPS:usize = 60;

extern crate termion;

// we need both Duration and Instant from std::time
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::ops::Sub;
use std::thread;
use std::sync::mpsc;
use std::io::{Write, stdin};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

struct Obstacle{
    x: usize,
    y: usize,
    height: usize,
}

struct State {
    player: Player,
    obstacles: Vec<Obstacle>,
    should_jump: bool,
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

    if state.should_jump && state.player.y_pos > 1 {
        state.player.y_pos -= 2;
        state.should_jump = false;
    }

    if tick % 10 == 0{
        if state.player.y_pos + 1 < HEIGHT {
            state.player.y_pos += 1;
        }else{
            state.player.y_pos = 0;
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
        println!("\r");
    }
}

fn main() {
    let (tx, rx) = mpsc::channel();
    let (exit_tx, exit_rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let mut stdout = std::io::stdout().into_raw_mode().unwrap();
        stdout.flush().unwrap();

        write!(stdout, "{}", termion::cursor::Hide).unwrap();

        for c in stdin().keys() {
            match exit_rx.try_recv() {
                Ok(val) => { break; },
                Err(_) => {}
            }


            match c.unwrap() {
                Key::Char(' ') => {
                    tx.send(()).unwrap();
                },
                _ => { }
            }
        }


        stdout.flush().unwrap();

        write!(stdout, "{}", termion::cursor::Show).unwrap();
        stdout.suspend_raw_mode().unwrap();
    });


    let mut display = [[Pixel::Empty; HEIGHT]; WIDTH];
    let mut state = State{
        player: Player {
            y_pos: 0
        },
        obstacles: Vec::new(),
        should_jump: false
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
    for _ in 0..1000 {
        let now = Instant::now();

        let should_jump: bool = match rx.try_recv() {
            Ok(x) => { true }
            _ => { false }
        };

        state.should_jump = should_jump;

        physics(&mut display, &mut state, tick);

        clear_terminal_and_reset_cursor();

        draw(&display);


        println!("\rThis frame took {:#?}", now.elapsed());
        sleep(Duration::from_secs_f32(1f32 / (FPS as f32)).sub(now.elapsed()));
        println!("\rTotal frame time: {:#?}", now.elapsed());

        tick += 1;
    }

    println!("\rExtra time: {:#?}ms", (program_start.elapsed().as_secs_f64() - 1f64) * 1000f64);

    println!("\r");
    exit_tx.send(()).unwrap();
    handle.join().unwrap();
}
