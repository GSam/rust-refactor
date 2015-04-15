extern crate refactor;

static ANALYSIS: &'static str = "crate,file_name,\"test4.rs\",file_line,1,file_col,0,extent_start,0,extent_start_bytes,0,file_line_end,15,file_col_end,0,extent_end,204,extent_end_bytes,204,name,\"unknown_crate\"
external_crate,name,\"alloc\",crate,\"5\",file_name,\"test4.rs\"
external_crate,name,\"rand\",crate,\"7\",file_name,\"test4.rs\"
external_crate,name,\"std\",crate,\"1\",file_name,\"test4.rs\"
external_crate,name,\"collections\",crate,\"3\",file_name,\"test4.rs\"
external_crate,name,\"unicode\",crate,\"4\",file_name,\"test4.rs\"
external_crate,name,\"libc\",crate,\"6\",file_name,\"test4.rs\"
external_crate,name,\"core\",crate,\"2\",file_name,\"test4.rs\"
end_external_crates
function,file_name,\"test4.rs\",file_line,1,file_col,3,extent_start,3,extent_start_bytes,3,file_line_end,1,file_col_end,7,extent_end,7,extent_end_bytes,7,id,\"4\",qualname,\"::main\",declid,\"\",declidcrate,\"\",scopeid,\"0\"
variable,file_name,\"test4.rs\",file_line,2,file_col,5,extent_start,18,extent_start_bytes,18,file_line_end,2,file_col_end,6,extent_end,19,extent_end_bytes,19,id,\"9\",name,\"x\",qualname,\"x$9\",value,\"x = 10\",type,\"i32\",scopeid,\"0\"
variable,file_name,\"test4.rs\",file_line,3,file_col,5,extent_start,32,extent_start_bytes,32,file_line_end,3,file_col_end,6,extent_end,33,extent_end_bytes,33,id,\"13\",name,\"y\",qualname,\"y$13\",value,\"y = 20\",type,\"i32\",scopeid,\"0\"
variable,file_name,\"test4.rs\",file_line,4,file_col,5,extent_start,46,extent_start_bytes,46,file_line_end,4,file_col_end,6,extent_end,47,extent_end_bytes,47,id,\"17\",name,\"z\",qualname,\"z$17\",value,\"z = 30\",type,\"i32\",scopeid,\"0\"
variable,file_name,\"test4.rs\",file_line,6,file_col,9,extent_start,66,extent_start_bytes,66,file_line_end,6,file_col_end,10,extent_end,67,extent_end_bytes,67,id,\"21\",name,\"i\",qualname,\"i$21\",value,\"<mutable>\",type,\"i32\",scopeid,\"0\"
var_ref,file_name,\"test4.rs\",file_line,7,file_col,7,extent_start,81,extent_start_bytes,81,file_line_end,7,file_col_end,8,extent_end,82,extent_end_bytes,82,refid,\"21\",refidcrate,\"0\",qualname,\"\",scopeid,\"4\"
variable,file_name,\"test4.rs\",file_line,8,file_col,6,extent_start,97,extent_start_bytes,97,file_line_end,8,file_col_end,7,extent_end,98,extent_end_bytes,98,id,\"30\",name,\"j\",qualname,\"j$30\",value,\"j = 2 * i\",type,\"i32\",scopeid,\"0\"
var_ref,file_name,\"test4.rs\",file_line,8,file_col,14,extent_start,105,extent_start_bytes,105,file_line_end,8,file_col_end,15,extent_end,106,extent_end_bytes,106,refid,\"21\",refidcrate,\"0\",qualname,\"\",scopeid,\"4\"
variable,file_name,\"test4.rs\",file_line,9,file_col,6,extent_start,115,extent_start_bytes,115,file_line_end,9,file_col_end,7,extent_end,116,extent_end_bytes,116,id,\"36\",name,\"k\",qualname,\"k$36\",value,\"k = 2 * i * j\",type,\"i32\",scopeid,\"0\"
var_ref,file_name,\"test4.rs\",file_line,9,file_col,14,extent_start,123,extent_start_bytes,123,file_line_end,9,file_col_end,15,extent_end,124,extent_end_bytes,124,refid,\"21\",refidcrate,\"0\",qualname,\"\",scopeid,\"4\"
var_ref,file_name,\"test4.rs\",file_line,9,file_col,18,extent_start,127,extent_start_bytes,127,file_line_end,9,file_col_end,19,extent_end,128,extent_end_bytes,128,refid,\"30\",refidcrate,\"0\",qualname,\"\",scopeid,\"4\"
variable,file_name,\"test4.rs\",file_line,10,file_col,6,extent_start,137,extent_start_bytes,137,file_line_end,10,file_col_end,7,extent_end,138,extent_end_bytes,138,id,\"44\",name,\"z\",qualname,\"z$44\",value,\"z = z * y * x\",type,\"i32\",scopeid,\"0\"
var_ref,file_name,\"test4.rs\",file_line,10,file_col,10,extent_start,141,extent_start_bytes,141,file_line_end,10,file_col_end,11,extent_end,142,extent_end_bytes,142,refid,\"17\",refidcrate,\"0\",qualname,\"\",scopeid,\"4\"
var_ref,file_name,\"test4.rs\",file_line,10,file_col,14,extent_start,145,extent_start_bytes,145,file_line_end,10,file_col_end,15,extent_end,146,extent_end_bytes,146,refid,\"13\",refidcrate,\"0\",qualname,\"\",scopeid,\"4\"
var_ref,file_name,\"test4.rs\",file_line,10,file_col,18,extent_start,149,extent_start_bytes,149,file_line_end,10,file_col_end,19,extent_end,150,extent_end_bytes,150,refid,\"9\",refidcrate,\"0\",qualname,\"\",scopeid,\"4\"
var_ref,file_name,\"test4.rs\",file_line,12,file_col,2,extent_start,189,extent_start_bytes,189,file_line_end,12,file_col_end,3,extent_end,190,extent_end_bytes,190,refid,\"21\",refidcrate,\"0\",qualname,\"\",scopeid,\"4\"";

#[test]
fn working_rename_1() {
	let input = "fn main() {
	let x = 10;
	let y = 20;
	let z = 30;

	let mut i = 0;
	while i < 10 {
		let j = 2 * i;
		let k = 2 * i * j;
		let z = z * y * x;
		println!(\"{} {} {}\", j, k, z);
		i += 1;
	};

}";

	let output = "fn main() {
	let hello = 10;
	let y = 20;
	let z = 30;

	let mut i = 0;
	while i < 10 {
		let j = 2 * i;
		let k = 2 * i * j;
		let z = z * y * hello;
		println!(\"{} {} {}\", j, k, z);
		i += 1;
	};

}";

	assert_eq!(output, refactor::refactor::rename_variable(&input, &ANALYSIS, "hello", "9"));

}

#[test]
fn working_rename_2() {
	let input = "fn main() {
	let x = 10;
	let y = 20;
	let z = 30;

	let mut i = 0;
	while i < 10 {
		let j = 2 * i;
		let k = 2 * i * j;
		let z = z * y * x;
		println!(\"{} {} {}\", j, k, z);
		i += 1;
	};

}";

	let output = "fn main() {
	let x = 10;
	let y = 20;
	let hello = 30;

	let mut i = 0;
	while i < 10 {
		let j = 2 * i;
		let k = 2 * i * j;
		let hello = hello * y * x;
		println!(\"{} {} {}\", j, k, hello);
		i += 1;
	};

}";

	refactor::refactor::rename_variable(&input, &ANALYSIS, "hello", "9");

}
