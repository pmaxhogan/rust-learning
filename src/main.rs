// // termion is used for some raw terminal control things
// // it helps to improve portability
// extern crate termion;
//
// use std::io::{stdin, Write};
// use std::sync::mpsc;
// use std::thread;
// use std::thread::sleep;
// use std::time::{Duration, Instant};
//
// // RNG for obstacle heights
// use rand::Rng;
// use termion::event::Key;
// use termion::input::TermRead;
// use termion::raw::IntoRawMode;
//
// // constant variables (inlined by the compiler)
//
// // the width of the "screen" in terminal characters
// const WIDTH: usize = 100;
// // the height of the "screen" in terminal characters
// const HEIGHT: usize = 50;
// // how fast the death animation is
// const ANIMATION_SPEED: usize = 90;
// // how far away the first obstacle is
// const INITIAL_SPACING: usize = 50;
// // screen (and physics!) updates per second
// const FPS: usize = 60;
// const AUTO_RESPAWN: bool = true;// respawn after death animation
//
// // keyboard events
// // we need PartialEq to compare values
// #[derive(PartialEq, Copy, Clone)]
// enum KeyEvent {
//     Jump,
//     Quit,
// }
//
// #[derive(Debug)]
// struct Obstacle {
//     x: f64,
//     y: f64,
//     height: f64,
// }
//
// #[derive(PartialEq, Copy, Clone)]
// enum GameState {
//     Playing,
//     DeathAnimation,
//     Death,
// }
//
// struct State {
//     player: Player,
//     obstacles: Vec<Obstacle>,
//     // how high a jump is
//     jump_size: usize,
//     // vertical gap between obstacles
//     gap: usize,
//     // horizontal spacing between obstacles
//     spacing: usize,
//     game_state: GameState,
//     // death animation timer
//     dead_timer: usize,
//     score: usize,
// }
//
// struct Player {
//     y_pos: f64,
//     x_pos: f64,
//     // how much jump is left
//     jump_left: f64,
//     fall_speed: f64,// how fast the player is falling
// }
//
// // the "pixels" the display is made out of
// // Copy and Clone to be able to use the stack for storage (faster)
// #[derive(PartialEq, Copy, Clone)]
// enum Pixel {
//     // a predefined type,
//     Empty,
//     Vertical,
//     Full,
//     FullAlt,// alternate "full" character
// }
//
// // creates a "spiral" vector with a provided width and height
// // returns a vector containing coordinate positions for a spiral that begins in the upper-left
// // corner, and goes counterclockwise until the center
// fn gen_spiral_vector(width: usize, height: usize) -> Vec<(usize, usize)> {
//     gen_spiral_vector_with_offset(0, 0, width, height)
// }
//
// fn gen_spiral_vector_with_offset(x_offset: usize, y_offset: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
//     let mut v: Vec<(usize, usize)> = Vec::new();
//
//     // draw downwards
//     for y in 0..height {
//         v.push((x_offset, y + y_offset));
//     }
//
//     if width > 1 {
//         // to the right
//         for x in 1..width {
//             v.push((x + x_offset, height - 1 + y_offset));
//         }
//         // up
//         for y in (0..(height - 1)).rev() {
//             v.push((width - 1 + x_offset, y + y_offset));
//         }
//         // and to the left
//         if height > 1 {
//             for x in (1..(width - 1)).rev() {
//                 v.push((x + x_offset, y_offset));
//             }
//         }
//     }
//
//     // draw the next ring if needed
//     if width > 2 && height > 2 {
//         v.append(&mut gen_spiral_vector_with_offset(1 + x_offset, 1 + y_offset, width - 2, height - 2));
//     }
//
//     v
// }
//
// fn clear_terminal_and_reset_cursor() {
//     print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
// }
//
// // empties the display
// fn initialize_display(display: &mut [[Pixel; HEIGHT]; WIDTH]) {
//     for y in 0..display[0].len() {
//         for x in 0..display.len() {
//             display[x][y] = Pixel::Empty;
//         }
//     }
// }
//
// fn draw_display(display: &mut [[Pixel; HEIGHT]; WIDTH], mut state: &mut State) {
//     // set up our display
//     initialize_display(display);
//
//     // add the player to our display
//     display[state.player.x_pos.round() as usize][state.player.y_pos.round() as usize] = Pixel::Vertical;
//
//     // draw obstacles
//     // for each obstacle...
//     for obstacle in &state.obstacles {
//         let rounded_y = obstacle.y.round() as usize;
//
//         // ...for each y-value in the obstacle...
//         for y in rounded_y..(rounded_y + obstacle.height.round() as usize) {
//             // ...if the coordinate is inside the screen...
//             if (obstacle.x.round() as usize) < WIDTH && rounded_y < HEIGHT {
//                 // ...write it to the display
//                 display[obstacle.x.round() as usize][y] = Pixel::Full;
//             }
//         }
//     }
//
//     // draw the death animation
//     // it was originally a spiral that i spent a lot of time making
//     // but then i realized that it was too slow
//     // so i made it faster
//     // and now it looks like it just goes in from the edges
//     // even though it's still a spiral
//     if state.game_state == GameState::DeathAnimation && state.dead_timer < WIDTH * HEIGHT {
//         // increase our animation timer
//         state.dead_timer += ANIMATION_SPEED;
//
//         // get us a vector of x and y coordinates of the entire spiral, then slice it to the part
//         // that we're supposed to show now
//         let vec = &gen_spiral_vector(WIDTH, HEIGHT)[0..state.dead_timer.min(WIDTH * HEIGHT)];
//
//         // for each coordinate...
//         for (x, y) in vec {
//             // ...write it to the display
//             // note: *x and *y are used because the iterator of the vector gives us a reference, not the value
//             // we need to dereference this because we need the number, so we use the * operator
//             display[*x][*y] = Pixel::FullAlt;
//         }
//
//         // if our death timer is higher than the number of "pixel"s in the display, go to the death state
//         if state.dead_timer >= WIDTH * HEIGHT {
//             state.game_state = GameState::Death;
//         }
//     }
// }
//
// // handles movement, jumping, falling, collision, and score
// fn physics(mut state: &mut State, tick: i32) {
//     let mut passed_obstacle = false;
//
//     // for each obstacle,
//     for obstacle in &state.obstacles {
//         // if this obstacle is in the same vertical column as the player,
//         if state.player.x_pos == obstacle.x as f64 {
//             // check if they collide on any y value
//             if state.player.y_pos >= obstacle.y as f64 && state.player.y_pos <= obstacle.y + obstacle.height {
//                 // dead!
//                 state.game_state = GameState::DeathAnimation;
//                 return;
//             }
//
//             // we're still here?
//             // it means that we were in the vertical area of an obstacle, yet didn't hit it
//             // we need to ensure that we passed the other obstacle before increasing the score
//             if passed_obstacle {
//                 state.score += 1;
//             } else {
//                 passed_obstacle = true;
//             }
//         }
//     }
//
//     // if we need to jump
//     if state.player.jump_left > 0f64 {
//         // if we can go up (not at the top of the screen)
//         if state.player.y_pos > 0f64 {
//             // go up
//             state.player.y_pos -= 0.5;
//         }
//         state.player.jump_left -= 0.5;
//     }
//
//     let fall_height = state.player.fall_speed.round();
//     if state.player.y_pos + 1f64 < HEIGHT as f64 && state.player.jump_left <= 0f64 {
//         // ensure that we don't go further than the bottom
//         state.player.y_pos = (state.player.y_pos + fall_height).min((HEIGHT - 1) as f64);
//
//         // accelerate falling
//         state.player.fall_speed += 0.03;
//     }
//
//     // discard objects that have moved off the screen
//     state.obstacles.retain(|obstacle| &obstacle.x > &0f64);
//
//     // find the furthest away obstacle
//     let mut highest_x = 0f64;
//     for idx in 0..state.obstacles.len() {
//         let obstacle = &mut state.obstacles[idx];
//         obstacle.x -= 0.1;
//         highest_x = highest_x.max(obstacle.x);
//     }
//
//     // if the furthest obstacle is far enough away from the right edge, make a new one
//     if WIDTH as f64 > highest_x && WIDTH as f64 - highest_x >= state.spacing as f64 {
//         add_obstacle_pair(&mut state, WIDTH - 1);
//     }
// }
//
// // "renders" the display by writing it to the terminal
// fn render_display(display: &[[Pixel; HEIGHT]; WIDTH]) {
//     // draw the top border
//     // note: \r is needed in terminal raw mode to reset the x-position of the cursor
//     println!("{}", "@".repeat(WIDTH + 2) + "\r");
//
//     // we use y for the outer to allow us to do display[x][y] instead of display[y][x]
//     for y in 0..display[0].len() {
//         // draw the left border
//         print!("@");
//         for x in 0..display.len() {
//             let elem = display[x][y];
//
//             // draw the pixels depending on what they are
//             match elem {
//                 Pixel::Empty => {
//                     print!(" ");
//                 }
//                 Pixel::Vertical => {
//                     print!("|");
//                 }
//                 Pixel::Full => {
//                     print!("#");
//                 }
//                 Pixel::FullAlt => {
//                     print!("@");
//                 }
//             }
//         }
//         // draw the right border
//         println!("@\r");
//     }
//
//     // draw the bottom border
//     println!("{}", "@".repeat(WIDTH + 2) + "\r");
// }
//
// // make (return) a pair of obstacles at the given x position
// // requires the state to know the gap
// fn make_obstacle_pair(state: &State, x: usize) -> (Obstacle, Obstacle) {
//     // what height the top obstacle ends
//     let split_height = rand::thread_rng().gen_range(0, HEIGHT - state.gap);
//     (Obstacle {
//         x: x as f64,
//         y: 0f64,
//         height: split_height as f64,
//     },
//      Obstacle {
//          x: x as f64,
//          y: (split_height + state.gap) as f64,
//          height: (HEIGHT - split_height - state.gap) as f64,
//      })
// }
//
// // add a pair of obstacles to the state's obstacle list at the provided x position
// fn add_obstacle_pair(state: &mut State, x: usize) {
//     let pair = make_obstacle_pair(&state, x);
//     state.obstacles.push(pair.0);
//     state.obstacles.push(pair.1);
// }
//
// // sets up the display and state variables
// fn setup() -> ([[Pixel; 50]; 100], State) {
//     let display = [[Pixel::Empty; HEIGHT]; WIDTH];
//     let mut state = State {
//         player: Player {
//             x_pos: 15f64,
//             y_pos: 0f64,
//             jump_left: 0f64,
//             fall_speed: 0.1f64,
//         },
//         obstacles: Vec::new(),
//         jump_size: 3,
//         gap: 3,
//         spacing: 25,
//         game_state: GameState::Playing,
//         dead_timer: 0,
//         score: 0,
//     };
//
//     // find out how many obstacles that we need to draw initially
//     let num_obstacles = (WIDTH as f32 / state.spacing as f32).floor() as usize;
//     for x in 0..num_obstacles {
//         // we need to pull this var out because state is mutably borrowed below
//         let spacing = state.spacing;
//         add_obstacle_pair(&mut state, x * spacing + INITIAL_SPACING);
//     }
//
//     (display, state)
// }
//
// //noinspection RsRedundantElse,RsRedundantElse
// fn main() {
//     // the send and receiver for messages between our main (game) thread and our key thread
//     let (tx, rx) = mpsc::channel();
//
//     // use a separate thread for key input to prevent game code from interfering with input
//     // in hindsight, this was likely unnecessary, but it's too late and rust threads are cool anyway
//     #[allow(unused_variables)] let key_thread_handle = thread::spawn(move || {
//         // turns the current stdout into raw mode so we can do fancy things with drawing
//         let mut stdout = std::io::stdout().into_raw_mode()
//             .expect("Expected stdout to be a TTY capable of raw mode");
//         stdout.flush().expect("Expected to be able to write to stdout");
//
//         // stdout should still be around, right???
//         write!(stdout, "{}", termion::cursor::Hide).unwrap();
//
//         // runs whenever we get a key, blocking until we exit it
//         for c in stdin().keys() {
//             match c.expect("Expected to have a key?? idk") {
//                 Key::Char(' ') => {// pressed space
//                     tx.send(KeyEvent::Jump).unwrap();
//                 }
//                 Key::Char('q') => {// pressed q
//                     // tells the main thread to quit
//                     // note: this is not read immediately, rather the main thread reads it when it
//                     // gets around to it
//                     tx.send(KeyEvent::Quit).unwrap();
//
//                     // exit this loop (and thus the main thread)
//                     break;
//                 }
//                 _ => {}// all other keys are irrelevant
//             }
//         }
//
//         // i mean at this point if stdout doesn't work that's your fault not mine
//         stdout.flush().unwrap();
//
//         write!(stdout, "{}", termion::cursor::Show).unwrap();
//         stdout.suspend_raw_mode().unwrap();
//     });
//
//     // a counter incremented every loop
//     // used for physics
//     let mut tick = 0;
//
//     // setup the display and state
//     let (mut display, mut state) = setup();
//
//     // game loop
//     loop {
//         // start timing as soon as the loop starts
//         let now = Instant::now();
//
//         // did our game thread say anything?
//         match rx.try_recv() {
//             Ok(event) => {// we got a KeyEvent!
//                 match event {
//                     KeyEvent::Jump => {// jump key
//                         if state.game_state == GameState::Playing {// if we're playing...
//                             state.player.jump_left = state.jump_size as f64;// ...start jumping...
//                             state.player.fall_speed = 0.1f64; // ...and reset our fall speed
//                         } else if state.game_state == GameState::DeathAnimation {// if we're showing the death animation...
//                             // ...skip it
//                             state.game_state = GameState::Death;
//                         }
//                     }
//                     KeyEvent::Quit => {// quit key
//                         // exit the loop
//                         break;
//                     }
//                 };
//             }
//             _ => {}// no? ok
//         };
//
//         // clear terminal and reset cursor
//         clear_terminal_and_reset_cursor();
//
//         // note: this does not draw the display to the terminal, rather it puts stuff in &mut display given the state
//         draw_display(&mut display, &mut state);
//
//         // main game state machine
//         match state.game_state {
//             GameState::Playing => {
//                 physics(&mut state, tick);
//             }
//             GameState::Death => {
//                 // do we respawn after death?
//                 if AUTO_RESPAWN {
//                     // get new display, state and tick variables
//                     let (display2, state2) = setup();
//
//                     // replace the old ones with new ones
//                     display = display2;
//                     state = state2;
//                 }
//             }
//             _ => {}
//         }
//
//         // display our game score & other info at the top of the screen
//         println!("Flappy Rust \t Score: {} \t Press space to jump, q to quit\r", state.score);
//
//         // render the display
//         render_display(&display);
//
//         // sleep the amount of time needed to target the given framerate
//         // this means that no matter how long the game loop took, the game won't update slower or faster
//         // as long as it finishes before when the next game loop should run
//         // if it doesn't, then i need to optimize my game or your computer is too slow
//         let time = Duration::from_secs_f32(1f32 / (FPS as f32)).checked_sub(now.elapsed());
//         match time {
//             Some(duration) => { sleep(duration); }
//             None => {}
//         }
//
//         tick += 1;
//     }
//
//     // clear the screen on close
//     clear_terminal_and_reset_cursor();
// }

use sfml::{
    graphics::{Color, CustomShape, CustomShapePoints, RenderTarget, RenderWindow, Shape},
    system::Vector2f,
    window::{Event, Key, Style},
};
use sfml::graphics::{RectangleShape, Transformable};
use sfml::audio::listener::position;

// #[derive(Clone, Copy)]
// pub struct TriangleShape;
//
// impl CustomShapePoints for TriangleShape {
//     fn point_count(&self) -> u32 {
//         3
//     }
//
//     fn point(&self, point: u32) -> Vector2f {
//         match point {
//             0 => Vector2f { x: 20., y: 580. },
//             1 => Vector2f { x: 400., y: 20. },
//             2 => Vector2f { x: 780., y: 580. },
//             p => panic!("Non-existent point: {}", p),
//         }
//     }
// }

use std::thread;
use std::sync::mpsc;
use rand::Rng;
use std::time::{Instant, SystemTime, Duration};

#[derive(Debug)]
struct Block{
    x: f32,
    y: f32,
    width: f32,
    height: f32
}

#[derive(Debug)]
struct Player{
    x: f32,
    y: f32
}

#[derive(Debug)]
struct State{
    blocks: Vec<Block>,
    player: Player
}

const WIDTH: u32 = 500;
const HEIGHT: u32 = 500;
const PLAYER_WIDTH: u32 = 50;
const PLAYER_HEIGHT: u32 = 50;
const GRAVITY: f32 = 0.98;

fn physics(state : &mut State, elapsed : u128){
    if elapsed > u32::max_value() as u128 {
        panic!("Physics not called frequently enough!");
    }
    let elapsed = elapsed as f32;

    thread::sleep(Duration::from_millis(10));

    println!("elapsed: {}", elapsed);

    let mut delta_y = elapsed * GRAVITY;

    for allowed_delta_y in 0i32..(delta_y.round() as i32) {
        for block in &state.blocks {
            if state.player.x < block.x + block.width && state.player.x + PLAYER_WIDTH as f32 > block.x && state.player.y < block.y + allowed_delta_y as f32 + block.height && state.player.y + PLAYER_HEIGHT as f32 > block.y + allowed_delta_y as f32{
                if allowed_delta_y == 0{
                    delta_y = 0.;
                }else {
                    delta_y = delta_y.min((allowed_delta_y - 1) as f32);
                }
                break;
            }
        }
    }

    state.player.y += delta_y;
}

fn main() {
    let mut state = State{
        blocks: Vec::new(),
        player: Player{
            x: 0.,
            y: 0.
        }
    };

    for _ in 0..20 {
        state.blocks.push(Block {
            x: rand::thread_rng().gen_range(0, (WIDTH / 50)) as f32 * 50.,
            y: rand::thread_rng().gen_range(0, (HEIGHT / 50)) as f32 * 50.,
            width: 50.,
            height: 50.
        });
    }

    let mut window = RenderWindow::new(
        (WIDTH, HEIGHT),
        "Game i guess",
        Style::CLOSE,
        &Default::default(),
    );

    window.set_vertical_sync_enabled(true);
    window.set_mouse_cursor_visible(false);

    let mut now = SystemTime::now();

    'draw_loop:
    loop {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed
                | Event::KeyPressed {
                    code: Key::Escape, ..
                } => break 'draw_loop,
                _ => {}
            }
        }
        match now.elapsed(){
            Ok(elapsed) => {
                physics(&mut state, elapsed.as_millis());
                now = SystemTime::now();
            },
            Err(e) => {
                panic!("system time change! {:?}", e);
            }
        }


        window.clear(Color::BLACK);

        for block in &state.blocks {
            let mut rect = RectangleShape::new();
            rect.set_fill_color(Color::RED);
            rect.set_position((block.x, block.y));
            rect.set_size((block.width, block.height));
            window.draw(&rect);
        }

        let mut player_rect = RectangleShape::new();
        player_rect.set_fill_color(Color::WHITE);
        player_rect.set_position((state.player.x, state.player.y));
        player_rect.set_size((PLAYER_WIDTH as f32, PLAYER_HEIGHT as f32));
        window.draw(&player_rect);

        // window.draw(&shape);
        window.display();
    }
}
