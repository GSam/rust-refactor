use std::fmt::Debug;

fn foo<T:Debug>(x: T) {
    println!("{:?}", x);
}

fn main() {
    foo(2);
    foo("STRING");
}
