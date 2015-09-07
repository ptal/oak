// Copyright 2015 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Bottom-up tuple inference replaces `Identity` type of non-terminal symbol with a tuple type. It can be a simple 1-tuple forwarding the sub-expression type but can also be a n-tuple. This is necessary to correctly unpack tuple values into semantic action call. This transformation always terminates since we checked for recursive type in the `typing::recursive_type` analysis.

use middle::typing::ast::*;
use monad::partial::Partial;

pub fn bottom_up_tuple_inference(grammar: Grammar)
  -> Partial<Grammar>
{
  BottomUpTupleInference::transform(&grammar.rules);
  Partial::Value(grammar)
}

pub struct BottomUpTupleInference<'a>
{
  rules: &'a HashMap<Ident, Rule>,
  visited: HashMap<Ident, bool>
}

impl<'a> BottomUpTupleInference<'a>
{
  fn transform(rules: &'a HashMap<Ident, Rule>) {
    let mut bottom_up_tuple = BottomUpTupleInference::new(rules);
    bottom_up_tuple.visit_rules();
  }

  fn new(rules: &'a HashMap<Ident, Rule>) -> BottomUpTupleInference<'a> {
    let mut visited = HashMap::with_capacity(rules.len());
    for id in rules.keys() {
      visited.insert(id.clone(), false);
    }
    BottomUpTupleInference {
      rules: rules,
      visited: visited
    }
  }

  fn visit_rules(&mut self) {
    for rule in self.rules.values() {
      self.visit_rule(rule);
    }
  }

  fn visit_rule(&mut self, rule: &Rule) {
    let ident = rule.name.node;
    if !self.visited[&ident] {
      *self.visited.get_mut(&ident).unwrap() = true;
      self.visit_expr(&rule.def);
    }
  }
}

impl<'a> Visitor<Expression, ()> for BottomUpTupleInference<'a>
{
  unit_visitor_impl!(Expression, str_literal);
  unit_visitor_impl!(Expression, character);
  unit_visitor_impl!(Expression, sequence);
  unit_visitor_impl!(Expression, choice);

  fn visit_non_terminal_symbol(&mut self, parent: &Box<Expression>, ident: Ident) {
    let rule = &self.rules[&ident];
    self.visit_rule(rule);
    let indexes = match rule.def.ty_clone() {
      ExprTy::Tuple(indexes) => indexes,
      _ => vec![0]
    };
    parent.to_tuple_type(indexes);
  }
}
