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

use middle::typing::ast::*;
use middle::typing::ast::IType::*;
use middle::typing::type_rewriting::*;

pub struct Surface<'a, 'b: 'a>
{
  pub grammar: IGrammar<'a, 'b>,
  recursion_path: Vec<Ident>
}

impl<'a, 'b> Surface<'a, 'b>
{
  pub fn new(grammar: IGrammar<'a, 'b>) -> Surface<'a, 'b> {
    Surface {
      grammar: grammar,
      recursion_path: vec![]
    }
  }

  pub fn surface(&mut self, rules: Vec<Ident>) {
    for rule in rules {
      if self.grammar.type_of_rule(rule) == Infer {
        self.visit_rule(rule);
      }
    }
  }

  fn visit_rule(&mut self, rule: Ident) -> IType {
    let expr_idx = self.grammar.expr_index_of_rule(rule);
    if self.is_rec(rule) {
      self.infer_rec_type(rule)
    }
    else {
      self.recursion_path.push(rule);
      let ty = self.visit_expr(expr_idx);
      self.recursion_path.pop();
      let reduced_ty = TypeRewriting::reduce_rec(rule, ty);
      self.type_expr(expr_idx, reduced_ty)
    }
  }

  fn is_rec(&self, rule: Ident) -> bool {
    self.recursion_path.iter().any(|r| *r == rule)
  }

  pub fn type_of(&self, expr_idx: usize) -> IType {
    self.grammar[expr_idx].ty()
  }

  pub fn type_expr(&mut self, expr_idx: usize, ty: IType) -> IType {
    self.grammar[expr_idx].ty = ty.clone();
    ty
  }

  fn infer_rec_type(&mut self, entry_rule: Ident) -> IType {
    let rec_path = self.recursion_path.clone();
    let mut rec_rules = vec![entry_rule];
    rec_rules.extend(
      rec_path.into_iter()
        .rev()
        .take_while(|r| *r != entry_rule));
    Rec(rec_rules)
  }

  // fn warn_recursive_type(&mut self) {
  //   let in_cycle = self.current_inline_path.pop().unwrap();
  //   // Consider the smallest cycle which is garantee since we extract the element that closed the cycle.
  //   let mut errors = vec![(
  //     self.grammar.rules[&in_cycle].span(),
  //     format!("Inlining cycle detected. \
  //     The type of a rule must be inlined into itself (indirectly or not), which is impossible.")
  //   )];
  //   for cycle_node in trimmed_cycle.iter() {
  //     errors.push((
  //       self.grammar.rules[cycle_node].span(),
  //       format!("This rule is part of the recursive type.")));
  //   }
  //   errors.push((
  //     self.grammar.rules[&in_cycle].span(),
  //     format!("Recursive data types are not handled automatically, \
  //     you must create it yourself with a semantic action.\nIf you don't care about the value of this rule, \
  //     annotate it with `rule = e -> ()` or annotate leaf rules that produce values with `rule = e -> (^)`.")));
  //   self.grammar.multi_locations_err(errors);
  // }
}

impl<'a, 'b> ExprByIndex for Surface<'a, 'b>
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.grammar.expr_by_index(index)
  }
}

impl<'a, 'b> Visitor<IType> for Surface<'a, 'b>
{
  fn visit_expr(&mut self, this: usize) -> IType {
    let mut this_ty = self.type_of(this);
    if this_ty == Infer {
      this_ty = walk_expr(self, this);
      let reduced_ty = TypeRewriting::reduce(&self.grammar, this_ty);
      self.type_expr(this, reduced_ty)
    }
    else {
      this_ty
    }
  }

  // Axioms

  fn visit_str_literal(&mut self, _this: usize, _lit: String) -> IType {
    IType::Invisible
  }

  fn visit_syntactic_predicate(&mut self, _this: usize, _child: usize) -> IType {
    IType::Invisible
  }

  fn visit_type_ascription(&mut self, _this: usize, _child: usize, ty: IType) -> IType {
    ty
  }

  fn visit_atom(&mut self, _this: usize) -> IType {
    IType::Regular(Type::Atom)
  }

  fn visit_semantic_action(&mut self, this: usize, _child: usize, action: Ident) -> IType {
    self.grammar.action_type(this, action)
  }

  // Inductive rules

  fn visit_non_terminal_symbol(&mut self, _this: usize, rule: Ident) -> IType {
    self.visit_rule(rule)
  }

  fn visit_repeat(&mut self, _this: usize, child: usize) -> IType {
    self.visit_expr(child);
    IType::Regular(Type::List(child))
  }

  fn visit_optional(&mut self, _this: usize, child: usize) -> IType {
    self.visit_expr(child);
    IType::Regular(Type::Optional(child))
  }

  fn visit_sequence(&mut self, _this: usize, children: Vec<usize>) -> IType {
    walk_exprs(self, children.clone());
    IType::Regular(Type::Tuple(children))
  }

  fn visit_choice(&mut self, _this: usize, children: Vec<usize>) -> IType {
    walk_exprs(self, children)[0].clone()
  }
}
