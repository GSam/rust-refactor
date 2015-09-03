use std::fmt::Debug;

fn foo<'a>(x: &'a Debug) {
    println!("{:?}", x);
}

fn main() {
    foo(&2);
    foo(&"STRING");
}
