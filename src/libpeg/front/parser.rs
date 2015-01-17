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

use rust::{ParserAttr, respan};
use rust::Token as rtok;
use rust::BinOpToken as rbtok;
use std::str::Chars;

use front::ast::*;
use front::ast::Expression_::*;

pub struct Parser<'a>
{
  rp: rust::Parser<'a>,
  inner_attrs: Vec<rust::Attribute>
}

impl<'a> Parser<'a>
{
  pub fn new(sess: &'a rust::ParseSess,
         cfg: rust::CrateConfig,
         tts: Vec<rust::TokenTree>) -> Parser<'a>
  {
    Parser{
      rp: rust::new_parser_from_tts(sess, cfg, tts),
      inner_attrs: Vec::new()}
  }

  pub fn parse_grammar(&mut self) -> Grammar
  {
    let grammar_name = self.parse_grammar_decl();
    let (rules, rust_items) = self.parse_blocks();
    Grammar{name: grammar_name, rules: rules, rust_items: rust_items, attributes: self.inner_attrs.to_vec()}
  }

  fn parse_grammar_decl(&mut self) -> Ident
  {
    let outer_attrs = self.parse_attributes();
    if !outer_attrs.is_empty() {
      self.rp.span_err(outer_attrs.iter().next().unwrap().span,
        "Unknown attribute. Use #![...] for global attributes.");
    }
    if !self.eat_grammar_keyword() {
      let token_string = self.rp.this_token_to_string();
      let span = self.rp.span;
      self.rp.span_fatal(span,
        format!("Expected grammar declaration (of the form: `grammar <grammar-name>;`), \
                but found `{}`",
          token_string).as_slice())
    }
    let grammar_name = self.rp.parse_ident();
    self.rp.expect(&rtok::Semi);
    grammar_name
  }

  fn eat_grammar_keyword(&mut self) -> bool
  {
    let is_grammar_kw = match self.rp.token {
      rtok::Ident(sid, rust::IdentStyle::Plain) => "grammar" == id_to_string(sid).as_slice(),
      _ => false
    };
    if is_grammar_kw { self.rp.bump() }
    is_grammar_kw
  }

  fn parse_blocks(&mut self) -> (Vec<Rule>, Vec<rust::P<rust::Item>>)
  {
    let mut rules = vec![];
    let mut rust_items = vec![];
    while self.rp.token != rtok::Eof
    {
      self.rp.parse_item(vec![]).map_or_else(
        || rules.push(self.parse_rule()),
        |item| rust_items.push(item))
    }
    (rules, rust_items)
  }

  fn parse_rule(&mut self) -> Rule
  {
    let outer_attrs = self.parse_attributes();
    let name = self.parse_rule_decl();
    self.rp.expect(&rtok::Eq);
    let body = self.parse_rule_rhs(id_to_string(name.node).as_slice());
    Rule{name: name, attributes: outer_attrs, def: body}
  }

  // Outer attributes are attached to the next item.
  // Inner attributes are attached to the englobing item.
  fn parse_attributes(&mut self) -> Vec<rust::Attribute>
  {
    let (inners, mut outers) = self.rp.parse_inner_attrs_and_next();
    self.inner_attrs.push_all(inners.as_slice());
    if !outers.is_empty() {
      outers.push_all(self.rp.parse_outer_attributes().as_slice());
    }
    outers
  }

  fn parse_rule_decl(&mut self) -> rust::SpannedIdent
  {
    let sp = self.rp.span;
    respan(sp, self.rp.parse_ident())
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
      let seq = self.parse_rule_seq(rule_name);
      let semantic_action = self.parse_semantic_action(seq);
      choices.push(semantic_action);
      let token = self.rp.token.clone();
      match token {
        rtok::BinOp(rbtok::Slash) => self.rp.bump(),
        _ => break
      }
    }
    let hi = self.rp.last_span.hi;
    if choices.len() == 1 {
      choices.pop().unwrap()
    } else {
      spanned_expr(lo, hi, Choice(choices))
    }
  }

  fn parse_semantic_action(&mut self, expr: Box<Expression>) -> Box<Expression>
  {
    let token = self.rp.token.clone();
    match token {
      rtok::Gt => {
        self.rp.bump();
        let fun_name = self.rp.parse_ident();
        self.last_respan(SemanticAction(expr, fun_name))
      }
      _ => expr
    }
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
        format!("In rule {}: must defined at least one expression.",
          rule_name).as_slice());
    }
    spanned_expr(lo, hi, Sequence(seq))
  }

  fn parse_rule_prefixed(&mut self, rule_name: &str) -> Option<Box<Expression>>
  {
    let token = self.rp.token.clone();
    match token {
      rtok::Not => {
        self.parse_prefix(rule_name, |e| NotPredicate(e))
      }
      rtok::BinOp(rbtok::And) => {
        self.parse_prefix(rule_name, |e| AndPredicate(e))
      }
      _ => self.parse_rule_suffixed(rule_name)
    }
  }

  fn parse_prefix<F>(&mut self, rule_name: &str, make_prefix: F) -> Option<Box<Expression>>
   where F: Fn(Box<Expression>) -> ExpressionNode
  {
    let lo = self.rp.span.lo;
    self.rp.bump();
    let expr = match self.parse_rule_suffixed(rule_name) {
      Some(expr) => expr,
      None => {
        let span = self.rp.span;
        self.rp.span_err(
          span,
          format!("In rule {}: A not predicate (`!expr`) is not followed by a \
            valid expression. Do not forget it must be in front of the expression.",
            rule_name).as_slice()
        );
        return None
      }
    };
    let hi = self.rp.span.hi;
    Some(spanned_expr(lo, hi, make_prefix(expr)))
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
      rtok::BinOp(rbtok::Star) => {
        self.rp.bump();
        Some(spanned_expr(lo, hi, ZeroOrMore(expr)))
      },
      rtok::BinOp(rbtok::Plus) => {
        self.rp.bump();
        Some(spanned_expr(lo, hi, OneOrMore(expr)))
      },
      rtok::Question => {
        self.rp.bump();
        Some(spanned_expr(lo, hi, Optional(expr)))
      },
      _ => Some(expr)
    }
  }

  fn last_respan(&self, expr: ExpressionNode) -> Box<Expression>
  {
    respan_expr(self.rp.last_span, expr)
  }

  fn parse_rule_atom(&mut self, rule_name: &str) -> Option<Box<Expression>>
  {
    let token = self.rp.token.clone();
    if token.is_keyword(rust::Keyword::Fn) { return None }
    match token {
      rtok::Literal(rust::token::Lit::Str_(name),_) => {
        self.rp.bump();
        Some(self.last_respan(StrLiteral(name_to_string(name))))
      },
      rtok::Dot => {
        self.rp.bump();
        Some(self.last_respan(AnySingleChar))
      },
      rtok::OpenDelim(rust::DelimToken::Paren) => {
        self.rp.bump();
        let res = self.parse_rule_rhs(rule_name);
        self.rp.expect(&rtok::CloseDelim(rust::DelimToken::Paren));
        Some(res)
      },
      rtok::Ident(id, _) => {
        if self.is_rule_lhs() { None }
        else {
          self.rp.bump();
          Some(self.last_respan(NonTerminalSymbol(id)))
        }
      },
      rtok::OpenDelim(rust::DelimToken::Bracket) => {
        self.rp.bump();
        let res = self.parse_char_class(rule_name);
        match self.rp.token {
          rtok::CloseDelim(rust::DelimToken::Bracket) => {
            self.rp.bump();
            res
          },
          _ => {
            let span = self.rp.span;
            self.rp.span_fatal(
              span,
              format!("In rule {}: A character class must always be terminated by `]` \
                and can only contain a string literal (such as in `[\"a-z\"]`",
                rule_name).as_slice()
            );
          }
        }
      },
      _ => { None }
    }
  }

  fn parse_char_class(&mut self, rule_name: &str) -> Option<Box<Expression>>
  {
    let token = self.rp.token.clone();
    match token {
      rtok::Literal(rust::token::Lit::Str_(name),_) => {
        self.rp.bump();
        let cooked_lit = rust::str_lit(name_to_string(name).as_slice());
        self.parse_set_of_char_range(&cooked_lit, rule_name)
      },
      _ => {
        let span = self.rp.span;
        self.rp.span_fatal(
          span,
          format!("In rule {}: An expected character occurred in this character class. \
            `[` must only be followed by a string literal (such as in `[\"a-z\"]`",
            rule_name).as_slice()
        );
      }
    }
  }

  fn parse_set_of_char_range(&mut self, ranges: &String, rule_name: &str) -> Option<Box<Expression>>
  {
    let ranges = ranges.as_slice();
    let mut ranges = ranges.chars();
    let mut intervals = vec![];
    match ranges.peekable().peek() {
      Some(&sep) if sep == '-' => {
        intervals.push(CharacterInterval{lo: '-', hi: '-'});
        ranges.next();
      }
      _ => ()
    }
    loop {
      match self.parse_char_range(&mut ranges, rule_name) {
        Some(char_set) => intervals.push_all(char_set.as_slice()),
        None => break
      }
    }
    Some(respan_expr(self.rp.span, CharacterClass(CharacterClassExpr{intervals: intervals})))
  }

  fn parse_char_range<'b>(&mut self, ranges: &mut Chars<'b>, rule_name: &str) -> Option<Vec<CharacterInterval>>
  {
    let mut res = vec![];
    let separator_err = format!(
      "In rule {}: Unexpected separator `-`. Put it in the start or the end if you want \
      to accept it as a character in the set. Otherwise, you should only use it for \
      character intervals as in `[\"a-z\"]`.",
      rule_name);
    let span = self.rp.span;
    let lo = ranges.next();
    let mut peekable = ranges.peekable();
    let next = peekable.peek();
    match (lo, next) {
      (Some('-'), Some(_)) => {
        self.rp.span_err(span, separator_err.as_slice());
        None
      },
      (Some(lo), Some(&sep)) if sep == '-' => {
        ranges.next();
        match ranges.next() {
          Some('-') => { self.rp.span_err(span, separator_err.as_slice()); None }
          Some(hi) => {
            res.push(CharacterInterval{lo: lo, hi: hi});
            Some(res)
          }
          None => {
            res.push(CharacterInterval{lo:lo, hi:lo});
            res.push(CharacterInterval{lo:'-', hi: '-'});
            Some(res)
          }
        }

      },
      (Some(lo), _) => {
        res.push(CharacterInterval{lo: lo, hi: lo}); // If lo == '-', it ends the class, allowed.
        Some(res)
      }
      (None, _) => None
    }
  }

  fn is_rule_lhs(&mut self) -> bool
  {
    self.rp.look_ahead(1, |t| match t { &rtok::Eq => true, _ => false})
  }
}
