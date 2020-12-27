
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
    blocks: Vec<Block>
}

const WIDTH: u32 = 500;
const HEIGHT: u32 = 500;
const PLAYER_WIDTH: u32 = 10;
const PLAYER_HEIGHT: u32 = 10;

fn main() {
    let mut state = State{
        blocks: Vec::new()
    };
    let mut player = Player{
        x: 0.,
        y: 0.
    };
    for i in 0..100 {
        state.blocks.push(Block {
            x: rand::thread_rng().gen_range(0, (WIDTH / 10)) as f32 * 10.,
            y: rand::thread_rng().gen_range(0, (HEIGHT / 10)) as f32 * 10.,
            width: 10.,
            height: 10.
        });
    }

    let mut window = RenderWindow::new(
        (WIDTH, HEIGHT),
        "Game i guess",
        Style::CLOSE,
        &Default::default(),
    );
    window.set_vertical_sync_enabled(true);

    // let mut shape = CustomShape::new(Box::new(TriangleShape));
    // shape.set_fill_color(Color::RED);
    // shape.set_outline_color(Color::GREEN);
    // shape.set_outline_thickness(3.);


    let (exit_tx, exit_rx) = mpsc::channel();
    let game_loop = thread::Builder::new().name("game thread".to_string()).spawn(move || {
        loop{
            // println!("{:#?}", state);
            match exit_rx.try_recv() {
                Ok(_) => {
                    break;
                },
                Err(_) => {}
            }

        }
    });

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

        window.clear(Color::BLACK);

        for block in &state.blocks {
            let mut rect = RectangleShape::new();
            rect.set_fill_color(Color::YELLOW);
            rect.set_position((block.x, block.y));
            rect.set_size((block.width, block.height));
            window.draw(&rect);
        }

        let mut player_rect = RectangleShape::new();
        player_rect.set_fill_color(Color::WHITE);
        player_rect.set_position((player.x, player.y));
        player_rect.set_size((PLAYER_WIDTH as f32, PLAYER_HEIGHT as f32));
        window.draw(&player_rect);

        // window.draw(&shape);
        window.display();
    }

    exit_tx.send(());
    game_loop.unwrap().join();
}
