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
use std::collections::HashSet;

enum SetInclusion {
    Empty,
    Conjunction(Vec<SetInclusion>),
    Disjunction(Vec<SetInclusion>)
}

pub struct UnreachableRule<'a: 'c, 'b: 'a, 'c>
{
  grammar: &'c AGrammar<'a, 'b>,
  in_choice: bool,
  hash_set_vector: Vec<HashSet<String>>,
  current_set: HashSet<String>
}

impl <'a, 'b, 'c> UnreachableRule<'a, 'b, 'c>
{
  pub fn analyse(grammar: AGrammar<'a, 'b>) -> Partial<AGrammar<'a, 'b>> {
    UnreachableRule::check_unreachable_rule(&grammar);
    Partial::Value(grammar)
  }

  fn check_unreachable_rule(grammar: &'c AGrammar<'a, 'b>){
    let mut analyser = UnreachableRule{
      grammar: grammar,
      in_choice: false,
      hash_set_vector: vec![],
      current_set: HashSet::new()
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
    unit_visitor_impl!(atom);
    unit_visitor_impl!(sequence);
    unit_visitor_impl!(non_terminal);

    fn visit_str_literal(&mut self, _this: usize, lit: String){
        if self.in_choice {
            println!("{}",lit.as_str());
            self.current_set.insert(lit);
        }
    }

    fn visit_choice(&mut self, this: usize, children: Vec<usize>){
        self.in_choice = true;
        for child in children {
            self.hash_set_vector.push(self.current_set.iter().cloned().collect());
            self.current_set.clear();
            self.visit_expr(child)
        }
        let len = self.hash_set_vector.len();
        for i in 0..len {
            for j in i+1..len {
                println!("{},{}",i,j);
                if self.hash_set_vector[i].is_superset(&self.hash_set_vector[j]) {
                    self.grammar.span_warn(
                        self.grammar[this].span(),
                        format!("Test")
                    )
                }
            }
        }
        self.hash_set_vector.clear();
        self.in_choice = false
    }
}
