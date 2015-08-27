use std::fmt::Debug;

fn foo<'a>(x: &'a Debug, y: &Debug) {
    println!("{:?} {:?}", x, y);
}

fn main() {
    foo(&2, &2);
    foo(&"STRING", &"STRING");
}

