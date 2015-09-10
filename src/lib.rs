#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(catch_panic)]
#![feature(rustc_private)]
#![feature(result_expect)]
#![feature(slice_splits)]
#![feature(vec_push_all)]

#[macro_use]
extern crate log;

extern crate getopts;
extern crate rustc;
extern crate rustc_driver;
extern crate rustc_front;
extern crate rustc_lint;
extern crate rustc_resolve;
extern crate rustc_trans as trans;
extern crate syntax;

extern crate strings;

pub mod folder;
pub mod lifetime_walker;
pub mod loader;
pub mod rebuilder;
pub mod refactor;
pub mod ast_map;
