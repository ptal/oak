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

#![feature(plugin, box_syntax, rustc_private)]
#![plugin(oak)]

/// This is a test framework for grammars and inputs that should be accepted or rejected by these grammars.
/// The grammars to test are sub-modules and the inputs are in the directory `data/` at the root of this project. There is one directory per grammar to test with the same name. There is two test mode possible:
/// * Bulk test: Two files are present in the directory and finish with either `.bulk.pass` or `.bulk.fail`. Each line of these files represent one input to test for the grammar considered.
/// * Full test: Two directories are present: `run-pass` and `run-fail` and each files in these directories represent a full input to test against the considered grammar.

extern crate oak_runtime;
extern crate term;

use oak_runtime::*;
use oak_runtime::ParseResult::*;
use grammars::*;

use std::path::{PathBuf, Path};
use std::fs::{File, read_dir};
use std::io;
use std::io::Read;

use term::*;
use ExpectedResult::*;

mod grammars;

type RecognizerFn = Box<for<'a> Fn(ParseState<StrStream<'a>, ()>) -> ParseState<StrStream<'a>, ()>>;

#[test]
fn test_data_directory()
{
  let data_path = Path::new("data/");
  if !data_path.is_dir() {
    panic!(format!("`{}` is not a valid data directory.", data_path.display()));
  }
  let mut test_path = PathBuf::new();
  test_path.push(data_path);
  test_path.push(Path::new("test"));
  let mut test_engine = TestEngine::new(test_path);
  test_engine.register("ntcc", None, Box::new(
    |s| ntcc::recognize_ntcc(s)));
  test_engine.register("type_name", None, Box::new(
    |s| type_name::recognize_type_names(s)));
  test_engine.register("calc", None, Box::new(
    |s| calc::recognize_program(s)));
  test_engine.register("combinators", Some(format!("str_literal")), Box::new(
    |s| combinators::recognize_str_literal(s)));
  test_engine.register("combinators", Some(format!("sequence")), Box::new(
    |s| combinators::recognize_sequence(s)));
  test_engine.register("combinators", Some(format!("any_single_char")), Box::new(
    |s| combinators::recognize_any_single_char(s)));
  test_engine.register("combinators", Some(format!("choice")), Box::new(
    |s| combinators::recognize_choice(s)));
  test_engine.register("combinators", Some(format!("repeat")), Box::new(
    |s| combinators::recognize_repeat(s)));
  test_engine.register("combinators", Some(format!("syntactic_predicate")), Box::new(
    |s| combinators::recognize_predicate(s)));
  test_engine.register("combinators", Some(format!("optional")), Box::new(
    |s| combinators::recognize_optional(s)));
  test_engine.register("combinators", Some(format!("char_class")), Box::new(
    |s| combinators::recognize_char_class(s)));
  test_engine.register("combinators", Some(format!("non_terminal")), Box::new(
    |s| combinators::recognize_non_terminal(s)));
  test_engine.run();
}

struct TestEngine
{
  test_path: PathBuf,
  grammars: Vec<GrammarInfo>,
  display: TestDisplay
}

impl TestEngine
{
  fn new(test_path: PathBuf) -> TestEngine
  {
    if !test_path.is_dir() {
      panic!(format!("`{}` is not a valid grammar directory.", test_path.display()));
    }
    TestEngine{
      test_path: test_path,
      grammars: Vec::new(),
      display: TestDisplay::new()
    }
  }

  fn register(&mut self, name: &str, bulk: Option<String>, recognizer: RecognizerFn)
  {
    self.grammars.push(GrammarInfo::new(name, bulk, recognizer));
  }

  fn run(&mut self)
  {
    self.display.title("    Oak library tests suite");
    for grammar in self.grammars.iter() {
      let grammar_path = self.test_path.join(Path::new(grammar.name.as_str()));
      self.display.info(format!("Start tests of the grammar `{}`", grammar.name));
      self.display.path(grammar_path.clone());
      let mut test = Test{
        info: grammar,
        display: &mut self.display
      };
      if let Some(ref bulk_file) = grammar.bulk_file {
        test.test_bulk_file(format!("Bulk Run and Pass test of `{}/{}`", grammar.name, bulk_file),
          TestEngine::bulk_file_path(&grammar_path, bulk_file, "pass"), Match);
        test.test_bulk_file(format!("Bulk Run and Partial Pass test of `{}/{}`", grammar.name, bulk_file),
          TestEngine::bulk_file_path(&grammar_path, bulk_file, "partial"), PartialMatch);
        test.test_bulk_file(format!("Bulk Run and Fail test of `{}/{}`", grammar.name, bulk_file),
          TestEngine::bulk_file_path(&grammar_path, bulk_file, "fail"), Error);
      }
      else {
        test.test_directory(format!("Run and Pass tests of `{}`", grammar.name),
          grammar_path.join(Path::new("run-pass")), Match);
        test.test_directory(format!("Run and Partial Pass tests of `{}`", grammar.name),
          grammar_path.join(Path::new("run-partial")), PartialMatch);
        test.test_directory(format!("Run and Fail tests of `{}`", grammar.name),
          grammar_path.join(Path::new("run-fail")), Error);
      }
    }
    self.display.stats();
    self.display.panic_if_failure();
  }

  fn bulk_file_path(grammar_path: &PathBuf, bulk_file: &String, extension: &str) -> PathBuf {
    grammar_path.join(
      Path::new(format!("{}.bulk.{}", bulk_file, extension).as_str()))
  }
}

struct GrammarInfo
{
  name: String,
  bulk_file: Option<String>,
  recognizer: RecognizerFn
}

impl GrammarInfo
{
  fn new(name: &str, bulk_file: Option<String>, recognizer: RecognizerFn) -> GrammarInfo {
    GrammarInfo {
      name: String::from(name),
      bulk_file: bulk_file,
      recognizer: recognizer
    }
  }
}

#[derive(Clone)]
enum ExpectedResult {
  Match,
  PartialMatch,
  Error
}

struct Test<'a>
{
  info: &'a GrammarInfo,
  display: &'a mut TestDisplay,
}

impl<'a> Test<'a>
{

  fn test_bulk_file(&mut self, start_msg: String, bulk_file: PathBuf, expectation: ExpectedResult) {
    self.display.info(start_msg);
    self.test_file(bulk_file, true, expectation);
  }

  fn test_directory(&mut self, start_msg: String, directory: PathBuf, expectation: ExpectedResult) {
    self.display.info(start_msg);
    match read_dir(&directory) {
      Ok(dir_entries) => {
        for entry in dir_entries.map(Result::unwrap).map(|entry| entry.path()) {
          if entry.is_file() {
            self.test_file(entry, false, expectation.clone());
          } else {
            self.display.warn(format!("Entry ignored because it's not a file."));
            self.display.path(entry);
          }
        }
      }
      Err(ref io_err) => {
        self.display.fs_error("Can't read directory.", directory, io_err);
      }
    }
  }

  fn test_file(&mut self, filepath: PathBuf, bulk: bool, expectation: ExpectedResult) {
    let mut file = File::open(filepath.clone());
    match file {
      Ok(ref mut file) => {
        let mut buf_contents = vec![];
        let contents = file.read_to_end(&mut buf_contents);
        match contents {
          Ok(_) => {
            let utf8_contents = std::str::from_utf8(buf_contents.as_slice());
            self.test_input_from_file(utf8_contents.unwrap(), bulk, expectation, filepath);
          },
          Err(ref io_err) => {
            self.display.fs_error("Can't read file.", filepath, io_err);
          }
        }
      },
      Err(ref io_err) => {
        self.display.fs_error("Can't open file.", filepath, io_err);
      }
    }
  }

  fn test_input_from_file(&mut self, input: &str, bulk: bool, expectation: ExpectedResult, test_path: PathBuf) {
    let file_name = self.file_name(test_path.clone());
    if bulk {
      for (i, line) in input.lines().enumerate() {
        let test_name = format!(
          "{} (line {})", file_name, i+1);
        self.test_input(line, expectation.clone(), test_path.clone(), test_name);
      }
    }
    else {
      self.test_input(input, expectation, test_path, file_name);
    }
  }

  fn file_name(&self, path: PathBuf) -> String {
    format!("{}", path.file_name().unwrap().to_str().unwrap())
  }

  fn test_input(&mut self, input: &str, expectation: ExpectedResult, test_path: PathBuf, test_name: String) {
    let state = (self.info.recognizer)(input.into_state());
    let result = state.into_result();
    match (expectation.clone(), result) {
      (Match, Success(_))
    | (PartialMatch, Partial(_, _))
    | (Error, Failure(_)) => self.display.success(test_name),
      (_, state) => {
        self.display.failure(test_path, test_name, expectation, state);
      }
    }
  }
}

struct TestDisplay
{
  terminal: Box<StdoutTerminal>,
  num_success: u32,
  num_failure: u32,
  num_system_failure: u32
}

impl TestDisplay
{
  pub fn new() -> TestDisplay {
    TestDisplay{
      terminal: term::stdout().unwrap(),
      num_success: 0,
      num_failure: 0,
      num_system_failure: 0
    }
  }

  pub fn title(&mut self, msg: &str) {
    self.write_header(term::color::CYAN, msg);
    self.write_msg("\n\n");
  }

  pub fn info(&mut self, msg: String) {
    self.write_line(term::color::CYAN, "\n[ info ] ", msg);
  }

  pub fn error(&mut self, msg: String) {
    self.write_line(term::color::RED, "  [ error ] ", msg);
  }

  pub fn path(&mut self, path: PathBuf) {
    self.write_line(term::color::CYAN, "  [ path ] ",
      format!("{}", path.display()));
  }

  pub fn stats(&mut self) {
    let system_failure_plural = if self.num_system_failure > 1 { "s" } else { "" };
    let msg = format!("{} passed, {} failed, {} system failure{}.",
        self.num_success, self.num_failure, self.num_system_failure,
        system_failure_plural);
    self.write_line(term::color::BLUE, "\n\n[ stats ] ", msg);
  }

  pub fn panic_if_failure(&self) {
    if self.num_failure > 0 || self.num_system_failure > 0 {
      panic!("");
    }
  }

  pub fn failure(&mut self, path: PathBuf, test_name: String, expectation: ExpectedResult,
    result: ParseResult<StrStream, ()>)
  {
    self.num_failure += 1;
    self.write_line(term::color::RED, "[ failed ] ", test_name);
    self.path(path);
    self.expected(expectation);
    self.wrong_result(result);
  }

  fn expected(&mut self, expectation: ExpectedResult) {
    let msg = match expectation {
      Match => "Fully match",
      PartialMatch => "Partial match",
      Error => "Error"
    };
    self.write_line(term::color::CYAN, "  [ expected ] ", format!("{}", msg))
  }

  fn wrong_result(&mut self, result: ParseResult<StrStream, ()>) {
    let msg = match result {
      Success(_) => format!("Fully matched."),
      Partial(_, expectation) => {
        format!("Partial match. `{:?}`", expectation)
      }
      Failure(expectation) => {
        format!("{:?}", expectation)
      }
    };
    self.error(msg)
  }

  pub fn success(&mut self, test_name: String) {
    self.num_success += 1;
    self.write_line(term::color::GREEN, "[ passed ] ", test_name);
  }

  pub fn warn(&mut self, msg: String) {
    self.write_line(term::color::YELLOW, "  [ warning ] ", msg);
  }

  pub fn fs_error(&mut self, msg: &str, path: PathBuf, io_err: &io::Error) {
    self.system_failure(format!("{}", msg));
    self.path(path);
    self.error(format!("{}", io_err));
  }

  pub fn system_failure(&mut self, msg: String) {
    self.num_system_failure += 1;
    self.write_line(term::color::RED, "[ system error ] ", msg);
  }

  fn write_line(&mut self, color: color::Color, header: &str, msg: String) {
    self.write_header(color, header);
    self.write_msg(msg.as_str());
    self.write_msg("\n");
  }

  fn write_header(&mut self, color: color::Color, header: &str) {
    self.terminal.fg(color).unwrap();
    self.write_msg(header);
    self.terminal.reset().unwrap();
  }

  fn write_msg(&mut self, msg: &str) {
    (write!(self.terminal, "{}", msg)).unwrap();
  }
}
