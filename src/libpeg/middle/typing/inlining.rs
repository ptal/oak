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

use middle::typing::ast::ExpressionType::*;
use middle::typing::inlining_loop::InliningLoop;

// The RuleTypePlaceholder(ident) are replaced following these rules:
//  * if rules[ident].inline --> rules[ident].type
//  * if rules[ident].invisible --> UnitPropagate
// No loop can arise thanks to the InliningLoop analysis.

pub fn inlining_phase(cx: &ExtCtxt, grammar: &mut Grammar)
{
  let has_cycle = InliningLoop::analyse(cx, grammar.attributes.starting_rule.clone(), &grammar.rules);
  if !has_cycle {
    Inliner::inline(&grammar.rules);
  }
}

struct Inliner<'a>
{
  rules: &'a HashMap<Ident, Rule>
}

impl<'a> Inliner<'a>
{
  pub fn inline(rules: &'a HashMap<Ident, Rule>)
  {
    let mut inliner = Inliner {
      rules: rules
    };
    inliner.inline_rules();
  }

  fn inline_rules(&mut self)
  {
    for rule in self.rules.values() {
      self.visit_rule(rule);
    }
  }
}

impl<'a> Visitor for Inliner<'a>
{
  fn visit_rule_type_ph(&mut self, parent: &PTy, ident: Ident)
  {
    let rule = self.rules.get(&ident).unwrap();
    match &rule.attributes.ty.style {
      &RuleTypeStyle::Inline => {
        let this = self;
        *parent.borrow_mut() = this.rules.get(&ident).unwrap().def.ty.borrow().clone();
      },
      &RuleTypeStyle::Invisible(_) => *parent.borrow_mut() = Rc::new(UnitPropagate)
    }
  }
}
