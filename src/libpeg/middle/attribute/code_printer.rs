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

pub struct CodePrinter
{
  pub info: bool,
  pub ast: bool,
  pub parser: bool
}

pub struct CodePrinterBuilder<'a>
{
  print_lvl_to_attr: HashMap<InternedString, AttributeInfo<bool>>,
  print: InternedString,
  cx: &'a ExtCtxt<'a>
}

impl<'a> CodePrinterBuilder<'a>
{
  pub fn new(cx: &'a ExtCtxt) -> CodePrinterBuilder<'a>
  {
    let mut print_lvl_to_attr = HashMap::new();
    let print_levels = vec!["parser", "ast", "code", "info", "all"];
    for lvl in print_levels.iter() {
      print_lvl_to_attr.insert(
        InternedString::new(*lvl),
        AttributeInfo::new(false));
    }
    CodePrinterBuilder {
      print_lvl_to_attr: print_lvl_to_attr,
      print: InternedString::new("print"),
      cx: cx
    }
  }

  pub fn from_attr(&mut self, attr: &rust::Attribute) -> bool
  {
    let levels = match attr.node.value.node {
      MetaList(ref head, ref tail) if *head == self.print => tail,
      _ => return true
    };

    for level in levels.iter() {
      match level.node {
        MetaWord(ref print_level) => self.insert_level(print_level, level.span),
        _ => self.cx.parse_sess.span_diagnostic.span_err(level.span,
               "unknown attribute in the print level list.")
      }
    }
    false
  }

  // check if the print_level is known and if it's not already used.
  fn insert_level(&mut self, level: &InternedString, span: Span)
  {
    let mut print_attr = self.print_lvl_to_attr.find_mut(level);
    match print_attr {
      None => self.cx.parse_sess.span_diagnostic.span_err(span,
        format!("Unknown print level `{}`. The different print levels are `parser`, \
          `ast`, `info`, `code`, `all`. For example: `#![print(code)]`.",
          level.get()).as_slice()),
      Some(ref attr_info) if attr_info.has_value() => {
        self.cx.parse_sess.span_diagnostic.span_warn(span,
          format!("The print level `{}` is already set.", level.get()).as_slice());
        self.cx.parse_sess.span_diagnostic.span_note(attr_info.span,
          "Previous declaration here.");
      },
      Some(ref mut attr_info) => {
        attr_info.set(true, span);
      }
    }
  }

  pub fn build(&self) -> CodePrinter
  {
    let info = self.value_of("info");
    let parser = self.value_of("parser");
    let ast = self.value_of("ast");
    let code = self.value_of("code");
    let all = self.value_of("all");
    CodePrinter {
      info: info || all,
      ast: ast || code || all,
      parser: parser || code || all
    }
  }

  fn value_of(&self, level: &'static str) -> bool
  {
    let level = &InternedString::new(level);
    self.print_lvl_to_attr.find(level).unwrap().value_or_default()
  }
}
