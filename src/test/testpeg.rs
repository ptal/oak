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

#![feature(phase,globs)]

#[phase(plugin,link)]
extern crate peg;

use std::os;
use std::io::File;
use std::io::fs;

use peg::Parser;

pub mod ntcc;

enum ExpectedResult {
  Match,
  PartialMatch,
  Error
}

fn expectation_to_string(expected: ExpectedResult) -> &'static str
{
  match expected {
    Match => "fully match",
    PartialMatch => "partially match",
    Error => "fail"
  }
}

fn parsing_result_to_string<'a>(res: &Result<Option<&'a str>, String>) -> String
{
  match res {
    &Ok(None) => String::from_str("fully matched"),
    &Ok(Some(ref rest)) => 
      format!("partially matched (it remains `{}`)", rest),
    &Err(ref msg) =>
      format!("failed with the error \"{}\"", msg)
  }
}

fn skip_test(test_name: &String, reason: &String)
{
  fail!(format!("[Skip] {}: {}.", test_name, reason));
}

struct Test
{
  name: String,
  parser: Box<Parser>
}

impl Test
{
  fn test_directory(&self, directory: Path, expectation: ExpectedResult)
  {
    match fs::readdir(&directory) {
      Ok(dir_entries) => {
        for entry in dir_entries.iter() {
          if entry.is_file() {
            self.test_file(entry, expectation);
          }
        }
      }
      Err(io_err) => skip_test(&self.name,
        &format!("Impossible to read the directory `{}`, the error is `{}`", 
          directory.display(), io_err))
    }
  }

  fn test_file(&self, filepath: &Path, expectation: ExpectedResult)
  {
    let mut file = File::open(filepath);
    match file {
      Err(io_err) => fail!(format!("[Failure] Could not open test `{}`, the error is `{}`",
        filepath.display(), io_err)),
      Ok(ref mut file) => {
        let contents = file.read_to_end();
        match contents {
          Err(io_err) => fail!(format!("[Failure] Could not read test `{}`, the error is `{}`",
            filepath.display(), io_err)),
          Ok(contents) => {
            let utf8_contents = std::str::from_utf8(contents.as_slice());
            self.test_input(utf8_contents.unwrap(), expectation);
          }
        }
      }
    }
  }

  fn test_input(&self, input: &str, expectation: ExpectedResult)
  {
    match (expectation, self.parser.parse(input)) {
      (Match, Ok(None))
    | (PartialMatch, Ok(Some(_)))
    | (Error, Err(_)) => (),
      (_, res) => {
        fail!(format!("Grammar {}: `{}` was expected to {} but {}.",
          self.name, input, expectation_to_string(expectation),
          parsing_result_to_string(&res)));
      }
    }
  }
}

struct TestEngine
{
  test_path : Path,
  tests : Vec<Test>,
  current_test_idx: uint
}

impl TestEngine
{
  fn new(test_path: Path) -> TestEngine
  {
    if !test_path.is_dir() {
      fail!(format!("`{}` is not a valid grammar directory.", test_path.display()));
    }
    TestEngine{
      test_path: test_path,
      tests : Vec::new(),
      current_test_idx : 0
    }
  }

  fn register(&mut self, name: &str, parser: Box<Parser>)
  {
    self.tests.push(Test{name: String::from_str(name), parser: parser});
  }

  fn run(&self)
  {
    for test in self.tests.iter() {
      let path = self.test_path.join(Path::new(test.name.clone()));
      test.test_directory(path.join(Path::new("run-pass")), Match);
      test.test_directory(path.join(Path::new("run-fail")), Error);
    }
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
  let mut test_path = data_path.clone();
  test_path.push("test");
  let mut test_engine = TestEngine::new(test_path);
  test_engine.register("ntcc", box ntcc::ntcc::Parser::new());
  test_engine.run();
}
