struct Point {
    x: u32,
    y: u32
}

fn main() {
    let Point{x, y} = Point{x: 1, y: 2};
    println!("{} {}", x, y);
}
