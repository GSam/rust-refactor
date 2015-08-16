
use trans::save::{generated_code, recorder, SaveContext, Data};

use rustc::session::Session;

use rustc::ast_map;
use rustc::middle::def::{self, PathResolution};
use rustc::middle::ty;
use rustc_resolve as resolve;
use std::collections::HashMap;
use syntax::ast::*;
use syntax::codemap::{Span, Spanned, NO_EXPANSION};
use syntax::fold::Folder;
use syntax::fold::noop_fold_expr;
use syntax::ptr::P;
use syntax::visit::{self, Visitor};
use syntax::util::small_vector::SmallVector;
use trans::save::span_utils::SpanUtils;

pub struct InlineFolder<'l, 'tcx: 'l> {
    save_ctxt: SaveContext<'l, 'tcx>,
    sess: &'l Session,
    tcx: &'l ty::ctxt<'tcx>,
    analysis: &'l ty::CrateAnalysis,
    span: SpanUtils<'l>,
    node_to_find: NodeId,
    pub to_replace: Option<P<Expr>>,
    pub usages: u32,
    pub mutable: bool,
    pub paths: HashMap<Path, def::Def>,
    pub base_def: Option<def::Def>,
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
            usages: 0,
            mutable: false,
            paths: HashMap::new(),
            base_def: None,
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

        let path_data = self.save_ctxt.get_path_data(id, path);
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
                        // Run the resolver to get the defid
                        visit::walk_crate(&mut resolver, &self.tcx.map.krate());
                        let PathResolution {base_def, ..} = resolver.resolve_path(self.node_to_find, &path, 0, resolve::Namespace::ValueNS, true).unwrap();
                        debug!("BASEDEF {:?}", base_def);
                        self.base_def = Some(base_def);
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
                self.process_path(ex.id, path, None);
                visit::walk_crate(&mut resolver, &self.tcx.map.krate());
                let resolution = resolver.resolve_path(self.node_to_find, &path,
                                                       0, resolve::Namespace::ValueNS,
                                                       true);
                if let Some(resolution) = resolution {
                    let PathResolution {base_def, ..} = resolution;
                    self.paths.insert(path.clone(), base_def);
                }

                let resolution = resolver.resolve_path(self.node_to_find, &path,
                                                       0, resolve::Namespace::TypeNS,
                                                       true);
                if let Some(resolution) = resolution {
                    let PathResolution {base_def, ..} = resolution;
                    self.paths.insert(path.clone(), base_def);
                }

                visit::walk_expr(self, ex);
            },
            _ => visit::walk_expr(self, ex)
        }
    }
}

