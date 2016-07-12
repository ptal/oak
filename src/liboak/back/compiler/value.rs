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
use rust::AstBuilder;

pub fn tuple_value(cx: &ExtCtxt, span: Span, vars_names: Vec<Ident>) -> RExpr
{
  let values: Vec<_> = vars_names.into_iter()
    .map(|name| quote_expr!(cx, $name))
    .collect();
  if values.len() == 0 {
    quote_expr!(cx, ())
  }
  else if values.len() == 1 {
    values[0].clone()
  }
  else {
    cx.expr_tuple(span, values)
  }
}

pub fn tuple_pattern(cx: &ExtCtxt, span: Span, vars_names: Vec<Ident>) -> RPat
{
  let values: Vec<_> = vars_names.into_iter()
    .map(|name| quote_pat!(cx, $name))
    .collect();
  if values.len() == 0 {
    quote_pat!(cx, ())
  }
  else if values.len() == 1 {
    values[0].clone()
  }
  else {
    cx.pat_tuple(span, values)
  }
}
