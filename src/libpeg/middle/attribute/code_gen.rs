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
use rust::{ExtCtxt, InternedString, MetaList, MetaWord, Span};
use middle::attribute::attribute::*;
use std::collections::hashmap::HashMap;

pub struct CodeGeneration
{
  pub ast: bool,
  pub parser: bool
}

pub struct CodeGenerationBuilder<'a>
{
  gen_lvl_to_attr: HashMap<InternedString, AttributeInfo<bool>>,
  gen: InternedString,
  cx: &'a ExtCtxt<'a>
}

impl<'a> CodeGenerationBuilder<'a>
{
  pub fn new(cx: &'a ExtCtxt) -> CodeGenerationBuilder<'a>
  {
    let mut gen_lvl_to_attr = HashMap::new();
    let gen_levels = vec!["parser", "ast"];
    for lvl in gen_levels.iter() {
      gen_lvl_to_attr.insert(
        InternedString::new(*lvl),
        AttributeInfo::new(false));
    }
    CodeGenerationBuilder {
      gen_lvl_to_attr: gen_lvl_to_attr,
      gen: InternedString::new("disable_code"),
      cx: cx
    }
  }

  pub fn from_attr(&mut self, attr: &rust::Attribute) -> bool
  {
    let levels = match attr.node.value.node {
      MetaList(ref head, ref tail) if *head == self.gen => tail,
      _ => return true
    };

    for level in levels.iter() {
      match level.node {
        MetaWord(ref gen_level) => self.insert_level(gen_level, level.span),
        _ => self.cx.parse_sess.span_diagnostic.span_err(level.span,
               "unknown attribute in the code-to-generate list.")
      }
    }
    false
  }

  fn insert_level(&mut self, level: &InternedString, span: Span)
  {
    let mut gen_attr = self.gen_lvl_to_attr.find_mut(level);
    match gen_attr {
      None => self.cx.parse_sess.span_diagnostic.span_err(span,
        format!("Unknown code-to-generate `{}`. The different code generation attributes are \
          `parser`, `ast`. For example: `#![disable_code(parser)]`.",
          level.get()).as_slice()),
      Some(ref attr_info) if attr_info.has_value() => {
        self.cx.parse_sess.span_diagnostic.span_warn(span,
          format!("The code-to-generate attribute `{}` is already set.", level.get()).as_slice());
        self.cx.parse_sess.span_diagnostic.span_note(attr_info.span,
          "Previous declaration here.");
      },
      Some(ref mut attr_info) => {
        attr_info.set(true, span);
      }
    }
  }

  pub fn build(&self) -> CodeGeneration
  {
    CodeGeneration {
      parser: !self.value_of("parser"),
      ast: !self.value_of("ast")
    }
  }

  fn value_of(&self, level: &'static str) -> bool
  {
    let level = &InternedString::new(level);
    self.gen_lvl_to_attr.find(level).unwrap().value_or_default()
  }
}
