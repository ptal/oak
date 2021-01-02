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

use middle::typing::ast::*;
use middle::analysis::ast::AGrammar;

pub use front::ast::FGrammar;
use partial::*;

pub mod analysis;
pub mod typing;

pub fn typecheck(fgrammar: FGrammar) -> Partial<IGrammar> {
  Partial::Value(fgrammar)
    .and_then(|grammar| at_least_one_rule_declared(grammar))
    .and_then(|grammar| analysis::analyse(grammar))
    .and_then(|grammar| extract_stream_type(grammar))
    .and_then(|grammar| typing::type_inference(grammar))
}

fn at_least_one_rule_declared(fgrammar: FGrammar) -> Partial<FGrammar> {
  if fgrammar.rules.len() == 0 {
    fgrammar.start_span.unstable()
      .error("At least one rule must be declared.")
      .emit();
    Partial::Nothing
  } else {
    Partial::Value(fgrammar)
  }
}

/// Modify the default Stream type in the grammar if the user redefined it in its item list.
fn extract_stream_type(mut grammar: AGrammar)
  -> Partial<AGrammar>
{
  let mut stream_redefined = false;
  {
    let stream_alias =
      grammar.rust_items.iter().find_map(|item| {
        match item {
          &syn::Item::Type(ref ty) => {
            if ty.ident.to_string() == "Stream" {
              Some(ty.clone())
            }
            else { None }
          }
          _ => None
        }
      });

    if let Some(ty) = stream_alias {
      grammar.stream_alias = ty;
      stream_redefined = true;
    }
  }
  if !stream_redefined {
    grammar.rust_items.push(syn::Item::Type(grammar.stream_alias.clone()));
  }
  Partial::Value(grammar)
}
