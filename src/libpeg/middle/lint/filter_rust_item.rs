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

use front::ast::*;
use monad::partial::Partial;
use rust::{P, Item, ExtCtxt};
use rustc::ast_map::blocks::MaybeFnLike;

pub struct FilterRustItem;

impl FilterRustItem
{
  pub fn analyse(cx: &ExtCtxt, mut grammar: Grammar) -> Partial<Grammar>
  {
    grammar.rust_items = grammar
      .rust_items.into_iter()
      .filter(|item| FilterRustItem::filter(cx, item))
      .collect();
    Partial::Value(grammar)
  }

  fn filter(cx: &ExtCtxt, item: &P<Item>) -> bool {
    if !item.is_fn_like() {
      FilterRustItem::warn_ignored_item(cx, item);
      false
    } else {
      true
    }
  }

  fn warn_ignored_item(cx: &ExtCtxt, item: &P<Item>) {
    cx.span_warn(item.span, format!(
      "`{}` is not a function and will be ignored.",
      item.ident.as_str()).as_str());
  }
}
