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

#![feature(plugin, convert, path_ext, box_syntax, rustc_private)]

#![plugin(oak)]

extern crate oak_runtime;
extern crate term;

use oak_runtime::*;
use grammars::*;

use std::path::{PathBuf, Path};
use std::fs::{File, read_dir, PathExt};
use std::io;
use std::io::Read;

use term::*;
use ExpectedResult::*;

mod grammars;

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
  test_engine.register("ntcc", Box::new(|content|
    ntcc::recognize_ntcc(content.stream())));
  test_engine.register("type_name", Box::new(|content|
    type_name::recognize_type_names(content.stream())));
  test_engine.register("calc", Box::new(|content|
    calc::recognize_program(content.stream())));

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

  fn register(&mut self, name: &str, recognizer: Box<for<'a> Fn(&'a str) -> ParseState<StrStream<'a>, ()>>)
  {
    self.grammars.push(GrammarInfo{name: String::from(name), recognizer: recognizer});
  }

  fn run(&mut self)
  {
    self.display.title("    Oak library tests suite");
    for grammar in self.grammars.iter() {
      let grammar_path = self.test_path.join(Path::new(grammar.name.as_str()));
      self.display.info(&format!("Start tests of the grammar `{}`", grammar.name));
      self.display.path(&grammar_path);
      let mut test = Test{
        info: grammar,
        display: &mut self.display
      };
      test.test_directory(&format!("Run and Pass tests of `{}`", grammar.name),
        &grammar_path.join(Path::new("run-pass")), Match);
      test.test_directory(&format!("Run and Fail tests of `{}`", grammar.name),
        &grammar_path.join(Path::new("run-fail")), Error);
    }
    self.display.stats();
    self.display.panic_if_failure();
  }
}

struct GrammarInfo
{
  name: String,
  recognizer: Box<for<'a> Fn(&'a str) -> ParseState<StrStream<'a>, ()>>
}

#[derive(Clone)]
enum ExpectedResult {
  Match,
  Error
}

struct Test<'a>
{
  info: &'a GrammarInfo,
  display: &'a mut TestDisplay,
}

impl<'a> Test<'a>
{
  fn test_directory(&mut self, start_msg: &String, directory: &Path, expectation: ExpectedResult)
  {
    self.display.info(start_msg);

    match read_dir(directory) {
      Ok(dir_entries) => {
        for entry in dir_entries.map(Result::unwrap).map(|entry| entry.path()) {
          if entry.is_file() {
            self.test_file(entry.as_path(), expectation.clone());
          } else {
            self.display.warn(&format!("Entry ignored because it's not a file."));
            self.display.path(entry.as_path());
          }
        }
      }
      Err(ref io_err) => {
        self.display.fs_error("Can't read directory.", directory, io_err);
      }
    }
  }

  fn test_file(&mut self, filepath: &Path, expectation: ExpectedResult)
  {
    let mut file = File::open(filepath);
    match file {
      Ok(ref mut file) => {
        let mut buf_contents = vec![];
        let contents = file.read_to_end(&mut buf_contents);
        match contents {
          Ok(_) => {
            let utf8_contents = std::str::from_utf8(buf_contents.as_slice());
            self.test_input(utf8_contents.unwrap(), expectation, filepath);
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

  fn test_input(&mut self, input: &str, expectation: ExpectedResult, test_path: &Path)
  {
    let state = (self.info.recognizer)(input);
    let result = state.into_result();
    match (expectation.clone(), result) {
      (Match, Ok((ref state, _))) if state.full_read() => self.display.success(test_path),
      (Error, Ok((ref state, _))) if state.partial_read() => self.display.success(test_path),
      (Error, Err(_)) => self.display.success(test_path),
      (_, state) => {
        self.display.failure(test_path, expectation, state);
      }
    }
  }
}

struct TestDisplay
{
  terminal: Box<Terminal<WriterWrapper>+'static>,
  num_success: u32,
  num_failure: u32,
  num_system_failure: u32
}

impl TestDisplay
{
  pub fn new() -> TestDisplay
  {
    TestDisplay{
      terminal: term::stdout().unwrap(),
      num_success: 0,
      num_failure: 0,
      num_system_failure: 0
    }
  }

  pub fn title(&mut self, msg: &str)
  {
    self.write_header(term::color::CYAN, msg);
    self.write_msg("\n\n");
  }

  pub fn info(&mut self, msg: &String)
  {
    self.write_line(term::color::CYAN, "\n[ info ] ", msg);
  }

  pub fn error(&mut self, msg: &String)
  {
    self.write_line(term::color::RED, "  [ error ] ", msg);
  }

  pub fn path(&mut self, path: &Path)
  {
    self.write_line(term::color::CYAN, "  [ path ] ",
      &format!("{}", path.display()));
  }

  pub fn stats(&mut self)
  {
    let system_failure_plural = if self.num_system_failure > 1 { "s" } else { "" };
    let msg = format!("{} passed, {} failed, {} system failure{}.",
        self.num_success, self.num_failure, self.num_system_failure,
        system_failure_plural);
    self.write_line(term::color::BLUE, "\n\n[ stats ] ", &msg);
  }

  pub fn panic_if_failure(&self)
  {
    if self.num_failure > 0 || self.num_system_failure > 0 {
      panic!("");
    }
  }

  pub fn failure(&mut self, path: &Path, expectation: ExpectedResult,
    result: ParseResult<StrStream, ()>)
  {
    self.num_failure += 1;
    let test_name = self.file_stem(path);
    self.write_line(term::color::RED, "[ failed ] ", &test_name);
    self.path(path);
    self.expected(expectation);
    self.wrong_result(result);
  }

  fn expected(&mut self, expectation: ExpectedResult)
  {
    let msg = match expectation {
      Match => "Fully match",
      Error => "Error"
    };
    self.write_line(term::color::CYAN, "  [ expected ] ", &format!("{}", msg))
  }

  fn wrong_result(&mut self, result: ParseResult<StrStream, ()>)
  {
    let msg = match result {
      Ok((ref state, ref err)) if state.partial_read() => {
        format!("Partial match. `{}`", err)
      }
      Ok(_) => format!("Fully matched."),
      Err(err) => format!("{}", err)
    };
    self.error(&msg)
  }

  pub fn success(&mut self, path: &Path)
  {
    self.num_success += 1;
    let test_name = self.file_stem(path);
    self.write_line(term::color::GREEN, "[ passed ] ", &test_name);
  }

  fn file_stem(&self, path: &Path) -> String
  {
    format!("{}", path.file_stem().unwrap().to_str().unwrap())
  }

  pub fn warn(&mut self, msg: &String)
  {
    self.write_line(term::color::YELLOW, "  [ warning ] ", msg);
  }

  pub fn fs_error(&mut self, msg: &str, path: &Path, io_err: &io::Error)
  {
    self.system_failure(&format!("{}", msg));
    self.path(path);
    self.error(&format!("{}", io_err));
  }

  pub fn system_failure(&mut self, msg: &String)
  {
    self.num_system_failure += 1;
    self.write_line(term::color::RED, "[ system error ] ", msg);
  }

  fn write_line(&mut self, color: color::Color, header: &str, msg: &String)
  {
    self.write_header(color, header);
    self.write_msg(msg.as_str());
    self.write_msg("\n");
  }

  fn write_header(&mut self, color: color::Color, header: &str)
  {
    self.terminal.fg(color).unwrap();
    self.write_msg(header);
    self.terminal.reset().unwrap();
  }

  fn write_msg(&mut self, msg: &str)
  {
    (write!(self.terminal, "{}", msg)).unwrap();
  }
}
