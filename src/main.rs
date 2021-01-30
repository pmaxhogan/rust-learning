use sfml::{
    graphics::{Color, RenderTarget, RenderWindow, Shape},
    window::{Event, Key, Style},
};
use sfml::graphics::{RectangleShape, Transformable, Text, Font};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

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
    jump_timeout: u8,// jump timeout is how many more frames they can jump
    horiz_movement_direction: HorizMovementDirection,
    jump_key_pressed: bool
}

#[derive(Debug)]
struct State{
    player: Player,
    blocks: Vec<Block>,
    quit: bool
}

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const PLAYER_WIDTH: u32 = 50;
const PLAYER_HEIGHT: u32 = 50;
const PLAYER_MAX_JUMP: u8 = (PLAYER_HEIGHT * 4) as u8;
const BLOCK_SIZE:i32 = 50;
const STOP_JUMP_ON_CEIL_HIT:bool = false;

/**
* h is [0, 360]
* s and v are [0, 1]
*/
fn hsv_to_rgb(h : u32, s : f32, v : f32) -> (u32, u32, u32) {
    if h > 360 || s < 0f32 || s > 1f32 || v < 0f32 || v > 1f32{
        panic!("Parameters to hsv_to_rgb() should be in the ranges [0, 360], [0, 1], and [0, 1] respectively")
    }

    let c = v * s;
    let x = c * (1f32 - ((h as f32 / 60f32) % 2f32 - 1f32).abs());
    let m = v - c;
    let (r_2, g_2, b_2) = match h {
        0 ..= 60 => (c, x, 0f32),
        61 ..= 120 => (x, c, 0f32),
        121 ..= 180 => (0f32, c, x),
        181 ..= 240 => (0f32, x, c),
        241 ..= 300 => (x, 0f32, c),
        301 ..= 360 => (c, 0f32, x),
        _ => {
            panic!("Not a valid H value!");
        }
    };

    let (r, g, b) = ((r_2 +m) * 255f32, (g_2 + m) * 255f32, (b_2 + m) * 255f32);

    return (r.round() as u32, g.round() as u32, b.round() as u32);
}

fn main() {
    // determines the density of the blocks at a given y-level
    // a density of .25 means that 1/4 spaces are filled with a block at that y-level
    // https://www.desmos.com/calculator/bxlbewssw0
    let density = |y: f64| -> f64 {
        ((-y as f64 / 200f64) + 1.5f64).cos() / 4f64 + 0.25f64
    };

    // returns true if there is a block at the provided coords
    // uses a hashmap as a cache because the random thing is kinda slow
    let mut seed_cache = HashMap::new();
    let mut is_block_at_coords = move | x:i32, y:i32| -> bool {
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

    let mut physics = move|state: &mut State| -> () {
        // the bounding coordinates of the blocks that can be seen on the screen
        // the - 1 and + 1 ensure that blocks are loaded just before they can be seen to prevent pop-in / pop-out
        let screen_x_min = ((state.player.x as i32 - (WIDTH / 2) as i32) / BLOCK_SIZE) as i32 - 1;
        let screen_y_min = ((state.player.y as i32 - (HEIGHT / 2) as i32) / BLOCK_SIZE) as i32 - 1;
        let screen_x_max = ((state.player.x as i32 + (WIDTH / 2) as i32) / BLOCK_SIZE) as i32 + 1;
        let screen_y_max = ((state.player.y as i32 + (HEIGHT / 2) as i32) / BLOCK_SIZE) as i32 + 1;

        // add new blocks if needed
        for x in screen_x_min..screen_x_max {
            for y in screen_y_min..screen_y_max {
                // don't include already added blocks
                if is_block_at_coords(x, y) && !state.blocks.iter().any(|&block| block.x == x * BLOCK_SIZE && block.y == y * BLOCK_SIZE) {
                    state.blocks.push(Block {
                        x: x * BLOCK_SIZE,
                        y: y * BLOCK_SIZE,
                        width: BLOCK_SIZE,
                        height: BLOCK_SIZE
                    });
                }
            }
        }

        // removes blocks that can't be rendered to improve performance
        let mut i = 0;
        while i != state.blocks.len() {
            let block = state.blocks[i];
            if block.x / BLOCK_SIZE < screen_x_min || block.y / BLOCK_SIZE < screen_y_min || block.x / BLOCK_SIZE > screen_x_max || block.y / BLOCK_SIZE > screen_y_max {
                state.blocks.remove(i);
            } else {
                i += 1;
            }
        }


        enum MovementDirection {
            X,
            Y
        }

        // attempts to move the player in the given direction by the given delta
        // delta must be +/- 1
        // returns true if successful
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

        let on_vert_surface = !move_player(state, MovementDirection::Y, if state.player.is_jumping { -1f32 } else { 1f32 });

        if on_vert_surface && !state.player.is_jumping{
            state.player.jump_timeout = PLAYER_MAX_JUMP;
        }

        if !on_vert_surface && !state.player.is_jumping  {
            state.player.jump_timeout = 0;
        }

        if STOP_JUMP_ON_CEIL_HIT {
            if on_vert_surface && state.player.is_jumping {
                state.player.jump_timeout = 0;
            }
        }
    };

    let mut window = RenderWindow::new(
        (WIDTH, HEIGHT),
        "Game",
        Style::NONE,
        &Default::default(),
    );

    // v-sync eliminates screen tearing at the cost of latency and performance
    window.set_vertical_sync_enabled(true);
    window.set_mouse_cursor_visible(false);
    window.set_key_repeat_enabled(false);


    // include_bytes! builds this font into our executable, meaning that we do not need to bring
    // a resources/ folder around. very handy!
    // we unwrap because it should crash if the font isn't there (a bug)
    let font = Font::from_memory(include_bytes!("resources/sansation.ttf")).unwrap();


    let state = State{
        player: Player{
            x: 0.,
            y: 0.,
            is_jumping: false,
            jump_key_pressed: false,
            jump_timeout: 0,
            horiz_movement_direction: HorizMovementDirection::None
        },
        blocks: Vec::new(),
        quit: false
    };


    // not completely sure how Arc<Mutex<T>> works but it does work
    // see https://doc.rust-lang.org/book/ch16-03-shared-state.html
    let state_holder = Arc::new(Mutex::new(state));

    let physics_thread;
    {
        let our_state_holder = Arc::clone(&state_holder);
        physics_thread = thread::spawn(move || {
            loop {
                // we need this block to ensure that our MutexGuard goes out of scope (and is freed)
                // before we sleep. if we sleep before releasing the lock, then we will basically
                // get the lock as soon we release it, preventing the main thread from getting it!
                {
                    let mut state_guard = our_state_holder.lock().unwrap();
                    let state = &mut *state_guard;

                    if state.quit{ break; }

                    physics(state);
                    physics(state);
                }
                // TODO: make this calculated from above time
                thread::sleep(Duration::from_millis(5));
            }
        });
    }

    'draw_loop:
    loop {
        // see above for why we have a block here
        {
            let our_state_holder = Arc::clone(&state_holder);
            let mut state_guard = our_state_holder.lock().unwrap();

            let state = &mut *state_guard;

            // did we get any key events?
            while let Some(event) = window.poll_event() {
                match event {
                    Event::Closed
                    | Event::KeyPressed {
                        code: Key::Escape, ..
                    } => break 'draw_loop,

                    Event::KeyPressed {
                        code: Key::Up, ..
                    } => state.player.jump_key_pressed = true,
                    Event::KeyReleased {
                        code: Key::Up, ..
                    } => state.player.jump_key_pressed = false,

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

            state.player.is_jumping = state.player.jump_key_pressed;

            window.clear(Color::BLACK);

            // draw the player
            let mut player_rect = RectangleShape::new();
            player_rect.set_fill_color(Color::WHITE);
            player_rect.set_position(((WIDTH / 2) as f32, (HEIGHT / 2) as f32));
            player_rect.set_size((PLAYER_WIDTH as f32, PLAYER_HEIGHT as f32));
            window.draw(&player_rect);

            // draw blocks
            for block in &state.blocks {
                let mut rect = RectangleShape::new();
                let (r, g, b) = hsv_to_rgb(((block.x.abs() as u32) / 100) % 360, 1.0, 1.0);
                rect.set_fill_color(Color::rgb(r as u8, g as u8, b as u8));
                rect.set_position((block.x as f32 - state.player.x + (WIDTH / 2) as f32, block.y as f32 - state.player.y + (HEIGHT / 2) as f32));
                rect.set_size((block.width as f32, block.height as f32));
                window.draw(&rect);
            }

            // draw height info
            let y = state.player.y as f64 / BLOCK_SIZE as f64;
            let mut text = Text::new(&format!("X:{}\nY: {}\nDensity: {:.3}", state.player.x / BLOCK_SIZE as f32, y, density(y)), &font, 16);
            text.set_fill_color(Color::WHITE);
            text.set_position((0., 0.));
            window.draw(&text);
        }

        window.display();
    }

    (*state_holder.lock().unwrap()).quit = true;
    physics_thread.join().unwrap();
}
