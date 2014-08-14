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
  pub ty: RuleTyping,
  pub def: Box<Expression>,
}

pub enum RuleTyping
{
  Typed(RuleType),
  UnTyped,
  Both(RuleType)
}

pub enum RuleType
{
  Inline(ExpressionType),
  New(NamedExpressionType)
}

pub enum ExpressionType
{
  Character,
  RuleTypePlaceholder(Ident),
  Vector(Box<ExpressionType>),
  Tuple(Vec<Box<ExpressionType>>),
  OptionalTy(Box<ExpressionType>),
}

pub enum NamedExpressionType
{
  Struct(String, Vec<(String, Box<ExpressionType>)>),
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
  grammar.rules = arules.move_iter().map(|(id, rule)| (id, Rule{
    name: rule.name,
    ty: UnTyped,
    def: rule.def
  })).collect();
}

  // fn type_of_rule(&self, rule: &Rule) -> Option<Box<ExpressionType>>
  // {
  //   self.type_of_expr(&rule.def)
  // }

  // fn type_of_expr(&self, expr: &Box<Expression>) -> Option<Box<ExpressionType>>
  // {
  //   match &expr.node {
  //     &StrLiteral(_) |
  //     &AnySingleChar |
  //     &NotPredicate(_) |
  //     &AndPredicate(_) => None,
  //     &NonTerminalSymbol(ident) => Some(box RuleTypePlaceholder(ident)),
  //     &CharacterClass(_) => Some(box Character),
  //     &Sequence(ref expr) => self.type_of_seq_expr(expr),
  //     &Choice(ref expr) => self.type_of_choice_expr(expr),
  //     &ZeroOrMore(ref expr) |
  //     &OneOrMore(ref expr) => self.type_of_expr(expr).map(|r| box Vector(r)),
  //     &Optional(ref expr) => self.type_of_expr(expr).map(|r| box OptionalTy(r))
  //   }
  // }

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

  // fn type_of_seq_expr(&self, exprs: &Vec<Box<Expression>>) -> Option<Box<ExpressionType>>
  // {
  //   let tys: Vec<Box<ExpressionType>> = exprs.iter()
  //     .filter_map(|expr| self.type_of_expr(expr))
  //     .collect();
    
  //   if tys.is_empty() {
  //     None
  //   } else {
  //     Some(box Tuple(tys))
  //   }
  // }