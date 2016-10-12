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

pub use self::ntcc::*;

grammar! ntcc {

  // #![debug_api]
  // #![debug_typing]

  ntcc = spacing expression -> (^)

  expression
    = sum
    / par
    / tell
    / next
    / async
    / rep
    / unless
    / let_in
    / skip_kw

  sum
    = pick_kw or? when sum_body* end_kw?

  sum_body
    = or when -> (^)

  par
    = par_kw oror? expression par_body* end_kw?

  par_body
    = oror expression -> (^)

  tell
    = store_kw left_arrow constraint

  next
    = next_kw expression

  async
    = async_kw expression

  rep
    = rep_kw expression

  unless
    = unless_kw entailed_by next

  when
    = when_kw entails right_arrow expression

  entails
    = store_kw entail constraint

  entailed_by
    = constraint entail store_kw

  constraint
    = constraint_operand comparison constraint_operand

  constraint_operand
    = integer
    / var_ident

  comparison = le / neq / lt / ge / gt / eq

  spacing = [" \n\t"]* -> (^)

  let_in = let_kw var_decl in_kw expression

  var_decl = var_ident eq_bind var_range

  var_range
    = range
    / domain

  domain = dom_kw var_ident

  // max x .. 10 / min x .. max y / 0..10
  range = range_bound dotdot range_bound

  range_bound
    = integer
    / min_bound
    / max_bound

  min_bound = min_kw var_ident
  max_bound = max_kw var_ident

  integer = ["0-9"]+ spacing -> (^)
  var_ident = !["0-9"] ["a-zA-Z0-9_"]+ spacing -> (^)

  pick_kw = "pick" spacing
  when_kw = "when" spacing
  store_kw = "store" spacing
  skip_kw = "skip" spacing
  let_kw = "let" spacing
  in_kw = "in" spacing
  dom_kw = "dom" spacing
  min_kw = "min" spacing
  max_kw = "max" spacing
  end_kw = "end" spacing
  par_kw = "par" spacing
  next_kw = "next" spacing
  async_kw = "async" spacing
  rep_kw = "rep" spacing
  unless_kw = "unless" spacing

  or = "|" spacing
  oror = "||" spacing
  entail = "|=" spacing
  lt = "<" spacing
  le = "<=" spacing
  gt = ">" spacing
  ge = ">=" spacing
  eq = "==" spacing
  neq = "<>" spacing
  right_arrow = "->" spacing
  left_arrow = "<-" spacing
  dotdot = ".." spacing
  eq_bind = "=" spacing
}
