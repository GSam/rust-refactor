#![feature(rustc_private)]
#![feature(vec_push_all)]
#![feature(catch_panic)]

extern crate rustc;
extern crate rustc_driver;
extern crate rustc_front;
extern crate rustc_lint;
extern crate rustc_resolve;
extern crate rustc_trans;
extern crate syntax;

extern crate csv;
extern crate getopts;
#[macro_use]
extern crate log;
extern crate strings;

pub mod analysis;
mod compiler;
mod rebuilder;
pub mod refactor;
pub mod util;

pub use analysis::{
    AnalysisData
};

pub use refactor::{
    Response,
    elide_fn_lifetime,
    inline_local,
    rename_function,
    rename_type,
    rename_variable,
    restore_fn_lifetime
};

pub use util::{
    identify_id
};
