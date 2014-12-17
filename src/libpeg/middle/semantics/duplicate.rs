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

pub use rust::{ExtCtxt,Span};
pub use rust;
pub use identifier::*;
pub use std::collections::HashMap;
pub use monad::partial::Partial;

use middle::semantics::ast::*;
use monad::partial::Partial::*;

pub trait ItemIdent {
  fn ident(&self) -> Ident;
}

pub trait ItemSpan {
  fn span(&self) -> Span;
}

impl ItemIdent for rust::Item {
  fn ident(&self) -> Ident {
    self.ident.clone()
  }
}

impl ItemSpan for rust::Item {
  fn span(&self) -> Span {
    self.span.clone()
  }
}

impl<InnerItem: ItemIdent> ItemIdent for rust::P<InnerItem> {
  fn ident(&self) -> Ident {
    self.deref().ident()
  }
}

impl<InnerItem: ItemSpan> ItemSpan for rust::P<InnerItem> {
  fn span(&self) -> Span {
    self.deref().span()
  }
}

impl ItemIdent for Rule {
  fn ident(&self) -> Ident {
    self.name.node.clone()
  }
}

impl ItemSpan for Rule {
  fn span(&self) -> Span {
    self.name.span.clone()
  }
}

pub struct DuplicateItem<'a, Item>
{
  cx: &'a ExtCtxt<'a>,
  items: HashMap<Ident, Item>,
  has_duplicate: bool,
  what_is_duplicate: String
}

impl<'a, Item: ItemIdent + ItemSpan> DuplicateItem<'a, Item>
{
  pub fn analyse<ItemIter: Iterator<Item>>(cx: &'a ExtCtxt<'a>, iter: ItemIter, item_kind: String) -> Partial<HashMap<Ident, Item>>
  {
    let (min_size, _) = iter.size_hint();
    DuplicateItem {
      cx: cx,
      items: HashMap::with_capacity(min_size),
      has_duplicate: false,
      what_is_duplicate: item_kind
    }.populate(iter)
     .make()
  }

  fn populate<ItemIter: Iterator<Item>>(mut self, iter: ItemIter) -> DuplicateItem<'a, Item>
  {
    let mut iter = iter;
    for item in iter {
      let ident = item.ident();
      if self.items.contains_key(&ident) {
        self.duplicate_items(self.items.get(&ident).unwrap(), item);
        self.has_duplicate = true;
      } else {
        self.items.insert(ident, item);
      }
    }
    self
  }

  fn duplicate_items(&self, pre: &Item, current: Item)
  {
    self.cx.span_err(current.span(), format!(
      "duplicate definition of {} `{}`",
      self.what_is_duplicate, current.ident()).as_slice());
    self.cx.span_note(pre.span(), format!(
      "previous definition of {} `{}` here",
      self.what_is_duplicate, pre.ident()).as_slice());
  }

  fn make(self) -> Partial<HashMap<Ident, Item>>
  {
    if self.has_duplicate {
      Fake(self.items)
    } else {
      Value(self.items)
    }
  }
}