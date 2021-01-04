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

use proc_macro2::Ident;
use quote::format_ident;

pub fn parser_name(rule_name: Ident) -> Ident {
  format_ident!("parse_{}", rule_name)
}

pub fn recognizer_name(rule_name: Ident) -> Ident {
  format_ident!("recognize_{}", rule_name)
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

  pub fn next_mark_name(&mut self) -> Ident {
    self.mark_uid += 1;
    format_ident!("mark{}", self.mark_uid)
  }

  pub fn next_branch_failed_name(&mut self) -> Ident {
    self.branch_failed_uid += 1;
    format_ident!("branch_failed_{}", self.branch_failed_uid)
  }

  pub fn next_closure_name(&mut self) -> Ident {
    self.closure_uid += 1;
    format_ident!("success_continuation_{}", self.closure_uid)
  }

  pub fn next_counter_name(&mut self) -> Ident {
    self.counter_uid += 1;
    format_ident!("counter{}", self.counter_uid)
  }

  pub fn fresh_vars(&mut self, cardinality: usize) -> Vec<Ident> {
    let prefix = self.next_var_prefix();
    (0..cardinality)
      .map(|i| format_ident!("{}{}", prefix, i))
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
