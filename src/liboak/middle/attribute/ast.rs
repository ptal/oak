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

pub use front::ast::{Expression_, Expression, CharacterInterval, CharacterClassExpr};
pub use front::ast::Expression_::*;

pub use rust::{ExtCtxt, Span, Spanned, SpannedIdent, Attribute};
pub use identifier::*;
pub use middle::analysis::ast::Grammar as SGrammar;
pub use middle::analysis::ast::Rule as SRule;

use rust;
use rust::{P, MetaItem};
use rust::MetaItem_::*;
use monad::partial::Partial;

pub use std::collections::HashMap;

pub struct Grammar{
  pub name: Ident,
  pub rules: HashMap<Ident, Rule>,
  pub rust_items: HashMap<Ident, rust::P<rust::Item>>,
  pub attributes: GrammarAttributes
}

pub struct Rule{
  pub name: SpannedIdent,
  pub def: Box<Expression>,
}

impl Rule
{
  fn new(name: SpannedIdent, def: Box<Expression>) -> Rule
  {
    Rule{
      name: name,
      def: def
    }
  }
}

pub struct GrammarAttributes
{
  pub code_printer: CodePrinter
}

impl GrammarAttributes {
  pub fn new(code_printer: CodePrinter) -> GrammarAttributes {
    GrammarAttributes {
      code_printer: code_printer
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CodePrinter
{
  DebugApi,
  ShowApi,
  Nothing
}

impl CodePrinter {
  pub fn merge(self, other: CodePrinter) -> CodePrinter {
    use self::CodePrinter::*;
    match (self, other) {
        (Nothing, DebugApi)
      | (ShowApi, DebugApi) => DebugApi,
      (Nothing, ShowApi) => ShowApi,
      _ => Nothing
    }
  }

  pub fn is_debug(self) -> bool {
    self == CodePrinter::DebugApi
  }
}

impl Grammar
{
  pub fn new(cx: &ExtCtxt, sgrammar: SGrammar) -> Partial<Grammar>
  {
    Grammar::check_rules_attributes(cx, &sgrammar.rules);
    let code_printer = Grammar::check_grammar_attributes(cx, &sgrammar.attributes);
    let rules: HashMap<_,_> =
      sgrammar.rules.into_iter()
      .map(|(id, rule)| {
        (id, Rule::new(rule.name, rule.def))
      })
      .collect();

    let grammar = Grammar{
      name: sgrammar.name,
      rules: rules,
      rust_items: sgrammar.rust_items,
      attributes: GrammarAttributes::new(code_printer)
    };
    Partial::Value(grammar)
  }

  fn check_grammar_attributes(cx: &ExtCtxt, attrs: &Vec<Attribute>) -> CodePrinter
  {
    let mut code_printer = CodePrinter::Nothing;
    for attr in attrs {
      let meta_item = attr.node.value.clone();
      code_printer = code_printer.merge(Grammar::check_grammar_attr(cx, meta_item));
    }
    code_printer
  }

  fn check_grammar_attr(cx: &ExtCtxt, meta_item: P<MetaItem>) -> CodePrinter
  {
    match &meta_item.node {
      &MetaWord(ref name) if *name == "debug_api" => {
        CodePrinter::DebugApi
      },
      &MetaWord(ref name) if *name == "show_api" => {
        CodePrinter::ShowApi
      },
        &MetaWord(ref name)
      | &MetaList(ref name, _)
      | &MetaNameValue(ref name, _) => {
        cx.parse_sess.span_diagnostic.handler.warn(
          format!("Unknown attribute `{}`: it will be ignored.", name).as_str());
        CodePrinter::Nothing
      }
    }
  }

  fn check_rules_attributes(cx: &ExtCtxt, rules: &HashMap<Ident, SRule>)
  {
    for (id, rule) in rules {
      for attr in &rule.attributes {
        let meta_item = attr.node.value.clone();
        Grammar::check_rule_attr(cx, *id, meta_item);
      }
    }
  }

  fn check_rule_attr(cx: &ExtCtxt, rule_name: Ident, meta_item: P<MetaItem>)
  {
    match &meta_item.node {
        &MetaWord(ref name)
      | &MetaList(ref name, _)
      | &MetaNameValue(ref name, _) => {
        cx.parse_sess.span_diagnostic.handler.warn(
          format!("Unknown attribute `{}` attached to the rule `{}`: it will be ignored.", name, rule_name).as_str());
        }
    }
  }
}