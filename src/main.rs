use sfml::{graphics::*, window::*};

fn main() {
    let mut window = RenderWindow::new(
        (800, 600),
        "Test",
        Style::CLOSE,
        &Default::default(),
    );

    let font = Font::from_file("src/resources/sansation.ttf").unwrap();
    let text = Text::new(&String::from("Hello, World!"), &font, 24);

    'main: loop {
        while let Some(ev) = window.poll_event() {
            match ev {
                Event::Closed => break 'main,
                _ => {}
            }
        }

        window.clear(Color::BLACK);
        window.draw(&text);
        window.display();
    }
}
