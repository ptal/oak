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

use rust::{ExtCtxt, Ident, Span};
use front::ast::*;
use middle::attribute::{CodePrinterBuilder, CodeGenerationBuilder, StartRuleBuilder};
use middle::visitor::Visitor;
use std::collections::hashmap::HashMap;
use std::iter::count;

mod lint;
mod visitor;
mod attribute;

pub mod clean_ast
{
  use rust::Ident;
  use front::ast::*;
  use middle::attribute::{CodeGeneration, CodePrinter};

  pub struct Grammar{
    pub name: Ident,
    pub rules: Vec<Rule>,
    pub start_rule_idx: uint,
    pub code_printer: CodePrinter,
    pub code_gen: CodeGeneration
  }

  pub struct Rule{
    pub name: Ident,
    pub def: Box<Expression>
  }
}

pub struct SemanticAnalyser<'a>
{
  cx: &'a ExtCtxt<'a>,
  grammar: &'a Grammar
}

impl<'a> SemanticAnalyser<'a>
{
  pub fn analyse(cx: &'a ExtCtxt, grammar: &'a Grammar) -> Option<clean_ast::Grammar>
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

    let mut start_rule_builder = StartRuleBuilder::new(self.cx);

    let _rules_attrs : Vec<&Attribute> = self.grammar.rules.iter().enumerate()
      .flat_map(|(idx, r)| r.attributes.iter().zip(count(idx, 0)))
      .filter(|&(a, idx)| start_rule_builder.from_attr(idx, a))
      .map(|(a, _)| a)
      .collect();

    let mut start_rule_idx = start_rule_builder.build();

    let mut ident_to_rule_idx = HashMap::new();
    if self.multiple_rule_declaration(&mut ident_to_rule_idx) {
      return None
    }

    if UndeclaredRule::analyse(self.cx, self.grammar, &ident_to_rule_idx) {
      return None
    }

    let mut unused_rule_analyser = lint::unused_rule::UnusedRule::new(self.cx, self.grammar, 
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
    let mut code_printer_builder = CodePrinterBuilder::new(self.cx);
    let mut code_gen_builder = CodeGenerationBuilder::new(self.cx);
    let _attr : Vec<&Attribute> = self.grammar.attributes.iter()
      .filter(|a| code_printer_builder.from_attr(*a))
      .filter(|a| code_gen_builder.from_attr(*a))
      .collect();
    Some(clean_ast::Grammar{
      name: self.grammar.name,
      rules: rules,
      start_rule_idx: start_rule_idx,
      code_printer: code_printer_builder.build(),
      code_gen: code_gen_builder.build()
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

struct UndeclaredRule<'a>
{
  cx: &'a ExtCtxt<'a>,
  ident_to_rule_idx: &'a HashMap<Ident, uint>,
  has_undeclared: bool
}

impl<'a> UndeclaredRule<'a>
{
  fn analyse(cx: &'a ExtCtxt<'a>, grammar: &Grammar,
    ident_to_rule_idx: &'a HashMap<Ident, uint>) -> bool
  {
    let mut analyser = UndeclaredRule {
      cx: cx,
      ident_to_rule_idx: ident_to_rule_idx,
      has_undeclared: false
    };
    analyser.visit_grammar(grammar);
    analyser.has_undeclared
  }
}

impl<'a> Visitor for UndeclaredRule<'a>
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
