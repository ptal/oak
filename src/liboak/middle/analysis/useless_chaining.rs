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
      // println!("\nRegle");
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

  fn warn_useless_chaining(&self, pred: (&'static str, &'static str, &'static str, &'static str), span1: Span, span2: Span){
      let (detected,help,first,second) = pred;
      self.grammar.span_warn(
          span1,
          format!(
              "Detected useless chaining: {}
              \nHelp: {}
              \nFirst predicate {}"
          ,detected,help,first)
      );
      self.grammar.span_note(
          span2,
          format!("Second predicate {}", second)
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
                  \nFirst occurence of {}"
              ,warn,note,warn)
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
 }

impl<'a, 'b, 'c> ExprByIndex for UselessChaining<'a, 'b, 'c>
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.grammar.expr_by_index(index).clone()
  }
}

impl<'a, 'b, 'c> Visitor<()> for UselessChaining<'a, 'b, 'c>
{
    unit_visitor_impl!(str_literal);
    unit_visitor_impl!(atom);

  fn visit_expr(&mut self, this: usize) {
      match self.expr_by_index(this) {
        NonTerminalSymbol(rule) => {
          // println!("NonTerminalSymbol");
          self.visit_non_terminal_symbol(this, rule)
        }
        ZeroOrMore(child) => {
          // println!("ZeroOrMore");
          self.visit_zero_or_more(this, child)
        }
        OneOrMore(child) => {
          // println!("OneOrMore");
          self.visit_one_or_more(this, child)
        }
        NotPredicate(child) => {
          // println!("NotPredicate");
          self.visit_not_predicate(this, child)
        }
        AndPredicate(child) => {
          // println!("AndPredicate");
          self.visit_and_predicate(this, child)
        }
        Choice(choices) => {
          // println!("choice")
          self.visit_choice(this, choices)
        }
        Sequence(seq) => {
          // println!("StrLiteral: {}",strl)
          self.visit_sequence(this, seq)
        }
        _ => {
          self.pred=Predicate::Nothing;
          self.verify_multiple();
        }
      }
  }

  fn visit_non_terminal_symbol(&mut self, _this: usize, rule: Ident){
    // println!("non_terminal");
    self.visit_expr(self.grammar.find_rule_by_ident(rule).expr_idx)
  }

  fn visit_one_or_more(&mut self, this: usize, child: usize){
    // println!("one_or_more");
    match self.pred {
        Predicate::Zom(t) => {
            let first_span = self.grammar[this].span();
            let second_span = self.grammar[t].span();
            self.warn_useless_chaining(
                ("(e+)*","(e+)* -> e+","One or more","Zero or more"),
                self.span_end_point(first_span),
                self.span_end_point(second_span)
            );
        }
        _ => ()
    }
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
    match self.pred {
        Predicate::Not(t) => {
            let f_lo = self.grammar[this].span().lo();
            let s_lo = self.grammar[t].span().lo();
            let first_span = Span::new(f_lo, f_lo, NO_EXPANSION);
            let second_span = Span::new(s_lo, s_lo, NO_EXPANSION);
            self.warn_useless_chaining(
                ("!(!e)","!(!e) -> &e","not","not"),
                first_span,
                second_span
            );
        }
        Predicate::And(t) => {
            let f_lo = self.grammar[this].span().lo();
            let s_lo = self.grammar[t].span().lo();
            let first_span = Span::new(f_lo, f_lo, NO_EXPANSION);
            let second_span = Span::new(s_lo, s_lo, NO_EXPANSION);
            self.warn_useless_chaining(
                ("&(!e)","&(!e) -> !e","not","and"),
                first_span,
                second_span
            );
        }
        _ => {}
    }
    self.pred=Predicate::Not(this);
    self.verify_multiple();
    self.visit_expr(child)
  }

  fn visit_and_predicate(&mut self, this: usize, child: usize){
    // println!("and_predicate");
    match self.pred {
        Predicate::Not(t) => {
            let f_lo = self.grammar[this].span().lo();
            let s_lo = self.grammar[t].span().lo();
            let first_span = Span::new(f_lo, f_lo, NO_EXPANSION);
            let second_span = Span::new(s_lo, s_lo, NO_EXPANSION);
            self.warn_useless_chaining(
                ("!(&e)","!(&e) -> !e","and","not"),
                first_span,
                second_span
            );
        }
        _ => {}
    }
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
        self.verify_multiple();
        self.visit_expr(child);
    }
  }

  fn visit_choice(&mut self, _: usize, _: Vec<usize>){
      // for child in children {
      //     self.verify_multiple();
      //     self.visit_expr(child);
      // }
      self.verify_multiple();
  }
}
