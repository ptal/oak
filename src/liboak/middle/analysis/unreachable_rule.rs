// Copyright 2018 Chao Lin & William Sergeant (Sorbonne University)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![macro_use]
use middle::analysis::ast::*;

pub struct UnreachableRule<'a: 'c, 'b: 'a, 'c>
{
  grammar: &'c AGrammar<'a, 'b>
}

impl <'a, 'b, 'c> UnreachableRule<'a, 'b, 'c>
{
  pub fn analyse(grammar: AGrammar<'a, 'b>) -> Partial<AGrammar<'a, 'b>> {
    UnreachableRule::check_unreachable_rule(&grammar);
    Partial::Value(grammar)
  }

  fn check_unreachable_rule(grammar: &'c AGrammar<'a, 'b>){
    let mut analyser = UnreachableRule{
      grammar: grammar
    };

    for rule in &grammar.rules {
      analyser.visit_expr(rule.expr_idx)
    }
  }
}

impl<'a, 'b, 'c> ExprByIndex for UnreachableRule<'a, 'b, 'c>
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.grammar.expr_by_index(index).clone()
  }
}

impl<'a, 'b, 'c> Visitor<()> for UnreachableRule<'a, 'b, 'c>
{
    unit_visitor_impl!(str_literal);
    unit_visitor_impl!(atom);
    unit_visitor_impl!(sequence);
    unit_visitor_impl!(non_terminal);

    fn visit_choice(&mut self, _: usize, children: Vec<usize>){
        for child in children {
            self.visit_expr(child);
        }
    }
}
