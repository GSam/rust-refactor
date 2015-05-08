trait Foo {
    fn foo(&self) { println!("FOO"); }
}

struct Bar;
impl Foo for Bar {
    fn foo(&self) { println!("BARFOO"); }
}
impl Bar {
    fn baz(&self) { println!("BAZ"); }
}

fn main() {
    let x = Bar;
    x.foo();
    x.baz();
}
