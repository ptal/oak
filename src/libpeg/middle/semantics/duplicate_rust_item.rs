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

use rust::{ExtCtxt, P, Item};

use middle::semantics::ast::Grammar;
use middle::semantics::duplicate::*;
use monad::partial::Partial;

pub fn rust_item_duplicate<'a>(cx: &'a ExtCtxt<'a>, grammar: Grammar,
  items: Vec<P<Item>>) -> Partial<Grammar>
{
  DuplicateItem::analyse(cx, items.into_iter(), String::from("rust item"))
    .map(move |rust_items| grammar.with_rust_items(rust_items))
}
