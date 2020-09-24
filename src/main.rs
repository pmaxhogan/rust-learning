// returns the nth fibonacci number
// u32 gives the best performance for the sizes of numbers that we can compute in a reasonable amount
// of time (tested with benchmark)
// this means that the highest number we can return or take as input is 4294967295
fn fibonacci (number : u32) -> u32 {
    if number == 0 || number == 1 {
        return number;
    }
    return fibonacci(number - 1) + fibonacci(number - 2);
}

fn main() {
    println!("Fibonacci benchmark");

    for i in 0..40 {
        println!("#{}: {}", i, fibonacci(i))
    }
}
