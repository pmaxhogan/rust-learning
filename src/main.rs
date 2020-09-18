use rand::Rng;
use std::io;
use std::cmp::Ordering;

fn game_loop() {
    println!("Guess the number!");

    // pick a random number 1-101 inclusive
    let secret_number = rand::thread_rng().gen_range(1, 101);

    // loop until we guess correctly
    loop {
        println!("Please input your guess, 1-100.");

        let mut guess = String::new();

        // read a line to the guess tring
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

    // once you've won, ask to play again
    println!("Would you like to play again (yes/no)?");
    loop {
        // read the answers
        let mut answer = String::new();
        io::stdin()
            .read_line(&mut answer)
            .expect("Failed to read answer");


        // not really sure what as_ref() does tbh
        match answer.trim().as_ref() {
            "yes" => {
                // restart the game
                game_loop();
            }
            "no" => {
                // end the game
                println!("Goodbye!");
                break;
            }
            _ => {
                // prompt them again
                println!("Please input yes or no.");
                continue;
            }
        }
    }
}

fn main() {
    println!("Welcome to Guess the Number!");

    game_loop();
}
