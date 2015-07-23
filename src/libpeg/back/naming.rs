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

#[derive(Clone)]
pub struct GenFunNames
{
  pub recognizer: Ident,
  pub parser: Ident
}

impl GenFunNames
{
  fn from_base_name(base_name: String) -> GenFunNames
  {
    GenFunNames {
      recognizer: GenFunNames::gen_ident("recognize", &base_name),
      parser: GenFunNames::gen_ident("parse", &base_name)
    }
  }

  fn gen_ident(prefix: &str, base_name: &String) -> Ident
  {
    rust::gensym_ident(format!("{}_{}", prefix, base_name).as_str())
  }
}

pub struct NameFactory
{
  rule_name_memoization: HashMap<Ident, GenFunNames>,
  unique_id: u32
}

impl NameFactory
{
  pub fn new() -> NameFactory
  {
    NameFactory {
      rule_name_memoization: HashMap::new(),
      unique_id: 0
    }
  }

  pub fn expression_name(&mut self, expr: &str, current_rule: &Ident) -> GenFunNames
  {
    GenFunNames::from_base_name(
      format!("{}_in_rule_{}_{}",
        expr,
        ident_to_lowercase(current_rule),
        self.gen_uid()
      ))
  }

  pub fn rule_name(&mut self, rule: &Ident) -> GenFunNames
  {
    match self.rule_name_memoization.get(rule).cloned() {
      Some(fun_name) => fun_name,
      None => {
        let fun_name = GenFunNames::from_base_name(ident_to_lowercase(rule));
        self.rule_name_memoization.insert(rule.clone(), fun_name.clone());
        fun_name
      }
    }
  }

  fn gen_uid(&mut self) -> u32
  {
    self.unique_id += 1;
    self.unique_id - 1
  }
}
