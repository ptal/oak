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
use middle::typing::ast::*;

// The UnitPropagate (expressed with P in the following rules) nodes 
// are propagated following these rules:
//  * Vector(P) -> P
//  * OptionalTy(P) -> P
//  * Tuple([P*]) -> P // Tested before any other propagation, if there is a (), it doesn't propagate.
//  * Tuple([e, P, e']) -> Tuple([e, e'])
//  * Tuple([e, (), e']) -> Tuple([e, e'])
//  * UnnamedSum([e, P, e']) -> UnnamedSum([e, (), e'])
// The sum stops the propagation (as well as rules).

// There are automatic propagation rules:
//  * Tuple([e]) -> e
//  * Tuple([]) -> ()

// The parser already lifted up this expression:
//  * UnnamedSum([e]) -> e

// There is no propagation loops since it stops propagating at rule level.

pub fn propagation_phase(cx: &ExtCtxt, grammar: &mut Grammar)
{
  Propagator::propagate(cx, &grammar.rules);
  PropagatorCleaner::clean(&grammar.rules);
}

struct Propagator<'a>
{
  cx: &'a ExtCtxt<'a>,
  rules: &'a HashMap<Ident, Rule>
}

impl<'a> Propagator<'a>
{
  pub fn propagate(cx: &'a ExtCtxt, rules: &'a HashMap<Ident, Rule>)
  {
    let mut propagator = Propagator {
      cx: cx,
      rules: rules
    };
    propagator.propagate_rules();
  }

  fn propagate_rules(&mut self)
  {
    for (ident, rule) in self.rules.iter() {
      self.visit_rule(rule);
    }
  }

  fn check_and_propagate(&mut self, parent: &PTy, inner: &PTy)
  {
    walk_ty(self, inner);
    if inner.borrow().must_propagate() {
      *parent.borrow_mut() = Rc::new(UnitPropagate);
    }
  }
}

// Post-order traversal of the tree, we propagate the children to know
// if the current type needs to be lifted up.
impl<'a> Visitor for Propagator<'a>
{
  fn visit_vector(&mut self, parent: &PTy, inner: &PTy)
  {
    self.check_and_propagate(parent, inner);
  }

  fn visit_optional(&mut self, parent: &PTy, inner: &PTy)
  {
    self.check_and_propagate(parent, inner);
  }

  fn visit_tuple(&mut self, parent: &PTy, inners: &Vec<PTy>)
  {
    walk_tys(self, inners);
    // If all children are UnitPropagate, we propagate too.
    if inners.iter().all(|ty| ty.borrow().must_propagate()) {
      *parent.borrow_mut() = Rc::new(UnitPropagate);
    } else {
      // Remove Unit and UnitPropagate.
      let mut inners: Vec<PTy> = inners.iter()
        .filter(|ty| !ty.borrow().is_unit())
        .map(|ty| RefCell::new(ty.borrow().clone()))
        .collect();

      *parent.borrow_mut() = 
        if inners.is_empty() {
          Rc::new(Unit)
        } else if inners.len() == 1 {
          inners.pop().unwrap().borrow().clone()
        } else {
          Rc::new(Tuple(inners))
        };
    }
  }

  fn visit_unnamed_sum(&mut self, _parent: &PTy, inners: &Vec<PTy>)
  {
    assert!(inners.len() > 0, "PEG compiler bug: A sum type with only one branch \
      has been detected during propagation (it should be lifted up during parsing).");
    walk_tys(self, inners);
  }


  fn visit_rule_type_ph(&mut self, _parent: &PTy, _ident: Ident)
  {
    assert!(false, "PEG compiler bug: Hit a type placeholder node while propagating, \
      they should all be removed during the inlining phase.");
  }
}

// Remove UnitPropagate nodes.
struct PropagatorCleaner<'a>
{
  rules: &'a HashMap<Ident, Rule>
}

impl<'a> PropagatorCleaner<'a>
{
  pub fn clean(rules: &'a HashMap<Ident, Rule>)
  {
    let mut cleaner = PropagatorCleaner {
      rules: rules
    };
    cleaner.clean_rules();
  }

  fn clean_rules(&mut self)
  {
    for (ident, rule) in self.rules.iter() {
      self.visit_rule(rule);
    }
  }
}

impl<'a> Visitor for PropagatorCleaner<'a>
{
  fn visit_unit_propagate(&mut self, parent: &PTy)
  {
    *parent.borrow_mut() = Rc::new(Unit);
  }
}
