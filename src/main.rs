const WIDTH:usize = 100;
const HEIGHT:usize = 50;
const ANIMATION_SPEED:usize = 40;
const INITIAL_SPACING:usize = 40;
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
use rand::Rng;

#[derive(PartialEq,Copy,Clone)]
enum KeyEvent{
    Jump,
    Quit
}

#[derive(Debug)]
struct Obstacle{
    x: usize,
    y: usize,
    height: usize,
}

#[derive(PartialEq,Copy,Clone)]
enum GameState{
    Playing,
    DeathAnimation,
    Death
}

struct State {
    player: Player,
    obstacles: Vec<Obstacle>,
    jump_left: usize,
    jump_size: usize,
    gap: usize,
    spacing: usize,
    game_state: GameState,
    dead_timer: usize,
}

struct Player {
    y_pos: usize,
    x_pos: usize
}

// we need PartialEq to compare values
// Copy and Clone to be able to use the stack for storage (faster)
#[derive(PartialEq,Copy,Clone)]
enum Pixel{
    Empty,
    Vertical,
    Full,
    Char(char),
}

// creates a "spiral" vector with a provided width and height
// returns a vector containing coordinate positions for a spiral that begins in the upper-left
// corner, and goes counterclockwise until the center
fn gen_spiral_vector(width: usize, height: usize) -> Vec<(usize, usize)> {
    gen_spiral_vector_with_offset(0, 0, width, height)
}
fn gen_spiral_vector_with_offset(x_offset: usize, y_offset: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let mut v: Vec<(usize, usize)> = Vec::new();

    for y in 0..height {
        v.push((x_offset, y + y_offset));
    }

    if width > 1 {
        for x in 1..width {
            v.push((x + x_offset, height - 1 + y_offset));
        }
        for y in (0..(height - 1)).rev() {
            v.push((width - 1 + x_offset, y + y_offset));
        }
        if height > 1 {
            for x in (1..(width - 1)).rev() {
                v.push((x + x_offset, y_offset));
            }
        }
    }

    if width > 2 && height > 2 {
        v.append(&mut gen_spiral_vector_with_offset(1 + x_offset, 1 + y_offset, width - 2, height - 2));
    }

    v
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

fn draw_display(display: &mut [[Pixel; HEIGHT]; WIDTH], mut state: &mut State){
    initialize_display(display);
    display[state.player.x_pos][state.player.y_pos] = Pixel::Vertical;
    for obstacle in &state.obstacles{
        for y in obstacle.y..(obstacle.y + obstacle.height) {
            if obstacle.x < WIDTH && obstacle.y < HEIGHT {
                display[obstacle.x][y] = Pixel::Full;
            }
        }
    }

    if state.game_state == GameState::DeathAnimation && state.dead_timer < WIDTH * HEIGHT {
        state.dead_timer += ANIMATION_SPEED;

        let vec = &gen_spiral_vector(WIDTH, HEIGHT)[0..state.dead_timer.min(WIDTH * HEIGHT)];

        for (x, y) in vec{
            display[*x][*y] = Pixel::Full;
        }

        if state.dead_timer >= WIDTH * HEIGHT{
            state.game_state = GameState::Death;
        }
    }
}

fn physics(display: &mut [[Pixel; HEIGHT]; WIDTH], mut state: &mut State, tick: i32) {
    display[state.player.x_pos][state.player.y_pos] = Pixel::Vertical;
    for obstacle in &state.obstacles{
        for y in obstacle.y..(obstacle.y + obstacle.height) {
            if state.player.x_pos == obstacle.x && state.player.y_pos == y {
                state.game_state = GameState::DeathAnimation;
                return;
            }
        }
    }

    if state.jump_left > 0 {
        if state.player.y_pos > 0 {
            state.player.y_pos -= 1;
        }
        state.jump_left -= 1;
    }

    if tick % 10 == 0{
        if state.player.y_pos + 1 < HEIGHT && state.jump_left == 0 {
            state.player.y_pos += 1;
        }

        state.obstacles.retain(|obstacle| &obstacle.x > &0);

        let mut highest_x = 0;
        for idx in 0..state.obstacles.len() {
            let obstacle = &mut state.obstacles[idx];
            obstacle.x -= 1;
            highest_x = highest_x.max(obstacle.x);
        }

        // if the furthest obstacle is far enough away from the right edge, make a new one
        if WIDTH > highest_x && WIDTH - highest_x >= state.spacing {
            add_obstacle_pair(&mut state, WIDTH - 1);
        }
    }
}

fn render_display(display: &[[Pixel; HEIGHT]; WIDTH]) {
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
                Pixel::Char(c) => {
                    print!("{}", c);
                }
            }
        }
        println!("\r");
    }
}

fn make_obstacle_pair(state:&State, x:usize) -> (Obstacle, Obstacle){
    let split_height = rand::thread_rng().gen_range(0, HEIGHT - state.gap);
    (Obstacle {
        x,
        y: 0,
        height: split_height
    },
     Obstacle {
         x,
         y: split_height + state.gap,
         height: HEIGHT - split_height - state.gap
     })
}

fn add_obstacle_pair(state: &mut State, x:usize){
    let pair = make_obstacle_pair(&state, x);
    state.obstacles.push(pair.0);
    state.obstacles.push(pair.1);
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
                Ok(_) => { break; },
                Err(_) => {}
            }


            match c.unwrap() {
                Key::Char(' ') => {
                    tx.send(KeyEvent::Jump).unwrap();
                },
                Key::Char('q') => {
                    tx.send(KeyEvent::Quit).unwrap();
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
            x_pos: 3,
            y_pos: 0
        },
        obstacles: Vec::new(),
        jump_left: 0,
        jump_size: 2,
        gap: 3,
        spacing: 25,
        game_state: GameState::Playing,
        dead_timer: 0
    };

    for x in 0..display.len() {
        display[x][display[0].len() - 1] = Pixel::Full;
    }

    let num_obstacles = (WIDTH as f32 / state.spacing as f32).floor() as usize;
    for x in 0..num_obstacles{
        // we need to pull this var out because state is mutably borrowed below
        let spacing = state.spacing;
        add_obstacle_pair(&mut state, x * spacing + INITIAL_SPACING);
    }

    let mut tick = 0;
    loop {
        let now = Instant::now();

        match rx.try_recv() {
            Ok(event) => {
                match event{
                    KeyEvent::Jump => {
                        state.jump_left = state.jump_size;
                    },
                    KeyEvent::Quit => {
                        break;
                    }
                };
            }
            _ => {}
        };

        clear_terminal_and_reset_cursor();

        draw_display(&mut display, &mut state);

        match state.game_state {
            GameState::Playing => {
                physics(&mut display, &mut state, tick);
            }
            GameState::Death => {
                display.iter_mut().for_each(|row| row.iter_mut().for_each(|pixel| *pixel = Pixel::Full));

                let message = "You Suck.";

                let start_x = ((WIDTH - message.len()) as f32 / 2 as f32).floor() as usize;
                let total = message.len();
                for x in 0..total{
                    display[x + start_x][HEIGHT / 2] = Pixel::Char(message.chars().collect::<Vec<char>>()[x]);
                }
            }
            _ => {}
        }

        render_display(&display);


        // println!("\rThis frame took {:#?}", now.elapsed());
        sleep(Duration::from_secs_f32(1f32 / (FPS as f32)).sub(now.elapsed()));
        // println!("\rTotal frame time: {:#?}", now.elapsed());

        tick += 1;
    }

    println!("\rPress any key to continue to your terminal.\r");
    exit_tx.send(()).unwrap();
    handle.join().unwrap();
}
