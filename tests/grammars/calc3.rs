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

use oak::oak;
use self::Expression::*;
use self::BinOp::*;
use grammars::calc2;

pub type PExpr = Box<Expression>;

#[derive(Debug)]
pub enum Expression {
  Variable(String),
  Number(u32),
  BinaryExpr(BinOp, PExpr, PExpr),
  LetIn(String, PExpr, PExpr)
}

#[derive(Debug)]
pub enum BinOp {
  Add, Sub, Mul, Div, Exp
}

oak! {
  // Optional stream declaration.
  type Stream<'a> = StrStream<'a>;

  program = spacing expression

  expression
    = term (term_op term)* > fold_left

  term
    = exponent (factor_op exponent)* > fold_left

  exponent
    = (factor exponent_op)* factor > fold_right

  factor: Box<Expression>
    = calc2::number > box Number
    / calc2::identifier > box Variable
    / let_expr > box LetIn
    / lparen expression rparen

  let_expr = let_kw let_binding in_kw expression
  let_binding = (calc2::identifier:String) bind_op expression

  term_op: BinOp
    = add_op > Add
    / sub_op > Sub

  factor_op: BinOp
    = mul_op > Mul
    / div_op > Div

  exponent_op: BinOp = exp_op > Exp

  spacing = [" \n\r\t"]*:(^)

  kw_tail = !calc2::ident_char spacing

  keyword = let_kw / in_kw
  let_kw = "let" kw_tail
  in_kw = "in" kw_tail

  bind_op = "=" spacing
  add_op = "+" spacing
  sub_op = "-" spacing
  mul_op = "*" spacing
  div_op = "/" spacing
  exp_op = "^" spacing
  lparen = "(" spacing
  rparen = ")" spacing

  fn fold_left(head: PExpr, rest: Vec<(BinOp, PExpr)>) -> PExpr {
    rest.into_iter().fold(head,
      |accu, (op, expr)| Box::new(BinaryExpr(op, accu, expr)))
  }

  fn fold_right(front: Vec<(PExpr, BinOp)>, last: PExpr) -> PExpr {
    front.into_iter().rev().fold(last,
      |accu, (expr, op)| Box::new(BinaryExpr(op, expr, accu)))
  }
}
