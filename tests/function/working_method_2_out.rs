trait Foo {
    fn func(&self) { println!("FOO"); }
}

struct Bar;
impl Foo for Bar {
    fn func(&self) { println!("BAR"); }
}

fn main() {
    let x = Bar;
    x.func();
}
