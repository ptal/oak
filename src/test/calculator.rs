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

pub use calculator::calculator::*;

grammar! calculator{
  #![debug_api]

  #[start]
  expression = sum

  sum
    = product ("+" product)* > add

  product
    = value ("*" value)* > mult

  value
    = ["0-9"]+ > to_digit
    / "(" expression ")"

  pub type Digit = u32;

  fn add(x: Digit, rest: Vec<Digit>) -> Digit {
    rest.iter().fold(x, |x,y| x+y)
  }

  fn mult(x: Digit, rest: Vec<Digit>) -> Digit {
    rest.iter().fold(x, |x,y| x*y)
  }

  fn to_digit(raw_text: Vec<char>) -> Digit {
    use std::str::FromStr;
    let text: String = raw_text.into_iter().collect();
    u32::from_str(&*text).unwrap()
  }
}