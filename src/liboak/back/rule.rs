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
use back::rtype::*;
use back::value::*;

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
      compiler.compile_recognizer(name_factory),
      compiler.compile_parser(name_factory)
    ]
  }

  fn new(grammar: &'c TGrammar<'a, 'b>, rule: Rule) -> RuleCompiler<'a, 'b, 'c> {
    RuleCompiler {
      grammar: grammar,
      rule: rule
    }
  }

  fn compile_recognizer(&self, name_factory: &mut NameFactory) -> RItem {
    let fn_name = self.recognizer_name(name_factory);
    let success = quote_expr!(self.cx(), state.success(()));
    let body = self.compile_expr(name_factory, recognizer_compiler, success);
    self.unit_function(fn_name, true, body)
  }

  fn compile_parser(&self, name_factory: &mut NameFactory) -> RItem {
    let fn_name = self.parser_name(name_factory);
    if self.parser_equals_recognizer() {
      let recognizer_fn = self.recognizer_name(name_factory);
      self.unit_function(fn_name, false,
        quote_expr!(self.cx(), $recognizer_fn(state)))
    }
    else {
      let values_names = self.open_namespace(name_factory);
      let value = tuple_value(self.grammar, self.rule.expr_idx, values_names);
      let success = quote_expr!(self.cx(), state.success($value));
      let body = self.compile_expr(name_factory, parser_compiler, success);
      name_factory.close_namespace();
      let ty = TypeCompiler::compile(self.grammar, self.rule.expr_idx);
      self.function(fn_name, true, body, ty)
    }
  }

  fn compile_expr(&self, name_factory: &mut NameFactory,
    compiler_fn: ExprCompilerFn, success: RExpr) -> RExpr
  {
    let compiler = compiler_fn(&self.grammar, self.rule.expr_idx);
    let mut context = Context::new(&self.grammar, name_factory);
    let failure = quote_expr!(self.cx(), state.failure());
    compiler.compile_expr(&mut context, Continuation::new(success, failure))
  }

  fn parser_equals_recognizer(&self) -> bool {
    self.grammar[self.rule.expr_idx].ty.is_unit()
  }

  fn parser_name(&self, name_factory: &mut NameFactory) -> Ident {
    name_factory.parser_name(self.cx(), self.rule.ident())
  }

  fn recognizer_name(&self, name_factory: &mut NameFactory) -> Ident {
    name_factory.recognizer_name(self.cx(), self.rule.ident())
  }

  fn open_namespace(&self, name_factory: &mut NameFactory) -> Vec<Ident> {
    let cardinality = self.grammar[self.rule.expr_idx].type_cardinality();
    name_factory.open_namespace(self.grammar.cx, cardinality)
  }

  fn unit_function(&self, name: Ident, state_mut: bool, body: RExpr) -> RItem {
    self.function(name, state_mut, body, quote_ty!(self.cx(), ()))
  }

  #[allow(unused_imports)] // `quote_tokens` generates a warning.
  fn function(&self, name: Ident, state_mut: bool, body: RExpr, ty: RTy) -> RItem {
    let mut_kw = if state_mut {
      Some(quote_tokens!(self.cx(), mut))
    } else {
      None
    };
    quote_item!(self.cx(),
      #[inline]
      pub fn $name<S>($mut_kw state: oak_runtime::ParseState<S, ()>) -> oak_runtime::ParseState<S, $ty> where
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
