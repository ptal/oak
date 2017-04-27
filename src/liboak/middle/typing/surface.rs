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
use middle::typing::typing_printer::*;

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

  pub fn surface(&mut self) {
    for rule in self.grammar.rules.clone() {
      self.visit_rule(rule.ident());
    }
    if self.grammar.attributes.print_typing.debug() {
      println!("After applying Surface\n.");
      print_debug(&self.grammar);
      println!("");
    }
  }

  fn visit_rule(&mut self, rule: Ident) -> IType {
    let expr_idx = self.grammar.expr_index_of_rule(rule);
    let rule_ty = self.grammar.type_of(expr_idx);
    if rule_ty == Infer {
      if self.is_rec(rule) {
        self.infer_rec_type(rule)
      }
      else {
        self.recursion_path.push(rule);
        let ty = self.visit_expr(expr_idx);
        self.recursion_path.pop();
        let reduced_ty = TypeRewriting::reduce_rec_entry_point(rule, ty);
        self.type_expr(expr_idx, reduced_ty)
      }
    }
    else {
      rule_ty
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
    let mut rec_shorter_path = vec![entry_rule];
    rec_shorter_path.extend(
      rec_path.into_iter()
        .rev()
        .take_while(|r| *r != entry_rule));
    IType::rec(RecKind::Unit, rec_shorter_path)
  }

  fn type_mismatch_branches(&self, rec_set: RecSet, sum_expr: usize, branches: Vec<usize>, tys: Vec<IType>) {
    let mut errors = vec![(
      self.grammar[sum_expr].span(),
      format!("Type mismatch between branches of the choice operator.")
    )];
    for i in 0..branches.len() {
      errors.push((
        self.grammar[branches[i]].span(),
        format!("{}", tys[i].display(&self.grammar))));
    }
    if !rec_set.is_empty() {
      let entry_point = self.grammar.find_rule_by_ident(rec_set.entry_point());
      errors.push((
        entry_point.span(),
        format!("Types annotated with `*` have been reduced to (^) because they are involved \
          in one of the following rule cycle (generating recursive types):\n{}",
          rec_set.display())));
    }
    self.grammar.multi_locations_err(errors);
  }
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

  fn visit_spanned_expr(&mut self, _this: usize, child: usize) -> IType {
    self.visit_expr(child);
    IType::Regular(Type::Tuple(vec![self.grammar.span_ty_idx(), child]))
  }

  fn visit_sequence(&mut self, _this: usize, children: Vec<usize>) -> IType {
    walk_exprs(self, children.clone());
    IType::Regular(Type::Tuple(children))
  }

  fn visit_choice(&mut self, this: usize, children: Vec<usize>) -> IType {
    let tys = walk_exprs(self, children.clone());
    match TypeRewriting::reduce_sum(&self.grammar, tys.clone()) {
      Ok(principal_type) => principal_type,
      Err(rec_set) => {
        self.type_mismatch_branches(rec_set, this, children, tys);
        IType::Invisible
      }
    }
  }
}
