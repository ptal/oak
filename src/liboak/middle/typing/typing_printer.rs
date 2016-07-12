// Copyright 2015 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! It prints the grammar annotated with type and context.

use middle::typing::ast::*;

pub struct TypingPrinter<'a: 'c, 'b: 'a, 'c>
{
  grammar: &'c TGrammar<'a, 'b>
}

impl<'a, 'b, 'c> TypingPrinter<'a, 'b, 'c>
{
  pub fn new(grammar: &'c TGrammar<'a, 'b>) -> Self {
    TypingPrinter {
      grammar: grammar
    }
  }
}
