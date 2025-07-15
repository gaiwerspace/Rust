fn main() {

    let maybe_number = Some(42); // or None in some cases
    if let Some(num) = maybe_number {
        println!("The number is {}", num);
    } else {
        println!("No number provided.");
    }

    // let mut x = 0;
    // while x < 5 {
    //     println!("{}", x);
    //     if x == 3 {
    //         continue;
    //     } // skipping iteration when x is equal to 3.
    //     x += 1;
    // }


    for i in (1..=5). {
        println!("{}", i);
    }

    // a unit function that doesn't return anything
    fn print_sum(numbers: &[i32]) {
        let sum = numbers.iter().sum(); // Calculate the sum of elements in slice
        if sum % 2 == 0 {               // Check if sum is even
            println!("The sum is even.");
        } else {
            println!("The sum is odd.");
        }
    }

    let numbers = [1, 2, 3];      // Define a slice of integers
    print_sum(&numbers);          // Call the unit function with the slice as an argument

}
