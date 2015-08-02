use trans::save::{generated_code, recorder, SaveContext, Data};

use rustc::session::Session;

use rustc::middle::def;
use rustc::middle::ty::{self, Ty};
use syntax::fold::Folder;
use trans::save::span_utils::SpanUtils;

pub struct InlineFolder<'l, 'tcx: 'l> {
    save_ctxt: SaveContext<'l, 'tcx>,
    sess: &'l Session,
    tcx: &'l ty::ctxt<'tcx>,
    analysis: &'l ty::CrateAnalysis,

    span: SpanUtils<'l>,
}

impl <'l, 'tcx> InlineFolder<'l, 'tcx> {
    pub fn new(tcx: &'l ty::ctxt<'tcx>,
               analysis: &'l ty::CrateAnalysis)
               -> InlineFolder<'l, 'tcx> {
            let span_utils = SpanUtils::new(&tcx.sess);
            InlineFolder {
                sess: &tcx.sess,
                tcx: tcx,
                save_ctxt: SaveContext::from_span_utils(tcx, span_utils.clone()),
                analysis: analysis,
                span: span_utils.clone()
            }
        }
}

impl <'l, 'tcx> Folder for InlineFolder<'l, 'tcx> {}
