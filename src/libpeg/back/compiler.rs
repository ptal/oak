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
use middle::ast::*;
use back::naming::*;

use std::iter::*;
use std::ops::RangeFull;
use std::ops::Deref;

type RExpr = rust::P<rust::Expr>;

pub struct PegCompiler<'cx>
{
  names: NameFactory,
  parsing_functions: Vec<rust::P<rust::Item>>,
  cx: &'cx ExtCtxt<'cx>,
  current_rule_name: Ident
}

impl<'cx> PegCompiler<'cx>
{
  pub fn compile(cx: &'cx ExtCtxt, grammar: Grammar) -> Box<rust::MacResult + 'cx>
  {
    let mut compiler = PegCompiler{
      names: NameFactory::new(),
      parsing_functions: Vec::new(),
      cx: cx,
      current_rule_name: grammar.rules.keys().next().unwrap().clone()
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

  fn compile_parser(&mut self, grammar: &Grammar) -> Vec<rust::P<rust::Item>>
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
    parser.extend(self.parsing_functions.drain(RangeFull));
    parser.push(parser_impl);
    parser
  }

  fn compile_rules(&mut self, grammar: &Grammar) {
    for rule in grammar.rules.values() {
      let rule_def_fn = self.compile_expression(&rule.def);
      self.current_rule_name = rule.name.node.clone();
      let rule_fn_name = self.names.rule_recognizer_name(&self.current_rule_name);
      self.parsing_functions.push(quote_item!(self.cx,
        pub fn $rule_fn_name (input: &str, pos: usize) -> Result<peg::runtime::ParseState<()>, String>
        {
          $rule_def_fn(input, pos)
        }
      ).expect(format!("Quote the rule `{}`.", rule.name.node.clone()).as_str()));
    }
  }

  fn compile_entry_point(&mut self, grammar: &Grammar) -> rust::P<rust::Item>
  {
    let start_rule_name = self.names.rule_recognizer_name(&grammar.attributes.starting_rule);
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

  fn compile_expr_recognizer(&mut self, prefix: &str, body: RExpr) -> Ident {
    let fun_name = self.names.expression_recognizer_name(prefix, &self.current_rule_name);
    self.parsing_functions.push(quote_item!(self.cx,
      fn $fun_name(input: &str, pos: usize) -> Result<peg::runtime::ParseState<()>, String>
      {
        $body
      }
    ).expect(format!("Quotation of a parsing function ({}).", prefix).as_str()));
    fun_name
  }

  fn compile_parse_expr_fn<F>(&mut self, expr: &Box<Expression>, prefix: &str, make_body: F) -> Ident where
    F: Fn(&ExtCtxt<'cx>, RExpr) -> RExpr
  {
    let expr_fn_ident = self.compile_expression(expr);
    let body = make_body(self.cx, quote_expr!(self.cx, $expr_fn_ident(input, pos)));
    self.compile_expr_recognizer(prefix, body)
  }

  fn map_foldr_expr<'a, F>(&mut self, seq: &'a [Box<Expression>], f: F) -> RExpr where
    F: FnMut(RExpr, RExpr) -> RExpr
  {
    let cx = self.cx;
    let mut seq_it = seq
      .iter()
      .map(|e| self.compile_expression(e))
      .map(|fn_ident| quote_expr!(cx, $fn_ident(input, pos)))
      .rev();

    let block = seq_it.next().expect("Can not right fold an empty sequence.");
    seq_it.fold(quote_expr!(self.cx, $block), f)
  }

  fn compile_expression(&mut self, expr: &Box<Expression>) -> Ident
  {
    match &expr.node {
      &StrLiteral(ref lit_str) => {
        self.compile_str_literal(lit_str)
      },
      &AnySingleChar => {
        self.compile_any_single_char(expr.ty.borrow().deref(), expr.context)
      },
      &NonTerminalSymbol(ref id) => {
        self.compile_non_terminal_symbol(id)
      },
      &Sequence(ref seq) => {
        self.compile_sequence(seq.as_slice())
      },
      &Choice(ref choices) => {
        self.compile_choice(choices.as_slice())
      },
      &ZeroOrMore(ref e) => {
        self.compile_zero_or_more(e)
      },
      &OneOrMore(ref e) => {
        self.compile_one_or_more(e)
      },
      &Optional(ref e) => {
        self.compile_optional(e)
      },
      &NotPredicate(ref e) => {
        self.compile_not_predicate(e)
      },
      &AndPredicate(ref e) => {
        self.compile_and_predicate(e)
      },
      &CharacterClass(ref e) => {
        self.compile_character_class(e)
      },
      &SemanticAction(ref e, _) => {
        self.compile_expression(e)
      }
    }
  }

  fn compile_non_terminal_symbol(&mut self, rule_id: &Ident) -> Ident
  {
    self.names.rule_recognizer_name(rule_id)
  }

  fn compile_any_single_char(&mut self, _ty: &ExprTy, _context: EvaluationContext) -> Ident
  {
    self.compile_expr_recognizer("any_single_char", quote_expr!(self.cx,
      peg::runtime::recognize_any_single_char(input, pos)
    ))
  }

  fn compile_str_literal(&mut self, lit_str: &String) -> Ident
  {
    let lit_str = lit_str.as_str();
    let lit_len = lit_str.len();

    self.compile_expr_recognizer("str_literal", quote_expr!(self.cx,
      peg::runtime::recognize_match_literal(input, pos, $lit_str, $lit_len)
    ))
  }

  fn compile_sequence<'a>(&mut self, seq: &'a [Box<Expression>]) -> Ident
  {
    let cx = self.cx;
    let expr = self.map_foldr_expr(seq, |block, expr| {
      quote_expr!(cx,
        $expr.and_then(|peg::runtime::ParseState { offset: pos, ..}| { $block })
      )
    });
    self.compile_expr_recognizer("sequence", expr)
  }

  fn compile_choice<'a>(&mut self, choices: &'a [Box<Expression>]) -> Ident
  {
    let cx = self.cx;
    let expr = self.map_foldr_expr(choices, |block, expr| {
      quote_expr!(cx,
        $expr.or_else(|_| $block)
      )
    });
    self.compile_expr_recognizer("choice", expr)
  }

  fn compile_star(&mut self, expr_fn: Ident) -> Ident
  {
    let fun_name = self.names.expression_recognizer_name("star", &self.current_rule_name);
    let cx = self.cx;
    self.parsing_functions.push(quote_item!(cx,
      fn $fun_name(input: &str, pos: usize) -> Result<peg::runtime::ParseState<()>, String>
      {
        let mut npos = pos;
        while npos < input.len() {
          let pos = npos;
          match $expr_fn(input, pos) {
            Ok(state) => { npos = state.offset; }
            _ => break
          }
        }
        Ok(peg::runtime::ParseState::stateless(npos))
      }
    ).expect("Quote the parsing function of `expr*`."));
    fun_name
  }

  fn compile_zero_or_more(&mut self, expr: &Box<Expression>) -> Ident
  {
    let expr_fn = self.compile_expression(expr);
    self.compile_star(expr_fn)
  }

  fn compile_one_or_more(&mut self, expr: &Box<Expression>) -> Ident
  {
    let expr_fn = self.compile_expression(expr);
    let star_fn = self.compile_star(expr_fn);
    self.compile_expr_recognizer("plus",
      quote_expr!(self.cx, $expr_fn(input, pos).and_then(|state| $star_fn(input, state.offset))))
  }

  fn compile_optional(&mut self, expr: &Box<Expression>) -> Ident
  {
    self.compile_parse_expr_fn(expr, "optional", |cx, inner_call| quote_expr!(cx,
      $inner_call.or_else(|_| Ok(peg::runtime::ParseState::stateless(pos)))
    ))
  }

  fn compile_not_predicate(&mut self, expr: &Box<Expression>) -> Ident
  {
    self.compile_parse_expr_fn(expr, "not_predicate", |cx, inner_call| quote_expr!(cx,
      match $inner_call {
        Ok(_) => Err(format!("An `!expr` failed.")),
        _ => Ok(peg::runtime::ParseState::stateless(pos))
      }
    ))
  }

  fn compile_and_predicate(&mut self, expr: &Box<Expression>) -> Ident
  {
    self.compile_parse_expr_fn(expr, "and_predicate", |cx, inner_call| quote_expr!(cx,
      $inner_call.map(|_| peg::runtime::ParseState::stateless(pos))
    ))
  }

  fn compile_character_class(&mut self, expr: &CharacterClassExpr) -> Ident
  {
    let fun_name = self.names.expression_recognizer_name("class_char", &self.current_rule_name);
    let cx = self.cx;

    let mut seq_it = expr.intervals.iter();

    let CharacterInterval{lo, hi} = *seq_it.next()
      .expect("Empty character intervals should be forbidden at the parsing stage.");
    let cond = seq_it.fold(quote_expr!(cx, (current >= $lo && current <= $hi)),
      |accu, &CharacterInterval{lo, hi}| {
        quote_expr!(cx, $accu || (current >= $lo && current <= $hi))
      }
    );

    self.parsing_functions.push(quote_item!(cx,
      fn $fun_name(input: &str, pos: usize) -> Result<peg::runtime::ParseState<()>, String>
      {
        let char_range = input.char_range_at(pos);
        let current = char_range.ch;
        if $cond {
          Ok(peg::runtime::ParseState::stateless(char_range.next))
        } else {
          Err(format!("It doesn't match the character class."))
        }
      }
    ).expect("Quotation of a character class (e.g. `[0-9]`)."));
    fun_name
  }
}
