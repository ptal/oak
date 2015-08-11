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
use back::ast::Expression_::*;
use back::naming::*;
use back::function::*;
use back::code_printer::*;

use std::iter::*;

pub fn generate_rust_code<'cx>(cx: &'cx ExtCtxt, grammar: Grammar) -> Box<rust::MacResult + 'cx>
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

struct CodeGenerator<'cx>
{
  cx: &'cx ExtCtxt<'cx>,
  function_gen: FunctionGenerator<'cx>,
  current_rule_name: Ident
}

impl<'cx> CodeGenerator<'cx>
{
  fn compile(cx: &'cx ExtCtxt, grammar: Grammar) -> Box<rust::MacResult + 'cx>
  {
    let mut compiler = CodeGenerator {
      cx: cx,
      function_gen: FunctionGenerator::new(cx),
      current_rule_name: *grammar.rules.keys().next().unwrap()
    };
    compiler.compile_peg(&grammar)
  }

  fn compile_peg(&mut self, grammar: &Grammar) -> Box<rust::MacResult + 'cx>
  {
    let parser = self.compile_parser(grammar);
    let grammar_module = self.compile_grammar_module(grammar, parser);
    print_code(self.cx, grammar.attributes.print_attr, &grammar_module);
    rust::MacEager::items(rust::SmallVector::one(grammar_module))
  }

  fn compile_grammar_module(&self, grammar: &Grammar, parser: Vec<RItem>) -> RItem
  {
    let grammar_name = grammar.name;
    let grammar_module = quote_item!(self.cx,
      pub mod $grammar_name
      {
        #![allow(dead_code)]
        #![allow(unused_parens, unused_variables)]

        $parser
      }
    ).expect("Quote the grammar module.");
    self.insert_runtime_crate(grammar_module)
  }

  // RUST BUG: We cannot quote `extern crate oak_runtime;` before the grammar module, so we use this workaround
  // for adding the external crate after the creation of the module.
  fn insert_runtime_crate(&self, grammar_module: RItem)
    -> rust::P<rust::Item>
  {
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

  fn compile_parser(&mut self, grammar: &Grammar) -> Vec<RItem>
  {
    self.compile_rules(grammar);
    let mut rust_code: Vec<RItem> = grammar.rust_items.values().cloned().collect();
    rust_code.extend(self.function_gen.code().into_iter());
    rust_code
  }

  fn compile_rules(&mut self, grammar: &Grammar) {
    for rule in grammar.rules.values() {
      self.current_rule_name = rule.name.node;
      let expr_fn = self.compile_expression(&rule.def);
      self.function_gen.generate_rule(rule.def.kind(), self.current_rule_name, expr_fn);
    }
  }

  fn compile_expression(&mut self, expr: &Box<Expression>) -> GenFunNames
  {
    match &expr.node {
      &StrLiteral(ref lit_str) => {
        self.compile_str_literal(expr, lit_str)
      },
      &AnySingleChar => {
        self.compile_any_single_char(expr)
      },
      &CharacterClass(ref e) => {
        self.compile_character_class(expr, e)
      },
      &NonTerminalSymbol(id) => {
        self.compile_non_terminal_symbol(id)
      },
      &NotPredicate(ref e) => {
        self.compile_not_predicate(expr, e)
      },
      &AndPredicate(ref e) => {
        self.compile_and_predicate(expr, e)
      },
      &Optional(ref e) => {
        self.compile_optional(expr, e)
      },
      &Sequence(ref seq) => {
        self.compile_sequence(expr, seq.as_slice())
      },
      &Choice(ref choices) => {
        self.compile_choice(expr, choices.as_slice())
      },
      &ZeroOrMore(ref e) => {
        self.compile_zero_or_more(expr, e)
      },
      &OneOrMore(ref e) => {
        self.compile_one_or_more(expr, e)
      },
      &SemanticAction(ref e, id) => {
        self.compile_semantic_action(expr, e, id)
      }
    }
  }

  fn compile_exprs(&mut self, exprs: &[Box<Expression>]) -> Vec<GenFunNames>
  {
    let res: Vec<_> = exprs.iter().map(|expr| self.compile_expression(expr)).collect();
    res
  }

  fn compile_non_terminal_symbol(&mut self, rule_id: Ident) -> GenFunNames
  {
    self.function_gen.names_of_rule(rule_id)
  }

  fn compile_any_single_char(&mut self, parent: &Box<Expression>) -> GenFunNames
  {
    self.function_gen.generate_expr("any_single_char", self.current_rule_name, parent.kind(),
      quote_expr!(self.cx, oak_runtime::recognize_any_single_char(input, pos)),
      quote_expr!(self.cx, oak_runtime::parse_any_single_char(input, pos))
    )
  }

  fn compile_str_literal(&mut self, parent: &Box<Expression>, lit_str: &String) -> GenFunNames
  {
    let lit_str = lit_str.as_str();
    let lit_len = lit_str.len();
    self.function_gen.generate_expr("str_literal", self.current_rule_name, parent.kind(),
      quote_expr!(self.cx, oak_runtime::recognize_match_literal(input, pos, $lit_str, $lit_len)),
      quote_expr!(self.cx, oak_runtime::parse_match_literal(input, pos, $lit_str, $lit_len))
    )
  }

  fn compile_character_class(&mut self, parent: &Box<Expression>, classes: &CharacterClassExpr) -> GenFunNames
  {
    let cx = self.cx;
    let mut seq_it = classes.intervals.iter();

    let CharacterInterval{lo, hi} = *seq_it.next()
      .expect("Empty character intervals should be forbidden at the parsing stage.");
    let cond = seq_it.fold(quote_expr!(cx, (current >= $lo && current <= $hi)),
      |accu, &CharacterInterval{lo, hi}| {
        quote_expr!(cx, $accu || (current >= $lo && current <= $hi))
      }
    );

    let make_char_class_body = |result: RExpr| quote_expr!(cx, {
      let char_range = input.char_range_at(pos);
      let current = char_range.ch;
      if $cond {
        Ok($result)
      } else {
        Err(format!("It doesn't match the character class."))
      }}
    );

    self.function_gen.generate_expr("class_char", self.current_rule_name, parent.kind(),
      make_char_class_body(quote_expr!(cx, oak_runtime::ParseState::stateless(char_range.next))),
      make_char_class_body(quote_expr!(cx, oak_runtime::ParseState::new(current, char_range.next)))
    )
  }

  fn compile_not_predicate(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> GenFunNames
  {
    let recognizer_name = self.compile_expression(expr).recognizer;
    let body = quote_expr!(self.cx,
      oak_runtime::not_predicate($recognizer_name(input, pos), pos)
    );
    self.function_gen.generate_unit_expr(
      "not_predicate", self.current_rule_name, parent.kind(), body)
  }

  fn compile_and_predicate(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> GenFunNames
  {
    let recognizer_name = self.compile_expression(expr).recognizer;
    let body = quote_expr!(self.cx,
      oak_runtime::and_predicate($recognizer_name(input, pos), pos)
    );
    self.function_gen.generate_unit_expr(
      "and_predicate", self.current_rule_name, parent.kind(), body)
  }

  fn compile_optional(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> GenFunNames
  {
    let GenFunNames{recognizer, parser} = self.compile_expression(expr);
    let recognizer_body = quote_expr!(self.cx,
      oak_runtime::optional_recognizer($recognizer(input, pos), pos)
    );
    let parser_body = quote_expr!(self.cx,
      oak_runtime::optional_parser($parser(input, pos), pos)
    );
    self.function_gen.generate_expr("optional", self.current_rule_name, parent.kind(),
      recognizer_body,
      parser_body)
  }

  fn compile_star(&mut self, parent: &Box<Expression>, expr: &Box<Expression>,
    recognizer_res: RExpr, parser_res: RExpr) -> GenFunNames
  {
    let GenFunNames{recognizer, parser} = self.compile_expression(expr);

    let recognizer_body = quote_expr!(self.cx, {
      let mut current = pos;
      while current < input.len() {
        match $recognizer(input, current) {
          Ok(state) => { current = state.offset; }
          _ => break
        }
      }
      $recognizer_res
    });

    let parser_body = quote_expr!(self.cx, {
      let mut data = vec![];
      let mut current = pos;
      while current < input.len() {
        match $parser(input, current) {
          Ok(state) => {
            data.push(state.data);
            current = state.offset;
          }
          _ => break
        }
      }
      $parser_res
    });
    self.function_gen.generate_expr("star", self.current_rule_name, parent.kind(),
      recognizer_body,
      parser_body)
  }

  fn compile_zero_or_more(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> GenFunNames
  {
    let cx = self.cx;
    self.compile_star(parent, expr,
      quote_expr!(cx, Ok(oak_runtime::ParseState::stateless(current))),
      quote_expr!(cx, Ok(oak_runtime::ParseState::new(data, current))))
  }

  fn compile_one_or_more(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> GenFunNames
  {
    let cx = self.cx;
    let make_result = |res:RExpr| {
      quote_expr!(cx, {
        if current == pos {
          Err(format!("Expected at least one occurrence of an expression in `e+`."))
        } else {
          $res
        }
      })
    };
    self.compile_star(parent, expr,
      make_result(quote_expr!(cx, Ok(oak_runtime::ParseState::stateless(current)))),
      make_result(quote_expr!(cx, Ok(oak_runtime::ParseState::new(data, current)))))
  }

  fn compile_sequence(&mut self, parent: &Box<Expression>, seq: &[Box<Expression>]) -> GenFunNames
  {
    let seq = self.compile_exprs(seq);

    let cx = self.cx;
    let recognizer_body = map_foldr(seq.clone(),
      quote_expr!(cx, Ok(state)),
      |name| name.recognizer,
      |accu: RExpr, name: Ident| {
        quote_expr!(cx, $name(input, pos).and_then(|state| {
          let pos = state.offset;
          $accu
        }))
      }
    );

    let state_names: Vec<Ident> = seq.iter().enumerate()
      .map(|(idx, _)| {
        rust::gensym_ident(format!("state{}", idx).as_str())
      })
      .rev()
      .collect();

    let tuple_indexes = parent.tuple_indexes();

    let mut tuple_result: Vec<RExpr> = tuple_indexes.into_iter()
      .map(|idx| state_names[idx])
      .map(|name| quote_expr!(cx, $name.data))
      .collect();

    let value =
      if tuple_result.len() == 1 {
        tuple_result.pop().unwrap()
      } else {
        cx.expr_tuple(parent.span, tuple_result)
      };
    let value = quote_expr!(cx, Ok(oak_runtime::ParseState::new($value, pos)));

    let parser_body = map_foldr(seq,
      (value, state_names.len()),
      |name| name.parser,
      |(accu, state_idx): (RExpr, usize), name: Ident| {
        let state_idx = state_idx - 1;
        let state_name = state_names[state_idx];
        (
          quote_expr!(cx, $name(input, pos).and_then(|$state_name| {
            let pos = $state_name.offset;
            $accu
          })),
          state_idx
        )
      }
    ).0;
    self.function_gen.generate_expr("sequence", self.current_rule_name, parent.kind(),
      recognizer_body,
      parser_body)
  }

  fn compile_choice(&mut self, parent: &Box<Expression>, choices: &[Box<Expression>]) -> GenFunNames
  {
    let choices = self.compile_exprs(choices);

    let cx = self.cx;
    let error = quote_expr!(cx, Err(err));
    let make_body = |accu:RExpr, name:Ident| {
      quote_expr!(cx, $name(input, pos).or_else(|err| $accu))
    };
    let recognizer_body = map_foldr(choices.clone(),
      error.clone(),
      |name| name.recognizer,
      &make_body
    );
    let parser_body = map_foldr(choices,
      error,
      |name| name.parser,
      make_body
    );
    self.function_gen.generate_expr("choice", self.current_rule_name, parent.kind(),
      recognizer_body,
      parser_body)
  }

  fn compile_semantic_action(&mut self, parent: &Box<Expression>, expr: &Box<Expression>, action_name: Ident) -> GenFunNames
  {
    let GenFunNames{recognizer, parser} = self.compile_expression(expr);

    let recognizer_body = quote_expr!(self.cx,
      $recognizer(input, pos)
    );

    let ty = expr.ty.clone();
    let state_data = quote_expr!(self.cx, state.data);
    let action_params = match ty {
      ExprTy::Tuple(ref indexes) if indexes.len() > 1 => {
        indexes.iter()
          .map(|&idx| self.cx.expr_tup_field_access(parent.span, state_data.clone(), idx))
          .collect()
      },
      ExprTy::Tuple(ref indexes) if indexes.len() == 0 => {
        vec![]
      }
      _ => {
        vec![state_data]
      }
    };

    let action_call = self.cx.expr_call_ident(parent.span, action_name, action_params);

    let parser_body = quote_expr!(self.cx,
      $parser(input, pos).map(|state| {
        let data = $action_call;
        oak_runtime::ParseState::new(data, state.offset)
      })
    );
    self.function_gen.generate_expr("semantic_action", self.current_rule_name, parent.kind(),
      recognizer_body,
      parser_body)
  }
}
