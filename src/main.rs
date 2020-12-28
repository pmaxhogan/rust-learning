use sfml::{
    graphics::{Color, RenderTarget, RenderWindow, Shape},
    window::{Event, Key, Style},
};
use sfml::graphics::{RectangleShape, Transformable};

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

use rand::Rng;
use std::time::Instant;
use uint::static_assertions::_core::time::Duration;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::Sender;

#[derive(Debug, Copy, Clone)]
struct Block{
    x: f32,
    y: f32,
    width: f32,
    height: f32
}

#[derive(Debug, Copy, Clone)]
enum HorizMovementDirection{
    None,
    Left,
    Right
}

#[derive(Debug, Copy, Clone)]
struct Player{
    x: f32,
    y: f32,
    is_jumping: bool,
    jump_timeout: u8,
    horiz_movement_direction: HorizMovementDirection
}

#[derive(Debug, Copy, Clone)]
struct State{
    player: Player
}

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const PLAYER_WIDTH: u32 = 50;
const PLAYER_HEIGHT: u32 = 50;
const PLAYER_MAX_JUMP: u8 = 100;

fn physics(state : &mut State, blocks: &Vec<Block>){
    enum MovementDirection {
        X,
        Y
    }

    fn move_player(state: &mut State, blocks: &Vec<Block>, direction: MovementDirection, delta: f32) -> bool {
        let mut can_move = true;
        let mut delta_x = 0f32;
        let mut delta_y = 0f32;
        match direction {
            MovementDirection::X => {
                delta_x = delta;
            }
            MovementDirection::Y => {
                delta_y = delta;
            }
        }

        for block in blocks {
            if state.player.x + delta_x < block.x + block.width && state.player.x + delta_x + PLAYER_WIDTH as f32 > block.x && state.player.y + delta_y < block.y + block.height && state.player.y + PLAYER_HEIGHT as f32 + delta_y > block.y {
                can_move = false;
                break;
            }
        }

        if can_move {
            state.player.x += delta_x;
            state.player.y += delta_y;
            return true
        }
        false
    }

    if state.player.jump_timeout == 0 {
        state.player.is_jumping = false;
    }

    if state.player.is_jumping {
        state.player.jump_timeout -= 1;
    }

    move_player(state, &blocks, MovementDirection::X, match state.player.horiz_movement_direction {
        HorizMovementDirection::Left => -1f32,
        HorizMovementDirection::Right => 1f32,
        HorizMovementDirection::None => 0f32
    });

    let on_floor = !move_player(state,  &blocks, MovementDirection::Y, if state.player.is_jumping { -1f32 } else { 1f32 });

    if !state.player.is_jumping {
        state.player.jump_timeout = if on_floor { PLAYER_MAX_JUMP } else { 0 };
    }
}

fn main() {

    let mut window = RenderWindow::new(
        (WIDTH, HEIGHT),
        "Game i guess",
        Style::CLOSE,
        &Default::default(),
    );

    // window.set_vertical_sync_enabled(true);
    window.set_mouse_cursor_visible(false);


    let (tx, rx) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    let (blocks_tx, blocks_rx) = mpsc::channel();

    let mut blocks = Vec::new();

    for _ in 0..200 {
        blocks.push(Block {
            x: rand::thread_rng().gen_range(0, WIDTH / 50) as f32 * 50.,
            y: rand::thread_rng().gen_range(0, HEIGHT / 50) as f32 * 50.,
            width: 50.,
            height: 50.
        });
    }

    let physics_thread = thread::spawn(move || {
        let mut blocks:[Block];
        let mut state = State{
            player: Player{
                x: 0.,
                y: 0.,
                is_jumping: false,
                jump_timeout: 0,
                horiz_movement_direction: HorizMovementDirection::None
            }
        };


        loop {
            blocks = blocks_rx.recv().unwrap().to_vec();

            match rx2.try_recv() {
                Ok(new_state) => state = new_state,
                _ => {}
            }

            let now = Instant::now();
            physics(&mut state, &blocks);
            thread::sleep(Duration::from_millis(10) - now.elapsed());

            tx.send(state);
        }
    });

    'draw_loop:
    loop {
        let mut state = rx.recv().unwrap();
        blocks_tx.send(blocks[..]);

        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed
                | Event::KeyPressed {
                    code: Key::Escape, ..
                } => break 'draw_loop,

                Event::KeyPressed {
                    code: Key::Up, ..
                } => state.player.is_jumping = true,
                Event::KeyReleased {
                    code: Key::Up, ..
                } => state.player.is_jumping = false,


                Event::KeyPressed {
                    code: Key::Left, ..
                } => state.player.horiz_movement_direction = HorizMovementDirection::Left,
                Event::KeyReleased {
                    code: Key::Left, ..
                } => state.player.horiz_movement_direction = HorizMovementDirection::None,

                Event::KeyPressed {
                    code: Key::Right, ..
                } => state.player.horiz_movement_direction = HorizMovementDirection::Right,
                Event::KeyReleased {
                    code: Key::Right, ..
                } => state.player.horiz_movement_direction = HorizMovementDirection::None,
                _ => {}
            }
        }

        tx2.send(state);

        window.clear(Color::BLACK);

        let mut player_rect = RectangleShape::new();
        player_rect.set_fill_color(Color::WHITE);
        player_rect.set_position(((WIDTH / 2) as f32, (HEIGHT / 2) as f32));
        player_rect.set_size((PLAYER_WIDTH as f32, PLAYER_HEIGHT as f32));
        window.draw(&player_rect);

        for block in blocks {
            let mut rect = RectangleShape::new();
            rect.set_fill_color(Color::RED);
            rect.set_position((block.x - state.player.x + (WIDTH / 2) as f32, block.y - state.player.y + (HEIGHT / 2) as f32));
            rect.set_size((block.width, block.height));
            window.draw(&rect);
        }

        // window.draw(&shape);
        window.display();
    }

    physics_thread.join().unwrap();
}
