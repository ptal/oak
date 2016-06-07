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

pub use std::collections::HashMap;
use front::ast::FRule;

use middle::analysis::ast::*;
use monad::partial::Partial::*;

use rust;

pub fn rule_duplicate<'a>(cx: &'a ExtCtxt<'a>, mut grammar: AGrammar,
  rules: Vec<FRule>) -> Partial<AGrammar>
{
  DuplicateItem::analyse(cx, rules.into_iter(), String::from("rule"))
  .map(|rules|
    rules.into_iter().map(|(id, frule)| (id, Rule::new(frule.name, frule.def))).collect())
  .map(move |rules| { grammar.rules = rules; grammar })
}

pub fn rust_functions_duplicate<'a>(cx: &'a ExtCtxt<'a>, mut grammar: AGrammar,
  items: Vec<RItem>) -> Partial<AGrammar>
{
  let mut functions = vec![];
  let mut others = vec![];
  for item in items {
    if let &rust::ItemKind::Fn(..) = &item.node {
      functions.push(item);
    }
    else {
      others.push(item);
    }
  }
  DuplicateItem::analyse(cx, functions.into_iter(), String::from("rust function"))
    .map(move |functions| {
      grammar.rust_functions = functions;
      grammar.rust_items = others;
      grammar
    })
}

struct DuplicateItem<'a, Item>
{
  cx: &'a ExtCtxt<'a>,
  items: HashMap<Ident, Item>,
  has_duplicate: bool,
  what_is_duplicated: String
}

impl<'a, Item> DuplicateItem<'a, Item> where
 Item: ItemIdent + ItemSpan
{
  pub fn analyse<ItemIter>(cx: &'a ExtCtxt<'a>, iter: ItemIter, item_kind: String)
    -> Partial<HashMap<Ident, Item>> where
   ItemIter: Iterator<Item=Item>
  {
    let (min_size, _) = iter.size_hint();
    DuplicateItem {
      cx: cx,
      items: HashMap::with_capacity(min_size),
      has_duplicate: false,
      what_is_duplicated: item_kind
    }.populate(iter)
     .make()
  }

  fn populate<ItemIter: Iterator<Item=Item>>(mut self, iter: ItemIter)
    -> DuplicateItem<'a, Item>
  {
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

  fn duplicate_items(&self, pre: &Item, current: Item) {
    let mut db = self.cx.struct_span_err(current.span(), format!(
      "duplicate definition of {} `{}`",
      self.what_is_duplicated, current.ident()).as_str());
    db.span_note(pre.span(), format!(
      "previous definition of {} `{}` here",
      self.what_is_duplicated, pre.ident()).as_str()).emit();
  }

  fn make(self) -> Partial<HashMap<Ident, Item>> {
    if self.has_duplicate {
      Fake(self.items)
    } else {
      Value(self.items)
    }
  }
}
