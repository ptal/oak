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

//! This module performs analysis on the PEG and gives a type to each expressions in the AST.

//! The `analysis` module performs some verifications on the grammar description and the `typing` module gives a type to each rule and expression.

use rust;
use middle::typing::ast::*;
use middle::analysis::ast::AGrammar;

pub use front::ast::FGrammar;

pub mod analysis;
pub mod typing;

pub fn typecheck<'a, 'b>(cx: &'a ExtCtxt<'b>, fgrammar: FGrammar) -> Partial<TGrammar<'a, 'b>> {
  Partial::Value(fgrammar)
    .and_then(|grammar| at_least_one_rule_declared(cx, grammar))
    .and_then(|grammar| analysis::analyse(cx, grammar))
    .and_then(|grammar| extract_stream_type(grammar))
    .and_then(|grammar| typing::type_inference(grammar))
}

fn at_least_one_rule_declared(cx: &ExtCtxt, fgrammar: FGrammar) -> Partial<FGrammar> {
  if fgrammar.rules.len() == 0 {
    cx.parse_sess.span_diagnostic.err(
      "At least one rule must be declared.");
    Partial::Nothing
  } else {
    Partial::Value(fgrammar)
  }
}

/// Modify the default Stream type in the grammar if the user redefined it in its item list.
fn extract_stream_type<'a, 'b>(mut grammar: AGrammar<'a, 'b>)
  -> Partial<AGrammar<'a, 'b>>
{
  let mut stream_redefined = false;
  {
    let stream_alias =
      grammar.rust_items.iter().find(|item| {
        match &item.node {
          &rust::ItemKind::Ty(_,_) => {
            &*item.ident.name.as_str() == "Stream"
          }
          _ => false
        }
      });

    if let Some(ty) = stream_alias {
      grammar.stream_alias = ty.clone();
      stream_redefined = true;
    }
  }
  if !stream_redefined {
    grammar.rust_items.push(grammar.stream_alias.clone());
  }
  Partial::Value(grammar)
}
