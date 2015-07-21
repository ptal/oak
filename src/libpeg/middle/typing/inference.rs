// Copyright 2014 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Give a type to any expression of the grammar. There are only three types, see `typing::ast` for explanations. It also reads the expression type annotations (invisible type `(^)` and the unit type `()`) and modify the type accordingly. It does not propagate the invisible types, this step is done in `typing::propagation`.
//! Literals (e.g. `"lit"`) and syntactic predicates (e.g. `&e` and `!e`) are by default invisibles.

pub use middle::attribute::ast::Grammar as AGrammar;
pub use middle::attribute::ast::Rule as ARule;
pub use middle::attribute::ast::Expression as AExpression;

use front::ast::TypeAnnotation;
use middle::typing::ast::*;
use middle::typing::ast::ExprTy::*;
use rust;
use std::iter::FromIterator;

pub struct InferenceEngine<'r>
{
  grammar: &'r mut Grammar
}

impl<'r> InferenceEngine<'r>
{
  pub fn infer(grammar: &'r mut Grammar, arules: HashMap<Ident, ARule>) {
    let mut engine = InferenceEngine {
      grammar: grammar
    };
    engine.infer_rules_type(arules);
  }

  fn infer_rules_type(&mut self, arules: HashMap<Ident, ARule>)
  {
    for (id, rule) in arules.into_iter() {
      let typed_rule = self.infer_rule_type(rule);
      self.grammar.rules.insert(id, typed_rule);
    }
  }

  fn infer_rule_type(&self, rule: ARule) -> Rule
  {
    let expr = self.infer_expr_type(rule.def);
    Rule{
      name: rule.name,
      def: expr
    }
  }

  fn infer_expr_type(&self, expr: Box<AExpression>) -> Box<Expression>
  {
    let sp = expr.span.clone();
    let ty = expr.ty.clone();
    let typed_expr = match expr.node {
      AnySingleChar => self.infer_identity_expr(sp, AnySingleChar),
      CharacterClass(c) => self.infer_identity_expr(sp, CharacterClass(c)),
      StrLiteral(s) => self.infer_unit_expr(sp, StrLiteral(s)),
      NotPredicate(sub) => self.infer_sub_unit_expr(sp, sub, |e| NotPredicate(e)),
      AndPredicate(sub) => self.infer_sub_unit_expr(sp, sub, |e| AndPredicate(e)),
      NonTerminalSymbol(ident) => self.infer_rule_type_ph(sp, ident),
      ZeroOrMore(sub) => self.infer_sub_expr(sp, sub, |e| ZeroOrMore(e), Identity),
      OneOrMore(sub) => self.infer_sub_expr(sp, sub, |e| OneOrMore(e), Identity),
      Optional(sub) =>  self.infer_sub_expr(sp, sub, |e| Optional(e), Identity),
      Sequence(sub) => self.infer_tuple_expr(sp, sub),
      Choice(sub) => self.infer_choice_expr(sp, sub),
      workaround => { // Waiting for Rust FIX: collaterally moved values.
        if let SemanticAction(sub, ident) = workaround {
          self.infer_semantic_action(sp, sub, ident)
        } else {
          unreachable!();
        }
      }
    };
    self.type_annotation(typed_expr, ty)
  }

  fn type_annotation(&self, expr: Box<Expression>, ty: Option<TypeAnnotation>) -> Box<Expression>
  {
    if let Some(ty) = ty {
      match ty {
        TypeAnnotation::Invisible => {
          expr.to_invisible_type();
        }
        TypeAnnotation::Unit => {
          expr.to_unit_type();
        }
      }
    }
    expr
  }

  fn infer_identity_expr(&self, sp: Span, node: ExpressionNode) -> Box<Expression>
  {
    box Expression::new(sp, node, Identity)
  }

  fn infer_unit_expr(&self, sp: Span, node: ExpressionNode) -> Box<Expression>
  {
    box Expression::new(sp, node, ExprTy::unit())
  }

  fn infer_sub_unit_expr<F>(&self, sp: Span, sub: Box<AExpression>, make_node: F) -> Box<Expression>
    where F: Fn(Box<Expression>) -> ExpressionNode
  {
    self.infer_unit_expr(sp, make_node(self.infer_expr_type(sub)))
  }

  fn infer_rule_type_ph(&self, sp: Span, ident: Ident) -> Box<Expression>
  {
    box Expression::new(sp,
      NonTerminalSymbol(ident.clone()),
      Identity)
  }

  fn infer_sub_expr<FNode>(&self, sp: Span, sub: Box<AExpression>,
    make_node: FNode, ty: ExprTy) -> Box<Expression>
   where FNode: Fn(Box<Expression>) -> ExpressionNode
  {
    let node = self.infer_expr_type(sub);
    box Expression::new(sp, make_node(node), ty)
  }

  fn infer_list_expr(&self, subs: Vec<Box<AExpression>>)
    -> Vec<Box<Expression>>
  {
    subs.into_iter()
      .map(|sub| self.infer_expr_type(sub))
      .collect()
  }

  fn infer_tuple_expr(&self, sp: Span, subs: Vec<Box<AExpression>>) -> Box<Expression>
  {
    let nodes = self.infer_list_expr(subs);
    if nodes.len() == 1 {
      nodes.into_iter().next().unwrap()
    } else {
      let tys:Vec<usize> = FromIterator::from_iter(0..nodes.len());
      box Expression::new(sp, Sequence(nodes), Tuple(tys))
    }
  }

  fn infer_choice_expr(&self, sp: Span, subs: Vec<Box<AExpression>>) -> Box<Expression>
  {
    let nodes = self.infer_list_expr(subs);
    box Expression::new(sp, Choice(nodes), Identity)
  }

  fn infer_semantic_action(&self, sp: Span, expr: Box<AExpression>, action_name: Ident) -> Box<Expression>
  {
    let sub_expr = self.infer_expr_type(expr);
    let action_ty = match &self.grammar.rust_items.get(&action_name).unwrap().node {
      &rust::Item_::ItemFn(ref decl, _,_,_,_,_) => decl.output.clone(),
      _ => panic!("Only function items are currently allowed.")
    };
    box Expression::new(sp, SemanticAction(sub_expr, action_name), Action(action_ty))
  }
}