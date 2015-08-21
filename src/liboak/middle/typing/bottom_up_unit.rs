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

//! Bottom-up unit inference consists of propagating invisible unit types up in the expressions.
//!
//! The typing rules are of the form `expr:ty => expr':ty'` which means that if `expr` has type `ty` then `expr'` has type `ty'`:
//! * Basic combinators (`e*`, `e+`, `e?`):
//!    * `f(e:(^)) => f(e):(^)`
//!    * `f(e:t) => f(e):Identity`
//! * Syntactic predicates (`&e`, `!e`):
//!    * `f(e:t) => f(e):(^)`
//! * Semantics actions: `(e:t > g) => (e > g): Action`.
//! * Non terminal symbol (`R` being a function from rule identifier to type)
//!    * `ident:Identity => ident:(^)` if `R(ident) = (^)`.
//!    * `ident:Identity => ident:()` if `R(ident) = ()`.
//! * Sequence (symmetric cases not shown, easily generalizable for n-tuples):
//!    * `e:t e':t' => (e e'): (t, t')`
//!    * `(e e'): ((^), (^)) => (e e'): (^)`
//!    * `(e e'): (t, (^)) => (e e'): t`
//!    * `(e e'): (t, ()) => (e e'): t`
//! * Choice:
//!    * `e:t / e':t => (e / e'): t` if `t` is equal to `()` or `(^)`
//!    * `e:t / e':t' => (e / e'):Ìdentity` if `t=t'`
//!    * `e:(^) / e':() => (e / e'): ()`
//! * Explicit typing operator `->`:
//!    * `e:t -> () => e:()`
//!    * `e:t -> (^) => e:(^)`

//! One of the difficulty for implementing this is to deal with the recursion introduced by the typing rule of non-terminal symbol with the `R` function. Untypable recursive types must not generate errors here because the rule might be called in a context where the AST does not need to be build. The recursive type analysis will be performed after the top-down unit propagation (see `typing::top_down_unit`).
//! The algorithm is divided in two steps, it first propagates unit types inside rules (`IntraRule`) and then between the rules (`InterRule`). The inter-rule propagation does not loop. We start from the root grammar rule, whenever we encounter an already visited node, it means that the expression is not typable and let the type of the non-terminal symbol to `Identity`. Of course, after the inter-rule propagation, `Identity`-type loop can arise but generating or not an error is decided by the recursive type analysis that uses the value context in addition.

use middle::typing::ast::*;
use middle::typing::ast::ExprTy::*;

pub fn bottom_up_unit_inference(grammar: &mut Grammar) {
  IntraRule::propagate(&grammar.rules);
  InterRule::propagate(&grammar.rules);
}

trait BottomUpAnalysis
{
  fn visit_rules(&mut self, rules: &HashMap<Ident, Rule>) {
    for rule in rules.values() {
      self.visit_rule(rule);
    }
  }

  fn visit_rule(&mut self, rule: &Rule) {
    self.visit_expr(&rule.def);
  }

  fn visit_expr(&mut self, expr: &Box<Expression>) {
    match &expr.node {
        &ZeroOrMore(ref sub)
      | &OneOrMore(ref sub)
      | &Optional(ref sub) => self.propagate_from_inner(expr, sub),
        &SemanticAction(ref sub, id) => self.visit_semantic_action(expr, sub, id),
        &NotPredicate(ref sub)
      | &AndPredicate(ref sub) => self.visit_syntactic_predicate(expr, sub),
      &NonTerminalSymbol(id) => self.visit_non_terminal(expr, id),
      &Sequence(ref subs) => self.visit_sequence(expr, subs),
      &Choice(ref subs) => self.visit_choice(expr, subs),
      _ => ()
    }
  }

  fn propagate_from_inner(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) {
    self.visit_expr(expr);
    if expr.is_invisible() {
      parent.to_invisible_type();
    }
  }

  fn visit_non_terminal(&mut self, _parent: &Box<Expression>, _ident: Ident) {}

  fn visit_semantic_action(&mut self, _parent: &Box<Expression>,expr: &Box<Expression>, _ident: Ident) {
    self.visit_expr(expr);
  }

  fn visit_syntactic_predicate(&mut self, _parent: &Box<Expression>, expr: &Box<Expression>) {
    self.visit_expr(expr);
  }

  fn all_invisible(&self, exprs: &Vec<Box<Expression>>) -> bool {
    exprs.iter().all(|expr| expr.is_invisible())
  }

  fn all_unit(&self, exprs: &Vec<Box<Expression>>) -> bool {
    exprs.iter().all(|expr| expr.is_unit())
  }

  fn propagate_invisibility(&self, parent: &Box<Expression>, exprs: &Vec<Box<Expression>>) -> bool {
    let all_invisible = self.all_invisible(exprs);
    if all_invisible {
      parent.to_invisible_type();
    }
    all_invisible
  }

  fn propagate_unit(&self, parent: &Box<Expression>, exprs: &Vec<Box<Expression>>) -> bool {
    let all_unit = self.all_unit(&exprs);
    if all_unit {
      parent.to_unit_type();
    }
    all_unit
  }

  fn visit_choice(&mut self, parent: &Box<Expression>, exprs: &Vec<Box<Expression>>) {
    self.visit_exprs(exprs);
    if !self.propagate_invisibility(parent, &exprs)
    {
      self.propagate_unit(parent, &exprs);
    }
  }

  fn visit_sequence(&mut self, parent: &Box<Expression>, exprs: &Vec<Box<Expression>>) {
    self.visit_exprs(exprs);
    let parent_ty = parent.ty_clone();
    if let Tuple(inners) = parent_ty {
      if !self.propagate_invisibility(parent, exprs) {
        // Remove unit types from the tuple.
        let inners: Vec<usize> = inners.into_iter()
          .filter(|&idx| !exprs[idx].is_unit())
          .collect();

        if inners.is_empty() {
          parent.to_unit_type();
        }
        else {
          parent.to_tuple_type(inners);
        }
      }
    }
  }

  fn visit_exprs(&mut self, exprs: &Vec<Box<Expression>>) {
    for expr in exprs {
      self.visit_expr(expr);
    }
  }
}

struct IntraRule;

impl IntraRule
{
  pub fn propagate(rules: &HashMap<Ident, Rule>) {
    IntraRule.visit_rules(rules);
  }
}

impl BottomUpAnalysis for IntraRule {}

struct InterRule<'a>
{
  rules: &'a HashMap<Ident, Rule>,
  visited: HashMap<Ident, bool>
}

impl<'a> InterRule<'a>
{
  pub fn propagate(rules: &'a HashMap<Ident, Rule>) {
    let mut visited = HashMap::with_capacity(rules.len());
    for id in rules.keys() {
      visited.insert(*id, false);
    }
    let mut propagator = InterRule {
      rules: rules,
      visited: visited
    };
    propagator.visit_rules(rules);
  }

  fn visit_rule(&mut self, rule: &Rule) {
    let ident = &rule.name.node;
    if !*self.visited.get(ident).unwrap() {
      *self.visited.get_mut(ident).unwrap() = true;
      self.visit_expr(&rule.def);
    }
  }
}

impl<'a> BottomUpAnalysis for InterRule<'a>
{
  fn visit_non_terminal(&mut self, parent: &Box<Expression>, id: Ident) {
    let rule = self.rules.get(&id).unwrap();
    self.visit_rule(rule);
    if rule.def.is_invisible() {
      parent.to_invisible_type();
    } else if rule.def.is_unit() {
      parent.to_unit_type();
    }
  }

  fn visit_syntactic_predicate(&mut self, _parent: &Box<Expression>, _expr: &Box<Expression>)
  {}
}
