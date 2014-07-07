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
use syntax::codemap::{Spanned, Span, mk_sp, spanned, respan};
use syntax::ext::base::{ExtCtxt, MacResult, MacExpr, MacItem, DummyResult};
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ToTokens;
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
  def: Box<Expression>,
}

enum Expression_{
  StrLiteral(String), // "match me"
  AnySingleChar, // .
  NonTerminalSymbol(Ident), // another_rule
  Sequence(Vec<Box<Expression>>)
}

type Expression = Spanned<Expression_>;

struct ParseError{
  span: Span,
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
         tts: Vec<ast::TokenTree>) -> PegParser<'a> 
  {
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
    let mut rules = vec![];
    while(self.rp.token != token::EOF)
    {
      rules.push(self.parse_rule());
    }
    rules
  }

  fn parse_rule(&mut self) -> Rule
  {
    let name = self.parse_rule_decl();
    self.rp.expect(&token::EQ);
    let body = self.parse_rule_rhs(id_to_string(name).as_slice());
    Rule{name: name, def: body}
  }

  fn parse_rule_decl(&mut self) -> Ident
  {
    self.rp.parse_ident()
  }

  fn parse_rule_rhs(&mut self, rule_name: &str) -> Box<Expression>
  {
    self.parse_rule_seq(rule_name)
  }

  fn parse_rule_seq(&mut self, rule_name: &str) -> Box<Expression>
  {
    let lo = self.rp.span.lo;
    let mut seq = Vec::new();
    loop{
      match self.parse_rule_atom(rule_name){
        Some(expr) => seq.push(expr),
        None => break
      }
    }
    let hi = self.rp.last_span.hi;
    if seq.len() == 0 {
      self.rp.span_fatal(
        mk_sp(lo, hi),
        format!("In rule {}: must defined at least one parsing expression.", rule_name).as_slice());
    }
    box spanned(lo, hi, Sequence(seq))
  }

  fn last_respan<T>(&self, t: T) -> Box<Spanned<T>>
  {
    box respan(self.rp.last_span, t)
  }

  fn parse_rule_atom(&mut self, rule_name: &str) -> Option<Box<Expression>>
  {
    let token = self.rp.token.clone();
    match token {
      token::LIT_STR(id) => {
        self.rp.bump();
        Some(self.last_respan(StrLiteral(id_to_string(id))))
      },
      token::DOT => {
        self.rp.bump();
        Some(self.last_respan(AnySingleChar))
      },
      token::LPAREN => {
        self.rp.bump();
        let res = self.parse_rule_rhs(rule_name);
        self.rp.expect(&token::RPAREN);
        Some(res)
      },
      token::IDENT(id, _) => {
        if(self.is_rule_lhs()) { None }
        else {
          self.rp.bump();
          Some(self.last_respan(NonTerminalSymbol(id)))
        }
      },
      token::EOF => { None },
      _ => { self.rp.unexpected_last(&token); }
    }
  }

  fn is_rule_lhs(&mut self) -> bool
  {
    self.rp.look_ahead(1, |t| match t { &token::EQ => true, _ => false})
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

fn expand(cx: &mut ExtCtxt, sp: Span, tts: &[ast::TokenTree]) -> Box<MacResult> {
  parse(cx, tts)
}

fn parse(cx: &mut ExtCtxt, tts: &[ast::TokenTree]) -> Box<MacResult> {
  use syntax::print::pprust;

  let mut parser = PegParser::new(cx.parse_sess(), cx.cfg(), Vec::from_slice(tts));
  let peg = parser.parse_grammar();
  
  compile_peg(cx, &peg)
}

struct ToTokensVec<T>
{
  v: Vec<T>
}

impl<T: ToTokens> ToTokens for ToTokensVec<T>
{
  fn to_tokens(&self, cx: &ExtCtxt) -> Vec<ast::TokenTree> {
    let mut tts = Vec::new();
    for e in self.v.iter() {
      tts = tts.append(e.to_tokens(cx).as_slice());
    }
    tts
  }
}

fn compile_peg(cx: &ExtCtxt, peg: &Peg) -> Box<MacResult>
{
  let grammar_name = peg.name;

  let parse_fn = compile_entry_point(cx, &peg.rules.as_slice()[0].name);

  let mut rule_items = Vec::new();

  for rule in peg.rules.iter()
  {
    let rule_name = rule.name;
    let rule_def = compile_rule_rhs(cx, &rule.def);
    rule_items.push(quote_item!(cx,
      fn $rule_name (input: &str, pos: uint) -> Result<uint, String>
      {
        $rule_def
      }
    ).unwrap())
  }

  let items = ToTokensVec{v: rule_items};

  MacItem::new(quote_item!(cx,
    pub mod $grammar_name
    {
      $parse_fn
      $items
    }
  ).unwrap())
}

fn compile_entry_point(cx: &ExtCtxt, start_rule: &Ident) -> ast::P<ast::Item>
{
  (quote_item!(cx,
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

fn compile_rule_rhs(cx: &ExtCtxt, expr: &Box<Expression>) -> ast::P<ast::Expr>
{
  match &expr.node {
    &StrLiteral(ref lit_str) => {
      compile_str_literal(cx, lit_str)
    },
    &AnySingleChar => {
      compile_any_single_char(cx)
    },
    &NonTerminalSymbol(ref id) => {
      compile_non_terminal_symbol(cx, id)
    },
    &Sequence(ref seq) => {
      compile_sequence(cx, seq.as_slice())
    }
  }
}

fn compile_non_terminal_symbol(cx: &ExtCtxt, id: &Ident) -> ast::P<ast::Expr>
{
  quote_expr!(cx,
    $id(input, pos)
  )
}

fn compile_any_single_char(cx: &ExtCtxt) -> ast::P<ast::Expr>
{
  quote_expr!(cx,
    if input.len() - pos > 0 {
      Ok(pos + 1)
    } else {
      Err(format!("End of input when matching `.`"))
    }
  )
}

fn compile_str_literal(cx: &ExtCtxt, lit_str: &String) -> ast::P<ast::Expr>
{
  let s_len = lit_str.len();
  let lit_str_slice = lit_str.as_slice();
  quote_expr!(cx,
    if input.len() - pos == 0 {
      Err(format!("End of input when matching the literal `{}`", $lit_str_slice))
    } else if input.slice_from(pos).starts_with($lit_str_slice) {
      Ok(pos + $s_len)
    } else {
      Err(format!("Expected {} but got `{}`", $lit_str_slice, input.slice_from(pos)))
    }
  )
}

fn compile_sequence<'a>(cx: &ExtCtxt, seq: &'a [Box<Expression>]) -> ast::P<ast::Expr>
{
  assert!(seq.len() > 0);
  let head = compile_rule_rhs(cx, seq.head().unwrap());
  if seq.len() == 1 {
    head
  } else {
    let tail = compile_sequence(cx, seq.tail());
    quote_expr!(cx,
      match $head {
        Ok(pos) => {
          $tail
        }
        x => x
      }
    )
  }
}

