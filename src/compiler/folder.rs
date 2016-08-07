use rustc_front::hir;
use rustc_front::fold::{self, Folder};
use rustc_front::intravisit::{self as visit, Visitor};
use rustc_front::lowering::LoweringContext;
use rustc_trans::save::{generated_code, recorder, SaveContext, Data};
use rustc_trans::save::span_utils::SpanUtils;
use rustc::front::map as ast_map;
use rustc::metadata::cstore::LOCAL_CRATE;
use rustc::middle::def::{self, PathResolution};
use rustc::middle::def_id::DefId;
use rustc::middle::ty;
use rustc_resolve as resolve;
use rustc_resolve::Namespace;
use std::collections::HashMap;
use syntax::ast::{self, NodeId};
use syntax::codemap::{DUMMY_SP, Span, Spanned, NO_EXPANSION};
use syntax::ext::{base, expand};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;

use super::util::build_path;

// Fold local variable and replace usages with initializing expression.
pub struct InlineFolder<'l, 'tcx: 'l, 'lcx:'l> {
    //save_ctxt: SaveContext<'l, 'tcx>,
    tcx: &'l ty::ctxt<'tcx>,
    lcx: &'l LoweringContext<'lcx>,
    span: SpanUtils<'l>,
    node_to_find: NodeId,
    pub to_replace: Option<P<hir::Expr>>,
    pub type_node_id: NodeId,
    pub usages: u32,
    pub mutable: bool,
    pub paths: HashMap<(hir::Path, Namespace), def::Def>,
    pub changed_paths: bool,
}

impl <'l, 'tcx, 'lcx> InlineFolder<'l, 'tcx, 'lcx> {
    pub fn new(tcx: &'l ty::ctxt<'tcx>,
               lcx: &'l LoweringContext<'lcx>,
               node_to_find: NodeId)
               -> InlineFolder<'l, 'tcx, 'lcx> {
        InlineFolder {
            tcx: tcx,
            lcx: lcx,
            //lcx: lcx,
            //save_ctxt: SaveContext::from_span_utils(tcx, span_utils.clone()),
            span: SpanUtils::new(&tcx.sess),
            node_to_find: node_to_find,
            to_replace: None,
            type_node_id: 0,
            usages: 0,
            mutable: false,
            paths: HashMap::new(),
            changed_paths: false,
        }
    }

    // TODO: Need to make sure that double decl are not inlined...
    fn noop_fold_decl(&mut self, d: P<hir::Decl>) -> P<hir::Decl> {
        d.and_then(|Spanned {node, span}| match node {
            hir::DeclLocal(ref l) if l.pat.id == self.node_to_find => {
                self.to_replace = l.init.clone();
                l.init.clone().unwrap().and_then(
                    |expr| visit::walk_expr(self, &expr)
                );

                if let Some(def_type) = l.ty.as_ref() {
                    self.type_node_id = def_type.id;
                }

                if let hir::PatIdent(ref binding, _, _) = l.pat.node {
                    self.mutable = match *binding {
                        hir::BindByRef(hir::MutMutable) => true,
                        hir::BindByValue(hir::MutMutable) => true,
                        _ => false
                    };
                }

                //SmallVector::zero()
                // FIXME: return something meaningful
                panic!("folder.rs")
            },
            hir::DeclLocal(l) => P(Spanned {
                node: hir::DeclLocal(self.fold_local(l)),
                span: self.new_span(span)
            }),
            hir::DeclItem(it) => P(Spanned {
                node: hir::DeclItem(self.fold_item_id(it)),
                span: self.new_span(span)
            })
        })
    }

    fn process_path(&mut self, id: NodeId, path: &hir::Path, _: Option<recorder::Row>) -> bool {
        let mut not_generated = path.clone();
        let mut path = path;
        if generated_code(path.span) {
            not_generated.span = Span { lo: path.span.lo, hi: path.span.hi, expn_id: NO_EXPANSION };
            path = &not_generated;
        }

        // FIXME: does this work properly? Or should we save the context between calls?
        let save_ctxt = SaveContext::from_span_utils(self.tcx, self.lcx, self.span.clone());
        let path_data = save_ctxt.get_path_data(id, &self.front_to_ast(path));
        let path_data = match path_data {
            Some(pd) => pd,
            None => {
                self.tcx.sess.span_bug(path.span,
                                       &format!("Unexpected def kind while looking \
                                                 up path in `{}`",
                                                self.span.snippet(path.span)))
            }
        };
        match path_data {
            Data::VariableRefData(ref vrd) => {
                /*self.fmt.ref_str(ref_kind.unwrap_or(recorder::Row::VarRef),
                                                    path.span,
                                                    Some(vrd.span),
                                                    vrd.ref_id,
                                                    vrd.scope);*/
                let DefId { krate, index } = vrd.ref_id;
                if krate == LOCAL_CRATE && index.as_u32()  == self.node_to_find {
                    self.usages += 1;
                    return true;
                }
            }
            Data::TypeRefData(_) => {
                /*self.fmt.ref_str(recorder::Row::TypeRef,
                                 path.span,
                                 Some(trd.span),
                                 trd.ref_id,
                                 trd.scope);*/
            }
            Data::MethodCallData(_) => {
                /*self.fmt.meth_call_str(path.span,
                                       Some(mcd.span),
                                       mcd.ref_id,
                                       mcd.decl_id,
                                       mcd.scope);*/
            }
            Data::FunctionCallData(_) => {
                /*self.fmt.fn_call_str(path.span,
                                     Some(fcd.span),
                                     fcd.ref_id,
                                     fcd.scope);*/
            }
            _ => {
                self.tcx.sess.span_bug(path.span,
                                   &format!("Unexpected data: {:?}", path_data));
            }
        }

        false
    }

    fn front_to_ast(&self, input: &hir::Path) -> ast::Path {
        let ps = &self.tcx.sess.parse_sess;
        let mut tmp = vec![];
        let cx = base::ExtCtxt::new(ps, Vec::new(),
                                        expand::ExpansionConfig::default("".to_string()),
                                        &mut tmp);

        cx.path_ident(input.span.clone(), input.segments[0].identifier)
    }
}

impl <'l, 'tcx, 'lcx> Folder for InlineFolder<'l, 'tcx, 'lcx> {
    fn fold_decl(&mut self, d: P<hir::Decl>) -> P<hir::Decl> {
        self.noop_fold_decl(d)
    }

    fn fold_expr(&mut self, e: P<hir::Expr>) -> P<hir::Expr> {
        debug!("{:?}", e);
        e.map(|e| {
            match e.node {
                hir::ExprPath(_, ref path) => {
                    if self.process_path(e.id, path, None) {
                        let node_to_find = e.id;
                        let s_ctx = path.segments[0].clone().identifier.ctxt;
                        let mut resolver = resolve::create_resolver(&self.tcx.sess, &self.tcx.map,
                                                                    &self.tcx.map.krate(),
                                                                    resolve::MakeGlobMap::No,
                            Some(Box::new(move |node: ast_map::Node, resolved: &mut bool| {
                                if *resolved {
                                    return true;
                                }
                                match node {
                                    ast_map::NodeExpr(expr) => {
                                        if expr.id == node_to_find {
                                            *resolved = true;
                                            return true;
                                        }
                                    },
                                    _ => ()
                                }
                                false
                            }
                        )));
                        // Run the resolver to get the defid
                        debug!("DID RESOLVE");
                        visit::walk_crate(&mut resolver, &self.tcx.map.krate());
                        debug!("DID RESOLVE");
                        for (path, def) in self.paths.iter() {
                            let mut resolution = None;
                            // Syntax contexts prevent resolution at different places
                            // Fix for the current simple variable case
                            if path.1 == Namespace::ValueNS && path.0.segments.len() == 1 {
                                let mut t = path.0.segments[0].clone().identifier;

                                t.ctxt = s_ctx;
                                let path = build_path(DUMMY_SP, vec![t]);
                                //let path = lowering::lower_path(&path);
                                resolution = resolver.resolve_path(self.node_to_find, &path, 0, Namespace::ValueNS, true);
                            } else {
                                resolution = resolver.resolve_path(self.node_to_find, &path.0, 0, path.1, true);
                            }
                            if let Some(res) = resolution {
                                debug!("BASEDEF {:?}", res.base_def);
                                if res.base_def != *def {
                                    debug!("OH DEAR, DEF IS NOW DIFFERENT");
                                    self.changed_paths = true;
                                }
                            } else {
                                debug!("OH DEAR, NO DEF PRESENT");
                                self.changed_paths = true;
                            }
                        }
                        let next = self.to_replace.clone();
                        if let Some(replace) = next {
                            return (*replace).clone()
                        }
                    }
                    //visit::walk_expr(self, ex);
                },
                _ => {}

            }
            fold::noop_fold_expr(e, self)
        })
    }
}

impl<'l, 'tcx, 'lcx, 'v> Visitor<'v> for InlineFolder<'l, 'tcx, 'lcx> {
    fn visit_expr(&mut self, ex: &hir::Expr) {
        let node_to_find = self.node_to_find;
        let mut resolver = resolve::create_resolver(&self.tcx.sess, &self.tcx.map,
                                                    &self.tcx.map.krate(),
                                                    resolve::MakeGlobMap::No,
            Some(Box::new(move |node: ast_map::Node, resolved: &mut bool| {
                if *resolved {
                    return true;
                }
                match node {
                    ast_map::NodeLocal(pat) => {
                        if pat.id == node_to_find {
                            *resolved = true;
                            return true;
                        }
                    },
                    _ => ()
                }
                false
            }
        )));
        match ex.node {
            hir::ExprPath(_, ref path) => {
                //self.process_path(ex.id, path, None);
                visit::walk_crate(&mut resolver, &self.tcx.map.krate());
                let resolution = resolver.resolve_path(self.node_to_find, &path,
                                                       0, Namespace::ValueNS,
                                                       true);
                if let Some(resolution) = resolution {
                    let PathResolution {base_def, ..} = resolution;
                    debug!("{:?}", base_def);
                    self.paths.insert((path.clone(), Namespace::ValueNS), base_def);
                }

                let resolution = resolver.resolve_path(self.node_to_find, &path,
                                                       0, resolve::Namespace::TypeNS,
                                                       true);
                if let Some(resolution) = resolution {
                    let PathResolution {base_def, ..} = resolution;
                    debug!("{:?}", base_def);
                    self.paths.insert((path.clone(), Namespace::TypeNS), base_def);
                }

                visit::walk_expr(self, ex);
            },
            _ => visit::walk_expr(self, ex)
        }
    }
}

// Fold lifetimes so they are all removed.
pub struct LifetimeFolder {
    pub has_bounds: bool,
    pub expl_self: ast::Name,
}

impl Folder for LifetimeFolder {
    fn fold_opt_lifetime(&mut self, _: Option<hir::Lifetime>) -> Option<hir::Lifetime> {
        None
    }

    fn fold_generics(&mut self, hir::Generics {ty_params, lifetimes, where_clause}: hir::Generics) -> hir::Generics {
        for lifetime in lifetimes.iter() {
            if lifetime.bounds.len() > 0 {
                self.has_bounds = true;
            }
        }
        hir::Generics {
            ty_params: self.fold_ty_params(ty_params),
            lifetimes: Vec::new(),
            where_clause: self.fold_where_clause(where_clause),
        }
    }

    fn fold_explicit_self_underscore(&mut self, es: hir::ExplicitSelf_) -> hir::ExplicitSelf_ {
        match es {
            hir::SelfRegion(Some(lifetime), _, _) => {
                self.expl_self = lifetime.name;
            }
            _ => ()
        }

        fold::noop_fold_explicit_self_underscore(es, self)
    }
}
