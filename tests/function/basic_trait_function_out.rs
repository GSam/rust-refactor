use std::fmt::Debug;

fn bar(x: &Debug) {
    println!("{:?}", x);
}

fn main() {
    bar(&2);
    bar(&"STRING");
}
