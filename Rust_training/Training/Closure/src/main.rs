fn main() {
    let message = "Hello, Rus from anonymous function!";
    let func = || {
        println!("Message: {}", message);
    };

    func();

    let mut number = 0;;
    let mut add_to_number = || {
        number = number + 1;
        println!("Updated number: {}", number);
    };
    add_to_number();
    add_to_number();
}
