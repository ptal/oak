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
use back::compiler::str_literal::*;
use back::code_printer::*;
use back::name_factory::*;
use rust;

mod str_literal;

pub struct Context<'a: 'c, 'b: 'a, 'c>
{
  pub grammar: &'c TGrammar<'a, 'b>,
  pub name_factory: &'c mut NameFactory,
  pub success: RExpr,
  pub failure: RExpr
}

impl<'a, 'b, 'c> Context<'a, 'b, 'c>
{
  pub fn new(grammar: &'c TGrammar<'a, 'b>,
    name_factory: &'c mut NameFactory,
    success: RExpr, failure: RExpr) -> Context<'a, 'b, 'c>
  {
    Context {
      grammar: grammar,
      name_factory: name_factory,
      success: success,
      failure: failure
    }
  }
}

pub trait CompileParser
{
  fn compile_parser<'a, 'b, 'c>(&self, context: Context<'a, 'b, 'c>) -> RExpr;
  fn compile_recognizer<'a, 'b, 'c>(&self, context: Context<'a, 'b, 'c>) -> RExpr;
}

pub fn expression_compiler(grammar: &TGrammar, idx: usize) -> Box<CompileParser> {
  match grammar.expr_by_index(idx) {
    StrLiteral(lit) => Box::new(StrLiteralCompiler::new(lit)),
    _ => unimplemented!()
    // AnySingleChar =>
    // NonTerminalSymbol(id) =>
    // Sequence(seq) =>
    // Choice(choices) =>
    // ZeroOrMore(expr) =>
    // OneOrMore(expr) =>
    // Optional(expr) =>
    // NotPredicate(expr) =>
    // AndPredicate(expr) =>
    // CharacterClass(char_class) =>
    // SemanticAction(expr, id) =>
  }
}

struct GrammarCompiler<'a, 'b: 'a>
{
  grammar: TGrammar<'a, 'b>
}

impl<'a, 'b> GrammarCompiler<'a, 'b>
{
  pub fn compile(grammar: TGrammar<'a, 'b>) -> Box<rust::MacResult + 'a> {
    let mut compiler = GrammarCompiler::new(grammar);
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
        // #![allow(dead_code)]
        // #![allow(unused_parens, unused_variables, unused_mut, unused_imports)]
        // use oak_runtime::parse_state::MergeSuccess;

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
          span: rust::DUMMY_SP
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
    let mut name_factory = NameFactory::new();
    self.grammar.rules.values().cloned()
      .flat_map(|rule| self.compile_rule(rule, &mut name_factory).into_iter())
      .collect()
  }

  fn compile_rule(&self, rule: Rule, name_factory: &mut NameFactory) -> Vec<RItem> {
    let recognizer_fn_name = name_factory.recognizer_name(self.cx(), rule.ident());
    let expr_compiler = expression_compiler(&self.grammar, rule.expr_idx);
    let success =
      if self.grammar[rule.expr_idx].ty.is_unit() {
        quote_expr!(self.cx(), state)
      }
      else {
        quote_expr!(self.cx(), state)//state.to_data($data))
      };
    let context = Context::new(
      &self.grammar,
      name_factory,
      success,
      quote_expr!(self.cx(),
        state
      )
    );
    let recognizer_body = expr_compiler.compile_recognizer(context);
    let unit_ty = quote_ty!(self.cx(), ());
    vec![
      self.function(recognizer_fn_name, recognizer_body, unit_ty)
    ]
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
