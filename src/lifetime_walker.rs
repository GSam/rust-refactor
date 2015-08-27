
use syntax::ast::*;
use syntax::codemap::Span;
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

    fn visit_opt_lifetime_ref(&mut self,
                              _span: Span,
                              opt_lifetime: &'v Option<Lifetime>) {
        self.total += 1;
        match *opt_lifetime {
            Some(ref l) => self.visit_lifetime_ref(l),
            None => self.anon += 1
        }
    }

    fn visit_explicit_self(&mut self, es: &'v ExplicitSelf) {
        self.expl_self = self.anon;
        visit::walk_explicit_self(self, es);
    }
}
