# Full Calc Grammar

The following code is the grammar of the `Calc` language which is incrementally built and explained in the [previous chapter](learn-oak.md).

```rust
extern crate oak_runtime;
use oak_runtime::*;
use oak::oak;
use self::Expression::*;
use self::BinOp::*;
use std::str::FromStr;

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

  factor: PExpr
    = number > box Number
    / identifier > box Variable
    / let_expr > box LetIn
    / lparen expression rparen

  let_expr = let_kw let_binding in_kw expression
  let_binding = identifier bind_op expression

  term_op: BinOp
    = add_op > Add
    / sub_op > Sub

  factor_op: BinOp
    = mul_op > Mul
    / div_op > Div

  exponent_op: BinOp = exp_op > Exp

  identifier = !digit !keyword ident_char+ spacing > to_string
  ident_char = ["a-zA-Z0-9_"]

  digit = ["0-9"]
  number = digit+ spacing > to_number
  spacing = [" \n\r\t"]*:(^)

  kw_tail = !ident_char spacing

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

  fn to_number(raw_text: Vec<char>) -> u32 {
    u32::from_str(&*to_string(raw_text)).unwrap()
  }

  fn to_string(raw_text: Vec<char>) -> String {
    raw_text.into_iter().collect()
  }

  fn fold_left(head: PExpr, rest: Vec<(BinOp, PExpr)>) -> PExpr {
    rest.into_iter().fold(head,
      |accu, (op, expr)| Box::new(BinaryExpr(op, accu, expr)))
  }

  fn fold_right(front: Vec<(PExpr, BinOp)>, last: PExpr) -> PExpr {
    front.into_iter().rev().fold(last,
      |accu, (expr, op)| Box::new(BinaryExpr(op, expr, accu)))
  }
}


fn analyse_state(state: ParseState<StrStream, PExpr>) {
  use oak_runtime::parse_state::ParseResult::*;
  match state.into_result() {
    Success(data) => println!("Full match: {:?}", data),
    Partial(data, expectation) => {
      println!("Partial match: {:?} because: {:?}", data, expectation);
    }
    Failure(expectation) => {
      println!("Failure: {:?}", expectation);
    }
  }
}

fn main() {
  analyse_state(parse_program("2 * a".into_state())); // Complete
  analyse_state(parse_program("2 *  ".into_state())); // Partial
  analyse_state(parse_program("  * a".into_state())); // Erroneous

  let program1 =
    "let a = 5 in \
     let b = 2 in \
     a^2 + b^2 + (a - b)^2 \
    ";
  analyse_state(parse_program(program1.into_state()));

  let program2 =
    "let a = \
       let b = 7^3 in 2 * b \
     in \
     a^2 - (let x = a in x * 2) \
    ";
  println!("{:?}", parse_program(program2.into_state()).into_result());
}
```
