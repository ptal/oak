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

peg!(
  grammar ntcc;

  #![print_generated]

  #[start]
  start = spacing expression

  expression = sum
             / skip_kw

  sum
    = pick_kw sum_body

  sum_body
    = or when sum_body$

  when
    = when_kw entails arrow_right expression

  entails
    = store_kw entail constraint

  constraint
    = constraint_operand comparison constraint_operand

  constraint_operand
    = integer
    / var_ident

  comparison = le / neq / lt / ge / gt / eq

  spacing = [" \n\t"]*

  integer = ["0-9"]+ spacing
  var_ident = !["0-9"] ["a-zA-Z0-9"]+ spacing

  pick_kw = "pick" spacing
  when_kw = "when" spacing
  store_kw = "store" spacing
  skip_kw = "skip" spacing
  
  or = "|" spacing
  entail = "|=" spacing
  lt = "<" spacing
  le = "<=" spacing
  gt = ">" spacing
  ge = ">=" spacing
  eq = "==" spacing
  neq = "<>" spacing
  arrow_right = "->" spacing
)
