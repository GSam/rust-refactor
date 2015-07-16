#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(catch_panic)]
#![feature(collections)]
#![feature(core)]
#![feature(env)]
#![feature(io)]
#![feature(path)]
#![feature(rustc_private)]
#![feature(result_expect)]
#![feature(unicode)]

#[macro_use]
extern crate log;
extern crate getopts;
extern crate rustc;
extern crate rustc_driver;
extern crate rustc_resolve;
extern crate syntax;

extern crate strings;

pub mod refactor;
pub mod loader;
