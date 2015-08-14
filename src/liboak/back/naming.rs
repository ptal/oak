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

use middle::typing::ast::*;

#[derive(Clone, Copy, Debug)]
pub struct GenFunNames
{
  pub recognizer: Ident,
  pub parser: Ident
}

pub struct NameFactory<'cx>
{
  cx: &'cx ExtCtxt<'cx>,
  unique_id: u32
}

impl<'cx> NameFactory<'cx>
{
  pub fn new(cx: &'cx ExtCtxt) -> NameFactory<'cx> {
    NameFactory {
      cx: cx,
      unique_id: 0
    }
  }

  pub fn expression_name(&mut self, expr_desc: &str, current_rule: Ident) -> GenFunNames {
    let uid = self.gen_uid();
    self.from_base_name(
      format!("{}_in_rule_{}_{}",
        expr_desc,
        ident_to_lowercase(current_rule),
        uid
      ))
  }

  pub fn names_of_rule(&mut self, rule_name: Ident) -> GenFunNames {
    self.from_base_name(ident_to_lowercase(rule_name))
  }

  fn gen_uid(&mut self) -> u32 {
    self.unique_id += 1;
    self.unique_id - 1
  }

  fn from_base_name(&self, base_name: String) -> GenFunNames {
    GenFunNames {
      recognizer: self.ident_of("recognize", &base_name),
      parser: self.ident_of("parse", &base_name)
    }
  }

  fn ident_of(&self, prefix: &str, base_name: &String) -> Ident {
    self.cx.ident_of(format!("{}_{}", prefix, base_name).as_str())
  }
}
