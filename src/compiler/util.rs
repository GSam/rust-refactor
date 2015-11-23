use std::io::{self, Read};
use std::path::PathBuf;

use rustc::front::map as ast_map;
use rustc::session::config::Input;
use rustc_front::hir as ast;
use syntax::ast::{Ident, NodeId};
use syntax::codemap::Span;
use syntax::owned_slice::OwnedSlice;
use syntax::ptr::P;

use getopts;

// The functions below have been copied from librustc_driver/lib.rs

// Extract output directory and file from matches.
pub fn make_output(matches: &getopts::Matches) -> (Option<PathBuf>, Option<PathBuf>) {
    let odir = matches.opt_str("out-dir").map(PathBuf::from);
    let ofile = matches.opt_str("o").map(PathBuf::from);
    (odir, ofile)
}

// Extract input (string or file and optional path) from matches.
pub fn make_input(free_matches: &[String]) -> Option<(Input, Option<PathBuf>)> {
    if free_matches.len() == 1 {
        let ifile = &free_matches[0][..];
        if ifile == "-" {
            let mut src = String::new();
            io::stdin().read_to_string(&mut src).unwrap();
            Some((Input::Str(src), None))
        } else {
            Some((Input::File(PathBuf::from(ifile)), Some(PathBuf::from(ifile))))
        }
    } else {
        None
    }
}

// The functions below are needed because of HIR lowering
pub fn build_path(span: Span, strs: Vec<Ident> ) -> ast::Path {
    path_all(span, false, strs, Vec::new(), Vec::new(), Vec::new())
}

pub fn build_path_ident(span: Span, id: Ident) -> ast::Path {
    build_path(span, vec!(id))
}

pub fn build_path_global(span: Span, strs: Vec<Ident> ) -> ast::Path {
    path_all(span, true, strs, Vec::new(), Vec::new(), Vec::new())
}

fn path_all(sp: Span,
            global: bool,
            mut idents: Vec<Ident> ,
            lifetimes: Vec<ast::Lifetime>,
            types: Vec<P<ast::Ty>>,
            bindings: Vec<P<ast::TypeBinding>> )
            -> ast::Path {
    let last_identifier = idents.pop().unwrap();
    let mut segments: Vec<ast::PathSegment> = idents.into_iter()
                                                  .map(|ident| {
        ast::PathSegment {
            identifier: ident,
            parameters: ast::PathParameters::none(),
        }
    }).collect();
    segments.push(ast::PathSegment {
        identifier: last_identifier,
        parameters: ast::AngleBracketedParameters(ast::AngleBracketedParameterData {
            lifetimes: lifetimes,
            types: OwnedSlice::from_vec(types),
            bindings: OwnedSlice::from_vec(bindings),
        })
    });
    ast::Path {
        span: sp,
        global: global,
        segments: segments,
    }
}

// Identifies the current lifetimes in scope for finding an unused lifetime.
pub fn lifetimes_in_scope(map: &ast_map::Map,
                      scope_id: NodeId)
                      -> Vec<ast::LifetimeDef> {
    let mut taken = Vec::new();
    let method_id_opt = match map.find(scope_id) {
        Some(node) => match node {
            ast_map::NodeItem(item) => match item.node {
                ast::ItemFn(_, _, _, _, ref gen, _) => {
                    taken.push_all(&gen.lifetimes);
                    None
                },
                _ => None
            },
            ast_map::NodeImplItem(ii) => {
                match ii.node {
                    ast::ImplItemKind::Method(ref sig, _) => {
                        taken.push_all(&sig.generics.lifetimes);
                        Some(ii.id)
                    }
                    //ast::MacImplItem(_) => tcx.sess.bug("unexpanded macro"),
                    _ => None,
                }
            }
            _ => None
        },
        None => None
    };
    if method_id_opt.is_some() {
        let method_id = method_id_opt.unwrap();
        let parent = map.get_parent(method_id);
        match map.find(parent) {
            Some(node) => match node {
                ast_map::NodeItem(item) => match item.node {
                    ast::ItemImpl(_, _, ref gen, _, _, _) => {
                        taken.push_all(&gen.lifetimes);
                    }
                    _ => ()
                },
                _ => ()
            },
            None => ()
        }
    }
    return taken;
}
