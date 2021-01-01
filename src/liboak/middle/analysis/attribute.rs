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

pub fn decorate_with_attributes(mut grammar: AGrammar,
  attributes: Vec<syn::Attribute>) -> Partial<AGrammar>
{
  merge_grammar_attributes(&mut grammar, attributes);
  Partial::Value(grammar)
}

fn warn_ignore_attr(span: Span) {
    span.unstable().warning(format!(
      "unknown attribute: it will be ignored."))
    .emit();
}

fn merge_grammar_attributes(grammar: &mut AGrammar, attrs: Vec<syn::Attribute>) {
  for attr in attrs {
    if let Some(ident) = attr.path.get_ident() {
      merge_grammar_attr(grammar, ident);
    }
    else {
      warn_ignore_attr(attr.span());
    }
  }
}

fn merge_grammar_attr(grammar: &mut AGrammar, ident: &Ident) {
  match &*ident.to_string() {
    "debug_api" => {
      grammar.merge_print_code(PrintLevel::Debug);
    },
    "show_api" => {
      grammar.merge_print_code(PrintLevel::Show);
    },
    "debug_typing" => {
      grammar.merge_print_typing(PrintLevel::Debug);
    },
    "show_typing" => {
      grammar.merge_print_typing(PrintLevel::Show);
    },
    _ => {
      warn_ignore_attr(ident.span());
    }
  }
}
