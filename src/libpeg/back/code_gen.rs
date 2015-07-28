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

use rust;
use back::ast::*;
use back::ast::Expression_::*;
use back::naming::*;
use back::function::*;

use std::iter::*;

pub fn generate_rust_code<'cx>(cx: &'cx ExtCtxt, grammar: Grammar) -> Box<rust::MacResult + 'cx>
{
  CodeGenerator::compile(cx, grammar)
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
    let parser =
      if grammar.attributes.code_gen.parser {
        Some(self.compile_parser(grammar))
      } else {
        None
      };

    let grammar_module = self.compile_grammar_module(grammar, parser);

    if grammar.attributes.code_printer.parser {
      self.cx.parse_sess.span_diagnostic.handler.note(
        rust::item_to_string(&*grammar_module).as_str());
    }

    rust::MacEager::items(rust::SmallVector::one(grammar_module))
  }

  fn compile_grammar_module(&self, grammar: &Grammar, parser: Option<Vec<rust::P<rust::Item>>>)
    -> rust::P<rust::Item>
  {
    let grammar_name = grammar.name;
    let grammar_module = quote_item!(self.cx,
      pub mod $grammar_name
      {
        #![allow(dead_code)]
        #![allow(unused_parens)]
        #![allow(plugin_as_library)] // for the runtime.

        $parser
      }
    ).expect("Quote the grammar module.");

    self.insert_peg_crate(grammar_module)
  }

  // RUST BUG: We cannot quote `extern crate peg;` before the grammar module, so we use this workaround
  // for adding the external crate after the creation of the module.
  fn insert_peg_crate(&self, grammar_module: rust::P<rust::Item>)
    -> rust::P<rust::Item>
  {
    let peg_crate = quote_item!(self.cx,
      extern crate peg;
    ).expect("Quote the extern PEG crate.");

    match &grammar_module.node {
      &rust::ItemMod(ref module_code) => {
        let mut items = vec![peg_crate];
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

    let parser_impl = self.compile_entry_point(grammar);

    let mut parser = vec![];
    parser.push(quote_item!(self.cx, pub struct Parser;).
      expect("Quote the `Parser` declaration."));
    parser.push(quote_item!(self.cx,
        impl Parser
        {
          pub fn new() -> Parser
          {
            Parser
          }
        }).expect("Quote the `Parser` implementation."));
    ;
    parser.extend(self.function_gen.code().into_iter());
    parser.push(parser_impl);
    parser
  }

  fn compile_rules(&mut self, grammar: &Grammar) {
    for rule in grammar.rules.values() {
      self.current_rule_name = rule.name.node;
      let expr_fn = self.compile_expression(&rule.def);
      self.function_gen.generate_rule(rule.def.kind(), self.current_rule_name, expr_fn);
    }
  }

  // TODO: decide whether we want to remove start attribute or not.
  fn compile_entry_point(&mut self, grammar: &Grammar) -> RItem
  {
    let start_rule_name = self.function_gen.names_of_rule(grammar.attributes.starting_rule).recognizer;
    (quote_item!(self.cx,
      impl peg::Parser for Parser
      {
        fn parse<'a>(&self, input: &'a str) -> Result<Option<&'a str>, String>
        {
          peg::runtime::make_result(input,
            &$start_rule_name(input, 0))
        }
      })).expect("Quote the implementation of `peg::Parser` for Parser.")
  }

  fn map_foldr_expr<'a, F>(&mut self, seq: &'a [Box<Expression>], f: F) -> RExpr where
    F: FnMut(RExpr, RExpr) -> RExpr
  {
    let cx = self.cx;
    let mut seq_it = seq
      .iter()
      .map(|e| self.compile_expression(e))
      .map(|GenFunNames{recognizer,..}| quote_expr!(cx, $recognizer(input, pos)))
      .rev();

    let block = seq_it.next().expect("Can not right fold an empty sequence.");
    seq_it.fold(quote_expr!(self.cx, $block), f)
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

  fn compile_non_terminal_symbol(&mut self, rule_id: Ident) -> GenFunNames
  {
    self.function_gen.names_of_rule(rule_id)
  }

  fn compile_any_single_char(&mut self, parent: &Box<Expression>) -> GenFunNames
  {
    self.function_gen.generate_expr("any_single_char", self.current_rule_name, parent.kind(),
      quote_expr!(self.cx, peg::runtime::recognize_any_single_char(input, pos)),
      quote_expr!(self.cx, peg::runtime::parse_any_single_char(input, pos))
    )
  }

  fn compile_str_literal(&mut self, parent: &Box<Expression>, lit_str: &String) -> GenFunNames
  {
    let lit_str = lit_str.as_str();
    let lit_len = lit_str.len();
    self.function_gen.generate_expr("str_literal", self.current_rule_name, parent.kind(),
      quote_expr!(self.cx, peg::runtime::recognize_match_literal(input, pos, $lit_str, $lit_len)),
      quote_expr!(self.cx, peg::runtime::parse_match_literal(input, pos, $lit_str, $lit_len))
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
      make_char_class_body(quote_expr!(cx, peg::runtime::ParseState::stateless(char_range.next))),
      make_char_class_body(quote_expr!(cx, peg::runtime::ParseState::new(current, char_range.next)))
    )
  }

  fn compile_not_predicate(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> GenFunNames
  {
    let recognizer_name = self.compile_expression(expr).recognizer;
    let body = quote_expr!(self.cx,
      match $recognizer_name(input, pos) {
        Ok(_) => Err(format!("An `!expr` failed.")),
        _ => Ok(peg::runtime::ParseState::stateless(pos))
      }
    );
    self.function_gen.generate_unit_expr(
      "not_predicate", self.current_rule_name, parent.kind(), body)
  }

  fn compile_and_predicate(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> GenFunNames
  {
    let recognizer_name = self.compile_expression(expr).recognizer;
    let body = quote_expr!(self.cx,
      $recognizer_name(input, pos)
      .map(|_| peg::runtime::ParseState::stateless(pos))
    );
    self.function_gen.generate_unit_expr(
      "and_predicate", self.current_rule_name, parent.kind(), body)
  }

  fn compile_optional(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> GenFunNames
  {
    self.compile_expression(expr)
    // let GenFunNames{recognizer, parser} = self.compile_expression(expr);
    // let recognizer_body = quote_expr!(self.cx,
    //   $recognizer(input, pos).or_else(
    //     |_| Ok(peg::runtime::ParseState::stateless(pos)))
    // );
    // let parser_body = quote_expr!(self.cx,
    //   $parser(input, pos).or_else(
    //     |_| Ok(peg::runtime::ParseState::stateless(pos)))
    // );
    // self.function_gen.generate_expr("optional", self.current_rule_name, parent.kind(),
    //   recognizer_body,
    //   parser_body)
  }

  // fn compile_star(&mut self, expr_fn: GenFunNames) -> GenFunNames
  // {
  //   let fun_names = self.names.expression_name("star", &self.current_rule_name);
  //   let recognizer = fun_names.recognizer;
  //   let expr_recognizer = expr_fn.recognizer;
  //   let cx = self.cx;
  //   self.gen_functions.insert(recognizer.clone(), quote_item!(cx,
  //     fn $recognizer(input: &str, pos: usize) -> Result<peg::runtime::ParseState<()>, String>
  //     {
  //       let mut npos = pos;
  //       while npos < input.len() {
  //         let pos = npos;
  //         match $expr_recognizer(input, pos) {
  //           Ok(state) => { npos = state.offset; }
  //           _ => break
  //         }
  //       }
  //       Ok(peg::runtime::ParseState::stateless(npos))
  //     }
  //   ).expect("Quote the parsing function of `expr*`."));
  //   fun_names
  // }

  fn compile_zero_or_more(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> GenFunNames
  {
    self.compile_expression(expr)
    // let expr_fn = self.compile_expression(expr);
    // self.compile_star(expr_fn)
  }

  fn compile_one_or_more(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> GenFunNames
  {
    self.compile_expression(expr)
    // let expr_fn = self.compile_expression(expr);
    // let expr_recognizer = expr_fn.recognizer;
    // let star_fn = self.compile_star(expr_fn);
    // let star_recognizer = star_fn.recognizer;
    // self.compile_expr_recognizer("plus",
    //   quote_expr!(self.cx, $expr_recognizer(input, pos).and_then(|state| $star_recognizer(input, state.offset))))
  }

  fn compile_sequence<'a>(&mut self, parent: &Box<Expression>, seq: &'a [Box<Expression>]) -> GenFunNames
  {
    self.compile_expression(&seq[0])
    // let cx = self.cx;
    // let expr = self.map_foldr_expr(seq, |block, expr| {
    //   quote_expr!(cx,
    //     $expr.and_then(|peg::runtime::ParseState { offset: pos, ..}| { $block })
    //   )
    // });
    // self.compile_expr_recognizer("sequence", expr)
  }

  fn compile_choice<'a>(&mut self, parent: &Box<Expression>, choices: &'a [Box<Expression>]) -> GenFunNames
  {
    self.compile_expression(&choices[0])
    // let cx = self.cx;
    // let expr = self.map_foldr_expr(choices, |block, expr| {
    //   quote_expr!(cx,
    //     $expr.or_else(|_| $block)
    //   )
    // });
    // self.compile_expr_recognizer("choice", expr)
  }

  fn compile_semantic_action(&mut self, parent: &Box<Expression>, expr: &Box<Expression>, action_name: Ident) -> GenFunNames
  {
    self.compile_expression(expr)
    // let cx = self.cx;
    // let expr = self.map_foldr_expr(choices, |block, expr| {
    //   quote_expr!(cx,
    //     $expr.or_else(|_| $block)
    //   )
    // });
    // self.compile_expr_recognizer("choice", expr)
  }
}
