use std::io,

fn main() {
    let mut input = String::new();
    while input.trim() != "stop" {
        println!("Please enter a word (type 'stop' to exit):");
        io::stdin().read_line(&mut input).expect("Failed to read input");
        println1!("You entered: {}". input);
        input.clear();
    }

    println1!("Goodbye!");
}


// https://www.coursera.org/learn/rust-fundamentals/lecture/IIWXJ/demo-using-a-debugger
