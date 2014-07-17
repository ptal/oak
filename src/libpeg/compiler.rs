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

use syntax::ext::quote::rt::ToTokens;
use syntax::print::pprust;
use syntax::ast;
use syntax::parse::token;
use syntax::ext::base::{ExtCtxt, MacResult, MacItem};
use syntax::codemap::DUMMY_SP;
use ast::*;
use utility::*;
use std::gc::GC;

struct ToTokensVec<'a, T>
{
  v: &'a Vec<T>
}

impl<'a, T: ToTokens> ToTokens for ToTokensVec<'a, T>
{
  fn to_tokens(&self, cx: &ExtCtxt) -> Vec<ast::TokenTree> {
    let mut tts = Vec::new();
    for e in self.v.iter() {
      tts = tts.append(e.to_tokens(cx).as_slice());
    }
    tts
  }
}

pub struct PegCompiler<'a>
{
  top_level_items: Vec<ast::P<ast::Item>>,
  cx: &'a ExtCtxt<'a>,
  unique_id: uint,
  grammar: &'a Peg,
  current_rule_idx: uint,
  starting_rule: uint
}

impl<'a> PegCompiler<'a>
{
  pub fn compile(cx: &'a ExtCtxt, grammar: &'a Peg) -> Box<MacResult>
  {
    let mut compiler = PegCompiler{
      top_level_items: Vec::new(),
      cx: cx,
      unique_id: 0,
      grammar: grammar,
      current_rule_idx: 0,
      starting_rule: 0
    };
    compiler.compile_peg()
  }

  fn compile_peg(&mut self) -> Box<MacResult>
  {
    let grammar_name = self.grammar.name;

    for rule in self.grammar.rules.iter() {
      self.compile_rule_attributes(&rule.attributes);
      let rule_name = rule.name;
      let rule_def = self.compile_rule_rhs(&rule.def);
      self.top_level_items.push(quote_item!(self.cx,
        fn $rule_name (input: &str, pos: uint) -> Result<uint, String>
        {
          $rule_def
        }
      ).unwrap());
      self.current_rule_idx += 1;
    }

    let parser_impl = self.compile_entry_point();

    let items = ToTokensVec{v: &self.top_level_items};

    let grammar = quote_item!(self.cx,
      pub mod $grammar_name
      {
        #![allow(dead_code)]
        #![allow(non_snake_case_functions)]
        #![allow(unnecessary_parens)]

        pub struct Parser;

        impl Parser
        {
          pub fn new() -> Parser
          {
            Parser
          }
          $items
        }

        $parser_impl
      }
    ).unwrap();
    
    let peg_crate = ast::ViewItem {
      node: ast::ViewItemExternCrate(token::str_to_ident("peg"), None, ast::DUMMY_NODE_ID),
      attrs: vec![],
      vis: ast::Inherited,
      span: DUMMY_SP
    };

    let grammar = match &grammar.node {
      &ast::ItemMod(ref module) => {
        box(GC) ast::Item {
          ident: grammar.ident,
          attrs: grammar.attrs.clone(),
          id: ast::DUMMY_NODE_ID,
          node: ast::ItemMod(ast::Mod{
            inner: DUMMY_SP,
            view_items: module.view_items.clone().append_one(peg_crate),
            items: module.items.clone()
          }),
          vis: ast::Public,
          span: DUMMY_SP
        }
      },
      _ => fail!("Bug")
    };

    match get_attribute(&self.grammar.attributes, "print_generated") {
      Some(_) => self.cx.parse_sess.span_diagnostic.handler.note(
        pprust::item_to_string(grammar).as_slice()),
      None => ()
    };

    MacItem::new(grammar)
  }

  fn compile_rule_attributes(&mut self, attrs: &Vec<Attribute>)
  {
    match get_attribute(attrs, "start") {
      Some(_) => self.starting_rule = self.current_rule_idx,
      _ => ()
    }
  }

  fn compile_entry_point(&mut self) -> ast::P<ast::Item>
  {
    let start_idx = self.starting_rule;
    let start_rule = self.grammar.rules.as_slice()[start_idx].name;
    (quote_item!(self.cx,
      impl peg::Parser for Parser
      {
        fn parse<'a>(&self, input: &'a str) -> Result<Option<&'a str>, String>
        {
          peg::runtime::make_result(input,
            &Parser::$start_rule(input, 0))
        }
      })).unwrap()
  }

  fn compile_rule_rhs(&mut self, expr: &Box<Expression>) -> ast::P<ast::Expr>
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

  fn compile_non_terminal_symbol(&mut self, id: Ident) -> ast::P<ast::Expr>
  {
    quote_expr!(self.cx,
      Parser::$id(input, pos)
    )
  }

  fn compile_any_single_char(&mut self) -> ast::P<ast::Expr>
  {
    quote_expr!(self.cx, peg::runtime::any_single_char(input, pos))
  }

  fn compile_str_literal(&mut self, lit_str: &String) -> ast::P<ast::Expr>
  {
    let lit_str = lit_str.as_slice();
    let lit_len = lit_str.len();
    quote_expr!(self.cx,
      peg::runtime::match_literal(input, pos, $lit_str, $lit_len)
    )
  }

  fn map_foldr_expr<'a>(&mut self, seq: &'a [Box<Expression>], 
    f: |ast::P<ast::Expr>, ast::P<ast::Expr>| -> ast::P<ast::Expr>) -> ast::P<ast::Expr>
  {
    assert!(seq.len() > 0);
    let mut seq_it = seq
      .iter()
      .map(|e| { self.compile_rule_rhs(e) })
      .rev();

    let head = seq_it.next().unwrap();
    seq_it.fold(head, f)
  }

  fn compile_sequence<'a>(&mut self, seq: &'a [Box<Expression>]) -> ast::P<ast::Expr>
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

  fn compile_choice<'a>(&mut self, choices: &'a [Box<Expression>]) -> ast::P<ast::Expr>
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

  fn current_rule_name(&self) -> String
  {
    id_to_string(self.grammar.rules.as_slice()[self.current_rule_idx].name)
  }

  fn gensym<'a>(&mut self, prefix: &'a str) -> Ident
  {
    token::gensym_ident(format!(
      "{}_{}_{}", prefix, 
        self.current_rule_name(), 
        self.gen_uid()).as_slice())
  }

  fn compile_star(&mut self, expr: &ast::P<ast::Expr>) -> ast::P<ast::Expr>
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

  fn compile_zero_or_more(&mut self, expr: &Box<Expression>) -> ast::P<ast::Expr>
  {
    let expr = self.compile_rule_rhs(expr);
    self.compile_star(&expr)
  }

  fn compile_one_or_more(&mut self, expr: &Box<Expression>) -> ast::P<ast::Expr>
  {
    let expr = self.compile_rule_rhs(expr);
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

  fn compile_optional(&mut self, expr: &Box<Expression>) -> ast::P<ast::Expr>
  {
    let expr = self.compile_rule_rhs(expr);
    quote_expr!(self.cx,
      match $expr {
        Ok(pos) => Ok(pos),
        _ => Ok(pos)
      }
    )
  }

  fn compile_not_predicate(&mut self, expr: &Box<Expression>) -> ast::P<ast::Expr>
  {
    let expr = self.compile_rule_rhs(expr);
    quote_expr!(self.cx,
      match $expr {
        Ok(_) => Err(format!("An `!expr` failed.")),
        _ => Ok(pos)
    })
  }

  fn compile_and_predicate(&mut self, expr: &Box<Expression>) -> ast::P<ast::Expr>
  {
    let expr = self.compile_rule_rhs(expr);
    quote_expr!(self.cx,
      match $expr {
        Ok(_) => Ok(pos),
        x => x
    })
  }

  fn compile_character_class(&mut self, expr: &CharacterClassExpr) -> ast::P<ast::Expr>
  {
    let fun_name = self.gensym("class_char");
    let cx = self.cx;
    assert!(expr.intervals.len() > 0);

    let mut seq_it = expr.intervals.iter();

    let CharacterInterval{lo:lo, hi:hi} = *seq_it.next().unwrap();
    let cond = seq_it.fold(quote_expr!(cx, (current >= $lo && current <= $hi)), |accu, &CharacterInterval{lo:lo, hi:hi}| {
      quote_expr!(cx, $accu || (current >= $lo && current <= $hi))
    });

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
}
