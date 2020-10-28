const WIDTH:usize = 100;
const HEIGHT:usize = 50;
const ANIMATION_SPEED:usize = 90;
const INITIAL_SPACING:usize = 40;
const FPS:usize = 60;
const AUTO_RESPAWN:bool = true;

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
    jump_size: usize,
    gap: usize,
    spacing: usize,
    game_state: GameState,
    dead_timer: usize,
    score: usize
}

struct Player {
    y_pos: usize,
    x_pos: usize,
    jump_left: usize,
    fall_speed: f32
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
    // set up our display
    initialize_display(display);

    // add the player to our display
    display[state.player.x_pos][state.player.y_pos] = Pixel::Vertical;

    // draw obstacles
    // for each obstacle...
    for obstacle in &state.obstacles{
        // ...for each y-value in the obstacle...
        for y in obstacle.y..(obstacle.y + obstacle.height) {
            // ...if the coordinate is inside the screen...
            if obstacle.x < WIDTH && obstacle.y < HEIGHT {
                // ...write it to the display
                display[obstacle.x][y] = Pixel::Full;
            }
        }
    }

    // draw the death animation
    // it was originally a spiral that i spent a lot of time making
    // but then i realized that it was too slow
    // so i made it faster
    // and now it looks like it just goes in from the edges
    // even though it's still a spiral
    if state.game_state == GameState::DeathAnimation && state.dead_timer < WIDTH * HEIGHT {
        // increase our animation timer
        state.dead_timer += ANIMATION_SPEED;

        // get us a vector of x and y coordinates of the entire spiral, then slice it to the part
        // that we're supposed to show now
        let vec = &gen_spiral_vector(WIDTH, HEIGHT)[0..state.dead_timer.min(WIDTH * HEIGHT)];

        // for each coordinate...
        for (x, y) in vec{
            // ...write it to the display
            // note: *x and *y are used because the iterator of the vector gives us a reference, not the value
            // we need to dereference this because we need the number, so we use the * operator
            display[*x][*y] = Pixel::Full;
        }

        // if our death timer is higher than the number of "pixel"s in the display, go to the death state
        if state.dead_timer >= WIDTH * HEIGHT{
            state.game_state = GameState::Death;
        }
    }
}

fn physics(mut state: &mut State, tick: i32) {
    let mut passed_obstacle = false;
    for obstacle in &state.obstacles{
        if state.player.x_pos == obstacle.x {
            for y in obstacle.y..(obstacle.y + obstacle.height) {
                if state.player.y_pos == y {
                    state.game_state = GameState::DeathAnimation;
                    return;
                }
            }
            // we're still here?
            // it means that we were in the vertical area of an obstacle, yet didn't hit it
            // we need to ensure that we passed the other obstacle before increasing the score

            if passed_obstacle {
                state.score += 1;
            } else {
                passed_obstacle = true;
            }
        }
    }

    if tick % 2 == 0 {
        if state.player.jump_left > 0 {
            if state.player.y_pos > 0 {
                state.player.y_pos -= 1;
            }
            state.player.jump_left -= 1;
        }
    }

    if tick % 10 == 0{
        let fall_height = state.player.fall_speed.round() as usize;
        if state.player.y_pos + 1 < HEIGHT && state.player.jump_left == 0 {
            // ensure that we don't go further than the bottom
            state.player.y_pos = (state.player.y_pos + fall_height).min(HEIGHT - 1);

            // accelerate falling
            state.player.fall_speed += 0.3;
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

// sets up the display, state and tick variables
fn setup() -> ([[Pixel; 50]; 100], State, i32) {
    let mut display = [[Pixel::Empty; HEIGHT]; WIDTH];
    let mut state = State{
        player: Player {
            x_pos: 3,
            y_pos: 0,
            jump_left: 0,
            fall_speed: 1f32
        },
        obstacles: Vec::new(),
        jump_size: 3,
        gap: 3,
        spacing: 25,
        game_state: GameState::Playing,
        dead_timer: 0,
        score: 0
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

    (display, state, tick)
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

    let (mut display, mut state, mut tick) = setup();

    loop {
        let now = Instant::now();

        match rx.try_recv() {
            Ok(event) => {
                match event{
                    KeyEvent::Jump => {
                        if state.game_state == GameState::Playing {// if we're playing...
                            state.player.jump_left = state.jump_size;// ...start jumping...
                            state.player.fall_speed = 1f32; // ...and reset our fall speed
                        }else if state.game_state == GameState::DeathAnimation{// if we're showing the death animation...
                            // ...skip it
                            state.game_state = GameState::Death;
                        }
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
                physics(&mut state, tick);
            }
            GameState::Death => {
                if AUTO_RESPAWN {
                    // get new display, state and tick variables
                    let (display2, state2, tick2) = setup();

                    // replace the old ones with new ones
                    display = display2;
                    state = state2;
                    tick = tick2;
                } else {
                    display.iter_mut().for_each(|row| row.iter_mut().for_each(|pixel| *pixel = Pixel::Full));

                    let message = "You Died.";

                    let start_x = ((WIDTH - message.len()) as f32 / 2 as f32).floor() as usize;
                    let total = message.len();
                    for x in 0..total {
                        display[x + start_x][HEIGHT / 2] = Pixel::Char(message.chars().collect::<Vec<char>>()[x]);
                    }
                }
            }
            _ => {}
        }

        println!("\rScore: {}\r", state.score);
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
