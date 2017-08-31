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

use middle::typing::ast::*;
use back::code_printer::*;
use back::compiler::rule::*;
use rust;

pub struct GrammarCompiler<'a, 'b: 'a>
{
  grammar: TGrammar<'a, 'b>
}

impl<'a, 'b> GrammarCompiler<'a, 'b>
{
  pub fn compile(grammar: TGrammar<'a, 'b>) -> Box<rust::MacResult + 'a> {
    let compiler = GrammarCompiler::new(grammar);
    let mod_content = compiler.compile_mod_content();
    let module = compiler.compile_grammar_module(mod_content);
    print_code(&compiler.grammar, &module);
    rust::MacEager::items(rust::SmallVector::one(module))
  }

  fn new(grammar: TGrammar<'a, 'b>) -> GrammarCompiler<'a, 'b> {
    GrammarCompiler {
      grammar: grammar
    }
  }

  fn compile_grammar_module(&self, module_content: Vec<RItem>) -> RItem {
    let grammar_name = self.grammar.name;
    let module = quote_item!(self.cx(),
      pub mod $grammar_name
      {
        #![allow(unused_mut)]
        use oak_runtime::stream::*;
        #[allow(unused_imports)]
        use oak_runtime::str_stream::StrStream;
        #[allow(unused_imports)]
        use std::ops::Range;

        $module_content
      }
    ).expect("Quote the grammar module.");
    self.insert_runtime_crate(module)
  }

  // RUST BUG: We cannot quote `extern crate oak_runtime;` before the grammar module, so we use this workaround
  // for adding the external crate after the creation of the module.
  fn insert_runtime_crate(&self, module: RItem) -> RItem {
    let runtime_crate = quote_item!(self.cx(),
      extern crate oak_runtime;
    ).expect("Quote the extern PEG crate.");

    match &module.node {
      &rust::ItemKind::Mod(ref module_code) => {
        let mut items = vec![runtime_crate];
        items.extend_from_slice(module_code.items.clone().as_slice());
        rust::P(rust::Item {
          ident: module.ident,
          attrs: module.attrs.clone(),
          id: rust::DUMMY_NODE_ID,
          node: rust::ItemKind::Mod(rust::Mod{
            inner: rust::DUMMY_SP,
            items: items
          }),
          vis: rust::Visibility::Public,
          span: rust::DUMMY_SP,
          tokens: None
        })
      },
      _ => unreachable!()
    }
  }

  fn compile_mod_content(&self) -> Vec<RItem> {
    let mut mod_content = self.compile_rules();
    mod_content.extend(self.grammar.rust_items.clone().into_iter());
    mod_content.extend(self.grammar.rust_functions.values().cloned());
    mod_content
  }

  fn compile_rules(&self) -> Vec<RItem> {
    self.grammar.rules.iter()
      .flat_map(|&rule| RuleCompiler::compile(&self.grammar, rule).into_iter())
      .collect()
  }

  fn cx(&self) -> &'a ExtCtxt<'b> {
    &self.grammar.cx
  }
}

