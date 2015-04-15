fn main() {
	let x = 10;
	let y = 20;
	let hello = 30;

	let mut i = 0;
	while i < 10 {
		let j = 2 * i;
		let k = 2 * i * j;
		let z = hello * y * x;
		println!("{} {} {}", j, k, z);
		i += 1;
	}

}
