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

//! The types of the rules of this grammar must be valid (Bug #75).

pub use self::recursive_type::*;

grammar! recursive_type {

  r = "a" r
    / "b"

  factor
    = integer
    / unary_arith_expr

  unary_arith_expr
    = "+" factor > id
    / "-" factor > make_neg_expr

  integer
    = ["0-9"]+ > make_integer

  use std::str::FromStr;

  pub enum Expr {
    Number(u64),
    NegExpr(PExpr)
  }

  pub type PExpr = Box<Expr>;

  fn id(e: PExpr) -> PExpr { e }

  fn make_integer(raw_number: Vec<char>) -> PExpr {
    match u64::from_str(&*to_string(raw_number)).ok() {
      Some(x) => Box::new(Expr::Number(x)),
      None => panic!("int literal is too large")
    }
  }

  fn to_string(raw_text: Vec<char>) -> String {
    raw_text.into_iter().collect()
  }

  fn make_neg_expr(expr: PExpr) -> PExpr {
    Box::new(Expr::NegExpr(expr))
  }
}
