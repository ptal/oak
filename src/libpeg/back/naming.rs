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

use rust;
use middle::ast::*;

pub struct NameFactory
{
  rule_id_to_recognizer_id: HashMap<Ident, Ident>,
  rule_id_to_parser_id: HashMap<Ident, Ident>,
  unique_id: u32,
}

impl NameFactory
{
  pub fn new() -> NameFactory
  {
    NameFactory {
      rule_id_to_recognizer_id: HashMap::new(),
      rule_id_to_parser_id: HashMap::new(),
      unique_id: 0
    }
  }

  pub fn expression_recognizer_name(&mut self, expr: &str, current_rule: &Ident) -> Ident
  {
    self.expression_name("recognize", expr, current_rule)
  }

  pub fn expression_parser_name(&mut self, expr: &str, current_rule: &Ident) -> Ident
  {
    self.expression_name("parse", expr, current_rule)
  }

  pub fn rule_recognizer_name(&mut self, rule: &Ident) -> Ident
  {
    NameFactory::rule_name("recognize", rule, &mut self.rule_id_to_recognizer_id)
  }

  pub fn rule_parser_name(&mut self, rule: &Ident) -> Ident
  {
    NameFactory::rule_name("parse", rule, &mut self.rule_id_to_parser_id)
  }

  fn gen_uid(&mut self) -> u32
  {
    self.unique_id += 1;
    self.unique_id - 1
  }

  fn expression_name(&mut self, action: &str, expr: &str, current_rule: &Ident) -> Ident
  {
    rust::gensym_ident(format!(
      "{}_{}_in_rule_{}_{}", action, expr,
        ident_to_lowercase(current_rule),
        self.gen_uid()).as_str())
  }

  fn rule_name(action: &str, rule: &Ident, memoization: &mut HashMap<Ident, Ident>) -> Ident
  {
    match memoization.get(rule).cloned() {
      Some(id) => id,
      None => {
        let fun_name = format!("{}_{}", action, ident_to_lowercase(rule));
        let fun_id = rust::gensym_ident(fun_name.as_str());
        memoization.insert(rule.clone(), fun_id.clone());
        fun_id
      }
    }
  }
}
