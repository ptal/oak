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
use rust::ExtCtxt;
use rust::P;
use std::iter::*;
use identifier::*;
use middle::ast::*;

struct ToTokensVec<'a, T: 'a>
{
  v: &'a Vec<T>
}

impl<'a, T: 'a + rust::ToTokens> rust::ToTokens for ToTokensVec<'a, T>
{
  fn to_tokens(&self, cx: &ExtCtxt) -> Vec<rust::TokenTree> {
    let mut tts = Vec::new();
    for e in self.v.iter() {
      tts.extend(e.to_tokens(cx).into_iter());
    }
    tts
  }
}

pub struct PegCompiler<'cx>
{
  top_level_items: Vec<rust::P<rust::Item>>,
  cx: &'cx ExtCtxt<'cx>,
  unique_id: uint,
  current_rule_name: Ident
}

impl<'cx> PegCompiler<'cx>
{
  pub fn compile<'cx>(cx: &'cx ExtCtxt, grammar: Grammar) -> Box<rust::MacResult + 'cx>
  {
    let mut compiler = PegCompiler{
      top_level_items: Vec::new(),
      cx: cx,
      unique_id: 0,
      current_rule_name: grammar.rules.keys().next().unwrap().clone()
    };
    compiler.compile_peg(&grammar)
  }

  fn compile_peg(&mut self, grammar: &Grammar) -> Box<rust::MacResult + 'cx>
  {
    let ast =
      if grammar.attributes.code_gen.ast {
        Some(self.compile_ast(grammar))
      } else {
        None
      };

    let parser =
      if grammar.attributes.code_gen.parser {
        Some(self.compile_parser(grammar))
      } else {
        None
      };

    let grammar_name = grammar.name;
    let code = quote_item!(self.cx,
      pub mod $grammar_name
      {
        #![allow(dead_code)]
        #![allow(unnecessary_parens)]

        $ast
        $parser
      }
    ).unwrap();

    let peg_crate = rust::ViewItem {
      node: rust::ViewItemExternCrate(rust::str_to_ident("peg"), None, rust::DUMMY_NODE_ID),
      attrs: vec![],
      vis: rust::Inherited,
      span: rust::DUMMY_SP
    };

    let code = match &code.node {
      &rust::ItemMod(ref module) => {
        let mut view_items = module.view_items.clone();
        view_items.push(peg_crate);
        P(rust::Item {
          ident: code.ident,
          attrs: code.attrs.clone(),
          id: rust::DUMMY_NODE_ID,
          node: rust::ItemMod(rust::Mod{
            inner: rust::DUMMY_SP,
            view_items: view_items,
            items: module.items.clone()
          }),
          vis: rust::Public,
          span: rust::DUMMY_SP
        })
      },
      _ => panic!("Bug")
    };

    if grammar.attributes.code_printer.parser {
      self.cx.parse_sess.span_diagnostic.handler.note(
        rust::item_to_string(&*code).as_slice());
    } else {
    }

    rust::MacItems::new(Some(code).into_iter())
  }

  fn compile_parser(&mut self, grammar: &Grammar) -> Vec<rust::P<rust::Item>>
  {
    for rule in grammar.rules.values() {
      let rule_name = rule.name.node.clone();
      let rule_def = self.compile_expression(&rule.def);
      self.top_level_items.push(quote_item!(self.cx,
        fn $rule_name (input: &str, pos: uint) -> Result<uint, String>
        {
          $rule_def
        }
      ).unwrap());
      self.current_rule_name = rule_name;
    }

    let parser_impl = self.compile_entry_point(grammar);

    let items = ToTokensVec{v: &self.top_level_items};

    let mut parser = vec![];
    parser.push(quote_item!(self.cx, pub struct Parser;).unwrap());
    parser.push(quote_item!(self.cx,
        impl Parser
        {
          pub fn new() -> Parser
          {
            Parser
          }
          $items
        }).unwrap());
    parser.push(parser_impl);
    parser
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
            &Parser::$start_rule_name(input, 0))
        }
      })).unwrap()
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
      }
    }
  }

  fn compile_non_terminal_symbol(&mut self, id: Ident) -> rust::P<rust::Expr>
  {
    quote_expr!(self.cx,
      Parser::$id(input, pos)
    )
  }

  fn compile_any_single_char(&mut self) -> rust::P<rust::Expr>
  {
    quote_expr!(self.cx, peg::runtime::any_single_char(input, pos))
  }

  fn compile_str_literal(&mut self, lit_str: &String) -> rust::P<rust::Expr>
  {
    let lit_str = lit_str.as_slice();
    let lit_len = lit_str.len();
    quote_expr!(self.cx,
      peg::runtime::match_literal(input, pos, $lit_str, $lit_len)
    )
  }

  fn map_foldr_expr<'a>(&mut self, seq: &'a [Box<Expression>],
    f: |rust::P<rust::Expr>, rust::P<rust::Expr>| -> rust::P<rust::Expr>) -> rust::P<rust::Expr>
  {
    assert!(seq.len() > 0);
    let mut seq_it = seq
      .iter()
      .map(|e| { self.compile_expression(e) })
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

  fn gen_uid(&mut self) -> uint
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
        self.gen_uid()).as_slice())
  }

  fn compile_star(&mut self, expr: &rust::P<rust::Expr>) -> rust::P<rust::Expr>
  {
    let fun_name = self.gensym("star");
    let cx = self.cx;
    self.top_level_items.push(quote_item!(cx,
      fn $fun_name(input: &str, pos: uint) -> Result<uint, String>
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
    ).unwrap());
    quote_expr!(self.cx, Parser::$fun_name(input, pos))
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
    self.top_level_items.push(quote_item!(cx,
      fn $fun_name(input: &str, pos: uint) -> Result<uint, String>
      {
        match $expr {
          Ok(pos) => $star_fn,
          x => x
        }
      }
    ).unwrap());
    quote_expr!(self.cx, Parser::$fun_name(input, pos))
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

    let CharacterInterval{lo, hi} = *seq_it.next().unwrap();
    let cond = seq_it.fold(quote_expr!(cx, (current >= $lo && current <= $hi)),
      |accu, &CharacterInterval{lo, hi}| {
        quote_expr!(cx, $accu || (current >= $lo && current <= $hi))
      }
    );

    self.top_level_items.push(quote_item!(cx,
      fn $fun_name(input: &str, pos: uint) -> Result<uint, String>
      {
        let current = input.char_range_at(pos).ch;
        if $cond {
          Ok(input.char_range_at(pos).next)
        } else {
          Err(format!("It doesn't match the character class."))
        }
      }
    ).unwrap());
    quote_expr!(self.cx, Parser::$fun_name(input, pos))
  }

  fn compile_ast(&mut self, _grammar: &Grammar) -> rust::P<rust::Item>
  {
    let ast = quote_item!(self.cx,
      pub mod ast
      {
      }
    ).unwrap();
    ast
  }
}
