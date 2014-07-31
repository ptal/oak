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

use rust;
pub use rust::{SpannedIdent, Spanned, Attribute};
pub use identifier::*;

pub struct Grammar{
  pub name: Ident,
  pub rules: Vec<Rule>,
  pub attributes: Vec<Attribute>
}

pub struct Rule{
  pub name: SpannedIdent,
  pub attributes: Vec<Attribute>,
  pub def: Box<Expression>
}

#[deriving(Clone)]
pub enum Expression_{
  StrLiteral(String), // "match me"
  AnySingleChar, // .
  NonTerminalSymbol(Ident), // a_rule
  Sequence(Vec<Box<Expression>>), // a_rule next_rule
  Choice(Vec<Box<Expression>>), // try_this / or_try_this_one
  ZeroOrMore(Box<Expression>), // space*
  OneOrMore(Box<Expression>), // space+
  Optional(Box<Expression>), // space? - `?` replaced by `$`
  NotPredicate(Box<Expression>), // !space
  AndPredicate(Box<Expression>), // &space
  CharacterClass(CharacterClassExpr)
}

#[deriving(Clone)]
pub struct CharacterClassExpr {
  pub intervals: Vec<CharacterInterval>
}

#[deriving(Clone)]
pub struct CharacterInterval {
  pub lo: char,
  pub hi: char
}

pub type Expression = Spanned<Expression_>;

pub fn get_attribute<'a>(rule_attrs: &'a Vec<Attribute>,
 attr_name: &str) -> Option<&'a Attribute>
{
  for attr in rule_attrs.iter() {
    match attr.node.value.node {
      rust::MetaWord(ref w) if w.get() == attr_name =>
        return Some(attr),
      _ => ()
    }
  }
  None
}
