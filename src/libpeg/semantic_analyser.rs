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

use front::ast::*;
use utility::*;

use syntax::ext::base::ExtCtxt;
use syntax::codemap::Span;
use std::collections::hashmap::HashMap;

pub mod clean_ast
{
  use syntax::ast::Ident;
  use front::ast::*;

  pub struct Grammar{
    pub name: Ident,
    pub rules: Vec<Rule>,
    pub start_rule_idx: uint,
    pub print_generated: bool
  }

  pub struct Rule{
    pub name: Ident,
    pub def: Box<Expression>
  }
}

pub struct SemanticAnalyser<'a>
{
  cx: &'a ExtCtxt<'a>,
  grammar: &'a Peg
}

impl<'a> SemanticAnalyser<'a>
{
  pub fn analyse(cx: &'a ExtCtxt, grammar: &'a Peg) -> Option<clean_ast::Grammar>
  {
    let analyser = SemanticAnalyser {
      cx: cx,
      grammar: grammar
    };
    analyser.check()
  }
  
  fn check(&self) -> Option<clean_ast::Grammar>
  {
    if !self.at_least_one_rule_declared() {
      return None
    }

    let mut start_rule_idx = self.start_rule();

    let mut ident_to_rule_idx = HashMap::new();
    if self.multiple_rule_declaration(&mut ident_to_rule_idx) {
      return None
    }

    if UndeclaredRule::analyse(self.cx, &self.grammar.rules, &ident_to_rule_idx) {
      return None
    }

    let mut unused_rule_analyser = UnusedRule::new(self.cx, &self.grammar.rules, 
      &ident_to_rule_idx);
    unused_rule_analyser.analyse(start_rule_idx);

    let mut rules = vec![];
    for (idx, rule) in self.grammar.rules.iter().enumerate() {
      if unused_rule_analyser.is_used[idx] {
        rules.push(clean_ast::Rule{
          name: rule.name.node,
          def: rule.def.clone()
        });
        if idx == start_rule_idx {
          start_rule_idx = rules.len() - 1;
        }
      }
    }
    Some(clean_ast::Grammar{
      name: self.grammar.name,
      rules: rules,
      start_rule_idx: start_rule_idx,
      print_generated: get_attribute(&self.grammar.attributes, "print_generated").is_some()
    })
  }

  fn at_least_one_rule_declared(&self) -> bool
  {
    if self.grammar.rules.len() == 0 {
      self.cx.parse_sess.span_diagnostic.handler.err(
        "At least one rule must be declared.");
      false
    } else {
      true
    }
  }

  fn start_rule(&self) -> uint
  {
    let mut start_rule_idx = None;
    for (idx, rule) in self.grammar.rules.iter().enumerate() {
      if self.can_be_start_attr(start_rule_idx, rule) {
          start_rule_idx = Some(idx);
      }
    }
    match start_rule_idx {
      None => {
        self.cx.parse_sess.span_diagnostic.handler.warn(
          "No rule has been specified as the starting point (attribute `#[start]`). \
          The first rule will be automatically considered as such.");
        0
      },
      Some(idx) => idx
    }
  }

  fn can_be_start_attr(&self, start_rule_idx: Option<uint>, rule: &Rule) -> bool
  {
    match (start_rule_idx, get_attribute(&rule.attributes, "start")) {
      (Some(idx), Some(attr)) => {
        self.span_err(attr.span, format!(
          "Multiple `start` attributes are forbidden. \
          Rules `{}` and `{}` conflict.",
          id_to_string(self.grammar.rules[idx].name.node),
          id_to_string(rule.name.node)).as_slice());
        false
      },
      (None, Some(_)) => true,
      _ => false
    }
  }

  fn multiple_rule_declaration(&self, ident_to_rule_idx: &mut HashMap<Ident, uint>) -> bool
  {
    let mut multiple_decl = false;
    for (idx, rule) in self.grammar.rules.iter().enumerate() {
      let first_rule_def = ident_to_rule_idx.find_copy(&rule.name.node);
      match first_rule_def {
        Some(first_rule_idx) => {
          self.span_err(rule.name.span,
            format!("duplicate definition of rule `{}`", 
              id_to_string(rule.name.node)).as_slice());
          let first_rule_name = self.grammar.rules[first_rule_idx].name;
          self.span_note(first_rule_name.span,
            format!("first definition of rule `{}` here",
              id_to_string(first_rule_name.node)).as_slice());
          multiple_decl = true;
        }
        None => { ident_to_rule_idx.insert(rule.name.node, idx); }
      }
    }
    multiple_decl
  }

  fn span_err(&self, sp: Span, m: &str) 
  {
    self.cx.parse_sess.span_diagnostic.span_err(sp, m);
  }

  fn span_note(&self, sp: Span, m: &str) 
  {
    self.cx.parse_sess.span_diagnostic.span_note(sp, m);
  }
}

pub trait ExprVisitor
{
  fn visit_expr(&mut self, expr: &Box<Expression>)
  {
    let sp = expr.span;
    match &expr.node {
      &StrLiteral(ref lit) => {
        self.visit_str_literal(sp, lit)
      }
      &AnySingleChar => {
        self.visit_any_single_char(sp)
      }
      &NonTerminalSymbol(id) => {
        self.visit_non_terminal_symbol(sp, id)
      }
      &Sequence(ref seq) => {
        self.visit_sequence(sp, seq)
      }
      &Choice(ref choices) => {
        self.visit_choice(sp, choices)
      }
      &ZeroOrMore(ref expr) => {
        self.visit_zero_or_more(sp, expr)
      }
      &OneOrMore(ref expr) => {
        self.visit_one_or_more(sp, expr)
      }
      &Optional(ref expr) => {
        self.visit_optional(sp, expr)
      }
      &NotPredicate(ref expr) => {
        self.visit_not_predicate(sp, expr)
      }
      &AndPredicate(ref expr) => {
        self.visit_and_predicate(sp, expr)
      }
      &CharacterClass(ref charClass) => {
        self.visit_character_class(sp, charClass)
      }
    }
  }

  fn visit_str_literal(&mut self, _sp: Span, _lit: &String) {}
  fn visit_any_single_char(&mut self, _sp: Span) {}
  fn visit_non_terminal_symbol(&mut self, _sp: Span, _id: Ident) {}

  fn visit_sequence(&mut self, _sp: Span, expr: &Vec<Box<Expression>>)
  {
    self.visit_expr_slice(expr.as_slice())
  }

  fn visit_choice(&mut self, _sp: Span, expr: &Vec<Box<Expression>>)
  {
    self.visit_expr_slice(expr.as_slice())
  }

  fn visit_zero_or_more(&mut self, _sp: Span, expr: &Box<Expression>)
  {
    self.visit_expr(expr)
  }

  fn visit_one_or_more(&mut self, _sp: Span, expr: &Box<Expression>)
  {
    self.visit_expr(expr)
  }

  fn visit_optional(&mut self, _sp: Span, expr: &Box<Expression>)
  {
    self.visit_expr(expr)
  }

  fn visit_not_predicate(&mut self, _sp: Span, expr: &Box<Expression>)
  {
    self.visit_expr(expr)
  }

  fn visit_and_predicate(&mut self, _sp: Span, expr: &Box<Expression>)
  {
    self.visit_expr(expr)
  }

  fn visit_character_class(&mut self, _sp: Span, _expr: &CharacterClassExpr) {}

  fn visit_expr_slice<'a>(&mut self, seq: &'a [Box<Expression>])
  {
    assert!(seq.len() > 0);
    for expr in seq.iter() {
      self.visit_expr(expr);
    }
  }
}

struct UndeclaredRule<'a>
{
  cx: &'a ExtCtxt<'a>,
  ident_to_rule_idx: &'a HashMap<Ident, uint>,
  has_undeclared: bool
}

impl<'a> UndeclaredRule<'a>
{
  fn analyse(cx: &'a ExtCtxt<'a>, rules: &Vec<Rule>,
    ident_to_rule_idx: &'a HashMap<Ident, uint>) -> bool
  {
    let mut analyser = UndeclaredRule {
      cx: cx,
      ident_to_rule_idx: ident_to_rule_idx,
      has_undeclared: false
    };
    for rule in rules.iter() {
      analyser.visit_expr(&rule.def);
    }
    analyser.has_undeclared
  }
}

impl<'a> ExprVisitor for UndeclaredRule<'a>
{
  fn visit_non_terminal_symbol(&mut self, sp: Span, id: Ident)
  {
    if self.ident_to_rule_idx.find(&id).is_none() {
      self.cx.parse_sess.span_diagnostic.span_err(sp, 
        format!("You try to call the rule `{}` which is not declared.",
          id_to_string(id)).as_slice());
      self.has_undeclared = true;
    }
  }
}

struct UnusedRule<'a>
{
  cx: &'a ExtCtxt<'a>,
  ident_to_rule_idx: &'a HashMap<Ident, uint>,
  rules: &'a Vec<Rule>,
  is_used: Vec<bool>
}

impl<'a> UnusedRule<'a>
{
  fn new(cx: &'a ExtCtxt<'a>, rules: &'a Vec<Rule>,
    ident_to_rule_idx: &'a HashMap<Ident, uint>) -> UnusedRule<'a>
  {
    UnusedRule {
      cx: cx,
      ident_to_rule_idx: ident_to_rule_idx,
      rules: rules,
      is_used: Vec::from_fn(rules.len(), |_| false)
    }
  }

  fn analyse(&mut self, start_rule_idx: uint)
  {
    *self.is_used.get_mut(start_rule_idx) = true;
    self.visit_expr(&self.rules[start_rule_idx].def);
    for (idx, used) in self.is_used.iter().enumerate() {
      if !used {
        let sp = self.rules[idx].name.span;
        self.cx.parse_sess.span_diagnostic.span_warn(sp, 
          format!("The rule `{}` is not used.",
            id_to_string(self.rules[idx].name.node)).as_slice());
      }
    }
  }
}

impl<'a> ExprVisitor for UnusedRule<'a>
{
  fn visit_non_terminal_symbol(&mut self, _sp: Span, id: Ident)
  {
    let idx = *self.ident_to_rule_idx.find(&id).unwrap();
    if !self.is_used[idx] {
      *self.is_used.get_mut(idx) = true;
      self.visit_expr(&self.rules[idx].def);
    }
  }
}
