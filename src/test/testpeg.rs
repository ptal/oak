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

#[phase(plugin, link)]
extern crate peg;
extern crate term;

use std::os;
use std::io::File;
use std::io::fs;
use std::io;

use peg::Parser;

use term::*;

pub mod ntcc;

enum ExpectedResult {
  Match,
  Error
}

struct TestDisplay
{
  terminal: Box<Terminal<WriterWrapper>>,
  code_snippet_len: uint,
  num_success: uint,
  num_failure: uint,
  num_system_failure: uint
}

impl TestDisplay
{
  pub fn new(code_snippet_len: uint) -> TestDisplay
  {
    TestDisplay{
      terminal: term::stdout().unwrap(),
      code_snippet_len: code_snippet_len,
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

  pub fn failure<'a>(&mut self, path: &Path, expectation: ExpectedResult, 
    result: &Result<Option<&'a str>, String>)
  {
    self.num_failure += 1;
    let test_name = self.filestem(path);
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

  fn wrong_result<'a>(&mut self, result: &Result<Option<&'a str>, String>)
  {
    let msg = match result {
      &Ok(Some(input)) => format!("Partial match, stopped at `{}`.", self.code_snippet(input)),
      &Ok(None) => format!("Fully matched."),
      &Err(ref msg) => msg.clone()
    };
    self.error(&msg)
  }

  fn code_snippet<'a>(&self, code: &'a str) -> &'a str
  {
    code.slice_to(std::cmp::min(code.len()-1, self.code_snippet_len))
  }

  pub fn success(&mut self, path: &Path)
  {
    self.num_success += 1;
    let test_name = self.filestem(path);
    self.write_line(term::color::GREEN, "[ passed ] ", &test_name);
  }

  fn filestem(&self, path: &Path) -> String
  {
    format!("{}", path.filestem_str().unwrap())
  }

  pub fn warn(&mut self, msg: &String)
  {
    self.write_line(term::color::YELLOW, "  [ warning ] ", msg);
  }

  pub fn fs_error(&mut self, msg: &str, path: &Path, io_err: &io::IoError)
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
    self.write_msg(msg.as_slice());
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

struct GrammarInfo
{
  name: String,
  parser: Box<Parser>
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

    match fs::readdir(directory) {
      Ok(dir_entries) => {
        for entry in dir_entries.iter() {
          if entry.is_file() {
            self.test_file(entry, expectation);
          } else {
            self.display.warn(&format!("Entry ignored because it's not a file."));
            self.display.path(entry);
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
        let contents = file.read_to_end();
        match contents {
          Ok(contents) => {
            let utf8_contents = std::str::from_utf8(contents.as_slice());
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
    match (expectation, self.info.parser.parse(input)) {
      (Match, Ok(None))
    | (Error, Ok(Some(_)))
    | (Error, Err(_)) => self.display.success(test_path),
      (_, ref res) => {
        self.display.failure(test_path, expectation, res);
      }
    }
  }
}

struct TestEngine
{
  test_path : Path,
  grammars : Vec<GrammarInfo>,
  display : TestDisplay
}

impl TestEngine
{
  fn new(test_path: Path) -> TestEngine
  {
    if !test_path.is_dir() {
      fail!(format!("`{}` is not a valid grammar directory.", test_path.display()));
    }
    TestEngine{
      test_path: test_path.clone(),
      grammars : Vec::new(),
      display : TestDisplay::new(20)
    }
  }

  fn register(&mut self, name: &str, parser: Box<Parser>)
  {
    self.grammars.push(GrammarInfo{name: String::from_str(name), parser: parser});
  }

  fn run(&mut self)
  {
    self.display.title("    PEG library tests suite");
    for grammar in self.grammars.iter() {
      let grammar_path = self.test_path.join(Path::new(grammar.name.clone()));
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
