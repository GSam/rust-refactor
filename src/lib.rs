#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(collections)]
#![feature(rustc_private)]
#![feature(core)]
#![feature(unicode)]
#![feature(path)]
#![feature(env)]
#![feature(io)]
#![feature(catch_panic)]

#[macro_use]
extern crate log;
extern crate getopts;
extern crate rustc;
extern crate rustc_driver;
extern crate rustc_resolve;
extern crate syntax;

extern crate strings;

pub mod refactor;
