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

pub use middle::attribute::ast::{Expression_, Expression, CharacterInterval, CharacterClassExpr};
pub use middle::attribute::ast::{
  StrLiteral, AnySingleChar, NonTerminalSymbol, Sequence,
  Choice, ZeroOrMore, OneOrMore, Optional, NotPredicate,
  AndPredicate, CharacterClass};

pub use middle::attribute::attribute::*;

pub use rust::{ExtCtxt, Span, Spanned, SpannedIdent};
pub use std::collections::hashmap::HashMap;
pub use identifier::*;

pub use AGrammar = middle::attribute::ast::Grammar;
pub use ARule = middle::attribute::ast::Rule;

pub struct Grammar{
  pub name: Ident,
  pub rules: HashMap<Ident, Rule>,
  pub attributes: GrammarAttributes
}

pub struct Rule{
  pub name: SpannedIdent,
  pub ty: RuleType,
  pub def: Box<Expression>,
}

pub enum RuleType
{
  InlineTy(Box<ExpressionType>),
  NewTy(Box<NamedExpressionType>)
}

pub enum ExpressionType
{
  Character,
  Unit,
  UnitPropagate,
  RuleTypePlaceholder(Ident),
  Vector(Box<ExpressionType>),
  Tuple(Vec<Box<ExpressionType>>),
  OptionalTy(Box<ExpressionType>),
}

impl ExpressionType
{
  fn map(self, f: |ExpressionType| -> ExpressionType) -> ExpressionType
  {
    match self {
      UnitPropagate => UnitPropagate,
      expr => f(expr)
    }
  }

  fn is_unit(&self) -> bool
  {
    match self {
      &UnitPropagate => true,
      &Unit => true,
      _ => false
    }
  }

  fn is_type_ph(&self) -> bool
  {
    match self {
      &RuleTypePlaceholder(_) => true,
      _ => false
    }
  }

  fn ph_ident(&self) -> Ident
  {
    match self {
      &RuleTypePlaceholder(ref ident) => ident.clone(),
      _ => fail!("Cannot extract ident of `RuleTypePlaceholder` from `ExpressionType`.")
    }
  }
}

pub enum NamedExpressionType
{
  Struct(String, Vec<(String, Box<ExpressionType>)>),
  StructTuple(String, Vec<Box<ExpressionType>>),
  Sum(String, Vec<(String, Box<ExpressionType>)>),
  TypeAlias(String, Box<ExpressionType>)
}

pub fn grammar_typing(cx: &ExtCtxt, agrammar: AGrammar) -> Option<Grammar>
{
  let mut grammar = Grammar {
    name: agrammar.name,
    rules: HashMap::with_capacity(agrammar.rules.len()),
    attributes: agrammar.attributes
  };
  type_of_rules(cx, &mut grammar, agrammar.rules);
  Some(grammar)
}

pub fn type_of_rules(cx: &ExtCtxt, grammar: &mut Grammar, arules: HashMap<Ident, ARule>)
{
  for (id, rule) in arules.move_iter() {
    let rule_type = type_of_rule(cx, &rule);
    let typed_rule = Rule{
      name: rule.name,
      ty: rule_type,
      def: rule.def
    };
    grammar.rules.insert(id, typed_rule);
  }
}

fn type_of_rule(cx: &ExtCtxt, rule: &ARule) -> RuleType
{
  match rule.attributes.ty.style.clone() {
    New => named_type_of_rule(cx, rule),
    Inline(_) => InlineTy(type_of_expr(cx, &rule.def)),
    Invisible(_) => InlineTy(box UnitPropagate)
  }
}

fn named_type_of_rule(cx: &ExtCtxt, rule: &ARule) -> RuleType
{
  match &rule.def.node {
    &Choice(ref expr) => NewTy(named_choice_of_rule(cx, rule, expr)),
    &Sequence(_) => named_sequence_of_rule(cx, rule),
    _ => type_alias_of_rule(cx, rule, type_of_expr(cx, &rule.def))
  }
}

fn named_choice_of_rule(cx: &ExtCtxt, rule: &ARule, exprs: &Vec<Box<Expression>>) -> Box<NamedExpressionType>
{
  let mut branches = vec![];
  for expr in exprs.iter() {
    let ty = type_of_expr(cx, expr);
    match ty {
      box RuleTypePlaceholder(ident) => 
        branches.push((name_of_sum(ident.clone()), box RuleTypePlaceholder(ident))),
      _ => {
        cx.span_err(expr.span.clone(), "Missing name of this expression. Name is \
          needed to build the AST of the current choice statement.");
      }
    }
  }
  box Sum(name_of_sum(rule.name.node), branches)
}

fn name_of_sum(ident: Ident) -> String
{
  id_to_camel_case(ident)
}

fn named_sequence_of_rule(cx: &ExtCtxt, rule: &ARule) -> RuleType
{
  let ty = type_of_expr(cx, &rule.def);
  match *ty {
    Tuple(tys) => NewTy(named_seq_tuple_of_rule(cx, rule, tys)),
    Unit => InlineTy(box Unit),
    UnitPropagate => InlineTy(box UnitPropagate),
    _ => type_alias_of_rule(cx, rule, ty)
  }
}

fn named_seq_tuple_of_rule(cx: &ExtCtxt, rule: &ARule,
  tys: Vec<Box<ExpressionType>>) -> Box<NamedExpressionType>
{
  if tys.iter().all(|ty| ty.is_type_ph()) {
    let names_tys = tys.move_iter()
      .map(|ty| (id_to_snake_case(ty.ph_ident()), ty))
      .collect();
    box Struct(type_name_of_rule(rule), names_tys)
  } else {
    box StructTuple(type_name_of_rule(rule), tys)
  }
}

fn type_alias_of_rule(cx: &ExtCtxt, rule: &ARule, ty: Box<ExpressionType>) -> RuleType
{
  NewTy(box TypeAlias(type_name_of_rule(rule), ty))
}

fn type_name_of_rule(rule: &ARule) -> String
{
  id_to_camel_case(rule.name.node.clone())
}

fn type_of_expr(cx: &ExtCtxt, expr: &Box<Expression>) -> Box<ExpressionType>
{
  let span = expr.span.clone();
  match &expr.node {
    &AnySingleChar |
    &CharacterClass(_) => box Character,
    &StrLiteral(_) |
    &NotPredicate(_) |
    &AndPredicate(_) => box Unit,
    &NonTerminalSymbol(ref ident) => box RuleTypePlaceholder(ident.clone()),
    &ZeroOrMore(ref expr) |
    &OneOrMore(ref expr) => box type_of_expr(cx, expr).map(|ty| Vector(box ty)),
    &Optional(ref expr) => box type_of_expr(cx, expr).map(|ty| OptionalTy(box ty)),
    &Sequence(ref expr) => type_of_sequence(cx, expr),
    &Choice(ref expr) => type_of_choice(cx, span, expr)
  }
}

fn type_of_sequence(cx: &ExtCtxt, exprs: &Vec<Box<Expression>>) -> Box<ExpressionType>
{
  let mut tys: Vec<Box<ExpressionType>> = exprs.iter()
    .map(|expr| type_of_expr(cx, expr))
    .filter(|ty| !ty.is_unit())
    .collect();
  
  if tys.is_empty() {
    box Unit
  } else if tys.len() == 1 {
    tys.pop().unwrap()
  } else {
    box Tuple(tys)
  }
}

fn type_of_choice(cx: &ExtCtxt, span: Span, _exprs: &Vec<Box<Expression>>) -> Box<ExpressionType>
{
  cx.span_err(span, "Choice statement type required but the name of the type and constructors \
    cannot be inferred from the context. Use the attribute `type_name` or move this expression in \
    a dedicated rule.");
  box Unit
}

// fn type_of_choice_expr(&self, exprs: &Vec<Box<Expression>>) -> Option<Box<ExpressionType>>
// {
//   fn flatten_tuple(ty: Box<ExpressionType>) -> Vec<Box<ExpressionType>>
//   {
//     match ty {
//       box Tuple(tys) => tys,
//       _ => vec![ty]
//     }
//   };

//   let ty = exprs.iter()
//     .map(|expr| self.type_of_expr(expr))
//     .map(|ty| ty.map_or(vec![], flatten_tuple))
//     .map(|tys| box SumBranch(tys))
//     .collect();

//   Some(box Sum(ty))
// }
