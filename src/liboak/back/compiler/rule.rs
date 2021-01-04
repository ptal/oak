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

use back::compiler::*;
use back::compiler::value::*;

pub struct RuleCompiler<'a>
{
  grammar: &'a TGrammar,
  rule: Rule
}

impl<'a> RuleCompiler<'a>
{
  pub fn compile(grammar: &'a TGrammar, rule: Rule) -> Vec<syn::Item> {
    let compiler = RuleCompiler::new(grammar, rule);
    vec![
      compiler.compile_recognizer(),
      compiler.compile_parser()
    ]
  }

  fn new(grammar: &'a TGrammar, rule: Rule) -> Self {
    RuleCompiler {
      grammar: grammar,
      rule: rule
    }
  }

  fn compile_recognizer(&self) -> syn::Item {
    let mut context = Context::new(self.grammar);
    let success = parse_quote!(state.success(()));
    let failure = parse_quote!(state.failure());

    let body = context.compile(recognizer_compiler,
      self.expr(), success, failure);

    context.into_recognizer_function(body, self.rule.clone())
  }

  fn compile_parser(&self) -> syn::Item {
    let mut context = Context::new(self.grammar);
    if self.parser_equals_recognizer() {
      context.into_parser_alias(self.rule.clone())
    }
    else {
      let scope = context.open_scope(self.expr());
      let vars = tuple_value(context.free_variables());

      let success = parse_quote!(state.success(#vars));
      let failure = parse_quote!(state.failure());
      let body = context.compile(parser_compiler,
        self.expr(), success, failure);

      context.close_scope(scope);
      context.into_parser_function(body, self.rule.clone())
    }
  }

  fn parser_equals_recognizer(&self) -> bool {
    self.grammar[self.expr()].ty == Type::Unit
  }

  fn expr(&self) -> usize {
    self.rule.expr_idx
  }
}
