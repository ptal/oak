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

use middle::typing::ast::*;
use rust;
use rust::{State, PrintState, Visibility, Mod};
use std::io;

pub fn print_code(grammar: &TGrammar, grammar_module: &RItem) {
  let print_code = grammar.attributes.print_code;
  if print_code.debug() {
    grammar.cx.parse_sess.span_diagnostic.note_without_error(
      rust::item_to_string(grammar_module).as_str());
  }
  else if print_code.show() {
    if let &rust::ItemKind::Mod(ref module) = &grammar_module.node {
      let res = rust::to_string(|s| {
        print_module(s, module, grammar_module.ident, grammar_module.vis.clone(), grammar_module.span)
      });
      grammar.cx.parse_sess.span_diagnostic.note_without_error(res.as_str());
    } else {
      panic!("Expected the grammar module.");
    }
  }
}

fn print_module(s: &mut State, module: &Mod, ident: Ident, vis: Visibility, span: Span)
  -> io::Result<()>
{
  s.head(&rust::visibility_qualified(&vis, "mod"))?;
  s.print_ident(ident)?;
  s.nbsp()?;
  s.bopen()?;

  for item in &module.items {
    print_visible_fn(s, item)?;
  }
  s.bclose(span)
}

fn print_visible_fn(s: &mut State, item: &RItem) -> io::Result<()> {
  if item.vis.node == rust::VisibilityKind::Public {
    if let &rust::ItemKind::Fn(ref decl, unsafety, constness, abi, ref generics, _) = &item.node {
      s.hardbreak_if_not_bol()?;
      s.head("")?;
      s.print_fn(decl, unsafety, constness.node, abi, Some(item.ident), generics, &item.vis)?;
      s.end()?; // end head-ibox
      s.writer().word(";")?;
      s.end()?; // end the outer fn box
    }
  }
  Ok(())
}
