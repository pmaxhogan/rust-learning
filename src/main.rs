// https://en.wikipedia.org/wiki/Collatz_conjecture
// returns the nth hailstone number
// u32 gives the best performance for the sizes of numbers that we can compute in a reasonable amount
// of time (tested with benchmark)
// this means that the highest number we can return or take as input is 4294967295
fn hailstone (number : u128) -> u128 {
    print!("{} ", number);

    return if number == 1 {
        1
    } else if number % 2 == 0 {
        hailstone(number / 2)
    } else {
        hailstone(3 * number + 1)
    }
}

fn main() {
    println!("Hailstone sequence");

    // calculate the hailstone sequence (?) for the highest unsigned 128-bit number
    hailstone(u128::max_value());

    print!("");
}
