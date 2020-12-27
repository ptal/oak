// Copyright 2018 Chao Lin & William Sergeant (Sorbonne University)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![macro_use]
use middle::analysis::ast::*;
use ast::Expression::*;
use std::cmp;
use syntax::codemap::BytePos;
pub use rust::NO_EXPANSION;

enum Predicate {
    And(usize),
    Not(usize),
    Oom(usize),
    Zom(usize),
    Nothing
}

enum VecType {
    And,
    OneOrMore,
    Nothing
}

pub struct UselessChaining<'a: 'c, 'b: 'a, 'c>
{
  grammar: &'c AGrammar<'a, 'b>,
  pred: Predicate,
  vec_pred : Vec<Predicate>,
  vec_type : VecType
}

impl <'a, 'b, 'c> UselessChaining<'a, 'b, 'c>
{
  pub fn analyse(grammar: AGrammar<'a, 'b>) -> Partial<AGrammar<'a, 'b>> {
    UselessChaining::check_chaining(&grammar);
    Partial::Value(grammar)
  }

  fn check_chaining(grammar: &'c AGrammar<'a, 'b>){
    let mut analyser = UselessChaining{
      grammar: grammar,
      pred: Predicate::Nothing,
      vec_pred: vec![],
      vec_type: VecType::Nothing
    };
    for rule in &grammar.rules {
      analyser.visit_expr(rule.expr_idx)
    }
  }

  fn span_end_point(&self, sp: Span) -> Span{
      let lo = cmp::max(sp.hi().0 - 1, sp.lo().0);
      sp.with_lo(BytePos(lo))
  }

  fn get_th(&self, n: usize) -> &'static str {
      match n {
          1 => "st",
          2 => "nd",
          3 => "rd",
          _ => "th"
      }
  }

  fn warn_useless_chaining(&self, pred: (&'static str, &'static str, &'static str, &'static str), span2: Span, span1: Span){
      let (detected,help,first,second) = pred;
      self.grammar.span_warn(
          span1,
          format!(
              "Detected useless chaining: {}
              \nHelp: {}
              \n1{} predicate {}"
          ,detected,help,self.get_th(1),first)
      );
      self.grammar.span_note(
          span2,
          format!("2{} predicate {}",self.get_th(2),second)
      )
  }

  fn warn_vec_useless_chaining(&self, pred: (&'static str, &'static str, &'static str), i: usize, span: Span){
      let (warn,note,span_note) = pred;
      if i==0 {
          self.grammar.span_warn(
              span,
              format!(
                  "Detected useless chaining: multiple {}
                  \nHelp: {}
                  \n1{} occurence of {}"
              ,warn,note,self.get_th(1),warn)
          )
      }
      else{
          self.grammar.span_note(
              span,
              format!("{}{} occurence of {}",i+1,self.get_th(i+1),span_note)
          )
      }

  }

  fn warn_verify_multiple(&self) {
      for (i, x) in self.vec_pred.iter().enumerate() {
          match x {
              &Predicate::And(this) => {
                  let lo = self.grammar[this].span().lo();
                  let span = Span::new(lo,lo,NO_EXPANSION);
                  self.warn_vec_useless_chaining(("&","&(&e) -> &e","and"),i,span);
              }
              &Predicate::Oom(this) => {
                  let sp = self.grammar[this].span();
                  let span = self.span_end_point(sp);
                  self.warn_vec_useless_chaining(("+","(e+)+ -> e+","one or more"),i,span);
              }
              _ => unreachable!()
          }
      }
  }

  fn verify_multiple(&mut self){
      if self.vec_pred.len()>=2 {
          self.warn_verify_multiple()
      }
      self.vec_pred.clear();
      self.vec_type=VecType::Nothing;
  }

  fn warning(&mut self, current: Predicate){
      match (&self.pred, current) {
          (&Predicate::Zom(t),Predicate::Oom(this)) => {
              self.warn_useless_chaining(
                  ("(e+)*","(e+)* -> e+","One or more","Zero or more"),
                  self.span_end_point(self.grammar[this].span()),
                  self.span_end_point(self.grammar[t].span())
              );
          }
          (&Predicate::Not(t),Predicate::Not(this)) => {
              let f_lo = self.grammar[this].span().lo();
              let s_lo = self.grammar[t].span().lo();
              self.warn_useless_chaining(
                  ("!(!e)","!(!e) -> &e","not","not"),
                  Span::new(f_lo, f_lo, NO_EXPANSION),
                  Span::new(s_lo, s_lo, NO_EXPANSION)
              );
          }
          (&Predicate::And(t),Predicate::Not(this)) => {
              let f_lo = self.grammar[this].span().lo();
              let s_lo = self.grammar[t].span().lo();
              self.warn_useless_chaining(
                  ("&(!e)","&(!e) -> !e","not","and"),
                  Span::new(f_lo, f_lo, NO_EXPANSION),
                  Span::new(s_lo, s_lo, NO_EXPANSION)
              );
          }
          (&Predicate::Not(t),Predicate::And(this)) => {
              let f_lo = self.grammar[this].span().lo();
              let s_lo = self.grammar[t].span().lo();
              self.warn_useless_chaining(
                  ("!(&e)","!(&e) -> !e","and","not"),
                  Span::new(f_lo, f_lo, NO_EXPANSION),
                  Span::new(s_lo, s_lo, NO_EXPANSION)
              );
          }
          _ => {}
      }
  }

 }

impl<'a, 'b, 'c> ExprByIndex for UselessChaining<'a, 'b, 'c>
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.grammar.expr_by_index(index).clone()
  }
}

impl<'a, 'b, 'c> Visitor<()> for UselessChaining<'a, 'b, 'c>
{

  fn visit_str_literal(&mut self, _this: usize, _lit: String){
      self.pred=Predicate::Nothing;
      self.verify_multiple();
  }

  fn visit_atom(&mut self, _this: usize){
      self.pred=Predicate::Nothing;
      self.verify_multiple();
  }

  fn visit_non_terminal_symbol(&mut self, _this: usize, rule: Ident){
      let predicate = self.grammar.find_rule_by_ident(rule).expr_idx;
      match self.expr_by_index(predicate){
          ZeroOrMore(_child) => {
            self.warning(Predicate::Zom(predicate));
          }
          OneOrMore(_child) => {
            self.warning(Predicate::Oom(predicate));
            match self.vec_type {
                VecType::OneOrMore => {
                    self.vec_pred.push(Predicate::Oom(predicate));
                }
                _ => ()
            }
          }
          NotPredicate(_child) => {
            self.warning(Predicate::Not(predicate));
          }
          AndPredicate(_child) => {
            self.warning(Predicate::And(predicate));
            match self.vec_type {
                VecType::And => {
                    self.vec_pred.push(Predicate::And(predicate));
                }
                _ => ()
            }
          }
          _ => {}
      }
      self.pred=Predicate::Nothing;
      self.verify_multiple();
  }

  fn visit_one_or_more(&mut self, this: usize, child: usize){
    // println!("one_or_more");
    self.warning(Predicate::Oom(this));
    match self.vec_type {
        VecType::And => {
            self.verify_multiple()
        }
        VecType::Nothing => {
            self.vec_type=VecType::OneOrMore
        }
        _ => ()
    }
    self.vec_pred.push(Predicate::Oom(this));
    self.visit_expr(child)
  }

  fn visit_zero_or_more(&mut self, this: usize, child: usize){
    // println!("zero_or_more");
    self.pred = Predicate::Zom(this);
    self.verify_multiple();
    self.visit_expr(child)
  }

  fn visit_not_predicate(&mut self, this: usize, child: usize){
    // println!("not_predicate");
    self.warning(Predicate::Not(this));
    self.pred=Predicate::Not(this);
    self.verify_multiple();
    self.visit_expr(child)
  }

  fn visit_and_predicate(&mut self, this: usize, child: usize){
    // println!("and_predicate");
    self.warning(Predicate::And(this));
    self.pred=Predicate::And(this);

    match self.vec_type {
        VecType::OneOrMore => {
            self.verify_multiple()
        }
        VecType::Nothing => {
            self.vec_type=VecType::And;
        }
        _ => ()
    }
    self.vec_pred.push(Predicate::And(this));
    self.visit_expr(child)
  }

  fn visit_sequence(&mut self, _: usize, children: Vec<usize>){
    for child in children {
        self.pred=Predicate::Nothing;
        self.verify_multiple();
        self.visit_expr(child);
    }
  }

  fn visit_choice(&mut self, _: usize, children: Vec<usize>){
      for child in children {
          self.pred=Predicate::Nothing;
          self.verify_multiple();
          self.visit_expr(child);
      }
  }
}
