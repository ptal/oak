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

pub fn decorate_with_attributes<'a, 'b>(mut grammar: AGrammar<'a, 'b>,
  attributes: Vec<Attribute>) -> Partial<AGrammar<'a, 'b>>
{
  merge_grammar_attributes(&mut grammar, attributes);
  Partial::Value(grammar)
}

fn merge_grammar_attributes<'a, 'b>(grammar: &mut AGrammar<'a, 'b>, attrs: Vec<Attribute>) {
  for attr in attrs {
    merge_grammar_attr(grammar, attr);
  }
}

fn merge_grammar_attr<'a, 'b>(grammar: &mut AGrammar<'a, 'b>, attr: Attribute) {
    if attr.tokens.is_empty() {
        if attr.path == "debug_api" {
            grammar.merge_print_code(PrintLevel::Debug);
        } else if attr.path == "show_api" {
            grammar.merge_print_code(PrintLevel::Show);
        } else if attr.path == "debug_typing" {
            grammar.merge_print_typing(PrintLevel::Debug);
        } else if attr.path == "show_typing" {
            grammar.merge_print_typing(PrintLevel::Show);
        } else {
            grammar.warn(format!("Unknown attribute `{}`: it will be ignored.", attr.path));
        }
    }
}
