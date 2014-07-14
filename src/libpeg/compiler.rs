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
use ast::*;
use utility::*;

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

    let peg_lib = self.compile_peg_library();

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

    let parse_fn = self.compile_entry_point();
    self.compile_peg_library();

    let items = ToTokensVec{v: &self.top_level_items};

    let grammar = quote_item!(self.cx,
      pub mod $grammar_name
      {
        $peg_lib
        $parse_fn
        $items
      }
    ).unwrap();

    self.cx.parse_sess.span_diagnostic.handler.note(pprust::item_to_string(grammar).as_slice());

    MacItem::new(grammar)
  }

  fn compile_rule_attributes(&mut self, attrs: &Vec<Attribute>)
  {
    match start_attribute(attrs) {
      Some(_) => self.starting_rule = self.current_rule_idx,
      _ => ()
    }
  }

  fn compile_function(&mut self, fun_name: &Ident, body: &ast::P<ast::Expr>) -> ast::P<ast::Item>
  {
    (quote_item!(self.cx,
      pub fn $fun_name<'a>(input: &'a str, pos: uint) -> Result<uint, String>
      {
        $body
      }
    )).unwrap()
  }

  fn compile_lib_any_single_char(&mut self) -> ast::P<ast::Item>
  {
    let cx = self.cx;
    let fun_name = token::gensym_ident("any_single_char");
    self.compile_function(&fun_name, &quote_expr!(cx,
      if input.len() - pos > 0 {
        Ok(input.char_range_at(pos).next)
      } else {
        Err(format!("End of input when matching `.`"))
      }
    ))
  }

  fn compile_lib_match_literal(&mut self) -> ast::P<ast::Item>
  {
    (quote_item!(self.cx,
      pub fn match_literal<'a, 'b>(input: &'a str, pos: uint, lit: &'a str, lit_len: uint)
        -> Result<uint, String>
      {
        if input.len() - pos == 0 {
          Err(format!("End of input when matching the literal `{}`", lit))
        } else if input.slice_from(pos).starts_with(lit) {
          Ok(pos + lit_len)
        } else {
          Err(format!("Expected `{}` but got `{}`", lit, input.slice_from(pos)))
        }
      })).unwrap()
  }

  fn compile_peg_library(&mut self) -> ast::P<ast::Item>
  {
    let any_single_char = self.compile_lib_any_single_char();
    let match_literal = self.compile_lib_match_literal();
    (quote_item!(self.cx,
      pub mod peg {
        $any_single_char
        $match_literal
      }
    )).unwrap()
  }

  fn compile_entry_point(&mut self) -> ast::P<ast::Item>
  {
    let start_idx = self.starting_rule;
    let start_rule = self.grammar.rules.as_slice()[start_idx].name;
    (quote_item!(self.cx,
      pub fn parse<'a>(input: &'a str) -> Result<Option<&'a str>, String>
      {
        match $start_rule(input, 0) {
          Ok(pos) => {
            assert!(pos <= input.len())
            if pos == input.len() {
              Ok(None) 
            } else {
              Ok(Some(input.slice_from(pos)))
            }
          },
          Err(msg) => Err(msg)
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
      $id(input, pos)
    )
  }

  fn compile_any_single_char(&mut self) -> ast::P<ast::Expr>
  {
    quote_expr!(self.cx, peg::any_single_char(input, pos))
  }

  fn compile_str_literal(&mut self, lit_str: &String) -> ast::P<ast::Expr>
  {
    let lit_str = lit_str.as_slice();
    let lit_len = lit_str.len();
    quote_expr!(self.cx,
      peg::match_literal(input, pos, $lit_str, $lit_len)
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
          Err(msg) => {
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
      fn $fun_name<'a>(input: &'a str, pos: uint) -> Result<uint, String>
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
    quote_expr!(self.cx, $fun_name(input, pos))
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
      fn $fun_name<'a>(input: &'a str, pos: uint) -> Result<uint, String>
      {
        match $expr {
          Ok(pos) => $star_fn,
          x => x
        }
      }
    ).unwrap());
    quote_expr!(self.cx, $fun_name(input, pos))
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
      fn $fun_name<'a>(input: &'a str, pos: uint) -> Result<uint, String>
      {
        let current = input.char_range_at(pos).ch;
        if $cond {
          Ok(input.char_range_at(pos).next)
        } else {
          Err(format!("It doesn't match the character class."))
        }
      }
    ).unwrap());
    quote_expr!(self.cx, $fun_name(input, pos))
  }
}
