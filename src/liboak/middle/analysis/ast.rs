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

pub use rust::{ExtCtxt,P,Item,Attribute};
pub use identifier::*;
pub use front::ast::Rule;
pub use monad::partial::Partial;

use front::ast::Grammar as FGrammar;
use std::collections::HashMap;

pub struct Grammar{
  pub name: Ident,
  pub rules: HashMap<Ident, Rule>,
  pub rust_items: HashMap<Ident, P<Item>>,
  pub attributes: Vec<Attribute>
}

impl Grammar
{
  pub fn new(fgrammar: &FGrammar) -> Partial<Grammar>
  {
    let rules_len = fgrammar.rules.len();
    let rust_items_len = fgrammar.rust_items.len();
    let grammar = Grammar {
      name: fgrammar.name.clone(),
      rules: HashMap::with_capacity(rules_len),
      rust_items: HashMap::with_capacity(rust_items_len),
      attributes: fgrammar.attributes.clone()
    };
    Partial::Value(grammar)
  }

  pub fn with_rules(self, rules: HashMap<Ident, Rule>) -> Grammar
  {
    Grammar {
      name: self.name,
      rules: rules,
      rust_items: self.rust_items,
      attributes: self.attributes
    }
  }

  pub fn with_rust_items(self, rust_items: HashMap<Ident, P<Item>>) -> Grammar
  {
    Grammar {
      name: self.name,
      rules: self.rules,
      rust_items: rust_items,
      attributes: self.attributes
    }
  }
}
