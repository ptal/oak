// Copyright 2016 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub use middle::typing::ast::*;
use back::compiler::rule::*;

use quote::quote;

pub struct GrammarCompiler
{
  grammar: TGrammar
}

impl GrammarCompiler
{
  pub fn compile(grammar: TGrammar) -> proc_macro2::TokenStream {
    let compiler = GrammarCompiler::new(grammar);
    let mod_content = compiler.compile_mod_content();
    let result = compiler.compile_grammar_module(mod_content);
    result
  }

  fn new(grammar: TGrammar) -> GrammarCompiler {
    GrammarCompiler {
      grammar: grammar
    }
  }

  fn compile_grammar_module(&self, module_content: Vec<syn::Item>) -> proc_macro2::TokenStream {
    quote!(
      // #![allow(unused_mut)]
      #[allow(unused_imports)]
      use oak_runtime::stream::*;
      #[allow(unused_imports)]
      use oak_runtime::str_stream::StrStream;
      #[allow(unused_imports)]
      use std::ops::Range;

      #(#module_content)*
    )
  }

  fn compile_mod_content(&self) -> Vec<syn::Item> {
    let mut mod_content = self.grammar.rust_items.clone();
    mod_content.extend(self.compile_rules().into_iter());
    mod_content.extend(self.grammar.rust_functions.values().cloned()
      .map(syn::Item::Fn));
    mod_content
  }

  fn compile_rules(&self) -> Vec<syn::Item> {
    self.grammar.rules.iter()
      .flat_map(|rule| RuleCompiler::compile(&self.grammar, rule.clone()).into_iter())
      .collect()
  }
}
