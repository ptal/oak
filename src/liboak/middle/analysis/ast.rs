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

#[derive(Default)]
pub struct GrammarAttributes
{
  pub print_attr: PrintAttribute
}

impl GrammarAttributes
{
  pub fn new(print_attr: PrintAttribute) -> GrammarAttributes {
    GrammarAttributes {
      print_attr: print_attr
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PrintAttribute
{
  DebugApi,
  ShowApi,
  Nothing
}

impl PrintAttribute
{
  pub fn merge(self, other: PrintAttribute) -> PrintAttribute {
    use self::PrintAttribute::*;
    match (self, other) {
        (Nothing, DebugApi)
      | (ShowApi, DebugApi) => DebugApi,
      (Nothing, ShowApi) => ShowApi,
      _ => Nothing
    }
  }

  pub fn debug_api(self) -> bool {
    self == PrintAttribute::DebugApi
  }

  pub fn show_api(self) -> bool {
    self == PrintAttribute::ShowApi
  }
}

impl Default for PrintAttribute
{
  fn default() -> PrintAttribute {
    PrintAttribute::Nothing
  }
}