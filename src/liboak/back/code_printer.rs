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

use rust;
use back::ast::*;
use middle::attribute::ast::PrintAttribute;

pub fn print_code(cx: &ExtCtxt, print_attr: PrintAttribute, grammar_module: &RItem) {
  if print_attr.debug_api() {
    cx.parse_sess.span_diagnostic.handler.note(
      rust::item_to_string(grammar_module).as_str());
  }
  else if print_attr.show_api() {
    if let &rust::Item_::ItemMod(ref module) = &grammar_module.node {
      let mut res = String::new();
      for item in &module.items {
        if item.vis == rust::Visibility::Public {
          if let &rust::Item_::ItemFn(ref decl, unsafety, constness, abi, ref generics, _) = &item.node {
            res.extend(rust::to_string(|s| {
              try!(s.head("\n"));
              try!(s.print_fn(decl, unsafety, constness, abi, Some(item.ident), generics, None, item.vis));
              try!(s.end());
              s.end()
            }).chars());
          }
        }
      }
      cx.parse_sess.span_diagnostic.handler.note(res.as_str());
    } else {
      panic!("Expected the grammar module.")
    }
  }
}
