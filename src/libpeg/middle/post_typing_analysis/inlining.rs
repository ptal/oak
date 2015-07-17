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
use middle::post_typing_analysis::inlining_loop::InliningLoop;

// The RuleTypeOf(ident) are replaced following these rules:
//  * if rules[ident].inline ->
//    * if the type is a leaf (see is_leaf), it won't change so we take that type.
//    * otherwise we keep RuleTypeOf(ident)
//  * if rules[ident].invisible -> UnitPropagate
// No loop can arise due to the InliningLoop analysis.

pub fn inlining_phase(cx: &ExtCtxt, grammar: &mut Grammar)
{
  let start_rule = grammar.attributes.starting_rule.clone();
  let has_cycle = InliningLoop::analyse(cx, start_rule.clone(), &grammar.rules);
  if !has_cycle {
    Inliner::inline(start_rule, &grammar.rules);
  }
}

struct Inliner<'a>
{
  rules: &'a HashMap<Ident, Rule>,
  visited: HashMap<Ident, bool>,
}

impl<'a> Inliner<'a>
{
  pub fn inline(start_rule: Ident, rules: &'a HashMap<Ident, Rule>)
  {
    let mut visited = HashMap::with_capacity(rules.len());
    for id in rules.keys() {
      visited.insert(id.clone(), false);
    }
    let mut inliner = Inliner {
      rules: rules,
      visited: visited
    };
    inliner.visit_rule(rules.get(&start_rule).unwrap());
  }

  fn inline_ty(&self, rule: &Rule, expr: &Box<Expression>) {
    *expr.ty.borrow_mut() =
      match &rule.attributes.ty.style {
        &RuleTypeStyle::Inline => rule.def.deref_type(&self.rules),
        &RuleTypeStyle::Unit(_) => Unit,
        &RuleTypeStyle::Invisible(_) => UnitPropagate
      };
  }
}

impl<'a> Visitor for Inliner<'a>
{
  fn visit_rule(&mut self, rule: &Rule)
  {
    let ident = rule.name.node.clone();
    *self.visited.get_mut(&ident).unwrap() = true;
    walk_rule(self, rule);
  }

  fn visit_expr(&mut self, expr: &Box<Expression>)
  {
    if let NonTerminalSymbol(ident) = expr.node {
      let rule = self.rules.get(&ident).unwrap();
      if !*self.visited.get(&ident).unwrap() {
        self.visit_rule(rule);
      }
      (&*self).inline_ty(&rule, expr);
    }
    else {
      walk_expr(self, expr);
    }
  }
}
