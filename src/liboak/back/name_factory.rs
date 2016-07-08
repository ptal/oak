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

pub type Namespace = Vec<Ident>;

pub struct NameFactory
{
  namespaces: Vec<Namespace>,
  num_vars_in_scope: usize,
  mark_uid: usize,
  closure_uid: usize
}

impl NameFactory
{
  pub fn new() -> NameFactory {
    NameFactory {
      namespaces: vec![vec![]],
      num_vars_in_scope: 0,
      mark_uid: 0,
      closure_uid: 0
    }
  }

  pub fn next_mark_name(&mut self, cx: &ExtCtxt) -> Ident {
    self.mark_uid += 1;
    string_to_ident(cx, format!("mark_{}", self.mark_uid))
  }

  pub fn next_closure_name(&mut self, cx: &ExtCtxt) -> Ident {
    self.closure_uid += 1;
    string_to_ident(cx, format!("success_continuation_{}", self.closure_uid))
  }

  pub fn vars_in_scope(&self) -> Vec<Ident> {
    self.current_namespace()[0..self.num_vars_in_scope]
      .iter().cloned().collect()
  }

  pub fn next_unbounded_var(&mut self) -> Ident {
    assert!(self.num_vars_in_scope > 0, "Request a variable name in an empty namespace.");
    self.num_vars_in_scope -= 1;
    self.current_namespace()[self.num_vars_in_scope]
  }

  pub fn open_namespace(&mut self, cx: &ExtCtxt, cardinality: usize) -> Namespace {
    let namespace: Namespace = (0..cardinality)
      .map(|i| string_to_ident(cx, format!("v_{}", i)))
      .collect();
    self.namespaces.push(namespace.clone());
    self.num_vars_in_scope = namespace.len();
    namespace
  }

  pub fn close_namespace(&mut self) {
    assert!(self.namespaces.len() > 1,
      "close_namespace: There is no namespace opened.");
    assert!(self.num_vars_in_scope == 0,
      "Try to close a namespace that has not been fully consumed.");
    self.namespaces.pop();
  }

  fn current_namespace<'a>(&'a self) -> &'a Namespace {
    let last = self.namespaces.len() - 1;
    &self.namespaces[last]
  }

  pub fn save_namespace(&self) -> usize {
    self.num_vars_in_scope
  }

  pub fn restore_namespace(&mut self, num_vars_in_scope: usize) {
    assert!(self.current_namespace().len() >= num_vars_in_scope,
      "Mismatch between the number of variable in scope and the current namespace.");
    self.num_vars_in_scope = num_vars_in_scope;
  }
}
