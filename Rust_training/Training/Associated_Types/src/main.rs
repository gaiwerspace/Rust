struct Container(i32, i32);

// Типаж, который проверяет, сохранены ли 2 элемента в контейнере.
// Также он может вернуть первое или последнее значение.
trait Contains {
    // Объявляем общие типы, которые будут использовать методы.
    type A;
    type B;

    fn contains(&self, _: &Self::A, _: &Self::B) -> bool;
    fn first(&self) -> i32;
    fn last(&self) -> i32;
}

impl Contains for Container {
    // Определяем, какими будут типы `A` и `B`. Если `входящий` тип
    // `Container(i32, i32)`, тогда `выходящие` типы определяются, как
    // `i32` и `i32`.
    type A = i32;
    type B = i32;

    // `&Self::A` и `&Self::B` также будут здесь уместны.
    fn contains(&self, number_1: &i32, number_2: &i32) -> bool {
        (&self.0 == number_1) && (&self.1 == number_2)
    }
    // Берём первую цифру.
    fn first(&self) -> i32 { self.0 }

    // Берём последнюю цифру.
    fn last(&self) -> i32 { self.1 }
}

fn difference<C: Contains>(container: &C) -> i32 {
    container.last() - container.first()
}

fn main() {
    let number_1 = 3;
    let number_2 = 10;

    let container = Container(number_1, number_2);

    println!("Содержатся ли в контейнере {} и {}: {}",
        &number_1, &number_2,
        container.contains(&number_1, &number_2));
    println!("Первое число: {}", container.first());
    println!("Последнее число: {}", container.last());

    println!("Разница: {}", difference(&container));
}
