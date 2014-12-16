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

pub use front::ast::{Expression_, Expression, CharacterInterval, CharacterClassExpr};
pub use front::ast::Expression_::*;

pub use rust::{ExtCtxt, P, Item};
pub use identifier::*;
pub use std::collections::HashMap;

use middle::semantics::ast::*;

pub struct DuplicateRustItem<'a>
{
  cx: &'a ExtCtxt<'a>,
  grammar: Grammar,
  has_duplicate: bool
}

impl<'a> DuplicateRustItem<'a>
{
  pub fn analyse(cx: &'a ExtCtxt<'a>, grammar: Grammar, items: Vec<P<Item>>) -> Option<Grammar>
  {
    DuplicateRustItem {
      cx: cx,
      grammar: grammar,
      has_duplicate: false
    }.populate(items)
     .make()
  }

  fn populate(mut self, items: Vec<P<Item>>) -> DuplicateRustItem<'a>
  {
    for item in items.into_iter() {
      let name = item.ident.clone();
      if self.grammar.rust_items.contains_key(&name) {
        self.duplicate_items(self.grammar.rust_items.get(&name).unwrap(), &item);
        self.has_duplicate = true;
      } else {
        self.grammar.rust_items.insert(name, item);
      }
    }
    self
  }

  fn duplicate_items(&self, pre: &P<Item>, current: &P<Item>)
  {
    self.cx.span_err(current.span, "Duplicate rust item definition.");
    self.cx.span_note(pre.span, "Previous declaration here.");
  }

  fn make(self) -> Option<Grammar>
  {
    if self.has_duplicate {
      None
    } else {
      Some(self.grammar)
    }
  }
}