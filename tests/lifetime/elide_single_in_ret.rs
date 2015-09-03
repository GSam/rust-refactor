use std::fmt::Debug;

fn foo<'a>(x: &'a Debug) -> &'a Debug {
    println!("{:?}", x);
    x
}

fn main() {
    foo(&2);
    foo(&"STRING");
}
