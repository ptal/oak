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

use std::iter::*;
use std::ops::RangeFull;

pub struct PegCompiler<'cx>
{
  parsing_functions: Vec<rust::P<rust::Item>>,
  cx: &'cx ExtCtxt<'cx>,
  unique_id: u32,
  current_rule_name: Ident
}

impl<'cx> PegCompiler<'cx>
{
  pub fn compile(cx: &'cx ExtCtxt, grammar: Grammar) -> Box<rust::MacResult + 'cx>
  {
    let mut compiler = PegCompiler{
      parsing_functions: Vec::new(),
      cx: cx,
      unique_id: 0,
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
      let rule_name = rule.name.node.clone();
      let rule_def = self.compile_expression(&rule.def);
      self.parsing_functions.push(quote_item!(self.cx,
        fn $rule_name (input: &str, pos: usize) -> Result<usize, String>
        {
          $rule_def
        }
      ).expect(format!("Quote the rule `{}`.", rule.name.node.clone()).as_str()));
      self.current_rule_name = rule_name;
    }
  }

  fn compile_entry_point(&mut self, grammar: &Grammar) -> rust::P<rust::Item>
  {
    let start_rule_name = grammar.attributes.starting_rule;
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

  fn compile_expression(&mut self, expr: &Box<Expression>) -> rust::P<rust::Expr>
  {
    match &expr.node {
      &StrLiteral(ref lit_str) => {
        self.compile_str_literal(lit_str)
      },
      &AnySingleChar => {
        self.compile_any_single_char()
      },
      &NonTerminalSymbol(id) => {
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

  fn compile_non_terminal_symbol(&mut self, id: Ident) -> rust::P<rust::Expr>
  {
    quote_expr!(self.cx,
      $id(input, pos)
    )
  }

  fn compile_any_single_char(&mut self) -> rust::P<rust::Expr>
  {
    quote_expr!(self.cx, peg::runtime::any_single_char(input, pos))
  }

  fn compile_str_literal(&mut self, lit_str: &String) -> rust::P<rust::Expr>
  {
    let lit_str = lit_str.as_str();
    let lit_len = lit_str.len();
    quote_expr!(self.cx,
      peg::runtime::match_literal(input, pos, $lit_str, $lit_len)
    )
  }

  fn map_foldr_expr<'a, F: FnMut(rust::P<rust::Expr>, rust::P<rust::Expr>) -> rust::P<rust::Expr>>(
    &mut self, seq: &'a [Box<Expression>], f: F) -> rust::P<rust::Expr>
  {
    assert!(seq.len() > 0);
    let mut seq_it = seq
      .iter()
      .map(|e| self.compile_expression(e))
      .rev();

    let head = seq_it.next().unwrap();
    seq_it.fold(head, f)
  }

  fn compile_sequence<'a>(&mut self, seq: &'a [Box<Expression>]) -> rust::P<rust::Expr>
  {
    let cx = self.cx;
    self.map_foldr_expr(seq, |tail, head| {
      quote_expr!(cx,
        match $head {
          Ok(pos) => {
            $tail
          }
          x => x
        }
      )
    })
  }

  fn compile_choice<'a>(&mut self, choices: &'a [Box<Expression>]) -> rust::P<rust::Expr>
  {
    let cx = self.cx;
    self.map_foldr_expr(choices, |tail, head| {
      quote_expr!(cx,
        match $head {
          Err(_) => {
            $tail
          }
          x => x
        }
      )
    })
  }

  fn gen_uid(&mut self) -> u32
  {
    self.unique_id += 1;
    self.unique_id - 1
  }

  fn current_lc_rule_name(&self) -> String
  {
    let rule_name = id_to_string(self.current_rule_name);
    string_to_lowercase(&rule_name)
  }

  fn gensym<'a>(&mut self, prefix: &'a str) -> Ident
  {
    rust::gensym_ident(format!(
      "{}_{}_{}", prefix,
        self.current_lc_rule_name(),
        self.gen_uid()).as_str())
  }

  fn compile_star(&mut self, expr: &rust::P<rust::Expr>) -> rust::P<rust::Expr>
  {
    let fun_name = self.gensym("star");
    let cx = self.cx;
    self.parsing_functions.push(quote_item!(cx,
      fn $fun_name(input: &str, pos: usize) -> Result<usize, String>
      {
        let mut npos = pos;
        while npos < input.len() {
          let pos = npos;
          match $expr {
            Ok(pos) => {
              npos = pos;
            },
            _ => break
          }
        }
        Ok(npos)
      }
    ).expect("Quote the parsing function of `expr*`."));
    quote_expr!(self.cx, $fun_name(input, pos))
  }

  fn compile_zero_or_more(&mut self, expr: &Box<Expression>) -> rust::P<rust::Expr>
  {
    let expr = self.compile_expression(expr);
    self.compile_star(&expr)
  }

  fn compile_one_or_more(&mut self, expr: &Box<Expression>) -> rust::P<rust::Expr>
  {
    let expr = self.compile_expression(expr);
    let star_fn = self.compile_star(&expr);
    let fun_name = self.gensym("plus");
    let cx = self.cx;
    self.parsing_functions.push(quote_item!(cx,
      fn $fun_name(input: &str, pos: usize) -> Result<usize, String>
      {
        match $expr {
          Ok(pos) => $star_fn,
          x => x
        }
      }
    ).expect("Quote the parsing function of `expr+`."));
    quote_expr!(self.cx, $fun_name(input, pos))
  }

  fn compile_optional(&mut self, expr: &Box<Expression>) -> rust::P<rust::Expr>
  {
    let expr = self.compile_expression(expr);
    quote_expr!(self.cx,
      match $expr {
        Ok(pos) => Ok(pos),
        _ => Ok(pos)
      }
    )
  }

  fn compile_not_predicate(&mut self, expr: &Box<Expression>) -> rust::P<rust::Expr>
  {
    let expr = self.compile_expression(expr);
    quote_expr!(self.cx,
      match $expr {
        Ok(_) => Err(format!("An `!expr` failed.")),
        _ => Ok(pos)
    })
  }

  fn compile_and_predicate(&mut self, expr: &Box<Expression>) -> rust::P<rust::Expr>
  {
    let expr = self.compile_expression(expr);
    quote_expr!(self.cx,
      match $expr {
        Ok(_) => Ok(pos),
        x => x
    })
  }

  fn compile_character_class(&mut self, expr: &CharacterClassExpr) -> rust::P<rust::Expr>
  {
    let fun_name = self.gensym("class_char");
    let cx = self.cx;
    assert!(expr.intervals.len() > 0);

    let mut seq_it = expr.intervals.iter();

    let CharacterInterval{lo, hi} = *seq_it.next()
      .expect("Empty character intervals should be forbidden at the parsing stage.");
    let cond = seq_it.fold(quote_expr!(cx, (current >= $lo && current <= $hi)),
      |accu, &CharacterInterval{lo, hi}| {
        quote_expr!(cx, $accu || (current >= $lo && current <= $hi))
      }
    );

    self.parsing_functions.push(quote_item!(cx,
      fn $fun_name(input: &str, pos: usize) -> Result<usize, String>
      {
        let current = input.char_range_at(pos).ch;
        if $cond {
          Ok(input.char_range_at(pos).next)
        } else {
          Err(format!("It doesn't match the character class."))
        }
      }
    ).expect("Quote of the character class (e.g. `[0-9]`)."));
    quote_expr!(self.cx, $fun_name(input, pos))
  }
}
