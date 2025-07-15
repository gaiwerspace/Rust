use std::slice::Iter;

fn main() {

    //into_iter - итератор по значениямx    
    //iter - итератор по немутабельным ссылка
    //iter_mut - итератор по мутабельным ссылкам

    //into_iter - итератор по значениям, c семантикой перемещения
    let v = vec![1, 2, 3];
    let mut iter = v.into_iter();
    // let first = iter.next();
    // let second = iter.next();
    // let third = iter.next();
    // println!("{:#?} ", first);
    // println!("{:?} {:?} {:?}", first, second, third);

    for i in iter {
        println!("{}", i);
    }


    // iter - итератор по немутабельным ссылкам
    let v = vec![1, 2, 3, 4, 5];
    let ref_iter: Iter<i32> = v.iter();
    for i in ref_iter {
        println!("{}", i);
    }
    println!("First element in vector: {}", v[0]);  


    // iter_mut - итератор по мутабельным ссылкам
    let mut v = vec![1, 2, 3, 4, 5, 7, 8, 9, 10];
    let ref_iter_mut = v.iter_mut();
    for i in ref_iter_mut {
        *i += 3; // увеличиваем каждое значение на 1
    }
    println!("Second element in vector: {}", v[1]);  



    //map -Takes a closure and creates an iterator
    //which call that closure on each element
    //collect Transforms an iterator into a collection
    let mut v = vec![1, 2, 3, 4, 5];
    let v2: Vec<i32> = v.iter().map(|x| x + 1).collect();
    println!("Original vector: {:?}", v);
    println!("New vector: {:?}", v2);


    //Фильтрация векторов
    let vector= vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let new_vec= vector.iter()
        .filter(|&&x| x % 2 == 0) // фильтруем четные числа
        .map(|&x| x * 2) // умножаем на 2
        .collect::<Vec<i32>>(); // собираем в новый вектор

    print!("Original vector_2: {:?}\n", vector);
    print!("New vector_2: {:?}\n", new_vec);

    println!("The End!");
}
