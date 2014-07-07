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

use std::string::String;
use syntax::ast;
use syntax::ast::{Ident};
use syntax::codemap;
use syntax::ext::base::{ExtCtxt, MacResult, MacExpr, MacItem, DummyResult};
use syntax::ext::build::AstBuilder;
use syntax::parse;
use syntax::parse::{token, ParseSess};
use syntax::parse::token::Token;
use syntax::parse::parser::Parser;
use rustc::plugin::Registry;

struct Peg{
  name: Ident,
  rules: Vec<Rule>
}

struct Rule{
  name: Ident,
  def: Expression
}

enum Expression{
  LiteralStrExpr(String), // "match me"
  AnySingleCharExpr // .
}

struct ParseError{
  span: codemap::Span,
  msg: String
}

struct PegParser<'a>
{
  rp: Parser<'a> // rust parser
}

impl<'a> PegParser<'a>
{
  fn new(sess: &'a ParseSess,
         cfg: ast::CrateConfig,
         tts: Vec<ast::TokenTree>) -> PegParser<'a> {
    PegParser{rp: parse::new_parser_from_tts(sess, cfg, tts)}
  }

  fn parse_grammar(&mut self) -> Peg
  {
    let grammar_name = self.parse_grammar_decl();
    let rules = self.parse_rules(); 
    Peg{name: grammar_name, rules: rules}
  }

  fn parse_grammar_decl(&mut self) -> Ident
  {
    if !self.eat_grammar_keyword() {
      let token_str = self.rp.this_token_to_str();
      self.rp.fatal(
        format!("expected the grammar declaration (of the form: `grammar <grammar-name>;`), found instead `{}`",
          token_str).as_slice())
    }
    let grammar_name = self.rp.parse_ident();
    self.rp.expect(&token::SEMI);
    grammar_name
  }

  fn eat_grammar_keyword(&mut self) -> bool
  {
    let is_grammar_kw = match self.rp.token {
      token::IDENT(sid, false) => "grammar" == id_to_string(sid).as_slice(),
      _ => false
    };
    if is_grammar_kw { self.rp.bump() }
    is_grammar_kw
  }

  fn parse_rules(&mut self) -> Vec<Rule>
  {
    vec![self.parse_rule()]
  }

  fn parse_rule(&mut self) -> Rule
  {
    let name = self.parse_rule_decl();
    self.rp.expect(&token::EQ);
    let body = self.parse_rule_def();
    Rule{name: name, def: body}
  }

  fn parse_rule_decl(&mut self) -> Ident
  {
    self.rp.parse_ident()
  }

  fn parse_rule_def(&mut self) -> Expression
  {
    let token = self.rp.bump_and_get();
    match token{
      token::LIT_STR(id) => {
        LiteralStrExpr(id_to_string(id))
      },
      token::DOT => {
        AnySingleCharExpr
      },
      _ => { self.rp.unexpected_last(&token); }
    }
  }
}

fn id_to_string(id: Ident) -> String
{
  String::from_str(token::get_ident(id).get())
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
  reg.register_macro("peg", expand)
}

fn expand(cx: &mut ExtCtxt, sp: codemap::Span, tts: &[ast::TokenTree]) -> Box<MacResult> {
  parse(cx, tts)
}

fn parse(cx: &mut ExtCtxt, tts: &[ast::TokenTree]) -> Box<MacResult> {
  use syntax::print::pprust;

  let mut parser = PegParser::new(cx.parse_sess(), cx.cfg(), Vec::from_slice(tts));
  let peg = parser.parse_grammar();
  
  transform(cx, &peg)
}

fn transform(cx: &mut ExtCtxt, peg: &Peg) -> Box<MacResult>
{
  let grammar_name = peg.name;
  let rule = &peg.rules.as_slice()[0];
  let rule_name = rule.name;
  let rule_def = transform_rule_def(cx, &rule.def);
  MacItem::new((quote_item!(cx,
    pub mod $grammar_name
    {
      pub fn parse<'a>(input: &'a str) -> Result<Option<&'a str>, String>
      {
        match $rule_name(input, 0) {
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
      }

      fn $rule_name (input: &str, pos: uint) -> Result<uint, String>
      {
        $rule_def
      }
    }
  )).unwrap())
}

fn transform_rule_def(cx: &mut ExtCtxt, expr: &Expression) -> ast::P<ast::Expr>
{
  match expr {
    &LiteralStrExpr(ref lit_str) => {
      let s_len = lit_str.len();
      let lit_str_slice = lit_str.as_slice();
      quote_expr!(cx,
        if input.slice_from(pos).starts_with($lit_str_slice) {
          Ok(pos + $s_len)
        } else {
          Err(format!("Expected {} but got `{}`", $lit_str_slice, input.slice_from(pos)))
        }
      )
    },
    &AnySingleCharExpr => {
      quote_expr!(cx,
        Ok(pos + 1)
      )
    }
  }
}
