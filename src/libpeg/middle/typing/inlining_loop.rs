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

pub struct InliningLoop<'a>
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
    inlining_loop.visit_rule(rules.get(&start_rule).unwrap());
    inlining_loop.cycle_detected
  }

  fn new(cx: &'a ExtCtxt, rules: &'a HashMap<Ident, Rule>) -> InliningLoop<'a>
  {
    let mut visited = HashMap::with_capacity(rules.len());
    for id in rules.keys() {
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
      if *id == in_cycle {
        break;
      }
      trimmed_cycle.push(id.clone());
    }
    self.cx.span_err(self.rules.get(&in_cycle).unwrap().name.span, "Inlining cycle detected. \
      The type of a rule must be inlined into itself (indirectly or not), which is impossible.");
    self.cx.span_note(self.rules.get(&in_cycle).unwrap().name.span, "Recursive data type are not handled automatically, \
      you must create it yourself with a semantic action and a function. If you don't care about the value of this rule,
      annotate it with #[invisible_type].");
    for cycle_node in trimmed_cycle.iter().rev() {
      self.cx.span_note(self.rules.get(cycle_node).unwrap().name.span, "This rule is in the inlining loop.");
    }
  }
}

// We don't visit the expression but instead only its type.
impl<'a> Visitor for InliningLoop<'a>
{
  // On the rule vertex.
  fn visit_rule(&mut self, rule: &Rule)
  {
    let ident = rule.name.node.clone();
    *self.visited.get_mut(&ident).unwrap() = true;
    if rule.is_inline() {
      self.current_inline_path.push(ident);
      walk_rule_ty(self, rule);
      self.current_inline_path.pop();
    } else {
      let current_inline_path = self.current_inline_path.clone();
      self.current_inline_path.clear();
      walk_rule_ty(self, rule);
      self.current_inline_path = current_inline_path;
    }
  }

  // On the (inline) edge.
  fn visit_rule_type_ph(&mut self, _ty: &PTy, ident: Ident)
  {
    if !self.cycle_detected {
      let rule = self.rules.get(&ident).unwrap();
      let ident = rule.name.node.clone();
      if rule.is_inline() && self.current_inline_path.contains(&ident) {
        self.current_inline_path.push(ident);
        self.loop_detected();
      }
      else if !*self.visited.get(&ident).unwrap() {
        self.visit_rule(rule);
      }
    }
  }

  // Sum type breaks the potential cycles since it cannot be unnamed.
  // Semantic action also break them because the action is already typed by the user.
  // character, unit and unit_propagate don't generate loops (trivial cases).

  fn visit_vector(&mut self, _parent: &PTy, inner: &PTy)
  {
    walk_ty(self, inner);
  }

  fn visit_tuple(&mut self, _parent: &PTy, inners: &Vec<PTy>)
  {
    walk_tys(self, inners);
  }

  fn visit_optional_ty(&mut self, _parent: &PTy, inner: &PTy)
  {
    walk_ty(self, inner);
  }
}

pub fn walk_rule_ty<V: Visitor>(visitor: &mut V, rule: &Rule)
{
  walk_ty(visitor, &rule.def.ty);
}

fn walk_tys<V: Visitor>(visitor: &mut V, tys: &Vec<PTy>)
{
  for ty in tys.iter() {
    walk_ty(visitor, ty);
  }
}
