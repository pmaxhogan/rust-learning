use std::thread;
use std::time::Duration;
use std::sync::{Mutex, Arc};

#[derive(Debug)]
struct Thing{
    a: bool
}

#[derive(Debug)]
struct State{
    some_vec: Vec<Thing>
}

fn main(){
    let mut state = State{
        some_vec: Vec::new()
    };

    state.some_vec.push(Thing {
        a: true
    });


    // not completely sure how Arc<Mutex<T>> works but it does
    // see https://doc.rust-lang.org/book/ch16-03-shared-state.html
    let state_holder = Arc::new(Mutex::new(state));
    let mut handles = vec![];

    {
        let counter = Arc::clone(&state_holder);
        let handle = thread::spawn(move || {
            loop {
                // we need this block to ensure that our MutexGuard goes out of scope (and is freed)
                // before we sleep. if we sleep before releasing the lock, then we will basically
                // get the lock as soon we release it, preventing the main thread from getting it!
                {
                    let mut state_guard = counter.lock().unwrap();
                    let state = &mut *state_guard;
                    state.some_vec.push(Thing {
                        a: true
                    });
                }
                thread::sleep(Duration::from_millis(1000 / 15));
            }
        });
        handles.push(handle);
    }

    loop{
        {
            let counter = Arc::clone(&state_holder);

            println!("getting lock");
            let mut state_guard = counter.lock().unwrap();
            println!("got lock");

            let state = &mut *state_guard;
            println!("{:#?}", state.some_vec.len());
        }
        thread::sleep(Duration::from_millis(1000 / 60));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
