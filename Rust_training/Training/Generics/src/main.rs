struct User<T> {
    id: T,
    name: String,
}


impl User <u32> {
    fn compare(&self, other: &User<u32>) -> bool {
        self.id == other.id
    }
}

fn get_return<T>(value: T) -> T {
    value
}

fn unless_function<T, U>(p: T) -> Option<U> {
    Option::None
}


fn main() {
    let mut p1 = User {
        id: 1,
        name: String::from("Nikita"),
    };

    get_return(1);
    get_return("some".to_string());
    p1 = get_return(p1);

    let result : Option<String> = unless_function(1);

    let x = p1.compare_id(1);
}
