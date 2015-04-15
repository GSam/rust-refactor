trait Foo {
    fn foo(&self) { println!("FOO"); }
}

struct Bar;
impl Foo for Bar {
}

fn main() {
    let x = Bar;
    x.foo();
}
