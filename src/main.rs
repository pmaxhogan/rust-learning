use uint::construct_uint;
// U1024 with 1024 bits consisting of 16 x 64-bit words
construct_uint! {
	pub struct u1024(16);
}

// https://en.wikipedia.org/wiki/Collatz_conjecture
// returns the nth hailstone number
fn hailstone (number : u1024) -> u1024 {
    print!("{} ", number);

    return if number == u1024::from(1) {
        u1024::from(1)
    } else if number % 2 == u1024::from(0) {
        hailstone(number / 2)
    } else {
        hailstone(u1024::from(3) * number + u1024::from(1))
    }
}

fn main() {
    println!("Hailstone sequence");

    // calculate the hailstone sequence (?) for a really high number
    // this number is *really* big, 308 digits in total
    hailstone(u1024::max_value() / u1024::from(5));

    print!("");
}

