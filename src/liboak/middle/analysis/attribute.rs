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

use middle::analysis::ast::*;
use front::ast::Grammar as FGrammar;
use front::ast::Rule as FRule;

use rust::{P, MetaItemKind, MetaItem};

pub fn decorate_with_attributes(cx: &ExtCtxt, fgrammar: &FGrammar,
  mut grammar: Grammar) -> Partial<Grammar>
{
  check_rules_attributes(cx, &fgrammar.rules);
  let print_attr = check_grammar_attributes(cx, &fgrammar.attributes);
  grammar.attributes = GrammarAttributes::new(print_attr);
  Partial::Value(grammar)
}

fn check_grammar_attributes(cx: &ExtCtxt, attrs: &Vec<Attribute>) -> PrintAttribute {
  let mut print_attr = PrintAttribute::Nothing;
  for attr in attrs {
    let meta_item = attr.node.value.clone();
    print_attr = print_attr.merge(check_grammar_attr(cx, meta_item));
  }
  print_attr
}

fn check_grammar_attr(cx: &ExtCtxt, meta_item: P<MetaItem>) -> PrintAttribute {
  match &meta_item.node {
    &MetaItemKind::Word(ref name) if *name == "debug_api" => {
      PrintAttribute::DebugApi
    },
    &MetaItemKind::Word(ref name) if *name == "show_api" => {
      PrintAttribute::ShowApi
    },
      &MetaItemKind::Word(ref name)
    | &MetaItemKind::List(ref name, _)
    | &MetaItemKind::NameValue(ref name, _) => {
      cx.parse_sess.span_diagnostic.warn(
        format!("Unknown attribute `{}`: it will be ignored.", name).as_str());
      PrintAttribute::Nothing
    }
  }
}

fn check_rules_attributes(cx: &ExtCtxt, rules: &Vec<FRule>) {
  for rule in rules {
    for attr in &rule.attributes {
      let meta_item = attr.node.value.clone();
      check_rule_attr(cx, rule.name.node, meta_item);
    }
  }
}

fn check_rule_attr(cx: &ExtCtxt, rule_name: Ident, meta_item: P<MetaItem>) {
  match &meta_item.node {
      &MetaItemKind::Word(ref name)
    | &MetaItemKind::List(ref name, _)
    | &MetaItemKind::NameValue(ref name, _) => {
      cx.parse_sess.span_diagnostic.warn(
        format!("Unknown attribute `{}` attached to the rule `{}`: it will be ignored.", name, rule_name).as_str());
      }
  }
}
