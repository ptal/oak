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

//! AST of a PEG expression that is shared across all the compiling steps.

pub use identifier::*;

#[derive(Clone, Debug)]
pub enum Expression_<SubExpr>{
  StrLiteral(String), // "match me"
  AnySingleChar, // .
  CharacterClass(CharacterClassExpr), // [0-9]
  NonTerminalSymbol(Ident), // a_rule
  Sequence(Vec<Box<SubExpr>>), // a_rule next_rule
  Choice(Vec<Box<SubExpr>>), // try_this / or_try_this_one
  ZeroOrMore(Box<SubExpr>), // space*
  OneOrMore(Box<SubExpr>), // space+
  Optional(Box<SubExpr>), // space?
  NotPredicate(Box<SubExpr>), // !space
  AndPredicate(Box<SubExpr>), // &space
  SemanticAction(Box<SubExpr>, Ident) // rule > function
}

#[derive(Clone, Debug)]
pub struct CharacterClassExpr {
  pub intervals: Vec<CharacterInterval>
}

#[derive(Clone, Debug)]
pub struct CharacterInterval {
  pub lo: char,
  pub hi: char
}
