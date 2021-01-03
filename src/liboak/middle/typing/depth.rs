// Copyright 2016 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use middle::typing::ast::*;
use middle::typing::type_rewriting::*;
use middle::typing::ast::Type::*;
use middle::typing::ast::IType::*;
use middle::typing::surface::*;
use middle::typing::typing_printer::*;

pub struct Depth
{
  surface: Surface,
  exprs_info: Vec<ExprType>,
  under_unit: bool
}

impl Depth
{
  pub fn infer(grammar: IGrammar) -> TGrammar {
    let mut engine = Depth::new(grammar);
    engine.surface.surface();
    engine.warn_recursive_type();
    engine.reduce_all_rec();
    engine.depth();
    engine.reduce_all_invisible();
    let grammar = engine.surface.grammar;
    if grammar.attributes.print_typing.debug() {
      println!("After applying Depth.");
      print_debug(&grammar);
    }
    grammar.map_exprs_info(engine.exprs_info)
  }

  fn new(grammar: IGrammar) -> Depth {
    Depth {
      surface: Surface::new(grammar),
      exprs_info: vec![],
      under_unit: false
    }
  }

  fn surface_expr(&mut self, expr_idx: usize) {
    self.surface.visit_expr(expr_idx);
    if self.surface.grammar.attributes.print_typing.debug() {
      println!("Applying Surface in Depth on expr {}.", expr_idx);
      print_debug(&self.surface.grammar);
      println!("");
    }
  }

  fn reduce_all_rec(&mut self) {
    let tys: Vec<_> = self.surface.grammar.exprs_info.iter().map(|e| e.ty.clone()).collect();
    let tys: Vec<_> = tys.into_iter().map(|ty| TypeRewriting::reduce_rec(&self.surface.grammar, ty)).collect();
    for (expr_info, ty) in self.surface.grammar.exprs_info.iter_mut().zip(tys.into_iter()) {
      expr_info.ty = ty;
    }
  }

  fn reduce_all_invisible(&mut self) {
    for expr_info in self.surface.grammar.exprs_info.clone() {
      let ty = TypeRewriting::reduce_final(expr_info.ty);
      self.exprs_info.push(ExprType::new(expr_info.span, ty))
    }
  }

  fn depth(&mut self) {
    for rule in self.surface.grammar.rules.clone() {
      self.visit_expr(rule.expr_idx);
    }
  }

  fn type_of(&self, expr_idx: usize) -> IType {
    self.surface.type_of(expr_idx)
  }

  fn warn_recursive_type(&mut self) {
    let mut rec_set = RecSet::empty();
    for rule in &self.surface.grammar.rules {
      if let Rec(r) = self.type_of(rule.expr_idx) {
        rec_set = rec_set.union(r);
      }
    }
    rec_set = rec_set.keep_only_polymorphic_paths();
    if !rec_set.is_empty() {
      for rec_path in rec_set.path_set {
        self.surface.grammar.find_rule_by_ident(&rec_path.path[0]).span().unstable()
          .warning(format!("infinite recursive type automatically replaced by `(^)`: {}\n\
            Semantic actions along the path are ignored.", rec_path.display()))
          .emit();
      }
    }
  }
}

impl ExprByIndex for Depth
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.surface.expr_by_index(index)
  }
}

impl Visitor<()> for Depth
{
  fn visit_expr(&mut self, this: usize) {
    let this_ty = self.type_of(this);
    assert!(this_ty != Infer,
      format!("Every expression must be typed during the surface inference: {}: {:?}",
        this, self.expr_by_index(this)));

    if self.under_unit {
      self.surface.type_expr(this, Regular(Unit));
      walk_expr(self, this);
    }
    else if this_ty.is_unit_kind() {
      let old = self.under_unit;
      self.under_unit = true;
      walk_expr(self, this);
      self.under_unit = old;
    }
    else {
      walk_expr(self, this);
    }
  }

  // Depth rules
  unit_visitor_impl!(sequence);
  unit_visitor_impl!(choice);

  fn visit_type_ascription(&mut self, _this: usize, child: usize, _ty: IType) {
    self.surface_expr(child);
    self.visit_expr(child);
  }

  fn visit_syntactic_predicate(&mut self, _this: usize, child: usize) {
    self.surface_expr(child);
    self.visit_expr(child);
  }

  fn visit_semantic_action(&mut self, _this: usize, child: usize, _action: syn::Expr) {
    self.surface_expr(child);
    self.visit_expr(child);
  }
}
