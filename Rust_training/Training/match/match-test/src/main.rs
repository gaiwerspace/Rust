fn main() {
    println!("Hello, match!");
    let result = devide(0, 0);

    match result {
        Ok(message) => println!("Result: {}", message),
        Err(data) => println!("Error: {}", data),
    }

}

fn devide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        return Err("Division by zero".to_string());
    }
     else {
        Ok(a / b)
     }
}
