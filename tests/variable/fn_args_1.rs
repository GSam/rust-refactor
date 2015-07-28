fn foo(a:u32, b:u32, c:u32) {
    let _ = a + b + c;
    println!("{} {} {}", a, b, c);
}

fn main() {
    foo(1, 2, 3);
}
