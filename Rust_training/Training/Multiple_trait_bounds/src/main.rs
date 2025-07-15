trait Area {
    fn area(&self) -> f64;
}
trait Perimeter {
    fn perimeter(&self) -> f64;
}

fn get_area_and_perimeter<T>(shape: &T) -> (f64, f64) where T: Area + Perimeter {
    (shape.area(), shape.perimeter())
}

struct Rectangle {
    width: f64,
    height: f64,
}
impl Area for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

impl Perimeter for Rectangle {
    fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }
}

struct Container<T: Area + Perimeter> {
    shape1: T,
    shape2: T,
}

fn main() {
    println!("Start Multiple_trait_bounds!");

    let r1 = Rectangle {
        width: 3.0,
        height: 4.0,
    };

    let t = get_area_and_perimeter(&r1);
    println!("Area: {}, Perimeter: {}", t.0, t.1);

     let r2 = Rectangle {
        width: 19.0,
        height: 5.0,
    };

    let c = Container {
        shape1: r1,
        shape2: r2,
    };
    println!(
        "Container shapes area: {}, perimeter: {}",
        c.shape1.area() + c.shape2.area(),
        c.shape1.perimeter() + c.shape2.perimeter()
    );

    println!("End Multiple_trait_bounds!");


}
