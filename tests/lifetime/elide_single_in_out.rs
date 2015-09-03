use std::fmt::Debug;

fn foo(x: &Debug) {
    println!("{:?}", x);
}

fn main() {
    foo(&2);
    foo(&"STRING");
}
