use trans::save::{generated_code, recorder, SaveContext, Data};

use rustc::session::Session;

use syntax::codemap::{Span, Spanned};
use rustc::middle::def;
use rustc::middle::ty;
use syntax::fold::Folder;
use syntax::ptr::P;
use syntax::ast::*;
use syntax::util::small_vector::SmallVector;
use trans::save::span_utils::SpanUtils;

pub struct InlineFolder<'l, 'tcx: 'l> {
    save_ctxt: SaveContext<'l, 'tcx>,
    sess: &'l Session,
    tcx: &'l ty::ctxt<'tcx>,
    analysis: &'l ty::CrateAnalysis,

    span: SpanUtils<'l>,

    node_to_find: NodeId,
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

            node_to_find: node_to_find
        }
    }

    // TODO: Need to make sure that double decl are not inlined...
    fn noop_fold_decl(&mut self, d: P<Decl>) -> SmallVector<P<Decl>> {
        d.and_then(|Spanned {node, span}| match node {
            DeclLocal(ref l) if l.pat.id == self.node_to_find => SmallVector::zero(),
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

}

impl <'l, 'tcx> Folder for InlineFolder<'l, 'tcx> {
    fn fold_decl(&mut self, d: P<Decl>) -> SmallVector<P<Decl>> {
        self.noop_fold_decl(d)
    }
}

