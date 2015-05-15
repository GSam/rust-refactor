#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(collections)]
#![feature(rustc_private)]
#![feature(core)]
#![feature(unicode)]
#![feature(path)]
#![feature(env)]
#![feature(io)]

#[macro_use]
extern crate log;
extern crate strings;
extern crate getopts;
extern crate rustc;
extern crate rustc_driver;
extern crate syntax;
extern crate rustc_resolve;

pub mod refactor;
