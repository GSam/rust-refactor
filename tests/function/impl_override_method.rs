trait Foo {
    fn foo(&self) { println!("FOO"); }
}

struct Bar;
impl Foo for Bar {
    fn foo(&self) { println!("BAR"); }
}

fn main() {
    let x = Bar;
    x.foo();
}
