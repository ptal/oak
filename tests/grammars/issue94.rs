pub use self::issue94::*;

grammar! issue94 {
  underscore = "_"
  digits = (underscore* digit)+ > id
  digit = ["0-9"]

  fn id(v: Vec<char>) -> Vec<char> {
    v
  }
}