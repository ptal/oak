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
use middle::typing::inlining::*;
pub use middle::attribute::ast::Grammar as AGrammar;
pub use middle::attribute::ast::Rule as ARule;
pub use middle::attribute::ast::Expression as AExpression;

pub fn grammar_typing(cx: &ExtCtxt, agrammar: AGrammar) -> Option<Grammar>
{
  let mut grammar = Grammar {
    name: agrammar.name,
    rules: HashMap::with_capacity(agrammar.rules.len()),
    named_types: HashMap::with_capacity(agrammar.rules.len()),
    attributes: agrammar.attributes
  };
  infer_rules_type(cx, &mut grammar, agrammar.rules);
  inlining_phase(cx, &mut grammar);
  Some(grammar)
}

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
  // match rule.attributes.ty.style {
  //   New => infer_new_rule_type(cx, rule.name, expr),
  //   Inline(_) => infer_inline_rule_type(rule.name, expr),
  //   Invisible(_) => infer_invisible_rule_type(rule.name, expr)
  // }
}

// fn infer_inline_rule_type(name: SpannedIdent, expr: Box<Expression>) -> Rule
// {
//   let ty = expr.ty.clone();
//   Rule{
//     name: name,
//     def: expr,
//     ty: InlineTy(ty)
//   }
// }

// fn infer_invisible_rule_type(name: SpannedIdent, expr: Box<Expression>) -> Rule
// {
//   Rule{
//     name: name,
//     def: expr,
//     ty: InlineTy(Rc::new(UnitPropagate))
//   }
// }

// fn infer_new_rule_type(cx: &ExtCtxt, name: SpannedIdent, expr: Box<Expression>) -> Rule
// {
//   let rule_ty = infer_rule_type_structure(cx, name.node.clone(), &expr);
//   Rule{
//     name: name,
//     def: expr,
//     ty: rule_ty
//   }
// }

// fn infer_rule_type_structure(cx: &ExtCtxt, rule_name: Ident, expr: &Box<Expression>) -> RuleType
// {
//   match &expr.node {
//     &Choice(ref expr) => named_choice_of_rule(cx, rule_name, expr),
//     &Sequence(_) => named_sequence_of_rule(rule_name, &expr.ty),
//     _ => type_alias_of_rule(rule_name, expr.ty.clone())
//   }
// }

// fn named_choice_of_rule(cx: &ExtCtxt, rule_name: Ident, exprs: &Vec<Box<Expression>>) -> RuleType
// {
//   let mut branches = vec![];
//   for expr in exprs.iter() {
//     let ty = expr.ty.clone();
//     match &*expr.ty {
//       &RuleTypePlaceholder(ref ident) |
//       &RuleTypeName(ref ident) => 
//         branches.push((name_of_sum(ident.clone()), ty)),
//       e => {
//         cx.span_err(expr.span.clone(), format!("{}, Name missing from this expression. Name is \
//           needed to build the AST of the current choice statement.", e).as_slice());
//       }
//     }
//   }
//   NewTy(box Sum(name_of_sum(rule_name), branches))
// }

// fn name_of_sum(ident: Ident) -> String
// {
//   id_to_camel_case(ident)
// }

// fn named_sequence_of_rule(rule_name: Ident, ty: &Rc<ExpressionType>) -> RuleType
// {
//   match &**ty {
//     &Tuple(ref tys) => NewTy(named_seq_tuple_of_rule(rule_name, tys)),
//     &Unit => InlineTy(Rc::new(Unit)),
//     &UnitPropagate => InlineTy(Rc::new(UnitPropagate)),
//     _ => type_alias_of_rule(rule_name, ty.clone())
//   }
// }

// fn named_seq_tuple_of_rule(rule_name: Ident,
//   tys: &Vec<Rc<ExpressionType>>) -> Box<NamedExpressionType>
// {
//   if tys.iter().all(|ty| ty.is_type_ph()) {
//     let names_tys = tys.iter()
//       .map(|ty| (id_to_snake_case(ty.ph_ident()), ty.clone()))
//       .collect();
//     box Struct(type_name_of_rule(rule_name), names_tys)
//   } else {
//     box StructTuple(type_name_of_rule(rule_name), tys.clone())
//   }
// }

// fn type_alias_of_rule(rule_name: Ident, ty: Rc<ExpressionType>) -> RuleType
// {
//   NewTy(box TypeAlias(type_name_of_rule(rule_name), ty))
// }

// fn type_name_of_rule(rule_name: Ident) -> String
// {
//   id_to_camel_case(rule_name)
// }

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
    ty: Rc::new(Character)
  }
}

fn infer_unit_expr(sp: Span, node: ExpressionNode) -> Box<Expression>
{
  box Expression {
    span: sp,
    node: node,
    ty: Rc::new(Unit)
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
    ty: Rc::new(RuleTypePlaceholder(ident))
  }
}

fn infer_sub_expr(cx: &ExtCtxt, sp: Span, sub: Box<AExpression>,
  make_node: |Box<Expression>| -> ExpressionNode,
  make_type: |Rc<ExpressionType>| -> ExpressionType) -> Box<Expression>
{
  let node = infer_expr_type(cx, sub);
  let ty = node.ty.clone();
  box Expression {
    span: sp,
    node: make_node(node),
    ty: Rc::new(make_type(ty))
  }
}

fn infer_list_expr(cx: &ExtCtxt, subs: Vec<Box<AExpression>>) 
  -> (Vec<Box<Expression>>, Vec<Rc<ExpressionType>>)
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
      ty: Rc::new(Tuple(tys))
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
    ty: Rc::new(UnnamedSum(tys))
  }
}
