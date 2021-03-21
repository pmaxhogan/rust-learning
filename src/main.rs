// TODOs:
// - re-check that both scroll directions work
// - pause functionality
// - mp3 support
// - test it with other real songs
// - user-configured delay and constants
// - inspect performance with crox
// - add measure lines
// - auto-updater

use std::cmp::max;
use std::collections::HashMap;
use std::fs;
use std::str::Chars;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use sfml::{
    graphics::{Color, RenderTarget, RenderWindow, Shape},
    window::{Event, Key, Style},
    audio::{Sound, SoundBuffer},
    graphics::{Font, RectangleShape, Text, Transformable, View},
    system::{Time, Vector2}
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum KeyDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone)]
struct Message {
    val: MessageVal,
    direction: KeyDirection,
    time: f64,
}

#[derive(Debug, Copy, Clone)]
enum MessageVal {
    Perfect,
    Fantastic,
    Excellent,
    Great,
    Good,
    Mediocre,
    Awful,
    Miss,
}

#[derive(Debug, Copy, Clone)]
struct MapKey {
    direction: KeyDirection,
    time: f64,
    hit: bool,
    hit_start: bool,
    time_end: f64// set to -1 if it's normal, otherwise the time to hold until
}

#[derive(Debug, Copy, Clone)]
struct KeysPressed {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

#[derive(Debug)]
struct State {
    keys: KeysPressed,
    keys_released: KeysPressed,
    map: Vec<MapKey>,
    messages: Vec<Message>,
    game_time: f64,
    score: f64,
    quit: bool,
    paused: bool,
    song_playing: bool,
    errors: Vec<f64>,
    song_meta: SmMetadata
}

#[derive(Debug, Clone)]
struct SmMetadata{
    delay: f32,
    title: String,
    subtitle: String,
    artist: String
}

#[derive(Debug)]
struct SmFileResult {
    difficulties: Vec<String>,
    difficulties_map: HashMap<String, Vec<MapKey>>,
    meta: SmMetadata,
}

/// GRAPHICS SETTINGS
// default height
const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

// why would you do this
const ENABLE_KEY_REPEAT:bool = false;

const WINDOW_STYLE:Style = Style::RESIZE;

/// VISUAL SETTINGS
// "visual BPM", how fast the notes scroll
// does not impact actual song speed
const SCROLL_SPEED: f32 = 50f32;

// font size of messages
const MESSAGE_SIZE: u32 = 20;

// set to false for the notes to scroll upwards
const SCROLL_DOWNWARDS: bool = true;

// how long a message stays on the screen
const MESSAGE_DURATION: f64 = 250.;

// how many pixels a note needs to be offscreen to avoid it being rendered
const NO_RENDER_BUFFER:u32 = 50;

const NOTE_COLUMN_WIDTH:f32 = 100.;

// height of the line that you hit
const LINE_HEIGHT: f32 = 100f32;

/// ERROR HISTOGRAM SETTINGS
const INCLUDE_MISSES_IN_HISTOGRAM: bool = false;

const HISTOGRAM_WIDTH:f32 = 500.;

const HISTOGRAM_HEIGHT: f32 = 50.;

const HISTOGRAM_BACKGROUND: bool = false;

// should be an odd number
const HISTOGRAM_BINS:usize = 41;

/// TIMING SETTINGS
// value in ms
// compensate for sound not lining up with when a key hits a line, around 100ms for bluetooth
const AUDIO_DELAY:f32 = 200f32;

// skip this many seconds into the song
// moves both note track and audio
const SECONDS_TO_SKIP: i64 = -2;

// how many seconds of warmup timer you get
const WARMUP_SECS: i32 = 0;

// how many ms to subtract from the game clock
// should be set to the latency of your system
// if you tend to hit notes late but the audio seems right, set this to a positive number
const KEY_LATENCY_OFFSET: f64 = 0.;

// how many ms you can be off for a key to register as a hit
// if you are more than this late then it will register as a miss
const KEY_ERROR_RANGE: f64 = 250f64;

/// DEBUG SETTINGS
// echos keys to the console when you press them
const RECORD: bool = false;

/// OTHER SETTINGS
// disables the penalty for hitting a key when there is no note within [-KEY_ERROR_RANGE, KEY_ERROR_RANGE]
const DISABLE_MISS_PENALTY: bool = true;


/**
* h is [0, 360]
* s and v are [0, 1]
*/
fn hsv_to_rgb(h: u32, s: f32, v: f32) -> (u32, u32, u32) {
    if h > 360 || s < 0f32 || s > 1f32 || v < 0f32 || v > 1f32 {
        panic!("Parameters to hsv_to_rgb() should be in the ranges [0, 360], [0, 1], and [0, 1] respectively")
    }

    let c = v * s;
    let x = c * (1f32 - ((h as f32 / 60f32) % 2f32 - 1f32).abs());
    let m = v - c;
    let (r_2, g_2, b_2) = match h {
        0..=60 => (c, x, 0f32),
        61..=120 => (x, c, 0f32),
        121..=180 => (0f32, c, x),
        181..=240 => (0f32, x, c),
        241..=300 => (x, 0f32, c),
        301..=360 => (c, 0f32, x),
        _ => {
            panic!("Not a valid H value!");
        }
    };

    let (r, g, b) = ((r_2 + m) * 255f32, (g_2 + m) * 255f32, (b_2 + m) * 255f32);

    return (r.round() as u32, g.round() as u32, b.round() as u32);
}

fn sm_to_keys(str: String) -> SmFileResult {
    let map_key_disp_order: Vec<KeyDirection> = vec![KeyDirection::Left, KeyDirection::Down, KeyDirection::Up, KeyDirection::Right];
    let mut keys: Vec<MapKey> = vec![];
    let mut difficulties_map = HashMap::new();
    let mut difficulties:Vec<String> = Vec::new();

    // maps measure to BPM
    let mut bpms_map = HashMap::new();
    let mut bpms_str = String::new();
    let mut bpms_on = false;

    // could be used in the future for measure lines
    let mut _measure = 0;

    let mut current_beat = 0;
    let mut current_bpm = 0.;

    let mut measure_string = String::new();
    let mut current_time: f64 = 0.;

    let mut holds_map = HashMap::new();

    let mut is_difficulty_header = false;
    let mut difficulty_values:Vec<&str> = Vec::with_capacity(7);
    let mut difficulty = "";

    let mut meta_title = String::new();
    let mut meta_sub = String::new();
    let mut meta_artist = String::new();

    let mut delay = 0f32;

    let mut is_note_section = false;
    for line in str.split("\n") {
        let mut line = line;
        if line.ends_with("\r"){
            line = &line[0..(line.len() - 1)];
        }

        if line.trim().is_empty() { continue; }

        let num_line = line.starts_with('0') || line.starts_with('1') || line.starts_with('2') || line.starts_with('3') || line.starts_with('M');

        if num_line && is_difficulty_header{
            is_difficulty_header = false;

            if difficulty_values.len() != 6 {
                panic!(format!("Confusing difficulty header! {:#?}", difficulty_values));
            }

            let len = difficulty_values[3].len();
            difficulty = difficulty_values[3][0..(len - 1)].trim();
            difficulties.push(difficulty.to_string());
        }

        if line.starts_with("//--"){
            is_difficulty_header = true;
        }

        if line.starts_with("#") {
            let split: Vec<&str> = line.split(":").collect();
            let key = split[0];
            let value = split[1];

            // dereference the value, remove the semi, re-reference it
            let value_fixed = if value.len() > 1 {&((*(value))[0..((value.len())-1)]) } else { "" };

            if is_difficulty_header && key != "#NOTES"{
                panic!("Unknown key found in notes ".to_owned() + line);
            }

            match key{
                "#BPMS" => {
                    bpms_str += value;
                    bpms_on = true;
                }
                "#OFFSET" => {
                    // parse and convert from s to ms
                    delay = -(value_fixed.parse::<f32>().unwrap() * 1000.);
                },
                "#TITLE" => {
                    meta_title = value_fixed.to_string();
                },
                "#SUBTITLE" => {
                    meta_sub = value_fixed.to_string();
                },
                "#ARTIST" => {
                    meta_artist = value_fixed.to_string();
                },
                _ => {

                }
            }

            if key != "#BPMS" && bpms_on {
                bpms_on = false;

                bpms_str = bpms_str.trim_end().parse().unwrap();

                let bpms_split: Vec<&str> = bpms_str[0..bpms_str.len() - 1].split(",").collect();
                for bpm_bit in bpms_split {
                    let measure_and_bpm: Vec<&str> = bpm_bit.split('=').collect();
                    let measure = measure_and_bpm[0];
                    let bpm = measure_and_bpm[1];

                    bpms_map.insert(measure.parse::<f64>().unwrap() as usize, bpm.parse::<f64>().unwrap());
                }
            }
        } else if bpms_on && !line.starts_with("/") && !line.starts_with("#") {
            bpms_str += line;
        } else if is_difficulty_header{
            difficulty_values.push(line);
        }

        if !is_note_section && num_line {
            is_note_section = true;
        }

        if line.starts_with(",") || line.starts_with(";") {

            let measure_vec: Vec<&str> = measure_string.split("\n").collect();
            let notes_in_measure = measure_vec.len();

            let mut note_in_measure: isize = -1;

            for measure_line in measure_vec {
                if ((note_in_measure as f64) / (notes_in_measure as f64 * 4.)) % 1. == 0. {
                    current_beat += 1;
                }

                match bpms_map.get(&(current_beat as usize)) {
                    None => {}
                    Some(bpm) => {
                        current_bpm = *bpm;
                    }
                }

                let current_beat_duration = 60f64 / current_bpm;

                let split: Chars = measure_line.chars();

                let mut i = 0;

                // for each column...
                for num in split {
                    match num {
                        '0' => {}
                        '1' => {
                            keys.push(MapKey {
                                direction: map_key_disp_order[i],
                                time: current_time * 1000.,
                                hit: false,
                                hit_start: false,
                                time_end: -1.
                            })
                        }
                        '2' => {
                            holds_map.insert(i, current_time);
                        }
                        '3' => {
                            keys.push(MapKey {
                                direction: map_key_disp_order[i],
                                time: *holds_map.get(&i).unwrap() * 1000.,
                                hit: false,
                                hit_start: false,
                                time_end: current_time * 1000.
                            });
                            holds_map.remove(&i);
                        }
                        'M' => {}
                        _ => { panic!("Unknown number ".to_owned() + measure_line) }
                    }

                    i += 1;
                }

                current_time += current_beat_duration / (notes_in_measure as f64 / 4.);

                note_in_measure += 1;
            }

            measure_string = String::new();

            _measure += 1;
        }


        if line == ";"{
            difficulties_map.insert(difficulty.to_string(), keys);
            keys = vec![];

            _measure = 0;
            current_beat = 0;
            current_bpm = 0.;
            current_time = 0.;
            measure_string = String::new();

            if !holds_map.is_empty(){
                panic!(format!("Still had holds left over! {:#?}", holds_map));
            }

            holds_map.clear();

            difficulty_values.clear();
        }

        if num_line {
            measure_string += &*(line.to_owned() + "\n");
        }
    }


    if !holds_map.is_empty(){
        panic!(format!("Still had holds left over! {:#?}", holds_map));
    }
    SmFileResult{
        difficulties,
        difficulties_map,
        meta: SmMetadata{
            delay,
            title: meta_title,
            subtitle: meta_sub,
            artist: meta_artist
        }
    }
}

#[allow(dead_code)]
fn convert_to_csv(keys: &Vec<MapKey>) {
    let mut keys = keys.clone();
    let mut data = String::from("Time\tDirection\n");
    keys.sort_by(|a, b| (&a.time).partial_cmp(&b.time).unwrap());
    for key in keys {
        let key_direction = match key.direction {
            KeyDirection::Up => "U",
            KeyDirection::Down => "D",
            KeyDirection::Left => "L",
            KeyDirection::Right => "R"
        };

        data += &*format!("{}\t{}\n", key.time, key_direction);
    }
    fs::write("converted-maps/map.csv", data).unwrap();
}

fn csv_to_keys(str: String) -> Vec<MapKey> {
    let mut res: Vec<MapKey> = vec![];
    if str.starts_with("Time\tDirection\n") {
        for line in str.split("\n").skip(1) {
            if line == "" { continue; }

            let split: Vec<&str> = line.split("\t").collect();
            if split.len() != 2 {
                panic!("Invalid file");
            }

            let time = split[0];
            let direction = split[1];

            res.push(MapKey {
                direction: match direction {
                    "U" => KeyDirection::Up,
                    "D" => KeyDirection::Down,
                    "L" => KeyDirection::Left,
                    "R" => KeyDirection::Right,
                    _ => { panic!("Invalid key ".to_owned() + direction); }
                },
                time: time.parse().unwrap(),
                hit: false,
                hit_start: false,
                time_end: -1.
            });
        }
    } else {
        panic!("Invalid file");
    }

    res
}

fn time_to_screen_height(time:f32, height_of_window: u32, include_key_delay: bool) -> f32 {
    let height_of_window = height_of_window as f32;

    let val = 0.00001f32 * SCROLL_SPEED * height_of_window * (time + if include_key_delay { AUDIO_DELAY } else {0.}) + LINE_HEIGHT;

    if SCROLL_DOWNWARDS {
        return height_of_window - val
    }
    val
}

// simple function for fading in and out something
// https://www.desmos.com/calculator/rzbucjsfyh
fn fade_in_out(time: f64) -> f64{
    if time < 0. || time > 1. {
        panic!("Invalid time ".to_owned() + &time.to_string());
    }

    (1.5 - 2.7 * (time - 0.5).abs()).min(1.)
}

struct MapLoadResult{
    difficulties: Vec<String>,
    difficulties_map: HashMap<String, Vec<MapKey>>,
    song: String,
    meta: SmMetadata
}

fn load_map_folder(folder: &str) -> Result<MapLoadResult, String> {
    let mut song = None;
    let mut keys = None;
    let read = fs::read_dir(folder);

    match read {
        Ok(entries) => {
            for entry in entries {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    return Err("Folder should not contain directories".to_string());
                } else {
                    match path.extension() {
                        Some(ext) => {
                            match ext.to_str().unwrap() {
                                "ogg" => {
                                    song = Some(path.to_str().unwrap().to_string());
                                },
                                "sm" => {
                                    if keys.is_some() {
                                        return Err("Cannot have two map files in directory".parse().unwrap());
                                    }

                                    if let Ok(str) = fs::read_to_string(path) {
                                        keys = Some(sm_to_keys(str));
                                    }
                                },
                                "csv" => {
                                    if keys.is_some() {
                                        return Err("Cannot have two map files in directory".parse().unwrap());
                                    }

                                    if let Ok(str) = fs::read_to_string(path) {
                                        let mut difficulties_map = HashMap::new();
                                        difficulties_map.insert("Normal".to_string(), csv_to_keys(str));

                                        keys = Some(SmFileResult{
                                            difficulties: vec!["Normal".to_string()],
                                            difficulties_map,
                                            meta: SmMetadata{
                                                delay: 0.0,
                                                title: "Unknown".to_string(),
                                                subtitle: "".to_string(),
                                                artist: "Unknown".to_string()
                                            }
                                        });
                                    }
                                },
                                // TODO: implement this
                                // "png" => { ... display the png somehow ... }
                                _ => {}
                            }
                        }
                        None => {}
                    }
                }
            }

            if keys.is_some() && song.is_some() {
                let unwrapped = keys.unwrap();

                Ok(MapLoadResult {
                    difficulties: unwrapped.difficulties,
                    difficulties_map: unwrapped.difficulties_map,
                    song: song.unwrap().parse().unwrap(),
                    meta: unwrapped.meta
                })
            } else {
                Err("Did not find song and map in folder!".parse().unwrap())
            }
        },
        Err(e) => {
            Err(e.to_string())
        }
    }
}

fn closest_to_zero (num1: f64, num2: f64) -> f64 {
    if num1.abs() > num2.abs(){
        return num2;
    }
    return num1;
}

fn main() {
    let mut width = WIDTH;
    let mut height = HEIGHT;

    let height_and_saturation_map: Vec<(f32, f32)> = vec![(30f32, 0.25), (20f32, 0.5), (4f32, 1.)];
    let map_key_disp_order: Vec<KeyDirection> = vec![KeyDirection::Left, KeyDirection::Down, KeyDirection::Up, KeyDirection::Right];
    let messages_order = [MessageVal::Perfect, MessageVal::Fantastic, MessageVal::Excellent, MessageVal::Great, MessageVal::Good, MessageVal::Mediocre, MessageVal::Awful];

    let delay_to_message_val = move |delay: f64| -> MessageVal {
        let delay_ratio = (delay.abs() / KEY_ERROR_RANGE * (messages_order.len() - 1) as f64).ceil();

        messages_order[delay_ratio as usize]
    };

    let physics = move |state: &mut State| -> () {
        let game_time = state.game_time as f64 - KEY_LATENCY_OFFSET - AUDIO_DELAY as f64;

        let check_direction = move |dir: KeyDirection, state: &mut State, key_was_released: bool| {
            if RECORD {
                println!("{} {}", state.map.iter().filter(|x| x.hit).count(), game_time);
            }

            if !state.map.is_empty() {
                let mut hit_key_opt = None;
                let mut lowest_abs = f64::INFINITY;
                let error;
                for map_key in &mut state.map {
                    let abs = if key_was_released {
                        if game_time < map_key.time_end && game_time > map_key.time{
                            0.
                        }else {
                            (map_key.time_end - game_time).abs()
                        }
                    } else {
                        (map_key.time - game_time).abs()
                    };

                    let start_hit_condition = if key_was_released { map_key.hit_start } else { !map_key.hit_start };
                    if start_hit_condition && !map_key.hit && map_key.direction == dir && abs < lowest_abs && abs < KEY_ERROR_RANGE {
                        hit_key_opt = Some(map_key);
                        lowest_abs = abs;
                    }
                }

                // hit a key
                if let Some(hit_key) = hit_key_opt {
                    let is_hold = hit_key.time_end != -1.;

                    if is_hold && !hit_key.hit_start {
                        hit_key.hit_start = true;
                    }else{
                        hit_key.hit = true;
                    }

                    let error_unclamped = if key_was_released { hit_key.time_end - game_time } else {hit_key.time - game_time};

                    error = closest_to_zero(error_unclamped, KEY_ERROR_RANGE * error_unclamped.signum());

                    state.score += error.abs();

                    state.errors.push(error);

                    state.messages.push(Message { val: delay_to_message_val(error.abs()), time: game_time, direction: dir });
                } else {
                    // you didn't hit anything
                    if !DISABLE_MISS_PENALTY {
                        error = KEY_ERROR_RANGE;
                        state.score += error.abs();

                        state.messages.push(Message { val: MessageVal::Miss, time: game_time, direction: dir });
                    }
                }
            }
        };

        if state.keys.up {
            check_direction(KeyDirection::Up, state, false);
            state.keys.up = false;
        }

        if state.keys.down {
            check_direction(KeyDirection::Down, state, false);
            state.keys.down = false;
        }

        if state.keys.left {
            check_direction(KeyDirection::Left, state, false);
            state.keys.left = false;
        }

        if state.keys.right {
            check_direction(KeyDirection::Right, state, false);
            state.keys.right = false;
        }

        if state.keys_released.up {
            check_direction(KeyDirection::Up, state, true);
            state.keys_released.up = false;
        }

        if state.keys_released.down {
            check_direction(KeyDirection::Down, state, true);
            state.keys_released.down = false;
        }

        if state.keys_released.left {
            check_direction(KeyDirection::Left, state, true);
            state.keys_released.left = false;
        }

        if state.keys_released.right {
            check_direction(KeyDirection::Right, state, true);
            state.keys_released.right = false;
        }

        // entirely missed a key
        for map_key in &mut state.map {
            let mut error = if map_key.hit_start { map_key.time_end - game_time } else { map_key.time - game_time };
            if !map_key.hit && -error > KEY_ERROR_RANGE {
                let is_hold = map_key.time_end != -1.;

                if is_hold && !map_key.hit_start {
                    map_key.hit_start = true;
                }else{
                    map_key.hit = true;
                }

                error = -KEY_ERROR_RANGE;

                state.score += error.abs();
                if INCLUDE_MISSES_IN_HISTOGRAM {
                    state.errors.push(error);
                }

                state.messages.push(Message { val: MessageVal::Miss, time: game_time, direction: map_key.direction });
            }
        }
    };

    let mut window = RenderWindow::new(
        (width, height),
        "Rhythm Game!",
        WINDOW_STYLE,
        &Default::default(),
    );

    // no v-sync to help with latency
    window.set_vertical_sync_enabled(false);
    window.set_mouse_cursor_visible(false);
    window.set_key_repeat_enabled(ENABLE_KEY_REPEAT);

    // include_bytes! builds this font into our executable, meaning that we do not need to bring
    // a resources/ folder around. very handy!
    // we unwrap because it should crash if the font isn't there (a bug)
    let font = Font::from_memory(include_bytes!("resources/Roboto-Regular.ttf")).unwrap();

    let folder_result = load_map_folder("maps/included/clicktrack").unwrap();


    let easy_str = &"Easy".to_string();

    let easy = if folder_result.difficulties.contains(easy_str) { easy_str } else { &folder_result.difficulties[folder_result.difficulties.len() - 1] };

    let state = State {
        keys: KeysPressed {
            up: false,
            down: false,
            left: false,
            right: false,
        },
        keys_released: KeysPressed {
            up: false,
            down: false,
            left: false,
            right: false,
        },
        messages: vec![],
        map: folder_result.difficulties_map.get(easy).unwrap().to_vec(),
        score: 0f64,
        game_time: -folder_result.meta.clone().delay as f64,
        quit: false,
        paused: true,
        song_playing: false,
        errors: vec![],
        song_meta: folder_result.meta
    };


    // supports ogg, wav, flac, aiff, and others, see https://docs.rs/sfml/0.14.0/sfml/audio/struct.SoundBuffer.html
    let song_buffer = SoundBuffer::from_file(&folder_result.song).unwrap();
    let mut song = Sound::with_buffer(&song_buffer);

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

                    if state.quit { break; }

                    if state.paused { continue; }

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

    let mut last_60_frames: Vec<u128> = vec![];

    let mut frame = 0;

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
                    } => state.keys_released.up = true,

                    Event::KeyPressed {
                        code: Key::Down, ..
                    } => state.keys.down = true,
                    Event::KeyReleased {
                        code: Key::Down, ..
                    } => state.keys_released.down = true,

                    Event::KeyPressed {
                        code: Key::Left, ..
                    } => state.keys.left = true,
                    Event::KeyReleased {
                        code: Key::Left, ..
                    } => state.keys_released.left = true,

                    Event::KeyPressed {
                        code: Key::Right, ..
                    } => state.keys.right = true,
                    Event::KeyReleased {
                        code: Key::Right, ..
                    } => state.keys_released.right = true,

                    Event::Resized { width: new_width, height: new_height } => {
                        width = new_width;
                        height = new_height;
                        window.set_view(&*View::new(Vector2::from((width as f32 / 2., height as f32 / 2.)), Vector2::from((width as f32, height as f32))));
                    }
                    _ => {}
                }
            }

            window.clear(Color::BLACK);

            let duration = start.elapsed();

            // unpause once the timer runs out
            if state.paused && duration.as_secs() >= WARMUP_SECS as u64 {
                state.paused = false;
            }

            // pause timer
            if state.paused {
                let mut text = Text::new(&format!("{}", WARMUP_SECS as u64 - duration.as_secs()), &font, 100);
                text.set_fill_color(Color::WHITE);
                text.set_position((0., 0.));
                window.draw(&text);
            } else {
                // set the time
                state.game_time = (duration.as_micros()) as f64 / 1000f64 - (WARMUP_SECS * 1000) as f64 + (SECONDS_TO_SKIP * 1000) as f64 +  (-state.song_meta.delay as f64);
            }

            if !state.song_playing && !state.paused && state.game_time - (-state.song_meta.delay as f64) >= 0.{
                song.set_playing_offset(Time::seconds(0.));
                song.play();
                state.song_playing = true;
            }

            // draw map keys
            for map_key in &state.map {

                let mut rect = RectangleShape::new();
                let screen_time = (map_key.time - state.game_time) as f32;
                let screen_time_end = (map_key.time_end - state.game_time) as f32;

                // screen_pos is the exact pixel that you want to "hit"
                let screen_pos = time_to_screen_height(screen_time, height, true);

                // screen_pos is the exact pixel that you want to "hit"
                let screen_pos_end = if map_key.time_end == -1. { screen_pos } else { time_to_screen_height(screen_time_end, height, true)  };

                // if the key is definitely not on the screen, we can quit right now
                if ((screen_pos < -(NO_RENDER_BUFFER as f32) && screen_pos_end < -(NO_RENDER_BUFFER as f32)) ||
                    (screen_pos > (height + NO_RENDER_BUFFER) as f32 && screen_pos_end > (height + NO_RENDER_BUFFER) as f32)) ||
                    map_key.hit {
                    continue;
                }

                let key_idx = map_key_disp_order.iter().position(|&r| r == map_key.direction).unwrap();
                let x = 100. + key_idx as f32 * NOTE_COLUMN_WIDTH;


                for key_range in &height_and_saturation_map {
                    // should be an even integer
                    let rect_height = key_range.0;
                    let mut saturation = key_range.1;
                    let mut value = 1.0;

                    if map_key.hit_start {
                        saturation /= 2.;
                        value /= 3.;
                    }

                    let (r, g, b) = hsv_to_rgb(((map_key.time as usize / 75) % 360) as u32, saturation, value);

                    rect.set_fill_color(Color::rgb(r as u8, g as u8, b as u8));
                    rect.set_position((x, (screen_pos as f32) - rect_height / 2.));

                    rect.set_size((100f32, rect_height - (screen_pos_end - screen_pos).abs()));
                    window.draw(&rect);
                }

            }

            // draw key names & messages
            for direction in &map_key_disp_order {
                let key_idx = map_key_disp_order.iter().position(|&r| r == *direction).unwrap();
                let x = 100. + key_idx as f32 * NOTE_COLUMN_WIDTH;


                let mut y: f32 = 0.;
                for message in &state.messages {
                    if message.direction != *direction { continue; }

                    let mut text = Text::new(&format!("{:?}", message.val), &font, MESSAGE_SIZE);

                    let time_ratio = ((state.game_time - message.time) / MESSAGE_DURATION).min(1.).max(0.);

                    let color = (fade_in_out(time_ratio) * 256.) as u8;

                    text.set_fill_color(Color::rgb(color, color, color));

                    let msg_height = y * MESSAGE_SIZE as f32 + 1.;
                    text.set_position((x, if SCROLL_DOWNWARDS { height as f32 - msg_height - MESSAGE_SIZE as f32 } else { msg_height }));
                    window.draw(&text);

                    y += 1.;
                }

                let mut text = Text::new(&format!("{:?}", direction), &font, 20);
                text.set_fill_color(Color::WHITE);
                text.set_position((x, if SCROLL_DOWNWARDS { height as f32 - 70. - MESSAGE_SIZE as f32 } else { 70. }));
                window.draw(&text);
            }

            {
                // the line should be at time "0"
                let line_height = time_to_screen_height(0., height, false) - 1.;

                let oldest_time_key = state.map.iter().max_by(|x, y| (if x.time_end == -1. { x.time } else { x.time_end }).partial_cmp(if y.time_end == -1. { &y.time } else { &y.time_end }).unwrap()).unwrap();
                let oldest_time = if oldest_time_key.time_end == -1. { oldest_time_key.time } else { oldest_time_key.time_end };

                let time_ratio = (state.game_time / oldest_time).min(1.).max(0.);

                let time_screen = width as f32 * time_ratio as f32;

                let mut hit_line = RectangleShape::new();
                hit_line.set_position((0., line_height));
                hit_line.set_size((width as f32, 3.));
                hit_line.set_fill_color(Color::rgb(127, 127, 127));

                window.draw(&hit_line);

                let mut hit_line_progress = RectangleShape::new();
                hit_line_progress.set_position((0., line_height));
                hit_line_progress.set_size((time_screen, 3.));
                hit_line_progress.set_fill_color(Color::WHITE);

                window.draw(&hit_line_progress);
            }

            // draw histogram
            {
                let screen_center = width as f32 / 2.;
                let bar_start = screen_center - (HISTOGRAM_WIDTH / 2.);

                let histogram_bottom = if SCROLL_DOWNWARDS { height as f32 } else { LINE_HEIGHT };

                if HISTOGRAM_BACKGROUND {
                    let mut error_bar_line = RectangleShape::new();
                    error_bar_line.set_fill_color(Color::WHITE);
                    error_bar_line.set_position((bar_start, (histogram_bottom - HISTOGRAM_HEIGHT)));
                    error_bar_line.set_size((HISTOGRAM_WIDTH, HISTOGRAM_HEIGHT));
                    window.draw(&error_bar_line);
                }

                if !state.errors.is_empty() {
                    let mut histogram_bins: Vec<usize> = vec![0; HISTOGRAM_BINS];

                    for error in &state.errors {
                        // [-1, 1], inverted so that right = too late, left = too early
                        let normalized_error = -(error / KEY_ERROR_RANGE);
                        // https://www.desmos.com/calculator/dyfup8vcsz
                        let bin = ((normalized_error / 2. + 0.5) * HISTOGRAM_BINS as f64).floor() as usize;

                        match histogram_bins.get_mut(bin) {
                            Some(elem) => *elem += 1,
                            None => histogram_bins.insert(bin, 1)
                        }
                    }

                    let highest_val_in_histogram = *histogram_bins.iter().max().unwrap();
                    for bin_pos in 0..histogram_bins.len() {
                        let bin_val = histogram_bins[bin_pos];
                        let bin_x_pos = bar_start + (bin_pos as f32 / HISTOGRAM_BINS as f32) * HISTOGRAM_WIDTH;
                        let bin_height = (bin_val as f32 / highest_val_in_histogram as f32) * HISTOGRAM_HEIGHT;

                        let mut bin = RectangleShape::new();

                        let (r, g, b) = hsv_to_rgb(((-(bin_pos as f32 / HISTOGRAM_BINS as f32 - 0.5).abs() + 0.5) * 256.) as u32, 1., 1.);
                        bin.set_fill_color(Color::rgb(r as u8, g as u8, b as u8));
                        bin.set_position((bin_x_pos, histogram_bottom - bin_height));
                        bin.set_size((HISTOGRAM_WIDTH / HISTOGRAM_BINS as f32, bin_height));
                        window.draw(&bin);
                    }
                }

                let histogram_lines = vec![screen_center - (HISTOGRAM_WIDTH / 2.), screen_center - 1., screen_center + (HISTOGRAM_WIDTH / 2.)];
                for x in histogram_lines {
                    let mut histogram_line = RectangleShape::new();
                    histogram_line.set_fill_color(Color::WHITE);
                    histogram_line.set_position((x, (histogram_bottom - HISTOGRAM_HEIGHT)));
                    histogram_line.set_size((2., HISTOGRAM_HEIGHT));
                    window.draw(&histogram_line);
                }
            }

            let elapsed = last_frame.elapsed().as_millis();

            let current_fps = 1000u128 / max(elapsed, 1u128);

            if frame % 10 == 0 { last_60_frames.push(current_fps); }
            if last_60_frames.len() > 60 {
                last_60_frames.remove(0);
            }
            let fps = last_60_frames.iter().sum::<u128>() as usize / last_60_frames.len();

            let text_height = if SCROLL_DOWNWARDS { height as f32 - LINE_HEIGHT } else { 0. };

            let height_direction = if SCROLL_DOWNWARDS { -1. } else { 1. };

            let meta = &state.song_meta;

            let mut title_text = Text::new(&format!("{}", meta.title), &font, 22);
            title_text.set_fill_color(Color::WHITE);
            title_text.set_position((700., text_height));
            window.draw(&title_text);

            let mut sub_text = Text::new(&format!("{} - {}", meta.subtitle, meta.artist), &font, 12);
            sub_text.set_fill_color(Color::WHITE);
            sub_text.set_position((700., text_height + 25.));
            window.draw(&sub_text);


            let mut score_and_fps = Text::new(&format!("{} FPS\nScore: {:.3}", fps, state.score / 1000.), &font, 15);
            score_and_fps.set_fill_color(Color::WHITE);
            score_and_fps.set_position((WIDTH as f32 - 120., text_height));
            window.draw(&score_and_fps);

            let mut c = state.messages.clone();
            c.retain(|message| state.game_time - message.time < MESSAGE_DURATION);
            state.messages = c;
        }

        last_frame = Instant::now();

        frame += 1;

        window.display();
    }

    (*state_holder.lock().unwrap()).quit = true;
    physics_thread.join().unwrap();
}
