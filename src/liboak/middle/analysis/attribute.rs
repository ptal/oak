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

use rust::{MetaItemKind, MetaItem};

pub fn decorate_with_attributes<'a, 'b>(mut grammar: AGrammar<'a, 'b>,
  attributes: Vec<Attribute>) -> Partial<AGrammar<'a, 'b>>
{
  merge_grammar_attributes(&mut grammar, attributes);
  Partial::Value(grammar)
}

fn merge_grammar_attributes<'a, 'b>(grammar: &mut AGrammar<'a, 'b>, attrs: Vec<Attribute>) {
  for attr in attrs {
    attr.meta().map(|meta_item| {
        merge_grammar_attr(grammar, meta_item);
    });
  }
}

fn merge_grammar_attr<'a, 'b>(grammar: &mut AGrammar<'a, 'b>, meta_item: MetaItem) {
  match &meta_item.node {
    &MetaItemKind::Word if meta_item.name == "debug_api" => {
      grammar.merge_print_code(PrintLevel::Debug);
    },
    &MetaItemKind::Word if meta_item.name == "show_api" => {
      grammar.merge_print_code(PrintLevel::Show);
    },
    &MetaItemKind::Word if meta_item.name == "debug_typing" => {
      grammar.merge_print_typing(PrintLevel::Debug);
    },
    &MetaItemKind::Word if meta_item.name == "show_typing" => {
      grammar.merge_print_typing(PrintLevel::Show);
    },
      &MetaItemKind::Word
    | &MetaItemKind::List(_)
    | &MetaItemKind::NameValue(_) => {
      grammar.warn(format!(
        "Unknown attribute `{}`: it will be ignored.",
        meta_item.name));
    }
  }
}
