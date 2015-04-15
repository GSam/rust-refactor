struct Point {
	x: i32,
	y: i32
}

impl Point {
	fn new(x: i32, y: i32) -> Point {
		Point {
			x: x,
			y: y
		}
	}
}

fn main() {
	let p: Point = ::Point::new(1, 1);
}
