use rand::Rng;
use std::io;
use std::cmp::Ordering;

fn main() {
    println!("Guess the number!");

    // pick a random number 1-101 inclusive
    let secret_number = rand::thread_rng().gen_range(1, 101);

    // loop until we guess correctly
    loop {
        println!("Please input your guess, 1-100.");

        let mut guess = String::new();

        // read a number
        io::stdin()
            .read_line(&mut guess)
            .expect("Fail to read line");

        // continue jumps to the beginning of the loop, re-asking for the number, if you don't input a number.
        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        // print what you guessed (not super necessary but whatever)
        println!("You guessed: {}", guess);

        // compare the guess using a fancy match expression
        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                // print a win and exit the loop to end the program nicely
                println!("You win!");
                break;
            }
        }
    }
}
