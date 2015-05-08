static mut FOO : i32 = 42;

unsafe fn do_it() -> i32 {
  let foo : i32 = 55;
  return FOO + foo;
}

fn main() {
  unsafe {
    println!("Values of variables once: {}", FOO);
    FOO = 37;
    println!("Values of variables twice: {}", do_it());
  }
}
