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

#![feature(phase)]

#[phase(plugin)]
extern crate peg;

#[cfg(test)]
mod tests{

  peg!(
    grammar ntcc;

    #[start]
    start = spacing test

    test = STORE ENTAIL
         / ENTAIL STORE

    ENTAIL = "|" !spacing "=" &spacing spacing
    STORE = ("store" spacing)+
    spacing = " "+
  )

  enum ExpectedResult {
    Match,
    PartialMatch,
    Error
  }

  fn expected_to_string(expected: ExpectedResult) -> &'static str
  {
    match expected {
      Match => "fully match",
      PartialMatch => "partially match",
      Error => "fail"
    }
  }

  fn parse_res_to_string<'a>(res: &Result<Option<&'a str>, String>) -> String
  {
    match res {
      &Ok(None) => String::from_str("fully matched"),
      &Ok(Some(ref rest)) => 
        format!("partially matched (it remains `{}`)", rest),
      &Err(ref msg) =>
        format!("failed with the error \"{}\"", msg)
    }
  }

  fn test_ntcc(expected: ExpectedResult, input: &str)
  {
    match (expected, ntcc::parse(input)) {
        (Match, Ok(None))
      | (PartialMatch, Ok(Some(_)))
      | (Error, Err(_)) => (),
      (expected, res) => {
        fail!(format!("`{}` was expected to {} but {}.",
          input, expected_to_string(expected),
          parse_res_to_string(&res)));
      }
    }
  }

  #[test]
  fn test1() { test_ntcc(Match, " store store |= "); }
}


// Need a main for Cargo to compile this...
// We can't test directly in the lib.rs because of the procedural macro.
#[allow(dead_code)]
fn main() {}
