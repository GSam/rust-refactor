
use syntax::ast::*;
use syntax::visit::{self, Visitor};

pub struct LifetimeWalker {
    pub anon: u32,
    pub total: u32,
    pub expl_self: u32,
}

impl LifetimeWalker {
    pub fn new() -> LifetimeWalker {
        LifetimeWalker {
            anon: 0,
            total: 0,
            expl_self: 0,
        }
    }
}

impl<'v> Visitor<'v> for LifetimeWalker {
    fn visit_expr(&mut self, ex: &Expr) {

    }
}
