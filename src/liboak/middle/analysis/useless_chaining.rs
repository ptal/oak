#![macro_use]
use middle::analysis::ast::*;
use ast::Expression::*;
pub use rust::NO_EXPANSION;

enum Predicate {
    And(usize,usize),
    Not(usize,usize),
    Oom(usize,usize),
    Zom(usize,usize),
    Nothing
}

pub struct UselessChaining<'a: 'c, 'b: 'a, 'c>
{
  grammar: &'c AGrammar<'a, 'b>,
  pred: Predicate,
  vec_pred : Vec<Predicate>,
  non_terminal : bool
}

impl <'a, 'b, 'c> UselessChaining<'a, 'b, 'c>
{
  pub fn analyse(grammar: AGrammar<'a, 'b>) -> Partial<AGrammar<'a, 'b>> {
    UselessChaining::check_chaining(&grammar);
    Partial::Nothing
  }

  fn check_chaining(grammar: &'c AGrammar<'a, 'b>){
    let mut analyser = UselessChaining{
      grammar: grammar,
      pred: Predicate::Nothing,
      vec_pred: vec![],
      non_terminal : false
  };
    for rule in &grammar.rules {
      println!("\nRegle");
      analyser.visit_expr(rule.expr_idx);
      analyser.non_terminal = false;
    }
  }

  fn verify_multiple(&mut self){
      if self.vec_pred.len()>=2 {
          match self.vec_pred.remove(0) {
              Predicate::And(first_and,first_and_child) =>{
                  let mut lo=self.grammar[first_and].span().lo();
                  if self.non_terminal {
                      lo = self.grammar[first_and_child].span().lo();
                  }
                  match self.vec_pred.pop().expect("Error: The vector of predicate is empty.") {
                      Predicate::And(last_and,last_and_child) => {
                          let mut hi = self.grammar[last_and_child].span().lo();
                          if self.non_terminal {
                              hi = self.grammar[last_and].span().lo();
                          }
                          self.grammar.cx.span_warn(
                              Span::new(lo,hi,NO_EXPANSION),
                              "Detected useless chaining: multiple & \n Help: &(&e) -> &e"
                          );
                      }
                      _ => println!("Error: found in vec_pred other predicate than And"),
                  }
              }
              Predicate::Oom(foom,_) =>{
                  let lo=self.grammar[foom].span().hi();
                  match self.vec_pred.pop().expect("Error: The vector of predicate is empty.") {
                      Predicate::Oom(_,loom) =>{
                          let hi = self.grammar[loom].span().hi();
                          self.grammar.cx.span_warn(
                              Span::new(lo,hi,NO_EXPANSION),
                              "Detected useless chaining: multiple + \n Help: (e+)+ -> e+"
                          );
                      }
                      _ => println!("Error: found in vec_pred other predicate than OneOrMore"),
                  }
              }
              _ => println!("Error: found in vec_pred other predicate than And and OneOrMore"),
          }

      }
      self.vec_pred.clear()
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
    unit_visitor_impl!(sequence);
    unit_visitor_impl!(choice);

  fn visit_expr(&mut self, this: usize) {
      match self.expr_by_index(this) {
        NonTerminalSymbol(rule) => {
          self.visit_non_terminal_symbol(this, rule)
        }
        ZeroOrMore(child) => {
          self.visit_zero_or_more(this, child)
        }
        OneOrMore(child) => {
          self.visit_one_or_more(this, child)
        }
        NotPredicate(child) => {
          self.visit_not_predicate(this, child)
        }
        AndPredicate(child) => {
          self.visit_and_predicate(this, child)
        }
        _ => {
            println!("literral");
            self.pred=Predicate::Nothing;
            self.verify_multiple();
        }
      }
  }

  fn visit_non_terminal_symbol(&mut self, _this: usize, rule: Ident){
    println!("non_terminal");
    self.non_terminal = true;
    self.visit_expr(self.grammar.find_rule_by_ident(rule).expr_idx)
  }

  fn visit_one_or_more(&mut self, this: usize, child: usize){
    println!("one_or_more");
    match self.pred {
        Predicate::Zom(t,_) => {
            self.grammar.cx.span_warn(
                Span::new(
                    self.grammar[child].span().hi(),
                    self.grammar[t].span().hi(),
                    NO_EXPANSION
                ),
                "Detected useless chaining: (e+)* \nHelp: (e+)* -> e+"
            );
        }
        _ => {}
    }
    self.pred=Predicate::Oom(this,child);
    if self.vec_pred.last().is_none() {
        self.vec_pred.push(Predicate::Oom(this,child));
    }
    else{
        match self.vec_pred.last().expect("Error: The vector of predicate is empty.") {
            &Predicate::Oom(_,_) => {
                self.vec_pred.push(Predicate::Oom(this,child));
            }
            _ => {
                self.verify_multiple();
            }
        }
    }
    self.visit_expr(child)
  }

  fn visit_zero_or_more(&mut self, this: usize, child: usize){
    println!("zero_or_more");
    self.pred = Predicate::Zom(this,child);
    self.verify_multiple();
    self.visit_expr(child)
  }

  fn visit_not_predicate(&mut self, this: usize, child: usize){
    println!("not_predicate");
    match self.pred {
        Predicate::Not(t,c) => {
            let mut lo = self.grammar[t].span().lo();
            let mut hi = self.grammar[child].span().lo();
            if self.non_terminal {
                lo = self.grammar[c].span().lo();
                hi = self.grammar[this].span().lo();
            }
            self.grammar.cx.span_warn(
                Span::new(
                    lo,
                    hi,
                    NO_EXPANSION
                ),
                "Detected useless chaining: !(!e) \nHelp: !(!e) -> &e"
            );
        }
        Predicate::And(t,c) => {
            let mut lo = self.grammar[t].span().lo();
            let mut hi = self.grammar[child].span().lo();
            if self.non_terminal {
                lo = self.grammar[c].span().lo();
                hi = self.grammar[this].span().lo();
            }
            self.grammar.cx.span_warn(
                Span::new(
                    lo,
                    hi,
                    NO_EXPANSION
                ),
                "Detected useless chaining: &(!e) \nHelp: &(!e) -> !e"
            );
        }
        _ => {}
    }
    self.pred=Predicate::Not(this,child);
    self.verify_multiple();
    self.visit_expr(child)
  }

  fn visit_and_predicate(&mut self, this: usize, child: usize){
    println!("and_predicate");
    match self.pred {
        Predicate::Not(t,c) => {
            let mut lo = self.grammar[t].span().lo();
            let mut hi = self.grammar[child].span().lo();
            if self.non_terminal {
                lo = self.grammar[c].span().lo();
                hi = self.grammar[this].span().lo();
            }
            self.grammar.cx.span_warn(
                Span::new(
                    lo,
                    hi,
                    NO_EXPANSION
                ),
                "Detected useless chaining: !(&e) \nHelp: !(&e) -> !e"
            );
        }
        _ => {}
    }
    self.pred=Predicate::And(this,child);

    if self.vec_pred.last().is_none() {
        self.vec_pred.push(Predicate::And(this,child));
    }
    else{
        match self.vec_pred.last().expect("Error: The vector of predicate is empty.") {
            &Predicate::And(_,_) => {
                self.vec_pred.push(Predicate::And(this,child));
            }
            _ => {
                self.verify_multiple();
            }
        }
    }
    self.visit_expr(child)
  }
}
