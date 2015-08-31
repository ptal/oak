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

//! This part generates the Rust code from the AST built during the previous phases. For a single expression, two functions can be generated: recognizer and parser. The difference is that recognizer does not build any value while parser does.
//!
//! Semantics actions `expr > f` are compiled into `f(expr)` with `expr` expanded if `expr` is a tuple. Semantics actions are not called in recognizers.

use rust;
use rust::AstBuilder;
use back::ast::*;
use back::naming::*;
use back::function::*;
use back::code_printer::*;

use std::iter::*;

pub fn generate_rust_code<'cx>(cx: &'cx ExtCtxt, grammar: Grammar)
  -> Box<rust::MacResult + 'cx>
{
  CodeGenerator::compile(cx, grammar)
}

fn map_foldr<T, U, V, F, G>(data: Vec<T>, accu: V, f: F, g: G) -> V where
 F: Fn(T) -> U,
 G: Fn(V, U) -> V
{
  let data = data.into_iter()
    .map(f)
    .rev();
  data.fold(accu, g)
}

fn map_foldr_init<T, U, V, I, F, G>(data: Vec<T>, init: I, f: F, g: G) -> V where
 I: FnOnce(U) -> V,
 F: Fn(T) -> U,
 G: Fn(V, U) -> V
{
  let mut data = data.into_iter()
    .map(f)
    .rev();
  let accu = data.next().expect("map_foldr_init expect at least one element.");
  data.fold(init(accu), g)
}


struct CodeGenerator<'cx>
{
  cx: &'cx ExtCtxt<'cx>,
  function_gen: FunctionGenerator<'cx>,
  current_rule_name: Ident
}

impl<'cx> CodeGenerator<'cx>
{
  fn compile(cx: &'cx ExtCtxt, grammar: Grammar) -> Box<rust::MacResult + 'cx> {
    let mut compiler = CodeGenerator {
      cx: cx,
      function_gen: FunctionGenerator::new(cx),
      current_rule_name: *grammar.rules.keys().next().unwrap()
    };
    compiler.compile_peg(&grammar)
  }

  fn compile_peg(&mut self, grammar: &Grammar) -> Box<rust::MacResult + 'cx> {
    let parser = self.compile_parser(grammar);
    let grammar_module = self.compile_grammar_module(grammar, parser);
    print_code(self.cx, grammar.attributes.print_attr, &grammar_module);
    rust::MacEager::items(rust::SmallVector::one(grammar_module))
  }

  fn compile_grammar_module(&self, grammar: &Grammar, parser: Vec<RItem>) -> RItem {
    let grammar_name = grammar.name;
    let grammar_module = quote_item!(self.cx,
      pub mod $grammar_name
      {
        #![allow(dead_code)]
        #![allow(unused_parens, unused_variables, unused_mut)]

        $parser
      }
    ).expect("Quote the grammar module.");
    self.insert_runtime_crate(grammar_module)
  }

  // RUST BUG: We cannot quote `extern crate oak_runtime;` before the grammar module, so we use this workaround
  // for adding the external crate after the creation of the module.
  fn insert_runtime_crate(&self, grammar_module: RItem) -> RItem {
    let runtime_crate = quote_item!(self.cx,
      extern crate oak_runtime;
    ).expect("Quote the extern PEG crate.");

    match &grammar_module.node {
      &rust::ItemMod(ref module_code) => {
        let mut items = vec![runtime_crate];
        items.push_all(module_code.items.clone().as_slice());
        rust::P(rust::Item {
          ident: grammar_module.ident,
          attrs: grammar_module.attrs.clone(),
          id: rust::DUMMY_NODE_ID,
          node: rust::ItemMod(rust::Mod{
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

  fn compile_parser(&mut self, grammar: &Grammar) -> Vec<RItem> {
    self.compile_rules(grammar);
    let mut rust_code: Vec<RItem> = grammar.rust_items.values().cloned().collect();
    rust_code.extend(self.function_gen.code().into_iter());
    rust_code
  }

  fn compile_rules(&mut self, grammar: &Grammar) {
    for rule in grammar.rules.values() {
      self.current_rule_name = rule.name.node;
      let expr_fn = self.visit_expr(&rule.def);
      self.function_gen.generate_rule(rule.def.kind(), self.current_rule_name, expr_fn);
    }
  }

  fn compile_star(&mut self, parent: &Box<Expression>, expr: &Box<Expression>,
    result: RExpr) -> GenFunNames
  {
    let cx = self.cx;
    let GenFunNames{recognizer, parser} = self.visit_expr(expr);
    let recognizer_init = quote_expr!(cx, oak_runtime::ParseState::stateless(stream.clone()));
    let parser_init = quote_expr!(cx, oak_runtime::ParseState::success(stream.clone(), vec![]));
    let recognizer_body = self.compile_star_body(recognizer, recognizer_init, result.clone());
    let parser_body = self.compile_star_body(parser, parser_init, result);
    self.function_gen.generate_expr("star", self.current_rule_name, parent.kind(),
      recognizer_body,
      parser_body)
  }

  fn compile_star_body(&self, expr: Ident, result_init: RExpr, result: RExpr) -> RExpr {
    quote_expr!(self.cx, {
      let mut state = $result_init;
      while state.has_successor() {
        let next = $expr(state.stream());
        state = state.join(next.error);
        if let Some(success) = next.success {
          state.merge_success(success);
        }
        else {
          break;
        }
      }
      $result
    })
  }

  fn compile_sequence_recognizer_body(&self, exprs: Vec<GenFunNames>) -> RExpr {
    map_foldr_init(exprs,
      |name: Ident| quote_expr!(self.cx, $name(stream)),
      |name| name.recognizer,
      |accu: RExpr, name: Ident| {
        quote_expr!(self.cx, $name(stream).and_then(|success| {
          let stream = success.stream;
          $accu
        }))
      }
    )
  }

  fn compile_sequence_parser_body(&self, parent: &Box<Expression>, exprs: Vec<GenFunNames>) -> RExpr {
    let state_names: Vec<Ident> = exprs.iter().enumerate()
      .map(|(idx, _)| {
        rust::gensym_ident(format!("state{}", idx).as_str())
      })
      .rev()
      .collect();

    let return_value = self.compile_sequence_result(parent, &state_names);

    map_foldr(exprs,
      (return_value, state_names.len()),
      |name| name.parser,
      |(accu, state_idx): (RExpr, usize), name: Ident| {
        let state_idx = state_idx - 1;
        let state_name = state_names[state_idx];
        (
          quote_expr!(self.cx, $name(stream).and_then(|$state_name| {
            let stream = $state_name.stream.clone();
            $accu
          })),
          state_idx
        )
      }
    ).0
  }

  fn compile_sequence_result(&self, parent: &Box<Expression>, state_names: &Vec<Ident>) -> RExpr {
    let tuple_indexes = parent.tuple_indexes();

    let mut tuple_result: Vec<RExpr> = tuple_indexes.into_iter()
      .map(|idx| state_names[idx])
      .map(|name| quote_expr!(self.cx, $name.data))
      .collect();

    let result =
      if tuple_result.len() == 1 {
        tuple_result.pop().unwrap()
      } else {
        self.cx.expr_tuple(parent.span, tuple_result)
      };
    quote_expr!(self.cx, oak_runtime::ParseState::success(stream, $result))
  }

  fn compile_semantic_action_call(&self, parent: &Box<Expression>,
    expr: &Box<Expression>, action_name: Ident) -> RExpr
  {
    let ty = expr.ty.clone();
    let access_data = quote_expr!(self.cx, data);
    let action_params = match ty {
      ExprTy::Tuple(ref indexes) if indexes.len() > 1 => {
        indexes.iter()
          .map(|&idx| self.cx.expr_tup_field_access(parent.span, access_data.clone(), idx))
          .collect()
      },
      ExprTy::Tuple(ref indexes) if indexes.len() == 0 => {
        vec![]
      }
      _ => {
        vec![access_data]
      }
    };
    self.cx.expr_call_ident(parent.span, action_name, action_params)
  }
}

impl<'cx> Visitor<Expression, GenFunNames> for CodeGenerator<'cx>
{
  fn visit_str_literal(&mut self, parent: &Box<Expression>, lit_str: &String) -> GenFunNames {
    let lit_str = lit_str.as_str();
    self.function_gen.generate_expr("str_literal", self.current_rule_name, parent.kind(),
      quote_expr!(self.cx, oak_runtime::recognize_match_literal(stream, $lit_str)),
      quote_expr!(self.cx, oak_runtime::parse_match_literal(stream, $lit_str))
    )
  }

  fn visit_non_terminal_symbol(&mut self, _parent: &Box<Expression>, rule_id: Ident) -> GenFunNames {
    self.function_gen.names_of_rule(rule_id)
  }

  fn visit_character(&mut self, _parent: &Box<Expression>) -> GenFunNames {
    unreachable!();
  }

  fn visit_any_single_char(&mut self, parent: &Box<Expression>) -> GenFunNames {
    self.function_gen.generate_expr("any_single_char", self.current_rule_name, parent.kind(),
      quote_expr!(self.cx, oak_runtime::recognize_any_single_char(stream)),
      quote_expr!(self.cx, oak_runtime::parse_any_single_char(stream))
    )
  }

  fn visit_character_class(&mut self, parent: &Box<Expression>, classes: &CharacterClassExpr) -> GenFunNames {
    let cx = self.cx;
    let mut seq_it = classes.intervals.iter();

    let CharacterInterval{lo, hi} = *seq_it.next()
      .expect("Empty character intervals should be forbidden at the parsing stage.");
    let cond = seq_it.fold(quote_expr!(cx, (current >= $lo && current <= $hi)),
      |accu, &CharacterInterval{lo, hi}| {
        quote_expr!(cx, $accu || (current >= $lo && current <= $hi))
      }
    );

    let classes_desc = format!("{}", classes);
    let classes_desc_str = classes_desc.as_str();

    let make_char_class_body = |result: RExpr| quote_expr!(cx, {
      match stream.next() {
        Some(current) if $cond => {
          $result
        }
        _ => {
          oak_runtime::ParseState::error(stream, $classes_desc_str)
        }
      }
    });

    self.function_gen.generate_expr("class_char", self.current_rule_name, parent.kind(),
      make_char_class_body(quote_expr!(cx, oak_runtime::ParseState::stateless(stream))),
      make_char_class_body(quote_expr!(cx, oak_runtime::ParseState::success(stream, current)))
    )
  }

  fn visit_sequence(&mut self, parent: &Box<Expression>, seq: &Vec<Box<Expression>>) -> GenFunNames {
    let exprs = walk_exprs(self, seq);

    let recognizer_body = self.compile_sequence_recognizer_body(exprs.clone());
    let parser_body = self.compile_sequence_parser_body(parent, exprs);

    self.function_gen.generate_expr("sequence", self.current_rule_name, parent.kind(),
      recognizer_body,
      parser_body)
  }

  fn visit_choice(&mut self, parent: &Box<Expression>, choices: &Vec<Box<Expression>>) -> GenFunNames {
    let exprs = walk_exprs(self, choices);

    let cx = self.cx;
    let init = |name: Ident| quote_expr!(cx, $name(stream));
    let make_body = |accu:RExpr, name:Ident| {
      quote_expr!(cx, $name(stream.clone()).or_else_join(|| $accu))
    };
    let recognizer_body = map_foldr_init(exprs.clone(),
      &init,
      |name| name.recognizer,
      &make_body
    );
    let parser_body = map_foldr_init(exprs,
      init,
      |name| name.parser,
      make_body
    );
    self.function_gen.generate_expr("choice", self.current_rule_name, parent.kind(),
      recognizer_body,
      parser_body)
  }

  fn visit_zero_or_more(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> GenFunNames {
    let cx = self.cx;
    let result = quote_expr!(cx, state);
    self.compile_star(parent, expr, result)
  }

  fn visit_one_or_more(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> GenFunNames {
    let cx = self.cx;
    let result = quote_expr!(cx, {
      if state.stream_eq(&stream) {
        state.to_error()
      } else {
        state
      }
    });
    self.compile_star(parent, expr, result)
  }

  fn visit_optional(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> GenFunNames {
    let GenFunNames{recognizer, parser} = self.visit_expr(expr);
    let recognizer_body = quote_expr!(self.cx,
      oak_runtime::optional_recognizer($recognizer(stream.clone()), stream)
    );
    let parser_body = quote_expr!(self.cx,
      oak_runtime::optional_parser($parser(stream.clone()), stream)
    );
    self.function_gen.generate_expr("optional", self.current_rule_name, parent.kind(),
      recognizer_body,
      parser_body)
  }

  fn visit_not_predicate(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> GenFunNames {
    let recognizer_name = self.visit_expr(expr).recognizer;
    let body = quote_expr!(self.cx,
      oak_runtime::not_predicate($recognizer_name(stream.clone()), stream)
    );
    self.function_gen.generate_unit_expr(
      "not_predicate", self.current_rule_name, parent.kind(), body)
  }

  fn visit_and_predicate(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> GenFunNames {
    let recognizer_name = self.visit_expr(expr).recognizer;
    let body = quote_expr!(self.cx,
      oak_runtime::and_predicate($recognizer_name(stream.clone()), stream)
    );
    self.function_gen.generate_unit_expr(
      "and_predicate", self.current_rule_name, parent.kind(), body)
  }

  fn visit_semantic_action(&mut self, parent: &Box<Expression>,
    expr: &Box<Expression>, action_name: Ident) -> GenFunNames
  {
    let GenFunNames{recognizer, parser} = self.visit_expr(expr);
    let recognizer_body = quote_expr!(self.cx,
      $recognizer(stream)
    );
    let action_call = self.compile_semantic_action_call(parent, expr, action_name);
    let parser_body = quote_expr!(self.cx,
      $parser(stream).map_data(|data| $action_call)
    );
    self.function_gen.generate_expr("semantic_action", self.current_rule_name, parent.kind(),
      recognizer_body,
      parser_body)
  }
}
