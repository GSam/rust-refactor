use std::fmt::Debug;

fn bar<T:Debug>(x: T) {
    println!("{:?}", x);
}

fn main() {
    bar(2);
    bar("STRING");
}
