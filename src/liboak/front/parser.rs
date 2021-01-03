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

use std::str::Chars;
use std::iter::Peekable;

use syn::{Token, Ident, Attribute, Result, Error, LitStr, parenthesized, bracketed};
use syn::parse::{Parse, ParseStream};

use front::ast::*;
use front::ast::Expression::*;

impl Parse for FGrammar {
  fn parse(ps: ParseStream) -> Result<Self> {
    let mut grammar = FGrammar::new(ps.span());
    grammar.parse_blocks(ps)?;
    Ok(grammar)
  }
}

impl FGrammar {
  fn span_of(&self, index: usize) -> Span {
    (&self.exprs_info[index] as &FExpressionInfo).span()
  }

  fn parse_blocks(&mut self, ps: ParseStream) -> Result<()> {
    while !ps.is_empty() {
      self.push_attrs(ps.call(Attribute::parse_inner)?);
      if self.peek_rule_lhs(ps) {
        self.parse_rule(ps)?;
      }
      else {
        self.push_rust_item(ps.parse()?);
      }
    }
    Ok(())
  }

  // A rule can have two shapes:
  //   1. rule1 = ...     (untyped)
  //   2. rule2:ty = ...  (typed)
  // I am unsure if it is enough to just check <ident> followed by ":" for the typed version, or if we need to perform more speculative parsing.
  fn peek_rule_lhs(&mut self, ps: ParseStream) -> bool {
    if ps.peek(Ident) {
      if ps.peek2(Token![=]) {
        true
      }
      else {
        let ps2 = ps.fork();
        let _: Result<Ident> = ps2.parse();
        match Self::parse_type(&ps2) {
          Ok(_) => {
            ps2.peek(Token![=])
          }
          _ => { false }
        }
      }
    }
    else { false }
  }

  fn parse_rule(&mut self, ps: ParseStream) -> Result<()> {
    let name: Ident = ps.parse()?;
    let (span, ty) = Self::parse_type(ps)?;
    let _: Token![=] = ps.parse()?;
    let mut body = self.parse_rule_choice(ps, name.to_string().as_str())?;
    // `r: T = e` is turned into `r = e:T`.
    if ty != IType::Infer {
      body = self.alloc_expr(span, TypeAscription(body, ty))
    }
    self.push_rule(name, body);
    Ok(())
  }

  fn parse_rule_choice(&mut self, ps: ParseStream, rule_name: &str) -> Result<usize> {
    let mut choices = Vec::new();
    loop {
      let spanned_expr = self.parse_spanned_expr(ps, rule_name)?;
      choices.push(self.parse_semantic_action(ps, spanned_expr)?);
      if ps.peek(Token![/]) {
        let _: Token![/] = ps.parse()?;
      }
      else {
        break
      }
    }
    let res =
      if choices.len() == 1 {
        choices.pop().unwrap()
      } else {
        let lo = self.span_of(choices[0]);
        let hi = self.span_of(choices[choices.len() - 1]);
        self.alloc_expr(lo.join(hi).unwrap(), Choice(choices))
      };
    Ok(res)
  }

  fn peek_unit_type(ps: ParseStream, invisible: bool) -> bool {
    let ps2 = ps.fork();
    let try = || {
      let sub_ps;
      let _ = parenthesized!(sub_ps in ps2);
      if invisible {
        Ok(sub_ps.peek(Token![^]))
      }
      else {
        Ok(sub_ps.is_empty())
      }
    };
    match try() {
      Err(_) => false,
      Ok(b) => b
    }
  }

  // `:` followed by `()` or `(^)` or a Rust type.
  // For instance: `rule: ast::Expr = r1:(^) "let" r2:u32`.
  fn parse_type(ps: ParseStream) -> Result<(Span, IType)> {
    if !ps.peek(Token![:]) {
      Ok((ps.span(), IType::Infer))
    }
    else {
      let _: Token![:] = ps.parse()?;
      let span = ps.span();
      if Self::peek_unit_type(ps, false) {
        let _sub_ps;
        let _ = parenthesized!(_sub_ps in ps);
        Ok((span, IType::Regular(Type::Unit)))
      }
      else if Self::peek_unit_type(ps, true) {
        let sub_ps;
        let _ = parenthesized!(sub_ps in ps);
        let _: Token![^] = sub_ps.parse()?;
        Ok((span, IType::Invisible))
      }
      else {
        let ty: Result<syn::Type> = ps.parse();
        match ty {
          Ok(ty) => Ok((span, IType::Regular(Type::Rust(ty)))),
          Err(mut err) => {
            err.combine(Error::new(err.span(),
            "note: you might need to parenthesize the Rust type.\n\
             For instance `rule:ast::Expr (rule2 rule3)` might generate an error because the Rust parser fails on parsing `ast::Expr(rule2 rule3)` thinking it is a Rust type.\n\
             Instead, you can write `rule:(ast::Expr) (rule2 rule3)` or `(rule:ast::Expr) (rule2 rule3)`."));
            Err(err)
          }
        }
      }
    }
  }

  fn parse_semantic_action(&mut self, ps: ParseStream, expr: usize) -> Result<usize> {
    if ps.peek(Token![>]) {
      let _: Token![>] = ps.parse()?;
      let span = ps.span();
      let action: syn::Expr =
        if ps.peek(syn::token::Brace) {
          let action: syn::ExprBlock = ps.parse()?;
          syn::Expr::Block(action)
        }
        else {
          let action: syn::ExprPath = ps.parse()?;
          syn::Expr::Path(action)
        };
      Ok(self.alloc_expr(span, SemanticAction(expr, action)))
    }
    else {
      Ok(expr)
    }
  }

  // An expression starting with `..` to capture the span of the current sequence.
  fn parse_spanned_expr(&mut self, ps: ParseStream, rule_name: &str) -> Result<usize> {
    if ps.peek(Token![..]) {
      let span = ps.span();
      let _: Token![..] = ps.parse()?;
      let seq = self.parse_seq(ps, rule_name)?;
      Ok(self.alloc_expr(span, SpannedExpr(seq)))
    }
    else {
      self.parse_seq(ps, rule_name)
    }
  }

  fn parse_seq(&mut self, ps: ParseStream, rule_name: &str) -> Result<usize> {
    let lo = ps.span();
    let mut seq = Vec::new();
    while let Some(expr) = self.parse_typed_expr(ps, rule_name)? {
      seq.push(expr);
    }
    if seq.len() == 0 {
      return Err(Error::new(lo, format!("expect at least one expression (in rule `{}`).",
          rule_name).as_str()))
    }
    else if seq.len() == 1 {
      return Ok(seq[0]);
    }
    let hi = self.span_of(seq[seq.len() - 1]);
    Ok(self.alloc_expr(lo.join(hi).unwrap(), Sequence(seq)))
  }

  fn parse_typed_expr(&mut self, ps: ParseStream, rule_name: &str) -> Result<Option<usize>> {
    let expr = self.parse_prefixed_expr(ps, rule_name)?;
    match Self::parse_type(ps)? {
      (_, IType::Infer) => { Ok(expr) }
      (span, ty) => {
        match expr {
          None => { Err(Error::new(span, format!("an expression must precede a type ascription (in rule `{}`). \
              For instance: `r1:u32` or `([\"0-9\"]+):()`.", rule_name).as_str())) }
          Some(expr) => { Ok(Some(self.alloc_expr(span, TypeAscription(expr, ty)))) }
        }
      }
    }
  }

  // Parse prefixed expressions of the form `!e` and `&e`.
  fn parse_prefixed_expr(&mut self, ps: ParseStream, rule_name: &str) -> Result<Option<usize>> {
    let span = ps.span();
    if ps.peek(Token![!]) {
      let _: Token![!] = ps.parse()?;
      self.parse_prefixed_expr2(ps, span, rule_name, |e| NotPredicate(e), "A 'not' predicate (`!expr`)").map(Some)
    }
    else if ps.peek(Token![&]) {
      let _: Token![&] = ps.parse()?;
      self.parse_prefixed_expr2(ps, span, rule_name, |e| AndPredicate(e), "A 'and' predicate (`&expr`)").map(Some)
    }
    else {
      self.parse_suffixed_expr(ps, rule_name)
    }
  }

  fn parse_prefixed_expr2<F>(&mut self, ps: ParseStream, lo: Span, rule_name: &str, make_prefix: F, pred_name: &str) -> Result<usize>
   where F: Fn(usize) -> Expression
  {
    match self.parse_suffixed_expr(ps, rule_name)? {
      Some(expr) => {
        let span = lo.join(self.span_of(expr)).unwrap();
        Ok(self.alloc_expr(span, make_prefix(expr)))
      }
      None => {
        Err(Error::new(lo,
          format!("{} is not followed by a valid expression (in rule {}).
            Do not forget it must be in front of the expression.",
            rule_name, pred_name).as_str()
        ))
      }
    }
  }

  // Parse suffixed expressions of the form `e*`, `e+` and `e?`.
  fn parse_suffixed_expr(&mut self, ps: ParseStream, rule_name: &str) -> Result<Option<usize>> {
    let lo = ps.span();
    let expr = match self.parse_rule_atom(ps, rule_name)? {
      Some(expr) => expr,
      None => return Ok(None),
    };
    let span = lo.join(ps.span()).unwrap();
    let res =
      if ps.peek(Token![*]) {
        let _: Token![*] = ps.parse()?;
        self.alloc_expr(span, ZeroOrMore(expr))
      }
      else if ps.peek(Token![+]) {
        let _: Token![+] = ps.parse()?;
        self.alloc_expr(span, OneOrMore(expr))
      }
      else if ps.peek(Token![?]) {
        let _: Token![?] = ps.parse()?;
        self.alloc_expr(span, ZeroOrOne(expr))
      }
      else { expr };
    Ok(Some(res))
  }

  fn peek_paren(ps: ParseStream) -> bool {
    let ps2 = ps.fork();
    let try = || {
      let _sub_ps;
      let _ = parenthesized!(_sub_ps in ps2);
      Ok(())
    };
    match try() {
      Err(_) => false,
      Ok(_) => true
    }
  }

  fn peek_bracket(ps: ParseStream) -> bool {
    let ps2 = ps.fork();
    let try = || {
      let _sub_ps;
      let _ = bracketed!(_sub_ps in ps2);
      Ok(())
    };
    match try() {
      Err(_) => false,
      Ok(_) => true
    }
  }

  fn peek_path(ps: ParseStream) -> bool {
    let ps2 = ps.fork();
    let res: Result<syn::Path> = ps2.parse();
    res.is_ok()
  }

  fn parse_rule_atom(&mut self, ps: ParseStream, rule_name: &str) -> Result<Option<usize>> {
    let span = ps.span();
    let res =
      // String literal "let", "fn", ...
      if ps.peek(LitStr) {
        let lit_str: LitStr = ps.parse()?;
        Some(self.alloc_expr(span, StrLiteral(lit_str.value())))
      }
      // Any character `.`
      else if ps.peek(Token![.]) {
        let _: Token![.] = ps.parse()?;
        Some(self.alloc_expr(span, AnySingleChar))
      }
      // Parenthesized expression `(r1 / r2)`
      else if Self::peek_paren(ps) {
        let sub_ps;
        let _ = parenthesized!(sub_ps in ps);
        if sub_ps.is_empty() {
          return Err(Error::new(span,
            format!("unit type must follow an expression (in rule {}).
              For instance: `[\"0-9\"]:()`.", rule_name).as_str()))
        }
        Some(self.parse_rule_choice(&sub_ps, rule_name)?)
      }
      // Rule call `r1`
      else if Self::peek_path(ps) {
        if self.peek_rule_lhs(ps) { None }
        else {
          let name: syn::Path = ps.parse()?;
          Some(self.alloc_expr(span, ExternalNonTerminalSymbol(name)))
        }
      }
      // Character class `["0-9"]`
      else if Self::peek_bracket(ps) {
        let sub_ps;
        let _ = bracketed!(sub_ps in ps);
        Some(self.parse_char_class(&sub_ps, span, rule_name)?)
      }
      else if ps.peek(Token![..]) {
        return Err(Error::new(span,
          format!("A span expression `.. e1 e2` must always start a sequence (in rule {}). \
            You can force this by grouping the spanned expression with parenthesis: `e1 (.. e2)` instead of `e1 .. e2`.",
            rule_name).as_str()));
      }
      else {
        None
      };
    Ok(res)
  }

  fn parse_char_class(&mut self, ps: ParseStream, span: Span, rule_name: &str) -> Result<usize> {
    if ps.peek(LitStr) {
      let lit_str: LitStr = ps.parse()?;
      if lit_str.value().is_empty() {
        return Err(Error::new(span,
          "Empty character classes are forbidden. For empty expression \
          you can use the empty string literal `\"\"`."))
      }
      self.parse_set_of_char_range(span, lit_str.value(), rule_name)
    }
    else {
      Err(Error::new(span,
        format!("Unexpected character in this character class (in rule {}). \
            `[` must only be followed by a string literal (such as in `[\"a-z\"]`).", rule_name).as_str()))
    }
  }

  fn parse_set_of_char_range(&mut self, span: Span, ranges: String, rule_name: &str) -> Result<usize> {
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
      let char_set = self.parse_char_range(span, &mut ranges, rule_name)?;
      intervals.extend_from_slice(char_set.as_slice());
      if char_set.is_empty() {
          break;
      }
    }
    Ok(self.alloc_expr(span, CharacterClass(CharacterClassExpr::new(intervals))))
  }

  fn parse_char_range<'b>(&mut self, span: Span, ranges: &mut Peekable<Chars<'b>>, rule_name: &str) -> Result<Vec<CharacterInterval>> {
    let mut res = vec![];
    let separator_err = format!(
      "Unexpected separator `-`. Put it in the start or the end if you want \
      to accept it as a character in the set. Otherwise, you should only use it for \
      character intervals as in `[\"a-z\"]` (in rule {}).",
      rule_name);
    let lo = ranges.next();
    // Twisted logic due to the fact that `peek` borrows the ranges...
    let lo = {
      let next = ranges.peek();
      match (lo, next) {
        (Some('-'), Some(_)) => {
          return Err(Error::new(span, separator_err.as_str()));
        }
        (Some(lo), Some(&sep)) if sep == '-' => {
          lo
        }
        (Some(lo), _) => {
          res.push(CharacterInterval::new(lo, lo)); // If lo == '-', it ends the class, allowed.
          return Ok(res);
        }
        (None, _) => return Ok(res),
      }
    };
    ranges.next();
    match ranges.next() {
      Some('-') => { return Err(Error::new(span, separator_err.as_str())); }
      Some(hi) => {
        res.push(CharacterInterval::new(lo, hi));
      }
      None => {
        res.push(CharacterInterval::new(lo, lo));
        res.push(CharacterInterval::new('-', '-'));
      }
    };
    Ok(res)
  }
}
