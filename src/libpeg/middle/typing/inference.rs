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

pub use middle::attribute::ast::Grammar as AGrammar;
pub use middle::attribute::ast::Rule as ARule;
pub use middle::attribute::ast::Expression as AExpression;

use middle::typing::visitor::*;
use middle::typing::ast::ExpressionType::*;
use rust;

pub struct InferenceEngine<'cx, 'r>
{
  cx: &'cx ExtCtxt<'cx>,
  grammar: &'r mut Grammar
}

impl<'cx, 'r> InferenceEngine<'cx, 'r>
{
  pub fn infer(cx: &'cx ExtCtxt, grammar: &'r mut Grammar, arules: HashMap<Ident, ARule>) {
    let mut engine = InferenceEngine {
      cx: cx,
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
      def: expr,
      attributes: rule.attributes
    }
  }

  fn infer_expr_type(&self, expr: Box<AExpression>) -> Box<Expression>
  {
    let sp = expr.span.clone();
    match expr.node {
      AnySingleChar => self.infer_char_expr(sp, AnySingleChar),
      CharacterClass(c) => self.infer_char_expr(sp, CharacterClass(c)),
      StrLiteral(s) => self.infer_unit_expr(sp, StrLiteral(s)),
      NotPredicate(sub) => self.infer_sub_unit_expr(sp, sub, |e| NotPredicate(e)),
      AndPredicate(sub) => self.infer_sub_unit_expr(sp, sub, |e| AndPredicate(e)),
      NonTerminalSymbol(ident) => self.infer_rule_type_ph(sp, ident),
      ZeroOrMore(sub) => self.infer_sub_expr(sp, sub, |e| ZeroOrMore(e), |ty| Vector(ty)),
      OneOrMore(sub) => self.infer_sub_expr(sp, sub, |e| OneOrMore(e), |ty| Vector(ty)),
      Optional(sub) =>  self.infer_sub_expr(sp, sub, |e| Optional(e), |ty| OptionalTy(ty)),
      Sequence(sub) => self.infer_tuple_expr(sp, sub),
      Choice(sub) => self.infer_choice_expr(sp, sub),
      SemanticAction(ref expr, ref id) => self.infer_semantic_action(sp, expr.clone(), id.clone()) // weird bug without ref/clone...
    }
  }

  fn infer_char_expr(&self, sp: Span, node: ExpressionNode) -> Box<Expression>
  {
    box Expression::new(sp, node, make_pty(Character))
  }

  fn infer_unit_expr(&self, sp: Span, node: ExpressionNode) -> Box<Expression>
  {
    box Expression::new(sp, node, make_pty(Unit))
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
      make_pty(RuleTypePlaceholder(ident)))
  }

  fn infer_sub_expr<FNode, FType>(&self, sp: Span, sub: Box<AExpression>,
    make_node: FNode, make_type: FType) -> Box<Expression>
   where FNode: Fn(Box<Expression>) -> ExpressionNode,
         FType: Fn(PTy) -> ExpressionType
  {
    let node = self.infer_expr_type(sub);
    let ty = node.ty.clone();
    box Expression::new(sp, make_node(node), make_pty(make_type(ty)))
  }

  fn infer_list_expr(&self, subs: Vec<Box<AExpression>>)
    -> (Vec<Box<Expression>>, Vec<PTy>)
  {
    let nodes : Vec<Box<Expression>> = subs.into_iter()
      .map(|sub| self.infer_expr_type(sub))
      .collect();
    let tys = nodes.iter()
      .map(|node| node.ty.clone())
      .collect();
    (nodes, tys)
  }

  fn infer_tuple_expr(&self, sp: Span, subs: Vec<Box<AExpression>>) -> Box<Expression>
  {
    let (nodes, tys) = self.infer_list_expr(subs);
    if nodes.len() == 1 {
      nodes.into_iter().next().unwrap()
    } else {
      box Expression::new(sp, Sequence(nodes), make_pty(Tuple(tys)))
    }
  }

  fn infer_choice_expr(&self, sp: Span, subs: Vec<Box<AExpression>>) -> Box<Expression>
  {
    let (nodes, tys) = self.infer_list_expr(subs);
    box Expression::new(sp, Choice(nodes), make_pty(UnnamedSum(tys)))
  }

  fn infer_semantic_action(&self, sp: Span, expr: Box<AExpression>, action_name: Ident) -> Box<Expression>
  {
    let sub_expr = self.infer_expr_type(expr);
    let action_ty = match &self.grammar.rust_items.get(&action_name).unwrap().node {
      &rust::Item_::ItemFn(ref decl, _,_,_,_) => decl.output.clone(),
      _ => panic!("Only function items are currently allowed.")
    };
    box Expression::new(sp, SemanticAction(sub_expr, action_name), make_pty(Action(action_ty)))
  }
}