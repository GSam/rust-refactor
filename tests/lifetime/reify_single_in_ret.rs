use std::fmt::Debug;

fn foo(x: &Debug) -> &Debug {
    println!("{:?}", x);
    x
}

fn main() {
    foo(&2);
    foo(&"STRING");
}
