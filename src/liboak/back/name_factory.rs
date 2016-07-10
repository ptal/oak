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

use identifier::*;
use rust::ExtCtxt;

pub fn parser_name(cx: &ExtCtxt, rule_name: Ident) -> Ident {
  string_to_ident(cx, format!("parse_{}", ident_to_string(rule_name)))
}

pub fn recognizer_name(cx: &ExtCtxt, rule_name: Ident) -> Ident {
  string_to_ident(cx, format!("recognize_{}", ident_to_string(rule_name)))
}

pub struct NameFactory
{
  prefix_uid: usize,
  mark_uid: usize,
  branch_failed_uid: usize,
  counter_uid: usize,
  closure_uid: usize
}

impl NameFactory
{
  pub fn new() -> NameFactory {
    NameFactory {
      prefix_uid: 1,
      mark_uid: 0,
      branch_failed_uid: 0,
      counter_uid: 0,
      closure_uid: 0
    }
  }

  pub fn next_mark_name(&mut self, cx: &ExtCtxt) -> Ident {
    self.mark_uid += 1;
    string_to_ident(cx, format!("mark{}", self.mark_uid))
  }

  pub fn next_branch_failed_name(&mut self, cx: &ExtCtxt) -> Ident {
    self.branch_failed_uid += 1;
    string_to_ident(cx, format!("branch_failed_{}", self.branch_failed_uid))
  }

  pub fn next_closure_name(&mut self, cx: &ExtCtxt) -> Ident {
    self.closure_uid += 1;
    string_to_ident(cx, format!("success_continuation_{}", self.closure_uid))
  }

  pub fn next_counter_name(&mut self, cx: &ExtCtxt) -> Ident {
    self.counter_uid += 1;
    string_to_ident(cx, format!("counter{}", self.counter_uid))
  }

  pub fn fresh_vars(&mut self, cx: &ExtCtxt, cardinality: usize) -> Vec<Ident> {
    let prefix = self.next_var_prefix();
    (0..cardinality)
      .map(|i| string_to_ident(cx, format!("{}{}", prefix, i)))
      .collect()
  }

  pub fn next_var_prefix(&mut self) -> String {
    const LETTERS: &'static [u8] = b"abcdefghijklmnopqrstuvwxyz";
    let mut prefix = String::new();
    let mut num_letter = self.prefix_uid;
    while num_letter > 0 {
      let x = num_letter % LETTERS.len();
      num_letter /= LETTERS.len();
      prefix.push(LETTERS[x] as char);
    }
    self.prefix_uid += 1;
    prefix
  }
}
