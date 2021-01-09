use oak::*;


oak! {
  ra = "a" ra / "b" .
  // rb = "a" rb . / "b" .
  // rc = ra . / .

  rd = "a" rd // debatable to allow this rule. However, with Partial match it can make sense...

  re = . re

  factor: Expr
    = number > Expr::Number
    / identifier > Expr::Variable
    / lparen factor rparen

  lparen = "("
  rparen = ")"
  number = ["0-9"]+
  identifier = ["a-z"]+

  pub enum Expr {
    Number(Vec<char>),
    Variable(Vec<char>)
  }

  rule1 = "a" > test2
  // rule2 = "a" > test3 // Fail due to unit type
  // rule3 = "a" > test4 // Fail due to unit type

  type MyUnit = ();
  fn test2() -> MyUnit {}
  fn test3() -> () {}
  fn test4() {}

  rule4 = r#""foo""#
  rule5 = r##"foo #"# bar"##
}
