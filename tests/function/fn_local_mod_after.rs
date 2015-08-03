fn main() {
    foo::foo();
    mod foo {
        pub fn foo () {
        }
    }
}
