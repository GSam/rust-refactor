fn foo(a:u32, &z:&u32, &mut c:&mut u32) {
    let _ = a + z + c;
    println!("{} {} {}", a, b, c);
}

fn main() {
    foo(1, &2, &mut 3);
}
