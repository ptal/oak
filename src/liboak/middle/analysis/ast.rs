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

pub use ast::*;
pub use visitor::*;
pub use front::ast::FExpressionInfo;

use std::default::Default;

pub type AGrammar<'a, 'b> = Grammar<'a, 'b, FExpressionInfo>;

impl<'a, 'b> AGrammar<'a, 'b>
{
  pub fn merge_print_code(&mut self, level: PrintLevel) {
    self.attributes.print_code = self.attributes.print_code.merge(level);
  }

  pub fn merge_print_typing(&mut self, level: PrintLevel) {
    self.attributes.print_typing = self.attributes.print_typing.merge(level);
  }
}

pub struct GrammarAttributes
{
  pub print_code: PrintLevel,
  pub print_typing: PrintLevel

}

impl Default for GrammarAttributes {
  fn default() -> Self {
    GrammarAttributes {
      print_code: PrintLevel::default(),
      print_typing: PrintLevel::default()
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PrintLevel
{
  Debug,
  Show,
  Nothing
}

impl PrintLevel
{
  pub fn merge(self, other: PrintLevel) -> PrintLevel {
    use self::PrintLevel::*;
    match (self, other) {
        (Nothing, Debug)
      | (Show, Debug) => Debug,
      (Nothing, Show) => Show,
      _ => Nothing
    }
  }

  pub fn debug(self) -> bool {
    self == PrintLevel::Debug
  }

  pub fn show(self) -> bool {
    self == PrintLevel::Show
  }
}

impl Default for PrintLevel
{
  fn default() -> PrintLevel {
    PrintLevel::Nothing
  }
}