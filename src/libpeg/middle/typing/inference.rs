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

use middle::typing::visitor::*;
use middle::typing::ast::*;
pub use middle::attribute::ast::Grammar as AGrammar;
pub use middle::attribute::ast::Rule as ARule;
pub use middle::attribute::ast::Expression as AExpression;

pub fn infer_rules_type(cx: &ExtCtxt, grammar: &mut Grammar, arules: HashMap<Ident, ARule>)
{
  for (id, rule) in arules.move_iter() {
    let typed_rule = infer_rule_type(cx, rule);
    grammar.rules.insert(id, typed_rule);
  }
}

fn infer_rule_type(cx: &ExtCtxt, rule: ARule) -> Rule
{
  let expr = infer_expr_type(cx, rule.def);
  Rule{
    name: rule.name,
    def: expr,
    attributes: rule.attributes
  }
}

fn infer_expr_type(cx: &ExtCtxt, expr: Box<AExpression>) -> Box<Expression>
{
  let sp = expr.span.clone();
  match expr.node {
    AnySingleChar => infer_char_expr(sp, AnySingleChar),
    CharacterClass(c) => infer_char_expr(sp, CharacterClass(c)),
    StrLiteral(s) => infer_unit_expr(sp, StrLiteral(s)),
    NotPredicate(sub) => infer_sub_unit_expr(cx, sp, sub, |e| NotPredicate(e)),
    AndPredicate(sub) => infer_sub_unit_expr(cx, sp, sub, |e| AndPredicate(e)),
    NonTerminalSymbol(ident) => infer_rule_type_ph(sp, ident),
    ZeroOrMore(sub) => infer_sub_expr(cx, sp, sub, |e| ZeroOrMore(e), |ty| Vector(ty)),
    OneOrMore(sub) => infer_sub_expr(cx, sp, sub, |e| OneOrMore(e), |ty| Vector(ty)),
    Optional(sub) =>  infer_sub_expr(cx, sp, sub, |e| Optional(e), |ty| OptionalTy(ty)),
    Sequence(sub) => infer_tuple_expr(cx, sp, sub),
    Choice(sub) => type_of_choice(cx, sp, sub)
  }
}

fn infer_char_expr(sp: Span, node: ExpressionNode) -> Box<Expression>
{
  box Expression {
    span: sp,
    node: node,
    ty: make_pty(Character)
  }
}

fn infer_unit_expr(sp: Span, node: ExpressionNode) -> Box<Expression>
{
  box Expression {
    span: sp,
    node: node,
    ty: make_pty(Unit)
  }
}

fn infer_sub_unit_expr(cx: &ExtCtxt, sp: Span, sub: Box<AExpression>,
  make_node: |Box<Expression>| -> ExpressionNode) -> Box<Expression>
{
  infer_unit_expr(sp, make_node(infer_expr_type(cx, sub)))
}

fn infer_rule_type_ph(sp: Span, ident: Ident) -> Box<Expression>
{
  box Expression {
    span: sp,
    node: NonTerminalSymbol(ident.clone()),
    ty: make_pty(RuleTypePlaceholder(ident))
  }
}

fn infer_sub_expr(cx: &ExtCtxt, sp: Span, sub: Box<AExpression>,
  make_node: |Box<Expression>| -> ExpressionNode,
  make_type: |PTy| -> ExpressionType) -> Box<Expression>
{
  let node = infer_expr_type(cx, sub);
  let ty = node.ty.clone();
  box Expression {
    span: sp,
    node: make_node(node),
    ty: make_pty(make_type(ty))
  }
}

fn infer_list_expr(cx: &ExtCtxt, subs: Vec<Box<AExpression>>) 
  -> (Vec<Box<Expression>>, Vec<PTy>)
{
  let nodes : Vec<Box<Expression>> = subs.move_iter()
    .map(|sub| infer_expr_type(cx, sub))
    .collect();
  let tys = nodes.iter()
    .map(|node| node.ty.clone())
    .collect();
  (nodes, tys)
}

fn infer_tuple_expr(cx: &ExtCtxt, sp: Span, subs: Vec<Box<AExpression>>) -> Box<Expression>
{
  let (nodes, tys) = infer_list_expr(cx, subs);
  if nodes.len() == 1 {
    nodes.move_iter().next().unwrap()
  } else {
    box Expression {
      span: sp,
      node: Sequence(nodes),
      ty: make_pty(Tuple(tys))
    }
  }
}

fn type_of_choice(cx: &ExtCtxt, sp: Span, subs: Vec<Box<AExpression>>) -> Box<Expression>
{
  // cx.span_err(span, "Choice statement type required but the name of the type and constructors \
  //   cannot be inferred from the context. Use the attribute `type_name` or move this expression in \
  //   a dedicated rule.");
  let (nodes, tys) = infer_list_expr(cx, subs);

  box Expression {
    span: sp,
    node: Choice(nodes),
    ty: make_pty(UnnamedSum(tys))
  }
}
