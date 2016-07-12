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
use front::ast::FRule;

use rust::{P, MetaItemKind, MetaItem};

pub fn decorate_with_attributes<'a, 'b>(mut grammar: AGrammar<'a, 'b>,
  attributes: Vec<Attribute>, frules: Vec<FRule>) -> Partial<AGrammar<'a, 'b>>
{
  check_rules_attributes(&grammar, frules);
  merge_grammar_attributes(&mut grammar, attributes);
  Partial::Value(grammar)
}

fn merge_grammar_attributes<'a, 'b>(grammar: &mut AGrammar<'a, 'b>, attrs: Vec<Attribute>) {
  for attr in attrs {
    let meta_item = attr.node.value;
    merge_grammar_attr(grammar, meta_item);
  }
}

fn merge_grammar_attr<'a, 'b>(grammar: &mut AGrammar<'a, 'b>, meta_item: P<MetaItem>) {
  match &meta_item.node {
    &MetaItemKind::Word(ref name) if *name == "debug_api" => {
      grammar.merge_print_code(PrintLevel::Debug);
    },
    &MetaItemKind::Word(ref name) if *name == "show_api" => {
      grammar.merge_print_code(PrintLevel::Show);
    },
    &MetaItemKind::Word(ref name) if *name == "debug_typing" => {
      grammar.merge_print_typing(PrintLevel::Debug);
    },
    &MetaItemKind::Word(ref name) if *name == "show_typing" => {
      grammar.merge_print_typing(PrintLevel::Show);
    },
      &MetaItemKind::Word(ref name)
    | &MetaItemKind::List(ref name, _)
    | &MetaItemKind::NameValue(ref name, _) => {
      grammar.warn(format!(
        "Unknown attribute `{}`: it will be ignored.",
        name));
    }
  }
}

fn check_rules_attributes<'a, 'b>(grammar: &AGrammar<'a, 'b>, rules: Vec<FRule>) {
  for rule in rules {
    for attr in rule.attributes {
      let meta_item = attr.node.value;
      check_rule_attr(grammar, rule.name.node, meta_item);
    }
  }
}

fn check_rule_attr<'a, 'b>(grammar: &AGrammar<'a, 'b>, rule_name: Ident, meta_item: P<MetaItem>) {
  match &meta_item.node {
      &MetaItemKind::Word(ref name)
    | &MetaItemKind::List(ref name, _)
    | &MetaItemKind::NameValue(ref name, _) => {
      grammar.warn(format!(
        "Unknown attribute `{}` attached to the rule `{}`: it will be ignored.",
        name, rule_name));
      }
  }
}
