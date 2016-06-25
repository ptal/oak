// Copyright 2016 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Generates Rust value from Oak expression.

use middle::typing::ast::*;

pub fn tuple_value<'a, 'b>(grammar: &TGrammar<'a, 'b>, expr_idx: usize, values_names: Vec<Ident>) -> RExpr
{
  let span = self.grammar[expr_idx].span;
  let values = values_names.into_iter()
    .map(|name| quote_expr!(self.grammar.cx, $name))
    .collect();
  self.grammar.cx.expr_tuple(span, values)
}
