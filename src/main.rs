use sfml::{
    graphics::{Color, RenderTarget, RenderWindow, Shape},
    window::{Event, Key, Style},
};
use sfml::graphics::{RectangleShape, Transformable, Text, Font};

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

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
struct Block{
    x: i32,
    y: i32,
    width: i32,
    height: i32
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

#[derive(Debug)]
struct State{
    player: Player,
    blocks: Vec<Block>
}

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const PLAYER_WIDTH: u32 = 50;
const PLAYER_HEIGHT: u32 = 50;
const PLAYER_MAX_JUMP: u8 = (PLAYER_HEIGHT * 4) as u8;
const BLOCK_SIZE:i32 = 50;

fn physics(state : &mut State){
    enum MovementDirection {
        X,
        Y
    }

    fn move_player(state: &mut State, direction: MovementDirection, delta: f32) -> bool {
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

        for block in &state.blocks {
            if state.player.x + delta_x < (block.x + block.width) as f32 && state.player.x + delta_x + PLAYER_WIDTH as f32 > block.x as f32 && state.player.y + delta_y < (block.y + block.height) as f32 && state.player.y + PLAYER_HEIGHT as f32 + delta_y > block.y as f32 {
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

    move_player(state, MovementDirection::X, match state.player.horiz_movement_direction {
        HorizMovementDirection::Left => -1f32,
        HorizMovementDirection::Right => 1f32,
        HorizMovementDirection::None => 0f32
    });

    let on_floor = !move_player(state, MovementDirection::Y, if state.player.is_jumping { -1f32 } else { 1f32 });

    if !state.player.is_jumping {
        state.player.jump_timeout = if on_floor { PLAYER_MAX_JUMP } else { 0 };
    }
}

fn main() {
    let mut seed_cache = HashMap::new();
    let density = |y: f64| -> f64 {
        ((-y as f64 / 200f64) + 1.5f64).cos() / 4f64 + 0.25f64
    };

    let mut is_block_at_coords = |x:i32, y:i32| -> bool {
        if x == 0 && y == 0{
            return false;
        }

        let seed = x as i64 + ((y as i64 ) << 32);
        return match seed_cache.get(&seed) {
            Some(is_block) => *is_block,
            None => {
                let this_density = density(y as f64);

                let is_block = StdRng::seed_from_u64(seed as u64).gen_range(0f64, 1f64) < this_density;
                seed_cache.insert(seed, is_block);
                is_block
            }
        }

    };

    let mut window = RenderWindow::new(
        (WIDTH, HEIGHT),
        "Game i guess",
        Style::CLOSE,
        &Default::default(),
    );

    window.set_vertical_sync_enabled(true);
    window.set_mouse_cursor_visible(false);

    let mut state = State{
        player: Player{
            x: 0.,
            y: 0.,
            is_jumping: false,
            jump_timeout: 0,
            horiz_movement_direction: HorizMovementDirection::None
        },
        blocks: Vec::new()
    };
    for x in 0..20 {
        for y in 0..20 {
            if is_block_at_coords(x, y) {
                state.blocks.push(Block {
                    x: x * BLOCK_SIZE,
                    y: y * BLOCK_SIZE,
                    width: BLOCK_SIZE,
                    height: BLOCK_SIZE
                });
            }
        }
    }

    let font = Font::from_file("resources/sansation.ttf").unwrap();

    'draw_loop:
    loop {
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

        physics(&mut state);
        physics(&mut state);
        physics(&mut state);

        let screen_x_min = ((state.player.x as i32 - (WIDTH / 2) as i32) / BLOCK_SIZE) as i32 - 1;
        let screen_y_min = ((state.player.y as i32 - (HEIGHT / 2) as i32) / BLOCK_SIZE) as i32 - 1;
        let screen_x_max = ((state.player.x as i32 + (WIDTH / 2) as i32) / BLOCK_SIZE) as i32 + 1;
        let screen_y_max = ((state.player.y as i32 + (HEIGHT / 2) as i32) / BLOCK_SIZE) as i32 + 1;

        for x in screen_x_min..screen_x_max{
            for y in screen_y_min..screen_y_max{
                if is_block_at_coords(x, y) && !state.blocks.iter().any(|&block| block.x == x * BLOCK_SIZE && block.y == y * BLOCK_SIZE){
                    state.blocks.push(Block {
                        x: x * BLOCK_SIZE,
                        y: y * BLOCK_SIZE,
                        width: BLOCK_SIZE,
                        height: BLOCK_SIZE
                    });
                }
            }
        }

        let mut i = 0;
        while i != state.blocks.len() {
            let block = state.blocks[i];
            if block.x / BLOCK_SIZE < screen_x_min || block.y / BLOCK_SIZE < screen_y_min || block.x / BLOCK_SIZE > screen_x_max || block.y / BLOCK_SIZE > screen_y_max {
                state.blocks.remove(i);
            } else {
                i += 1;
            }
        }

        window.clear(Color::BLACK);

        let mut player_rect = RectangleShape::new();
        player_rect.set_fill_color(Color::WHITE);
        player_rect.set_position(((WIDTH / 2) as f32, (HEIGHT / 2) as f32));
        player_rect.set_size((PLAYER_WIDTH as f32, PLAYER_HEIGHT as f32));
        window.draw(&player_rect);

        for block in &state.blocks {
            let mut rect = RectangleShape::new();
            rect.set_fill_color(Color::RED);
            rect.set_position((block.x as f32 - state.player.x + (WIDTH / 2) as f32, block.y as f32 - state.player.y + (HEIGHT / 2) as f32));
            rect.set_size((block.width as f32, block.height as f32));
            window.draw(&rect);
        }

        let y = state.player.y as f64 / BLOCK_SIZE as f64;
        let mut text = Text::new(&format!("X:{}\nY: {}\nDensity: {:.3}", state.player.x / BLOCK_SIZE as f32, y, density(y)), &font, 16);
        text.set_fill_color(Color::WHITE);
        text.set_position((0., 0.));
        window.draw(&text);

        // window.draw(&shape);
        window.display();
    }
}
