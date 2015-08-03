fn main() {
    foo::bar();
    mod foo {
        pub fn bar () {
        }
    }
}
