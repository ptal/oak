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

pub struct RuleCompiler<'a: 'c, 'b: 'a, 'c>
{
  grammar: &'c TGrammar<'a, 'b>,
  rule: Rule
}

impl<'a, 'b, 'c> RuleCompiler<'a, 'b, 'c>
{
  pub fn compile(grammar: &'c TGrammar<'a, 'b>, rule: Rule, name_factory: &mut NameFactory) -> Vec<RItem> {
    let compiler = RuleCompiler::new(grammar, rule);
    vec![
      compiler.compile_recognizer(name_factory)
    ]
  }

  fn new(grammar: &'c TGrammar<'a, 'b>, rule: Rule) -> RuleCompiler<'a, 'b, 'c> {
    RuleCompiler {
      grammar: grammar,
      rule: rule
    }
  }

  fn compile_recognizer(&self, name_factory: &mut NameFactory) -> RItem {
    let recognizer_fn_name = name_factory.recognizer_name(self.cx(), self.rule.ident());
    let compiler = recognizer_compiler(&self.grammar, self.rule.expr_idx);
    let success =
      if self.grammar[self.rule.expr_idx].ty.is_unit() {
        quote_expr!(self.cx(), state.success(()))
      }
      else {
        quote_expr!(self.cx(), state)//state.success($data))
      };
    let context = Context::new(
      &self.grammar,
      name_factory,
      success,
      quote_expr!(self.cx(),
        state
      )
    );
    let recognizer_body = compiler.compile_expr(context);
    let unit_ty = quote_ty!(self.cx(), ());
    self.function(recognizer_fn_name, recognizer_body, unit_ty)
  }

  fn function(&self, name: Ident, body: RExpr, ty: RTy) -> RItem {
    quote_item!(self.cx(),
      #[inline]
      pub fn $name<S>(mut state: oak_runtime::ParseState<S, ()>) -> oak_runtime::ParseState<S, $ty> where
       S: oak_runtime::CharStream
      {
        $body
      }
    ).expect("Quotation of a generated function.")
  }

  fn cx(&self) -> &'a ExtCtxt<'b> {
    &self.grammar.cx
  }
}
