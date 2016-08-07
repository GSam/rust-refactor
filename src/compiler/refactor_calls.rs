use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;

use rustc::front::map::{self as ast_map, Node};
use rustc::session::Session;
use rustc::session::config::{self, Input};
use rustc::metadata::creader::LocalCrateReader;
use rustc_driver::{CompilerCalls, Compilation, driver, RustcDefaultCalls};
use rustc_front::fold::Folder;
use rustc_front::hir;
use rustc_front::hir::Item_::{ItemImpl, ItemStruct};
use rustc_front::intravisit as visit;
use rustc_front::print::pprust::{self, State};
use rustc_resolve as resolve;
use rustc::middle::def_id::{DefId, DefIndex};
use rustc::middle::lang_items;
use rustc::middle::infer::region_inference::SameRegions;
use rustc::middle::ty::BoundRegion::*;
use syntax::{attr, diagnostics};
use syntax::ast::{self, Name, NodeId};
use syntax::codemap::{DUMMY_SP, Pos, Spanned};
use syntax::ext::build::AstBuilder;
use syntax::ext::mtwt;
use syntax::parse::token;
use syntax::print::pp::eof;
use syntax::ptr::P;

use getopts;

use super::folder::{InlineFolder, LifetimeFolder};
use super::lifetime_walker::LifetimeWalker;
use super::util::{build_path, lifetimes_in_scope};
use rebuilder::{Rebuilder, LifeGiver};
use refactor::RefactorType;

fn default_defid() -> DefId {
    DefId {
        krate: 0,
        index: DefIndex::new(0)
    }
}

pub struct RefactorCalls {
    r_type: RefactorType,
    new_name: String,
    node_to_find: NodeId,
    input: Option<Vec<(String, String)>>,
    working_file: Option<String>,
    is_full: bool,
}

impl RefactorCalls {
    pub fn new(t: RefactorType,
           new_name: String,
           node: NodeId,
           new_file: Option<Vec<(String, String)>>,
           working_file: Option<String>,
           is_full: bool)
           -> RefactorCalls
    {
        RefactorCalls {
            r_type: t,
            new_name: new_name,
            node_to_find: node,
            input: new_file,
            working_file: working_file,
            is_full: is_full,
        }
    }
}

impl<'a> CompilerCalls<'a> for RefactorCalls {
    fn late_callback(&mut self,
                     m: &getopts::Matches,
                     s: &Session,
                     i: &Input,
                     odir: &Option<PathBuf>,
                     ofile: &Option<PathBuf>)
                     -> Compilation {
        RustcDefaultCalls.late_callback(m, s, i, odir, ofile);
        
        // FIXME: We ignore the result of late_callback... Why?
        Compilation::Continue
    }
    
    fn no_input(&mut self,
                m: &getopts::Matches,
                o: &config::Options,
                odir: &Option<PathBuf>,
                ofile: &Option<PathBuf>,
                r: &diagnostics::registry::Registry)
                -> Option<(Input, Option<PathBuf>)> {
        RustcDefaultCalls.no_input(m, o, odir, ofile, r);
        
        // FIXME: This is not optimal error handling.
        panic!("No input supplied");
    }

    fn build_controller(&mut self, _: &Session) -> driver::CompileController<'a> {
        let r_type = self.r_type;
        let is_full = self.is_full;
        let node_to_find = self.node_to_find;
        let input = self.working_file.clone().unwrap_or_default();

        // FIXME: I migrated a lot of stuff to use the HIR just to appease the compiler... Is this ok?
        let mut control = driver::CompileController::basic();
        if is_full {
            control.after_analysis.stop = Compilation::Stop;
            control.after_analysis.callback = Box::new(move |state| {
                let tcx = state.tcx.unwrap();
                let lcx = state.lcx.unwrap();
                
                // FIXME: doest his work properly?
                // Old code:
                // let mut new_for = Forest::new(krate.clone());
                // let ast_map_original = map_crate(&mut new_for);
                // New code:
                let ast_map_original = state.ast_map.unwrap().clone();
                let ast_map = &tcx.map;
                if r_type == RefactorType::InlineLocal {

                    debug!("{:?}", ast_map.get(ast_map.get_parent(node_to_find)));
                    debug!("{:?}", ast_map.get(node_to_find));

                    match ast_map.get(node_to_find) {
                        Node::NodeLocal(_) => { },
                        _ => { panic!(); }
                    }

                    let mut parent = None;
                    let mut other = None;
                    match ast_map_original.get(ast_map.get_parent(node_to_find)) {
                        Node::NodeItem(item) => {
                            parent = Some(P((*item).clone()));
                            other = Some((*item).clone());
                            debug!("{:?}", pprust::item_to_string(item));
                        },
                        _ => {}
                    }

                    let src;
                    src = state.session.codemap().get_filemap(&input[..])
                                                 .src
                                                 .as_ref()
                                                 .unwrap()
                                                 .as_bytes()
                                                 .to_vec();
                    // need to distinguish internal errors

                    let mut rdr = &src[..];
                    let mut out = Vec::new();
                    let ann = pprust::NoAnn;
                    {
                        let out_borrow: &mut Write = &mut out;
                        // FIXME: the last argument was added just to make this compile... Should it be changed to something else?
                        let mut pp_state = State::new_from_input(state.session.codemap(), state.session.diagnostic(), input.clone(), &mut rdr, Box::new(out_borrow), &ann, true, None);

                        if let Some(other) = other {
                            let _ = pp_state.print_item(&other);
                            //debug!("{:?}", v);
                            //pp_state.print_mod(&krate.module, &krate.attrs);
                        }
                        let _ = eof(&mut pp_state.s);
                    }
                    let _ = out.flush();

                    // Build save walker
                    let src2;
                    src2 = state.session.codemap().get_filemap(&input[..])
                                                 .src
                                                 .as_ref()
                                                 .unwrap()
                                                 .as_bytes()
                                                 .to_vec();
                    let mut rdr2 = &src2[..];

                    if let Some(par) = parent {
                        let outer_span = par.span;
                        //let mut visitor = DumpCsvVisitor::new(tcx, anal, output_file);
                        let mut folder = InlineFolder::new(tcx, lcx, node_to_find);
                        debug!("{:?}", folder.fold_item((*par).clone()));
                        debug!("Number of usages: {}", folder.usages);

                        // First we want to ignore destructuring locals, this has issues with lifetimes + type info.
                        // It should also actually BE a local, not just some variable-like item.
                        // TODO
                        // What about sensible destructors, operator overloading?
                        // BUT if we get a 'consider using a let binding error', then, we cannot inline.
                        if folder.usages <= 1 {
                            // This is generally OK, unless the expression contains an impure function/constructor
                            // e.g. let a = <changes external state>
                            //      <change external state some other way>
                            // Now if we move the first change after the second change, behaviour might change.
                            // If doesn't matter here if we have copy, move, borrow etc.
                            //
                            // Due to uniqueness constraints in Rust, if there is just a single usage, there really
                            // is just a single usage without any aliases.

                            // If any variables composing the initializer were redeclared in the meantime, return
                            if folder.changed_paths {
                                return;
                            }


                        } else {
                            // Otherwise, multiple references:

                            // Mutable case:
                            // If the variable is mutable, inlining is a bad idea!!!
                            // e.g. let mut a = 2;
                            // a = 3; // Now the expression is made the lvalue, but this holds no meaning
                            // Same again with *&mut a modifying the internal value.
                            let used_mutables = tcx.used_mut_nodes.borrow();
                            // CAVEAT:
                            // If the mutable was never used, then it should be considered mutable.
                            if folder.mutable && used_mutables.contains(&node_to_find) {
                                debug!("IS MUTABLE");
                                return;
                            }
                            // CAVEAT:
                            // If there is a refcell, or interior mutability, then it really is mutable.
                            let ty_cache = tcx.ast_ty_to_ty_cache.borrow();
                            let interior_unsafe = 0b0000_0000__0000_0000__0010;
                            if let Some(node_ctx) = ty_cache.get(&folder.type_node_id) {
                                debug!("BITS: {:?}", node_ctx.type_contents(tcx).bits);
                                if node_ctx.type_contents(tcx).bits & interior_unsafe != 0 {
                                    debug!("IS MUTABLE (REFCELL)");
                                    return;
                                }
                            }

                            // If the variable is a direct alias, then it might be alright.
                            // In this case, movements or borrows are irrelevant.
                            // e.g. let a = 2;
                            //      let b = a; // copy, but memory shouldn't mutate underneath
                            //   or let a = &2;
                            //      let b = a; // this duplicates the reference
                            //   or let a = &mut 2;
                            //      let b = a; // this moves &mut into b
                            //   or let a = vec![0];
                            //      let b = a; // this moves a into b
                            // Whether or not a is inlined, it must follow the normal lifetime rules.
                            // Whatever a refers to must exists for the right scopes.
                            // However, you must check no one redeclares a in the meantime!
                            if let Some(ref to_replace) = folder.to_replace {
                                match (**to_replace).node {
                                    hir::ExprPath(..) => {
                                        // Alias case:
                                    },
                                    _ => {
                                    }
                                }
                            }


                            debug!("IS NOT MUTABLE");
                            // Immutable case:
                            // Check which paths compose the initializer and ensure they resolve
                            // to the same item at the new call site.
                            // e.g. b = 2;
                            // let a = b + c
                            // let b = 3;
                            // println!("{}", a);

                            // If any variables composing the initializer were redeclared in the meantime, return
                            if folder.changed_paths {
                                return;
                            }

                            // If any variables composing the initializer mutated in the meantime, return
                            // TODO

                        }

                        let mut out = Vec::new();
                        {
                            let out_borrow: &mut Write = &mut out;
                            // FIXME: can the last argument be None?
                            let mut pp_state = State::new_from_input(state.session.codemap(), state.session.diagnostic(), input.clone(), &mut rdr2, Box::new(out_borrow), &ann, true, None);

                            let _ = pp_state.print_item(&folder.fold_item((*par).clone()));
                            //debug!("{:?}", v);
                            //pp_state.print_mod(&krate.module, &krate.attrs);
                            //pp_state.print_remaining_comments();
                            let _ = eof(&mut pp_state.s);
                        }
                        let _ = out.flush();
                        debug!("{:?}", out);
                        let hi_pos = state.session.codemap().lookup_byte_offset(outer_span.hi).pos.to_usize();
                        let lo_pos = state.session.codemap().lookup_byte_offset(outer_span.lo).pos.to_usize();
                        panic!((lo_pos, hi_pos, String::from_utf8(out).ok().expect("Pretty printer didn't output UTF-8"), 0));
                        //pprust::item_to_string(folder.fold_item(par).get(0))
                        //visit::walk_crate(&mut visitor, &krate);
                    }
                } else if r_type == RefactorType::ReifyLifetime || r_type == RefactorType::ElideLifetime {
                    debug!("{:?}", ast_map.get(node_to_find));

                    let taken = lifetimes_in_scope(&tcx.map, node_to_find);
                    let life_giver = LifeGiver::with_taken(&taken[..]);
                    let node_inner = match ast_map_original.find(node_to_find) {
                        Some(ref node) => match *node {
                            Node::NodeItem(ref item) => {
                                match item.node {
                                    hir::ItemFn(ref fn_decl, unsafety, constness, _, ref gen, ref body) => {
                                        Some((fn_decl, gen, unsafety, constness,
                                              item.name, None, item.span, body.span))
                                    },
                                    _ => None
                                }
                            }
                            Node::NodeImplItem(item) => {
                                match item.node {
                                    hir::ImplItemKind::Method(ref sig, ref body) => {
                                        Some((&sig.decl,
                                              &sig.generics,
                                              sig.unsafety,
                                              sig.constness,
                                              item.name,
                                              Some(&sig.explicit_self.node),
                                              item.span, body.span))
                                    }
                                    //ast::MacImplItem(_) => self.tcx.sess.bug("unexpanded macro"),
                                    _ => None,
                                }
                            },
                            Node::NodeTraitItem(item) => {
                                match item.node {
                                    hir::MethodTraitItem(ref sig, Some(ref body)) => {
                                        Some((&sig.decl,
                                              &sig.generics,
                                              sig.unsafety,
                                              sig.constness,
                                              item.name,
                                              Some(&sig.explicit_self.node),
                                              item.span, body.span))
                                    }
                                    _ => None
                                }
                            }
                            _ => None
                        },
                        None => None
                    };
                    let mut a = Vec::new();
                    let (fn_decl, generics, unsafety, constness, ident, expl_self, span, body_span)
                                                = node_inner.expect("expect item fn");

                    let mut folder = LifetimeFolder{ has_bounds: false, expl_self: Name(0) };
                    let elided_fn_decl = folder.fold_fn_decl(fn_decl.clone());
                    let elided_expl_self_tmp;
                    let mut elided_expl_self = None;

                    // Count input lifetimes and count output lifetimes.
                    let mut in_walker = LifetimeWalker::new();
                    let mut out_walker = LifetimeWalker::new();

                    if let Some(expl_self) = expl_self {
                        visit::walk_explicit_self(&mut in_walker, &Spanned {node: expl_self.clone(), span: DUMMY_SP});
                        elided_expl_self_tmp = folder.fold_explicit_self(Spanned {node: expl_self.clone(), span: DUMMY_SP});
                        elided_expl_self = Some(&elided_expl_self_tmp.node);
                    }

                    for argument in fn_decl.inputs.iter() {
                        debug!("FN DECL: {:?}", argument);
                        visit::walk_ty(&mut in_walker, &*argument.ty);
                    }

                    visit::walk_fn_ret_ty(&mut out_walker, &fn_decl.output);

                    if r_type == RefactorType::ElideLifetime {
                        let elided_generics = folder.fold_generics(generics.clone());
                        let mut parameterized = HashSet::new();
                        for lifetimes in generics.lifetimes.iter() {
                            parameterized.insert(lifetimes.lifetime.name);
                        }

                        // Can't elide if returning multiple lifetimes
                        if out_walker.names.len() > 1 {
                            return;
                        }

                        // Don't elide if return doesn't appear in generics (trait lifetime?)
                        let intersect: HashSet<_> = out_walker.names.intersection(&parameterized).cloned().collect();
                        if out_walker.names.len() > 0 && intersect.len() == 0 {
                            return;
                        }

                        // Make sure that each input lifetime is never used more than once
                        if in_walker.names.len() as u32 + in_walker.anon != in_walker.total {
                            return;
                        }

                        // If you have a return, either it has the same name as the only input, or that of self
                        let intersect: HashSet<_> = out_walker.names.intersection(&in_walker.names).cloned().collect();
                        if out_walker.names.len() > 0 && !out_walker.names.contains(&folder.expl_self)
                                           && (in_walker.names.len() > 1 || intersect.len() == 0) {
                            return;
                        }

                        // Make sure that input lifetimes are all parameterized
                        // TODO delete only unparameterized?
                        if !in_walker.names.is_subset(&parameterized) {
                            return;
                        }

                        // TODO move has_bounds out of the folder
                        if folder.has_bounds {
                            return;
                        }

                        let mut answer = pprust::fun_to_string(&elided_fn_decl, unsafety, constness, ident, elided_expl_self, &elided_generics);

                        // Add some likely spacing
                        answer.push_str(" ");

                        let hi_pos = state.session.codemap().lookup_byte_offset(body_span.lo).pos.to_usize();
                        let lo_pos = state.session.codemap().lookup_byte_offset(span.lo).pos.to_usize();
                        panic!((lo_pos, hi_pos, answer, 0));
                    }

                    // Count anonymous and count total.
                    // CASE 1: fn <'a> (x: &'a) -> &out
                    // CASE 2: fn (x: &a) -> &out
                    // If there is exactly 1 input lifetime, then reuse that lifetime for output (possibly multiple).
                    if in_walker.total == 1 {
                        let mut regions = Vec::new();
                        if in_walker.anon == 0 {
                            // CASE 1
                            regions.push(BrNamed(default_defid(), generics.lifetimes.get(0).unwrap().lifetime.name));
                            for x in 0..out_walker.anon {
                                regions.push(BrAnon(x));
                            }
                        } else {
                            // CASE 2
                            regions.push(BrAnon(0));
                            for x in 0..out_walker.anon {
                                regions.push(BrAnon(x+1));
                            }
                        }
                        a.push(SameRegions{scope_id: 0, regions: regions});

                    } else if in_walker.total > 1 {
                        // If there is more than one lifetime, then:
                        // fn <'a, 'b> (x: &'a, y: &'b, z: &) -> &'a out
                        // We can make concrete the anonymous input lifetimes but not the output.
                        for x in 0..in_walker.anon {
                            a.push(SameRegions{scope_id: 0, regions: vec![BrAnon(x)]});
                        }

                        // Unless, there is a self lifetime, then:
                        // fn <'a, 'b> (self: &'a, ...) -> &out
                        // We can make concrete the output lifetime as well (which may be multiple).
                        if let Some(expl_self) = expl_self {
                            match *expl_self {
                                hir::SelfRegion(ref life, _, _) => {
                                    if life.is_some() {
                                        // self has a named lifetime
                                        let mut regions = Vec::new();
                                        regions.push(BrNamed(default_defid(), life.unwrap().name));
                                        for x in 0..out_walker.anon {
                                            regions.push(BrAnon(in_walker.anon + x));
                                        }
                                        a.push(SameRegions{scope_id: 0, regions: regions});
                                    } else {
                                        // self is anonymous
                                        // TODO remove expl_self
                                        let mut regions = &mut a.get_mut(in_walker.expl_self as usize).expect("Missing expl self").regions;
                                        for x in 0..out_walker.anon {
                                            regions.push(BrAnon(in_walker.anon + x));
                                        }
                                    }
                                }
                                _ => ()
                            }
                        }

                    }

                    let rebuilder = Rebuilder::new(tcx, &*fn_decl, expl_self,
                                                   generics, &a/*same_regions*/, &life_giver);
                    let (fn_decl, expl_self, generics) = rebuilder.rebuild();
                    //self.give_expl_lifetime_param(&fn_decl, unsafety, constness, ident,
                    //                              expl_self.as_ref(), &generics, span);
                    debug!("{}", pprust::fun_to_string(&fn_decl, unsafety, constness, ident, expl_self.as_ref(), &generics));
                    println!("{}", pprust::fun_to_string(&fn_decl, unsafety, constness, ident, expl_self.as_ref(), &generics));
                    //debug!("{:?}", tcx.region_maps);
                    debug!("{:?}", tcx.named_region_map);
                    //debug!("{:?}", tcx.free_region_maps.borrow());
                    let mut answer = pprust::fun_to_string(&fn_decl, unsafety, constness, ident, expl_self.as_ref(), &generics);

                    // Add some likely spacing
                    answer.push_str(" ");

                    let hi_pos = state.session.codemap().lookup_byte_offset(body_span.lo).pos.to_usize();
                    let lo_pos = state.session.codemap().lookup_byte_offset(span.lo).pos.to_usize();
                    panic!((lo_pos, hi_pos, answer, 0));
                }
            });

            return control;
        }

        let new_name = self.new_name.clone();

        control.after_write_deps.stop = Compilation::Stop;
        control.after_write_deps.callback = Box::new(move |state| {
            //let krate = state.krate.unwrap().clone();
            let ast_map = state.ast_map.unwrap();
            let krate = ast_map.krate();
            LocalCrateReader::new(&state.session, &ast_map).read_crates(krate);
            let _ = lang_items::collect_language_items(&state.session, &ast_map);

            // According to some condition !
            //let ps = syntax::parse::ParseSess::new();
            //let ps = &state.session.parse_sess;
            let cratename = match attr::find_crate_name(&krate.attrs[..]) {
                Some(name) => name.to_string(),
                None => String::from("unknown_crate"),
            };
            debug!("{}", cratename);

            debug!("{:?}", token::str_to_ident(&new_name[..]));
            debug!("{}", node_to_find);
            let ast_node = ast_map.find(node_to_find);
            debug!("{:?}", ast_node);
            debug!("{:?}", token::str_to_ident(&new_name[..]));

            // find current path and syntax context
            let mut syntax_ctx = ast::SyntaxContext(0);
            // If None, then it is probably a field.
            // TODO fields have no super/sub-block conflict
            // Can we remove the compiler runs afterwards?
            if let Some(ast_node) = ast_node {
                match ast_node {
                    Node::NodeLocal(pat) => {
                        match pat.node {
                            hir::PatIdent(_, path, _) => {
                                syntax_ctx = path.node.ctxt;
                            },
                            _ => {}
                        }
                    },

                    _ => {}
                }
            }

            let path = build_path(DUMMY_SP, vec![token::str_to_ident(&new_name)]);
            // create resolver
            let mut resolver = resolve::create_resolver(&state.session, &ast_map, krate, resolve::MakeGlobMap::No,
            Some(Box::new(move |node: ast_map::Node, resolved: &mut bool| {
                if *resolved {
                    return true;
                }
                //debug!("Entered resolver callback");
                match node {
                    Node::NodeLocal(pat) => {
                        if pat.id == node_to_find {
                            debug!("Found node");
                            *resolved = true;
                            return true;
                        }
                    },
                    Node::NodeItem(item) => {
                        match item.node {
                            ItemImpl(_, _, _, _, _, ref impls) => {
                                for i in impls.iter() {
                                    if i.id == node_to_find {
                                        debug!("{:?}", i);
                                        debug!("Found node");
                                        *resolved = true;
                                        return true;
                                    }
                                }
                            },
                            ItemStruct(hir::VariantData::Struct(ref fields, _), _)
                            |  ItemStruct(hir::VariantData::Tuple(ref fields, _), _) => {
                                for field in fields.iter() {
                                    if field.node.id == node_to_find {
                                        *resolved = true;
                                        return true;
                                    }
                                }
                            },
                            _ => {}

                        }
                        if item.id == node_to_find {
                            debug!("Found node");
                            debug!("{:?}", item);
                            *resolved = true;
                            return true;
                        }
                    },
                    _ => {}
                }

                false
            })));

            match r_type {
                RefactorType::Type => {
                    let mut h = HashMap::new();
                    h.insert(String::new(), String::new());
                    debug!("{:?}", token::str_to_ident(&new_name[..]));
                    
                    let mut idens = ast_map.with_path(node_to_find, |path| {
                    let itr = token::get_ident_interner();

                    path.fold(Vec::new(), |mut s, e| {
                        let e = itr.get(e.name());
                        s.push(token::str_to_ident(&e[..]));
                        s })
                        //ast_map::path_to_string(path)
                    });


                    visit::walk_crate(&mut resolver, krate);

                    let new_iden = token::str_to_ident(&new_name[..]);
                    idens.pop();
                    idens.push(new_iden);

                    token::str_to_ident(&new_name[..]);
                    let path = build_path(DUMMY_SP, idens);

                    // resolver resolve node id
                    println!("{:?}", path);
                    if resolver.resolve_path(node_to_find, &path, 0, resolve::Namespace::TypeNS, true).is_some() {
                        // unwind at this location
                        panic!(h);
                    }
                },
                RefactorType::Variable => {
                    let mut t = token::str_to_ident(&new_name[..]);
                    t.ctxt = syntax_ctx;
                    debug!("{:?}", mtwt::resolve(t));
                    let path = build_path(DUMMY_SP, vec![t]);

                    visit::walk_crate(&mut resolver, krate);

                    let mut h = HashMap::new();
                    h.insert(String::new(), String::new());
                    debug!("{:?}", token::str_to_ident(&new_name[..]));
                    
                    // resolver resolve node id
                    //if resolver.resolve_path(node_to_find, &path) {
                    if resolver.resolve_path(node_to_find, &path, 0, resolve::Namespace::ValueNS, true).is_some() {
                        // unwind at this location
                        panic!(h);
                    }
                    //println!("{:?}", mtwt::resolve( token::str_to_ident(&new_name[..])));

                },
                RefactorType::Function => {
                    let mut idens = ast_map.with_path(node_to_find, |path| {
                    let itr = token::get_ident_interner();

                    path.fold(Vec::new(), |mut s, e| {
                        let e = itr.get(e.name());
                        s.push(token::str_to_ident(&e[..]));
                        s
                    })
                    //ast_map::path_to_string(path)

                    });

                    let new_iden = token::str_to_ident(&new_name[..]);
                    idens.pop();
                    idens.push(new_iden);

                    visit::walk_crate(&mut resolver, krate);

                    let mut h = HashMap::new();
                    h.insert(String::new(), String::new());
                    debug!("{:?}", token::str_to_ident(&new_name[..]));
                    
                    // TODO 
                    // let path = cx.path(DUMMY_SP, idens);
                    // resolver resolve node id
                    //if resolver.resolve_path(node_to_find, &path) {
                    if resolver.resolve_path(node_to_find, &path, 0, resolve::Namespace::ValueNS, true).is_some() {
                        // unwind at this location
                        debug!("BAD ValueNS");
                        panic!(h);
                    }

                    // Is it possible for type namespace to ever conflict with functions?
                    /*if resolver.resolve_path(node_to_find, &path, 0, resolve::Namespace::TypeNS, true).is_some() {
                        // unwind at this location
                        debug!("BAD TypeNS");
                        panic!(h);
                    }*/

                    //println!("{:?}", mtwt::resolve( token::str_to_ident(&new_name[..])));
                },
                _ => { /* Reduced graph check falls here */ }
            }
        });

        control
    }
}
