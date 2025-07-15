fn main () {
    let age = 20;
    let is_teenager = if age >= 13 && age <= 19 {
        true
    } else {
        false
    };
    println!("Is the person a teenager? {}", is_teenager);
}