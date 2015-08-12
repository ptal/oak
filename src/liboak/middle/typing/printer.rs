// Copyright 2015 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! It prints the rules annotated with type and context.

use middle::typing::ast::*;

pub fn print_annotated_rules(grammar: &Grammar)
{
  Printer::print(grammar);
}

struct Printer;

impl Printer
{
  pub fn print(grammar: &Grammar)
  {
    let mut printer = Printer;
    printer.visit_grammar(grammar)
  }

  fn visit_grammar(&mut self, grammar: &Grammar)
  {
    println!("Grammar: {}", grammar.name);
    self.visit_rules(&grammar.rules);
  }

  fn visit_rules(&mut self, rules: &HashMap<Ident, Rule>)
  {
    for rule in rules.values() {
      self.visit_rule(rule);
    }
  }

  fn visit_rule(&mut self, rule: &Rule)
  {
    println!("{}:({:?}, {:?})", rule.name.node, rule.def.ty, rule.def.context);
  }
}
