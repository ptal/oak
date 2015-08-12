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
use rust;
use std::str::Chars;
use std::iter::Peekable;

use front::ast::*;
use front::ast::Expression_::*;

pub struct Parser<'a>
{
  rp: rust::Parser<'a>,
  inner_attrs: Vec<rust::Attribute>,
  grammar_name: rust::Ident
}

impl<'a> Parser<'a>
{
  pub fn new(sess: &'a rust::ParseSess,
         cfg: rust::CrateConfig,
         tts: Vec<rust::TokenTree>,
         grammar_name: rust::Ident) -> Parser<'a>
  {
    Parser{
      rp: rust::new_parser_from_tts(sess, cfg, tts),
      inner_attrs: Vec::new(),
      grammar_name: grammar_name
    }
  }

  pub fn parse_grammar(&mut self) -> Grammar {
    let (rules, rust_items) = self.parse_blocks();
    Grammar{name: self.grammar_name, rules: rules, rust_items: rust_items, attributes: self.inner_attrs.to_vec()}
  }

  fn bump(&mut self) {
    self.rp.bump().unwrap()
  }

  fn parse_blocks(&mut self) -> (Vec<Rule>, Vec<RItem>) {
    let mut rules = vec![];
    let mut rust_items = vec![];
    while self.rp.token != rtok::Eof
    {
      // FIXME: #45
      if self.rp.token == rust::token::Pound {
        rules.push(self.parse_rule())
      }
      else {
        self.rp.parse_item().map_or_else(
          || rules.push(self.parse_rule()),
          |item| rust_items.push(item))
      }
    }
    (rules, rust_items)
  }

  fn parse_rule(&mut self) -> Rule {
    let outer_attrs = self.parse_attributes();
    let name = self.parse_rule_decl();
    self.rp.expect(&rtok::Eq).unwrap();
    let body = self.parse_rule_rhs(id_to_string(name.node).as_str());
    Rule{name: name, attributes: outer_attrs, def: body}
  }

  // Outer attributes are attached to the next item.
  // Inner attributes are attached to the englobing item.
  fn parse_attributes(&mut self) -> Vec<rust::Attribute> {
    let inners = self.rp.parse_inner_attributes();
    self.inner_attrs.push_all(inners.as_slice());
    self.rp.parse_outer_attributes()
  }

  fn parse_rule_decl(&mut self) -> rust::SpannedIdent {
    let sp = self.rp.span;
    respan(sp, self.rp.parse_ident().unwrap())
  }

  fn parse_rule_rhs(&mut self, rule_name: &str) -> Box<Expression> {
    self.parse_rule_choice(rule_name)
  }

  fn parse_rule_choice(&mut self, rule_name: &str) -> Box<Expression> {
    let lo = self.rp.span.lo;
    let mut choices = Vec::new();
    loop{
      let seq = self.parse_rule_seq(rule_name);
      let semantic_action = self.parse_semantic_action(seq);
      choices.push(semantic_action);
      let token = self.rp.token.clone();
      match token {
        rtok::BinOp(rbtok::Slash) => self.bump(),
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

  fn parse_semantic_action(&mut self, expr: Box<Expression>) -> Box<Expression> {
    let token = self.rp.token.clone();
    match token {
      rtok::Gt => {
        self.bump();
        let fun_name = self.rp.parse_ident().unwrap();
        self.last_respan(SemanticAction(expr, fun_name))
      }
      _ => expr
    }
  }

  fn parse_rule_seq(&mut self, rule_name: &str) -> Box<Expression> {
    let lo = self.rp.span.lo;
    let mut seq = Vec::new();
    loop{
      match self.parse_rule_prefixed(rule_name){
        Some(expr) => {
          let expr = self.parse_type_annotation(expr, rule_name);
          seq.push(expr)
        },
        None => break
      }
    }
    let hi = self.rp.last_span.hi;
    if seq.len() == 0 {
      self.rp.span_err(
        mk_sp(lo, hi),
        format!("In rule {}: must defined at least one expression.",
          rule_name).as_str());
    }
    spanned_expr(lo, hi, Sequence(seq))
  }

  // `e -> ty`
  fn parse_type_annotation(&mut self, expr: Box<Expression>, rule_name: &str) -> Box<Expression> {
    let token = self.rp.token.clone();
    match token {
      rtok::RArrow => {
        self.bump();
        self.parse_type(expr, rule_name)
      },
      _ => expr
    }
  }

  // `()` or `(^)`
  fn parse_type(&mut self, mut expr: Box<Expression>, rule_name: &str) -> Box<Expression> {
    let token = self.rp.token.clone();
    match token {
      rtok::OpenDelim(rust::DelimToken::Paren) => {
        self.bump();
        let token = self.rp.token.clone();
        let mut ty = TypeAnnotation::Unit;
        if token == rtok::BinOp(rbtok::Caret) {
          self.bump();
          ty = TypeAnnotation::Invisible;
        }
        self.rp.expect(&rtok::CloseDelim(rust::DelimToken::Paren)).unwrap();
        expr.ty = Some(ty);
        expr
      }
      _ => {
        let span = self.rp.span;
        self.rp.span_err(
          span,
          format!("In rule {}: Unknown token after `->`. Use the arrow to annotate an expression with the unit type `()` or the invisible type `(^)`.",
            rule_name).as_str()
        );
        expr
      }
    }
  }

  fn parse_rule_prefixed(&mut self, rule_name: &str) -> Option<Box<Expression>> {
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
    self.bump();
    let expr = match self.parse_rule_suffixed(rule_name) {
      Some(expr) => expr,
      None => {
        let span = self.rp.span;
        self.rp.span_err(
          span,
          format!("In rule {}: A not predicate (`!expr`) is not followed by a \
            valid expression. Do not forget it must be in front of the expression.",
            rule_name).as_str()
        );
        return None
      }
    };
    let hi = self.rp.span.hi;
    Some(spanned_expr(lo, hi, make_prefix(expr)))
  }

  fn parse_rule_suffixed(&mut self, rule_name: &str) -> Option<Box<Expression>> {
    let lo = self.rp.span.lo;
    let expr = match self.parse_rule_atom(rule_name){
      Some(expr) => expr,
      None => return None
    };
    let hi = self.rp.span.hi;
    let token = self.rp.token.clone();
    match token {
      rtok::BinOp(rbtok::Star) => {
        self.bump();
        Some(spanned_expr(lo, hi, ZeroOrMore(expr)))
      },
      rtok::BinOp(rbtok::Plus) => {
        self.bump();
        Some(spanned_expr(lo, hi, OneOrMore(expr)))
      },
      rtok::Question => {
        self.bump();
        Some(spanned_expr(lo, hi, Optional(expr)))
      },
      _ => Some(expr)
    }
  }

  fn last_respan(&self, expr: ExpressionNode) -> Box<Expression> {
    respan_expr(self.rp.last_span, expr)
  }

  fn parse_rule_atom(&mut self, rule_name: &str) -> Option<Box<Expression>> {
    let token = self.rp.token.clone();
    if token.is_any_keyword() {
      return None
    }
    match token {
      rtok::Literal(rust::token::Lit::Str_(name),_) => {
        self.bump();
        Some(self.last_respan(StrLiteral(name_to_string(name))))
      },
      rtok::Dot => {
        self.bump();
        Some(self.last_respan(AnySingleChar))
      },
      rtok::OpenDelim(rust::DelimToken::Paren) => {
        self.bump();
        let res = self.parse_rule_rhs(rule_name);
        self.rp.expect(&rtok::CloseDelim(rust::DelimToken::Paren)).unwrap();
        Some(res)
      },
      rtok::Ident(id, _) => {
        if self.is_rule_lhs() { None }
        else {
          self.bump();
          Some(self.last_respan(NonTerminalSymbol(id)))
        }
      },
      rtok::OpenDelim(rust::DelimToken::Bracket) => {
        self.bump();
        let res = self.parse_char_class(rule_name);
        match self.rp.token {
          rtok::CloseDelim(rust::DelimToken::Bracket) => {
            self.bump();
            res
          },
          _ => {
            let span = self.rp.span;
            panic!(self.rp.span_fatal(
              span,
              format!("In rule {}: A character class must always be terminated by `]` \
                and can only contain a string literal (such as in `[\"a-z\"]`",
                rule_name).as_str()
            ));
          }
        }
      },
      _ => { None }
    }
  }

  fn parse_char_class(&mut self, rule_name: &str) -> Option<Box<Expression>> {
    let token = self.rp.token.clone();
    match token {
      rtok::Literal(rust::token::Lit::Str_(name),_) => {
        self.bump();
        let cooked_lit = rust::str_lit(name_to_string(name).as_str());
        self.parse_set_of_char_range(&cooked_lit, rule_name)
      },
      _ => {
        let span = self.rp.span;
        panic!(self.rp.span_fatal(
          span,
          format!("In rule {}: An expected character occurred in this character class. \
            `[` must only be followed by a string literal (such as in `[\"a-z\"]`",
            rule_name).as_str()
        ));
      }
    }
  }

  fn parse_set_of_char_range(&mut self, ranges: &String, rule_name: &str) -> Option<Box<Expression>> {
    let mut ranges = ranges.chars().peekable();
    let mut intervals = vec![];
    match ranges.peek() {
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

  fn parse_char_range<'b>(&mut self, ranges: &mut Peekable<Chars<'b>>, rule_name: &str) -> Option<Vec<CharacterInterval>> {
    let mut res = vec![];
    let separator_err = format!(
      "In rule {}: Unexpected separator `-`. Put it in the start or the end if you want \
      to accept it as a character in the set. Otherwise, you should only use it for \
      character intervals as in `[\"a-z\"]`.",
      rule_name);
    let span = self.rp.span;
    let lo = ranges.next();
    // Twisted logic due to the fact that `peek` borrows the ranges...
    let lo = {
      let next = ranges.peek();
      match (lo, next) {
        (Some('-'), Some(_)) => {
          self.rp.span_err(span, separator_err.as_str());
          return None
        },
        (Some(lo), Some(&sep)) if sep == '-' => {
          lo
        },
        (Some(lo), _) => {
          res.push(CharacterInterval{lo: lo, hi: lo}); // If lo == '-', it ends the class, allowed.
          return Some(res)
        }
        (None, _) => return None
      }
    };
    ranges.next();
    match ranges.next() {
      Some('-') => { self.rp.span_err(span, separator_err.as_str()); None }
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
  }

  fn is_rule_lhs(&mut self) -> bool {
    self.rp.look_ahead(1, |t| match t { &rtok::Eq => true, _ => false})
  }
}
