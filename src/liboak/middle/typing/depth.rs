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

pub struct Depth<'a, 'b: 'a>
{
  surface: Surface<'a, 'b>,
  exprs_info: Vec<ExprType>,
  under_unit: bool
}

impl<'a, 'b> Depth<'a, 'b>
{
  pub fn infer(grammar: IGrammar<'a, 'b>) -> TGrammar<'a, 'b> {
    let mut engine = Depth::new(grammar);
    engine.surface.surface();
    engine.depth();
    engine.surface.grammar.map_exprs_info(engine.exprs_info)
  }

  fn new(grammar: IGrammar<'a, 'b>) -> Depth<'a, 'b> {
    Depth {
      surface: Surface::new(grammar),
      exprs_info: vec![],
      under_unit: false
    }
  }

  fn surface_expr(&mut self, expr_idx: usize) {
    self.surface.visit_expr(expr_idx);
  }

  fn depth(&mut self) {
    for rule in self.surface.grammar.rules.clone() {
      self.visit_expr(rule.expr_idx);
    }
  }

  fn type_of(&self, expr_idx: usize) -> IType {
    self.surface.type_of(expr_idx)
  }

  fn push_expr_info(&mut self, expr_idx: usize, ty: Type) {
    let current = self.surface.grammar[expr_idx].clone();
    self.exprs_info.push(ExprType::new(current.span, ty));
  }
}

impl<'a, 'b> ExprByIndex for Depth<'a, 'b>
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.surface.expr_by_index(index)
  }
}

impl<'a, 'b> Visitor<()> for Depth<'a, 'b>
{
  fn visit_expr(&mut self, this: usize) {
    let this_ty = self.type_of(this);
    assert!(this_ty != Infer,
      format!("Every expression must be typed during the surface inference: {:?}", self.expr_by_index(this)));

    let final_type =
      if self.under_unit {
        walk_expr(self, this);
        Unit
      }
      else {
        let reduced_ty = TypeRewriting::final_reduce(this_ty);
        if reduced_ty == Unit {
          let old = self.under_unit;
          self.under_unit = true;
          walk_expr(self, this);
          self.under_unit = old;
        }
        else {
          walk_expr(self, this);
        }
        reduced_ty
      };
    self.push_expr_info(this, final_type);
  }

  // Depth axioms

  unit_visitor_impl!(str_literal);
  unit_visitor_impl!(non_terminal);
  unit_visitor_impl!(atom);

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

  fn visit_semantic_action(&mut self, _this: usize, child: usize, _action: Ident) {
    self.surface_expr(child);
    self.visit_expr(child);
  }
}
