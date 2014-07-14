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

#![crate_name = "testpeg"]
#![experimental]
#![crate_type = "bin"]

#![feature(phase)]

#[phase(plugin)]
extern crate peg;

use std::os;
use std::io::File;
use std::io::fs;

pub mod ntcc;

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

fn skip_test(test_name: &str, reason: &String)
{
  fail!(format!("[Skip] {}: {}.", test_name, reason));
}

fn test_input<'b>(parse: <'a>|input:&'a str| -> Result<Option<&'a str>, String>,
  expectation: ExpectedResult, name: &str, input: &'b str)
{
  match (expectation, parse(input)) {
    (Match, Ok(None))
  | (PartialMatch, Ok(Some(_)))
  | (Error, Err(_)) => (),
    (expected, res) => {
      fail!(format!("Grammar {}: `{}` was expected to {} but {}.",
        name, input, expected_to_string(expected),
        parse_res_to_string(&res)));
    }
  }
}

fn test_grammar_source_file(parse: <'a>|input:&'a str| -> Result<Option<&'a str>, String>,
  expectation: ExpectedResult, name: &str, path: &Path)
{
  let mut file = File::open(path);
  match file {
    Err(io_err) => fail!(format!("[Failure] Could not open test `{}`, the error is `{}`",
      path.display(), io_err)),
    Ok(ref mut file) => {
      let contents = file.read_to_end();
      match contents {
        Err(io_err) => fail!(format!("[Failure] Could not read test `{}`, the error is `{}`",
          path.display(), io_err)),
        Ok(contents) => {
          let utf8_contents = std::str::from_utf8(contents.as_slice());
          test_input(|input| parse(input), expectation, name, utf8_contents.unwrap());
        }
      }
    }
  }
}

fn test_grammar_source_files(parse: <'a>|input:&'a str| -> Result<Option<&'a str>, String>,
  expectation: ExpectedResult, name: &str, path: Path)
{
  match fs::readdir(&path) {
    Ok(files_to_test) => {
      for file in files_to_test.iter() {
        test_grammar_source_file(|input| parse(input), expectation, name, file);
      }
    }
    Err(io_err) => skip_test(name,
      &format!("Impossible to read the directory `{}`, the error is `{}`", 
        path.display(), io_err))
  }
}

fn test_grammar(parse: <'a>|input:&'a str| -> Result<Option<&'a str>, String>,
  name: &str, path: Path)
{
  let mut path = path;
  path.push(name);
  for &(expectation, test_dir) in [(Error, "run-fail"), (Match, "run-pass")].iter() {
    let mut path = path.clone();
    path.push(test_dir);
    test_grammar_source_files(|input| parse(input), expectation, name, path);
  }
}

fn main()
{
  let args = os::args();
  if args.len() != 2 {
    fail!(format!("usage: {} <data-dir>", args.as_slice()[0]));
  }
  let data_path = Path::new(args.as_slice()[1].clone());
  if !data_path.is_dir() {
    fail!(format!("`{}` is not a valid data directory.", data_path.display()));
  }
  let mut grammar_path = data_path.clone();
  grammar_path.push("grammar");
  if !grammar_path.is_dir() {
    fail!(format!("`{}` is not a valid grammar directory.", grammar_path.display()));
  }
  test_grammar(|input| ntcc::ntcc::parse(input), "ntcc", grammar_path);
}
