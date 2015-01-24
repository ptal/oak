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

use middle::typing::visitor::*;
use middle::typing::ast::ExprTy::*;

// The UnitPropagate nodes (expressed with P in the following rules)
// are propagated following these rules:
//  * Vector(P) -> P
//  * OptionalTy(P) -> P
//  * Tuple([e,..,e']) -> P (with all e=P) // Tested before any other propagation, if there is a (), it doesn't propagate.
//  * Tuple([e, P, e']) -> Tuple([e, e'])
//  * Tuple([e, (), e']) -> Tuple([e, e'])
//  * Sum([e,...,e']) -> P (with all e=P)
// Semantic actions stop propagation.

// There are automatic propagation rules:
//  * Tuple([e]) -> e
//  * Tuple([]) -> ()

// The parser already lifted up this expression:
//  * Sum([e]) -> e

pub fn propagation_phase(grammar: &mut Grammar)
{
  InterRulePropagation::propagate(&grammar.rules);
  IntraRulePropagation::propagate(&grammar.rules);
  UnitPropagateCleaner::clean(&grammar.rules);
}

trait Propagator
{
  fn visit_expr(&mut self, expr: &Box<Expression>) -> ExprTy;

  fn propagate_from_inner(&mut self, parent: &Box<Expression>, expr: &Box<Expression>)
  {
    if self.visit_expr(expr) == UnitPropagate {
      *parent.ty.borrow_mut() = UnitPropagate;
    }
  }

  fn visit_choice(&mut self, parent: &Box<Expression>, exprs: &Vec<Box<Expression>>)
  {
    let sub_tys = self.visit_exprs(exprs);
    if sub_tys.iter().all(|ty| ty == UnitPropagate) {
      *parent.ty.borrow_mut() = UnitPropagate;
    }
  }

  fn visit_sequence(&mut self, parent: &Box<Expression>, exprs: &Vec<Box<Expression>>)
  {
    let sub_tys = self.visit_exprs(exprs);
    if let Tuple(inners) = parent.ty.borrow().clone() {
      // If all children are UnitPropagate, we propagate too.
      if inners.iter().all(|idx| sub_tys[idx] == UnitPropagate) {
        *parent.ty.borrow_mut() = UnitPropagate;
      } else {
        // Remove Unit and UnitPropagate.
        let mut inners: Vec<usize> = inners.into_iter()
          .filter(|idx| sub_tys[idx].is_unit())
          .collect();

        *parent.ty.borrow_mut() =
          if inners.is_empty() {
            Unit
          } else if inners.len() == 1 {
            sub_tys[inners[0]]
          } else {
            Tuple(inners)
          };
      }
    }
  }

  fn visit_exprs(&mut self, exprs: &Vec<Box<Expression>>) -> Vec<ExprTy>
  {
    exprs.iter().map(|e| self.visit_expr(e)).collect()
  }
}

// No risk of loop because we stop at leaf types and
// the inline loop analysis ensures there is no recursive
// types.
struct InterRulePropagation<'a>
{
  rules: &'a HashMap<Ident, Rule>,
  visited: HashMap<Ident, bool>
}

impl<'a> InterRulePropagation<'a>
{
  pub fn propagate(rules: &'a HashMap<Ident, Rule>)
  {
    let mut visited = HashMap::with_capacity(rules.len());
    for id in rules.keys() {
      visited.insert(id.clone(), false);
    }
    let mut propagator = InterRulePropagation {
      rules: rules,
      visited: visited
    };
    propagator.visit_rules();
  }

  fn visit_rules(&mut self)
  {
    for rule in self.rules.values() {
      self.visit_rule(rule);
    }
  }

  fn visit_rule(&mut self, rule: &Rule) -> ExprTy {
    let ident = &rule.node.ident;
    if !*self.visited.get(ident).unwrap() {
      *self.visited.get_mut(ident).unwrap() = true;
      self.visit_expr(&rule.def);
    }
    rule.def.ty.borrow().clone()
  }

  fn visit_non_terminal(&mut self, parent: &Box<Expression>, id: Ident)
  {
    if self.visit_rule(self.rules.get(&id).unwrap()) == UnitPropagate {
      *parent.ty.borrow_mut() = UnitPropagate;
    }
  }
}

impl<'a> Propagator for InterRulePropagation<'a>
{
  fn visit_expr(&mut self, expr: &Box<Expression>) -> ExprTy
  {
    if !expr.ty.borrow().is_leaf() {
      match &expr.node {
        &NonTerminalSymbol(id) => self.visit_non_terminal(expr, id),
        &Sequence(ref subs) => self.visit_sequence(expr, subs),
        &Choice(ref subs) => self.visit_choice(expr, subs),
          &ZeroOrMore(ref sub)
        | &OneOrMore(ref sub)
        | &Optional(ref sub) => self.propagate_from_inner(expr, sub),
        _ => ()
      }
    }
    expr.ty.borrow().clone()
  }
}

struct IntraRulePropagation<'a>
{
  rules: &'a HashMap<Ident, Rule>
}

impl<'a> IntraRulePropagation<'a>
{
  pub fn propagate(rules: &'a HashMap<Ident, Rule>)
  {
    let mut propagator = IntraRulePropagation {
      rules: rules
    };
    propagator.visit_rules();
  }

  fn visit_rules(&mut self)
  {
    for rule in self.rules.values() {
      self.visit_expr(&rule.def);
    }
  }
}

impl<'a> Propagator for IntraRulePropagation<'a>
{
  fn visit_expr(&mut self, expr: &Box<Expression>) -> ExprTy
  {
    match &expr.node {
      &NonTerminalSymbol(id) => (),
      &Sequence(ref subs) => self.visit_sequence(expr, subs),
      &Choice(ref subs) => self.visit_choice(expr, subs),
        &ZeroOrMore(ref sub)
      | &OneOrMore(ref sub)
      | &Optional(ref sub) => self.propagate_from_inner(expr, sub),
        &NotPredicate(ref sub)
      | &AndPredicate(ref sub)
      | &SemanticAction(ref sub, _) => self.visit_expr(sub),
      _ => ()
    }
    expr.ty.borrow().clone()
  }
}

// Remove UnitPropagate nodes.
struct UnitPropagateCleaner<'a>
{
  rules: &'a HashMap<Ident, Rule>
}

impl<'a> UnitPropagateCleaner<'a>
{
  pub fn clean(rules: &'a HashMap<Ident, Rule>)
  {
    let mut cleaner = UnitPropagateCleaner {
      rules: rules
    };
    cleaner.clean_rules();
  }

  fn clean_rules(&mut self)
  {
    for rule in self.rules.values() {
      self.visit_rule(rule);
    }
  }
}

impl<'a> Visitor for UnitPropagateCleaner<'a>
{
  fn visit_expr(&mut self, expr: &Box<Expression>)
  {
    if expr.ty.borrow() == UnitPropagate {
      *expr.ty.borrow_mut() = Unit;
    }
  }
}
