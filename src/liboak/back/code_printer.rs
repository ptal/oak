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

use syn::parse::*;
use quote::quote;

struct Printer {
  result: String
}

impl Parse for Printer {
  fn parse(ps: ParseStream) -> Result<Self> {
    let mut r = Printer { result: String::new() };
    while !ps.is_empty() {
      let item: syn::Item = ps.parse()?;
      r.print_item(item);
    }
    Ok(r)
  }
}

impl Printer {
  fn print_item(&mut self, item: syn::Item) {
    match item {
      syn::Item::Fn(mut item_fn) => {
        item_fn.block.stmts = vec![];
        self.result.push_str(format!("{}\n", quote!(#item_fn)).as_str())
      }
      _ => self.result.push_str(format!("{}\n", quote!(#item)).as_str())
    }
  }
}

pub fn print_code(grammar: &TGrammar, items: &proc_macro2::TokenStream) {
  let print_code = grammar.attributes.print_code;
  if print_code.debug() {
    grammar.start_span.unstable().note(format!("{}", items).as_str()).emit();
  }
  else if print_code.show() {
    let items = proc_macro::TokenStream::from(items.clone());
    match syn::parse::<Printer>(items) {
      Ok(p) => { grammar.start_span.unstable().note(p.result).emit(); },
      Err(e) => { let _ = e.into_compile_error(); }
    }
  }
}
