
use trans::save::{generated_code, recorder, SaveContext, Data};

use rustc::session::Session;

use rustc::front::map as ast_map;
use rustc_front::fold::Folder;
use rustc_front::fold::{noop_fold_expr, noop_fold_explicit_self_underscore};
use rustc_front::hir::*;
use rustc::middle::def::{self, PathResolution};
use rustc::middle::def_id::{DefId, LOCAL_CRATE};
use rustc::middle::ty;
use rustc_resolve as resolve;
use rustc_resolve::Namespace;
use std::collections::HashMap;
use syntax;
use syntax::ast::{Ident, Name, NodeId};
use syntax::codemap::{DUMMY_SP, Span, Spanned, NO_EXPANSION};
use syntax::ext::{base, expand};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;
use rustc_front::visit::{self, Visitor};
use syntax::util::small_vector::SmallVector;
use trans::save::span_utils::SpanUtils;

use refactor::{build_path, build_path_ident};

pub struct InlineFolder<'l, 'tcx: 'l> {
    save_ctxt: SaveContext<'l, 'tcx>,
    sess: &'l Session,
    tcx: &'l ty::ctxt<'tcx>,
    analysis: &'l ty::CrateAnalysis,
    span: SpanUtils<'l>,
    node_to_find: NodeId,
    pub to_replace: Option<P<Expr>>,
    pub type_node_id: NodeId,
    pub usages: u32,
    pub mutable: bool,
    pub paths: HashMap<(Path, Namespace), def::Def>,
    pub changed_paths: bool,
}

impl <'l, 'tcx> InlineFolder<'l, 'tcx> {
    pub fn new(tcx: &'l ty::ctxt<'tcx>,
               analysis: &'l ty::CrateAnalysis,
               node_to_find: NodeId)
               -> InlineFolder<'l, 'tcx> {
        let span_utils = SpanUtils::new(&tcx.sess);
        InlineFolder {
            sess: &tcx.sess,
            tcx: tcx,
            save_ctxt: SaveContext::from_span_utils(tcx, span_utils.clone()),
            analysis: analysis,
            span: span_utils.clone(),
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
    fn noop_fold_decl(&mut self, d: P<Decl>) -> SmallVector<P<Decl>> {
        d.and_then(|Spanned {node, span}| match node {
            DeclLocal(ref l) if l.pat.id == self.node_to_find => {
                self.to_replace = l.init.clone();
                l.init.clone().unwrap().and_then(
                    |expr|{ visit::walk_expr(self, &expr); }
                );
                if let Some(def_type) = l.ty.as_ref() {
                    self.type_node_id = def_type.id;
                }
                match l.pat.node {
                    PatIdent(ref binding, ref path, ref optpat) => {
                        self.mutable = match *binding {
                            BindByRef(MutMutable) => true,
                            BindByValue(MutMutable) => true,
                            _ => false
                        };
                    },
                    _ => ()
                }
                SmallVector::zero()
            },
            DeclLocal(l) => SmallVector::one(P(Spanned {
                node: DeclLocal(self.fold_local(l)),
                span: self.new_span(span)
            })),
            DeclItem(it) => self.fold_item(it).into_iter().map(|i| P(Spanned {
                node: DeclItem(i),
                span: self.new_span(span)
            })).collect()
        })
    }

    fn process_path(&mut self, id: NodeId, path: &Path, ref_kind: Option<recorder::Row>) -> bool {
        let mut not_generated = path.clone();
        let mut path = path;
        if generated_code(path.span) {
            not_generated.span = Span { lo: path.span.lo, hi: path.span.hi, expn_id: NO_EXPANSION };
            path = &not_generated;
        }

        let path_data = self.save_ctxt.get_path_data(id, &self.front_to_ast(path));
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
                let DefId { krate, node } = vrd.ref_id;
                if krate == LOCAL_CRATE && node  == self.node_to_find {
                    self.usages += 1;
                    return true;
                }
            }
            Data::TypeRefData(ref trd) => {
                /*self.fmt.ref_str(recorder::Row::TypeRef,
                                 path.span,
                                 Some(trd.span),
                                 trd.ref_id,
                                 trd.scope);*/
            }
            Data::MethodCallData(ref mcd) => {
                /*self.fmt.meth_call_str(path.span,
                                       Some(mcd.span),
                                       mcd.ref_id,
                                       mcd.decl_id,
                                       mcd.scope);*/
            }
            Data::FunctionCallData(fcd) => {
                /*self.fmt.fn_call_str(path.span,
                                     Some(fcd.span),
                                     fcd.ref_id,
                                     fcd.scope);*/
            }
            _ => {
                self.sess.span_bug(path.span,
                                   &format!("Unexpected data: {:?}", path_data));
            }
        }

        false
    }

    fn front_to_ast(&self, input: &Path) -> syntax::ast::Path {
        let krate = self.tcx.map.krate();
        let ps = &self.sess.parse_sess;
        let mut tmp = vec![];
        let mut cx = base::ExtCtxt::new(ps, Vec::new(),//krate.config.clone(),
                                        expand::ExpansionConfig::default("".to_string()),
                                        &mut tmp);

        cx.path_ident(input.span.clone(), input.segments[0].identifier)
    }

//pub fn noop_fold_expr<T: Folder>(Expr {id, node, span}: Expr, folder: &mut T) -> Expr {}

}

impl <'l, 'tcx> Folder for InlineFolder<'l, 'tcx> {
    fn fold_decl(&mut self, d: P<Decl>) -> SmallVector<P<Decl>> {
        self.noop_fold_decl(d)
    }

    fn fold_expr(&mut self, e: P<Expr>) -> P<Expr> {
        debug!("{:?}", e);
        e.map(|e| {
            match e.node {
                ExprPath(ref q, ref path) => {
                    if self.process_path(e.id, path, None) {
                        let node_to_find = e.id;
                        let s_ctx = path.segments[0].clone().identifier.ctxt;
                        let mut resolver = resolve::create_resolver(&self.sess, &self.tcx.map,
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
                                let krate = self.tcx.map.krate();
                                let ps = &self.sess.parse_sess;

                                t.ctxt = s_ctx;
                                let path = build_path(DUMMY_SP, vec![t]);
                                resolution = resolver.resolve_path(self.node_to_find, &path, 0, Namespace::ValueNS, true);
                            } else {
                                resolution = resolver.resolve_path(self.node_to_find, &path.0, 0, path.1, true);
                            }
                            if let Some(resolution) = resolution {
                                let PathResolution {base_def, ..} = resolution;
                                debug!("BASEDEF {:?}", base_def);
                                if base_def != *def {
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
            noop_fold_expr(e, self)
        })
    }
}

impl<'l, 'tcx, 'v> Visitor<'v> for InlineFolder<'l, 'tcx> {
    fn visit_expr(&mut self, ex: &Expr) {
        let node_to_find = self.node_to_find;
        let mut resolver = resolve::create_resolver(&self.sess, &self.tcx.map,
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
            ExprPath(_, ref path) => {
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

pub struct LifetimeFolder {
    pub has_bounds: bool,
    pub expl_self: Name,
}

impl Folder for LifetimeFolder {
    fn fold_opt_lifetime(&mut self, o_lt: Option<Lifetime>) -> Option<Lifetime> {
        None
    }

    fn fold_generics(&mut self, Generics {ty_params, lifetimes, where_clause}: Generics) -> Generics {
        for lifetime in lifetimes.iter() {
            if lifetime.bounds.len() > 0 {
                self.has_bounds = true;
            }
        }
        Generics {
            ty_params: self.fold_ty_params(ty_params),
            lifetimes: Vec::new(),
            where_clause: self.fold_where_clause(where_clause),
        }
    }

    fn fold_explicit_self_underscore(&mut self, es: ExplicitSelf_) -> ExplicitSelf_ {
        match es {
            SelfRegion(Some(lifetime), m, ident) => {
                self.expl_self = lifetime.name;
            }
            _ => ()
        }

        noop_fold_explicit_self_underscore(es, self)
    }
}
