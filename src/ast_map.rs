// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from front/map/mod.rs

pub use self::Node::*;
use self::MapEntry::*;

use syntax::ast::*;//{Name, NodeId, Ident, CRATE_NODE_ID, DUMMY_NODE_ID};
use rustc::metadata::inline::InlinedItem;
use syntax::visit::{self, Visitor};
use syntax::ast_util;
use syntax::codemap::{Span, Spanned};

use std::cell::RefCell;
use std::iter::{self, repeat};

/// A Visitor that walks over an AST and collects Node's into an AST Map.
struct NodeCollector<'ast> {
    map: Vec<MapEntry<'ast>>,
    parent_node: NodeId,
}


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PathElem {
    PathMod(Name),
    PathName(Name)
}

pub fn map_crate<'ast>(forest: &'ast mut Forest) -> Map<'ast> {
    let mut collector = NodeCollector {
        map: vec![],
        parent_node: CRATE_NODE_ID,
    };
    collector.insert_entry(CRATE_NODE_ID, RootCrate);
    visit::walk_crate(&mut collector, &forest.krate);
    let map = collector.map;

    Map {
        forest: forest,
        map: RefCell::new(map)
    }
}

/// Represents a mapping from Node IDs to AST elements and their parent
/// Node IDs
#[derive(Clone)]
pub struct Map<'ast> {
    /// The backing storage for all the AST nodes.
    pub forest: &'ast Forest,

    /// NodeIds are sequential integers from 0, so we can be
    /// super-compact by storing them in a vector. Not everything with
    /// a NodeId is in the map, but empirically the occupancy is about
    /// 75-80%, so there's not too much overhead (certainly less than
    /// a hashmap, since they (at the time of writing) have a maximum
    /// of 75% occupancy).
    ///
    /// Also, indexing is pretty quick when you've got a vector and
    /// plain old integers.
    map: RefCell<Vec<MapEntry<'ast>>>
}

/// Stores a crate and any number of inlined items from other crates.
pub struct Forest {
    pub krate: Crate//    inlined_items: TypedArena<InlinedParent>
}

impl Forest {
    pub fn new(krate: Crate) -> Forest {
        Forest {
            krate: krate,
            //inlined_items: TypedArena::new()
        }
    }

    pub fn krate<'ast>(&'ast self) -> &'ast Crate {
        &self.krate
    }
}

/// Represents an entry and its parent NodeID.
/// The odd layout is to bring down the total size.
#[derive(Copy, Debug)]
enum MapEntry<'ast> {
    /// Placeholder for holes in the map.
    NotPresent,

    /// All the node types, with a parent ID.
    EntryItem(NodeId, &'ast Item),
    EntryForeignItem(NodeId, &'ast ForeignItem),
    EntryTraitItem(NodeId, &'ast TraitItem),
    EntryImplItem(NodeId, &'ast ImplItem),
    EntryVariant(NodeId, &'ast Variant),
    EntryExpr(NodeId, &'ast Expr),
    EntryStmt(NodeId, &'ast Stmt),
    EntryArg(NodeId, &'ast Pat),
    EntryLocal(NodeId, &'ast Pat),
    EntryPat(NodeId, &'ast Pat),
    EntryBlock(NodeId, &'ast Block),
    EntryStructCtor(NodeId, &'ast StructDef),
    EntryLifetime(NodeId, &'ast Lifetime),
    EntryTyParam(NodeId, &'ast TyParam),

    /// Roots for node trees.
    RootCrate,
    RootInlinedParent(&'ast InlinedParent)
}

impl<'ast> Clone for MapEntry<'ast> {
    fn clone(&self) -> MapEntry<'ast> {
        *self
    }
}

#[derive(Debug)]
pub struct InlinedParent {
    path: Vec<PathElem>,
    ii: InlinedItem
}

#[derive(Copy, Clone, Debug)]
pub enum Node<'ast> {
    NodeItem(&'ast Item),
    NodeForeignItem(&'ast ForeignItem),
    NodeTraitItem(&'ast TraitItem),
    NodeImplItem(&'ast ImplItem),
    NodeVariant(&'ast Variant),
    NodeExpr(&'ast Expr),
    NodeStmt(&'ast Stmt),
    NodeArg(&'ast Pat),
    NodeLocal(&'ast Pat),
    NodePat(&'ast Pat),
    NodeBlock(&'ast Block),

    /// NodeStructCtor represents a tuple struct.
    NodeStructCtor(&'ast StructDef),

    NodeLifetime(&'ast Lifetime),
    NodeTyParam(&'ast TyParam)
}

impl<'ast> MapEntry<'ast> {
    fn from_node(p: NodeId, node: Node<'ast>) -> MapEntry<'ast> {
        match node {
            NodeItem(n) => EntryItem(p, n),
            NodeForeignItem(n) => EntryForeignItem(p, n),
            NodeTraitItem(n) => EntryTraitItem(p, n),
            NodeImplItem(n) => EntryImplItem(p, n),
            NodeVariant(n) => EntryVariant(p, n),
            NodeExpr(n) => EntryExpr(p, n),
            NodeStmt(n) => EntryStmt(p, n),
            NodeArg(n) => EntryArg(p, n),
            NodeLocal(n) => EntryLocal(p, n),
            NodePat(n) => EntryPat(p, n),
            NodeBlock(n) => EntryBlock(p, n),
            NodeStructCtor(n) => EntryStructCtor(p, n),
            NodeLifetime(n) => EntryLifetime(p, n),
            NodeTyParam(n) => EntryTyParam(p, n),
        }
    }

    fn parent_node(self) -> Option<NodeId> {
        Some(match self {
            EntryItem(id, _) => id,
            EntryForeignItem(id, _) => id,
            EntryTraitItem(id, _) => id,
            EntryImplItem(id, _) => id,
            EntryVariant(id, _) => id,
            EntryExpr(id, _) => id,
            EntryStmt(id, _) => id,
            EntryArg(id, _) => id,
            EntryLocal(id, _) => id,
            EntryPat(id, _) => id,
            EntryBlock(id, _) => id,
            EntryStructCtor(id, _) => id,
            EntryLifetime(id, _) => id,
            EntryTyParam(id, _) => id,
            _ => return None
        })
    }

    fn to_node(self) -> Option<Node<'ast>> {
        Some(match self {
            EntryItem(_, n) => NodeItem(n),
            EntryForeignItem(_, n) => NodeForeignItem(n),
            EntryTraitItem(_, n) => NodeTraitItem(n),
            EntryImplItem(_, n) => NodeImplItem(n),
            EntryVariant(_, n) => NodeVariant(n),
            EntryExpr(_, n) => NodeExpr(n),
            EntryStmt(_, n) => NodeStmt(n),
            EntryArg(_, n) => NodeArg(n),
            EntryLocal(_, n) => NodeLocal(n),
            EntryPat(_, n) => NodePat(n),
            EntryBlock(_, n) => NodeBlock(n),
            EntryStructCtor(_, n) => NodeStructCtor(n),
            EntryLifetime(_, n) => NodeLifetime(n),
            EntryTyParam(_, n) => NodeTyParam(n),
            _ => return None
        })
    }
}

impl<'ast> NodeCollector<'ast> {
    fn insert_entry(&mut self, id: NodeId, entry: MapEntry<'ast>) {
        debug!("ast_map: {:?} => {:?}", id, entry);
        let len = self.map.len();
        if id as usize >= len {
            self.map.extend(repeat(NotPresent).take(id as usize - len + 1));
        }
        self.map[id as usize] = entry;
    }

    fn insert(&mut self, id: NodeId, node: Node<'ast>) {
        let entry = MapEntry::from_node(self.parent_node, node);
        self.insert_entry(id, entry);
    }

    fn visit_fn_decl(&mut self, decl: &'ast FnDecl) {
        for a in &decl.inputs {
            self.insert(a.id, NodeArg(&*a.pat));
        }
    }
}

impl<'ast> Visitor<'ast> for NodeCollector<'ast> {
    fn visit_item(&mut self, i: &'ast Item) {
        self.insert(i.id, NodeItem(i));

        let parent_node = self.parent_node;
        self.parent_node = i.id;

        match i.node {
            ItemImpl(_, _, _, _, _, ref impl_items) => {
                for ii in impl_items {
                    self.insert(ii.id, NodeImplItem(ii));
                }
            }
            ItemEnum(ref enum_definition, _) => {
                for v in &enum_definition.variants {
                    self.insert(v.node.id, NodeVariant(&**v));
                }
            }
            ItemForeignMod(ref nm) => {
                for nitem in &nm.items {
                    self.insert(nitem.id, NodeForeignItem(&**nitem));
                }
            }
            ItemStruct(ref struct_def, _) => {
                // If this is a tuple-like struct, register the constructor.
                match struct_def.ctor_id {
                    Some(ctor_id) => {
                        self.insert(ctor_id, NodeStructCtor(&**struct_def));
                    }
                    None => {}
                }
            }
            ItemTrait(_, _, ref bounds, ref trait_items) => {
                for b in bounds.iter() {
                    if let TraitTyParamBound(ref t, TraitBoundModifier::None) = *b {
                        self.insert(t.trait_ref.ref_id, NodeItem(i));
                    }
                }

                for ti in trait_items {
                    self.insert(ti.id, NodeTraitItem(ti));
                }
            }
            ItemUse(ref view_path) => {
                match view_path.node {
                    ViewPathList(_, ref paths) => {
                        for path in paths {
                            self.insert(path.node.id(), NodeItem(i));
                        }
                    }
                    _ => ()
                }
            }
            _ => {}
        }
        visit::walk_item(self, i);
        self.parent_node = parent_node;
    }

    fn visit_generics(&mut self, generics: &'ast Generics) {
        for ty_param in generics.ty_params.iter() {
            self.insert(ty_param.id, NodeTyParam(ty_param));
        }

        visit::walk_generics(self, generics);
    }

    fn visit_trait_item(&mut self, ti: &'ast TraitItem) {
        let parent_node = self.parent_node;
        self.parent_node = ti.id;
        visit::walk_trait_item(self, ti);
        self.parent_node = parent_node;
    }

    fn visit_impl_item(&mut self, ii: &'ast ImplItem) {
        let parent_node = self.parent_node;
        self.parent_node = ii.id;

        visit::walk_impl_item(self, ii);

        self.parent_node = parent_node;
    }

    fn visit_pat(&mut self, pat: &'ast Pat) {
        self.insert(pat.id, match pat.node {
            // Note: this is at least *potentially* a pattern...
            PatIdent(..) => NodeLocal(pat),
            _ => NodePat(pat)
        });

        let parent_node = self.parent_node;
        self.parent_node = pat.id;
        visit::walk_pat(self, pat);
        self.parent_node = parent_node;
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        self.insert(expr.id, NodeExpr(expr));
        let parent_node = self.parent_node;
        self.parent_node = expr.id;
        visit::walk_expr(self, expr);
        self.parent_node = parent_node;
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        let id = ast_util::stmt_id(stmt);
        self.insert(id, NodeStmt(stmt));
        let parent_node = self.parent_node;
        self.parent_node = id;
        visit::walk_stmt(self, stmt);
        self.parent_node = parent_node;
    }

    fn visit_fn(&mut self, fk: visit::FnKind<'ast>, fd: &'ast FnDecl,
                b: &'ast Block, s: Span, id: NodeId) {
        let parent_node = self.parent_node;
        self.parent_node = id;
        self.visit_fn_decl(fd);
        visit::walk_fn(self, fk, fd, b, s);
        self.parent_node = parent_node;
    }

    fn visit_ty(&mut self, ty: &'ast Ty) {
        let parent_node = self.parent_node;
        self.parent_node = ty.id;
        match ty.node {
            TyBareFn(ref fd) => {
                self.visit_fn_decl(&*fd.decl);
            }
            _ => {}
        }
        visit::walk_ty(self, ty);
        self.parent_node = parent_node;
    }

    fn visit_block(&mut self, block: &'ast Block) {
        self.insert(block.id, NodeBlock(block));
        let parent_node = self.parent_node;
        self.parent_node = block.id;
        visit::walk_block(self, block);
        self.parent_node = parent_node;
    }

    fn visit_lifetime_ref(&mut self, lifetime: &'ast Lifetime) {
        self.insert(lifetime.id, NodeLifetime(lifetime));
    }

    fn visit_lifetime_def(&mut self, def: &'ast LifetimeDef) {
        self.visit_lifetime_ref(&def.lifetime);
    }
}

impl<'ast> Map<'ast> {
    fn entry_count(&self) -> usize {
        self.map.borrow().len()
    }

    fn find_entry(&self, id: NodeId) -> Option<MapEntry<'ast>> {
        self.map.borrow().get(id as usize).cloned()
    }

    pub fn krate(&self) -> &'ast Crate {
        &self.forest.krate
    }

    /// Retrieve the Node corresponding to `id`, panicking if it cannot
    /// be found.
    pub fn get(&self, id: NodeId) -> Node<'ast> {
        match self.find(id) {
            Some(node) => node,
            None => panic!("couldn't find node id {} in the AST map", id)
        }
    }

    /// Retrieve the Node corresponding to `id`, returning None if
    /// cannot be found.
    pub fn find(&self, id: NodeId) -> Option<Node<'ast>> {
        self.find_entry(id).and_then(|x| x.to_node())
    }

    /// Similar to get_parent, returns the parent node id or id if there is no
    /// parent.
    /// This function returns the immediate parent in the AST, whereas get_parent
    /// returns the enclosing item. Note that this might not be the actual parent
    /// node in the AST - some kinds of nodes are not in the map and these will
    /// never appear as the parent_node. So you can always walk the parent_nodes
    /// from a node to the root of the ast (unless you get the same id back here
    /// that can happen if the id is not in the map itself or is just weird).
    pub fn get_parent_node(&self, id: NodeId) -> NodeId {
        self.find_entry(id).and_then(|x| x.parent_node()).unwrap_or(id)
    }
}
