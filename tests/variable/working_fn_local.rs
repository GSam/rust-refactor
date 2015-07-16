fn main() {
    let a = 2;
    {
        struct Foo;
        let b = Foo;
    }
}
