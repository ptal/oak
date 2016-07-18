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

use rust::respan;
use rust::Token as rtok;
use rust::BinOpToken as rbtok;
use rust;
use std::str::Chars;
use std::iter::Peekable;

use front::ast::*;
use front::ast::Expression::*;

pub struct Parser<'a>
{
  rp: rust::Parser<'a>,
  grammar: FGrammar
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
      grammar: FGrammar::new(grammar_name)
    }
  }

  fn alloc_expr(&mut self, lo: BytePos, hi: BytePos, expr: Expression) -> usize {
    self.grammar.alloc_expr(lo, hi, expr)
  }

  pub fn parse_grammar(mut self) -> rust::PResult<'a, FGrammar> {
    try!(self.parse_blocks());
    Ok(self.grammar)
  }

  fn bump(&mut self) {
    self.rp.bump()
  }

  fn parse_blocks(&mut self) -> rust::PResult<'a, ()> {
    while self.rp.token != rtok::Eof
    {
      try!(self.parse_inner_attributes());
      match try!(self.rp.parse_item()) {
        None => try!(self.parse_rule()),
        Some(item) => self.grammar.push_rust_item(item),
      }
    }
    Ok(())
  }

  fn parse_rule(&mut self) -> rust::PResult<'a, ()> {
    let outer_attrs = try!(self.rp.parse_outer_attributes());
    let name = try!(self.parse_rule_decl());
    try!(self.rp.expect(&rtok::Eq));
    let body = try!(self.parse_rule_rhs(ident_to_string(name.node).as_str()));
    self.grammar.push_rule(name, outer_attrs, body);
    Ok(())
  }

  fn parse_inner_attributes(&mut self) -> rust::PResult<'a, ()> {
    let inners = try!(self.rp.parse_inner_attributes());
    for attr in inners {
      self.grammar.push_attr(attr);
    }
    Ok(())
  }

  fn parse_rule_decl(&mut self) -> rust::PResult<'a, rust::SpannedIdent> {
    let sp = self.rp.span;
    Ok(respan(sp, try!(self.rp.parse_ident())))
  }

  fn parse_rule_rhs(&mut self, rule_name: &str) -> rust::PResult<'a, usize> {
    self.parse_rule_choice(rule_name)
  }

  fn parse_rule_choice(&mut self, rule_name: &str) -> rust::PResult<'a, usize> {
    let lo = self.rp.span.lo;
    let mut choices = Vec::new();
    loop{
      let seq = try!(self.parse_rule_seq(rule_name));
      choices.push(try!(self.parse_semantic_action_or_ty(seq, rule_name)));
      let token = self.rp.token.clone();
      match token {
        rtok::BinOp(rbtok::Slash) => self.bump(),
        _ => break
      }
    }
    let hi = self.rp.last_span.hi;
    let res = if choices.len() == 1 {
      choices.pop().unwrap()
    } else {
      self.alloc_expr(lo, hi, Choice(choices))
    };
    Ok(res)
  }

  fn parse_semantic_action_or_ty(&mut self, expr: usize, rule_name: &str) -> rust::PResult<'a, usize> {
    let token = self.rp.token.clone();
    let lo = self.rp.span.lo;
    match token {
      rtok::Gt => {
        self.bump();
        let fun_name = try!(self.rp.parse_ident());
        let hi = self.rp.span.hi;
        Ok(self.alloc_expr(lo, hi, SemanticAction(expr, fun_name)))
      },
      rtok::RArrow => {
        self.bump();
        self.parse_type(lo, expr, rule_name)
      }
      _ => Ok(expr)
    }
  }

  // `()` or `(^)`
  fn parse_type(&mut self, lo: BytePos, expr: usize, rule_name: &str) -> rust::PResult<'a, usize> {
    let token = self.rp.token.clone();
    match token {
      rtok::OpenDelim(rust::DelimToken::Paren) => {
        self.bump();
        let token = self.rp.token.clone();
        let ty =
          if token == rtok::BinOp(rbtok::Caret) {
            self.bump();
            IType::Invisible
          }
          else {
            IType::Regular(Type::Unit)
          };
        try!(self.rp.expect(&rtok::CloseDelim(rust::DelimToken::Paren)));
        let hi = self.rp.span.hi;
        Ok(self.alloc_expr(lo, hi, TypeAscription(expr, ty)))
      }
      _ => {
        let span = self.rp.span;
        self.rp.span_err(
          span,
          format!("In rule {}: Unknown token after `->`. Use the arrow to annotate an expression with the unit type `()` or the invisible type `(^)`.",
            rule_name).as_str()
        );
        Ok(expr)
      }
    }
  }

  fn parse_rule_seq(&mut self, rule_name: &str) -> rust::PResult<'a, usize> {
    let lo = self.rp.span.lo;
    let mut seq = Vec::new();
    while let Some(expr) = try!(self.parse_rule_prefixed(rule_name)) {
      seq.push(expr);
    }
    let hi = self.rp.last_span.hi;
    if seq.len() == 0 {
      self.rp.span_err(
        mk_sp(lo, hi),
        format!("In rule {}: must define at least one expression.",
          rule_name).as_str())
    }
    else if seq.len() == 1 {
      return Ok(seq[0]);
    }
    Ok(self.alloc_expr(lo, hi, Sequence(seq)))
  }

  fn parse_rule_prefixed(&mut self, rule_name: &str) -> rust::PResult<'a, Option<usize>> {
    let token = self.rp.token.clone();
    match token {
      rtok::Not => {
        self.parse_prefix(rule_name, |e| NotPredicate(e), "A not predicate (`!expr`)").map(Some)
      }
      rtok::BinOp(rbtok::And) => {
        self.parse_prefix(rule_name, |e| AndPredicate(e), "An and predicate (`&expr`)").map(Some)
      }
      _ => self.parse_rule_suffixed(rule_name)
    }
  }

  fn parse_prefix<F>(&mut self, rule_name: &str, make_prefix: F, pred_name: &str) -> rust::PResult<'a, usize>
   where F: Fn(usize) -> Expression
  {
    let lo = self.rp.span.lo;
    self.bump();
    match try!(self.parse_rule_suffixed(rule_name)) {
      Some(expr) => {
        let hi = self.rp.span.hi;
        Ok(self.alloc_expr(lo, hi, make_prefix(expr)))
      }
      None => {
        Err(self.fatal_error(
          format!("In rule {}: {} is not followed by a valid expression.
            Do not forget it must be in front of the expression.",
            rule_name, pred_name).as_str()
        ))
      }
    }
  }

  fn parse_rule_suffixed(&mut self, rule_name: &str) -> rust::PResult<'a, Option<usize>> {
    let lo = self.rp.span.lo;
    let expr = match try!(self.parse_rule_atom(rule_name)) {
      Some(expr) => expr,
      None => return Ok(None),
    };
    let hi = self.rp.span.hi;
    let token = self.rp.token.clone();
    let res = match token {
      rtok::BinOp(rbtok::Star) => {
        self.bump();
        self.alloc_expr(lo, hi, ZeroOrMore(expr))
      },
      rtok::BinOp(rbtok::Plus) => {
        self.bump();
        self.alloc_expr(lo, hi, OneOrMore(expr))
      },
      rtok::Question => {
        self.bump();
        self.alloc_expr(lo, hi, ZeroOrOne(expr))
      },
      _ => expr
    };
    Ok(Some(res))
  }

  fn last_respan(&mut self, expr: Expression) -> usize {
    let sp = self.rp.last_span;
    self.alloc_expr(sp.lo, sp.hi, expr)
  }

  fn fatal_error(&mut self, err_msg: &str) -> rust::DiagnosticBuilder<'a> {
    let span = self.rp.span;
    self.rp.span_fatal(span, err_msg)
  }

  fn parse_rule_atom(&mut self, rule_name: &str) -> rust::PResult<'a, Option<usize>> {
    let token = self.rp.token.clone();
    let res = match token {
      rtok::Literal(rust::token::Lit::Str_(name),_) => {
        self.bump();
        let cooked_lit = cook_lit(name);
        Some(self.last_respan(StrLiteral(cooked_lit)))
      },
      rtok::Dot => {
        self.bump();
        Some(self.last_respan(AnySingleChar))
      },
      rtok::OpenDelim(rust::DelimToken::Paren) => {
        self.bump();
        let res = try!(self.parse_rule_rhs(rule_name));
        try!(self.rp.expect(&rtok::CloseDelim(rust::DelimToken::Paren)));
        Some(res)
      },
      rtok::Ident(id) if !token.is_any_keyword() => {
        if self.is_rule_lhs() { None }
        else {
          self.bump();
          Some(self.last_respan(NonTerminalSymbol(id)))
        }
      },
      rtok::OpenDelim(rust::DelimToken::Bracket) => {
        self.bump();
        let res = try!(self.parse_char_class(rule_name));
        match self.rp.token {
          rtok::CloseDelim(rust::DelimToken::Bracket) => {
            self.bump();
            Some(res)
          },
          _ => {
            return Err(self.fatal_error(
              format!("In rule {}: A character class must always be terminated by `]` \
                and can only contain a string literal (such as in `[\"a-z\"]`",
                rule_name).as_str()
            ));
          }
        }
      },
      _ => { None }
    };
    Ok(res)
  }

  fn parse_char_class(&mut self, rule_name: &str) -> rust::PResult<'a, usize> {
    let token = self.rp.token.clone();
    match token {
      rtok::Literal(rust::token::Lit::Str_(name),_) => {
        let span = self.rp.span;
        self.bump();
        let cooked_lit = cook_lit(name);
        if cooked_lit.is_empty() {
          self.rp.span_err(span,
            "Empty character classes is forbidden. For empty expression \
            you can use the empty string literal `\"\"`.");
        }
        Ok(self.parse_set_of_char_range(&cooked_lit, rule_name))
      },
      _ => {
        Err(self.fatal_error(
          format!("In rule {}: Unexpected character in this character class. \
            `[` must only be followed by a string literal (such as in `[\"a-z\"]`)",
            rule_name).as_str()
        ))
      }
    }
  }

  fn parse_set_of_char_range(&mut self, ranges: &String, rule_name: &str) -> usize {
    let mut ranges = ranges.chars().peekable();
    let mut intervals = vec![];
    match ranges.peek() {
      Some(&sep) if sep == '-' => {
        intervals.push(CharacterInterval::new('-', '-'));
        ranges.next();
      }
      _ => ()
    }
    loop {
      let char_set = self.parse_char_range(&mut ranges, rule_name);
      intervals.extend_from_slice(char_set.as_slice());
      if char_set.is_empty() {
          break;
      }
    }
    let sp = self.rp.span;
    self.alloc_expr(sp.lo, sp.hi, CharacterClass(CharacterClassExpr::new(intervals)))
  }

  fn parse_char_range<'b>(&mut self, ranges: &mut Peekable<Chars<'b>>, rule_name: &str) -> Vec<CharacterInterval> {
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
          return res;
        }
        (Some(lo), Some(&sep)) if sep == '-' => {
          lo
        },
        (Some(lo), _) => {
          res.push(CharacterInterval::new(lo, lo)); // If lo == '-', it ends the class, allowed.
          return res;
        }
        (None, _) => return res,
      }
    };
    ranges.next();
    match ranges.next() {
      Some('-') => { self.rp.span_err(span, separator_err.as_str()); }
      Some(hi) => {
        res.push(CharacterInterval::new(lo, hi));
      }
      None => {
        res.push(CharacterInterval::new(lo, lo));
        res.push(CharacterInterval::new('-', '-'));
      }
    };
    res
  }

  fn is_rule_lhs(&mut self) -> bool {
    self.rp.look_ahead(1, |t| match t { &rtok::Eq => true, _ => false})
  }
}
