// Copyright 2012-2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Taken from middle/infer/error_reporting.rs

use self::FreshOrKept::*;

use rustc::ast_map;
use rustc::middle::def;
use rustc::middle::infer::{self, InferCtxt};
use rustc::middle::infer::region_inference::SameRegions;
use rustc::middle::ty::{self, Ty, TypeError, HasTypeFlags};
use rustc::middle::subst;
use std::cell::{Cell, RefCell};
use std::char::from_u32;
use std::collections::HashSet;
use syntax::ast;
use syntax::ast_util::name_to_dummy_lifetime;
use syntax::owned_slice::OwnedSlice;
use syntax::parse::token;
use syntax::print::pprust;
use syntax::ptr::P;

struct RebuildPathInfo<'a> {
    path: &'a ast::Path,
    // indexes to insert lifetime on path.lifetimes
    indexes: Vec<u32>,
    // number of lifetimes we expect to see on the type referred by `path`
    // (e.g., expected=1 for struct Foo<'a>)
    expected: u32,
    anon_nums: &'a HashSet<u32>,
    region_names: &'a HashSet<ast::Name>
}

struct Rebuilder<'a, 'tcx: 'a> {
    tcx: &'a ty::ctxt<'tcx>,
    fn_decl: &'a ast::FnDecl,
    expl_self_opt: Option<&'a ast::ExplicitSelf_>,
    generics: &'a ast::Generics,
    same_regions: &'a [SameRegions],
    life_giver: &'a LifeGiver,
    cur_anon: Cell<u32>,
    inserted_anons: RefCell<HashSet<u32>>,
}

enum FreshOrKept {
    Fresh,
    Kept
}

impl<'a, 'tcx> Rebuilder<'a, 'tcx> {
    fn new(tcx: &'a ty::ctxt<'tcx>,
           fn_decl: &'a ast::FnDecl,
           expl_self_opt: Option<&'a ast::ExplicitSelf_>,
           generics: &'a ast::Generics,
           same_regions: &'a [SameRegions],
           life_giver: &'a LifeGiver)
           -> Rebuilder<'a, 'tcx> {
        Rebuilder {
            tcx: tcx,
            fn_decl: fn_decl,
            expl_self_opt: expl_self_opt,
            generics: generics,
            same_regions: same_regions,
            life_giver: life_giver,
            cur_anon: Cell::new(0),
            inserted_anons: RefCell::new(HashSet::new()),
        }
    }

    fn rebuild(&self)
               -> (ast::FnDecl, Option<ast::ExplicitSelf_>, ast::Generics) {
        let mut expl_self_opt = self.expl_self_opt.cloned();
        let mut inputs = self.fn_decl.inputs.clone();
        let mut output = self.fn_decl.output.clone();
        let mut ty_params = self.generics.ty_params.clone();
        let where_clause = self.generics.where_clause.clone();
        let mut kept_lifetimes = HashSet::new();
        for sr in self.same_regions {
            self.cur_anon.set(0);
            self.offset_cur_anon();
            let (anon_nums, region_names) =
                                self.extract_anon_nums_and_names(sr);
            let (lifetime, fresh_or_kept) = self.pick_lifetime(&region_names);
            match fresh_or_kept {
                Kept => { kept_lifetimes.insert(lifetime.name); }
                _ => ()
            }
            expl_self_opt = self.rebuild_expl_self(expl_self_opt, lifetime,
                                                   &anon_nums, &region_names);
            inputs = self.rebuild_args_ty(&inputs[..], lifetime,
                                          &anon_nums, &region_names);
            output = self.rebuild_output(&output, lifetime, &anon_nums, &region_names);
            ty_params = self.rebuild_ty_params(ty_params, lifetime,
                                               &region_names);
        }
        let fresh_lifetimes = self.life_giver.get_generated_lifetimes();
        let all_region_names = self.extract_all_region_names();
        let generics = self.rebuild_generics(self.generics,
                                             &fresh_lifetimes,
                                             &kept_lifetimes,
                                             &all_region_names,
                                             ty_params,
                                             where_clause);
        let new_fn_decl = ast::FnDecl {
            inputs: inputs,
            output: output,
            variadic: self.fn_decl.variadic
        };
        (new_fn_decl, expl_self_opt, generics)
    }

    fn pick_lifetime(&self,
                     region_names: &HashSet<ast::Name>)
                     -> (ast::Lifetime, FreshOrKept) {
        if !region_names.is_empty() {
            // It's not necessary to convert the set of region names to a
            // vector of string and then sort them. However, it makes the
            // choice of lifetime name deterministic and thus easier to test.
            let mut names = Vec::new();
            for rn in region_names {
                let lt_name = rn.to_string();
                names.push(lt_name);
            }
            names.sort();
            let name = token::str_to_ident(&names[0]).name;
            return (name_to_dummy_lifetime(name), Kept);
        }
        return (self.life_giver.give_lifetime(), Fresh);
    }

    fn extract_anon_nums_and_names(&self, same_regions: &SameRegions)
                                   -> (HashSet<u32>, HashSet<ast::Name>) {
        let mut anon_nums = HashSet::new();
        let mut region_names = HashSet::new();
        for br in &same_regions.regions {
            match *br {
                ty::BrAnon(i) => {
                    anon_nums.insert(i);
                }
                ty::BrNamed(_, name) => {
                    region_names.insert(name);
                }
                _ => ()
            }
        }
        (anon_nums, region_names)
    }

    fn extract_all_region_names(&self) -> HashSet<ast::Name> {
        let mut all_region_names = HashSet::new();
        for sr in self.same_regions {
            for br in &sr.regions {
                match *br {
                    ty::BrNamed(_, name) => {
                        all_region_names.insert(name);
                    }
                    _ => ()
                }
            }
        }
        all_region_names
    }

    fn inc_cur_anon(&self, n: u32) {
        let anon = self.cur_anon.get();
        self.cur_anon.set(anon+n);
    }

    fn offset_cur_anon(&self) {
        let mut anon = self.cur_anon.get();
        while self.inserted_anons.borrow().contains(&anon) {
            anon += 1;
        }
        self.cur_anon.set(anon);
    }

    fn inc_and_offset_cur_anon(&self, n: u32) {
        self.inc_cur_anon(n);
        self.offset_cur_anon();
    }

    fn track_anon(&self, anon: u32) {
        self.inserted_anons.borrow_mut().insert(anon);
    }

    fn rebuild_ty_params(&self,
                         ty_params: OwnedSlice<ast::TyParam>,
                         lifetime: ast::Lifetime,
                         region_names: &HashSet<ast::Name>)
                         -> OwnedSlice<ast::TyParam> {
        ty_params.map(|ty_param| {
            let bounds = self.rebuild_ty_param_bounds(ty_param.bounds.clone(),
                                                      lifetime,
                                                      region_names);
            ast::TyParam {
                ident: ty_param.ident,
                id: ty_param.id,
                bounds: bounds,
                default: ty_param.default.clone(),
                span: ty_param.span,
            }
        })
    }

    fn rebuild_ty_param_bounds(&self,
                               ty_param_bounds: OwnedSlice<ast::TyParamBound>,
                               lifetime: ast::Lifetime,
                               region_names: &HashSet<ast::Name>)
                               -> OwnedSlice<ast::TyParamBound> {
        ty_param_bounds.map(|tpb| {
            match tpb {
                &ast::RegionTyParamBound(lt) => {
                    // FIXME -- it's unclear whether I'm supposed to
                    // substitute lifetime here. I suspect we need to
                    // be passing down a map.
                    ast::RegionTyParamBound(lt)
                }
                &ast::TraitTyParamBound(ref poly_tr, modifier) => {
                    let tr = &poly_tr.trait_ref;
                    let last_seg = tr.path.segments.last().unwrap();
                    let mut insert = Vec::new();
                    let lifetimes = last_seg.parameters.lifetimes();
                    for (i, lt) in lifetimes.iter().enumerate() {
                        if region_names.contains(&lt.name) {
                            insert.push(i as u32);
                        }
                    }
                    let rebuild_info = RebuildPathInfo {
                        path: &tr.path,
                        indexes: insert,
                        expected: lifetimes.len() as u32,
                        anon_nums: &HashSet::new(),
                        region_names: region_names
                    };
                    let new_path = self.rebuild_path(rebuild_info, lifetime);
                    ast::TraitTyParamBound(ast::PolyTraitRef {
                        bound_lifetimes: poly_tr.bound_lifetimes.clone(),
                        trait_ref: ast::TraitRef {
                            path: new_path,
                            ref_id: tr.ref_id,
                        },
                        span: poly_tr.span,
                    }, modifier)
                }
            }
        })
    }

    fn rebuild_expl_self(&self,
                         expl_self_opt: Option<ast::ExplicitSelf_>,
                         lifetime: ast::Lifetime,
                         anon_nums: &HashSet<u32>,
                         region_names: &HashSet<ast::Name>)
                         -> Option<ast::ExplicitSelf_> {
        match expl_self_opt {
            Some(ref expl_self) => match *expl_self {
                ast::SelfRegion(lt_opt, muta, id) => match lt_opt {
                    Some(lt) => if region_names.contains(&lt.name) {
                        return Some(ast::SelfRegion(Some(lifetime), muta, id));
                    },
                    None => {
                        let anon = self.cur_anon.get();
                        self.inc_and_offset_cur_anon(1);
                        if anon_nums.contains(&anon) {
                            self.track_anon(anon);
                            return Some(ast::SelfRegion(Some(lifetime), muta, id));
                        }
                    }
                },
                _ => ()
            },
            None => ()
        }
        expl_self_opt
    }

    fn rebuild_generics(&self,
                        generics: &ast::Generics,
                        add: &Vec<ast::Lifetime>,
                        keep: &HashSet<ast::Name>,
                        remove: &HashSet<ast::Name>,
                        ty_params: OwnedSlice<ast::TyParam>,
                        where_clause: ast::WhereClause)
                        -> ast::Generics {
        let mut lifetimes = Vec::new();
        for lt in add {
            lifetimes.push(ast::LifetimeDef { lifetime: *lt,
                                              bounds: Vec::new() });
        }
        for lt in &generics.lifetimes {
            if keep.contains(&lt.lifetime.name) ||
                !remove.contains(&lt.lifetime.name) {
                lifetimes.push((*lt).clone());
            }
        }
        ast::Generics {
            lifetimes: lifetimes,
            ty_params: ty_params,
            where_clause: where_clause,
        }
    }

    fn rebuild_args_ty(&self,
                       inputs: &[ast::Arg],
                       lifetime: ast::Lifetime,
                       anon_nums: &HashSet<u32>,
                       region_names: &HashSet<ast::Name>)
                       -> Vec<ast::Arg> {
        let mut new_inputs = Vec::new();
        for arg in inputs {
            let new_ty = self.rebuild_arg_ty_or_output(&*arg.ty, lifetime,
                                                       anon_nums, region_names);
            let possibly_new_arg = ast::Arg {
                ty: new_ty,
                pat: arg.pat.clone(),
                id: arg.id
            };
            new_inputs.push(possibly_new_arg);
        }
        new_inputs
    }

    fn rebuild_output(&self, ty: &ast::FunctionRetTy,
                      lifetime: ast::Lifetime,
                      anon_nums: &HashSet<u32>,
                      region_names: &HashSet<ast::Name>) -> ast::FunctionRetTy {
        match *ty {
            ast::Return(ref ret_ty) => ast::Return(
                self.rebuild_arg_ty_or_output(&**ret_ty, lifetime, anon_nums, region_names)
            ),
            ast::DefaultReturn(span) => ast::DefaultReturn(span),
            ast::NoReturn(span) => ast::NoReturn(span)
        }
    }

    fn rebuild_arg_ty_or_output(&self,
                                ty: &ast::Ty,
                                lifetime: ast::Lifetime,
                                anon_nums: &HashSet<u32>,
                                region_names: &HashSet<ast::Name>)
                                -> P<ast::Ty> {
        let mut new_ty = P(ty.clone());
        let mut ty_queue = vec!(ty);
        while !ty_queue.is_empty() {
            let cur_ty = ty_queue.remove(0);
            match cur_ty.node {
                ast::TyRptr(lt_opt, ref mut_ty) => {
                    let rebuild = match lt_opt {
                        Some(lt) => region_names.contains(&lt.name),
                        None => {
                            let anon = self.cur_anon.get();
                            let rebuild = anon_nums.contains(&anon);
                            if rebuild {
                                self.track_anon(anon);
                            }
                            self.inc_and_offset_cur_anon(1);
                            rebuild
                        }
                    };
                    if rebuild {
                        let to = ast::Ty {
                            id: cur_ty.id,
                            node: ast::TyRptr(Some(lifetime), mut_ty.clone()),
                            span: cur_ty.span
                        };
                        new_ty = self.rebuild_ty(new_ty, P(to));
                    }
                    ty_queue.push(&*mut_ty.ty);
                }
                ast::TyPath(ref maybe_qself, ref path) => {
                    let a_def = match self.tcx.def_map.borrow().get(&cur_ty.id) {
                        None => {
                            self.tcx
                                .sess
                                .fatal(&format!(
                                        "unbound path {}",
                                        pprust::path_to_string(path)))
                        }
                        Some(d) => d.full_def()
                    };
                    match a_def {
                        def::DefTy(did, _) | def::DefStruct(did) => {
                            let generics = self.tcx.lookup_item_type(did).generics;

                            let expected =
                                generics.regions.len(subst::TypeSpace) as u32;
                            let lifetimes =
                                path.segments.last().unwrap().parameters.lifetimes();
                            let mut insert = Vec::new();
                            if lifetimes.is_empty() {
                                let anon = self.cur_anon.get();
                                for (i, a) in (anon..anon+expected).enumerate() {
                                    if anon_nums.contains(&a) {
                                        insert.push(i as u32);
                                    }
                                    self.track_anon(a);
                                }
                                self.inc_and_offset_cur_anon(expected);
                            } else {
                                for (i, lt) in lifetimes.iter().enumerate() {
                                    if region_names.contains(&lt.name) {
                                        insert.push(i as u32);
                                    }
                                }
                            }
                            let rebuild_info = RebuildPathInfo {
                                path: path,
                                indexes: insert,
                                expected: expected,
                                anon_nums: anon_nums,
                                region_names: region_names
                            };
                            let new_path = self.rebuild_path(rebuild_info, lifetime);
                            let qself = maybe_qself.as_ref().map(|qself| {
                                ast::QSelf {
                                    ty: self.rebuild_arg_ty_or_output(&qself.ty, lifetime,
                                                                      anon_nums, region_names),
                                    position: qself.position
                                }
                            });
                            let to = ast::Ty {
                                id: cur_ty.id,
                                node: ast::TyPath(qself, new_path),
                                span: cur_ty.span
                            };
                            new_ty = self.rebuild_ty(new_ty, P(to));
                        }
                        _ => ()
                    }

                }

                ast::TyPtr(ref mut_ty) => {
                    ty_queue.push(&*mut_ty.ty);
                }
                ast::TyVec(ref ty) |
                ast::TyFixedLengthVec(ref ty, _) => {
                    ty_queue.push(&**ty);
                }
                ast::TyTup(ref tys) => ty_queue.extend(tys.iter().map(|ty| &**ty)),
                _ => {}
            }
        }
        new_ty
    }

    fn rebuild_ty(&self,
                  from: P<ast::Ty>,
                  to: P<ast::Ty>)
                  -> P<ast::Ty> {

        fn build_to(from: P<ast::Ty>,
                    to: &mut Option<P<ast::Ty>>)
                    -> P<ast::Ty> {
            if Some(from.id) == to.as_ref().map(|ty| ty.id) {
                return to.take().expect("`to` type found more than once during rebuild");
            }
            from.map(|ast::Ty {id, node, span}| {
                let new_node = match node {
                    ast::TyRptr(lifetime, mut_ty) => {
                        ast::TyRptr(lifetime, ast::MutTy {
                            mutbl: mut_ty.mutbl,
                            ty: build_to(mut_ty.ty, to),
                        })
                    }
                    ast::TyPtr(mut_ty) => {
                        ast::TyPtr(ast::MutTy {
                            mutbl: mut_ty.mutbl,
                            ty: build_to(mut_ty.ty, to),
                        })
                    }
                    ast::TyVec(ty) => ast::TyVec(build_to(ty, to)),
                    ast::TyFixedLengthVec(ty, e) => {
                        ast::TyFixedLengthVec(build_to(ty, to), e)
                    }
                    ast::TyTup(tys) => {
                        ast::TyTup(tys.into_iter().map(|ty| build_to(ty, to)).collect())
                    }
                    ast::TyParen(typ) => ast::TyParen(build_to(typ, to)),
                    other => other
                };
                ast::Ty { id: id, node: new_node, span: span }
            })
        }

        build_to(from, &mut Some(to))
    }

    fn rebuild_path(&self,
                    rebuild_info: RebuildPathInfo,
                    lifetime: ast::Lifetime)
                    -> ast::Path
    {
        let RebuildPathInfo {
            path,
            indexes,
            expected,
            anon_nums,
            region_names,
        } = rebuild_info;

        let last_seg = path.segments.last().unwrap();
        let new_parameters = match last_seg.parameters {
            ast::ParenthesizedParameters(..) => {
                last_seg.parameters.clone()
            }

            ast::AngleBracketedParameters(ref data) => {
                let mut new_lts = Vec::new();
                if data.lifetimes.is_empty() {
                    // traverse once to see if there's a need to insert lifetime
                    let need_insert = (0..expected).any(|i| {
                        indexes.contains(&i)
                    });
                    if need_insert {
                        for i in 0..expected {
                            if indexes.contains(&i) {
                                new_lts.push(lifetime);
                            } else {
                                new_lts.push(self.life_giver.give_lifetime());
                            }
                        }
                    }
                } else {
                    for (i, lt) in data.lifetimes.iter().enumerate() {
                        if indexes.contains(&(i as u32)) {
                            new_lts.push(lifetime);
                        } else {
                            new_lts.push(*lt);
                        }
                    }
                }
                let new_types = data.types.map(|t| {
                    self.rebuild_arg_ty_or_output(&**t, lifetime, anon_nums, region_names)
                });
                let new_bindings = data.bindings.map(|b| {
                    P(ast::TypeBinding {
                        id: b.id,
                        ident: b.ident,
                        ty: self.rebuild_arg_ty_or_output(&*b.ty,
                                                          lifetime,
                                                          anon_nums,
                                                          region_names),
                        span: b.span
                    })
                });
                ast::AngleBracketedParameters(ast::AngleBracketedParameterData {
                    lifetimes: new_lts,
                    types: new_types,
                    bindings: new_bindings,
               })
            }
        };
        let new_seg = ast::PathSegment {
            identifier: last_seg.identifier,
            parameters: new_parameters
        };
        let mut new_segs = Vec::new();
        new_segs.push_all(path.segments.split_last().unwrap().1);
        new_segs.push(new_seg);
        ast::Path {
            span: path.span,
            global: path.global,
            segments: new_segs
        }
    }
}

pub trait Resolvable<'tcx> {
    fn resolve<'a>(&self, infcx: &InferCtxt<'a, 'tcx>) -> Self;
}

impl<'tcx> Resolvable<'tcx> for Ty<'tcx> {
    fn resolve<'a>(&self, infcx: &InferCtxt<'a, 'tcx>) -> Ty<'tcx> {
        infcx.resolve_type_vars_if_possible(self)
    }
}

impl<'tcx> Resolvable<'tcx> for ty::TraitRef<'tcx> {
    fn resolve<'a>(&self, infcx: &InferCtxt<'a, 'tcx>)
                   -> ty::TraitRef<'tcx> {
        infcx.resolve_type_vars_if_possible(self)
    }
}

impl<'tcx> Resolvable<'tcx> for ty::PolyTraitRef<'tcx> {
    fn resolve<'a>(&self,
                   infcx: &InferCtxt<'a, 'tcx>)
                   -> ty::PolyTraitRef<'tcx>
    {
        infcx.resolve_type_vars_if_possible(self)
    }
}

fn lifetimes_in_scope(tcx: &ty::ctxt,
                      scope_id: ast::NodeId)
                      -> Vec<ast::LifetimeDef> {
    let mut taken = Vec::new();
    let parent = tcx.map.get_parent(scope_id);
    let method_id_opt = match tcx.map.find(parent) {
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
                    ast::MethodImplItem(ref sig, _) => {
                        taken.push_all(&sig.generics.lifetimes);
                        Some(ii.id)
                    }
                    ast::MacImplItem(_) => tcx.sess.bug("unexpanded macro"),
                    _ => None,
                }
            }
            _ => None
        },
        None => None
    };
    if method_id_opt.is_some() {
        let method_id = method_id_opt.unwrap();
        let parent = tcx.map.get_parent(method_id);
        match tcx.map.find(parent) {
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

// LifeGiver is responsible for generating fresh lifetime names
struct LifeGiver {
    taken: HashSet<String>,
    counter: Cell<usize>,
    generated: RefCell<Vec<ast::Lifetime>>,
}

impl LifeGiver {
    fn with_taken(taken: &[ast::LifetimeDef]) -> LifeGiver {
        let mut taken_ = HashSet::new();
        for lt in taken {
            let lt_name = lt.lifetime.name.to_string();
            taken_.insert(lt_name);
        }
        LifeGiver {
            taken: taken_,
            counter: Cell::new(0),
            generated: RefCell::new(Vec::new()),
        }
    }

    fn inc_counter(&self) {
        let c = self.counter.get();
        self.counter.set(c+1);
    }

    fn give_lifetime(&self) -> ast::Lifetime {
        let lifetime;
        loop {
            let mut s = String::from("'");
            s.push_str(&num_to_string(self.counter.get()));
            if !self.taken.contains(&s) {
                lifetime = name_to_dummy_lifetime(
                                    token::str_to_ident(&s[..]).name);
                self.generated.borrow_mut().push(lifetime);
                break;
            }
            self.inc_counter();
        }
        self.inc_counter();
        return lifetime;

        // 0 .. 25 generates a .. z, 26 .. 51 generates aa .. zz, and so on
        fn num_to_string(counter: usize) -> String {
            let mut s = String::new();
            let (n, r) = (counter/26 + 1, counter % 26);
            let letter: char = from_u32((r+97) as u32).unwrap();
            for _ in 0..n {
                s.push(letter);
            }
            s
        }
    }

    fn get_generated_lifetimes(&self) -> Vec<ast::Lifetime> {
        self.generated.borrow().clone()
    }
}
