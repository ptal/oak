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

// The RuleTypePlaceholder(ident) are replaced following these rules:
//  * if rules[ident].inline --> rules[ident].type
//  * if rules[ident].invisible --> UnitPropagate
//  * if rules[ident].new --> RuleTypeName(ident)
// No loop can arise due to the InliningLoop analysis.

pub fn inlining_phase(cx: &ExtCtxt, grammar: &mut Grammar)
{
  let has_cycle = InliningLoop::analyse(cx, grammar.attributes.starting_rule.clone(), &grammar.rules);
  if !has_cycle {
    Inliner::inline(cx, &grammar.rules);
  }
}

struct Inliner<'a>
{
  cx: &'a ExtCtxt<'a>,
  rules: &'a HashMap<Ident, Rule>
}

impl<'a> Inliner<'a>
{
  pub fn inline(cx: &'a ExtCtxt, rules: &'a HashMap<Ident, Rule>)
  {
    let mut inliner = Inliner {
      cx: cx,
      rules: rules
    };
    inliner.inline_rules();
  }

  fn inline_rules(&mut self)
  {
    for (ident, rule) in self.rules.iter() {
      self.visit_rule(rule);
    }
  }
}

impl<'a> Visitor for Inliner<'a>
{
  fn visit_rule_type_ph(&mut self, ty: &PTy, ident: Ident)
  {
    let rule = self.rules.get(&ident);
    match &rule.attributes.ty.style {
      &New => *ty.borrow_mut() = Rc::new(RuleTypeName(ident)),
      &Inline(_) => {
        let this = self;
        *ty.borrow_mut() = this.rules.get(&ident).def.ty.borrow().clone();
      },
      &Invisible(_) => *ty.borrow_mut() = Rc::new(UnitPropagate)
    }
  }
}

struct InliningLoop<'a>
{
  cx: &'a ExtCtxt<'a>,
  rules: &'a HashMap<Ident, Rule>,
  visited: HashMap<Ident, bool>,
  current_inline_path: Vec<Ident>,
  cycle_detected: bool
}

impl<'a> InliningLoop<'a>
{
  pub fn analyse(cx: &'a ExtCtxt, start_rule: Ident, rules: &'a HashMap<Ident, Rule>) -> bool
  {
    let mut inlining_loop = InliningLoop::new(cx, rules);
    inlining_loop.visit_rule(rules.get(&start_rule));
    inlining_loop.cycle_detected
  }

  fn new(cx: &'a ExtCtxt, rules: &'a HashMap<Ident, Rule>) -> InliningLoop<'a>
  {
    let mut visited = HashMap::with_capacity(rules.len());
    for (id, rule) in rules.iter() {
      visited.insert(id.clone(), false);
    }
    InliningLoop {
      cx: cx,
      rules: rules,
      visited: visited,
      current_inline_path: vec![],
      cycle_detected: false
    }
  }

  fn loop_detected(&mut self)
  {
    self.cycle_detected = true;
    let in_cycle = self.current_inline_path.pop().unwrap();
    // Consider the smallest cycle.
    let mut trimmed_cycle = vec![in_cycle];
    for id in self.current_inline_path.iter().rev() {
      trimmed_cycle.push(id.clone());
      if *id == in_cycle {
        break;
      }
    }
    self.cx.span_err(self.rules.get(&in_cycle).name.span, "Inlining cycle detected. Indirectly (or not), \
      the type of a rule must be inlined into itself, which is impossible. Break the cycle by removing \
      one of the inlining annotations.");
    for cycle_node in trimmed_cycle.iter().rev() {
      self.cx.span_note(self.rules.get(cycle_node).name.span, "This rule is in the inlining loop.");
    }
  }
}

impl<'a> Visitor for InliningLoop<'a>
{
  // On the rule vertex.
  fn visit_rule(&mut self, rule: &Rule)
  {
    let ident = rule.name.node.clone();
    *self.visited.get_mut(&ident) = true;
    if rule.is_inline() {
      self.current_inline_path.push(ident);
      walk_rule(self, rule);
      self.current_inline_path.pop();
    } else {
      let current_inline_path = self.current_inline_path.clone();
      self.current_inline_path.clear();
      walk_rule(self, rule);
      self.current_inline_path = current_inline_path;
    }
  }

  // On the (inline) edge.
  fn visit_rule_type_ph(&mut self, _ty: &PTy, ident: Ident)
  {
    if !self.cycle_detected {
      let rule = self.rules.get(&ident);
      let ident = rule.name.node.clone();
      if rule.is_inline() && self.current_inline_path.contains(&ident) {
        self.current_inline_path.push(ident);
        self.loop_detected();
      } 
      else if !self.visited.get(&ident) {
        self.visit_rule(rule);
      }
    }
  }

  // Sum type breaks the potential cycles since it cannot be unnamed.
  fn visit_unnamed_sum(&mut self, _parent: &PTy, _inners: &Vec<PTy>)
  {}
}
