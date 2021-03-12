// TODOs:
// - support resizing
// - fix bug where later keys are prioritized
//

use sfml::{
    graphics::{Color, RenderTarget, RenderWindow, Shape},
    window::{Event, Key, Style},
};
use sfml::graphics::{RectangleShape, Transformable, Text, Font};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::cmp::max;
use sfml::audio::{SoundBuffer, Sound};
use sfml::system::Time;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum KeyDirection{
    Up,
    Down,
    Left,
    Right
}

#[derive(Debug, Copy, Clone)]
struct MapKey{
    direction: KeyDirection,
    time: f64,
    hit: bool
}

#[derive(Debug, Copy, Clone)]
struct KeysPressed{
    up: bool,
    down: bool,
    left: bool,
    right: bool
}

#[derive(Debug)]
struct State{
    keys: KeysPressed,
    map: Vec<MapKey>,
    game_time: f64,
    score: f64,
    quit: bool,
    paused: bool
}

const WIDTH:u32 = 1920;
const HEIGHT:u32 = 1080;
const SPEED:f32 = 20f32;
const SCREEN_OFFSET:f32 = 100f32;
const MAX_KEY_ERROR:f64 = 250f64;
const WARMUP_SECS:i32 = 1;
const RECORD:bool = true;
const SKIP:u64 = 145;


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
    // let height_and_saturation_map: Vec<(f32, f32)> = vec![(45f32, 0.25), (20f32, 0.5), (4f32, 1.)];
    let height_and_saturation_map: Vec<(f32, f32)> = vec![(30f32, 0.25), (20f32, 0.5), (4f32, 1.)];
    let map_key_disp_order: Vec<KeyDirection> = vec![KeyDirection::Left, KeyDirection::Down, KeyDirection::Up, KeyDirection::Right];

    let physics = move|state: &mut State| -> () {
        let game_time = state.game_time as f64;

        let check_direction = move |dir: KeyDirection, state: &mut State| {
            if RECORD {
                println!("{} {}", state.map.iter().filter(|x| x.hit).count(), state.game_time);
            }
            if !state.map.is_empty(){
                let mut hit_key_opt = None;
                let mut lowest_abs = f64::INFINITY;
                for map_key in &mut state.map {
                    let abs = (map_key.time - game_time).abs();
                    // println!("{}: {}", abs, map_key.time);
                    if !map_key.hit && map_key.direction == dir && abs < lowest_abs && abs < MAX_KEY_ERROR {
                        hit_key_opt = Some(map_key);
                        lowest_abs = abs;
                    }
                }

                if let Some(hit_key) = hit_key_opt {
                    // &state.map.sort_unstable_by(|a, b| (game_time - a.time).abs().partial_cmp(&(game_time - b.time).abs()).unwrap());
                    // let first_key = state.map.first_mut().unwrap();
                    hit_key.hit = true;
                    let error = hit_key.time - game_time;

                    state.score += error;
                }else{
                    // you didn't hit anything
                    state.score += MAX_KEY_ERROR;
                }
            }
        };

        // for map_key in &mut state.map{
        //     let abs = (map_key.time - game_time).abs();
        //     if abs < 10. {
        //         println!("FLASH!");
        //     }
        // }

        if state.keys.up {
            check_direction(KeyDirection::Up, state);
            state.keys.up = false;
        }

        if state.keys.down {
            check_direction(KeyDirection::Down, state);
            state.keys.down = false;
        }

        if state.keys.left {
            check_direction(KeyDirection::Left, state);
            state.keys.left = false;
        }

        if state.keys.right {
            check_direction(KeyDirection::Right, state);
            state.keys.right = false;
        }

        for map_key in &mut state.map {
            let mut error = map_key.time - state.game_time;
            if !map_key.hit && -error > MAX_KEY_ERROR {
                map_key.hit = true;

                error = MAX_KEY_ERROR;

                state.score += error;


                // println!("added MAX error: {}", error * error);
            }
        }
    };

    let mut window = RenderWindow::new(
        (WIDTH, HEIGHT),
        "Rhythm Game",
        Style::DEFAULT,
        &Default::default(),
    );

    // no v-sync to help with latency
    window.set_vertical_sync_enabled(false);
    window.set_mouse_cursor_visible(false);
    window.set_key_repeat_enabled(false);


    // include_bytes! builds this font into our executable, meaning that we do not need to bring
    // a resources/ folder around. very handy!
    // we unwrap because it should crash if the font isn't there (a bug)
    let font = Font::from_memory(include_bytes!("resources/sansation.ttf")).unwrap();

    // supports ogg, wav, flac, aiff, au, raw, paf, svx, nist, voc, ircam, w64, mat4, mat5 pvf, htk, sds, avr, sd2, caf, wve, mpc2k, rf64, see https://docs.rs/sfml/0.14.0/sfml/audio/struct.SoundBuffer.html
    let song_buffer = SoundBuffer::from_memory(include_bytes!("resources/song.ogg")).unwrap();
    let mut song = Sound::with_buffer(&song_buffer);

    let mut state = State{
        keys: KeysPressed {
            up: false,
            down: false,
            left: false,
            right: false
        },
        map: vec![
            // MapKey{ direction: KeyDirection::Up, time: 1000f64, hit: false },
            // MapKey{ direction: KeyDirection::Down, time: 1250f64, hit: false },
            // MapKey{ direction: KeyDirection::Left, time: 1500f64, hit: false },
            // MapKey{ direction: KeyDirection::Right, time: 1750f64, hit: false },
            // // MapKey{ direction: KeyDirection::Up, time: 2000f64, hit: false },
            // // MapKey{ direction: KeyDirection::Up, time: 3000f64, hit: false },
            // // MapKey{ direction: KeyDirection::Up, time: 4000f64, hit: false },
            // // MapKey{ direction: KeyDirection::Up, time: 5000f64, hit: false },
            // // MapKey{ direction: KeyDirection::Up, time: 6000f64, hit: false },
            // // MapKey{ direction: KeyDirection::Up, time: 7000f64, hit: false },
            // // MapKey{ direction: KeyDirection::Up, time: 8000f64, hit: false },
        ],
        score: 0f64,
        game_time: 0f64,
        quit: false,
        paused: true
    };

    let bpm = 60.;
    let bps = bpm / 60.;
    let spb = 1. / bps;
    println!("spb {}", spb * 1000.);
    for n in 1..80 {
        // BPM track
        state.map.push(
            MapKey{ direction: KeyDirection::Up, time: n as f64 * 1019.5 + 163., hit: false });

        // state.map.push(
        //     MapKey{ direction: KeyDirection::Up, time: n as f64 * (spb) * 1000., hit: false });
    }

    let left_track = vec![
        1532.6459999999997, 1737.5600000000004, 1992.0410000000002, 3540.915, 3778.6440000000002, 4049.8900000000003, 5599.089, 5836.790999999999, 6057.2080000000005, 7639.093999999999, 7875.986000000001, 8114.6900000000005, 9645.38, 9832.105, 10102.506, 11668.964, 11890.639, 12128.697, 13726.137999999999, 13913.324, 14202.106, 15732.668000000001, 15970.871,
        18049.605, 18321.779, 18542.606, 18831.468, 19289.196, 21601.739, 21840.146, 22094.872, 22349.784, 22655.996, 23130.606, 23369.512, 23760.783, 26209.604, 26396.233, 26719.18, 26974.122, 27450.223, 28095.434, 28435.398, 28809.754, 29591.244, 30254.493000000002, 34369.993, 34606.95, 34879.75, 35118.064, 35593.011, 37974.491, 38195.198, 38467.906, 38654.482, 38993.654, 39471.293, 39726.147, 40150.836, 42258.658, 42495.892, 42716.894, 42990.07, 43296.171, 43754.098, 44383.172, 44824.469, 45079.792, 45827.918, 46524.4,
        48378.883, 48618.516, 48838.986000000004, 50403.675, 50640.865, 50880.038, 52445.366, 52649.419, 52870.223, 54486.152, 54724.265, 54946.685, 56526.466, 56765.147, 57019.47, 58550.191, 58756.740000000005, 59026.554000000004, 60607.032999999996, 60844.853, 61066.505000000005, 62629.801, 62850.710999999996, 63088.613,
        66174.14600000001, 66391.89199999999, 66666.477, 66868.276, 67140.116, 67411.845, 67648.69, 68124.53, 70302.4, 70522.848, 70675.529, 70829.009, 71252.679, 71541.894, 72006.087, 72272.831, 72662.761, 74109.544, 74348., 74585.073, 74823.867, 75078.749, 75351.48300000001, 75605.536, 76080.04699999999, 76387.205, 76914.757, 77338.338, 77697.74799999999, 78190.765, 78411.834, 79158.958, 79362.568, 80163.239, 80417.27799999999,
        81604.001, 82214.081, 83526.756, 84136.319, 85528.97, 86297.37, 87572.245, 88282.575, 89523.928, 90240.363, 91596.73300000001, 92328.856, 93640.054, 94371.545, 95630.3, 96358.909, 97582.641, 98262.881, 99638.067, 100184.995, 101679.681, 102377.602, 103682.582, 104348.40299999999, 105726.26, 106403.893, 107751.50200000001, 108448.76, // 109464.833, 111658.527,
                          // 109666.916, 110482.976, 111553.51, 112271.10500000001, 112575.77900000001,
        115136.14600000001, 115369.976, 115627.106, 117055.00200000001, 117292.079, 117529.686, 119046.565, 119234.23300000001, 119502.694, 121031.738, 121220.468, 121478.048,
        122196.432, 122472.084, 122724.285, 123504.909, 123726.855, 123984.42, 124391.98, 126668.818, 126855.025, 127008.26699999999, 127368.407, 127672.144, 128116.592, 128317.047, 128809.163, 130136.536, 130374.594, 130628.441, 130851.391, 131071.959, 131292.045, 131563.986, 132022.75400000002, 132294.04, 133331.96600000001, 133571.951, 134268.191, 134952.241, 138024.484, 138248.886, 138482.804, 138720.345, 139775.21600000001, 140016.539, 140319.702, 140607.997, 142432.003, 142636.133, 142806.493, 143214.37099999998, 143451.219, 143892.307, 144180.1, 144607.408, 145899.832, 146138.847, 146428.404, 146664.192, 147124.364, 147561.59100000001, 148058.165, 148571.963, 149012.298, 149230.73, 149384.123, 149860.017, 150116.64299999998, 150796.343, 151088.57, 151782.02, 151976.139,
        // 153355.405,
        154027.602, 155333.763, 156091.994, 157367.41999999998, 158116.619, 159311.832, 160006.718, 161311.742, 162027.66, 163249.392, 163965.612, 165327.48, 166075.456, 167282.533, 168012.62900000002, 169271.217, 170001.721, 171226.40899999999, 172011.187, 173199.329, 173947.468, 175224.851, 175955.657, 177247.502, 177927.658, 179271.653, 180043.761,
        185467.501, 186028.846, 187321.526, 187937.162, 190367.044, 190585.70500000002, 190823.834, 191050.724, 191342.634, 191846.891, 192046.816, 192794.001, 193052.139, 193306.317, 194053.24599999998, 195077.047, 195327.612, 196076.005, 196602.462, 196827.131, 197098.898, 198290.521, 199223.022,
    201217.169, 201893.99, 202951.411, 203203.398, 203885.96600000001, 205244.961, 206177.823, 207149.768, 208153.683, 209206.191, 209900.377, 211161.528, 211859.26799999998, 213186.37, 214156.038, 215120.494, 216078.299, 217149.239, 217846.245, 218882.48, 219120.222, 219865.962, 221090.37900000002, 222062.536, 223029.079, 223980.228, 225017.946, 225785.711, 227009.774, 227787.989, 228982.53100000002, 229918.322, 230920.141, 231905.877, 232976.678, 233688.014, 234708.198, 234950.72, 235609.325, 236868.608, 237854.363, 238842.687, 239861.023, 240833.373, 241548.473, 242753.79700000002, 243483.858, 244757.46899999998, 245761.36599999998

    ];

    for time in left_track{
        state.map.push(MapKey{ direction: KeyDirection::Left, time, hit: false});
    }

    for n in 0..56{
        state.map.push(MapKey{ direction: KeyDirection::Up, time: n as f64 * 501.074 + 81630.4, hit: false });
    }

    for n in 0..137{
        state.map.push(MapKey{ direction: KeyDirection::Up, time: n as f64 * 495.715 + 113704., hit: false });
        if n % 4 == 3 {
            state.map.push(MapKey { direction: KeyDirection::Right, time: n as f64 * 495.715 + 113704., hit: false });
        }
    }

    for n in 141..267{
        state.map.push(MapKey{ direction: KeyDirection::Up, time: n as f64 * 495.667 + 115457., hit: false });
        if n % 4 == 3 {
            state.map.push(MapKey { direction: KeyDirection::Right, time: n as f64 * 495.667 + 115457., hit: false });
        }
    }

    let three_track = vec![81154.964, 64928.648, 65149.167, 109666.916, 110482.976, 111553.51, 112271.10500000001, 112575.77900000001, 153355.405,
                           181271.432, 182034.89, 183331.206, 183802.421, 184061.13];

    for time in three_track{
        state.map.push(MapKey{ direction: KeyDirection::Left, time, hit: false });
        state.map.push(MapKey{ direction: KeyDirection::Down, time, hit: false });
        state.map.push(MapKey{ direction: KeyDirection::Right, time, hit: false });
    }


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

                    if state.paused{ continue; }

                    physics(state);
                }
                // TODO: make this calculated from above time
                thread::sleep(Duration::from_millis(1));
            }
        });
    }

    let start = Instant::now();
    let mut last_frame = Instant::now();


    song.play();
    song.pause();
    song.set_playing_offset(Time::seconds(0.));

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
                    } => state.keys.up = true,
                    Event::KeyReleased {
                        code: Key::Up, ..
                    } => state.keys.up = false,

                    Event::KeyPressed {
                        code: Key::Down, ..
                    } => state.keys.down = true,
                    Event::KeyReleased {
                        code: Key::Down, ..
                    } => state.keys.down = false,

                    Event::KeyPressed {
                        code: Key::Left, ..
                    } => state.keys.left = true,
                    Event::KeyReleased {
                        code: Key::Left, ..
                    } => state.keys.left = false,

                    Event::KeyPressed {
                        code: Key::Right, ..
                    } => state.keys.right = true,
                    Event::KeyReleased {
                        code: Key::Right, ..
                    } => state.keys.right = false,
                    _ => {}
                }
            }

            window.clear(Color::BLACK);

            let duration = start.elapsed();

            if state.paused && duration.as_secs() >= WARMUP_SECS as u64 {
                state.paused = false;
                song.set_playing_offset(Time::seconds(SKIP as f32));
                song.play();
            }

            if state.paused {
                let mut text = Text::new(&format!("{}", 3 - duration.as_secs()), &font, 100);
                text.set_fill_color(Color::WHITE);
                text.set_position((0., 0.));
                window.draw(&text);
            }else{
                state.game_time = (duration.as_micros()) as f64 / 1000f64 - (WARMUP_SECS * 1000) as f64 + (SKIP * 1000) as f64;

                let mut hit_line = RectangleShape::new();
                hit_line.set_position((0., SCREEN_OFFSET));
                hit_line.set_size((WIDTH as f32, 1.));
                hit_line.set_fill_color(Color::WHITE);
                window.draw(&hit_line);

                // draw map keys
                for map_key in &state.map {
                    let mut rect = RectangleShape::new();
                    let screen_time = (map_key.time - state.game_time) as f32;

                    // screen_pos is the exact pixel that you want to "hit"
                    let screen_pos = 0.00001f32 * SPEED * (HEIGHT as f32) * screen_time + SCREEN_OFFSET;


                    let key_idx = map_key_disp_order.iter().position(|&r| r == map_key.direction).unwrap();
                    let x = 100. + key_idx as f32 * 150.;

                    for key_range in &height_and_saturation_map {
                        // should be an even integer
                        let rect_height = key_range.0;
                        let mut saturation = key_range.1;
                        let mut value = 1.0;

                        if map_key.hit {
                            // r = 255;
                            // g = 255;
                            // b = 255;
                            saturation /= 2.;
                            value /= 10.;
                        }

                        let (r, g, b) = hsv_to_rgb(((map_key.time as usize / 75) % 360) as u32, saturation, value);


                        rect.set_fill_color(Color::rgb(r as u8, g as u8, b as u8));
                        rect.set_position((x, (screen_pos as f32) - rect_height / 2.));
                        rect.set_size((100f32, rect_height));
                        window.draw(&rect);
                    }
                }

                for direction in &map_key_disp_order{
                    let key_idx = map_key_disp_order.iter().position(|&r| r == *direction).unwrap();
                    let x = 100. + key_idx as f32 * 150.;

                    let mut text = Text::new(&format!("{:?}", direction), &font, 15);
                    text.set_fill_color(Color::WHITE);
                    text.set_position((x, 50.));
                    window.draw(&text);
                }
            }

            // // draw height info
            // let x = state.player.x / BLOCK_SIZE as f32;
            // let y = state.player.y as f64 / BLOCK_SIZE as f64;
            // let mut text = Text::new(&format!("X:{}\nY: {}\nDensity: {:.3}", x, y, density(y)), &font, 16);
            // text.set_fill_color(Color::WHITE);
            // text.set_position((0., 66.));
            // window.draw(&text);
            //
            // let duration = start.elapsed();
            //
            let elapsed = last_frame.elapsed().as_millis();

            let mut text = Text::new(&format!("{} FPS\nScore: {}", 1000u128 / max(elapsed, 1u128), state.score / 1000.), &font, 15);
            text.set_fill_color(Color::WHITE);
            text.set_position((0., 0.));
            window.draw(&text);


            // if duration.as_secs() >= GAME_SECONDS

            last_frame = Instant::now();
        }

        window.display();
    }

    (*state_holder.lock().unwrap()).quit = true;
    physics_thread.join().unwrap();
}
