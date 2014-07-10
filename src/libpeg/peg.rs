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
use syntax::ast::{Ident, Attribute};
use syntax::codemap::{Spanned, Span, mk_sp, spanned, respan};
use syntax::ext::base::{ExtCtxt, MacResult, MacItem};
use syntax::ext::quote::rt::ToTokens;
use syntax::parse;
use syntax::parse::{token, ParseSess};
use syntax::parse::attr::ParserAttr;
use syntax::parse::parser::Parser;
// use syntax::print::pprust;
use rustc::plugin::Registry;

struct Peg{
  name: Ident,
  rules: Vec<Rule>,
  _attributes: Vec<ast::Attribute>
}

struct Rule{
  name: Ident,
  attributes: Vec<ast::Attribute>,
  def: Box<Expression>
}

enum Expression_{
  StrLiteral(String), // "match me"
  AnySingleChar, // .
  NonTerminalSymbol(Ident), // another_rule
  Sequence(Vec<Box<Expression>>), // a_rule next_rule
  Choice(Vec<Box<Expression>>), // try_this / or_try_this_one
  ZeroOrMore(Box<Expression>), // space*
  OneOrMore(Box<Expression>), // space+
  Optional(Box<Expression>), // space? - `?` replaced by `$`
  NotPredicate(Box<Expression>), // !space
  AndPredicate(Box<Expression>) // &space space
}

type Expression = Spanned<Expression_>;

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
    let (rules, attrs) = self.parse_rules(); 
    Peg{name: grammar_name, rules: rules, _attributes: attrs}
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

  fn parse_rules(&mut self) -> (Vec<Rule>, Vec<Attribute>)
  {
    let mut rules = vec![];
    let mut attrs = vec![];
    while self.rp.token != token::EOF
    {
      let (rule, mod_attrs) = self.parse_rule();
      rules.push(rule);
      attrs.push_all(mod_attrs.as_slice());
    }
    (rules, attrs)
  }

  fn parse_rule(&mut self) -> (Rule, Vec<Attribute>)
  {
    let (inner_attrs, outer_attrs) = self.parse_attributes();
    let name = self.parse_rule_decl();
    self.rp.expect(&token::EQ);
    let body = self.parse_rule_rhs(id_to_string(name).as_slice());
    (Rule{name: name, attributes: outer_attrs, def: body},
     inner_attrs)
  }

  // Outer attributes are attached to the next item.
  // Inner attributes are attached to the englobing item.
  fn parse_attributes(&mut self) -> (Vec<ast::Attribute>, Vec<ast::Attribute>)
  {
    let (inners, mut outers) = self.rp.parse_inner_attrs_and_next();
    if !outers.is_empty() {
      outers.push_all(self.rp.parse_outer_attributes().as_slice());
    }
    (inners, outers)
  }

  fn parse_rule_decl(&mut self) -> Ident
  {
    self.rp.parse_ident()
  }

  fn parse_rule_rhs(&mut self, rule_name: &str) -> Box<Expression>
  {
    self.parse_rule_choice(rule_name)
  }

  fn parse_rule_choice(&mut self, rule_name: &str) -> Box<Expression>
  {
    let lo = self.rp.span.lo;
    let mut choices = Vec::new();
    loop{
      choices.push(self.parse_rule_seq(rule_name));
      let token = self.rp.token.clone();
      match token {
        token::BINOP(token::SLASH) => self.rp.bump(),
        _ => break
      }
    }
    let hi = self.rp.last_span.hi;
    box spanned(lo, hi, Choice(choices))
  }

  fn parse_rule_seq(&mut self, rule_name: &str) -> Box<Expression>
  {
    let lo = self.rp.span.lo;
    let mut seq = Vec::new();
    loop{
      match self.parse_rule_prefixed(rule_name){
        Some(expr) => seq.push(expr),
        None => break
      }
    }
    let hi = self.rp.last_span.hi;
    if seq.len() == 0 {
      self.rp.span_err(
        mk_sp(lo, hi),
        format!("In rule {}: must defined at least one parsing expression.", rule_name).as_slice());
    }
    box spanned(lo, hi, Sequence(seq))
  }

  fn parse_rule_prefixed(&mut self, rule_name: &str) -> Option<Box<Expression>>
  {
    let token = self.rp.token.clone();
    match token {
      token::NOT => {
        self.parse_prefix(rule_name, |e| NotPredicate(e))
      }
      token::BINOP(token::AND) => {
        self.parse_prefix(rule_name, |e| AndPredicate(e))
      }
      _ => self.parse_rule_suffixed(rule_name)
    }
  }

  fn parse_prefix(&mut self, rule_name: &str, 
    make_prefix: |Box<Expression>| -> Expression_) -> Option<Box<Expression>>
  {
    let lo = self.rp.span.lo;
    self.rp.bump();
    let expr = match self.parse_rule_suffixed(rule_name) {
      Some(expr) => expr,
      None => {
        let span = self.rp.span;
        self.rp.span_err(
          span,
          format!("In rule {}: A not predicate (`!expr`) is not followed by a valid expression. Do not forget it must be in front of the expression.",
            rule_name).as_slice()
        );
        return None
      }
    };
    let hi = self.rp.span.hi;
    Some(box spanned(lo, hi, make_prefix(expr)))
  }

  fn parse_rule_suffixed(&mut self, rule_name: &str) -> Option<Box<Expression>>
  {
    let lo = self.rp.span.lo;
    let expr = match self.parse_rule_atom(rule_name){
      Some(expr) => expr,
      None => return None
    };
    let hi = self.rp.span.hi;
    let token = self.rp.token.clone();
    match token {
      token::BINOP(token::STAR) => {
        self.rp.bump();
        Some(box spanned(lo, hi, ZeroOrMore(expr)))
      },
      token::BINOP(token::PLUS) => {
        self.rp.bump();
        Some(box spanned(lo, hi, OneOrMore(expr)))
      },
      token::DOLLAR => {
        self.rp.bump();
        Some(box spanned(lo, hi, Optional(expr)))
      }
      _ => Some(expr)
    }
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
        if self.is_rule_lhs() { None }
        else {
          self.rp.bump();
          Some(self.last_respan(NonTerminalSymbol(id)))
        }
      },
      _ => { None }
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

fn expand(cx: &mut ExtCtxt, _sp: Span, tts: &[ast::TokenTree]) -> Box<MacResult> {
  parse(cx, tts)
}

fn parse(cx: &mut ExtCtxt, tts: &[ast::TokenTree]) -> Box<MacResult> {
  let mut parser = PegParser::new(cx.parse_sess(), cx.cfg(), Vec::from_slice(tts));
  let peg = parser.parse_grammar();
  
  check_peg(cx, &peg);
  PegCompiler::compile(cx, &peg)
}

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

fn span_err(cx: &ExtCtxt, sp: Span, m: &str) {
  cx.parse_sess.span_diagnostic.span_err(sp, m);
}

fn start_attribute<'a>(rule_attrs: &'a Vec<Attribute>) -> Option<&'a Attribute>
{
  for attr in rule_attrs.iter() {
    match attr.node.value.node {
      ast::MetaWord(ref w) if w.get() == "start" =>
        return Some(attr),
      _ => ()
    }
  }
  None
}

fn check_start_attribute<'a>(cx: &ExtCtxt, starting_rule: &Option<&'a Rule>, rule: &'a Rule) -> bool
{
  let start_attr = start_attribute(&rule.attributes);
  match start_attr {
    Some(ref attr) => {
      match starting_rule {
        &None => true,
        &Some(starting_rule) => {
          span_err(cx, attr.span, format!(
            "Multiple `start` attributes are forbidden. Rules `{}` and `{}` conflict.",
            id_to_string(starting_rule.name),
            id_to_string(rule.name)).as_slice());
          false
        }
      }
    },
    _ => false
  }
}

fn check_peg(cx: &ExtCtxt, peg: &Peg)
{
  let mut starting_rule = None;
  for rule in peg.rules.iter() {
    check_rule_rhs(cx, peg, &rule.def);
    if check_start_attribute(cx, &starting_rule, rule) {
      starting_rule = Some(rule);
    }
  }
  match starting_rule {
    None =>
      cx.parse_sess.span_diagnostic.handler.warn(
        "No rule has been specified as the starting point (attribute `#[start]`). The first rule will be automatically considered as such."),
    _ => ()
  }
}

fn check_rule_rhs(cx: &ExtCtxt, peg: &Peg, expr: &Box<Expression>)
{
  match &expr.node {
    &NonTerminalSymbol(id) => {
      check_non_terminal_symbol(cx, peg, id, expr.span)
    }
    &Sequence(ref seq) => {
      check_expr_slice(cx, peg, seq.as_slice())
    }
    &Choice(ref choices) => {
      check_expr_slice(cx, peg, choices.as_slice())
    }
    _ => ()
  }
}

fn check_non_terminal_symbol(cx: &ExtCtxt, peg: &Peg, id: Ident, sp: Span)
{
  check_if_rule_is_declared(cx, peg, id, sp)
}

fn check_if_rule_is_declared(cx: &ExtCtxt, peg: &Peg, id: Ident, sp: Span)
{
  for rule in peg.rules.iter() {
    if rule.name == id {
      return;
    }
  }
  span_err(cx, sp, 
    format!("You try to call the rule `{}` which is not declared.", id_to_string(id)).as_slice());
}

fn check_expr_slice<'a>(cx: &ExtCtxt, peg: &Peg, seq: &'a [Box<Expression>])
{
  assert!(seq.len() > 0);
  for expr in seq.iter() {
    check_rule_rhs(cx, peg, expr);
  }
}

struct PegCompiler<'a>
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
  fn compile(cx: &'a ExtCtxt, grammar: &'a Peg) -> Box<MacResult>
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

    let parse_fn = self.compile_entry_point();

    let items = ToTokensVec{v: &self.top_level_items};

    let grammar = quote_item!(self.cx,
      pub mod $grammar_name
      {
        $parse_fn
        $items
      }
    ).unwrap();

    // self.cx.parse_sess.span_diagnostic.handler.note(pprust::item_to_str(grammar).as_slice());

    MacItem::new(grammar)
  }

  fn compile_rule_attributes(&mut self, attrs: &Vec<Attribute>)
  {
    match start_attribute(attrs) {
      Some(_) => self.starting_rule = self.current_rule_idx,
      _ => ()
    }
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
    quote_expr!(self.cx,
      if input.len() - pos > 0 {
        Ok(pos + 1)
      } else {
        Err(format!("End of input when matching `.`"))
      }
    )
  }

  fn compile_str_literal(&mut self, lit_str: &String) -> ast::P<ast::Expr>
  {
    let s_len = lit_str.len();
    let lit_str_slice = lit_str.as_slice();
    quote_expr!(self.cx,
      if input.len() - pos == 0 {
        Err(format!("End of input when matching the literal `{}`", $lit_str_slice))
      } else if input.slice_from(pos).starts_with($lit_str_slice) {
        Ok(pos + $s_len)
      } else {
        Err(format!("Expected `{}` but got `{}`", $lit_str_slice, input.slice_from(pos)))
      }
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
        loop {
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
}
