fn main() {
    println!("Hello, world!");
    let x = 5;
    let y = 10;
    let sum = x + y;
    println!("The sum of {} and {} is {} ", x, y, sum);

    let a: i32 = 3;
    let b: i32 = 4;
    let result: bool = a == b;
    print!("{} == {} is {}", a, b, result);
    let r = a != b;
    println!("{} != {} is {} ", a, b, r);

    let number: i32 = 5;
   
    match number {
        1 => println!("One"),
        2 => println!("Two"),
        3 => println!("Three"),
        4 => println!("Four"),
        5 => println!("Five"),
        _ => println!("Not a number between 1 and 5"),
    };

    let check= match number {
        1 => "one",
        2 => "two",
        3 => "three",
        4 => "four",
        5 => "five",
        _ => "not a number between 1 and 5",
    };


    println!("The check is {:?}", check);
}

#[test]
fn check() {

}