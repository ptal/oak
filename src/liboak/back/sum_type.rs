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

//! Sum type analysis ensures that branches of a sum combinator have the same type. The comparison is purely syntactic and not semantic but it should be enough for most purposes.

use back::ast::*;
use monad::partial::Partial;
use rust;

/// Precondition: Expects that the recursive analysis has been done.
pub fn sum_type_analysis(cx: &ExtCtxt, grammar: Grammar)
  -> Partial<Grammar>
{
  if SumType::analyse(cx, &grammar.rules) {
    Partial::Value(grammar)
  }
  else {
    Partial::Nothing
  }
}

pub struct SumType<'a>
{
  cx: &'a ExtCtxt<'a>,
  rules: &'a HashMap<Ident, Rule>,
  bad_type_detected: bool
}

impl<'a> SumType<'a>
{
  fn analyse(cx: &'a ExtCtxt, rules: &'a HashMap<Ident, Rule>) -> bool {
    let mut sum_type = SumType::new(cx, rules);
    sum_type.visit_rules();
    !sum_type.bad_type_detected
  }

  fn new(cx: &'a ExtCtxt, rules: &'a HashMap<Ident, Rule>) -> SumType<'a> {
    SumType {
      cx: cx,
      rules: rules,
      bad_type_detected: false
    }
  }

  fn visit_rules(&mut self) {
    for rule in self.rules.values() {
      self.visit_rule(rule);
    }
  }

  fn visit_rule(&mut self, rule: &Rule) {
    self.visit_expr(&rule.def);
  }

  /// Types are compared with their string description, so it is not a semantic comparison but a syntactic one. It should be sufficient for now but see issue #73 for more explanations.
  fn map_types_to_indices(&self, exprs: &Vec<Box<Expression>>) -> HashMap<String, Vec<usize>> {
    let mut tys_indices: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, expr) in exprs.iter().enumerate() {
      let mut updated = false;
      let rust_ty = expr.return_type(self.cx);
      let rust_ty_desc = rust::ty_to_string(&*rust_ty);
      if let Some(indices) = tys_indices.get_mut(&rust_ty_desc) {
        indices.push(idx);
        updated = true;
      }
      if !updated {
        tys_indices.insert(rust_ty_desc, vec![idx]);
      }
    }
    tys_indices
  }

  fn sum_type_error(&mut self, parent: &Box<Expression>, exprs: &Vec<Box<Expression>>,
    tys_indices: HashMap<String, Vec<usize>>)
  {
    self.bad_type_detected = true;
    self.cx.span_err(parent.span, "sum combinator arms have incompatible types:");
    for (ty_desc, indices) in tys_indices {
      for idx in indices {
        self.cx.span_note(exprs[idx].span, format!("has type {}", ty_desc).as_str());
      }
    }
  }
}

impl<'a> Visitor<Expression, ()> for SumType<'a>
{
  unit_visitor_impl!(Expression, str_literal);
  unit_visitor_impl!(Expression, character);
  unit_visitor_impl!(Expression, sequence);
  unit_visitor_impl!(Expression, non_terminal);

  fn visit_choice(&mut self, parent: &Box<Expression>, exprs: &Vec<Box<Expression>>) {
    if !parent.is_unit() {
      let tys_indices = self.map_types_to_indices(exprs);
      if tys_indices.len() > 1 {
        self.sum_type_error(parent, exprs, tys_indices);
      }
    }
    walk_exprs(self, exprs);
  }
}
