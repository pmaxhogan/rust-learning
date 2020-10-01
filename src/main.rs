use std::io;

fn main() {
    // where we store the numbers that we collect
    let mut numbers: Vec<u16> = Vec::new();

    loop{
        println!("Input a whole number (1-999) or \"done\". ({} numbers inputted so far)", numbers.len());

        let mut num = String::new();

        // read the num from stdin
        io::stdin()
            .read_line(&mut num)
            .expect("Could not read line");

        // convert to lowercase to allow for things like "Done" or "done" (or "dOnE")
        match num.trim().to_ascii_lowercase().as_ref() {
            "done" => {
                if numbers.len() < 1 {
                    println!("You need to input at least one number!");
                    continue;
                }

                // if we have at least one value, compute statistics below
                break;
            }
            _ => {
                let guess: u16 = match num.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("Please input a whole number or \"done\".");
                        continue;
                    }
                };

                if guess > 0 && guess < 1000 {
                    numbers.push(guess);
                }else {
                    println!("Your number must be from 1 to 999 inclusive.")
                }
            }
        }
    }


    // built-in min() and max() is cool,
    // making a new iter() for each one (required) and checking that a min or max exists (ie. the
    // list is not empty, even though we check that above!) is not cool, but that's ok
    let min_value = numbers.iter().min();
    match min_value {
        Some(min) => println!( "Min value: {}", min ),
        None => {}
    }

    let max_value = numbers.iter().max();
    match max_value {
        Some(max) => println!( "Max value: {}", max),
        None => {}
    }

    // we need to tell sum() that we want it to sum as a u16, otherwise we get a usize
    // and a usize won't work, because then we can't cast it to f32
    // but when we provide u16, then we can safe cast it to f32
    // we need it to be f32 so that we can divide later and still have decimal places
    let sum = numbers.iter().sum::<u16>() as f32;
    let len = numbers.len() as f32;
    let average : f32 = sum / len;

    // special println! syntax rounds to 2 decimal places!
    println!("Average: {:.2}", average);
}
