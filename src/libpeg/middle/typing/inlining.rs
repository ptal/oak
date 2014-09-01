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

pub fn inlining_phase(cx: &ExtCtxt, grammar: &mut Grammar)
{
  let has_cycle = InliningLoop::analyse(cx, grammar.attributes.starting_rule.clone(), &grammar.rules);
  if !has_cycle {
    // inline_rules(&mut grammar.rules);
  }
}

// struct Inliner<'a>
// {
//   cx: &'a ExtCtxt<'a>,
//   rules: &'a HashMap<Ident, Rule>,
//   tys: HashMap<Ident, RuleType>
// }

// impl<'a> Inliner<'a>
// {
//   pub fn new(cx: &'a ExtCtxt, rules: &'a HashMap<Ident, Rule>) -> Inliner<'a>
//   {
//     Inliner {
//       cx: cx,
//       rules: rules,
//       tys: HashMap::with_capacity(rules.len())
//     }
//   }

//   fn inline_rules(&mut self)
//   {
//     let mut tys = HashMap::with_capacity(rules.len());
//     let rules = &self.rules;
//     for rule in rules.values() {
//       self.inline_rule(rule);
//     }
//   }

//   fn inline_rule(&mut self, rule: &Rule)
//   {
//     if !tys.contains(&rule.name.node) {
//       let inlined_ty = self.inline_rule_ty(&rule.ty);
//       tys.insert(rule.name.node.clone(), inlined_ty);
//     }
//   }

//   fn inline_rule_ty(&mut self, ty: &RuleType) -> RuleType
//   {
//     match ty {
//       &NewTy(ref ty) => self.inline_expr_ty(ty),
//       &InlineTy(ref ty) => InlineTy(ty.clone())
//     }
//   }

//   fn inline_expr_ty(&mut self, ty: &Box<ExpressionType>) -> Box<ExpressionType>
//   {
//     match **ty {
//       Character => box Character,
//       Unit => box Unit,
//       UnitPropagate => box UnitPropagate,
//       RuleTypePlaceholder(ref ident) => self.inline_placeholder(ident),
//       Vector(ref ty) => self.inline_vector(ty),
//       Tuple(ref tys) => self.inline_tuple(tys),
//       OptionalTy(ref ty) => self.inline_optional(ty)
//     }
//   }
// }

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
  fn visit_rule_type_ph(&mut self, ident: Ident)
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
  fn visit_unnamed_sum(&mut self, _tys: &Vec<PTy>)
  {}
}

// fn type_of_choice_expr(&self, exprs: &Vec<Box<Expression>>) -> Option<Box<ExpressionType>>
// {
//   fn flatten_tuple(ty: Box<ExpressionType>) -> Vec<Box<ExpressionType>>
//   {
//     match ty {
//       box Tuple(tys) => tys,
//       _ => vec![ty]
//     }
//   };

//   let ty = exprs.iter()
//     .map(|expr| self.type_of_expr(expr))
//     .map(|ty| ty.map_or(vec![], flatten_tuple))
//     .map(|tys| box SumBranch(tys))
//     .collect();

//   Some(box Sum(ty))
// }
