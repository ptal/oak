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
  under_unit: bool,
  under_ty_ascription: Option<IType>
}

impl Depth
{
  pub fn infer(grammar: IGrammar) -> Partial<TGrammar> {
    let mut engine = Depth::new(grammar);
    engine.surface.surface();
    if engine.surface.error { return Partial::Nothing }
    engine.warn_recursive_type();
    engine.reduce_all_rec();
    engine.depth();
    if engine.surface.error { return Partial::Nothing }
    engine.reduce_all_invisible();
    engine.check_all_rules_have_type();
    let grammar = engine.surface.grammar;
    if grammar.attributes.print_typing.debug() {
      println!("After applying Depth.");
      print_debug(&grammar);
    }
    if engine.surface.error {
      Partial::Nothing
    }
    else {
      Partial::Value(grammar.map_exprs_info(engine.exprs_info))
    }
  }

  fn new(grammar: IGrammar) -> Depth {
    Depth {
      surface: Surface::new(grammar),
      exprs_info: vec![],
      under_unit: false,
      under_ty_ascription: None
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

  fn check_all_rules_have_type(&mut self) {
    for rule in self.surface.grammar.rules.clone() {
      if self.type_of(rule.expr_idx) == External {
        self.surface.error = true;
        rule.name.span().unstable()
          .error(format!("could not infer the type of this rule, please use type ascription, e.g. `r: Expr = e`."))
          .emit();
      }
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

  fn visit_expr_switch_ascription(&mut self, this: usize, new: Option<IType>) {
    let old = self.under_ty_ascription.clone();
    self.under_ty_ascription = new;
    self.visit_expr(this);
    self.under_ty_ascription = old;
  }

  fn error_if_not_match_ty_ascription(&mut self, this: usize, ty: IType, aty: IType) {
    if !ty.syntactic_eq(&self.surface.grammar, &aty) {
      self.surface.error = true;
      self.surface.grammar[this].span().unstable()
        .error(format!("found type {} but expected type {}",
          ty.display(&self.surface.grammar), aty.display(&self.surface.grammar)))
        .emit();
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
    else if this_ty.is_unit_kind() && self.under_ty_ascription.is_none() {
      let old = self.under_unit;
      self.under_unit = true;
      walk_expr(self, this);
      self.under_unit = old;
    }
    else {
      if let Some(aty) = self.under_ty_ascription.clone() {
        if this_ty == External {
          self.surface.type_expr(this, aty);
        }
        else {
          self.error_if_not_match_ty_ascription(this, this_ty, aty);
        }
      }
      walk_expr(self, this);
    }
  }

  // Depth rules
  unit_visitor_impl!(choice);

  // If we are under a type ascription, then a single element of the sequence must take that type, all others must be unit.
  fn visit_sequence(&mut self, _this: usize, children: Vec<usize>) {
    if self.under_ty_ascription.is_some() {
      for child in children {
        let child_ty = self.type_of(child);
        if child_ty.is_unit_kind() {
          self.visit_expr_switch_ascription(child, None);
        }
        else {
          self.visit_expr(child);
        }
      }
    }
    else {
      walk_exprs(self, children);
    }
  }

  fn visit_type_ascription(&mut self, this: usize, child: usize, ty: IType) {
    if let Some(aty) = self.under_ty_ascription.clone() {
      self.error_if_not_match_ty_ascription(this, ty.clone(), aty);
    }
    self.surface_expr(child);
    self.visit_expr_switch_ascription(child, Some(ty));
  }

  fn visit_syntactic_predicate(&mut self, _this: usize, child: usize) {
    self.surface_expr(child);
    self.visit_expr(child);
  }

  // We rely on the Rust compiler to spot type mismatch between semantic action's return type and type ascription.
  fn visit_semantic_action(&mut self, _this: usize, child: usize, _action: syn::Expr) {
    self.surface_expr(child);
    self.visit_expr_switch_ascription(child, None);
  }
}
